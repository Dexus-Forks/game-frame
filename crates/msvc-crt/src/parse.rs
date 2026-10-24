//! Integer text conversion from the MSVC CRT.
//!
//! `strtol`, `strtoul`, `strtoll` and `strtoull` parse ASCII digits in bases 2 through 36, or
//! choose a base with `radix == 0`. The Windows `long` type is 32 bits on both x86 and x64, so
//! `strtol` and `strtoul` return 32-bit values here. The `end` field is a byte offset into the
//! input slice. The result also reports range overflow and an invalid base instead of modifying
//! the CRT's thread-local `errno`.
//!
//! The parser follows the digit, sign, base-selection and overflow-detection paths visible in
//! MSVCR120's `strtol`, `strtoul` and shared conversion routine. Locale-specific whitespace and
//! digits are outside this module; it uses the six C-locale whitespace bytes and ASCII digits.

/// A parsed integer and the conditions that a C CRT reports through `errno` and `endptr`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseInt<T> {
    /// Parsed value, or zero when no digits were converted.
    pub value: T,
    /// Offset of the first unconverted byte. It is zero when no conversion occurred.
    pub end: usize,
    /// The magnitude exceeded the target type's range.
    pub overflow: bool,
    /// `radix` was not zero or in the range 2 through 36.
    pub invalid_base: bool,
}

struct Core {
    magnitude: u128,
    end: usize,
    negative: bool,
    overflow: bool,
    invalid_base: bool,
}

fn digit(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'z' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'Z' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

fn whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

fn c_len(input: &[u8]) -> usize {
    input
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(input.len())
}

fn parse_core(input: &[u8], radix: u32, positive_limit: u128, negative_limit: u128) -> Core {
    if radix != 0 && !(2..=36).contains(&radix) {
        return Core {
            magnitude: 0,
            end: 0,
            negative: false,
            overflow: false,
            invalid_base: true,
        };
    }

    let input = &input[..c_len(input)];
    let mut pos = 0;
    while pos < input.len() && whitespace(input[pos]) {
        pos += 1;
    }
    let negative = match input.get(pos) {
        Some(b'-') => {
            pos += 1;
            true
        }
        Some(b'+') => {
            pos += 1;
            false
        }
        _ => false,
    };

    let mut base = radix;
    if base == 0 {
        if input.get(pos) == Some(&b'0') {
            if matches!(input.get(pos + 1), Some(b'x' | b'X')) {
                base = 16;
                pos += 2;
            } else {
                base = 8;
            }
        } else {
            base = 10;
        }
    } else if base == 16
        && input.get(pos) == Some(&b'0')
        && matches!(input.get(pos + 1), Some(b'x' | b'X'))
    {
        pos += 2;
    }

    let limit = if negative {
        negative_limit
    } else {
        positive_limit
    };
    let mut magnitude = 0u128;
    let mut overflow = false;
    let first_digit = pos;
    while let Some(&byte) = input.get(pos) {
        let Some(value) = digit(byte) else { break };
        if value >= base {
            break;
        }
        if magnitude > limit / u128::from(base)
            || (magnitude == limit / u128::from(base)
                && u128::from(value) > limit % u128::from(base))
        {
            magnitude = limit;
            overflow = true;
        } else if !overflow {
            magnitude = magnitude * u128::from(base) + u128::from(value);
        }
        pos += 1;
    }

    if pos == first_digit {
        Core {
            magnitude: 0,
            end: 0,
            negative,
            overflow: false,
            invalid_base: false,
        }
    } else {
        Core {
            magnitude,
            end: pos,
            negative,
            overflow,
            invalid_base: false,
        }
    }
}

fn signed_value(core: &Core, bits: u32) -> i64 {
    if core.overflow {
        return if core.negative {
            -(1i64 << (bits - 1))
        } else {
            (1i64 << (bits - 1)) - 1
        };
    }
    if core.negative {
        if core.magnitude == (1u128 << (bits - 1)) {
            -(1i64 << (bits - 1))
        } else {
            -(core.magnitude as i64)
        }
    } else {
        core.magnitude as i64
    }
}

fn unsigned_value(core: &Core, bits: u32) -> u64 {
    let mask = (1u128 << bits) - 1;
    if core.overflow {
        mask as u64
    } else if core.negative {
        (0u128.wrapping_sub(core.magnitude) & mask) as u64
    } else {
        core.magnitude as u64
    }
}

/// Parses a 32-bit Windows `long`, like `strtol`.
pub fn strtol(input: &[u8], radix: u32) -> ParseInt<i32> {
    let core = parse_core(input, radix, i32::MAX as u128, (i32::MAX as u128) + 1);
    ParseInt {
        value: signed_value(&core, 32) as i32,
        end: core.end,
        overflow: core.overflow,
        invalid_base: core.invalid_base,
    }
}

/// Parses a 32-bit Windows `unsigned long`, like `strtoul`.
pub fn strtoul(input: &[u8], radix: u32) -> ParseInt<u32> {
    let max = u32::MAX as u128;
    let core = parse_core(input, radix, max, max);
    ParseInt {
        value: unsigned_value(&core, 32) as u32,
        end: core.end,
        overflow: core.overflow,
        invalid_base: core.invalid_base,
    }
}

/// Parses a 64-bit signed integer, like `strtoll`.
pub fn strtoll(input: &[u8], radix: u32) -> ParseInt<i64> {
    let core = parse_core(input, radix, i64::MAX as u128, (i64::MAX as u128) + 1);
    ParseInt {
        value: signed_value(&core, 64),
        end: core.end,
        overflow: core.overflow,
        invalid_base: core.invalid_base,
    }
}

/// Parses a 64-bit unsigned integer, like `strtoull`.
pub fn strtoull(input: &[u8], radix: u32) -> ParseInt<u64> {
    let max = u64::MAX as u128;
    let core = parse_core(input, radix, max, max);
    ParseInt {
        value: unsigned_value(&core, 64),
        end: core.end,
        overflow: core.overflow,
        invalid_base: core.invalid_base,
    }
}

/// Parses a decimal 32-bit integer, like `atoi`.
///
/// C leaves an out-of-range `atoi` result undefined. This function returns the saturated value.
pub fn atoi(input: &[u8]) -> i32 {
    strtol(input, 10).value
}

/// Parses a decimal Windows `long`, like `atol`.
pub fn atol(input: &[u8]) -> i32 {
    strtol(input, 10).value
}

/// Parses a decimal 64-bit integer, like `_atoi64`.
pub fn atoi64(input: &[u8]) -> i64 {
    strtoll(input, 10).value
}
