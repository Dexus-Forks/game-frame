//! Images whose bytes arrive later: an HTTP server that answers range requests, a file
//! dropped into a browser. Each image file is a [`disc_image::ChunkCache`]; reads run
//! synchronously against it, and when one fails with [`Missing`] the driver fetches those
//! chunks (from a persistent [`ChunkStore`] first, else from the [`Fetch`]) and runs it again.
//!
//! The futures are not `Send`: in the browser they hold JavaScript values.

use crate::GameFile;
use disc_image::{ChunkCache, Disc, Missing, ReadAt, Volume, parse_cue};
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;

pub type LocalFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Bytes `start..end` of one file, from wherever it lives.
pub trait Fetch {
    fn fetch(&self, start: u64, end: u64) -> LocalFuture<'_, io::Result<Vec<u8>>>;
}

/// A cache of chunks that outlives the page: OPFS or IndexedDB in the browser.
pub trait ChunkStore {
    fn get(&self, key: &str) -> LocalFuture<'_, Option<Vec<u8>>>;
    fn put(&self, key: &str, bytes: &[u8]) -> LocalFuture<'_, ()>;
}

/// The chunk size: big enough that a file needs few requests, small enough that a directory
/// read costs little.
pub const CHUNK: u64 = 1 << 20;
/// Most chunks fetched in one request.
const RUN: usize = 16;
/// Chunks held in memory before the cache is emptied (the store keeps them).
const HELD: usize = 64;

/// One file of an image (a BIN, an ISO), fetched on demand.
pub struct Part {
    pub name: String,
    pub cache: Arc<ChunkCache>,
    fetch: Box<dyn Fetch>,
    /// The store and the key prefix of this file in it.
    store: Option<(Arc<dyn ChunkStore>, String)>,
}

impl Part {
    /// `key` names the file in the store: the same key must mean the same bytes (the URL and
    /// its length, say).
    pub fn new(
        id: u64,
        name: &str,
        len: u64,
        fetch: Box<dyn Fetch>,
        store: Option<(Arc<dyn ChunkStore>, String)>,
    ) -> Part {
        Part {
            name: name.to_string(),
            cache: Arc::new(ChunkCache::new(id, len, CHUNK)),
            fetch,
            store,
        }
    }

    async fn fill(&self, chunks: &[u64]) -> io::Result<()> {
        let mut todo = Vec::new();
        for &i in chunks {
            let stored = match &self.store {
                Some((store, key)) => store.get(&format!("{key}-{i}")).await,
                None => None,
            };
            // A chunk of the wrong size (an older, broken store) is fetched again.
            if stored.is_none_or(|bytes| self.cache.insert(i, bytes).is_err()) {
                todo.push(i);
            }
        }
        // Consecutive chunks in one request.
        let mut runs: Vec<Vec<u64>> = Vec::new();
        for i in todo {
            match runs.last_mut() {
                Some(r) if r.last() == Some(&(i - 1)) && r.len() < RUN => r.push(i),
                _ => runs.push(vec![i]),
            }
        }
        for run in runs {
            let (start, end) = (
                self.cache.range(run[0]).start,
                self.cache.range(*run.last().unwrap()).end,
            );
            let bytes = self.fetch.fetch(start, end).await?;
            if bytes.len() as u64 != end - start {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!(
                        "{}: asked for {} bytes at {start}, got {}",
                        self.name,
                        end - start,
                        bytes.len()
                    ),
                ));
            }
            for &i in &run {
                let r = self.cache.range(i);
                let chunk = bytes[(r.start - start) as usize..(r.end - start) as usize].to_vec();
                if let Some((store, key)) = &self.store {
                    store.put(&format!("{key}-{i}"), &chunk).await;
                }
                self.cache.insert(i, chunk)?;
            }
        }
        Ok(())
    }
}

/// Run `op` until it no longer misses chunks of `parts`, fetching what it misses.
pub async fn resolve<T>(parts: &[Part], mut op: impl FnMut() -> io::Result<T>) -> io::Result<T> {
    loop {
        match op() {
            Err(e) => {
                let Some(m) = Missing::of(&e).cloned() else {
                    return Err(e);
                };
                let part = parts.iter().find(|p| p.cache.id() == m.cache).ok_or(e)?;
                if part.cache.held() + m.chunks.len() > HELD {
                    part.cache.clear();
                }
                part.fill(&m.chunks).await?;
            }
            done => return done,
        }
    }
}

/// A CD image fetched on demand: its parts and its ISO 9660 volume.
pub struct RemoteImage {
    pub parts: Vec<Part>,
    pub volume: Volume,
}

