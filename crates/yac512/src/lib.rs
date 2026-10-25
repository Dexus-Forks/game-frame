#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

//! Yamaha YAC512's serial input and digital sample-latch interface.
//!
//! This models the 16-bit LSB-first input stream, FSEL data format and channel latches selected
//! by SMP1 and SMP2. It does not model the floating-point DAC's analog conversion, analog
//! switches, sample-and-hold capacitors, voltage levels or board wiring.

/// Number of serial data bits shifted before a YAC512 channel latch in normal operation.
pub const SERIAL_WORD_BITS: usize = 16;

/// Digital representation selected by the YAC512's FSEL input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputFormat {
    /// FSEL high: unsigned offset-binary input, where `0x8000` represents zero.
    #[default]
    OffsetBinary,
    /// FSEL low: signed two's-complement input.
    TwosComplement,
}

impl InputFormat {
    /// Decode the logic level on FSEL. High selects offset binary; low selects two's complement.
    pub const fn from_fsel(high: bool) -> Self {
        if high {
            Self::OffsetBinary
        } else {
            Self::TwosComplement
        }
    }
}

/// One 16-bit word captured at a YAC512 sample-strobe falling edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LatchedWord {
    /// The input bits as they appeared on DIN, in the transmitted word's numeric order.
    pub raw: u16,
    /// The FSEL format in effect when the word was latched.
    pub format: InputFormat,
}

impl LatchedWord {
    /// Interpret the captured input encoding as a signed 16-bit value.
    ///
    /// This decodes the input format only. It does not reproduce the YAC512's floating DAC
    /// mantissa/exponent conversion or its analog output voltage.
    pub const fn signed_input(self) -> i16 {
        match self.format {
            InputFormat::OffsetBinary => (self.raw ^ 0x8000) as i16,
            InputFormat::TwosComplement => self.raw as i16,
        }
    }
}

/// Which YAC512 sample input changed from high to low.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SamplePin {
    /// Falling SMP1 latches CH2 and samples CH1.
    Smp1,
    /// Falling SMP2 latches CH1 and samples CH2.
    Smp2,
}

/// Report of channel input words captured by one sample-pin update.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LatchEvents {
    /// SMP2 fell, so the current serial word was latched for channel 1.
    pub channel1_latched: bool,
    /// SMP1 fell, so the current serial word was latched for channel 2.
    pub channel2_latched: bool,
}

/// Digital input words currently held by the YAC512 model.
///
/// A channel remains `None` until its corresponding sample pin has fallen for the first time.
/// Later captures replace that channel's word and leave the other channel unchanged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LatchedChannels {
    pub channel1: Option<LatchedWord>,
    pub channel2: Option<LatchedWord>,
}

/// YAC512 serial receiver and channel input latches.
pub struct Yac512 {
    serial_word: u16,
    format: InputFormat,
    smp1_high: bool,
    smp2_high: bool,
    channels: LatchedChannels,
}

impl Default for Yac512 {
    fn default() -> Self {
        Self::new()
    }
}

impl Yac512 {
    /// Create a receiver with FSEL high, which is the offset-binary wiring used by YMF262.
    pub const fn new() -> Self {
        Self {
            serial_word: 0,
            format: InputFormat::OffsetBinary,
            smp1_high: false,
            smp2_high: false,
            channels: LatchedChannels {
                channel1: None,
                channel2: None,
            },
        }
    }

    /// Set FSEL. The selected format is recorded with each word when a sample pin falls.
    pub fn set_fsel(&mut self, high: bool) {
        self.format = InputFormat::from_fsel(high);
    }

    /// Return the current input format selected by FSEL.
    pub const fn input_format(&self) -> InputFormat {
        self.format
    }

    /// Sample DIN on one rising edge of CLK.
    ///
    /// YAC512 consumes the serial input least-significant bit first. Call this once for every
    /// clock rising edge, including the two unspecified data clocks between words in a YMF262
    /// output frame. The caller supplies the actual DIN level for those clocks.
    pub fn clock_rising_edge(&mut self, din: bool) {
        self.serial_word = (self.serial_word >> 1) | ((din as u16) << 15);
    }

