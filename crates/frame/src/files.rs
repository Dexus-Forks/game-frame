//! The file lists behind a choice such as `"from": {"files": "DATA/TRACKS/*.TRK"}`, from a
//! folder or from a disc image.

use std::path::Path;

/// The names, without their extensions and sorted, of the files under `folder` that match
/// `pattern`. Folder names in the pattern are matched ignoring letter case, as on a game's disc;
/// only the last part may hold `*` (any run of characters) and `?` (one character). A folder
/// that does not exist gives an empty list.
pub fn list_files(folder: &Path, pattern: &str) -> Vec<String> {
    let parts: Vec<&str> = pattern.split(['/', '\\']).filter(|p| !p.is_empty()).collect();
    let Some((last, dirs)) = parts.split_last() else { return Vec::new() };
    let mut dir = folder.to_path_buf();
    for d in dirs {
        let Some(found) = entry(&dir, |name| name.eq_ignore_ascii_case(d)) else { return Vec::new() };
        dir = dir.join(found);
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| wildcard(&last.to_ascii_lowercase(), &name.to_ascii_lowercase()))
        .map(|name| Path::new(&name).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or(name))
        .collect();
    names.sort();
    names.dedup();
    names
}

/// The same list from a disc image (`.iso`, `.cue`, `.bin`), read in place: the files of its
/// ISO 9660 volume whose path matches `pattern`.
pub fn list_image_files(image: &Path, pattern: &str) -> std::io::Result<Vec<String>> {
    let volume = disc_image::Disc::open_path(image)?.volume()?;
    let parts: Vec<&str> = pattern.split(['/', '\\']).filter(|p| !p.is_empty()).collect();
    let Some((last, dirs)) = parts.split_last() else { return Ok(Vec::new()) };
    let last = last.to_ascii_lowercase();
    let mut names: Vec<String> = volume
        .files()
        .filter_map(|e| {
            let path: Vec<&str> = e.path.split('/').collect();
            let (name, folders) = path.split_last()?;
            let same = folders.len() == dirs.len() && folders.iter().zip(dirs).all(|(a, b)| a.eq_ignore_ascii_case(b));
            (same && wildcard(&last, &name.to_ascii_lowercase()))
                .then(|| Path::new(name).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| name.to_string()))
        })
        .collect();
    names.sort();
    names.dedup();
    Ok(names)
}

fn entry(dir: &Path, want: impl Fn(&str) -> bool) -> Option<String> {
    std::fs::read_dir(dir).ok()?.flatten().filter_map(|e| e.file_name().into_string().ok()).find(|n| want(n))
}

/// `*` matches any run of characters and `?` one character.
fn wildcard(pattern: &str, name: &str) -> bool {
    let (p, n): (Vec<char>, Vec<char>) = (pattern.chars().collect(), name.chars().collect());
    let (mut pi, mut ni, mut star, mut mark) = (0, 0, None, 0);
    while ni < n.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == n[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = ni;
            pi += 1;
        } else if let Some(s) = star {
            pi = s + 1;
            mark += 1;
            ni = mark;
        } else {
            return false;
        }
    }
    p[pi..].iter().all(|c| *c == '*')
}
