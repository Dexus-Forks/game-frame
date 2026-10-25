use std::collections::VecDeque;

use crate::adpcm::AdpcmDecoder;

mod commands;
mod transfer;

const RESET_MIN_NS: u64 = 3_000;
const RESET_READY_NS: u64 = 100_000;

/// A modeled Creative DSP revision. The exact version response is visible to DOS software.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DspVersion {
    pub major: u8,
    pub minor: u8,
}

impl DspVersion {
    pub const SB_1_5: Self = Self { major: 1, minor: 5 };
    pub const SB_2_01: Self = Self { major: 2, minor: 1 };
    pub const SB_PRO_2: Self = Self { major: 3, minor: 2 };
    pub const SB_16_4_05: Self = Self { major: 4, minor: 5 };

    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    const fn at_least(self, major: u8, minor: u8) -> bool {
        self.major > major || (self.major == major && self.minor >= minor)
    }
}

/// Transfer direction from the Sound Blaster's perspective.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmaDirection {
    /// ISA DMA reads data from system memory and sends it to the card.
    MemoryToCard,
    /// The card sends captured data to system memory through ISA DMA.
    CardToMemory,
}

/// Transfer width selected by the running DSP mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferWidth {
    EightBit,
    SixteenBit,
}

/// Source of a card interrupt before the machine routes it to a PIC line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterruptSource {
    EightBitDma,
    SixteenBitDma,
    SbMidi,
    Mpu401,
}

/// One scheduled request for the machine-level ISA DMA controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DmaRequest {
    pub at_ns: u64,
    pub direction: DmaDirection,
    pub width: TransferWidth,
}

/// Digital output channel selected by the sample stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PcmChannel {
    Mono,
    Left,
    Right,
}

/// A signed, normalized PCM value decoded by the DSP from one DMA transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PcmSample {
    pub at_ns: u64,
    pub channel: PcmChannel,
    pub value: i16,
}

