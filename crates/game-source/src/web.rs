//! The browser side: an image at a URL, read with HTTP range requests, or image files dropped
//! onto the page, read in slices. Fetched chunks are kept in the origin private file system
//! (OPFS), or in IndexedDB where OPFS cannot write, so a second visit fetches nothing.
//!
//! A URL on another origin must answer with `Access-Control-Allow-Origin` (CORS) or the
//! browser hides the bytes; a range of one part is a "safelisted" request header, so no
//! preflight is needed.

use crate::md5;
use crate::remote::{ChunkStore, Fetch, LocalFuture, Part, RemoteImage};
use crate::url::{file_name, is_cue, sibling};
use std::io;
use std::sync::Arc;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

fn js(e: JsValue) -> io::Error {
    let msg = e
        .dyn_ref::<js_sys::Error>()
        .map(|e| String::from(e.message()))
        .or_else(|| e.as_string())
        .unwrap_or_else(|| format!("{e:?}"));
    io::Error::other(msg)
}

fn window() -> io::Result<web_sys::Window> {
    web_sys::window().ok_or_else(|| io::Error::other("no window"))
}

async fn bytes_of(promise: js_sys::Promise) -> io::Result<Vec<u8>> {
    let buf = JsFuture::from(promise).await.map_err(js)?;
    Ok(js_sys::Uint8Array::new(&buf).to_vec())
}

async fn request(
    url: &str,
    method: &str,
    range: Option<(u64, u64)>,
) -> io::Result<web_sys::Response> {
    let init = web_sys::RequestInit::new();
    init.set_method(method);
    if let Some((start, end)) = range {
        let headers = web_sys::Headers::new().map_err(js)?;
        headers
            .set("Range", &format!("bytes={start}-{}", end - 1))
            .map_err(js)?;
        init.set_headers(&headers);
    }
    let resp = JsFuture::from(window()?.fetch_with_str_and_init(url, &init))
        .await
        .map_err(|e| {
            io::Error::other(format!(
                "{url}: {} (a server on another origin must allow it with CORS)",
                js(e)
            ))
        })?;
    let resp: web_sys::Response = resp.dyn_into().map_err(js)?;
    if !resp.ok() {
        return Err(io::Error::other(format!(
            "{url}: the server answered {}",
            resp.status()
        )));
    }
    Ok(resp)
}

/// The size of the file at `url`: Content-Length of a HEAD request, or the total of a
/// one-byte range.
pub async fn http_length(url: &str) -> io::Result<u64> {
    if let Ok(resp) = request(url, "HEAD", None).await
        && let Some(n) = resp
            .headers()
            .get("content-length")
            .ok()
            .flatten()
            .and_then(|v| v.parse().ok())
    {
        return Ok(n);
    }
    let resp = request(url, "GET", Some((0, 1))).await?;
    let range = resp.headers().get("content-range").ok().flatten();
    range.and_then(|r| r.rsplit('/').next()?.parse().ok()).ok_or_else(|| io::Error::other(format!("{url}: the server gives no size (Content-Length, or Content-Range exposed to this page)")))
}

pub async fn http_bytes(url: &str) -> io::Result<Vec<u8>> {
    bytes_of(
        request(url, "GET", None)
            .await?
            .array_buffer()
            .map_err(js)?,
    )
    .await
}

/// A file on an HTTP server that answers range requests.
pub struct HttpFetch(pub String);

impl Fetch for HttpFetch {
    fn fetch(&self, start: u64, end: u64) -> LocalFuture<'_, io::Result<Vec<u8>>> {
        Box::pin(async move {
            let resp = request(&self.0, "GET", Some((start, end))).await?;
            let whole = resp.status() == 200;
            let bytes = bytes_of(resp.array_buffer().map_err(js)?).await?;
            if whole {
                // The server ignored the range and sent the whole file.
                return bytes
                    .get(start as usize..end as usize)
                    .map(<[u8]>::to_vec)
                    .ok_or_else(|| {
                        io::Error::other(format!("{}: shorter than {end} bytes", self.0))
                    });
            }
            Ok(bytes)
        })
    }
}

/// A file the user dropped onto the page or chose: read a slice at a time.
pub struct FileFetch(pub web_sys::File);

impl Fetch for FileFetch {
    fn fetch(&self, start: u64, end: u64) -> LocalFuture<'_, io::Result<Vec<u8>>> {
        Box::pin(async move {
            let blob = self
                .0
                .slice_with_f64_and_f64(start as f64, end as f64)
                .map_err(js)?;
            bytes_of(blob.array_buffer()).await
        })
    }
}

