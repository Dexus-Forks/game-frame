use crate::{
    channel::{ChannelOutput, SamplingChannel, INTERLEAVE, PLAYBACK},
    midi::{MidiUnit, MIDI_BAUD_RATE},
    timers::{MmaTimers, TimerFlags},
    PCM_CHANNELS,
};
use std::collections::VecDeque;

// GSS.DRV initializes register 0x0d with 0x35 and pulses reset bits 1 and 3. The toolkit
// describes the four remaining controls as the digital/MIDI overrun, TRQ and RRQ masks.
const MIDI_MASK_DIGITAL_OVERRUN: u8 = 0x20;
const MIDI_MASK_OVERRUN: u8 = 0x10;
const MIDI_RESET_TX: u8 = 0x02;
const MIDI_MASK_TX_REQUEST: u8 = 0x04;
const MIDI_RESET_RX: u8 = 0x08;
const MIDI_MASK_RX_REQUEST: u8 = 0x01;

/// YMZ263B master clock specified in the Yamaha data sheet.
pub const YMZ263_MASTER_CLOCK_HZ: u64 = 16_934_400;
/// Minimum MMA port spacing, rounded up from the toolkit's 470 ns access delay.
pub const YMZ263_PORT_ACCESS_CLOCKS: u64 = 8;

/// Error in the chip's monotonic timestamp stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MmaAccessError {
    TimeWentBackwards { previous: u64, received: u64 },
    PortAccessTooSoon { wait_clocks: u8 },
}

/// Error returned when a caller selects a port outside the four-port MMA window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ymz263PortError {
    InvalidOffset(u8),
    Access(MmaAccessError),
}

impl From<MmaAccessError> for Ymz263PortError {
    fn from(value: MmaAccessError) -> Self {
        Self::Access(value)
    }
}

/// Result of a handled port write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MmaPortResult {
    /// Whether the chip currently asserts its active-low IRQ output.
    pub irq_pending: bool,
}

/// Digital output state from both sampling channels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MmaSampleOutput {
    pub channels: [ChannelOutput; PCM_CHANNELS],
}

/// A channel output update at a YMZ263B master-clock timestamp.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MmaOutputEvent {
    pub clock: u64,
    pub channel: u8,
    pub output: MmaSampleOutput,
}

/// Timestamped digital model for the YMZ263B multimedia audio and game interface controller.
///
/// Port offsets 0/1 address channel zero's index/data pair. Offsets 2/3 address channel one's
/// pair, matching the transactions issued by the Ad Lib Gold GSS Windows driver. The model does
/// not emulate the analog ADC/DAC filters, joystick RC timing, or external DMA controller.
pub struct Ymz263 {
    now: u64,
    last_access: Option<u64>,
    index: [u8; PCM_CHANNELS],
    common_registers: [u8; 16],
    channels: [SamplingChannel; PCM_CHANNELS],
    timers: MmaTimers,
    midi: MidiUnit,
    interleave_next: usize,
    output_events: VecDeque<MmaOutputEvent>,
}

impl Ymz263 {
    /// Create a chip with zeroed writable register state.
    pub fn new() -> Self {
        Self {
            now: 0,
            last_access: None,
            index: [0; PCM_CHANNELS],
            common_registers: [0; 16],
            channels: [SamplingChannel::new(), SamplingChannel::new()],
            timers: MmaTimers::new(),
            midi: MidiUnit::new(),
            interleave_next: 0,
            output_events: VecDeque::new(),
        }
    }

    /// Advance chip time and process every due PCM/ADPCM, timer, and MIDI event.
    pub fn advance_to(&mut self, master_clock: u64) -> Result<(), MmaAccessError> {
        if master_clock < self.now {
            return Err(MmaAccessError::TimeWentBackwards {
                previous: self.now,
                received: master_clock,
            });
        }

        loop {
            let mut next = self.timers.next_event();
            let interleaved = self.channels[0].format & INTERLEAVE != 0;
            for (index, channel) in self.channels.iter().enumerate() {
                if !interleaved || index == 0 {
                    if let Some(due) = channel.sample_due() {
                        next = Some(next.map_or(due, |current| current.min(due)));
                    }
                }
            }
            if let Some((_, midi_due)) = self.midi_next_event() {
                next = Some(next.map_or(midi_due, |current| current.min(midi_due)));
            }
            let Some(due) = next.filter(|due| *due <= master_clock) else {
                break;
            };

            self.timers.advance_event(due);
            self.midi.advance_to(due);
            if self.channels[0].format & INTERLEAVE != 0 {
                if self.channels[0].sample_due() == Some(due) {
                    let control = self.channels[0].control;
                    self.channels[0].reschedule(due, control);
                    if self.channels[0].dma_enabled() && self.channels[1].dma_enabled() {
                        let channel = self.interleave_next;
                        self.interleave_next ^= 1;
                        if let Some(event) =
                            self.channels[channel].sample_tick(due, channel as u8, control, false)
                        {
                            self.output_events.push_back(MmaOutputEvent {
                                clock: event.clock,
                                channel: event.channel,
                                output: self.sample_output(),
                            });
                        }
                    }
                }
            } else {
                for channel in 0..PCM_CHANNELS {
                    if self.channels[channel].sample_due() == Some(due) {
                        let control = self.channels[channel].control;
                        if let Some(event) =
                            self.channels[channel].sample_tick(due, channel as u8, control, true)
                        {
                            self.output_events.push_back(MmaOutputEvent {
                                clock: event.clock,
                                channel: event.channel,
                                output: self.sample_output(),
                            });
                        }
                    }
                }
            }
        }

        self.midi.advance_to(master_clock);
        self.now = master_clock;
        Ok(())
    }

