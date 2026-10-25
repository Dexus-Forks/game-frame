use crate::control::ControlChip;
use crate::mixer;
use crate::{
    AdlibGoldAudioInputs, AdlibGoldMixFrame, BoardIdentity, ControlChipError, ControlTiming,
    InterruptSource, Opl3, CONTROL_REGISTER_BUSY_CLOCKS, CONTROL_REGISTER_COUNT,
    MIXER_REGISTER_BUSY_CLOCKS, OPL3_MASTER_CLOCK_HZ,
};
use gameframe_ymz263::{MidiLineEvent, Ymz263, Ymz263PortError, YMZ263_MASTER_CLOCK_HZ};

/// Configuration for one GSS/AdLib Gold card revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdlibGoldConfig {
    /// Board model and option straps reported by Control Chip register zero.
    pub identity: BoardIdentity,
    /// Busy durations for EEPROM operations; hardware save latency is not specified by the toolkit.
    pub control_timing: ControlTiming,
    /// Control Chip register state at construction. The default sets only register `0x15` to
    /// the documented `0x71` audio-port base; other revision-specific defaults are unknown.
    pub initial_control_registers: [u8; CONTROL_REGISTER_COUNT],
    /// Persistent Control Chip register image used by the RT command. Defaults to zero.
    pub eeprom_registers: [u8; CONTROL_REGISTER_COUNT],
}

impl Default for AdlibGoldConfig {
    fn default() -> Self {
        let mut initial_control_registers = [0; CONTROL_REGISTER_COUNT];
        let mut eeprom_registers = [0; CONTROL_REGISTER_COUNT];
        initial_control_registers[0x15] = 0x71; // 388h / 8
        eeprom_registers[0x15] = 0x71;
        Self {
            identity: BoardIdentity::default(),
            control_timing: ControlTiming::default(),
            initial_control_registers,
            eeprom_registers,
        }
    }
}

/// Errors returned by a timestamped AdLib Gold port write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdlibGoldPortError {
    TimeWentBackwards { previous: u64, received: u64 },
    Opl3WriteTooSoon { wait_clocks: u8 },
    ControlChip(ControlChipError),
    Mma(Ymz263PortError),
}

/// AdLib Gold OPL3, MMA and Control Chip port block.
///
/// The audio block defaults to `388h` and occupies eight ports. This model implements the two FM
/// banks, the Control Chip overlay and the four MMA ports at offsets four through seven.
pub struct AdlibGold {
    chip: Opl3,
    mma: Ymz263,
    control: ControlChip,
    audio_base: u16,
    address: [u8; 2],
    control_access: bool,
    external_interrupts: u8,
    last_fm_write: Option<u64>,
    now: u64,
}

impl AdlibGold {
    /// Create a card with the configured hardware identity and startup port from register `0x15`.
    /// The default configuration uses `388h`.
    pub fn new(config: AdlibGoldConfig) -> Self {
        let audio_base = ((config.initial_control_registers[0x15] & 0x7F) as u16) << 3;
        Self {
            chip: Opl3::new(),
            mma: Ymz263::new(),
            control: ControlChip::new(
                config.identity,
                config.control_timing,
                config.initial_control_registers,
                config.eeprom_registers,
            ),
            audio_base,
            address: [0; 2],
            control_access: false,
            external_interrupts: 0,
            last_fm_write: None,
            now: 0,
        }
    }

