//! Byte-string functions from the MSVC CRT.
//!
//! The functions use C string rules over byte slices. They stop at the first zero byte. If a
//! slice has no zero byte, its end acts as an implicit terminator. Search results are byte
//! offsets, not pointers. `_stricmp` and `_strnicmp` use ASCII case folding, which matches the
//! default C locale for ASCII text; other locale and code-page behavior is outside this module.
//!
//! The matching CRT exports include `strlen`, `strcmp`, `strncmp`, `strchr`, `strrchr`,
//! `strstr`, `_stricmp` and `_strnicmp` in the 2008, 2012 and 2013 x86 runtimes.

/// A destination slice cannot hold the requested string and its required terminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringError {
    /// Bytes needed by the operation.
    pub needed: usize,
    /// Bytes available in the destination.
    pub available: usize,
}

fn c_len(s: &[u8]) -> usize {
    s.iter().position(|&byte| byte == 0).unwrap_or(s.len())
}

fn sign(a: u8, b: u8) -> i32 {
    sign_ordering(a.cmp(&b))
}

fn sign_ordering(ordering: std::cmp::Ordering) -> i32 {
    match ordering {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn lower_ascii(byte: u8) -> u8 {
    if byte.is_ascii_uppercase() {
        byte + (b'a' - b'A')
    } else {
        byte
    }
}

/// Returns the byte length before the first zero byte, like `strlen`.
pub fn strlen(s: &[u8]) -> usize {
    c_len(s)
}

/// Compares two zero-terminated byte strings, like `strcmp`.
pub fn strcmp(a: &[u8], b: &[u8]) -> i32 {
    let (a_len, b_len) = (c_len(a), c_len(b));
    for index in 0..a_len.min(b_len) {
        let result = sign(a[index], b[index]);
        if result != 0 {
            return result;
        }
    }
    sign_ordering(a_len.cmp(&b_len))
}

/// Compares at most `count` bytes of two zero-terminated byte strings, like `strncmp`.
pub fn strncmp(a: &[u8], b: &[u8], count: usize) -> i32 {
    let (a_len, b_len) = (c_len(a), c_len(b));
    let common = a_len.min(b_len).min(count);
    for index in 0..common {
        let left = a[index];
        let right = b[index];
        let result = sign(left, right);
        if result != 0 {
            return result;
        }
    }
    if common == count {
        0
    } else {
        sign_ordering(a_len.cmp(&b_len))
    }
}

/// Compares strings without ASCII letter case, like `_stricmp` in the C locale.
pub fn stricmp_ascii(a: &[u8], b: &[u8]) -> i32 {
    let (a_len, b_len) = (c_len(a), c_len(b));
    for index in 0..a_len.min(b_len) {
        let left = lower_ascii(a[index]);
        let right = lower_ascii(b[index]);
        let result = sign(left, right);
        if result != 0 {
            return result;
        }
    }
    sign_ordering(a_len.cmp(&b_len))
}

/// Compares at most `count` bytes without ASCII letter case, like `_strnicmp` in the C locale.
pub fn strnicmp_ascii(a: &[u8], b: &[u8], count: usize) -> i32 {
    let (a_len, b_len) = (c_len(a), c_len(b));
    let common = a_len.min(b_len).min(count);
    for index in 0..common {
        let left = lower_ascii(a[index]);
        let right = lower_ascii(b[index]);
        let result = sign(left, right);
        if result != 0 {
            return result;
        }
    }
    if common == count {
        0
    } else {
        sign_ordering(a_len.cmp(&b_len))
    }
}

/// Finds the first occurrence of `byte`, like `strchr`.
///
/// Searching for zero returns the terminator offset, including an implicit one at the slice end.
pub fn strchr(s: &[u8], byte: u8) -> Option<usize> {
    let len = c_len(s);
    if byte == 0 {
        return Some(len);
    }
    s[..len].iter().position(|&value| value == byte)
}

/// Finds the last occurrence of `byte`, like `strrchr`.
pub fn strrchr(s: &[u8], byte: u8) -> Option<usize> {
    let len = c_len(s);
    if byte == 0 {
        return Some(len);
    }
    s[..len].iter().rposition(|&value| value == byte)
}

/// Finds the first occurrence of `needle`, like `strstr`.
pub fn strstr(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let haystack_len = c_len(haystack);
    let needle_len = c_len(needle);
    if needle_len == 0 {
        return Some(0);
    }
    haystack[..haystack_len]
        .windows(needle_len)
        .position(|window| window == &needle[..needle_len])
}

/// Counts the initial bytes in `s` that occur in `accept`, like `strspn`.
pub fn strspn(s: &[u8], accept: &[u8]) -> usize {
    let len = c_len(s);
    let accept = &accept[..c_len(accept)];
    s[..len]
        .iter()
        .take_while(|byte| accept.contains(byte))
        .count()
}

/// Counts the initial bytes in `s` that do not occur in `reject`, like `strcspn`.
pub fn strcspn(s: &[u8], reject: &[u8]) -> usize {
    let len = c_len(s);
    let reject = &reject[..c_len(reject)];
    s[..len]
        .iter()
        .take_while(|byte| !reject.contains(byte))
        .count()
}

/// Finds the first byte in `s` that occurs in `accept`, like `strpbrk`.
pub fn strpbrk(s: &[u8], accept: &[u8]) -> Option<usize> {
    let len = c_len(s);
    let accept = &accept[..c_len(accept)];
    s[..len].iter().position(|byte| accept.contains(byte))
}

/// Copies `src` and its terminator into `dst`, like `strcpy`.
///
/// Returns the number of copied string bytes, excluding the terminator.
pub fn strcpy(dst: &mut [u8], src: &[u8]) -> Result<usize, StringError> {
    let len = c_len(src);
    let needed = len + 1;
    if dst.len() < needed {
        return Err(StringError {
            needed,
            available: dst.len(),
        });
    }
    dst[..len].copy_from_slice(&src[..len]);
    dst[len] = 0;
    Ok(len)
}

/// Copies up to `count` bytes, padding with zero bytes when `src` is shorter, like `strncpy`.
///
/// As in C, the result is not terminated when the source length is at least `count`.
pub fn strncpy(dst: &mut [u8], src: &[u8], count: usize) -> Result<(), StringError> {
    if dst.len() < count {
        return Err(StringError {
            needed: count,
            available: dst.len(),
        });
    }
    let copy_len = c_len(src).min(count);
    dst[..copy_len].copy_from_slice(&src[..copy_len]);
    dst[copy_len..count].fill(0);
    Ok(())
}

/// Appends `src` and its terminator to `dst`, like `strcat`.
///
/// Returns the resulting string length, excluding the terminator.
pub fn strcat(dst: &mut [u8], src: &[u8]) -> Result<usize, StringError> {
    strncat(dst, src, usize::MAX)
}

/// Appends at most `count` bytes from `src`, then a terminator, like `strncat`.
///
/// Returns the resulting string length, excluding the terminator.
pub fn strncat(dst: &mut [u8], src: &[u8], count: usize) -> Result<usize, StringError> {
    let dst_len = c_len(dst);
    let append_len = c_len(src).min(count);
    let Some(needed) = dst_len
        .checked_add(append_len)
        .and_then(|length| length.checked_add(1))
    else {
        return Err(StringError {
            needed: usize::MAX,
            available: dst.len(),
        });
    };
    if dst.len() < needed {
        return Err(StringError {
            needed,
            available: dst.len(),
        });
    }
    dst[dst_len..dst_len + append_len].copy_from_slice(&src[..append_len]);
    dst[dst_len + append_len] = 0;
    Ok(dst_len + append_len)
}
