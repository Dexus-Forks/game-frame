//! File names in URLs.

/// The URL without its query and fragment.
pub fn bare(url: &str) -> &str {
    url.split(['?', '#']).next().unwrap_or(url)
}

/// The file name a URL ends in, percent-decoded.
pub fn file_name(url: &str) -> String {
    let last = bare(url)
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or_default();
    let mut out = Vec::new();
    let b = last.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let hex = |c: u8| (c as char).to_digit(16);
        match (
            b[i],
            b.get(i + 1).and_then(|&c| hex(c)),
            b.get(i + 2).and_then(|&c| hex(c)),
        ) {
            (b'%', Some(h), Some(l)) => {
                out.push((h * 16 + l) as u8);
                i += 3;
            }
            (c, _, _) => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).replace(['/', '\\'], "_")
}

/// The URL of `name` in the same folder as `url`.
pub fn sibling(url: &str, name: &str) -> String {
    let base = bare(url);
    let dir = &base[..base.rfind('/').map_or(base.len(), |i| i + 1)];
    let enc: String = name
        .bytes()
        .map(|c| {
            if c.is_ascii_alphanumeric() || b"-._~".contains(&c) {
                (c as char).to_string()
            } else {
                format!("%{c:02X}")
            }
        })
        .collect();
    format!("{dir}{enc}")
}

/// Whether a URL names a CUE sheet.
pub fn is_cue(url: &str) -> bool {
    bare(url).to_ascii_lowercase().ends_with(".cue")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_from_urls() {
        assert_eq!(
            file_name("https://example.org/a/My%20Disc.cue?x=1#y"),
            "My Disc.cue"
        );
        assert_eq!(
            sibling("https://example.org/a/My%20Disc.cue?x=1", "My Disc (1).bin"),
            "https://example.org/a/My%20Disc%20%281%29.bin"
        );
        assert!(is_cue("http://127.0.0.1:5183/GAME.CUE?x") && !is_cue("http://h/a.bin"));
    }
}
