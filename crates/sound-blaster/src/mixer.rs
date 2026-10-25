/// Mixer-chip generation fitted to the modeled card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MixerKind {
    /// No indexed mixer fitted to the modeled early board.
    None,
    /// Mono CT1335 mixer on early CD-interface boards.
    Ct1335,
    /// Stereo CT1345 mixer on Sound Blaster Pro.
    Ct1345,
    /// CT1745 mixer on Sound Blaster 16.
    Ct1745,
}

/// Indexed register file for the Sound Blaster family mixer chips.
///
/// This models register selection, reset defaults, documented writable masks, and the CT1745
/// interrupt status. Analog filtering, tone curves and the amplifier response remain separate.
pub struct Mixer {
    kind: MixerKind,
    index: u8,
    registers: [u8; 256],
    interrupt_status: u8,
}

impl Mixer {
    pub fn new(kind: MixerKind) -> Self {
        let mut mixer = Self {
            kind,
            index: 0,
            registers: [0; 256],
            interrupt_status: 0,
        };
        mixer.reset();
        mixer
    }

    pub fn kind(&self) -> MixerKind {
        self.kind
    }

    pub fn index(&self) -> u8 {
        self.index
    }

    pub fn select(&mut self, index: u8) {
        self.index = index;
    }

    pub fn selected(&self) -> u8 {
        self.read(self.index)
    }

    pub fn write_selected(&mut self, value: u8) {
        self.write(self.index, value);
    }

    pub fn read(&self, index: u8) -> u8 {
        if self.kind == MixerKind::Ct1745 && index == 0x82 {
            return self.interrupt_status;
        }
        if self.is_defined(index) {
            self.registers[index as usize]
        } else {
            0xFF
        }
    }

    pub fn write(&mut self, index: u8, value: u8) {
        if index == 0 {
            self.reset();
            return;
        }
        if !self.is_writable(index) {
            return;
        }
        self.registers[index as usize] = value & self.write_mask(index);
    }

    /// Set the hardware interrupt-source bits visible through the CT1745 register 82h.
    pub fn set_interrupt_status(&mut self, status: u8) {
        self.interrupt_status = status & 0x07;
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.registers = [0; 256];
        self.interrupt_status = 0;
        match self.kind {
            MixerKind::None => {}
            MixerKind::Ct1335 => {
                self.registers[0x02] = 4;
                self.registers[0x06] = 4;
            }
            MixerKind::Ct1345 => {
                self.registers[0x04] = stereo3(4, 4);
                self.registers[0x22] = stereo3(4, 4);
                self.registers[0x26] = stereo3(4, 4);
            }
            MixerKind::Ct1745 => {
                self.registers[0x04] = 0xCC;
                self.registers[0x0C] = 0x00;
                self.registers[0x22] = 0xCC;
                self.registers[0x26] = 0xCC;
                self.registers[0x3C] = 0x1F;
                self.registers[0x3D] = 0x15;
                self.registers[0x3E] = 0x0B;
                self.registers[0x30] = 24;
                self.registers[0x31] = 24;
                self.registers[0x32] = 24;
                self.registers[0x33] = 24;
                self.registers[0x34] = 24;
                self.registers[0x35] = 24;
                self.registers[0x43] = 0;
                self.registers[0x80] = 0x02;
                self.registers[0x81] = 0x22;
                self.registers[0x44] = 8;
                self.registers[0x45] = 8;
                self.registers[0x46] = 8;
                self.registers[0x47] = 8;
            }
        }
    }

    fn is_defined(&self, index: u8) -> bool {
        match self.kind {
            MixerKind::None => false,
            MixerKind::Ct1335 => matches!(index, 0x00 | 0x02 | 0x06 | 0x08 | 0x0A),
            MixerKind::Ct1345 => matches!(
                index,
                0x00 | 0x04 | 0x0A | 0x0C | 0x0E | 0x22 | 0x26 | 0x28 | 0x2E
            ),
            MixerKind::Ct1745 => matches!(
                index,
                0x00 | 0x04
                    | 0x0A
                    | 0x22
                    | 0x26
                    | 0x28
                    | 0x2E
                    | 0x30..=0x47
                    | 0x80..=0x82
            ),
        }
    }

    fn is_writable(&self, index: u8) -> bool {
        self.is_defined(index) && !(self.kind == MixerKind::Ct1745 && index == 0x82)
    }

    fn write_mask(&self, index: u8) -> u8 {
        match self.kind {
            MixerKind::None => 0,
            MixerKind::Ct1335 => match index {
                0x02 | 0x06 | 0x08 => 0x07,
                0x0A => 0x03,
                _ => 0xFF,
            },
            MixerKind::Ct1345 => match index {
                0x04 | 0x22 | 0x26 | 0x28 | 0x2E => 0xEE,
                0x0A => 0x03,
                0x0C => 0x2E,
                0x0E => 0x22,
                _ => 0xFF,
            },
            MixerKind::Ct1745 => match index {
                0x0A => 0x07,
                0x3A => 0x1F,
                0x3B => 0x03,
                0x3C => 0x1F,
                0x3D | 0x3E => 0x7F,
                0x3F..=0x42 => 0x03,
                0x43 => 0x01,
                0x44..=0x47 => 0x0F,
                0x80 => 0x0F,
                0x81 => 0xEB,
                0x82 => 0,
                _ => 0xFF,
            },
        }
    }
}

const fn stereo3(left: u8, right: u8) -> u8 {
    (left << 5) | (right << 1)
}
