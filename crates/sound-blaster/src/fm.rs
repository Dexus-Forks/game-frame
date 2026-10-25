use gameframe_opl2::Opl2;
use gameframe_opl3::{Opl3, StereoSample, REGISTER_WRITE_SPACING_CLOCKS};

const LEGACY_FM_PORT: u16 = 0x388;
const OPL3_MASTER_CLOCK_HZ: u64 = 14_318_180;
const NS_PER_SECOND: u128 = 1_000_000_000;

/// FM chip fitted to a Sound Blaster profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FmChipKind {
    /// One YM3812, used by Sound Blaster, Sound Blaster 2.0 and compatible cards.
    Opl2,
    /// Two YM3812 chips with separate left and right outputs, used by the original Sound Blaster Pro.
    DualOpl2,
    /// One YMF262 with OPL2-compatible and native OPL3 register banks.
    Opl3,
}

/// One stereo frame from the FM synthesizer component.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FmSample {
    pub left: i16,
    pub right: i16,
}

enum Chip {
    Opl2(Opl2),
    DualOpl2([Opl2; 2]),
    Opl3(Opl3),
}

/// The FM synthesizer and its Sound Blaster register ports.
///
/// Sound Blaster cards expose FM registers at `base + 8` and at `388h`. OPL3 profiles map both
/// YMF262 address/data banks at `388h` through `38Bh`. Pro and SB16 profiles also expose their
/// native FM ports at `base + 0` through `base + 3`. The original Pro's two YM3812 chips use those
/// four ports as independent left and right chips. A YMF262 uses them as its two address/data
/// banks. Output samples are generated on demand by [`FmSynth::generate`].
pub struct FmSynth {
    kind: FmChipKind,
    chip: Chip,
    card_base: u16,
    card_ports_enabled: bool,
    address: [u8; 2],
    last_opl3_write_clock: Option<u64>,
}

impl FmSynth {
    pub(crate) fn new(kind: FmChipKind, card_base: u16, card_ports_enabled: bool) -> Self {
        let chip = match kind {
            FmChipKind::Opl2 => Chip::Opl2(Opl2::new()),
            FmChipKind::DualOpl2 => Chip::DualOpl2([Opl2::new(), Opl2::new()]),
            FmChipKind::Opl3 => Chip::Opl3(Opl3::new()),
        };
        Self {
            kind,
            chip,
            card_base,
            card_ports_enabled,
            address: [0; 2],
            last_opl3_write_clock: None,
        }
    }

    pub fn kind(&self) -> FmChipKind {
        self.kind
    }

    /// Read an OPL2 status port or a YMF262 status port. Data ports are write-only.
    pub fn read_port(&self, port: u16) -> Option<u8> {
        let bank = self.status_bank(port)?;
        Some(match &self.chip {
            Chip::Opl2(chip) => chip.status(),
            Chip::DualOpl2(chips) => chips[bank].status(),
            Chip::Opl3(chip) => chip.status(),
        })
    }

    /// Write an indexed FM port at monotonic virtual time. Returns `None` for an unmapped port,
    /// `Some(false)` when a YMF262 write is too early, and `Some(true)` after an accepted write.
    pub fn write_port_at(&mut self, port: u16, value: u8, at_ns: u64) -> Option<bool> {
        let (bank, data_port) = self.write_target(port)?;
        if matches!(self.chip, Chip::Opl3(_)) {
            let clock = (at_ns as u128 * OPL3_MASTER_CLOCK_HZ as u128 / NS_PER_SECOND) as u64;
            if let Some(previous) = self.last_opl3_write_clock {
                if clock.saturating_sub(previous) < REGISTER_WRITE_SPACING_CLOCKS {
                    return Some(false);
                }
            }
            self.last_opl3_write_clock = Some(clock);
        }

        match &mut self.chip {
            Chip::Opl2(chip) => write_opl2(chip, &mut self.address[0], data_port, value),
            Chip::DualOpl2(chips) => {
                write_opl2(&mut chips[bank], &mut self.address[bank], data_port, value)
            }
            Chip::Opl3(chip) => {
                if data_port {
                    chip.write(((bank as u16) << 8) | self.address[bank] as u16, value);
                } else {
                    self.address[bank] = value;
                }
            }
        }
        Some(true)
    }

