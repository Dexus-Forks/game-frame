//! The byte-oriented MSVC CRT memory functions.
//!
//! These safe slice APIs reproduce the visible byte behavior of `memcpy`, `memmove`,
//! `memset`, `memcmp` and `memchr`. They check bounds and return an error where a C function
//! would read or write outside its object. `memmove_within` is the overlap-safe form for moving
//! bytes inside one buffer.
//!
//! Original symbols are exported by MSVCR90, MSVCR100, MSVCR110 and MSVCR120. Their decompiled
//! loops compare and copy bytes; these Rust functions model the result, not the runtime's
//! optimized instruction path.

use std::ops::Range;

/// A requested byte range does not fit in one of the supplied slices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferError {
    /// The source does not contain the requested number of bytes.
    SourceTooShort { needed: usize, available: usize },
    /// The destination does not contain the requested number of bytes.
    DestinationTooShort { needed: usize, available: usize },
    /// A source or destination range is outside the buffer.
    RangeOutOfBounds,
}

/// Copies `count` bytes, like `memcpy`, when both slices are long enough.
///
/// Safe Rust references cannot overlap. Use [`memmove_within`] to move bytes within one buffer.
pub fn memcpy(dst: &mut [u8], src: &[u8], count: usize) -> Result<(), BufferError> {
    if src.len() < count {
        return Err(BufferError::SourceTooShort {
            needed: count,
            available: src.len(),
        });
    }
    if dst.len() < count {
        return Err(BufferError::DestinationTooShort {
            needed: count,
            available: dst.len(),
        });
    }
    dst[..count].copy_from_slice(&src[..count]);
    Ok(())
}

/// Copies `count` bytes between disjoint slices, like `memmove`.
///
/// Safe Rust references cannot overlap. Use [`memmove_within`] to move bytes within one buffer.
pub fn memmove(dst: &mut [u8], src: &[u8], count: usize) -> Result<(), BufferError> {
    memcpy(dst, src, count)
}

/// Moves a range of bytes inside one buffer. The source and destination may overlap.
pub fn memmove_within(
    buffer: &mut [u8],
    source: Range<usize>,
    destination: usize,
) -> Result<(), BufferError> {
    let Some(bytes) = buffer.get(source.clone()) else {
        return Err(BufferError::RangeOutOfBounds);
    };
    let Some(end) = destination.checked_add(bytes.len()) else {
        return Err(BufferError::RangeOutOfBounds);
    };
    if end > buffer.len() {
        return Err(BufferError::RangeOutOfBounds);
    }
    buffer.copy_within(source, destination);
    Ok(())
}

/// Fills the first `count` bytes with `byte`, like `memset`.
pub fn memset(dst: &mut [u8], byte: u8, count: usize) -> Result<(), BufferError> {
    if dst.len() < count {
        return Err(BufferError::DestinationTooShort {
            needed: count,
            available: dst.len(),
        });
    }
    dst[..count].fill(byte);
    Ok(())
}

/// Compares `count` bytes as unsigned bytes, like `memcmp`.
///
/// The result is `-1`, `0` or `1`. C only specifies the result's sign, not its magnitude.
pub fn memcmp(a: &[u8], b: &[u8], count: usize) -> Result<i32, BufferError> {
    if a.len() < count {
        return Err(BufferError::SourceTooShort {
            needed: count,
            available: a.len(),
        });
    }
    if b.len() < count {
        return Err(BufferError::SourceTooShort {
            needed: count,
            available: b.len(),
        });
    }
    for (&left, &right) in a[..count].iter().zip(&b[..count]) {
        match left.cmp(&right) {
            std::cmp::Ordering::Less => return Ok(-1),
            std::cmp::Ordering::Greater => return Ok(1),
            std::cmp::Ordering::Equal => {}
        }
    }
    Ok(0)
}

/// Finds the first `byte` in the first `count` bytes, like `memchr`.
pub fn memchr(src: &[u8], byte: u8, count: usize) -> Result<Option<usize>, BufferError> {
    if src.len() < count {
        return Err(BufferError::SourceTooShort {
            needed: count,
            available: src.len(),
        });
    }
    Ok(src[..count].iter().position(|&value| value == byte))
}
