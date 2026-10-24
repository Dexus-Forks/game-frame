//! Integer-to-text conversions from the MSVC CRT.
//!
//! These functions return owned ASCII strings instead of writing through C pointers. Radices
//! from 2 through 36 use the same digit alphabet. Signed inputs get a minus sign only in base 10,
//! matching `_itoa`, `_ltoa` and `_i64toa`; other bases format the input's unsigned bit pattern.

fn unsigned_to_string(mut value: u64, radix: u32) -> Option<String> {
    if !(2..=36).contains(&radix) {
        return None;
    }
    let mut digits = Vec::new();
    loop {
        let digit = (value % u64::from(radix)) as u8;
        digits.push(if digit < 10 {
            b'0' + digit
        } else {
            b'a' + (digit - 10)
        });
        value /= u64::from(radix);
        if value == 0 {
            break;
        }
    }
    digits.reverse();
    Some(String::from_utf8(digits).expect("digits are ASCII"))
}

/// Formats a 32-bit signed integer, like `_itoa`.
pub fn itoa(value: i32, radix: u32) -> Option<String> {
    if radix == 10 && value < 0 {
        let mut out = String::from("-");
        out.push_str(&unsigned_to_string(value.unsigned_abs() as u64, radix)?);
        Some(out)
    } else {
        unsigned_to_string(value as u32 as u64, radix)
    }
}

/// Formats a 32-bit unsigned integer, like `_ultoa`.
pub fn ultoa(value: u32, radix: u32) -> Option<String> {
    unsigned_to_string(u64::from(value), radix)
}

/// Formats a 64-bit signed integer, like `_i64toa`.
pub fn i64toa(value: i64, radix: u32) -> Option<String> {
    if radix == 10 && value < 0 {
        let mut out = String::from("-");
        out.push_str(&unsigned_to_string(value.unsigned_abs(), radix)?);
        Some(out)
    } else {
        unsigned_to_string(value as u64, radix)
    }
}

/// Formats a 64-bit unsigned integer, like `_ui64toa`.
pub fn ui64toa(value: u64, radix: u32) -> Option<String> {
    unsigned_to_string(value, radix)
}