    /// Update one SMP input and capture the shift register if that pin falls.
    pub fn set_sample_pin(&mut self, pin: SamplePin, high: bool) -> LatchEvents {
        let (smp1_high, smp2_high) = match pin {
            SamplePin::Smp1 => (high, self.smp2_high),
            SamplePin::Smp2 => (self.smp1_high, high),
        };
        self.set_sample_pins(smp1_high, smp2_high)
    }

    /// Update SMP1 and SMP2 levels and capture the shift register on each falling edge.
    ///
    /// SMP1 falling latches channel 2. SMP2 falling latches channel 1. Words remain held in
    /// [`channels`](Self::channels) until that channel is latched again. If both pins fall in
    /// one update, both channels capture the current word.
    pub fn set_sample_pins(&mut self, smp1_high: bool, smp2_high: bool) -> LatchEvents {
        let smp1_fell = self.smp1_high && !smp1_high;
        let smp2_fell = self.smp2_high && !smp2_high;
        let word = LatchedWord {
            raw: self.serial_word,
            format: self.format,
        };

        self.smp1_high = smp1_high;
        self.smp2_high = smp2_high;

        if smp1_fell {
            self.channels.channel2 = Some(word);
        }
        if smp2_fell {
            self.channels.channel1 = Some(word);
        }

        LatchEvents {
            channel1_latched: smp2_fell,
            channel2_latched: smp1_fell,
        }
    }

    /// Return the currently held digital input words for both channels.
    pub const fn channels(&self) -> LatchedChannels {
        self.channels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shift_word(dac: &mut Yac512, word: u16) {
        for bit in 0..SERIAL_WORD_BITS {
            dac.clock_rising_edge(word & (1 << bit) != 0);
        }
    }

    #[test]
    fn serial_input_is_lsb_first_and_strobes_latch_the_documented_channel() {
        let mut dac = Yac512::new();
        shift_word(&mut dac, 0x8001);

        dac.set_sample_pins(true, true);
        let events = dac.set_sample_pins(false, true);
        assert_eq!(
            events,
            LatchEvents {
                channel2_latched: true,
                ..LatchEvents::default()
            }
        );
        assert_eq!(dac.channels().channel2.unwrap().raw, 0x8001);
        assert_eq!(dac.channels().channel1, None);

        shift_word(&mut dac, 0x7ffe);
        dac.set_sample_pins(false, true);
        let events = dac.set_sample_pins(false, false);
        assert_eq!(
            events,
            LatchEvents {
                channel1_latched: true,
                ..LatchEvents::default()
            }
        );
        assert_eq!(dac.channels().channel1.unwrap().raw, 0x7ffe);
        assert_eq!(dac.channels().channel2.unwrap().raw, 0x8001);
    }

    #[test]
    fn fsel_format_is_captured_with_each_latched_word() {
        let mut dac = Yac512::new();
        shift_word(&mut dac, 0x8000);
        dac.set_sample_pins(false, true);
        dac.set_sample_pins(false, false);
        assert_eq!(dac.channels().channel1.unwrap().signed_input(), 0);

        dac.set_fsel(false);
        shift_word(&mut dac, 0xffff);
        dac.set_sample_pins(true, false);
        dac.set_sample_pins(false, false);
        let channel2 = dac.channels().channel2.unwrap();
        assert_eq!(channel2.format, InputFormat::TwosComplement);
        assert_eq!(channel2.signed_input(), -1);
        assert_eq!(
            dac.channels().channel1.unwrap().format,
            InputFormat::OffsetBinary
        );
    }

    #[test]
    fn latch_requires_a_high_to_low_transition() {
        let mut dac = Yac512::new();
        shift_word(&mut dac, 0x1234);
        assert!(!dac.set_sample_pins(false, true).channel1_latched);
        assert!(!dac.set_sample_pins(false, true).channel1_latched);
        assert!(dac.set_sample_pins(false, false).channel1_latched);
        assert!(!dac.set_sample_pins(false, false).channel1_latched);
        assert_eq!(dac.channels().channel1.unwrap().raw, 0x1234);
    }
}
