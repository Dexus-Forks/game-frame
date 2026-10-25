#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

//! Yamaha YMF262 (OPL3), built on GameFrame's YM3812 operator engine.
//!
//! The chip model supports both register banks, all 18 channels, six selectable four-operator
//! pairs, eight waveforms, four routed output channels, OPL2 compatibility mode, timer status and
//! indexed I/O ports. It returns sampled channel words and can encode native samples on the
//! YMF262's serial DAC pins. It does not model the YAC512's analog output stage.

use gameframe_opl2::Opl2;
pub use gameframe_opl2::SAMPLE_RATE;

/// YMF262 master clock used by the AdLib Gold and Sound Blaster 16 families.
pub const OPL3_MASTER_CLOCK_HZ: f64 = gameframe_opl2::CLOCK_HZ * 4.0;
/// Native output rate: one sample per 288 YMF262 master clocks.
pub const OPL3_SAMPLE_RATE: f64 = OPL3_MASTER_CLOCK_HZ / 288.0;
/// Minimum spacing between YMF262 register address/data writes, in master-clock cycles.
pub const REGISTER_WRITE_SPACING_CLOCKS: u64 = 32;
/// Number of φSY rising-edge slots in one YMF262 sample frame.
pub const DAC_SERIAL_CLOCKS_PER_FRAME: usize = 36;
/// Number of φSY slots assigned to each output channel on DOAB and DOCD.
pub const DAC_CHANNEL_SLOT_CLOCKS: usize = 18;
/// Number of serial data bits in each output-channel word.
pub const DAC_SERIAL_WORD_BITS: usize = 16;

/// A convenience stereo frame formed from native output channels A+C and B+D.
///
/// Actual cards can wire the four channel streams to their DACs and mixer differently. OPL2
/// compatibility mode returns the same mono sample in both fields.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct StereoSample {
    pub left: i16,
    pub right: i16,
}

/// One sampled word for each YMF262 output channel, before a card-level DAC or mixer.
///
/// The Yamaha datasheet calls these channels A, B, C and D. A/B share the DOAB serial pin and
/// C/D share DOCD. [`to_serial_frame`](Self::to_serial_frame) encodes these words on those pins.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct FourChannelSample {
    pub a: i16,
    pub b: i16,
    pub c: i16,
    pub d: i16,
}

impl FourChannelSample {
    /// Mix the four digital channels into the common A+C left, B+D right stereo layout.
    pub fn into_stereo(self) -> StereoSample {
        StereoSample {
            left: clip(self.a as i32 + self.c as i32),
            right: clip(self.b as i32 + self.d as i32),
        }
    }

    /// Encode channels A–D as one YMF262 serial DAC frame.
    pub fn to_serial_frame(self) -> Ymf262DacFrame {
        Ymf262DacFrame::from_channels(self)
    }
}

/// One YMF262 frame on its two serial DAC data pins and two sample/hold pins.
///
/// Each array entry represents one rising edge of φSY. DOAB carries A then B and DOCD carries C
/// then D. Each 18-clock slot has 16 LSB-first offset-binary data bits followed by two clocks
/// whose data level is unspecified. `None` marks those two clocks. SMPAC and SMPBD are high for
/// data bits 8–15 of their corresponding channel slots. This is the digital interface only; it
/// does not model edge delays, the YAC512 converter or analog sample/hold circuitry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ymf262DacFrame {
    pub doab: [Option<bool>; DAC_SERIAL_CLOCKS_PER_FRAME],
    pub docd: [Option<bool>; DAC_SERIAL_CLOCKS_PER_FRAME],
    pub smpac: [bool; DAC_SERIAL_CLOCKS_PER_FRAME],
    pub smpbd: [bool; DAC_SERIAL_CLOCKS_PER_FRAME],
}

impl Default for Ymf262DacFrame {
    fn default() -> Self {
        Self {
            doab: [None; DAC_SERIAL_CLOCKS_PER_FRAME],
            docd: [None; DAC_SERIAL_CLOCKS_PER_FRAME],
            smpac: [false; DAC_SERIAL_CLOCKS_PER_FRAME],
            smpbd: [false; DAC_SERIAL_CLOCKS_PER_FRAME],
        }
    }
}

