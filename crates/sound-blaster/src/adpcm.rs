//! Creative ADPCM decoder translated from CT1741 DSP firmware 4.05.

/// Creative's compressed sample width. The 3-bit stream uses two full codes and one short tail
/// code in each byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdpcmMode {
    TwoBit,
    ThreeBit,
    FourBit,
}

impl AdpcmMode {
    pub const fn bits_per_sample(self) -> u8 {
        match self {
            Self::TwoBit => 2,
            Self::ThreeBit => 3,
            Self::FourBit => 4,
        }
    }

    pub const fn samples_per_byte(self) -> u8 {
        match self {
            Self::TwoBit => 4,
            Self::ThreeBit => 3,
            Self::FourBit => 2,
        }
    }

    pub(crate) const fn from_bits(bits: u8) -> Option<Self> {
        match bits {
            2 => Some(Self::TwoBit),
            3 => Some(Self::ThreeBit),
            4 => Some(Self::FourBit),
            _ => None,
        }
    }
}

/// Decoded unsigned 8-bit PCM samples from one compressed byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdpcmSamples {
    pub values: [u8; 4],
    pub count: u8,
}

/// Stateful Creative ADPCM predictor and step register.
///
/// The recurrence follows CT1741 firmware v4.05 helpers at CODE:1056 (2-bit), CODE:10ad
/// (4-bit), and CODE:10f8 (3-bit). Reference setup at CODE:0dce and CODE:0e52 loads the
/// predictor from the reference byte and sets the step to one. The initial no-reference state
/// before any reference byte is not established by this firmware trace; it starts at zero here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdpcmDecoder {
    predictor: u8,
    step: u8,
}

impl Default for AdpcmDecoder {
    fn default() -> Self {
        Self {
            predictor: 0,
            step: 1,
        }
    }
}

impl AdpcmDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the decoder's predictor state. A later reference byte replaces it and restores step 1.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Initialize the predictor from a command's reference byte and return that initial sample.
    pub fn set_reference(&mut self, value: u8) -> u8 {
        self.predictor = value;
        self.step = 1;
        value
    }

    /// Decode one compressed input byte, keeping predictor and step state for continuation blocks.
    pub fn decode_byte(&mut self, mode: AdpcmMode, value: u8) -> AdpcmSamples {
        match mode {
            AdpcmMode::TwoBit => {
                let mut values = [0; 4];
                for (index, shift) in [6, 4, 2, 0].into_iter().enumerate() {
                    let code = (value >> shift) & 0x03;
                    values[index] = self.decode_code(mode, code >> 1, code & 1);
                }
                AdpcmSamples { values, count: 4 }
            }
            AdpcmMode::ThreeBit => {
                let codes = [
                    ((value >> 5) & 0x07, 3),
                    ((value >> 2) & 0x07, 3),
                    (value & 0x03, 2),
                ];
                let mut values = [0; 4];
                for (index, (code, width)) in codes.into_iter().enumerate() {
                    let magnitude_mask = (1 << (width - 1)) - 1;
                    let sign = (code >> (width - 1)) & 1;
                    values[index] = self.decode_code(mode, sign, code & magnitude_mask);
                }
                AdpcmSamples { values, count: 3 }
            }
            AdpcmMode::FourBit => {
                let mut values = [0; 4];
                for (index, shift) in [4, 0].into_iter().enumerate() {
                    let code = (value >> shift) & 0x0f;
                    values[index] = self.decode_code(mode, code >> 3, code & 0x07);
                }
                AdpcmSamples { values, count: 2 }
            }
        }
    }

    pub const fn predictor(&self) -> u8 {
        self.predictor
    }

    pub const fn step(&self) -> u8 {
        self.step
    }

    fn decode_code(&mut self, mode: AdpcmMode, sign: u8, magnitude: u8) -> u8 {
        let half_step = self.step >> 1;
        let delta = match mode {
            AdpcmMode::TwoBit => {
                if magnitude == 0 {
                    half_step
                } else {
                    self.step + half_step
                }
            }
            AdpcmMode::ThreeBit | AdpcmMode::FourBit => {
                magnitude.wrapping_mul(self.step).wrapping_add(half_step)
            }
        };

        self.predictor = if sign == 0 {
            self.predictor.saturating_add(delta)
        } else {
            self.predictor.saturating_sub(delta)
        };
        self.update_step(mode, magnitude);
        self.predictor
    }

    fn update_step(&mut self, mode: AdpcmMode, magnitude: u8) {
        match mode {
            AdpcmMode::TwoBit => {
                if magnitude == 0 {
                    self.step = (self.step >> 1).max(1);
                } else if self.step != 0x20 {
                    self.step = self.step.saturating_mul(2);
                }
            }
            AdpcmMode::ThreeBit => {
                if magnitude == 0 {
                    self.step = (self.step >> 1).max(1);
                } else if magnitude != 3 {
                    self.step = magnitude;
                } else if self.step == 0x10 {
                    self.step = 3;
                } else {
                    self.step = self.step.saturating_mul(2);
                }
            }
            AdpcmMode::FourBit => {
                if magnitude == 0 {
                    self.step = (self.step >> 1).max(1);
                } else if magnitude >= 5 {
                    let doubled = self.step.saturating_mul(2);
                    self.step = if doubled == 0x10 { 8 } else { doubled };
                }
            }
        }
    }
}