/// Observable work produced by advancing the device or servicing its DMA line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DspEvent {
    DmaRequest(DmaRequest),
    DmaWrite {
        at_ns: u64,
        value: u16,
        width: TransferWidth,
    },
    Pcm(PcmSample),
    /// One compressed byte and its decoded unsigned PCM samples.
    ///
    /// `samples` are in output order. The first sample is due at `at_ns`; each following sample
    /// is one `sample_rate_hz` period later. Slots at or after `sample_count` are unused.
    AdpcmByte {
        at_ns: u64,
        value: u8,
        bits_per_sample: u8,
        reference_byte: bool,
        sample_rate_hz: u32,
        samples: [i16; 4],
        sample_count: u8,
    },
    MidiOutput {
        at_ns: u64,
        value: u8,
    },
    InterruptRaised {
        at_ns: u64,
        source: InterruptSource,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MidiMode {
    Off,
    NormalPolling,
    NormalInterrupt,
    UartPolling,
    UartInterrupt,
    UartTimestampPolling,
    UartTimestampInterrupt,
}

impl MidiMode {
    fn interrupt(self) -> bool {
        matches!(
            self,
            Self::NormalInterrupt | Self::UartInterrupt | Self::UartTimestampInterrupt
        )
    }

    fn timestamp(self) -> bool {
        matches!(
            self,
            Self::UartTimestampPolling | Self::UartTimestampInterrupt
        )
    }

    fn uart(self) -> bool {
        matches!(
            self,
            Self::UartPolling
                | Self::UartInterrupt
                | Self::UartTimestampPolling
                | Self::UartTimestampInterrupt
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PendingCommand {
    command: u8,
    expected: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveTransfer {
    direction: DmaDirection,
    width: TransferWidth,
    stereo: bool,
    signed: bool,
    auto_init: bool,
    high_speed: bool,
    adpcm_bits: Option<u8>,
    adpcm_reference_pending: bool,
    remaining: Option<u32>,
    reload: Option<u32>,
    block_remaining: Option<u32>,
    exit_at_block: bool,
    paused: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PendingDma {
    request: DmaRequest,
    adpcm_reference_byte: bool,
}

/// The Sound Blaster DSP command processor and digital sample interface.
///
/// All time arguments are monotonic nanoseconds on the host machine's virtual timeline.
/// The host must route [`DspEvent::DmaRequest`] to its ISA DMA controller, service requests with
/// [`Dsp::service_dma_read`], and route raised interrupts through its PIC. This crate does not
/// emulate the ISA DMA controller, PIC, analog filters, amplifier or OPL synthesizer.
pub struct Dsp {
    version: DspVersion,
    now_ns: u64,
    reset_high_since: Option<u64>,
    reset_ready_at: Option<u64>,
    reset_min_pulse_met: bool,
    uart_mode_during_reset: Option<MidiMode>,
    response: VecDeque<u8>,
    pending_command: Option<PendingCommand>,
    parameters: Vec<u8>,
    write_busy_until_ns: u64,
    write_busy_ns: u64,
    speaker_on: bool,
    time_constant: u8,
    output_rate_hz: u32,
    input_rate_hz: u32,
    rate_phase: u128,
    legacy_input_stereo: bool,
    legacy_output_stereo: bool,
    direct_sample: Option<u8>,
    direct_input_pending: bool,
    capture_input: VecDeque<u16>,
    active_transfer: Option<ActiveTransfer>,
    pending_dma: VecDeque<PendingDma>,
    adpcm_decoder: AdpcmDecoder,
    adpcm_ticks_until_next_byte: u8,
    block_size: u32,
    pause_samples: u32,
    midi_mode: MidiMode,
    midi_mode_before_uart: Option<MidiMode>,
    midi_output_armed: bool,
    midi_irq: bool,
    irq_8bit: bool,
    irq_16bit: bool,
    events: VecDeque<DspEvent>,
    stereo_next_right: bool,
}

impl Dsp {
    pub fn new(version: DspVersion) -> Self {
        Self {
            version,
            now_ns: 0,
            reset_high_since: None,
            reset_ready_at: None,
            reset_min_pulse_met: false,
            uart_mode_during_reset: None,
            response: VecDeque::new(),
            pending_command: None,
            parameters: Vec::new(),
            write_busy_until_ns: 0,
            write_busy_ns: 0,
            speaker_on: false,
            time_constant: 0xA5,
            output_rate_hz: 22_050,
            input_rate_hz: 22_050,
            rate_phase: 0,
            legacy_input_stereo: false,
            legacy_output_stereo: false,
            direct_sample: None,
            direct_input_pending: false,
            capture_input: VecDeque::new(),
            active_transfer: None,
            pending_dma: VecDeque::new(),
            adpcm_decoder: AdpcmDecoder::new(),
            adpcm_ticks_until_next_byte: 0,
            block_size: 0x1000,
            pause_samples: 0,
            midi_mode: MidiMode::Off,
            midi_mode_before_uart: None,
            midi_output_armed: false,
            midi_irq: false,
            irq_8bit: false,
            irq_16bit: false,
            events: VecDeque::new(),
            stereo_next_right: false,
        }
    }

    pub fn version(&self) -> DspVersion {
        self.version
    }

    pub fn speaker_on(&self) -> bool {
        self.speaker_on
    }

    pub fn sample_rate_hz(&self) -> u32 {
        self.output_rate_hz
    }

    pub fn input_sample_rate_hz(&self) -> u32 {
        self.input_rate_hz
    }

    pub fn transfer_active(&self) -> bool {
        self.active_transfer.is_some()
    }

    pub fn pending_dma_request_count(&self) -> usize {
        self.pending_dma.len()
    }

    pub fn irq_pending(&self) -> bool {
        self.irq_8bit || self.irq_16bit || self.midi_irq
    }

    pub fn interrupt_status(&self) -> u8 {
        ((self.irq_8bit || self.midi_irq) as u8) | ((self.irq_16bit as u8) << 1)
    }

    /// Configure how long the write-status port stays busy after an accepted command byte.
    /// The default is zero because Creative's public programming reference gives no duration.
    pub fn set_write_busy_ns(&mut self, duration_ns: u64) {
        self.write_busy_ns = duration_ns;
    }

    /// Set the CT1345-style output stereo switch, which is separate from the DSP on SB Pro.
    pub fn set_legacy_output_stereo(&mut self, enabled: bool) {
        self.legacy_output_stereo = enabled;
    }

    pub fn write_status(&self) -> u8 {
        if self.now_ns < self.write_busy_until_ns {
            0x80
        } else {
            0x00
        }
    }

    pub fn read_status(&self) -> u8 {
        if self.response.is_empty() {
            0x00
        } else {
            0x80
        }
    }

    /// Apply one DSP reset-port write. A sufficiently long high pulse schedules the 0xAA reply.
    pub fn write_reset_at(&mut self, value: u8, at_ns: u64) -> Result<(), TimeError> {
        self.advance_clock_to(at_ns)?;
        if value & 1 != 0 {
            if self.midi_mode.uart() {
                self.uart_mode_during_reset = Some(self.midi_mode);
                self.midi_mode = MidiMode::Off;
            } else {
                self.reset_device_state();
            }
            self.reset_high_since = Some(at_ns);
            self.reset_ready_at = None;
            self.reset_min_pulse_met = false;
        } else if let Some(started) = self.reset_high_since.take() {
            self.reset_min_pulse_met = at_ns.saturating_sub(started) >= RESET_MIN_NS;
            if self.reset_min_pulse_met {
                if self.uart_mode_during_reset.take().is_some() {
                    self.restore_after_uart_reset();
                } else {
                    self.reset_device_state();
                }
                self.reset_ready_at = Some(at_ns.saturating_add(RESET_READY_NS));
            } else if let Some(mode) = self.uart_mode_during_reset.take() {
                self.midi_mode = mode;
            }
        }
        Ok(())
    }

    /// Read one byte from DSP data port 2xAh. None means its response FIFO is empty.
    pub fn read_data_at(&mut self, at_ns: u64) -> Result<Option<u8>, TimeError> {
        self.advance_clock_to(at_ns)?;
        Ok(self.response.pop_front())
    }

    /// Read port 2xEh and acknowledge the backward-compatible 8-bit/MIDI DSP interrupt.
    pub fn read_buffer_status_at(&mut self, at_ns: u64) -> Result<u8, TimeError> {
        self.advance_clock_to(at_ns)?;
        self.irq_8bit = false;
        // Creative documents a read of this port as the SB-MIDI interrupt acknowledge.
        self.midi_irq = false;
        Ok(self.read_status())
    }

    /// Acknowledge the SB16 16-bit DMA interrupt by reading port 2xFh.
    pub fn acknowledge_16bit_irq_at(&mut self, at_ns: u64) -> Result<(), TimeError> {
        self.advance_clock_to(at_ns)?;
        self.irq_16bit = false;
        Ok(())
    }

    /// Advance the DSP sample clock and return all timed requests and samples due by `at_ns`.
    pub fn advance_to(&mut self, at_ns: u64) -> Result<Vec<DspEvent>, TimeError> {
        self.advance_clock_to(at_ns)?;
        Ok(self.events.drain(..).collect())
    }

    /// Inject a byte received from an attached MIDI device.
    pub fn push_midi_input_at(&mut self, value: u8, at_ns: u64) -> Result<(), TimeError> {
        self.advance_clock_to(at_ns)?;
        if self.midi_mode == MidiMode::Off {
            return Ok(());
        }
        if self.midi_mode.timestamp() {
            // The Creative reference specifies a wrapping 24-bit millisecond timestamp, but
            // does not state its epoch. This model uses the host virtual clock's zero point.
            let millis = (at_ns / 1_000_000) as u32;
            self.response.push_back(millis as u8);
            self.response.push_back((millis >> 8) as u8);
            self.response.push_back((millis >> 16) as u8);
        }
        self.response.push_back(value);
        if self.midi_mode.interrupt() {
            self.midi_irq = true;
            self.events.push_back(DspEvent::InterruptRaised {
                at_ns,
                source: InterruptSource::SbMidi,
            });
        }
        Ok(())
    }

    pub fn drain_events(&mut self) -> Vec<DspEvent> {
        self.events.drain(..).collect()
    }

    fn reset_device_state(&mut self) {
        self.response.clear();
        self.pending_command = None;
        self.parameters.clear();
        self.write_busy_until_ns = self.now_ns;
        self.speaker_on = false;
        self.time_constant = 0xA5;
        self.output_rate_hz = 22_050;
        self.input_rate_hz = 22_050;
        self.rate_phase = 0;
        self.legacy_input_stereo = false;
        self.legacy_output_stereo = false;
        self.direct_sample = None;
        self.direct_input_pending = false;
        self.active_transfer = None;
        self.pending_dma.clear();
        self.adpcm_decoder.reset();
        self.adpcm_ticks_until_next_byte = 0;
        self.pause_samples = 0;
        self.midi_mode = MidiMode::Off;
        self.midi_mode_before_uart = None;
        self.uart_mode_during_reset = None;
        self.midi_output_armed = false;
        self.midi_irq = false;
        self.irq_8bit = false;
        self.irq_16bit = false;
        self.stereo_next_right = false;
    }

    fn restore_after_uart_reset(&mut self) {
        self.response.clear();
        self.pending_command = None;
        self.parameters.clear();
        self.midi_mode = self.midi_mode_before_uart.take().unwrap_or(MidiMode::Off);
        self.midi_output_armed = false;
        self.midi_irq = false;
        self.irq_8bit = false;
        self.irq_16bit = false;
    }
}

/// A virtual-time call moved backwards, which would make DSP state ambiguous.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeError {
    pub previous_ns: u64,
    pub requested_ns: u64,
}
