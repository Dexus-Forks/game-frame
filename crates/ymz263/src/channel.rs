use crate::{adpcm::AdpcmState, fifo::FixedFifo, PCM_FIFO_CAPACITY};

const RESET: u8 = 0x80;
const RIGHT: u8 = 0x40;
const LEFT: u8 = 0x20;
const FREQUENCY: u8 = 0x18;
const PCM_MODE: u8 = 0x04;
pub(crate) const PLAYBACK: u8 = 0x02;
const GO: u8 = 0x01;
pub(crate) const INTERLEAVE: u8 = 0x80;
const FORMAT: u8 = 0x60;
const FIFO_LEVEL: u8 = 0x1c;
const FIFO_MASK: u8 = 0x02;
const DMA_ENABLE: u8 = 0x01;

/// PCM byte layout selected by register 0x0C bits 6-5.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum PcmDataFormat {
    /// One byte carries sample bits 11-4.
    #[default]
    EightBit = 0,
    /// Low byte first, then the low nibble of the second byte carries bits 11-8.
    TwelveBitLowFirst = 1,
    /// The high nibble of the first byte carries bits 3-0; the second byte carries bits 11-4.
    TwelveBitHighFirst = 2,
    /// Reserved. The YMZ263B ignores format 3.
    Reserved = 3,
}

impl PcmDataFormat {
    pub(crate) const fn from_register(value: u8) -> Self {
        match (value & FORMAT) >> 5 {
            0 => Self::EightBit,
            1 => Self::TwelveBitLowFirst,
            2 => Self::TwelveBitHighFirst,
            _ => Self::Reserved,
        }
    }
}

/// PCM or ADPCM sampling rate selected in register 0x09.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SamplingRate {
    /// PCM 44.1 kHz, ADPCM 22.05 kHz.
    Rate0,
    /// PCM 22.05 kHz, ADPCM 11.025 kHz.
    Rate1,
    /// PCM 11.025 kHz, ADPCM 7.35 kHz.
    Rate2,
    /// PCM 7.35 kHz, ADPCM 5.5125 kHz.
    Rate3,
}

impl SamplingRate {
    pub(crate) const fn from_register(value: u8) -> Self {
        match (value & FREQUENCY) >> 3 {
            0 => Self::Rate0,
            1 => Self::Rate1,
            2 => Self::Rate2,
            _ => Self::Rate3,
        }
    }

