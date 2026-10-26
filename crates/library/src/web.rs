//! The store in the browser: IndexedDB, in the layout `templates/library-page/library.js`
//! uses, so the page and the game read and write the same files.
//!
//! The database is named after the spec's `id`, version 1, with three object stores:
//! - `files`: the bytes (`Uint8Array`), keyed `SECTION/NAME`;
//! - `entries`: each file's [`Entry`] as JSON text, keyed `SECTION/NAME`;
//! - `state`: each section's [`State`] as JSON text, keyed `SECTION`, and the encryption key
//!   under `#key`.
//!
//! With `encrypt` on (the default), each kept file is AES-GCM: 12 bytes of IV, then the
//! ciphertext and its 16-byte tag. The key is a WebCrypto key made by whichever side opens the
//! database first, marked non-extractable: scripts can use it but never read it, so a copy of the
//! database's files cannot be read without this browser. It hides the files from a casual copy;
//! it is not protection against code running in the page.
//!
//! A section added later needs no new object store, so the version never changes. A write
//! returns when its transaction has committed: a request's `success` comes first, and a page
//! that reloads between the two loses the write.

use crate::store::{check_name, Card, Entry, LocalFuture, State, Store};
use std::io;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AesGcmParams, AesKeyGenParams, CryptoKey, IdbDatabase, IdbKeyRange, IdbRequest, IdbTransaction,
    IdbTransactionMode, SubtleCrypto,
};

const FILES: &str = "files";
const ENTRIES: &str = "entries";
const STATE: &str = "state";
const KEY: &str = "#key";

fn js(e: JsValue) -> io::Error {
    io::Error::other(e.as_string().unwrap_or_else(|| format!("{e:?}")))
}

/// A promise that settles when `on_ok` or `on_err` fires on the target.
fn settle(set: impl FnOnce(&js_sys::Function, &js_sys::Function)) -> JsFuture {
    let mut set = Some(set);
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        if let Some(set) = set.take() {
            set(&resolve, &reject);
        }
    });
    JsFuture::from(promise)
}

async fn wait(req: &IdbRequest) -> io::Result<JsValue> {
    let r = req.clone();
    settle(|resolve, reject| {
        let (resolve, reject) = (resolve.clone(), reject.clone());
        let ok = Closure::once_into_js(move |_: web_sys::Event| {
            let _ = resolve.call1(&JsValue::NULL, &r.result().unwrap_or(JsValue::UNDEFINED));
        });
        let fail = Closure::once_into_js(move |_: web_sys::Event| {
            let _ = reject.call1(&JsValue::NULL, &JsValue::from_str("IndexedDB request failed"));
        });
        req.set_onsuccess(Some(ok.unchecked_ref()));
        req.set_onerror(Some(fail.unchecked_ref()));
    })
    .await
    .map_err(js)
}

/// Ends when the transaction has committed.
fn committed(tx: &IdbTransaction) -> JsFuture {
    settle(|resolve, reject| {
        let (resolve, reject2) = (resolve.clone(), reject.clone());
        let ok = Closure::once_into_js(move |_: web_sys::Event| {
            let _ = resolve.call0(&JsValue::NULL);
        });
        let reject = reject.clone();
        let fail = Closure::once_into_js(move |_: web_sys::Event| {
            let _ = reject.call1(&JsValue::NULL, &JsValue::from_str("IndexedDB transaction failed"));
        });
        let abort = Closure::once_into_js(move |_: web_sys::Event| {
            let _ = reject2.call1(&JsValue::NULL, &JsValue::from_str("IndexedDB transaction aborted"));
        });
        tx.set_oncomplete(Some(ok.unchecked_ref()));
        tx.set_onerror(Some(fail.unchecked_ref()));
        tx.set_onabort(Some(abort.unchecked_ref()));
    })
}

fn key(section: &str, name: &str) -> JsValue {
    JsValue::from_str(&format!("{section}/{name}"))
}

/// Every key of a section: `SECTION/` up to the highest character.
fn range(section: &str) -> io::Result<IdbKeyRange> {
    IdbKeyRange::bound(&JsValue::from_str(&format!("{section}/")), &JsValue::from_str(&format!("{section}/\u{ffff}")))
        .map_err(js)
}

pub struct IndexedDb {
    db: IdbDatabase,
    key: Option<CryptoKey>,
}

fn subtle() -> io::Result<SubtleCrypto> {
    Ok(web_sys::window().ok_or_else(|| io::Error::other("no window"))?.crypto().map_err(js)?.subtle())
}

impl IndexedDb {
    /// Open (or make) the store for `spec`: the database named by its id, encrypted when its
    /// `encrypt` is on.
    pub async fn open(spec: &crate::Spec) -> io::Result<IndexedDb> {
        let db = Self::open_db(&spec.id).await?;
        let mut store = IndexedDb { db, key: None };
        if spec.encrypt() {
            store.key = Some(store.the_key().await?);
        }
        Ok(store)
    }

