/// YMF262 clock used for card-level timing conversion.
pub const OPL3_MASTER_CLOCK_HZ: u64 = 14_318_180;
/// Number of addresses in the documented Control Chip register map, `0x00` through `0x18`.
pub const CONTROL_REGISTER_COUNT: usize = 0x19;
/// Busy duration after Control Chip writes to registers 4 through 8.
pub const MIXER_REGISTER_BUSY_CLOCKS: u64 = micros_to_clocks(450);
/// Busy duration after Control Chip writes to registers `0x09` through `0x16`.
pub const CONTROL_REGISTER_BUSY_CLOCKS: u64 = micros_to_clocks(5);
const RESTORE_BUSY_CLOCKS: u64 = micros_to_clocks(2_499);
const LAST_REGISTER: usize = CONTROL_REGISTER_COUNT - 1;

const fn micros_to_clocks(microseconds: u64) -> u64 {
    (microseconds * OPL3_MASTER_CLOCK_HZ).div_ceil(1_000_000)
}

/// GSS bus model encoded by Control Chip register zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum BusModel {
    /// 16-bit ISA GSS model (register model ID 0).
    #[default]
    Isa16 = 0,
    /// 8-bit ISA GSS model (register model ID 1).
    Isa8 = 1,
    /// Micro Channel GSS model (register model ID 2).
    MicroChannel = 2,
}

/// The option straps returned by Control Chip register zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoardIdentity {
    pub bus_model: BusModel,
    pub telephone: bool,
    pub surround: bool,
    pub cd_rom: bool,
}

impl BoardIdentity {
    fn register_value(self) -> u8 {
        let model = (self.bus_model as u8 & 0x03) << 2;
        let absent_options = ((!self.telephone as u8) << 4)
            | ((!self.surround as u8) << 5)
            | ((!self.cd_rom as u8) << 6);
        model | absent_options
    }
}

/// Time constants for EEPROM operations whose exact duration is not published.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlTiming {
    /// Busy duration after saving the register image. The toolkit specifies polling bit 7 but
    /// gives no write duration. The default is one master clock so an immediate status read can
    /// observe the operation without inventing a longer latency.
    pub eeprom_write_busy_clocks: u64,
    /// Busy duration while restoring registers. The default uses 2499 μs, the upper bound implied
    /// by the toolkit's “a bit less than 2.5 ms” guidance. This is a software-model choice.
    pub eeprom_restore_busy_clocks: u64,
}

impl Default for ControlTiming {
    fn default() -> Self {
        Self {
            eeprom_write_busy_clocks: 1,
            eeprom_restore_busy_clocks: RESTORE_BUSY_CLOCKS,
        }
    }
}

/// Control Chip interrupt input. A pending input reads as zero in the corresponding status bit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterruptSource {
    Fm,
    Sampling,
    Telephone,
    Scsi,
}

impl InterruptSource {
    pub(crate) const fn mask(self) -> u8 {
        match self {
            Self::Fm => 1 << 0,
            Self::Sampling => 1 << 1,
            Self::Telephone => 1 << 2,
            Self::Scsi => 1 << 3,
        }
    }
}

/// A Control Chip port access conflicts with an active operation or an unknown register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlChipError {
    Busy { status: u8 },
    UnsupportedRegister(u8),
}

#[derive(Clone, Copy)]
enum PendingRestore {
    None,
    At {
        clock: u64,
        image: [u8; LAST_REGISTER + 1],
    },
}

/// Driver-visible state of the AdLib Gold Control Chip.
pub(crate) struct ControlChip {
    identity: BoardIdentity,
    timing: ControlTiming,
    registers: [u8; LAST_REGISTER + 1],
    eeprom: [u8; LAST_REGISTER + 1],
    pending_interrupts: u8,
    telephone_ring: bool,
    register_busy_until: u64,
    eeprom_busy_until: u64,
    pending_restore: PendingRestore,
    relocated_audio_base: Option<u16>,
    now: u64,
}

impl ControlChip {
    pub(crate) fn new(
        identity: BoardIdentity,
        timing: ControlTiming,
        initial_registers: [u8; CONTROL_REGISTER_COUNT],
        eeprom_registers: [u8; CONTROL_REGISTER_COUNT],
    ) -> Self {
        Self {
            identity,
            timing,
            registers: initial_registers,
            eeprom: eeprom_registers,
            pending_interrupts: 0,
            telephone_ring: false,
            register_busy_until: 0,
            eeprom_busy_until: 0,
            pending_restore: PendingRestore::None,
            relocated_audio_base: None,
            now: 0,
        }
    }

