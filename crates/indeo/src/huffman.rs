//! Intel's codebooks (`FUN_100016e0` builds their lookup tables). A descriptor names a number of
//! rows and each row's bits. Row i's codes are i one bits, a zero bit (the last row has none),
//! then the row's bits with the most significant first; its symbols follow the rows before it,
//! one for each value of the row's bits. The descriptor of one row of 8 bits is Intel's special
//! case: its table is the identity, so the symbol is the 8 bits read in the stream's own order.

use super::bits::Bits;

#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct Codebook {
    rows: Vec<u8>,
    raw8: bool,
}

impl Codebook {
    /// From a descriptor (a count, then that many row sizes). Intel refuses a row of more
    /// than 8 bits and a longest code over 13 bits (`FUN_100016e0` returns -1).
    pub fn new(desc: &[u8]) -> Option<Self> {
        let n = *desc.first()? as usize;
        let rows: Vec<u8> = desc.get(1..1 + n)?.to_vec();
        if n == 0 || rows.iter().any(|&b| b > 8) {
            return None;
        }
        let longest = rows.iter().enumerate().map(|(i, &b)| b as usize + i + usize::from(i + 1 < n)).max()?;
        if longest > 13 {
            return None;
        }
        Some(Self { raw8: rows == [8], rows })
    }

    /// The code length of each symbol, as Intel's table at +0x200C holds it.
    pub fn len(&self, sym: u8) -> u32 {
        let mut base = 0usize;
        for (i, &b) in self.rows.iter().enumerate() {
            let count = 1usize << b;
            if (sym as usize) < base + count {
                return b as u32 + i as u32 + u32::from(i + 1 < self.rows.len());
            }
            base += count;
        }
        0
    }

    pub fn read(&self, bits: &mut Bits) -> u8 {
        if self.raw8 {
            return bits.get(8) as u8;
        }
        let n = self.rows.len();
        let mut base = 0u32;
        for (i, &b) in self.rows.iter().enumerate() {
            if i + 1 == n || bits.bit() == 0 {
                let v = (0..b).fold(0, |v, _| v << 1 | bits.bit());
                return (base + v).min(255) as u8;
            }
            base += 1 << b;
        }
        unreachable!()
    }
}