impl Ymf262DacFrame {
    /// Convert four signed software sample words to the YMF262's serial pin representation.
    pub fn from_channels(channels: FourChannelSample) -> Self {
        let mut frame = Self::default();
        let words = [channels.a, channels.b, channels.c, channels.d];

        for (channel, sample) in words.into_iter().enumerate() {
            let slot = channel % 2;
            let base = slot * DAC_CHANNEL_SLOT_CLOCKS;
            let word = (sample as u16) ^ 0x8000;

            for bit in 0..DAC_SERIAL_WORD_BITS {
                let value = Some(word & (1u16 << bit) != 0);
                if channel < 2 {
                    frame.doab[base + bit] = value;
                } else {
                    frame.docd[base + bit] = value;
                }
                if (8..16).contains(&bit) {
                    if slot == 0 {
                        frame.smpac[base + bit] = true;
                    } else {
                        frame.smpbd[base + bit] = true;
                    }
                }
            }
        }

        frame
    }
}

/// Output form selected by the YMF262 mode register for one sample frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opl3Output {
    /// YM3812-compatible mono output. The convenience stereo adapter duplicates this sample.
    CompatibilityMono(i16),
    /// Four separately routed native-mode output channels.
    Native(FourChannelSample),
}

/// A timestamped port write arrived before the YMF262's required access spacing elapsed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortAccessError {
    pub wait_clocks: u8,
}

impl Default for Opl3Output {
    fn default() -> Self {
        Self::CompatibilityMono(0)
    }
}

impl Opl3Output {
    /// Convert this chip output to the crate's common stereo convenience layout.
    pub fn into_stereo(self) -> StereoSample {
        match self {
            Self::CompatibilityMono(sample) => StereoSample {
                left: sample,
                right: sample,
            },
            Self::Native(sample) => sample.into_stereo(),
        }
    }
}

/// One YMF262 with its two register banks and four digital output channels.
pub struct Opl3 {
    banks: [Opl2; 2],
    output_route: [[u8; 9]; 2],
    opl3_mode: bool,
    four_op_mask: u8,
}

/// YMF262's indexed I/O port pairs around an [`Opl3`] chip.
///
/// Bank zero uses `base_port` for its register index and `base_port + 1` for data. Bank one
/// uses `base_port + 2` and `base_port + 3`. Status is readable only at `base_port`; the
/// secondary index and both data ports are write-only.
/// `write_port_at` enforces the datasheet's 32-master-clock spacing between writes. The simpler
/// `write_port` method applies writes immediately.
pub struct Ymf262Ports {
    chip: Opl3,
    base_port: u16,
    address: [u8; 2],
    last_write_clock: Option<u64>,
}

impl Ymf262Ports {
    /// Create a YMF262 port interface with a chip in its reset state.
    pub fn new(base_port: u16) -> Self {
        Self {
            chip: Opl3::new(),
            base_port,
            address: [0; 2],
            last_write_clock: None,
        }
    }

    /// Handle an I/O port write. Returns `true` for one of the four YMF262 ports.
    pub fn write_port(&mut self, port: u16, value: u8) -> bool {
        let Some(offset) = self.port_offset(port) else {
            return false;
        };
        if offset > 3 {
            return false;
        }

        self.last_write_clock = None;
        self.write_offset(offset, value);
        true
    }

    /// Handle an I/O port write at an absolute YMF262 master-clock timestamp.
    ///
    /// Returns `Ok(false)` if `port` is outside the four YMF262 ports. Returns
    /// `Err(PortAccessError)` if fewer than [`REGISTER_WRITE_SPACING_CLOCKS`] have elapsed since
    /// the previous write. Status reads do not change this timestamp.
    pub fn write_port_at(
        &mut self,
        port: u16,
        value: u8,
        master_clock: u64,
    ) -> Result<bool, PortAccessError> {
        let Some(offset) = self.port_offset(port) else {
            return Ok(false);
        };
        if offset > 3 {
            return Ok(false);
        }

        if let Some(previous) = self.last_write_clock {
            let elapsed = master_clock.saturating_sub(previous);
            if elapsed < REGISTER_WRITE_SPACING_CLOCKS {
                return Err(PortAccessError {
                    wait_clocks: (REGISTER_WRITE_SPACING_CLOCKS - elapsed) as u8,
                });
            }
        }

        self.write_offset(offset, value);
        self.last_write_clock = Some(master_clock);
        Ok(true)
    }