    /// The key kept in the database, made on first use.
    async fn the_key(&self) -> io::Result<CryptoKey> {
        let tx = self.tx(&[STATE], IdbTransactionMode::Readonly)?;
        let v = wait(&tx.object_store(STATE).map_err(js)?.get(&JsValue::from_str(KEY)).map_err(js)?).await?;
        if !v.is_undefined() {
            return Ok(v.unchecked_into());
        }
        let usages = js_sys::Array::of2(&"encrypt".into(), &"decrypt".into());
        let made = subtle()?
            .generate_key_with_object(&AesKeyGenParams::new("AES-GCM", 256), false, &usages)
            .map_err(js)?;
        let key: CryptoKey = JsFuture::from(made).await.map_err(js)?.unchecked_into();
        let tx = self.tx(&[STATE], IdbTransactionMode::Readwrite)?;
        let done = committed(&tx);
        tx.object_store(STATE).map_err(js)?.put_with_key(&key, &JsValue::from_str(KEY)).map_err(js)?;
        done.await.map_err(js)?;
        Ok(key)
    }

    async fn seal(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        let Some(key) = &self.key else { return Ok(bytes.to_vec()) };
        let mut iv = [0u8; 12];
        web_sys::window()
            .ok_or_else(|| io::Error::other("no window"))?
            .crypto()
            .map_err(js)?
            .get_random_values_with_u8_array(&mut iv)
            .map_err(js)?;
        let params = AesGcmParams::new_with_u8_slice("AES-GCM", &mut iv.clone());
        let p = subtle()?.encrypt_with_object_and_u8_array(&params, key, bytes).map_err(js)?;
        let sealed = js_sys::Uint8Array::new(&JsFuture::from(p).await.map_err(js)?).to_vec();
        Ok([&iv[..], &sealed].concat())
    }

    async fn unseal(&self, bytes: Vec<u8>) -> io::Result<Vec<u8>> {
        let Some(key) = &self.key else { return Ok(bytes) };
        if bytes.len() < 28 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "a kept file is too short to be encrypted"));
        }
        let mut iv = bytes[..12].to_vec();
        let params = AesGcmParams::new_with_u8_slice("AES-GCM", &mut iv);
        let p = subtle()?.decrypt_with_object_and_u8_array(&params, key, &bytes[12..]).map_err(js)?;
        let plain = JsFuture::from(p)
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "a kept file does not decrypt with this browser's key"))?;
        Ok(js_sys::Uint8Array::new(&plain).to_vec())
    }

    async fn open_db(id: &str) -> io::Result<IdbDatabase> {
        let factory = web_sys::window()
            .ok_or_else(|| io::Error::other("no window"))?
            .indexed_db()
            .map_err(js)?
            .ok_or_else(|| io::Error::other("this browser keeps no IndexedDB"))?;
        let req = factory.open_with_u32(id, 1).map_err(js)?;
        let r = req.clone();
        let upgrade = Closure::once_into_js(move |_: web_sys::Event| {
            if let Ok(db) = r.result().and_then(|d| d.dyn_into::<IdbDatabase>()) {
                for store in [FILES, ENTRIES, STATE] {
                    if !db.object_store_names().contains(store) {
                        let _ = db.create_object_store(store);
                    }
                }
            }
        });
        req.set_onupgradeneeded(Some(upgrade.unchecked_ref()));
        wait(&req).await?.dyn_into::<IdbDatabase>().map_err(js)
    }

    fn tx(&self, stores: &[&str], mode: IdbTransactionMode) -> io::Result<IdbTransaction> {
        let names = stores.iter().map(|s| JsValue::from_str(s)).collect::<js_sys::Array>();
        self.db.transaction_with_str_sequence_and_mode(&names, mode).map_err(js)
    }

    async fn list_now(&self, section: &str) -> io::Result<Vec<Entry>> {
        let tx = self.tx(&[ENTRIES], IdbTransactionMode::Readonly)?;
        let store = tx.object_store(ENTRIES).map_err(js)?;
        let values = wait(&store.get_all_with_key(&range(section)?.into()).map_err(js)?).await?;
        let values: js_sys::Array = values.dyn_into().map_err(js)?;
        Ok(values
            .iter()
            .filter_map(|v| v.as_string())
            .filter_map(|t| serde_json::from_str(&t).ok())
            .collect())
    }

    async fn read_now(&self, section: &str, name: &str) -> io::Result<Vec<u8>> {
        let tx = self.tx(&[FILES], IdbTransactionMode::Readonly)?;
        let v = wait(&tx.object_store(FILES).map_err(js)?.get(&key(section, name)).map_err(js)?).await?;
        if v.is_undefined() {
            return Err(io::Error::new(io::ErrorKind::NotFound, format!("{section}/{name}: not kept")));
        }
        self.unseal(js_sys::Uint8Array::new(&v).to_vec()).await
    }

    async fn write_now(&self, section: &str, name: &str, bytes: &[u8], card: Option<Card>) -> io::Result<()> {
        check_name(name)?;
        let entry = Entry { name: name.into(), size: bytes.len() as u64, modified: js_sys::Date::now(), card };
        let text = serde_json::to_string(&entry).map_err(io::Error::other)?;
        let sealed = self.seal(bytes).await?;
        let tx = self.tx(&[FILES, ENTRIES], IdbTransactionMode::Readwrite)?;
        let done = committed(&tx);
        let k = key(section, name);
        tx.object_store(FILES).map_err(js)?.put_with_key(&js_sys::Uint8Array::from(&sealed[..]), &k).map_err(js)?;
        tx.object_store(ENTRIES).map_err(js)?.put_with_key(&JsValue::from_str(&text), &k).map_err(js)?;
        done.await.map(|_| ()).map_err(js)
    }

    async fn delete_now(&self, section: &str, name: &str) -> io::Result<()> {
        let tx = self.tx(&[FILES, ENTRIES], IdbTransactionMode::Readwrite)?;
        let done = committed(&tx);
        let k = key(section, name);
        tx.object_store(FILES).map_err(js)?.delete(&k).map_err(js)?;
        tx.object_store(ENTRIES).map_err(js)?.delete(&k).map_err(js)?;
        done.await.map(|_| ()).map_err(js)
    }

    async fn state_now(&self, section: &str) -> io::Result<State> {
        let tx = self.tx(&[STATE], IdbTransactionMode::Readonly)?;
        let v = wait(&tx.object_store(STATE).map_err(js)?.get(&JsValue::from_str(section)).map_err(js)?).await?;
        Ok(v.as_string().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default())
    }

    async fn set_state_now(&self, section: &str, state: &State) -> io::Result<()> {
        let text = serde_json::to_string(state).map_err(io::Error::other)?;
        let tx = self.tx(&[STATE], IdbTransactionMode::Readwrite)?;
        let done = committed(&tx);
        tx.object_store(STATE)
            .map_err(js)?
            .put_with_key(&JsValue::from_str(&text), &JsValue::from_str(section))
            .map_err(js)?;
        done.await.map(|_| ()).map_err(js)
    }
}