/// Chunks in the origin private file system, one file each, in one folder.
pub struct Opfs(web_sys::FileSystemDirectoryHandle);

impl Opfs {
    /// The folder `name`, after a test write (some browsers only write OPFS from workers).
    pub async fn open(name: &str) -> io::Result<Opfs> {
        let root = JsFuture::from(window()?.navigator().storage().get_directory())
            .await
            .map_err(js)?;
        let root: web_sys::FileSystemDirectoryHandle = root.dyn_into().map_err(js)?;
        let opts = web_sys::FileSystemGetDirectoryOptions::new();
        opts.set_create(true);
        let dir = JsFuture::from(root.get_directory_handle_with_options(name, &opts))
            .await
            .map_err(js)?;
        let store = Opfs(dir.dyn_into().map_err(js)?);
        store.write("probe", b"ok").await?;
        Ok(store)
    }

    async fn write(&self, key: &str, bytes: &[u8]) -> io::Result<()> {
        let opts = web_sys::FileSystemGetFileOptions::new();
        opts.set_create(true);
        let handle: web_sys::FileSystemFileHandle =
            JsFuture::from(self.0.get_file_handle_with_options(key, &opts))
                .await
                .map_err(js)?
                .dyn_into()
                .map_err(js)?;
        let stream: web_sys::FileSystemWritableFileStream =
            JsFuture::from(handle.create_writable())
                .await
                .map_err(js)?
                .dyn_into()
                .map_err(js)?;
        JsFuture::from(stream.write_with_u8_array(bytes).map_err(js)?)
            .await
            .map_err(js)?;
        JsFuture::from(stream.close()).await.map_err(js)?;
        Ok(())
    }

    async fn read(&self, key: &str) -> io::Result<Vec<u8>> {
        let handle: web_sys::FileSystemFileHandle = JsFuture::from(self.0.get_file_handle(key))
            .await
            .map_err(js)?
            .dyn_into()
            .map_err(js)?;
        let file: web_sys::File = JsFuture::from(handle.get_file())
            .await
            .map_err(js)?
            .dyn_into()
            .map_err(js)?;
        bytes_of(file.array_buffer()).await
    }
}

impl ChunkStore for Opfs {
    fn get(&self, key: &str) -> LocalFuture<'_, Option<Vec<u8>>> {
        let key = key.to_string();
        Box::pin(async move { self.read(&key).await.ok() })
    }
    fn put(&self, key: &str, bytes: &[u8]) -> LocalFuture<'_, ()> {
        let (key, bytes) = (key.to_string(), bytes.to_vec());
        Box::pin(async move {
            if let Err(e) = self.write(&key, &bytes).await {
                log(&format!("OPFS: could not keep {key}: {e}"));
            }
        })
    }
}

/// Chunks in IndexedDB: database `name`, object store "chunks", keyed by name.
pub struct Idb(web_sys::IdbDatabase);

/// Wait for an IndexedDB request; its result.
async fn idb_wait(req: &web_sys::IdbRequest) -> io::Result<JsValue> {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let r = req.clone();
        let ok = wasm_bindgen::closure::Closure::once_into_js(move |_: web_sys::Event| {
            let _ = resolve.call1(&JsValue::NULL, &r.result().unwrap_or(JsValue::UNDEFINED));
        });
        let fail = wasm_bindgen::closure::Closure::once_into_js(move |e: web_sys::Event| {
            let _ = reject.call1(&JsValue::NULL, &e);
        });
        req.set_onsuccess(Some(ok.unchecked_ref()));
        req.set_onerror(Some(fail.unchecked_ref()));
    });
    JsFuture::from(promise)
        .await
        .map_err(|_| io::Error::other("IndexedDB request failed"))
}

impl Idb {
    pub async fn open(name: &str) -> io::Result<Idb> {
        let factory = window()?
            .indexed_db()
            .map_err(js)?
            .ok_or_else(|| io::Error::other("no IndexedDB"))?;
        let req = factory.open_with_u32(name, 1).map_err(js)?;
        let r = req.clone();
        let upgrade = wasm_bindgen::closure::Closure::once_into_js(move |_: web_sys::Event| {
            if let Ok(db) = r
                .result()
                .and_then(|db| db.dyn_into::<web_sys::IdbDatabase>())
            {
                let _ = db.create_object_store("chunks");
            }
        });
        req.set_onupgradeneeded(Some(upgrade.unchecked_ref()));
        let db = idb_wait(&req).await?;
        Ok(Idb(db.dyn_into().map_err(js)?))
    }