    /// Read one port in the four-port MMA block at the given chip-clock timestamp.
    pub fn read_port_at(
        &mut self,
        offset: u8,
        master_clock: u64,
    ) -> Result<Option<u8>, Ymz263PortError> {
        if offset > 3 {
            return Err(Ymz263PortError::InvalidOffset(offset));
        }
        self.advance_to(master_clock)?;
        self.check_spacing(master_clock)?;
        self.last_access = Some(master_clock);
        let channel = (offset / 2) as usize;
        if offset & 1 == 0 {
            return Ok(if channel == 0 {
                Some(self.status_byte())
            } else {
                None
            });
        }
        let register = self.index[channel];
        Ok(self.read_register(channel, register))
    }

    /// Write one port in the four-port MMA block at the given chip-clock timestamp.
    pub fn write_port_at(
        &mut self,
        offset: u8,
        value: u8,
        master_clock: u64,
    ) -> Result<MmaPortResult, Ymz263PortError> {
        if offset > 3 {
            return Err(Ymz263PortError::InvalidOffset(offset));
        }
        self.advance_to(master_clock)?;
        self.check_spacing(master_clock)?;
        self.last_access = Some(master_clock);
        let channel = (offset / 2) as usize;
        if offset & 1 == 0 {
            self.index[channel] = value & 0x0f;
        } else {
            self.write_register(channel, self.index[channel], value, master_clock);
        }
        Ok(MmaPortResult {
            irq_pending: self.irq_pending(),
        })
    }

    /// Current digital state of both channel DACs before card-level mixing.
    pub fn sample_output(&self) -> MmaSampleOutput {
        MmaSampleOutput {
            channels: [self.channels[0].output(), self.channels[1].output()],
        }
    }

    /// Drain sample updates produced since the previous drain.
    pub fn drain_output_events(&mut self) -> Vec<MmaOutputEvent> {
        self.output_events.drain(..).collect()
    }

    /// Drain MIDI bytes that have completed transmission on the UART line.
    pub fn drain_midi_events(&mut self) -> Vec<crate::MidiLineEvent> {
        self.midi.drain_events()
    }

    /// Add an externally received MIDI byte to the 16-byte UART receive FIFO.
    pub fn receive_midi_byte(
        &mut self,
        value: u8,
        master_clock: u64,
    ) -> Result<(), MmaAccessError> {
        self.advance_to(master_clock)?;
        self.midi.inject_rx(value);
        Ok(())
    }

    /// Set the analog input sample supplied to the two recording channels.
    ///
    /// The card's analog mixer, filter, ADC settling, and input routing are outside this crate.
    pub fn set_record_input(&mut self, channel: u8, sample: i16) -> Result<(), u8> {
        let Some(channel) = self.channels.get_mut(channel as usize) else {
            return Err(channel);
        };
        channel.set_record_input(sample);
        Ok(())
    }

    /// Current FIFO fill level for one PCM/ADPCM channel.
    pub fn fifo_level(&self, channel: u8) -> Option<usize> {
        self.channels
            .get(channel as usize)
            .map(|channel| channel.fifo_level())
    }

    /// External DMA request line for one channel.
    pub fn dma_request(&self, channel: u8) -> Option<bool> {
        let index = channel as usize;
        let channel = self.channels.get(index)?;
        let control = if index == 1 && self.channels[0].format & INTERLEAVE != 0 {
            self.channels[0].control
        } else {
            channel.control
        };
        Some(channel.dma_request_for(control))
    }

