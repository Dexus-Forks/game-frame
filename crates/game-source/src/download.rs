//! Download an image from a URL the user gives, natively: into a cache folder, resuming a
//! broken download with an HTTP range request, and checked against the MD5s the user gives.
//! A URL ending in `.cue` brings the files its sheet names, from beside it.

use crate::md5::Md5;
use crate::url::{file_name, sibling};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

fn other(msg: String) -> io::Error {
    io::Error::other(msg)
}

/// What a download reports as it goes: the file, the bytes so far, the size when known.
pub type Progress<'a> = &'a mut dyn FnMut(&str, u64, Option<u64>);

/// Download `url` into `cache` (kept when it is already complete) and return the local path
/// to open: the image, or the CUE sheet with its files beside it. Every MD5 in `md5s` must be
/// the MD5 of one of the image files (not the sheet); the computed MD5s are printed to the
/// progress as a file name of "md5 NAME HEX" so the user can compare them with the archive's.
pub fn fetch(url: &str, cache: &Path, md5s: &[String], progress: Progress) -> io::Result<PathBuf> {
    fs::create_dir_all(cache)?;
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into();
    let name = file_name(url);
    if name.is_empty() {
        return Err(other(format!("{url}: the URL names no file")));
    }
    let mut images = Vec::new();
    let path = if name.to_ascii_lowercase().ends_with(".cue") {
        let sheet = get_file(&agent, url, &cache.join(&name), progress)?;
        let cue = disc_image::parse_cue(&String::from_utf8_lossy(&fs::read(&sheet)?))?;
        for f in &cue.files {
            let want = f.name.rsplit(['/', '\\']).next().unwrap_or(&f.name);
            images.push(get_file(
                &agent,
                &sibling(url, want),
                &cache.join(want),
                progress,
            )?);
        }
        sheet
    } else {
        let p = get_file(&agent, url, &cache.join(&name), progress)?;
        images.push(p.clone());
        p
    };
    let mut sums = Vec::new();
    for p in &images {
        let sum = md5_of(p)?;
        progress(
            &format!(
                "md5 {} {sum}",
                p.file_name().unwrap_or_default().to_string_lossy()
            ),
            0,
            None,
        );
        sums.push(sum);
    }
    for want in md5s {
        if !sums.iter().any(|s| s.eq_ignore_ascii_case(want.trim())) {
            return Err(other(format!(
                "no downloaded file has the MD5 {want} (they have {}); the files stay in {} for you to look at",
                sums.join(", "),
                cache.display()
            )));
        }
    }
    Ok(path)
}

/// The MD5 of a file, remembered beside it in NAME.md5 (recomputed when the size changes).
fn md5_of(path: &Path) -> io::Result<String> {
    let size = fs::metadata(path)?.len();
    let note = path.with_file_name(format!(
        "{}.md5",
        path.file_name().unwrap_or_default().to_string_lossy()
    ));
    if let Ok(text) = fs::read_to_string(&note)
        && let Some((hex, n)) = text.trim().split_once(' ')
        && n.parse() == Ok(size)
    {
        return Ok(hex.to_string());
    }
    let mut m = Md5::new();
    let mut f = File::open(path)?;
    let mut buf = vec![0; 1 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        m.update(&buf[..n]);
    }
    let hex = m.hex();
    fs::write(&note, format!("{hex} {size}\n"))?;
    Ok(hex)
}

/// `bytes a-b/total` or `bytes */total`.
fn total_of(content_range: &str) -> Option<u64> {
    content_range.rsplit('/').next()?.trim().parse().ok()
}

