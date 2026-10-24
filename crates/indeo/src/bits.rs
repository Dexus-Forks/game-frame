//! Intel's bit order (`FUN_100208d0` and every reader under it): a field's first bit is the
//! lowest unread bit of the next byte, and a field of n bits is the next n bits with the first
//! one lowest. Reading past the end gives zero bits (Intel's reader would read whatever memory
//! follows the chunk).

#[derive(Clone)]
pub(super) struct Bits<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Bits<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// A reader at byte `offset` of `data`.
    pub fn at(data: &'a [u8], offset: usize) -> Self {
        Self { data, pos: 8 * offset }
    }

    pub fn bit(&mut self) -> u32 {
        let b = self.data.get(self.pos >> 3).map_or(0, |&b| (b >> (self.pos & 7)) & 1);
        self.pos += 1;
        b as u32
    }

    pub fn get(&mut self, n: u32) -> u32 {
        (0..n).fold(0, |v, i| v | self.bit() << i)
    }

    pub fn flag(&mut self) -> bool {
        self.bit() != 0
    }

    pub fn skip(&mut self, n: usize) {
        self.pos += n;
    }

    /// The bytes read so far, the last one counted whole.
    pub fn bytes_used(&self) -> usize {
        self.pos.div_ceil(8)
    }
}
