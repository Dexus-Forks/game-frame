use std::collections::VecDeque;

/// MIDI event emitted by the Sound Blaster 16's MPU-401 UART port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MpuEvent {
    pub at_ns: u64,
    pub value: u8,
}

/// The SB16's MPU-401 compatible UART interface. It models UART mode only.
pub struct Mpu401 {
    base: u16,
    uart_mode: bool,
    input: VecDeque<u8>,
    output: VecDeque<MpuEvent>,
    acknowledge: VecDeque<u8>,
    busy_until_ns: u64,
    write_busy_ns: u64,
    irq_pending: bool,
}

impl Mpu401 {
    pub fn new(base: u16) -> Self {
        Self {
            base,
            uart_mode: false,
            input: VecDeque::new(),
            output: VecDeque::new(),
            acknowledge: VecDeque::new(),
            busy_until_ns: 0,
            write_busy_ns: 0,
            irq_pending: false,
        }
    }

    pub fn base(&self) -> u16 {
        self.base
    }

    pub fn uart_mode(&self) -> bool {
        self.uart_mode
    }

    pub fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    /// Configure the output-ready interval after a write. The default is zero because the
    /// Creative programming reference specifies polling behavior but no byte timing.
    pub fn set_write_busy_ns(&mut self, duration_ns: u64) {
        self.write_busy_ns = duration_ns;
    }

    pub fn status(&self, at_ns: u64) -> u8 {
        let mut status = 0;
        if at_ns < self.busy_until_ns {
            status |= 0x40;
        }
        if self.input.is_empty() && self.acknowledge.is_empty() {
            status |= 0x80;
        }
        status
    }

    pub fn read_data(&mut self) -> Option<u8> {
        let value = self
            .acknowledge
            .pop_front()
            .or_else(|| self.input.pop_front());
        if value.is_some() {
            self.irq_pending = !self.input.is_empty();
        }
        value
    }

    pub fn write_data(&mut self, value: u8, at_ns: u64) -> Option<MpuEvent> {
        if !self.uart_mode {
            return None;
        }
        let event = MpuEvent { at_ns, value };
        self.output.push_back(event);
        self.busy_until_ns = at_ns.saturating_add(self.write_busy_ns);
        Some(event)
    }

    pub fn write_command(&mut self, value: u8, at_ns: u64) {
        if value == 0xFF {
            self.uart_mode = false;
            self.input.clear();
            self.irq_pending = false;
            self.acknowledge.push_back(0xFE);
        } else if value == 0x3F {
            self.uart_mode = true;
            self.acknowledge.push_back(0xFE);
            self.irq_pending = true;
        }
        self.busy_until_ns = at_ns.saturating_add(self.write_busy_ns);
    }

    pub fn push_input(&mut self, value: u8) {
        self.input.push_back(value);
        if self.uart_mode {
            self.irq_pending = true;
        }
    }

    pub fn drain_output(&mut self) -> Vec<MpuEvent> {
        self.output.drain(..).collect()
    }
}
