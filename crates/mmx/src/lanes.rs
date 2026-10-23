//! The 64-bit register and its lane views.

/// One MMX register (or 64-bit memory operand).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Mm(pub u64);

impl Mm {
    /// `movq`: all 64 bits.
    pub fn q(self) -> u64 {
        self.0
    }

    /// `movd mm, r32`: the low 32 bits set, the high 32 cleared.
    pub fn from_d(v: u32) -> Mm {
        Mm(v as u64)
    }

    /// `movd r32, mm`: the low 32 bits.
    pub fn low_d(self) -> u32 {
        self.0 as u32
    }

    pub fn bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    pub fn from_bytes(b: [u8; 8]) -> Mm {
        Mm(u64::from_le_bytes(b))
    }

    pub fn words(self) -> [u16; 4] {
        let b = self.bytes();
        [0, 1, 2, 3].map(|i| u16::from_le_bytes([b[2 * i], b[2 * i + 1]]))
    }

    pub fn from_words(w: [u16; 4]) -> Mm {
        Mm(w.iter().enumerate().fold(0, |acc, (i, &v)| acc | (v as u64) << (16 * i)))
    }

    pub fn swords(self) -> [i16; 4] {
        self.words().map(|v| v as i16)
    }

    pub fn from_swords(w: [i16; 4]) -> Mm {
        Mm::from_words(w.map(|v| v as u16))
    }

    pub fn dwords(self) -> [u32; 2] {
        [self.0 as u32, (self.0 >> 32) as u32]
    }

    pub fn from_dwords(d: [u32; 2]) -> Mm {
        Mm(d[0] as u64 | (d[1] as u64) << 32)
    }

    pub fn sdwords(self) -> [i32; 2] {
        self.dwords().map(|v| v as i32)
    }

    pub fn from_sdwords(d: [i32; 2]) -> Mm {
        Mm::from_dwords(d.map(|v| v as u32))
    }

    /// One 16-bit value in all four words, as `movd` then `punpcklwd` twice builds it.
    pub fn splat_w(v: u16) -> Mm {
        Mm::from_words([v; 4])
    }
}

/// Apply `f` lane by lane to two registers' bytes.
pub(crate) fn zip_b(a: Mm, b: Mm, f: impl Fn(u8, u8) -> u8) -> Mm {
    let (x, y) = (a.bytes(), b.bytes());
    Mm::from_bytes([0, 1, 2, 3, 4, 5, 6, 7].map(|i| f(x[i], y[i])))
}

pub(crate) fn zip_w(a: Mm, b: Mm, f: impl Fn(u16, u16) -> u16) -> Mm {
    let (x, y) = (a.words(), b.words());
    Mm::from_words([0, 1, 2, 3].map(|i| f(x[i], y[i])))
}

pub(crate) fn zip_d(a: Mm, b: Mm, f: impl Fn(u32, u32) -> u32) -> Mm {
    let (x, y) = (a.dwords(), b.dwords());
    Mm::from_dwords([f(x[0], y[0]), f(x[1], y[1])])
}