/// Download one file to `dest`, resuming `dest.part`; a complete `dest` is kept as it is.
fn get_file(
    agent: &ureq::Agent,
    url: &str,
    dest: &Path,
    progress: Progress,
) -> io::Result<PathBuf> {
    let shown = dest
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let url_note = dest.with_file_name(format!("{shown}.url"));
    if let Ok(old) = fs::read_to_string(&url_note)
        && old.trim() != url
    {
        return Err(other(format!(
            "{} came from {}, not {url}: give another --cache folder or remove it",
            dest.display(),
            old.trim()
        )));
    }
    fs::write(&url_note, format!("{url}\n"))?;
    if dest.exists() {
        return Ok(dest.to_path_buf());
    }
    let part = dest.with_file_name(format!("{shown}.part"));
    let mut tries = 0;
    loop {
        let have = fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
        let mut req = agent.get(url);
        if have > 0 {
            req = req.header("Range", &format!("bytes={have}-"));
        }
        let resp = req.call().map_err(|e| other(format!("{url}: {e}")))?;
        let status = resp.status().as_u16();
        let header = |k: &str| {
            resp.headers()
                .get(k)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        };
        let (mut file, mut done, total) = match status {
            206 => {
                let total = header("content-range").as_deref().and_then(total_of);
                (OpenOptions::new().append(true).open(&part)?, have, total)
            }
            200 => {
                let total = header("content-length").and_then(|v| v.parse().ok());
                (File::create(&part)?, 0, total)
            }
            // The part file already holds everything.
            416 if have > 0
                && header("content-range").as_deref().and_then(total_of) == Some(have) =>
            {
                fs::rename(&part, dest)?;
                return Ok(dest.to_path_buf());
            }
            s => return Err(other(format!("{url}: the server answered {s}"))),
        };
        let mut body = resp.into_body().into_with_config().limit(u64::MAX).reader();
        let mut buf = vec![0; 1 << 16];
        let mut last_report = 0;
        let result: io::Result<()> = (|| loop {
            let n = body.read(&mut buf)?;
            if n == 0 {
                return Ok(());
            }
            file.write_all(&buf[..n])?;
            done += n as u64;
            if done - last_report >= 1 << 22 {
                last_report = done;
                progress(&shown, done, total);
            }
        })();
        file.flush()?;
        drop(file);
        progress(&shown, done, total);
        match (result, total) {
            (Ok(()), Some(t)) if done == t => break,
            (Ok(()), None) => break,
            (Ok(()), Some(t)) if done > t => {
                return Err(other(format!(
                    "{url}: got {done} bytes, the server said {t}"
                )));
            }
            // Cut off: resume from where it stopped, a few times.
            (r, _) => {
                tries += 1;
                if tries > 5 {
                    r?;
                    return Err(other(format!(
                        "{url}: the download keeps stopping at {done} bytes"
                    )));
                }
            }
        }
    }
    fs::rename(&part, dest)?;
    Ok(dest.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_range_totals() {
        assert_eq!(total_of("bytes 0-15/854537054"), Some(854_537_054));
        assert_eq!(total_of("bytes */12"), Some(12));
    }

    /// A local server that answers ranges and breaks its first answer off half way.
    #[test]
    fn resumes_and_checks_the_md5() {
        use std::net::TcpListener;
        let body: Vec<u8> = (0..300_000u32).map(|i| (i * 7 % 256) as u8).collect();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let served = body.clone();
        std::thread::spawn(move || {
            for (n, stream) in listener.incoming().enumerate() {
                let mut s = stream.unwrap();
                let mut req = [0u8; 4096];
                let len = s.read(&mut req).unwrap();
                let req = String::from_utf8_lossy(&req[..len]).to_ascii_lowercase();
                let from: usize = req
                    .split("range: bytes=")
                    .nth(1)
                    .and_then(|r| r.split('-').next()?.parse().ok())
                    .unwrap_or(0);
                let head = if from > 0 {
                    format!(
                        "HTTP/1.1 206 Partial\r\nContent-Length: {}\r\nContent-Range: bytes {from}-{}/{}\r\nConnection: close\r\n\r\n",
                        served.len() - from,
                        served.len() - 1,
                        served.len()
                    )
                } else {
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        served.len()
                    )
                };
                s.write_all(head.as_bytes()).unwrap();
                // The first answer stops after 100,000 bytes.
                let end = if n == 0 { 100_000 } else { served.len() };
                let _ = s.write_all(&served[from..end.max(from)]);
            }
        });
        let dir = std::env::temp_dir().join(format!("game-source-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let url = format!("http://127.0.0.1:{port}/files/test.iso");
        let want = crate::md5::hex(&body);
        let mut seen = Vec::new();
        let p = fetch(&url, &dir, &[want.to_uppercase()], &mut |name, _, _| {
            seen.push(name.to_string())
        })
        .unwrap();
        assert_eq!(fs::read(&p).unwrap(), body);
        assert!(seen.contains(&format!("md5 test.iso {want}")));
        // Kept: a second call does not download, and a wrong MD5 is an error.
        assert!(fetch(&url, &dir, &["00".into()], &mut |_, _, _| {}).is_err());
        // Another URL with the same file name is refused.
        assert!(
            fetch(
                &format!("http://127.0.0.1:{port}/other/test.iso"),
                &dir,
                &[],
                &mut |_, _, _| {}
            )
            .is_err()
        );
        fs::remove_dir_all(&dir).unwrap();
    }
}
