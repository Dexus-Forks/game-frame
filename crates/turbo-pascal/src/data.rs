//! Reading and writing Borland Pascal data as it is laid out in memory and in files written with
//! `BlockWrite`: little-endian integers, a one-byte `Boolean`, the 6-byte `Real`, and
//! `string[N]` as a length byte followed by N bytes.
//!
//! GameFrame's `tools/dos/tdinfo.py rust-reader` generates this code for a project; this
//! module is the same reader and writer as a dependency.

use crate::real48::Real48;

/// A Pascal string[N]: `len` used bytes out of a fixed capacity. Bytes after `len` are kept, as
/// the original keeps them in memory.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PStr<const N: usize> {
    pub len: u8,
    pub bytes: [u8; N],
}

impl<const N: usize> PStr<N> {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..(self.len as usize).min(N)]
    }
    /// The text, reading each byte as one character (code page 437 is not mapped).
    pub fn to_string_lossy(&self) -> String {
        self.as_bytes().iter().map(|&b| b as char).collect()
    }
}

impl<const N: usize> Default for PStr<N> {
    fn default() -> Self {
        PStr { len: 0, bytes: [0; N] }
    }
}

impl<const N: usize> std::fmt::Debug for PStr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_string_lossy())
    }
}

/// An error: the data ended before a read finished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortRead {
    pub at: usize,
    pub wanted: usize,
}

/// A cursor over bytes laid out as Borland Pascal stores them.
pub struct PascalReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> PascalReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        PascalReader { data, pos: 0 }
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }
    pub fn seek(&mut self, pos: usize) {
        self.pos = pos;
    }
    pub fn bytes(&mut self, n: usize) -> Result<&'a [u8], ShortRead> {
        if self.pos + n > self.data.len() {
            return Err(ShortRead { at: self.pos, wanted: n });
        }
        let out = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(out)
    }
    pub fn array<const N: usize>(&mut self) -> Result<[u8; N], ShortRead> {
        let mut out = [0u8; N];
        out.copy_from_slice(self.bytes(N)?);
        Ok(out)
    }
    pub fn u8(&mut self) -> Result<u8, ShortRead> {
        Ok(self.bytes(1)?[0])
    }
    pub fn i8(&mut self) -> Result<i8, ShortRead> {
        Ok(self.u8()? as i8)
    }
    pub fn u16(&mut self) -> Result<u16, ShortRead> {
        Ok(u16::from_le_bytes(self.array()?))
    }
    pub fn i16(&mut self) -> Result<i16, ShortRead> {
        Ok(i16::from_le_bytes(self.array()?))
    }
    pub fn u32(&mut self) -> Result<u32, ShortRead> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    pub fn i32(&mut self) -> Result<i32, ShortRead> {
        Ok(i32::from_le_bytes(self.array()?))
    }
    pub fn f32(&mut self) -> Result<f32, ShortRead> {
        Ok(f32::from_le_bytes(self.array()?))
    }
    pub fn f64(&mut self) -> Result<f64, ShortRead> {
        Ok(f64::from_le_bytes(self.array()?))
    }
    /// Pascal Boolean: one byte; any value other than 0 is true (as `if b then` tests it).
    pub fn boolean(&mut self) -> Result<bool, ShortRead> {
        Ok(self.u8()? != 0)
    }
    pub fn real48(&mut self) -> Result<Real48, ShortRead> {
        Ok(Real48(self.array()?))
    }
    pub fn pstr<const N: usize>(&mut self) -> Result<PStr<N>, ShortRead> {
        let len = self.u8()?;
        Ok(PStr { len, bytes: self.array()? })
    }
    /// A fixed array of N values, each read by `f` (Pascal `array[a..b] of T`).
    pub fn arr<T, const N: usize>(
        &mut self,
        mut f: impl FnMut(&mut Self) -> Result<T, ShortRead>,
    ) -> Result<[T; N], ShortRead> {
        let mut err = None;
        let a: [Option<T>; N] = std::array::from_fn(|_| {
            if err.is_some() {
                return None;
            }
            match f(self) {
                Ok(v) => Some(v),
                Err(e) => {
                    err = Some(e);
                    None
                }
            }
        });
        match err {
            Some(e) => Err(e),
            None => Ok(a.map(|x| x.expect("read"))),
        }
    }
}

/// Writes values back in the same layout (the inverse of `PascalReader`). Writing past the
/// end of the buffer is ignored, so a 64 KB segment image can be written field by field.
pub struct PascalWriter<'a> {
    data: &'a mut [u8],
    pos: usize,
}

impl<'a> PascalWriter<'a> {
    pub fn new(data: &'a mut [u8]) -> Self {
        PascalWriter { data, pos: 0 }
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
    pub fn seek(&mut self, pos: usize) {
        self.pos = pos;
    }
    pub fn bytes(&mut self, b: &[u8]) {
        for &x in b {
            if let Some(d) = self.data.get_mut(self.pos) {
                *d = x;
            }
            self.pos += 1;
        }
    }
    pub fn u8(&mut self, v: u8) {
        self.bytes(&[v]);
    }
    pub fn i8(&mut self, v: i8) {
        self.bytes(&[v as u8]);
    }
    pub fn u16(&mut self, v: u16) {
        self.bytes(&v.to_le_bytes());
    }
    pub fn i16(&mut self, v: i16) {
        self.bytes(&v.to_le_bytes());
    }
    pub fn u32(&mut self, v: u32) {
        self.bytes(&v.to_le_bytes());
    }
    pub fn i32(&mut self, v: i32) {
        self.bytes(&v.to_le_bytes());
    }
    pub fn f32(&mut self, v: f32) {
        self.bytes(&v.to_le_bytes());
    }
    pub fn f64(&mut self, v: f64) {
        self.bytes(&v.to_le_bytes());
    }
    /// Pascal Boolean as one byte: 1 for true. (Globals keep Booleans as u8 to stay exact.)
    pub fn boolean(&mut self, v: bool) {
        self.u8(v as u8);
    }
    pub fn real48(&mut self, v: Real48) {
        self.bytes(&v.0);
    }
    pub fn pstr<const N: usize>(&mut self, v: &PStr<N>) {
        self.u8(v.len);
        self.bytes(&v.bytes);
    }
}
