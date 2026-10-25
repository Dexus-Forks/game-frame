use super::{
    ActiveTransfer, DmaDirection, DmaRequest, Dsp, DspEvent, InterruptSource, PcmChannel,
    PcmSample, PendingDma, TimeError, TransferWidth,
};
use crate::adpcm::{AdpcmMode, AdpcmSamples};

const NS_PER_SECOND: u128 = 1_000_000_000;

impl Dsp {
    pub(super) fn advance_clock_to(&mut self, at_ns: u64) -> Result<(), TimeError> {
        if at_ns < self.now_ns {
            return Err(TimeError {
                previous_ns: self.now_ns,
                requested_ns: at_ns,
            });
        }
        let elapsed = at_ns - self.now_ns;
        if let Some(ready) = self.reset_ready_at {
            if self.now_ns < ready && at_ns >= ready && self.reset_min_pulse_met {
                self.response.push_back(0xAA);
                self.reset_ready_at = None;
            }
        }
        self.now_ns = at_ns;

        let rate = self.transfer_rate_hz();
        if rate != 0 && elapsed != 0 {
            let phase_before = self.rate_phase;
            let accumulated = self.rate_phase + elapsed as u128 * rate as u128;
            let samples = (accumulated / NS_PER_SECOND).min(1_000_000) as u32;
            self.rate_phase = accumulated % NS_PER_SECOND;
            for index in 0..samples {
                let numerator = NS_PER_SECOND * (index as u128 + 1) - phase_before;
                let sample_offset = ((numerator + rate as u128 - 1) / rate as u128) as u64;
                let sample_at = at_ns
                    .saturating_sub(elapsed)
                    .saturating_add(sample_offset.min(elapsed));
                self.sample_tick(sample_at);
            }
        }
        Ok(())
    }

    /// Respond to the oldest pending memory-to-card DMA read request.
    /// A returned event contains one decoded PCM word or one ADPCM byte with its expanded samples.
    pub fn service_dma_read(&mut self, value: u16) -> Option<DspEvent> {
        let pending = self.pending_dma.pop_front()?;
        let request = pending.request;
        let mut transfer = self.active_transfer?;
        if transfer.direction != DmaDirection::MemoryToCard {
            self.pending_dma.push_front(pending);
            return None;
        }
        let event = if let Some(bits) = transfer.adpcm_bits {
            let mode = AdpcmMode::from_bits(bits)?;
            let decoded = if pending.adpcm_reference_byte {
                AdpcmSamples {
                    values: [self.adpcm_decoder.set_reference(value as u8), 0, 0, 0],
                    count: 1,
                }
            } else {
                self.adpcm_decoder.decode_byte(mode, value as u8)
            };
            let mut samples = [0; 4];
            for (destination, source) in samples.iter_mut().zip(decoded.values) {
                *destination = if self.speaker_on {
                    ((source as i16) - 128) << 8
                } else {
                    0
                };
            }
            DspEvent::AdpcmByte {
                at_ns: request.at_ns,
                value: value as u8,
                bits_per_sample: bits,
                reference_byte: pending.adpcm_reference_byte,
                sample_rate_hz: self.transfer_rate_hz(),
                samples,
                sample_count: decoded.count,
            }
        } else {
            let decoded = match transfer.width {
                TransferWidth::EightBit if transfer.signed => (value as u8 as i8 as i16) << 8,
                TransferWidth::EightBit => ((value as u8 as i16) - 128) << 8,
                TransferWidth::SixteenBit if transfer.signed => value as i16,
                TransferWidth::SixteenBit => (value ^ 0x8000) as i16,
            };
            let channel = if transfer.stereo {
                let channel = if self.stereo_next_right {
                    PcmChannel::Right
                } else {
                    PcmChannel::Left
                };
                self.stereo_next_right = !self.stereo_next_right;
                channel
            } else {
                PcmChannel::Mono
            };
            DspEvent::Pcm(PcmSample {
                at_ns: request.at_ns,
                channel,
                value: if self.speaker_on { decoded } else { 0 },
            })
        };
        self.complete_transfer(&mut transfer, request.at_ns);
        self.active_transfer =
            if transfer.remaining == Some(0) && (!transfer.auto_init || transfer.exit_at_block) {
                self.adpcm_ticks_until_next_byte = 0;
                None
            } else {
                Some(transfer)
            };
        Some(event)
    }

    /// Supply the next captured input sample. Used by direct mode and card-to-memory DMA.
    pub fn push_capture_sample(&mut self, value: u16) {
        self.capture_input.push_back(value);
    }

    /// Notify the DSP that the configured ISA DMA controller reached terminal count.
    /// This is required by older auto-initialize modes whose command has no block-size operand.
    pub fn dma_terminal_count_at(&mut self, at_ns: u64) -> Result<(), TimeError> {
        self.advance_clock_to(at_ns)?;
        let Some(transfer) = self.active_transfer else {
            return Ok(());
        };
        if !transfer.auto_init {
            return Ok(());
        }
        self.raise_irq(transfer.width, at_ns);
        if transfer.exit_at_block {
            self.active_transfer = None;
            self.pending_dma.clear();
        }
        Ok(())
    }

