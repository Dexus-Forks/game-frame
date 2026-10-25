use std::collections::VecDeque;

use crate::{
    DmaDirection, DmaRequest, Dsp, DspEvent, DspVersion, FmChipKind, FmSynth, InterruptSource,
    Mixer, MixerKind, Mpu401, MpuEvent, TimeError, TransferWidth,
};

/// I/O and chip revisions for one Creative Sound Blaster family card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoundBlasterConfig {
    pub base_port: u16,
    pub dsp_version: DspVersion,
    pub mixer: MixerKind,
    pub fm_chip: FmChipKind,
    pub mpu401_base: Option<u16>,
}

impl SoundBlasterConfig {
    pub const fn sb_15() -> Self {
        Self {
            base_port: 0x220,
            dsp_version: DspVersion::SB_1_5,
            mixer: MixerKind::None,
            fm_chip: FmChipKind::Opl2,
            mpu401_base: None,
        }
    }

    pub const fn sb_20() -> Self {
        Self {
            base_port: 0x220,
            dsp_version: DspVersion::new(2, 1),
            mixer: MixerKind::None,
            fm_chip: FmChipKind::Opl2,
            mpu401_base: None,
        }
    }

    pub const fn sb_2_cd() -> Self {
        Self {
            base_port: 0x220,
            dsp_version: DspVersion::new(2, 1),
            mixer: MixerKind::Ct1335,
            fm_chip: FmChipKind::Opl2,
            mpu401_base: None,
        }
    }

    /// Sound Blaster Pro 1 profile with separate left and right YM3812 chips.
    pub const fn sb_pro() -> Self {
        Self {
            base_port: 0x220,
            dsp_version: DspVersion::new(3, 1),
            mixer: MixerKind::Ct1345,
            fm_chip: FmChipKind::DualOpl2,
            mpu401_base: None,
        }
    }

    pub const fn sb_pro_2() -> Self {
        Self {
            base_port: 0x220,
            dsp_version: DspVersion::SB_PRO_2,
            mixer: MixerKind::Ct1345,
            fm_chip: FmChipKind::Opl3,
            mpu401_base: None,
        }
    }

    pub const fn sb16() -> Self {
        Self {
            base_port: 0x220,
            dsp_version: DspVersion::SB_16_4_05,
            mixer: MixerKind::Ct1745,
            fm_chip: FmChipKind::Opl3,
            mpu401_base: Some(0x330),
        }
    }
}

impl Default for SoundBlasterConfig {
    fn default() -> Self {
        Self::sb16()
    }
}

/// Port-level Sound Blaster model, composed from a DSP, card mixer and optional SB16 MPU-401.
///
/// The host still owns the ISA DMA controller and PIC. DSP timing and DMA request events are
/// returned by [`SoundBlaster::advance_to`]. Route those requests to the machine bus, pass DMA
/// reads to [`SoundBlaster::service_dma_read`], and route raised IRQ events through the PIC.
pub struct SoundBlaster {
    config: SoundBlasterConfig,
    dsp: Dsp,
    fm: FmSynth,
    mixer: Mixer,
    mpu: Option<Mpu401>,
    events: VecDeque<DspEvent>,
}

impl SoundBlaster {
    pub fn new(config: SoundBlasterConfig) -> Self {
        Self {
            config,
            dsp: Dsp::new(config.dsp_version),
            fm: FmSynth::new(
                config.fm_chip,
                config.base_port,
                matches!(config.mixer, MixerKind::Ct1345 | MixerKind::Ct1745),
            ),
            mixer: Mixer::new(config.mixer),
            mpu: config.mpu401_base.map(Mpu401::new),
            events: VecDeque::new(),
        }
    }

    pub fn config(&self) -> SoundBlasterConfig {
        self.config
    }

    pub fn dsp(&self) -> &Dsp {
        &self.dsp
    }

    pub fn dsp_mut(&mut self) -> &mut Dsp {
        &mut self.dsp
    }

    pub fn fm(&self) -> &FmSynth {
        &self.fm
    }

    pub fn fm_mut(&mut self) -> &mut FmSynth {
        &mut self.fm
    }

    pub fn mixer(&self) -> &Mixer {
        &self.mixer
    }

    pub fn mixer_mut(&mut self) -> &mut Mixer {
        &mut self.mixer
    }

    pub fn mpu401(&self) -> Option<&Mpu401> {
        self.mpu.as_ref()
    }

    pub fn mpu401_mut(&mut self) -> Option<&mut Mpu401> {
        self.mpu.as_mut()
    }

    /// Read a mapped port. None indicates an address outside this modeled card's ports.
    pub fn read_port_at(&mut self, port: u16, at_ns: u64) -> Result<Option<u8>, TimeError> {
        self.advance_and_queue(at_ns)?;
        if let Some(value) = self.fm.read_port(port) {
            return Ok(Some(value));
        }
        let offset = port.wrapping_sub(self.config.base_port);
        let value = match offset {
            0x05 if self.config.mixer != MixerKind::None => Some(self.read_mixer_data()),
            0x0A => Some(self.dsp.read_data_at(at_ns)?.unwrap_or(0xFF)),
            0x0C => Some(self.dsp.write_status()),
            0x0E => Some(self.dsp.read_buffer_status_at(at_ns)?),
            0x0F if self.config.dsp_version.major >= 4 => {
                self.dsp.acknowledge_16bit_irq_at(at_ns)?;
                Some(0xFF)
            }
            _ => self.read_mpu_port(port, at_ns),
        };
        self.refresh_mixer_status();
        Ok(value)
    }

