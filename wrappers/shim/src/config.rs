//! Settings every wrapper DLL shares, from environment variables (Wine passes the Unix
//! environment through). Plain Rust: the parsers are tested on the host.
//!
//! - `WRAPPER_WINDOW_CLASS`: the class name of the game's main window (`glide` and `sgl`
//!   draw into it). Without it they take the first visible top-level window of the process.
//! - `WRAPPER_MARKER`: the RGB565 colour that marks "not drawn by the game" in a 2D picture
//!   laid over the 3D frame (`glide`'s frame buffer writes, `sgl`'s address-mode surface).
//!   Hex (`0x0821`) or decimal. Default 0x0821, (8, 4, 8): almost black.
//! - `WRAPPER_PATCHES`: a file of in-memory patches to the game's code (`patch`), in the format
//!   `parse_patches` reads.

/// The default marker: RGB565 for (8, 4, 8). Choose another with `WRAPPER_MARKER` when the
/// game draws this exact colour in its 2D pictures.
pub const DEFAULT_MARKER: u16 = 0x0821;

fn var(k: &str) -> Option<String> {
    std::env::var(k).ok().filter(|v| !v.trim().is_empty())
}

/// A number in hex (`0x` prefix) or decimal.
pub fn parse_number(s: &str) -> Option<u32> {
    let s = s.trim();
    match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Some(h) => u32::from_str_radix(h, 16).ok(),
        None => s.parse().ok(),
    }
}

/// `WRAPPER_MARKER`, or the default. A value above 0xffff is ignored.
pub fn marker() -> u16 {
    static M: std::sync::OnceLock<u16> = std::sync::OnceLock::new();
    *M.get_or_init(|| {
        var("WRAPPER_MARKER").and_then(|v| parse_number(&v)).and_then(|v| u16::try_from(v).ok()).unwrap_or(DEFAULT_MARKER)
    })
}

/// `WRAPPER_WINDOW_CLASS`, as bytes to compare with `GetClassNameA`.
pub fn window_class() -> Option<&'static [u8]> {
    static C: std::sync::OnceLock<Option<Vec<u8>>> = std::sync::OnceLock::new();
    C.get_or_init(|| var("WRAPPER_WINDOW_CLASS").map(String::into_bytes)).as_deref()
}

/// `WRAPPER_PATCHES`: the file's name.
pub fn patch_file() -> Option<String> {
    var("WRAPPER_PATCHES")
}

/// One patch to the game's code: applied when the running EXE's PE timestamp is `stamp` and
/// the bytes at `address` are `old`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Patch {
    pub stamp: u32,
    pub name: String,
    pub address: u32,
    pub old: Vec<u8>,
    pub new: Vec<u8>,
    pub why: String,
}

fn hex_bytes(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect()
}

/// A patch file: one patch per line, fields split by spaces or tabs:
///
/// ```text
/// # stamp     build         address   old bytes       new bytes       why
/// 0x345270ab  GAME3DFX.EXE  0x486aa1  c705...00ff     c705...0004     unknown CPU: kind 4
/// ```
///
/// The timestamp is the EXE's PE header `TimeDateStamp`; the address is a virtual address in
/// the loaded EXE; old and new are hex bytes of the same length. Everything after the fifth
/// field is the reason, written to the log. Blank lines and lines starting with `#` are
/// skipped. A line that does not parse is an error naming its number.
pub fn parse_patches(text: &str) -> Result<Vec<Patch>, String> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let bad = |what: &str| format!("line {}: {what}", i + 1);
        let mut f = line.split_whitespace();
        let stamp = f.next().and_then(parse_number).ok_or_else(|| bad("no timestamp"))?;
        let name = f.next().ok_or_else(|| bad("no build name"))?.to_string();
        let address = f.next().and_then(parse_number).ok_or_else(|| bad("no address"))?;
        let old = f.next().and_then(hex_bytes).ok_or_else(|| bad("old bytes are not hex"))?;
        let new = f.next().and_then(hex_bytes).ok_or_else(|| bad("new bytes are not hex"))?;
        if old.len() != new.len() || old.is_empty() {
            return Err(bad("old and new bytes differ in length"));
        }
        let why = f.collect::<Vec<_>>().join(" ");
        out.push(Patch { stamp, name, address, old, new, why });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        assert_eq!(parse_number("0x0821"), Some(0x821));
        assert_eq!(parse_number("2081"), Some(2081));
        assert_eq!(parse_number("0xzz"), None);
    }

    #[test]
    fn patches() {
        let text = "# comment\n\n0x10 GAME.EXE 0x401000 c705ff c70504 unknown CPU: kind 4\n";
        let p = parse_patches(text).unwrap();
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].stamp, 0x10);
        assert_eq!(p[0].address, 0x401000);
        assert_eq!(p[0].old, [0xc7, 0x05, 0xff]);
        assert_eq!(p[0].new, [0xc7, 0x05, 0x04]);
        assert_eq!(p[0].why, "unknown CPU: kind 4");
        assert!(parse_patches("0x10 GAME.EXE 0x401000 c705 c7").unwrap_err().starts_with("line 1"));
        assert!(parse_patches("0x10 GAME.EXE 0x401000 c7g5 c705").is_err());
    }
}