    pub(crate) fn advance_to(&mut self, clock: u64) {
        self.now = self.now.max(clock);
        if let PendingRestore::At {
            clock: done_at,
            image,
        } = self.pending_restore
        {
            if done_at <= self.now {
                self.registers = image;
                self.relocated_audio_base = Some(((image[0x15] & 0x7F) as u16) << 3);
                self.pending_restore = PendingRestore::None;
            }
        }
    }

    pub(crate) fn take_relocated_audio_base(&mut self) -> Option<u16> {
        self.relocated_audio_base.take()
    }

    pub(crate) const fn mixer_registers(&self) -> &[u8; CONTROL_REGISTER_COUNT] {
        &self.registers
    }

    pub(crate) const fn telephone_installed(&self) -> bool {
        self.identity.telephone
    }

    pub(crate) const fn telephone_engaged(&self) -> bool {
        self.registers[0x01] & 1 != 0
    }

    pub(crate) fn status(&self) -> u8 {
        let interrupts = (!self.pending_interrupts) & 0x0F;
        let register_busy = u8::from(self.now < self.register_busy_until) << 6;
        let eeprom_busy = u8::from(self.now < self.eeprom_busy_until) << 7;
        interrupts | register_busy | eeprom_busy
    }

    pub(crate) fn set_interrupt_pending(&mut self, source: InterruptSource, pending: bool) {
        if pending {
            self.pending_interrupts |= source.mask();
        } else {
            self.pending_interrupts &= !source.mask();
        }
    }

    pub(crate) fn set_telephone_ring(&mut self, ringing: bool) {
        self.telephone_ring = ringing;
    }

    pub(crate) fn read_register(
        &mut self,
        register: u8,
        clock: u64,
    ) -> Result<u8, ControlChipError> {
        self.advance_to(clock);
        if self.is_busy() {
            return Err(ControlChipError::Busy {
                status: self.status(),
            });
        }
        if register as usize > LAST_REGISTER {
            return Err(ControlChipError::UnsupportedRegister(register));
        }

        Ok(match register {
            0x00 => self.identity.register_value(),
            0x01 => (u8::from(!self.telephone_ring) << 1) | (self.registers[1] & 1),
            0x12 => 0,
            _ => self.registers[register as usize],
        })
    }

    pub(crate) fn write_register(
        &mut self,
        register: u8,
        value: u8,
        clock: u64,
    ) -> Result<Option<u16>, ControlChipError> {
        self.advance_to(clock);
        if self.is_busy() {
            return Err(ControlChipError::Busy {
                status: self.status(),
            });
        }
        if register as usize > LAST_REGISTER {
            return Err(ControlChipError::UnsupportedRegister(register));
        }

        match register {
            0x00 => self.eeprom_command(value, clock),
            0x01 => self.registers[1] = value & 1,
            0x12 => self.registers[0x12] = 0,
            0x15 => {
                let port_div8 = value & 0x7F;
                self.registers[0x15] = port_div8;
                self.register_busy_until = clock.saturating_add(CONTROL_REGISTER_BUSY_CLOCKS);
                return Ok(Some((port_div8 as u16) << 3));
            }
            _ => {
                self.registers[register as usize] = value;
                let busy_clocks = match register {
                    0x04..=0x08 => MIXER_REGISTER_BUSY_CLOCKS,
                    0x09..=0x16 => CONTROL_REGISTER_BUSY_CLOCKS,
                    _ => 0,
                };
                self.register_busy_until = clock.saturating_add(busy_clocks);
            }
        }

        Ok(None)
    }

    fn eeprom_command(&mut self, value: u8, clock: u64) {
        match value & 0x03 {
            0x02 => {
                self.eeprom = self.registers;
                self.eeprom_busy_until = clock.saturating_add(self.timing.eeprom_write_busy_clocks);
            }
            0x01 => {
                let done_at = clock.saturating_add(self.timing.eeprom_restore_busy_clocks);
                self.eeprom_busy_until = done_at;
                if done_at == clock {
                    self.registers = self.eeprom;
                } else {
                    self.pending_restore = PendingRestore::At {
                        clock: done_at,
                        image: self.eeprom,
                    };
                }
            }
            // The toolkit describes ST and RT independently but does not specify both set.
            // Treat the combined command as reserved and perform neither operation.
            _ => {}
        }
    }

    fn is_busy(&self) -> bool {
        self.now < self.register_busy_until || self.now < self.eeprom_busy_until
    }
}

impl Default for ControlChip {
    fn default() -> Self {
        Self::new(
            BoardIdentity::default(),
            ControlTiming::default(),
            [0; CONTROL_REGISTER_COUNT],
            [0; CONTROL_REGISTER_COUNT],
        )
    }
}