    /// Whether any unmasked MMA source currently requests the card interrupt input.
    pub fn irq_pending(&self) -> bool {
        let interleaved = self.channels[0].format & INTERLEAVE != 0;
        let channel_irq = self.channels.iter().enumerate().any(|(index, channel)| {
            let control = if index == 1 && interleaved {
                self.channels[0].control
            } else {
                channel.control
            };
            channel.fifo_irq_status_for(control) && !channel.irq_masked()
        });
        let TimerFlags {
            timer0,
            timer1,
            timer2,
        } = self.timers.flags();
        let timer_control = self.timers.control();
        let timer_irq = (timer0 && timer_control & 0x10 == 0)
            || (timer1 && timer_control & 0x20 == 0)
            || (timer2 && timer_control & 0x40 == 0);
        let midi_control = self.common_registers[0x0d];
        let digital_overrun = self.channels.iter().any(|channel| channel.overrun)
            && midi_control & MIDI_MASK_DIGITAL_OVERRUN == 0;
        let midi_overrun = (self.midi.rx_overrun() || self.midi.tx_overrun())
            && midi_control & MIDI_MASK_OVERRUN == 0;
        let midi_tx_irq = self.midi.tx_fifo_empty() && midi_control & MIDI_MASK_TX_REQUEST == 0;
        let midi_rx_irq = self.midi.rx_ready() && midi_control & MIDI_MASK_RX_REQUEST == 0;
        channel_irq || timer_irq || digital_overrun || midi_overrun || midi_tx_irq || midi_rx_irq
    }

    /// Current timestamp in YMZ263B master clocks.
    pub const fn now(&self) -> u64 {
        self.now
    }

    fn check_spacing(&self, clock: u64) -> Result<(), MmaAccessError> {
        if let Some(previous) = self.last_access {
            let elapsed = clock.saturating_sub(previous);
            if elapsed < YMZ263_PORT_ACCESS_CLOCKS {
                return Err(MmaAccessError::PortAccessTooSoon {
                    wait_clocks: (YMZ263_PORT_ACCESS_CLOCKS - elapsed) as u8,
                });
            }
        }
        Ok(())
    }

    fn read_register(&mut self, channel: usize, register: u8) -> Option<u8> {
        match register {
            0x06 | 0x07 if channel == 0 => self.timers.read_counter(register),
            0x0b => self.channels[channel].pop_byte(),
            0x0e if channel == 0 => self.midi.read_rx(),
            _ => None,
        }
    }

    fn write_register(&mut self, channel: usize, register: u8, value: u8, clock: u64) {
        if channel != 0 && (register < 0x09 || register > 0x0c) {
            return;
        }
        match register {
            0x02..=0x07 => {
                self.common_registers[register as usize] = value;
                self.timers.write_counter(register, value);
            }
            0x08 => {
                self.common_registers[0x08] = value;
                self.timers.write_control(value, clock);
            }
            0x09 => {
                self.channels[channel].write_control(value, clock);
                if self.channels[0].format & INTERLEAVE != 0 {
                    self.channels[1].next_sample = None;
                }
                if value & 0x80 != 0 {
                    self.interleave_next = 0;
                }
            }
            0x0a => self.channels[channel].set_volume(value),
            0x0b => {
                let control = if self.channels[0].format & INTERLEAVE != 0 {
                    self.channels[0].control
                } else {
                    self.channels[channel].control
                };
                if control & PLAYBACK != 0 && self.channels[channel].push_byte(value).is_err() {
                    self.channels[channel].overrun = true;
                }
            }
            0x0c => {
                let was_interleaved = self.channels[0].format & INTERLEAVE != 0;
                self.channels[channel].write_format(value, channel == 0);
                let interleaved = self.channels[0].format & INTERLEAVE != 0;
                if channel == 0 && interleaved {
                    self.channels[1].next_sample = None;
                    self.interleave_next = 0;
                } else if channel == 0 && was_interleaved && !interleaved {
                    let control = self.channels[1].control;
                    self.channels[1].reschedule(clock, control);
                }
            }
            0x0d => {
                self.common_registers[0x0d] = value;
                if value & MIDI_RESET_TX != 0 {
                    self.midi.reset_tx();
                }
                if value & MIDI_RESET_RX != 0 {
                    self.midi.reset_rx();
                }
            }
            0x0e => self.midi.write(value, clock),
            _ if register < 0x10 => self.common_registers[register as usize] = value,
            _ => {}
        }
    }

    fn status_byte(&self) -> u8 {
        let flags = self.timers.flags();
        let interleaved = self.channels[0].format & INTERLEAVE != 0;
        let channel1_control = if interleaved {
            self.channels[0].control
        } else {
            self.channels[1].control
        };
        let overrun = self.channels.iter().any(|channel| channel.overrun) || self.midi.rx_overrun();
        (u8::from(overrun) << 7)
            | (u8::from(flags.timer2) << 6)
            | (u8::from(flags.timer1) << 5)
            | (u8::from(flags.timer0) << 4)
            | (u8::from(self.midi.tx_fifo_empty()) << 3)
            | (u8::from(self.midi.rx_ready()) << 2)
            | (u8::from(self.channels[1].fifo_irq_status_for(channel1_control)) << 1)
            | u8::from(self.channels[0].fifo_irq_status_for(self.channels[0].control))
    }

    fn midi_next_event(&self) -> Option<(u8, u64)> {
        // The active byte is private to the UART. The next deadline is surfaced through its
        // helper so the main clock loop can preserve line timing.
        self.midi.next_event()
    }
}

impl Default for Ymz263 {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = assert!(MIDI_BAUD_RATE == 31_250);