    /// Write a mapped port. False indicates an unmapped address or a write rejected while busy.
    pub fn write_port_at(&mut self, port: u16, value: u8, at_ns: u64) -> Result<bool, TimeError> {
        self.advance_and_queue(at_ns)?;
        if let Some(accepted) = self.fm.write_port_at(port, value, at_ns) {
            self.refresh_mixer_status();
            return Ok(accepted);
        }
        let offset = port.wrapping_sub(self.config.base_port);
        let accepted = match offset {
            0x04 if self.config.mixer != MixerKind::None => {
                self.mixer.select(value);
                true
            }
            0x05 if self.config.mixer != MixerKind::None => {
                let selected = self.mixer.index();
                self.mixer.write_selected(value);
                if selected == 0x0E && self.config.mixer == MixerKind::Ct1345 {
                    self.dsp.set_legacy_output_stereo(value & 0x02 != 0);
                }
                true
            }
            0x06 => {
                self.dsp.write_reset_at(value, at_ns)?;
                true
            }
            0x0C => self.dsp.write_command_at(value, at_ns)?,
            _ => self.write_mpu_port(port, value, at_ns),
        };
        self.refresh_mixer_status();
        Ok(accepted)
    }

    /// Advance all card-side clocks and drain events generated through the requested timestamp.
    pub fn advance_to(&mut self, at_ns: u64) -> Result<Vec<DspEvent>, TimeError> {
        self.advance_and_queue(at_ns)?;
        Ok(self.events.drain(..).collect())
    }

    /// Supply a word fetched by the machine's ISA DMA controller for a card playback request.
    pub fn service_dma_read(&mut self, value: u16) -> Option<DspEvent> {
        self.dsp.service_dma_read(value)
    }

    pub fn push_capture_sample(&mut self, value: u16) {
        self.dsp.push_capture_sample(value);
    }

    pub fn push_midi_input_at(&mut self, value: u8, at_ns: u64) -> Result<(), TimeError> {
        self.advance_and_queue(at_ns)?;
        self.dsp.push_midi_input_at(value, at_ns)
    }

    /// Inject a byte received by the SB16 MPU-401 UART. Returns false when this card has no MPU.
    pub fn push_mpu_input_at(&mut self, value: u8, at_ns: u64) -> Result<bool, TimeError> {
        self.advance_and_queue(at_ns)?;
        let Some(mpu) = self.mpu.as_mut() else {
            return Ok(false);
        };
        let was_pending = mpu.irq_pending();
        if !mpu.push_input(value) {
            return Ok(true);
        }
        if !was_pending && mpu.irq_pending() {
            self.events.push_back(DspEvent::InterruptRaised {
                at_ns,
                source: InterruptSource::Mpu401,
            });
        }
        self.refresh_mixer_status();
        Ok(true)
    }

    /// Read-only interrupt summary for the machine's PIC adapter.
    pub fn irq_pending(&self) -> bool {
        self.dsp.irq_pending() || self.mpu.as_ref().is_some_and(Mpu401::irq_pending)
    }

    pub fn drain_mpu_output(&mut self) -> Vec<MpuEvent> {
        self.mpu
            .as_mut()
            .map(Mpu401::drain_output)
            .unwrap_or_default()
    }

    pub fn drain_events(&mut self) -> Vec<DspEvent> {
        self.events.extend(self.dsp.drain_events());
        self.events.drain(..).collect()
    }

    fn advance_and_queue(&mut self, at_ns: u64) -> Result<(), TimeError> {
        self.events.extend(self.dsp.advance_to(at_ns)?);
        self.refresh_mixer_status();
        Ok(())
    }

    fn refresh_mixer_status(&mut self) {
        let irq_status = self.dsp.interrupt_status()
            | if self.mpu.as_ref().is_some_and(Mpu401::irq_pending) {
                0x04
            } else {
                0
            };
        self.mixer.set_interrupt_status(irq_status);
    }

    fn read_mixer_data(&self) -> u8 {
        let mut status = self.dsp.interrupt_status()
            | if self.mpu.as_ref().is_some_and(Mpu401::irq_pending) {
                0x04
            } else {
                0
            };
        if self.mixer.index() == 0x82 {
            status &= 0x07;
        }
        if self.config.mixer == MixerKind::Ct1745 && self.mixer.index() == 0x82 {
            status
        } else {
            self.mixer.selected()
        }
    }

    fn read_mpu_port(&mut self, port: u16, at_ns: u64) -> Option<u8> {
        let mpu = self.mpu.as_mut()?;
        match port {
            p if p == mpu.base() => mpu.read_data(),
            p if p == mpu.base().wrapping_add(1) => Some(mpu.status(at_ns)),
            _ => None,
        }
    }

    fn write_mpu_port(&mut self, port: u16, value: u8, at_ns: u64) -> bool {
        let Some(mpu) = self.mpu.as_mut() else {
            return false;
        };
        if port == mpu.base() {
            mpu.write_data(value, at_ns).is_some()
        } else if port == mpu.base().wrapping_add(1) {
            let was_pending = mpu.irq_pending();
            let accepted = mpu.write_command(value, at_ns);
            let raised = !was_pending && mpu.irq_pending();
            if raised {
                self.events.push_back(DspEvent::InterruptRaised {
                    at_ns,
                    source: InterruptSource::Mpu401,
                });
            }
            accepted
        } else {
            false
        }
    }
}

#[allow(dead_code)]
fn _card_interface_types(_: DmaDirection, _: DmaRequest, _: InterruptSource, _: TransferWidth) {}