    /// Handle an audio-block port write at a YMF262 master-clock timestamp.
    ///
    /// Returns `Ok(false)` for ports this crate does not implement. Writes to registers
    /// `0x04`–`0x08` and `0x09`–`0x16` start Control Chip busy intervals of 450 μs and 5 μs.
    /// Poll status and
    /// retry once its busy bits clear.
    pub fn write_port_at(
        &mut self,
        port: u16,
        value: u8,
        master_clock: u64,
    ) -> Result<bool, AdlibGoldPortError> {
        self.advance_to(master_clock)?;
        let Some(offset) = port.checked_sub(self.audio_base) else {
            return Ok(false);
        };
        if offset > 7 {
            return Ok(false);
        }

        if offset >= 4 {
            self.mma
                .write_port_at((offset - 4) as u8, value, self.mma_clock(master_clock))
                .map_err(AdlibGoldPortError::Mma)?;
            self.refresh_card_interrupts();
            return Ok(true);
        }

        match offset {
            0 => {
                self.check_fm_spacing(master_clock)?;
                self.address[0] = value;
                self.last_fm_write = Some(master_clock);
            }
            1 => {
                self.check_fm_spacing(master_clock)?;
                self.chip.write(self.address[0] as u16, value);
                self.last_fm_write = Some(master_clock);
            }
            2 if !self.control_access && value == 0xFF => self.control_access = true,
            2 if !self.control_access => {
                self.check_fm_spacing(master_clock)?;
                self.address[1] = value;
                self.last_fm_write = Some(master_clock);
            }
            2 if self.control_access && value == 0xFE => {
                if self.control.status() & 0xC0 != 0 {
                    return Err(AdlibGoldPortError::ControlChip(ControlChipError::Busy {
                        status: self.control.status(),
                    }));
                }
                self.control_access = false;
                self.address[1] = value;
            }
            2 if self.control_access => {
                if self.control.status() & 0xC0 != 0 {
                    return Err(AdlibGoldPortError::ControlChip(ControlChipError::Busy {
                        status: self.control.status(),
                    }));
                }
                self.address[1] = value;
            }
            3 if self.control_access => {
                let relocated = self
                    .control
                    .write_register(self.address[1], value, master_clock)
                    .map_err(AdlibGoldPortError::ControlChip)?;
                if let Some(new_base) = relocated {
                    self.audio_base = new_base;
                }
            }
            3 => {
                self.check_fm_spacing(master_clock)?;
                self.chip.write(0x100 | self.address[1] as u16, value);
                self.last_fm_write = Some(master_clock);
            }
            _ => unreachable!("only offsets zero through three reach the port router"),
        }

        self.refresh_card_interrupts();
        Ok(true)
    }

    /// Read the implemented status or Control Chip data port at a master-clock timestamp.
    pub fn read_port_at(
        &mut self,
        port: u16,
        master_clock: u64,
    ) -> Result<Option<u8>, AdlibGoldPortError> {
        self.advance_to(master_clock)?;
        let Some(offset) = port.checked_sub(self.audio_base) else {
            return Ok(None);
        };
        if (4..=7).contains(&offset) {
            let value = self
                .mma
                .read_port_at((offset - 4) as u8, self.mma_clock(master_clock))
                .map_err(AdlibGoldPortError::Mma)?;
            self.refresh_card_interrupts();
            return Ok(value);
        }
        let value = match offset {
            0 => Some(self.chip.status()),
            2 if self.control_access => Some(self.control.status()),
            3 if self.control_access => Some(
                self.control
                    .read_register(self.address[1], master_clock)
                    .map_err(AdlibGoldPortError::ControlChip)?,
            ),
            _ => None,
        };
        Ok(value)
    }

    /// Read the active-low pending state of one card interrupt input.
    pub fn set_interrupt_pending(&mut self, source: InterruptSource, pending: bool) {
        if pending {
            self.external_interrupts |= source.mask();
        } else {
            self.external_interrupts &= !source.mask();
        }
        self.refresh_card_interrupts();
    }

    /// Set the telephone ring input. A low ring state reads as bit 1 set in register 1.
    pub fn set_telephone_ringing(&mut self, ringing: bool) {
        self.control.set_telephone_ring(ringing);
    }

    /// Current base port of the eight-port audio block.
    pub const fn audio_base(&self) -> u16 {
        self.audio_base
    }

    /// Access the YMF262 for rendering and chip-specific inspection.
    pub fn chip(&self) -> &Opl3 {
        &self.chip
    }

