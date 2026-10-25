use crate::{PortReadBehavior, Sn76496, Sn76496Sample, TANDY_PSG_CLOCK_HZ};

/// Approximate CPU wait states added by Tandy 1000 writes to the PSG ports.
///
/// The Tandy 1000 service manual describes about 42 wait states. The machine adapter decides
/// how those states map to its CPU clock and bus cycle.
pub const TANDY_PSG_WRITE_WAIT_STATES: u8 = 42;

/// Tandy 1000 SN76496 port adapter.
///
/// The sound chip is decoded for writes at ports C0h through C7h. The adapter forwards all eight
/// aliases to the same serial register interface. The service manual gives about 42 CPU wait
/// states per write. Reads do not select the PSG; the host bus model supplies its open-bus value.
/// PIT input and output-source selection remain host machine responsibilities.
#[derive(Clone, Debug)]
pub struct TandyPsgCard {
    chip: Sn76496,
}

impl Default for TandyPsgCard {
    fn default() -> Self {
        Self::new()
    }
}

/// IBM PCjr SN76496 port adapter.
///
/// The PCjr selects its PSG at port C0h. The port is write-only. PIT and machine output routing
/// remain responsibilities of the host machine model.
#[derive(Clone, Debug)]
pub struct PcjrPsgCard {
    chip: Sn76496,
}

impl Default for PcjrPsgCard {
    fn default() -> Self {
        Self::new()
    }
}

impl PcjrPsgCard {
    pub fn new() -> Self {
        Self::with_clock(TANDY_PSG_CLOCK_HZ)
    }

    pub fn with_clock(clock_hz: u32) -> Self {
        Self {
            chip: Sn76496::with_clock(clock_hz),
        }
    }

    /// Route a write to the PCjr PSG port.
    pub fn write_port(&mut self, port: u16, value: u8) -> bool {
        if port == 0x00c0 {
            self.chip.write_byte(value);
            true
        } else {
            false
        }
    }

    /// Report whether the PSG drives the bus for this read.
    pub fn read_port(&self, port: u16) -> PortReadBehavior {
        if port == 0x00c0 {
            PortReadBehavior::Undriven
        } else {
            PortReadBehavior::Unmapped
        }
    }

    pub fn chip(&self) -> &Sn76496 {
        &self.chip
    }

    pub fn chip_mut(&mut self) -> &mut Sn76496 {
        &mut self.chip
    }

    pub fn sample(&mut self) -> Sn76496Sample {
        self.chip.sample()
    }

    pub fn generate(&mut self, output: &mut [Sn76496Sample]) {
        self.chip.generate(output);
    }
}

impl TandyPsgCard {
    pub fn new() -> Self {
        Self::with_clock(TANDY_PSG_CLOCK_HZ)
    }

    pub fn with_clock(clock_hz: u32) -> Self {
        Self {
            chip: Sn76496::with_clock(clock_hz),
        }
    }

    /// Route an I/O write to any of the eight decoded PSG ports.
    pub fn write_port(&mut self, port: u16, value: u8) -> bool {
        if (0x00c0..=0x00c7).contains(&port) {
            self.chip.write_byte(value);
            true
        } else {
            false
        }
    }

    /// Report the machine behavior for an I/O read.
    pub fn read_port(&self, port: u16) -> PortReadBehavior {
        if (0x00c0..=0x00c7).contains(&port) {
            PortReadBehavior::Undriven
        } else {
            PortReadBehavior::Unmapped
        }
    }

    pub fn chip(&self) -> &Sn76496 {
        &self.chip
    }

    pub fn chip_mut(&mut self) -> &mut Sn76496 {
        &mut self.chip
    }

    pub fn set_timer2_audio_input(&mut self, sample: i16) {
        self.chip.set_audio_input(sample);
    }

    pub fn sample(&mut self) -> Sn76496Sample {
        self.chip.sample()
    }

    pub fn generate(&mut self, output: &mut [Sn76496Sample]) {
        self.chip.generate(output);
    }
}
