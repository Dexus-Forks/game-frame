use crate::Sn76496Sample;

/// Sound source selected by port 61h bits 5 and 6 on the IBM PCjr.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PcjrAudioSource {
    Timer2,
    Cassette,
    IoChannel,
    Psg,
}

/// The IBM PCjr four-source output mux controlled by port 61h bits 5 and 6.
///
/// The IBM PCjr reference assigns 00 to timer 2, 01 to cassette audio, 10 to I/O channel audio,
/// and 11 to the PSG. The original Tandy 1000 uses a different mux circuit, so this mapping does
/// not claim to model its other three selections. Other port 61h bits remain host responsibilities.
#[derive(Clone, Debug, Default)]
pub struct PcjrAudioMux {
    control_port_61: u8,
}

impl PcjrAudioMux {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the saved value written to port 61h.
    pub fn write_control_port_61(&mut self, value: u8) {
        self.control_port_61 = value;
    }

    pub fn control_port_61(&self) -> u8 {
        self.control_port_61
    }

    pub fn selected_source(&self) -> PcjrAudioSource {
        match (self.control_port_61 >> 5) & 0x03 {
            0 => PcjrAudioSource::Timer2,
            1 => PcjrAudioSource::Cassette,
            2 => PcjrAudioSource::IoChannel,
            _ => PcjrAudioSource::Psg,
        }
    }

    /// Route normalized mono source samples through the PCjr output mux.
    pub fn route_sample(
        &self,
        psg: Sn76496Sample,
        timer2: i16,
        cassette: i16,
        io_channel: i16,
    ) -> Sn76496Sample {
        let mono = match self.selected_source() {
            PcjrAudioSource::Timer2 => timer2,
            PcjrAudioSource::Cassette => cassette,
            PcjrAudioSource::IoChannel => io_channel,
            PcjrAudioSource::Psg => psg.mono,
        };
        Sn76496Sample { mono }
    }
}