    /// Mutably access the YMF262 for rendering or direct register access.
    pub fn chip_mut(&mut self) -> &mut Opl3 {
        &mut self.chip
    }

    /// Access the MMA model for FIFO, output, DMA and MIDI state.
    pub fn mma(&self) -> &Ymz263 {
        &self.mma
    }

    /// Mutably access the MMA model for recording input and event draining.
    pub fn mma_mut(&mut self) -> &mut Ymz263 {
        &mut self.mma
    }

    /// Mix one set of digital and external inputs through the documented card volume controls.
    ///
    /// This applies Control Chip source volumes, channel selection, forced mono, mute and final
    /// output levels. Pseudo/spatial phase processing and the analog bass/treble filters are
    /// reported in the result but are not approximated by this digital stage.
    pub fn mix_sources(&self, inputs: AdlibGoldAudioInputs) -> AdlibGoldMixFrame {
        mixer::mix(
            inputs,
            self.control.mixer_registers(),
            self.control.telephone_installed() && self.control.telephone_engaged(),
        )
    }

    /// Drain completed MIDI bytes transmitted by the MMA UART.
    pub fn drain_midi_events(&mut self) -> Vec<MidiLineEvent> {
        self.mma.drain_midi_events()
    }

    /// Advance the card and both chips to a YMF262 master-clock timestamp.
    pub fn advance_to(&mut self, clock: u64) -> Result<(), AdlibGoldPortError> {
        self.advance_to_inner(clock)
    }

    fn advance_to_inner(&mut self, clock: u64) -> Result<(), AdlibGoldPortError> {
        if clock < self.now {
            return Err(AdlibGoldPortError::TimeWentBackwards {
                previous: self.now,
                received: clock,
            });
        }
        self.now = clock;
        self.control.advance_to(clock);
        self.mma
            .advance_to(self.mma_clock(clock))
            .map_err(|error| AdlibGoldPortError::Mma(Ymz263PortError::Access(error)))?;
        self.refresh_card_interrupts();
        if let Some(audio_base) = self.control.take_relocated_audio_base() {
            self.audio_base = audio_base;
        }
        Ok(())
    }

    fn mma_clock(&self, opl3_clock: u64) -> u64 {
        ((opl3_clock as u128 * YMZ263_MASTER_CLOCK_HZ as u128) / OPL3_MASTER_CLOCK_HZ as u128)
            as u64
    }

    fn refresh_card_interrupts(&mut self) {
        let fm_pending = self.external_interrupts & InterruptSource::Fm.mask() != 0
            || self.chip.status() & 0x80 != 0;
        let sampling_pending = self.external_interrupts & InterruptSource::Sampling.mask() != 0
            || self.mma.irq_pending();
        let telephone_pending = self.external_interrupts & InterruptSource::Telephone.mask() != 0;
        let scsi_pending = self.external_interrupts & InterruptSource::Scsi.mask() != 0;
        self.control
            .set_interrupt_pending(InterruptSource::Fm, fm_pending);
        self.control
            .set_interrupt_pending(InterruptSource::Sampling, sampling_pending);
        self.control
            .set_interrupt_pending(InterruptSource::Telephone, telephone_pending);
        self.control
            .set_interrupt_pending(InterruptSource::Scsi, scsi_pending);
    }

    fn check_fm_spacing(&self, clock: u64) -> Result<(), AdlibGoldPortError> {
        if let Some(previous) = self.last_fm_write {
            let elapsed = clock.saturating_sub(previous);
            if elapsed < 32 {
                return Err(AdlibGoldPortError::Opl3WriteTooSoon {
                    wait_clocks: (32 - elapsed) as u8,
                });
            }
        }
        Ok(())
    }
}

impl Default for AdlibGold {
    fn default() -> Self {
        Self::new(AdlibGoldConfig::default())
    }
}

const _: () = {
    assert!(MIXER_REGISTER_BUSY_CLOCKS == 6_444);
    assert!(CONTROL_REGISTER_BUSY_CLOCKS == 72);
};