impl RemoteImage {
    /// Open a CUE sheet whose FILEs are `parts` (matched by name, case-blind), or one part
    /// alone (a BIN or an ISO) when `cue` is `None`.
    pub async fn open(cue: Option<&str>, parts: Vec<Part>) -> io::Result<RemoteImage> {
        let sources: Vec<Arc<dyn ReadAt>> = parts
            .iter()
            .map(|p| p.cache.clone() as Arc<dyn ReadAt>)
            .collect();
        let volume = match cue {
            Some(text) => {
                let cue = parse_cue(text)?;
                resolve(&parts, || {
                    let disc = Disc::from_cue(&cue, |f| {
                        let want = f.name.rsplit(['/', '\\']).next().unwrap_or(&f.name);
                        let i = parts.iter().position(|p| p.name.eq_ignore_ascii_case(want));
                        i.map(|i| sources[i].clone()).ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::NotFound,
                                format!("the CUE sheet names {want}: add that file too"),
                            )
                        })
                    })?;
                    disc.volume()
                })
                .await?
            }
            None => {
                let only = sources
                    .first()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no image file"))?
                    .clone();
                let name = parts[0].name.clone();
                resolve(&parts, || Disc::from_raw(only.clone(), &name)?.volume()).await?
            }
        };
        Ok(RemoteImage { parts, volume })
    }

    /// Read every file under `subfolder` (named relative to it) for which `keep` says yes,
    /// handing each to `each` as it arrives; `first` (matched against the end of the path)
    /// comes first.
    pub async fn load(
        &self,
        subfolder: Option<&str>,
        keep: impl Fn(&str) -> bool,
        first: Option<&str>,
        mut each: impl FnMut(GameFile, usize, usize),
    ) -> io::Result<usize> {
        let prefix = subfolder
            .map(|s| format!("{}/", s.trim_matches('/').to_ascii_uppercase()))
            .unwrap_or_default();
        let mut todo: Vec<(&disc_image::Entry, String)> = self
            .volume
            .files()
            .filter_map(|e| {
                let p = e
                    .path
                    .get(..prefix.len())
                    .filter(|h| h.eq_ignore_ascii_case(&prefix))
                    .map(|_| e.path[prefix.len()..].to_string())?;
                keep(&p).then_some((e, p))
            })
            .collect();
        todo.sort_by(|a, b| a.1.cmp(&b.1));
        if let Some(want) = first.map(str::to_ascii_lowercase).filter(|w| !w.is_empty()) {
            todo.sort_by_key(|(_, p)| !p.to_ascii_lowercase().ends_with(&want));
        }
        let total = todo.len();
        for (n, (e, path)) in todo.into_iter().enumerate() {
            let bytes = resolve(&self.parts, || self.volume.read(e)).await?;
            each(
                GameFile {
                    path,
                    bytes: Arc::new(bytes),
                },
                n + 1,
                total,
            );
        }
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;

    struct Bytes(Vec<u8>, Cell<usize>);
    impl Fetch for Bytes {
        fn fetch(&self, start: u64, end: u64) -> LocalFuture<'_, io::Result<Vec<u8>>> {
            self.1.set(self.1.get() + 1);
            let out = self.0[start as usize..end as usize].to_vec();
            Box::pin(async move { Ok(out) })
        }
    }
    impl Fetch for Rc<Bytes> {
        fn fetch(&self, start: u64, end: u64) -> LocalFuture<'_, io::Result<Vec<u8>>> {
            (**self).fetch(start, end)
        }
    }
    use std::rc::Rc;

    #[derive(Default)]
    struct Store(RefCell<HashMap<String, Vec<u8>>>);
    impl ChunkStore for Store {
        fn get(&self, key: &str) -> LocalFuture<'_, Option<Vec<u8>>> {
            let v = self.0.borrow().get(key).cloned();
            Box::pin(async move { v })
        }
        fn put(&self, key: &str, bytes: &[u8]) -> LocalFuture<'_, ()> {
            self.0.borrow_mut().insert(key.to_string(), bytes.to_vec());
            Box::pin(async {})
        }
    }

    /// The futures here never wait, so polling once finishes them.
    fn block_on<T>(f: impl Future<Output = T>) -> T {
        let mut f = std::pin::pin!(f);
        match f
            .as_mut()
            .poll(&mut std::task::Context::from_waker(std::task::Waker::noop()))
        {
            std::task::Poll::Ready(v) => v,
            std::task::Poll::Pending => panic!("a test future waited"),
        }
    }

    /// A cooked ISO of `n` sectors with one file, DATA/A.TXT, at sector 20.
    fn tiny_iso() -> Vec<u8> {
        let mut img = vec![0u8; 3 * CHUNK as usize];
        let rec = |name: &[u8], lba: u32, size: u32, flags: u8| {
            let len = 33 + name.len() + (name.len() + 1) % 2;
            let mut r = vec![0; len];
            r[0] = len as u8;
            r[2..6].copy_from_slice(&lba.to_le_bytes());
            r[10..14].copy_from_slice(&size.to_le_bytes());
            r[25] = flags;
            r[32] = name.len() as u8;
            r[33..33 + name.len()].copy_from_slice(name);
            r
        };
        let mut pvd = vec![0; 2048];
        pvd[0] = 1;
        pvd[1..6].copy_from_slice(b"CD001");
        pvd[128..130].copy_from_slice(&2048u16.to_le_bytes());
        let root = rec(&[0], 18, 2048, 2);
        pvd[156..156 + root.len()].copy_from_slice(&root);
        img[16 * 2048..17 * 2048].copy_from_slice(&pvd);
        img[17 * 2048] = 255;
        img[17 * 2048 + 1..17 * 2048 + 6].copy_from_slice(b"CD001");
        let root_dir: Vec<u8> = [
            rec(&[0], 18, 2048, 2),
            rec(&[1], 18, 2048, 2),
            rec(b"DATA", 19, 2048, 2),
            rec(b"README.TXT;1", 20, 2, 0),
        ]
        .concat();
        img[18 * 2048..18 * 2048 + root_dir.len()].copy_from_slice(&root_dir);
        let data_dir: Vec<u8> = [
            rec(&[0], 19, 2048, 2),
            rec(&[1], 18, 2048, 2),
            rec(b"A.TXT;1", 600, 1_500_000, 0),
            rec(b"B.TXT;1", 20, 2, 0),
        ]
        .concat();
        img[19 * 2048..19 * 2048 + data_dir.len()].copy_from_slice(&data_dir);
        img[20 * 2048..20 * 2048 + 2].copy_from_slice(b"hi");
        for (i, b) in img[600 * 2048..600 * 2048 + 1_500_000]
            .iter_mut()
            .enumerate()
        {
            *b = (i % 253) as u8;
        }
        img
    }

    #[test]
    fn fetches_only_what_it_reads_and_reuses_the_store() {
        let img = tiny_iso();
        let store: Arc<dyn ChunkStore> = Arc::new(Store::default());
        let open = |fetch: Rc<Bytes>| {
            let part = Part::new(
                1,
                "tiny.iso",
                img.len() as u64,
                Box::new(fetch),
                Some((store.clone(), "tiny".into())),
            );
            block_on(RemoteImage::open(None, vec![part])).unwrap()
        };
        let fetch = Rc::new(Bytes(img.clone(), Cell::new(0)));
        let image = open(fetch.clone());
        assert_eq!(
            fetch.1.get(),
            1,
            "the descriptors and directories share the first chunk"
        );
        let mut got = Vec::new();
        let n = block_on(image.load(
            Some("data"),
            |p| p != "B.TXT",
            Some("a.txt"),
            |f, i, total| got.push((f.path, f.bytes.len(), i, total)),
        ))
        .unwrap();
        assert_eq!((n, got), (1, vec![("A.TXT".to_string(), 1_500_000, 1, 1)]));
        assert_eq!(fetch.1.get(), 2, "the file's two chunks in one request");

        // A second visit reads everything from the store.
        let again = Rc::new(Bytes(img.clone(), Cell::new(0)));
        let image = open(again.clone());
        block_on(image.load(None, |_| true, None, |_, _, _| {})).unwrap();
        assert_eq!(again.1.get(), 0);
    }

    #[test]
    fn a_cue_sheet_names_its_files() {
        let img = tiny_iso();
        let part = Part::new(
            3,
            "Game.bin",
            img.len() as u64,
            Box::new(Bytes(img.clone(), Cell::new(0))),
            None,
        );
        let cue = "FILE \"GAME.BIN\" BINARY\n TRACK 01 MODE1/2048\n  INDEX 01 00:00:00\n";
        let image = block_on(RemoteImage::open(Some(cue), vec![part])).unwrap();
        assert_eq!(image.volume.files().count(), 3);
        let other = Part::new(
            4,
            "other.bin",
            10,
            Box::new(Bytes(vec![0; 10], Cell::new(0))),
            None,
        );
        let err = block_on(RemoteImage::open(Some(cue), vec![other]))
            .err()
            .unwrap();
        assert!(err.to_string().contains("GAME.BIN"));
    }
}