    /// Generate one stereo frame. Mono OPL2 output is copied to both sides. The original Pro
    /// sends its left and right YM3812 outputs to separate mixer channels.
    pub fn sample(&mut self) -> FmSample {
        match &mut self.chip {
            Chip::Opl2(chip) => mono_frame(chip.sample()),
            Chip::DualOpl2(chips) => FmSample {
                left: chips[0].sample(),
                right: chips[1].sample(),
            },
            Chip::Opl3(chip) => stereo_frame(chip.sample()),
        }
    }

    /// Fill `out` with consecutive FM frames. The host chooses how to combine them with DSP PCM
    /// and card-mixer sources.
    pub fn generate(&mut self, out: &mut [FmSample]) {
        for sample in out {
            *sample = self.sample();
        }
    }

    /// Access the YM3812 in a mono or dual-OPL2 profile. `index` is 0 for left/mono and 1 for
    /// right on the original Sound Blaster Pro.
    pub fn opl2(&self, index: usize) -> Option<&Opl2> {
        match &self.chip {
            Chip::Opl2(chip) if index == 0 => Some(chip),
            Chip::DualOpl2(chips) => chips.get(index),
            _ => None,
        }
    }

    /// Mutably access a YM3812 in a mono or dual-OPL2 profile.
    pub fn opl2_mut(&mut self, index: usize) -> Option<&mut Opl2> {
        match &mut self.chip {
            Chip::Opl2(chip) if index == 0 => Some(chip),
            Chip::DualOpl2(chips) => chips.get_mut(index),
            _ => None,
        }
    }

    /// Access the YMF262 in an OPL3 profile.
    pub fn opl3(&self) -> Option<&Opl3> {
        match &self.chip {
            Chip::Opl3(chip) => Some(chip),
            _ => None,
        }
    }

    /// Mutably access the YMF262 in an OPL3 profile.
    pub fn opl3_mut(&mut self) -> Option<&mut Opl3> {
        match &mut self.chip {
            Chip::Opl3(chip) => Some(chip),
            _ => None,
        }
    }

    fn write_target(&self, port: u16) -> Option<(usize, bool)> {
        if self.card_ports_enabled {
            if let Some(offset) = port.checked_sub(self.card_base) {
                if offset <= 3 {
                    match self.kind {
                        FmChipKind::Opl2 if offset <= 1 => {
                            return Some((0, offset == 1));
                        }
                        FmChipKind::DualOpl2 | FmChipKind::Opl3 => {
                            return Some(((offset / 2) as usize, offset & 1 != 0));
                        }
                        FmChipKind::Opl2 => {}
                    }
                }
            }
        }

        let old_sb_port = self.card_base.wrapping_add(8);
        if port == old_sb_port || port == old_sb_port.wrapping_add(1) {
            return Some((0, port == old_sb_port.wrapping_add(1)));
        }
        let legacy_offset = port.checked_sub(LEGACY_FM_PORT)?;
        if matches!(self.chip, Chip::Opl3(_)) && legacy_offset <= 3 {
            return Some(((legacy_offset / 2) as usize, legacy_offset & 1 != 0));
        }
        if legacy_offset <= 1 {
            return Some((0, legacy_offset == 1));
        }
        None
    }

    fn status_bank(&self, port: u16) -> Option<usize> {
        if self.card_ports_enabled {
            if let Some(offset) = port.checked_sub(self.card_base) {
                match self.kind {
                    FmChipKind::Opl2 if offset == 0 => return Some(0),
                    FmChipKind::DualOpl2 if offset == 0 || offset == 2 => {
                        return Some((offset / 2) as usize)
                    }
                    FmChipKind::Opl3 if offset == 0 => return Some(0),
                    _ => {}
                }
            }
        }

        if port == self.card_base.wrapping_add(8) || port == LEGACY_FM_PORT {
            Some(0)
        } else {
            None
        }
    }
}

fn write_opl2(chip: &mut Opl2, address: &mut u8, data_port: bool, value: u8) {
    if data_port {
        chip.write(*address, value);
    } else {
        *address = value;
    }
}

fn mono_frame(value: i16) -> FmSample {
    FmSample {
        left: value,
        right: value,
    }
}

fn stereo_frame(value: StereoSample) -> FmSample {
    FmSample {
        left: value.left,
        right: value.right,
    }
}
