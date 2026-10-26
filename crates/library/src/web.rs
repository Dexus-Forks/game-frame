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
//!
//! A file of at least [`BIG`] bytes, such as a CD image, does not go into `files`. It is
//! streamed into the origin private file system (OPFS) as `gameframe-library-<id>/SECTION/NAME`,
//! a slice at a time, and kept plain: it is read back in slices ([`IndexedDb::file`]), which
//! AES-GCM over the whole file would not allow. Its [`Entry`] still goes into `entries`, so
//! listing is the same for both. Reading, getting and deleting look in `files` first and then
//! in OPFS. The HTML page lists a big file but cannot download it.

use crate::store::{check_name, Card, Entry, LocalFuture, State, Store};
use std::io;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AesGcmParams, AesKeyGenParams, Blob, CryptoKey, File, FileSystemDirectoryHandle, FileSystemFileHandle,
    FileSystemGetDirectoryOptions, FileSystemGetFileOptions, FileSystemWritableFileStream, IdbDatabase,
    IdbKeyRange, IdbRequest, IdbTransaction, IdbTransactionMode, SubtleCrypto,
};

const FILES: &str = "files";
const ENTRIES: &str = "entries";
const STATE: &str = "state";
const KEY: &str = "#key";

/// The size from which a file is kept in OPFS, plain, instead of in IndexedDB: 64 MiB.
pub const BIG: u64 = 64 << 20;
/// How much of a big file is copied into OPFS at a time: 8 MiB.
const SLICE: u64 = 8 << 20;
/// How much of a big file the game's hooks see: its first 1 MiB.
pub(crate) const HEAD: u64 = 1 << 20;

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
    /// The OPFS folder for big files: `gameframe-library-<id>`.
    folder: String,
}

/// Whether a rejection is the browser's `NotFoundError`.
fn not_found(e: &JsValue) -> bool {
    js_sys::Reflect::get(e, &"name".into()).ok().and_then(|n| n.as_string()).as_deref() == Some("NotFoundError")
}

/// The root of the origin private file system, where the browser has one.
async fn opfs_root() -> Option<FileSystemDirectoryHandle> {
    let navigator = web_sys::window()?.navigator();
    let storage = js_sys::Reflect::get(&navigator, &"storage".into()).ok().filter(JsValue::is_object)?;
    js_sys::Reflect::get(&storage, &"getDirectory".into()).ok().filter(JsValue::is_function)?;
    let root = JsFuture::from(navigator.storage().get_directory()).await.ok()?;
    root.dyn_into().ok()
}

/// Bytes `start..end` of a file or blob.
pub(crate) async fn blob_bytes(blob: &Blob, start: u64, end: u64) -> io::Result<Vec<u8>> {
    let part = blob.slice_with_f64_and_f64(start as f64, end as f64).map_err(js)?;
    let buf = JsFuture::from(part.array_buffer()).await.map_err(js)?;
    Ok(js_sys::Uint8Array::new(&buf).to_vec())
}

/// Bytes kept in memory as a [`File`] named `name`.
pub(crate) fn file_of(name: &str, bytes: &[u8]) -> io::Result<File> {
    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(bytes));
    File::new_with_u8_array_sequence(&parts, name).map_err(js)
}

fn subtle() -> io::Result<SubtleCrypto> {
    Ok(web_sys::window().ok_or_else(|| io::Error::other("no window"))?.crypto().map_err(js)?.subtle())
}

