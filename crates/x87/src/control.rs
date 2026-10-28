//! The x87 control word's precision and rounding fields.

/// The precision control (bits 8 and 9 of the control word): how many significand bits an
/// `fadd`, `fsub`, `fmul`, `fdiv` or `fsqrt` result keeps. Loads, stores, `fist` and `frndint`
/// do not use it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Precision {
    /// 24 bits (field 0), as a game or Direct3D sets for speed.
    Single,
    /// 53 bits (field 2), as the MSVC runtime of Windows NT sets.
    Double,
    /// 64 bits (field 3), what `fninit` leaves: the default.
    Extended,
}

impl Precision {
    /// The number of significand bits kept.
    pub const fn bits(self) -> u32 {
        match self {
            Precision::Single => 24,
            Precision::Double => 53,
            Precision::Extended => 64,
        }
    }
}

/// The rounding control (bits 10 and 11 of the control word).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rounding {
    /// To the nearest, ties to even (field 0): the default.
    Nearest,
    /// Toward −∞ (field 1).
    Down,
    /// Toward +∞ (field 2).
    Up,
    /// Toward zero (field 3), the "chop" mode Pascal's `Trunc` and C's casts set for a moment.
    Zero,
}

/// The two fields of the control word that change results. The exception masks are not
/// modelled: an unmasked exception stops the original program, so a port rules those cases out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Control {
    pub precision: Precision,
    pub rounding: Rounding,
}

impl Control {
    /// `0x037f`, what `fninit` leaves: 64 bits, to nearest.
    pub const DEFAULT: Control = Control::from_word(0x037f);

    /// Read the precision and rounding fields of a control word, such as `0x0c7f` (24 bits,
    /// toward zero) or Delphi's `$1332` (64 bits, to nearest). The reserved precision field 1
    /// is read as 64 bits, as the emulator (QEMU's x87) does.
    pub const fn from_word(word: u16) -> Control {
        let precision = match (word >> 8) & 3 {
            0 => Precision::Single,
            2 => Precision::Double,
            _ => Precision::Extended,
        };
        let rounding = match (word >> 10) & 3 {
            0 => Rounding::Nearest,
            1 => Rounding::Down,
            2 => Rounding::Up,
            _ => Rounding::Zero,
        };
        Control {
            precision,
            rounding,
        }
    }

    /// The same precision with another rounding.
    pub const fn with_rounding(self, rounding: Rounding) -> Control {
        Control {
            precision: self.precision,
            rounding,
        }
    }
}

impl Default for Control {
    fn default() -> Control {
        Control::DEFAULT
    }
}