    fn port_offset(&self, port: u16) -> Option<u16> {
        port.checked_sub(self.base_port)
    }

    fn write_offset(&mut self, offset: u16, value: u8) {
        let bank = (offset / 2) as usize;
        if (offset & 1) == 0 {
            self.address[bank] = value;
        } else {
            self.chip
                .write(((bank as u16) << 8) | self.address[bank] as u16, value);
        }
    }

    /// Return chip status from the primary index port. Other ports return `None`.
    pub fn read_port(&self, port: u16) -> Option<u8> {
        let offset = port.checked_sub(self.base_port)?;
        if offset == 0 {
            Some(self.chip.status())
        } else {
            None
        }
    }

    /// Read chip status without changing its state.
    pub fn chip(&self) -> &Opl3 {
        &self.chip
    }

    /// Access the chip for rendering generated audio or direct register writes.
    pub fn chip_mut(&mut self) -> &mut Opl3 {
        &mut self.chip
    }
}

impl Default for Opl3 {
    fn default() -> Self {
        Self::new()
    }
}

impl Opl3 {
    /// Create a YMF262 in its reset state, which is YM3812-compatible mode.
    pub fn new() -> Self {
        let mut bank_zero = Opl2::new();
        let mut bank_one = Opl2::new();
        bank_zero.set_csm_supported(false);
        bank_one.set_csm_supported(false);
        bank_zero.write(0x01, 0x20);
        bank_one.write(0x01, 0x20);
        Self {
            banks: [bank_zero, bank_one],
            output_route: [[0; 9]; 2],
            opl3_mode: false,
            four_op_mask: 0,
        }
    }

    /// Read YMF262 status. Timer flags occupy bits 5–7; bits 0–4 are zero in either synthesis
    /// mode, so the chip remains distinguishable from a YM3812 in compatibility mode.
    pub fn status(&self) -> u8 {
        self.banks[0].status() & 0xE0
    }

    /// Write a YMF262 register selected by its full 9-bit address.
    ///
    /// Addresses `0x000..=0x0ff` select bank zero. Addresses `0x100..=0x1ff` select bank one.
    /// Register `0x105` enables native OPL3 features when bit 0 is set. Register `0x104` selects
    /// up to six four-operator pairs when native mode is enabled.
    pub fn write(&mut self, reg: u16, value: u8) {
        if reg > 0x1FF {
            return;
        }

        match reg {
            0x105 => {
                self.opl3_mode = value & 1 != 0;
                self.update_waveform_modes();
                return;
            }
            0x104 => {
                if !self.opl3_mode {
                    return;
                }
                self.four_op_mask = value & 0x3F;
                self.update_waveform_modes();
                return;
            }
            _ => {}
        }

        let bank = (reg >> 8) as usize;
        let offset = reg as u8;

        if bank == 1 && !self.opl3_mode {
            return;
        }

        if bank == 1 && !is_voice_register(offset) {
            return;
        }

        if bank == 0 && offset == 0x08 {
            // This shared control register supplies note-select to both operator banks. The
            // YMF262 does not implement the YM3812's CSM mode, so bit 7 is ignored by each bank.
            self.banks[0].write(offset, value);
            self.banks[1].write(offset, value);
            return;
        }

        if bank == 0 && offset == 0x01 {
            // YMF262 ignores register 01h's YM3812 waveform-select-enable bit. The four
            // compatible waveforms remain selectable in compatibility mode; native mode adds
            // four more. Keep the shared YM3812 engine's selector gate open and ignore test bits.
            self.banks[0].write(offset, 0x20);
            return;
        }

        if (0xC0..=0xC8).contains(&offset) {
            self.output_route[bank][(offset - 0xC0) as usize] = value;
        }

        self.banks[bank].write(offset, value);
    }

    fn update_waveform_modes(&mut self) {
        for bank in 0..2 {
            self.banks[bank].set_extended_waveforms(self.opl3_mode);
            self.banks[bank].write(0x01, 0x20);
            let pairs = if self.opl3_mode {
                (self.four_op_mask >> (bank * 3)) & 0x07
            } else {
                0
            };
            self.banks[bank].set_four_operator_pairs(pairs);
        }
    }