    /// Return the exact integral divider in YMZ263B master clocks for this channel mode.
    pub const fn master_clock_divider(self, pcm: bool) -> u64 {
        match (self, pcm) {
            (Self::Rate0, true) => 384,
            (Self::Rate1, true) => 768,
            (Self::Rate2, true) => 1536,
            (Self::Rate3, true) => 2304,
            (Self::Rate0, false) => 768,
            (Self::Rate1, false) => 1536,
            (Self::Rate2, false) => 2304,
            (Self::Rate3, false) => 3072,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChannelOutput {
    /// Signed, 12-bit sample held by this channel's DAC.
    pub sample: i16,
    /// Register 0x0A's digital output-level value.
    pub volume: u8,
    /// The channel is routed to the left output.
    pub left_enabled: bool,
    /// The channel is routed to the right output.
    pub right_enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SampleEvent {
    /// YMZ263B master-clock timestamp of the sample update.
    pub clock: u64,
    /// Zero-based sample channel.
    pub channel: u8,
    /// Decoded signed 12-bit sample.
    pub sample: i16,
}

pub(crate) struct SamplingChannel {
    pub(crate) control: u8,
    pub(crate) format: u8,
    pub(crate) volume: u8,
    pub(crate) fifo: FixedFifo<PCM_FIFO_CAPACITY>,
    pub(crate) output: ChannelOutput,
    pub(crate) next_sample: Option<u64>,
    pub(crate) adpcm: AdpcmState,
    pending_adpcm_nibble: Option<u8>,
    pending_record_nibble: Option<u8>,
    record_input: i16,
    pub(crate) overrun: bool,
}

impl SamplingChannel {
    pub(crate) const fn new() -> Self {
        Self {
            control: 0,
            format: 0,
            volume: 0,
            fifo: FixedFifo::new(),
            output: ChannelOutput {
                sample: 0,
                volume: 0,
                left_enabled: false,
                right_enabled: false,
            },
            next_sample: None,
            adpcm: AdpcmState::new(),
            pending_adpcm_nibble: None,
            pending_record_nibble: None,
            record_input: 0,
            overrun: false,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.control = 0;
        self.format = 0;
        self.volume = 0;
        self.fifo.clear();
        self.output = ChannelOutput::default();
        self.next_sample = None;
        self.adpcm.reset();
        self.pending_adpcm_nibble = None;
        self.pending_record_nibble = None;
        self.record_input = 0;
        self.overrun = false;
    }

    pub(crate) fn write_control(&mut self, value: u8, now: u64) {
        if value & RESET != 0 {
            self.reset();
            return;
        }
        let was_running = self.control & GO != 0;
        self.control = value & !RESET;
        self.update_output_route();
        let is_running = self.control & GO != 0;
        if !was_running && is_running {
            let divider = self.sample_divider();
            self.next_sample = Some(now.saturating_add(divider));
            self.adpcm.reset();
            self.pending_adpcm_nibble = None;
            self.pending_record_nibble = None;
        } else if was_running && !is_running {
            self.next_sample = None;
        }
    }

    pub(crate) fn write_format(&mut self, value: u8, channel_zero: bool) {
        let interleave = self.format & INTERLEAVE;
        self.format = value;
        if !channel_zero {
            self.format = (self.format & !INTERLEAVE) | interleave;
        }
    }

    pub(crate) fn set_volume(&mut self, value: u8) {
        self.volume = value;
        self.output.volume = value;
    }

    pub(crate) fn set_record_input(&mut self, sample: i16) {
        self.record_input = sample.clamp(-2048, 2047);
    }

    pub(crate) fn push_byte(&mut self, value: u8) -> Result<(), u8> {
        self.fifo.push(value)
    }

    pub(crate) fn pop_byte(&mut self) -> Option<u8> {
        self.fifo.pop()
    }

    pub(crate) fn dma_enabled(&self) -> bool {
        self.format & DMA_ENABLE != 0
    }

    pub(crate) fn dma_request_for(&self, control: u8) -> bool {
        self.dma_enabled() && self.fifo_irq_condition_for(control)
    }

    pub(crate) fn irq_masked(&self) -> bool {
        self.format & FIFO_MASK != 0
    }

    pub(crate) fn output(&self) -> ChannelOutput {
        self.output
    }

    pub(crate) fn fifo_level(&self) -> usize {
        self.fifo.len()
    }

    pub(crate) fn sample_due(&self) -> Option<u64> {
        self.next_sample
    }

    pub(crate) fn reschedule(&mut self, clock: u64, control: u8) {
        if control & GO == 0 {
            self.next_sample = None;
        } else {
            let pcm = control & PCM_MODE != 0;
            let divider = SamplingRate::from_register(control).master_clock_divider(pcm);
            self.next_sample = Some(clock.saturating_add(divider));
        }
    }

    pub(crate) fn sample_tick(
        &mut self,
        clock: u64,
        channel: u8,
        control: u8,
        schedule_next: bool,
    ) -> Option<SampleEvent> {
        if control & GO == 0 {
            if schedule_next {
                self.next_sample = None;
            }
            return None;
        }

        let pcm = control & PCM_MODE != 0;
        let divider = SamplingRate::from_register(control).master_clock_divider(pcm);
        if schedule_next {
            self.next_sample = Some(clock.saturating_add(divider));
        }

        if control & PLAYBACK == 0 {
            self.record_tick(pcm);
            return None;
        }

        let sample = if pcm {
            self.decode_pcm()
        } else {
            self.decode_adpcm()
        };
        if let Some(sample) = sample {
            self.output.sample = sample;
            self.output.volume = self.volume;
            Some(SampleEvent {
                clock,
                channel,
                sample,
            })
        } else {
            self.overrun = true;
            None
        }
    }

    fn sample_divider(&self) -> u64 {
        SamplingRate::from_register(self.control).master_clock_divider(self.control & PCM_MODE != 0)
    }

    fn update_output_route(&mut self) {
        self.output.left_enabled = self.control & LEFT != 0;
        self.output.right_enabled = self.control & RIGHT != 0;
    }

    pub(crate) fn fifo_irq_status_for(&self, control: u8) -> bool {
        self.fifo_irq_condition_for(control)
    }

    fn fifo_irq_condition_for(&self, control: u8) -> bool {
        let point = ((self.format & FIFO_LEVEL) >> 2) as usize;
        if point >= 7 {
            return false;
        }
        let threshold = 112 - 16 * point;
        if control & PLAYBACK == 0 {
            self.fifo.len() >= threshold
        } else {
            self.fifo.len() <= threshold
        }
    }

    fn decode_pcm(&mut self) -> Option<i16> {
        let format = PcmDataFormat::from_register(self.format);
        let byte_count = match format {
            PcmDataFormat::EightBit => 1,
            PcmDataFormat::TwelveBitLowFirst | PcmDataFormat::TwelveBitHighFirst => 2,
            PcmDataFormat::Reserved => return None,
        };
        if self.fifo.len() < byte_count {
            return None;
        }
        let raw = match format {
            PcmDataFormat::EightBit => (self.fifo.pop()? as u16) << 4,
            PcmDataFormat::TwelveBitLowFirst => {
                let low = self.fifo.pop()? as u16;
                let high = self.fifo.pop()? as u16;
                ((high & 0x0f) << 8) | low
            }
            PcmDataFormat::TwelveBitHighFirst => {
                let first = self.fifo.pop()? as u16;
                let high = self.fifo.pop()? as u16;
                ((high << 4) | (first >> 4)) & 0x0fff
            }
            PcmDataFormat::Reserved => unreachable!("reserved format was rejected above"),
        };
        Some(((raw << 4) as i16) >> 4)
    }

    fn decode_adpcm(&mut self) -> Option<i16> {
        let nibble = match self.pending_adpcm_nibble.take() {
            Some(nibble) => nibble,
            None => {
                let byte = self.fifo.pop()?;
                self.pending_adpcm_nibble = Some(byte & 0x0f);
                byte >> 4
            }
        };
        Some(self.adpcm.decode(nibble))
    }

    fn record_tick(&mut self, pcm: bool) {
        if pcm {
            let raw = (self.record_input as u16) & 0x0fff;
            let format = PcmDataFormat::from_register(self.format);
            let bytes: [u8; 2] = match format {
                PcmDataFormat::EightBit => [((self.record_input >> 4) as i8) as u8, 0],
                PcmDataFormat::TwelveBitLowFirst => [
                    raw as u8,
                    ((raw >> 8) as u8 & 0x0f) | if raw & 0x0800 != 0 { 0xf0 } else { 0 },
                ],
                PcmDataFormat::TwelveBitHighFirst => [((raw & 0x0f) << 4) as u8, (raw >> 4) as u8],
                PcmDataFormat::Reserved => return,
            };
            let count = usize::from(format != PcmDataFormat::EightBit);
            if self.fifo.len() + count + 1 > PCM_FIFO_CAPACITY {
                self.overrun = true;
            } else {
                let _ = self.fifo.push(bytes[0]);
                if count != 0 {
                    let _ = self.fifo.push(bytes[1]);
                }
            }
        } else {
            let code = self.adpcm.encode(self.record_input);
            match self.pending_record_nibble.take() {
                None => self.pending_record_nibble = Some(code),
                Some(first) => {
                    if self.fifo.push((first << 4) | code).is_err() {
                        self.overrun = true;
                    }
                }
            }
        }
    }
}
