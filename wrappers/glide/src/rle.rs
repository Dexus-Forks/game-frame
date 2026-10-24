//! The 16-bit run-length code of `guEncodeRLE16`. Plain Rust, tested on the host.
//!
//! Each word of the code is a run: the count in the high 16 bits and the pixel in the low 16.
//! The encoder copies the original's own loop, with its faults:
//!
//! - The pixel is sign-extended before it is stored and compared. A pixel of 0x8000 or more
//!   never starts a run longer than one, and its word has 0xffff in the count's place.
//! - Each run takes one more from the count of pixels left than it covers. So the code
//!   covers fewer pixels than the image has. Sixteen equal pixels give one run of 15.
//! - The count is a signed 16-bit value. A run of more than 32767 equal pixels wraps it.
//!
//! *Measured*: `encode` gives the same words and the same byte count as Glide 2.43's
//! `guEncodeRLE16` (the Win95 `glide2x.dll`, run under `tools/emu/pe32_emu.py`) on 2009
//! images: 2000 random ones and 9 chosen ones, on which the original reads only its image. On
//! 3 more chosen images the original reads past its image. There the words agree up to the
//! point where this encoder stops. The count-only form (a null destination) gave the same
//! byte counts.

/// `guEncodeRLE16`: the words for the first `count` pixels of `src` (`count` is width ×
/// height). The original returns the number of bytes, four per word.
///
/// Original: `guEncodeRLE16` (glide2x.dll 2.43, 0x10003210).
///
/// Deviation: when the original's count of pixels left goes below zero, it reads past the
/// image and writes past the code without end. This encoder stops there. A run that reaches
/// the end of `src` ends there. A run of more than 32767 wraps to a negative count, which
/// moves the original back before the image; this encoder stops at that read.
pub fn encode(src: &[u16], count: u32) -> Vec<u32> {
    let mut out = Vec::new();
    // The pixels left, less one: the original decrements before it tests.
    let mut left = count;
    let mut pos: isize = 0;
    loop {
        let before = left;
        left = left.wrapping_sub(1);
        if before == 0 {
            break;
        }
        if left == 0 {
            // Deviation: the original scans on past the image here, and then its count wraps.
            break;
        }
        let Some(&first) = at(src, pos) else { break };
        let pixel = first as i16 as i32;
        let mut run: i16 = 1;
        if left != 1 {
            let mut p = pos + 1;
            // Deviation: past the image the original compares with whatever memory follows.
            // This encoder ends the run there.
            while let Some(&next) = at(src, p) {
                if next as i32 != pixel {
                    break;
                }
                run = run.wrapping_add(1);
                p += 1;
                if run as i32 as u32 == left {
                    break;
                }
            }
        }
        out.push(((run as i32 as u32) << 16) | pixel as u32);
        pos += run as isize;
        left = left.wrapping_sub(run as i32 as u32);
    }
    out
}

fn at(src: &[u16], i: isize) -> Option<&u16> {
    usize::try_from(i).ok().and_then(|i| src.get(i))
}

/// The pixels a code holds. A word whose high half is 0xffff and whose pixel has its top bit
/// set is one pixel: the encoder's sign-extended pixel. Every other word is a run of its high
/// half. Glide has no decoder for this code; this one reads what `encode` writes.
pub fn decode(words: &[u32]) -> Vec<u16> {
    let mut out = Vec::new();
    for &w in words {
        let pixel = w as u16;
        let run = if w >> 16 == 0xffff && pixel & 0x8000 != 0 { 1 } else { (w >> 16) as usize };
        out.extend(std::iter::repeat_n(pixel, run));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_pixels_lose_the_last() {
        assert_eq!(encode(&[7; 16], 16), vec![(15 << 16) | 7]);
        assert_eq!(encode(&[5; 3], 3), vec![(2 << 16) | 5]);
    }

    #[test]
    fn distinct_pixels() {
        assert_eq!(encode(&[1, 2, 3, 4], 4), vec![0x1_0001, 0x1_0002]);
        assert_eq!(encode(&[1, 1, 2, 2, 3, 3, 4, 4], 8), vec![0x2_0001, 0x2_0002, 0x1_0003]);
    }

    #[test]
    fn high_pixels_are_sign_extended() {
        assert_eq!(encode(&[0x8000; 4], 4), vec![0xffff_8000, 0xffff_8000]);
        assert_eq!(decode(&[0xffff_8000, 0x0003_1234]), vec![0x8000, 0x1234, 0x1234, 0x1234]);
    }

    #[test]
    fn where_the_original_runs_away_this_stops() {
        assert_eq!(encode(&[1, 2, 3], 3), vec![0x1_0001]);
        assert_eq!(encode(&[9], 1), Vec::<u32>::new());
        assert_eq!(encode(&[], 0), Vec::<u32>::new());
    }

    #[test]
    fn decoded_code_is_a_prefix_of_the_image() {
        let img: Vec<u16> = (0..64u16).map(|i| [0x1111, 0x2222][(i / 5) as usize % 2]).collect();
        let back = decode(&encode(&img, 64));
        assert!(!back.is_empty());
        assert_eq!(back[..], img[..back.len()]);
    }
}
