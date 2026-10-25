use crate::{fifo::FixedFifo, MIDI_FIFO_CAPACITY, YMZ263_MASTER_CLOCK_HZ};

/// MIDI DIN serial rate used by the YMZ263B UART.
pub const MIDI_BAUD_RATE: u32 = 31_250;
const MIDI_FRAME_BITS: u64 = 10;

/// One completed byte transmitted by the chip's MIDI UART.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MidiLineEvent {
    /// YMZ263B master-clock timestamp at the end of the stop bit.
    pub clock: u64,
    /// Transmitted MIDI byte.
    pub value: u8,
}

pub(crate) struct MidiUnit {
    tx: FixedFifo<MIDI_FIFO_CAPACITY>,
    rx: FixedFifo<MIDI_FIFO_CAPACITY>,
    tx_active: Option<(u8, u64)>,
    tx_fraction: u64,
    tx_overrun: bool,
    rx_overrun: bool,
    events: Vec<MidiLineEvent>,
}

impl MidiUnit {
    pub(crate) const fn new() -> Self {
        Self {
            tx: FixedFifo::new(),
            rx: FixedFifo::new(),
            tx_active: None,
            tx_fraction: 0,
            tx_overrun: false,
            rx_overrun: false,
            events: Vec::new(),
        }
    }

    pub(crate) fn write(&mut self, value: u8, now: u64) {
        if self.tx.push(value).is_err() {
            self.tx_overrun = true;
        }
        self.start_next(now);
    }

    pub(crate) fn inject_rx(&mut self, value: u8) {
        if self.rx.push(value).is_err() {
            self.rx_overrun = true;
        }
    }

    pub(crate) fn read_rx(&mut self) -> Option<u8> {
        self.rx.pop()
    }

    pub(crate) fn reset_tx(&mut self) {
        self.tx.clear();
        self.tx_active = None;
        self.tx_fraction = 0;
        self.tx_overrun = false;
    }

    pub(crate) fn reset_rx(&mut self) {
        self.rx.clear();
        self.rx_overrun = false;
    }

    pub(crate) fn advance_to(&mut self, clock: u64) {
        while let Some((value, done_at)) = self.tx_active {
            if done_at > clock {
                break;
            }
            self.events.push(MidiLineEvent {
                clock: done_at,
                value,
            });
            self.tx_active = None;
            self.start_next(done_at);
        }
    }

    pub(crate) fn drain_events(&mut self) -> Vec<MidiLineEvent> {
        std::mem::take(&mut self.events)
    }

    pub(crate) fn next_event(&self) -> Option<(u8, u64)> {
        self.tx_active
    }

    pub(crate) fn rx_ready(&self) -> bool {
        !self.rx.is_empty()
    }

    pub(crate) fn tx_fifo_empty(&self) -> bool {
        self.tx.is_empty()
    }

    pub(crate) fn tx_overrun(&self) -> bool {
        self.tx_overrun
    }

    pub(crate) fn rx_overrun(&self) -> bool {
        self.rx_overrun
    }

    fn start_next(&mut self, now: u64) {
        if self.tx_active.is_some() {
            return;
        }
        let Some(value) = self.tx.pop() else {
            return;
        };
        // Carry the fractional master-clock remainder so successive 10-bit frames average the
        // documented 31,250 baud rate instead of accumulating a rounded per-byte error.
        let numerator = YMZ263_MASTER_CLOCK_HZ * MIDI_FRAME_BITS + self.tx_fraction;
        let frame_clocks = numerator / MIDI_BAUD_RATE as u64;
        self.tx_fraction = numerator % MIDI_BAUD_RATE as u64;
        self.tx_active = Some((value, now.saturating_add(frame_clocks)));
    }
}