    fn store(&self, mode: web_sys::IdbTransactionMode) -> io::Result<web_sys::IdbObjectStore> {
        self.0
            .transaction_with_str_and_mode("chunks", mode)
            .map_err(js)?
            .object_store("chunks")
            .map_err(js)
    }
}

impl ChunkStore for Idb {
    fn get(&self, key: &str) -> LocalFuture<'_, Option<Vec<u8>>> {
        let key = key.to_string();
        Box::pin(async move {
            let req = self
                .store(web_sys::IdbTransactionMode::Readonly)
                .ok()?
                .get(&JsValue::from_str(&key))
                .ok()?;
            let v = idb_wait(&req).await.ok()?;
            (!v.is_undefined()).then(|| js_sys::Uint8Array::new(&v).to_vec())
        })
    }
    fn put(&self, key: &str, bytes: &[u8]) -> LocalFuture<'_, ()> {
        let (key, value) = (key.to_string(), js_sys::Uint8Array::from(bytes));
        Box::pin(async move {
            let done = match self
                .store(web_sys::IdbTransactionMode::Readwrite)
                .and_then(|s| s.put_with_key(&value, &JsValue::from_str(&key)).map_err(js))
            {
                Ok(req) => idb_wait(&req).await.map(|_| ()),
                Err(e) => Err(e),
            };
            if let Err(e) = done {
                log(&format!("IndexedDB: could not keep {key}: {e}"));
            }
        })
    }
}

fn log(msg: &str) {
    web_sys::console::log_1(&JsValue::from_str(msg));
}

/// The persistent store: OPFS, else IndexedDB, else none (chunks are then fetched again
/// on the next visit). The second value says which.
pub async fn open_store() -> (Option<Arc<dyn ChunkStore>>, &'static str) {
    match Opfs::open("game-source").await {
        Ok(s) => return (Some(Arc::new(s)), "OPFS"),
        Err(e) => log(&format!("OPFS unavailable ({e}); trying IndexedDB")),
    }
    match Idb::open("game-source").await {
        Ok(s) => (Some(Arc::new(s)), "IndexedDB"),
        Err(e) => {
            log(&format!("IndexedDB unavailable ({e}); chunks are not kept"));
            (None, "no cache")
        }
    }
}

/// The image at `url` (a `.cue` sheet with its files beside it, or one `.bin` or `.iso`),
/// fetched in ranges and kept in `store`.
pub async fn open_url(url: &str, store: Option<Arc<dyn ChunkStore>>) -> io::Result<RemoteImage> {
    let part = |id: u64, url: String, len: u64| {
        let key = format!("{}-{len}", md5::hex(url.as_bytes()));
        Part::new(
            id,
            &file_name(&url),
            len,
            Box::new(HttpFetch(url)),
            store.clone().map(|s| (s, key)),
        )
    };
    if is_cue(url) {
        let text = String::from_utf8_lossy(&http_bytes(url).await?).into_owned();
        let cue = disc_image::parse_cue(&text)?;
        let mut parts = Vec::new();
        for (i, f) in cue.files.iter().enumerate() {
            let u = sibling(url, f.name.rsplit(['/', '\\']).next().unwrap_or(&f.name));
            let len = http_length(&u).await?;
            parts.push(part(i as u64, u, len));
        }
        RemoteImage::open(Some(&text), parts).await
    } else {
        let len = http_length(url).await?;
        RemoteImage::open(None, vec![part(0, url.to_string(), len)]).await
    }
}

/// Image files the user dropped or chose: a `.cue` sheet with its files, or one image.
pub async fn open_files(files: Vec<web_sys::File>) -> io::Result<RemoteImage> {
    let cue = files
        .iter()
        .find(|f| f.name().to_ascii_lowercase().ends_with(".cue"))
        .cloned();
    let parts: Vec<Part> = files
        .into_iter()
        .filter(|f| !f.name().to_ascii_lowercase().ends_with(".cue"))
        .enumerate()
        .map(|(i, f)| {
            Part::new(
                i as u64,
                &f.name(),
                f.size() as u64,
                Box::new(FileFetch(f)),
                None,
            )
        })
        .collect();
    match cue {
        Some(c) => {
            let text = String::from_utf8_lossy(&bytes_of(c.array_buffer()).await?).into_owned();
            RemoteImage::open(Some(&text), parts).await
        }
        None if parts.len() == 1 => RemoteImage::open(None, parts).await,
        None => Err(io::Error::other(
            "drop one image file, or a .cue sheet with its files",
        )),
    }
}