    fn transfer_rate_hz(&self) -> u32 {
        let transfer = self.active_transfer;
        if let Some(transfer) = transfer {
            if transfer.paused {
                return 0;
            }
            if self.version.major >= 4 && !transfer.high_speed {
                let rate = match transfer.direction {
                    DmaDirection::MemoryToCard => self.output_rate_hz,
                    DmaDirection::CardToMemory => self.input_rate_hz,
                };
                return rate.saturating_mul(if transfer.stereo { 2 } else { 1 });
            }
            // The 1.xx–3.xx time constant represents the aggregate transfer rate. Stereo
            // callers encode two channels into this value using Creative's documented formula.
            return 1_000_000 / (256_u32 - self.time_constant as u32).max(1);
        }
        if self.direct_sample.is_some() || self.pause_samples > 0 {
            return 1_000_000 / (256_u32 - self.time_constant as u32).max(1);
        }
        if self.direct_input_pending {
            return if self.version.major >= 4 {
                self.input_rate_hz
            } else {
                1_000_000 / (256_u32 - self.time_constant as u32).max(1)
            };
        }
        0
    }

    fn sample_tick(&mut self, at_ns: u64) {
        if self.pause_samples > 0 {
            self.pause_samples -= 1;
            if self.pause_samples == 0 {
                self.raise_irq(TransferWidth::EightBit, at_ns);
            }
        }
        if let Some(value) = self.direct_sample.take() {
            self.events.push_back(DspEvent::Pcm(PcmSample {
                at_ns,
                channel: PcmChannel::Mono,
                value: if self.speaker_on {
                    ((value as i16) - 128) << 8
                } else {
                    0
                },
            }));
        }
        if self.direct_input_pending {
            self.direct_input_pending = false;
            let value = self.capture_input.pop_front().unwrap_or(0x80);
            self.response.push_back(value as u8);
        }
        let Some(transfer) = self.active_transfer else {
            return;
        };
        if transfer.paused {
            return;
        }
        let mut transfer = transfer;
        let adpcm_reference_byte = if let Some(bits) = transfer.adpcm_bits {
            if self.adpcm_ticks_until_next_byte > 0 {
                self.adpcm_ticks_until_next_byte -= 1;
                return;
            }
            let reference_byte = transfer.adpcm_reference_pending;
            transfer.adpcm_reference_pending = false;
            self.active_transfer = Some(transfer);
            self.adpcm_ticks_until_next_byte = if reference_byte {
                0
            } else {
                AdpcmMode::from_bits(bits)
                    .map(AdpcmMode::samples_per_byte)
                    .unwrap_or(1)
                    .saturating_sub(1)
            };
            reference_byte
        } else {
            false
        };
        let request = DmaRequest {
            at_ns,
            direction: transfer.direction,
            width: transfer.width,
        };
        match transfer.direction {
            DmaDirection::MemoryToCard => {
                self.pending_dma.push_back(PendingDma {
                    request,
                    adpcm_reference_byte,
                });
                self.events.push_back(DspEvent::DmaRequest(request));
            }
            DmaDirection::CardToMemory => {
                let value = self
                    .capture_input
                    .pop_front()
                    .unwrap_or(match transfer.width {
                        TransferWidth::EightBit => 0x80,
                        TransferWidth::SixteenBit => 0x8000,
                    });
                let value = if transfer.width == TransferWidth::EightBit {
                    value & 0xFF
                } else {
                    value
                };
                self.events.push_back(DspEvent::DmaWrite {
                    at_ns,
                    value,
                    width: transfer.width,
                });
                let mut active = transfer;
                self.complete_transfer(&mut active, at_ns);
                self.active_transfer =
                    if active.remaining == Some(0) && (!active.auto_init || active.exit_at_block) {
                        None
                    } else {
                        Some(active)
                    };
            }
        }
    }

    fn complete_transfer(&mut self, transfer: &mut ActiveTransfer, at_ns: u64) {
        if let Some(remaining) = transfer.remaining.as_mut() {
            *remaining = remaining.saturating_sub(1);
            if *remaining == 0 {
                self.raise_irq(transfer.width, at_ns);
                if transfer.auto_init && !transfer.exit_at_block {
                    *remaining = transfer.reload.unwrap_or(1);
                }
            }
        }
        if let Some(block_remaining) = transfer.block_remaining.as_mut() {
            *block_remaining = block_remaining.saturating_sub(1);
            if *block_remaining == 0 {
                if transfer.high_speed || transfer.auto_init {
                    self.raise_irq(transfer.width, at_ns);
                }
                if transfer.exit_at_block || (transfer.high_speed && !transfer.auto_init) {
                    transfer.remaining = Some(0);
                } else {
                    *block_remaining = self.block_size.max(1);
                }
            }
        }
    }

    fn raise_irq(&mut self, width: TransferWidth, at_ns: u64) {
        match width {
            TransferWidth::EightBit => self.irq_8bit = true,
            TransferWidth::SixteenBit => self.irq_16bit = true,
        }
        self.events.push_back(DspEvent::InterruptRaised {
            at_ns,
            source: match width {
                TransferWidth::EightBit => InterruptSource::EightBitDma,
                TransferWidth::SixteenBit => InterruptSource::SixteenBitDma,
            },
        });
    }
}