impl Store for IndexedDb {
    fn list<'a>(&'a self, section: &'a str) -> LocalFuture<'a, io::Result<Vec<Entry>>> {
        Box::pin(self.list_now(section))
    }
    fn read<'a>(&'a self, section: &'a str, name: &'a str) -> LocalFuture<'a, io::Result<Vec<u8>>> {
        Box::pin(self.read_now(section, name))
    }
    fn write<'a>(
        &'a self,
        section: &'a str,
        name: &'a str,
        bytes: &'a [u8],
        card: Option<Card>,
    ) -> LocalFuture<'a, io::Result<()>> {
        Box::pin(self.write_now(section, name, bytes, card))
    }
    fn delete<'a>(&'a self, section: &'a str, name: &'a str) -> LocalFuture<'a, io::Result<()>> {
        Box::pin(self.delete_now(section, name))
    }
    fn state<'a>(&'a self, section: &'a str) -> LocalFuture<'a, io::Result<State>> {
        Box::pin(self.state_now(section))
    }
    fn set_state<'a>(&'a self, section: &'a str, state: &'a State) -> LocalFuture<'a, io::Result<()>> {
        Box::pin(self.set_state_now(section, state))
    }
}

/// Let the page ask the game for cards and checks: sets `window.gameframeLibrary.describe` and
/// `.accept`, which `library.js` calls when the player adds a file. Call it once the game's
/// WebAssembly has started; files added before that keep their bare names until added again.
pub fn register_hooks<H: crate::Hooks + 'static>(spec: crate::Spec, hooks: H) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let api = js_sys::Reflect::get(&window, &"gameframeLibrary".into())?;
    let api = if api.is_object() {
        api
    } else {
        let o: JsValue = js_sys::Object::new().into();
        js_sys::Reflect::set(&window, &"gameframeLibrary".into(), &o)?;
        o
    };
    let shared = std::rc::Rc::new((spec, hooks));
    let s = shared.clone();
    let describe = Closure::<dyn Fn(String, String, js_sys::Uint8Array) -> JsValue>::new(
        move |section: String, name: String, bytes: js_sys::Uint8Array| {
            let (spec, hooks) = &*s;
            spec.section(&section)
                .and_then(|sec| hooks.describe(sec, &name, &bytes.to_vec()))
                .and_then(|card| serde_json::to_string(&card).ok())
                .map_or(JsValue::NULL, |t| JsValue::from_str(&t))
        },
    );
    let s = shared;
    let accept = Closure::<dyn Fn(String, String, js_sys::Uint8Array) -> JsValue>::new(
        move |section: String, name: String, bytes: js_sys::Uint8Array| {
            let (spec, hooks) = &*s;
            match spec.section(&section).map(|sec| hooks.accept(sec, &name, &bytes.to_vec())) {
                Some(Err(why)) => JsValue::from_str(&why),
                _ => JsValue::NULL,
            }
        },
    );
    js_sys::Reflect::set(&api, &"describe".into(), describe.as_ref())?;
    js_sys::Reflect::set(&api, &"accept".into(), accept.as_ref())?;
    describe.forget();
    accept.forget();
    Ok(())
}