impl IndexedDb {
    /// Open (or make) the store for `spec`: the database named by its id, encrypted when its
    /// `encrypt` is on.
    pub async fn open(spec: &crate::Spec) -> io::Result<IndexedDb> {
        let db = Self::open_db(&spec.id).await?;
        let mut store = IndexedDb { db, key: None, folder: format!("gameframe-library-{}", spec.id) };
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

    /// The sealed bytes in `files`, if the file is kept there.
    async fn kept_bytes(&self, section: &str, name: &str) -> io::Result<Option<Vec<u8>>> {
        check_name(name)?;
        let tx = self.tx(&[FILES], IdbTransactionMode::Readonly)?;
        let v = wait(&tx.object_store(FILES).map_err(js)?.get(&key(section, name)).map_err(js)?).await?;
        if v.is_undefined() {
            return Ok(None);
        }
        Ok(Some(self.unseal(js_sys::Uint8Array::new(&v).to_vec()).await?))
    }

    async fn read_now(&self, section: &str, name: &str) -> io::Result<Vec<u8>> {
        if let Some(bytes) = self.kept_bytes(section, name).await? {
            return Ok(bytes);
        }
        let file = self.big_file(section, name).await?;
        blob_bytes(&file, 0, file.size() as u64).await
    }

    async fn file_now(&self, section: &str, name: &str) -> io::Result<File> {
        match self.kept_bytes(section, name).await? {
            Some(bytes) => file_of(name, &bytes),
            None => self.big_file(section, name).await,
        }
    }

    /// Writes the entry, and the sealed bytes when `bytes` is given; without bytes, removes any
    /// copy in `files` (the file is in OPFS).
    async fn put_now(&self, section: &str, name: &str, entry: &Entry, bytes: Option<&[u8]>) -> io::Result<()> {
        let text = serde_json::to_string(entry).map_err(io::Error::other)?;
        let sealed = match bytes {
            Some(b) => Some(self.seal(b).await?),
            None => None,
        };
        let tx = self.tx(&[FILES, ENTRIES], IdbTransactionMode::Readwrite)?;
        let done = committed(&tx);
        let k = key(section, name);
        let files = tx.object_store(FILES).map_err(js)?;
        match sealed {
            Some(sealed) => files.put_with_key(&js_sys::Uint8Array::from(&sealed[..]), &k).map_err(js)?,
            None => files.delete(&k).map_err(js)?,
        };
        tx.object_store(ENTRIES).map_err(js)?.put_with_key(&JsValue::from_str(&text), &k).map_err(js)?;
        done.await.map(|_| ()).map_err(js)
    }

    async fn write_now(&self, section: &str, name: &str, bytes: &[u8], card: Option<Card>) -> io::Result<()> {
        check_name(name)?;
        let entry = Entry { name: name.into(), size: bytes.len() as u64, modified: js_sys::Date::now(), card };
        self.put_now(section, name, &entry, Some(bytes)).await?;
        self.remove_big(section, name).await
    }

    /// Keep a file the player chose: below [`BIG`] like [`Store::write`], from [`BIG`] up
    /// streamed into OPFS a slice at a time, so memory never holds more than one slice.
    pub async fn write_file(&self, section: &str, name: &str, file: &File, card: Option<Card>) -> io::Result<()> {
        check_name(name)?;
        let size = file.size() as u64;
        if size < BIG {
            let bytes = blob_bytes(file, 0, size).await?;
            return self.write_now(section, name, &bytes, card).await;
        }
        let dir = self.section_dir(section, true).await?.ok_or_else(|| io::Error::other("OPFS made no folder"))?;
        let opts = FileSystemGetFileOptions::new();
        opts.set_create(true);
        let handle: FileSystemFileHandle =
            JsFuture::from(dir.get_file_handle_with_options(name, &opts)).await.map_err(js)?.dyn_into().map_err(js)?;
        let stream: FileSystemWritableFileStream =
            JsFuture::from(handle.create_writable()).await.map_err(js)?.dyn_into().map_err(js)?;
        let mut at = 0;
        while at < size {
            let end = (at + SLICE).min(size);
            let part = file.slice_with_f64_and_f64(at as f64, end as f64).map_err(js)?;
            if let Err(e) = JsFuture::from(stream.write_with_blob(&part).map_err(js)?).await {
                // Dropping the half-written copy keeps the old file, if there was one.
                let _ = JsFuture::from(stream.abort()).await;
                return Err(js(e));
            }
            at = end;
        }
        JsFuture::from(stream.close()).await.map_err(js)?;
        let entry = Entry { name: name.into(), size, modified: js_sys::Date::now(), card };
        self.put_now(section, name, &entry, None).await
    }

    /// The kept file as a [`File`], to read in slices: a big file straight from OPFS, a small
    /// one decrypted into memory.
    pub async fn file(&self, section: &str, name: &str) -> io::Result<File> {
        self.file_now(section, name).await
    }

    async fn delete_now(&self, section: &str, name: &str) -> io::Result<()> {
        let tx = self.tx(&[FILES, ENTRIES], IdbTransactionMode::Readwrite)?;
        let done = committed(&tx);
        let k = key(section, name);
        tx.object_store(FILES).map_err(js)?.delete(&k).map_err(js)?;
        tx.object_store(ENTRIES).map_err(js)?.delete(&k).map_err(js)?;
        done.await.map(|_| ()).map_err(js)?;
        if check_name(name).is_ok() {
            self.remove_big(section, name).await?;
        }
        Ok(())
    }

    /// The OPFS folder of a section's big files; `None` when it does not exist and `create` is
    /// off.
    async fn section_dir(&self, section: &str, create: bool) -> io::Result<Option<FileSystemDirectoryHandle>> {
        let Some(root) = opfs_root().await else {
            // Nothing can be kept where there is no OPFS.
            return if create { Err(io::Error::other("this browser gives no private file system for big files")) } else { Ok(None) };
        };
        let mut dir = root;
        for part in [self.folder.as_str(), section] {
            let opts = FileSystemGetDirectoryOptions::new();
            opts.set_create(create);
            match JsFuture::from(dir.get_directory_handle_with_options(part, &opts)).await {
                Ok(d) => dir = d.dyn_into().map_err(js)?,
                Err(e) if !create && not_found(&e) => return Ok(None),
                Err(e) => return Err(js(e)),
            }
        }
        Ok(Some(dir))
    }

    /// A big file kept in OPFS.
    async fn big_file(&self, section: &str, name: &str) -> io::Result<File> {
        let missing = || io::Error::new(io::ErrorKind::NotFound, format!("{section}/{name}: not kept"));
        let Some(dir) = self.section_dir(section, false).await? else { return Err(missing()) };
        let handle = match JsFuture::from(dir.get_file_handle(name)).await {
            Ok(h) => h.dyn_into::<FileSystemFileHandle>().map_err(js)?,
            Err(e) if not_found(&e) => return Err(missing()),
            Err(e) => return Err(js(e)),
        };
        JsFuture::from(handle.get_file()).await.map_err(js)?.dyn_into().map_err(js)
    }

    /// Remove a big file's OPFS copy; a missing one is not an error.
    async fn remove_big(&self, section: &str, name: &str) -> io::Result<()> {
        let Some(dir) = self.section_dir(section, false).await? else { return Ok(()) };
        match JsFuture::from(dir.remove_entry(name)).await {
            Ok(_) => Ok(()),
            Err(e) if not_found(&e) => Ok(()),
            Err(e) => Err(js(e)),
        }
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
    fn write_file<'a>(
        &'a self,
        section: &'a str,
        name: &'a str,
        file: &'a File,
        card: Option<Card>,
    ) -> LocalFuture<'a, io::Result<()>> {
        Box::pin(IndexedDb::write_file(self, section, name, file, card))
    }
    fn file<'a>(&'a self, section: &'a str, name: &'a str) -> LocalFuture<'a, io::Result<File>> {
        Box::pin(self.file_now(section, name))
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
