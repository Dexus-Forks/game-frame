use crate::{Saa1099, Saa1099Sample};

/// CMS/Game Blaster default port base documented by Creative's CMS music driver notes.
pub const CMS_DEFAULT_BASE_PORT: u16 = 0x220;
/// Reported Game Blaster clock used by this card profile. A compatible-card manual reports the
/// oscillator; callers can override it for a board with a different or verified clock.
pub const CMS_DEFAULT_CLOCK_HZ: u32 = 7_159_090;

/// Two SAA1099 chips behind the Creative Music System/Game Blaster port window.
///
/// Original driver evidence: `CMSDRV.COM` at 1000:0CE4–0D0D and Sierra's `CMS35.BIN` at
/// 1000:0E1A–0F10. The trace and port mapping are in `docs/hardware/cms-saa1099.md`.
#[derive(Clone, Debug)]
pub struct CmsCard {
    base_port: u16,
    chips: [Saa1099; 2],
    detection_latches: [u8; 2],
}

impl Default for CmsCard {
    fn default() -> Self {
        Self::new()
    }
}

impl CmsCard {
    pub fn new() -> Self {
        Self::with_clock(CMS_DEFAULT_CLOCK_HZ)
    }

    pub fn with_clock(clock_hz: u32) -> Self {
        Self {
            base_port: CMS_DEFAULT_BASE_PORT,
            chips: [Saa1099::with_clock(clock_hz), Saa1099::with_clock(clock_hz)],
            detection_latches: [0; 2],
        }
    }

    pub fn with_base_port(base_port: u16, clock_hz: u32) -> Self {
        let mut card = Self::with_clock(clock_hz);
        card.base_port = base_port;
        card
    }

    pub fn base_port(&self) -> u16 {
        self.base_port
    }

    /// Route a write to the CMS chips or auxiliary detection latches. Even ports in the first
    /// four-port window write chip data; odd ports select a register. Detection latches are at
    /// offsets 6 and 7. Returns false for unmodeled ports.
    pub fn write_port(&mut self, port: u16, value: u8) -> bool {
        let Some(offset) = port.checked_sub(self.base_port) else {
            return false;
        };
        match offset {
            0..=3 => {
                let chip = (offset / 2) as usize;
                if offset & 1 == 0 {
                    self.chips[chip].write_data(value);
                } else {
                    self.chips[chip].write_address(value);
                }
                true
            }
            6..=7 => {
                self.detection_latches[(offset - 6) as usize] = value;
                true
            }
            _ => false,
        }
    }

    /// Read the card's compatibility status port and auxiliary detection latches. The auxiliary
    /// latch echo follows a modern Game Blaster-compatible card manual; the exact original custom
    /// IC behavior is not fully documented.
    pub fn read_port(&self, port: u16) -> Option<u8> {
        let offset = port.checked_sub(self.base_port)?;
        match offset {
            4 => Some(0x7f),
            10..=11 => Some(self.detection_latches[(offset - 10) as usize]),
            _ => None,
        }
    }

    pub fn chip(&self, index: usize) -> Option<&Saa1099> {
        self.chips.get(index)
    }

    /// Generate one mixed stereo sample. Both chips use the same configured clock and output
    /// divider.
    pub fn sample(&mut self) -> Saa1099Sample {
        let first = self.chips[0].sample();
        let second = self.chips[1].sample();
        Saa1099Sample {
            left: (i32::from(first.left) + i32::from(second.left))
                .clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            right: (i32::from(first.right) + i32::from(second.right))
                .clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        }
    }

    pub fn generate(&mut self, output: &mut [Saa1099Sample]) {
        for sample in output {
            *sample = self.sample();
        }
    }
}