    /// Generate the chip's native output for one frame at [`OPL3_SAMPLE_RATE`].
    pub fn sample_output(&mut self) -> Opl3Output {
        let bank_zero = self.banks[0].sample_channels_with_four_operator_pairs();
        let bank_one = self.banks[1].sample_channels_with_four_operator_pairs();
        let channels = [bank_zero, bank_one];

        if !self.opl3_mode {
            return Opl3Output::CompatibilityMono(clip(channels[0].iter().sum()));
        }

        let mut outputs = [0i32; 4];
        for bank in 0..2 {
            for channel in 0..9 {
                let output = channels[bank][channel];
                let route = self.output_route[bank][channel];
                for (index, mask) in [0x10, 0x20, 0x40, 0x80].into_iter().enumerate() {
                    if route & mask != 0 {
                        outputs[index] += output;
                    }
                }
            }
        }

        Opl3Output::Native(FourChannelSample {
            a: clip(outputs[0]),
            b: clip(outputs[1]),
            c: clip(outputs[2]),
            d: clip(outputs[3]),
        })
    }

    /// Generate one stereo frame at [`OPL3_SAMPLE_RATE`] using A+C left and B+D right.
    pub fn sample(&mut self) -> StereoSample {
        self.sample_output().into_stereo()
    }

    /// Fill `out` with consecutive four-channel or compatibility-mono frames.
    pub fn generate_outputs(&mut self, out: &mut [Opl3Output]) {
        for frame in out {
            *frame = self.sample_output();
        }
    }

    /// Fill `out` with consecutive stereo frames using A+C left and B+D right.
    pub fn generate(&mut self, out: &mut [StereoSample]) {
        for frame in out {
            *frame = self.sample();
        }
    }

    /// Number of chip samples generated since reset.
    pub fn samples_generated(&self) -> u64 {
        self.banks[0].samples_generated()
    }
}

fn clip(sample: i32) -> i16 {
    sample.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(chip: &mut Opl3) {
        // Channel 0: a plain sine on the carrier, modulator silent, instant attack.
        for (register, value) in [
            (0x020, 0x21),
            (0x023, 0x21),
            (0x040, 0x3f),
            (0x043, 0x00),
            (0x060, 0xf0),
            (0x063, 0xf0),
            (0x080, 0x0f),
            (0x083, 0x0f),
            (0x0a0, 0x41),
            (0x0b0, 0x32),
        ] {
            chip.write(register, value);
        }
    }

    #[test]
    fn native_output_routes_channel_zero_to_the_selected_dac_word() {
        let mut compatibility = Opl3::new();
        let mut native = Opl3::new();
        tone(&mut compatibility);
        tone(&mut native);
        native.write(0x105, 0x01);
        native.write(0x0c0, 0x10); // output channel A only

        let mut saw_signal = false;
        for _ in 0..256 {
            let Opl3Output::CompatibilityMono(reference) = compatibility.sample_output() else {
                unreachable!("new chip starts in compatibility mode");
            };
            let Opl3Output::Native(channels) = native.sample_output() else {
                unreachable!("native mode was enabled");
            };
            assert_eq!(
                channels,
                FourChannelSample {
                    a: reference,
                    ..FourChannelSample::default()
                }
            );
            saw_signal |= reference != 0;
        }
        assert!(saw_signal);
    }

    #[test]
    fn four_operator_pair_mask_is_writable_only_in_native_mode() {
        let mut chip = Opl3::new();
        chip.write(0x104, 0x3f);
        assert_eq!(chip.four_op_mask, 0);

        chip.write(0x105, 0x01);
        chip.write(0x104, 0xff);
        assert_eq!(chip.four_op_mask, 0x3f);
    }
}

fn is_voice_register(offset: u8) -> bool {
    matches!(
        offset,
        0x20..=0x35
            | 0x40..=0x55
            | 0x60..=0x75
            | 0x80..=0x95
            | 0xA0..=0xA8
            | 0xB0..=0xB8
            | 0xC0..=0xC8
            | 0xE0..=0xF5
    )
}
