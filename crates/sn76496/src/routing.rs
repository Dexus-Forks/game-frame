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

/// External audio source selected by the original Tandy 1000's port 61h PB5/PB6 bits.
///
/// The Tandy 1000 MS-DOS Programmer's Reference Manual documents BIOS source codes 00 for PIT
/// channel 2, 10 for bus audio input, and 11 for the complex sound generator. Mapping those codes
/// directly to the port 61h select bits is inferred from the matching values and PPI wiring. The
/// manual does not describe the fourth analog-mux input selected by 01.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TandyAudioSource {
    Timer2,
    Unspecified,
    BusAudioInput,
    ComplexSoundGenerator,
}

/// Original Tandy 1000 external audio-output mux controlled by port 61h bits 5 and 6.
///
/// This mux drives the external Audio Out connector. The internal speaker has a separate path.
/// The complex sound generator source includes whatever signal the host routes to the SN76496
/// AUDIO IN pin. `route_sample` returns `None` for selection 01 because the manual does not
/// identify that mux input.
#[derive(Clone, Debug, Default)]
pub struct TandyAudioOutputMux {
    control_port_61: u8,
}

impl TandyAudioOutputMux {
    pub fn new() -> Self {
        Self::default()
    }

    /// Save the value written to port 61h.
    pub fn write_control_port_61(&mut self, value: u8) {
        self.control_port_61 = value;
    }

    pub fn control_port_61(&self) -> u8 {
        self.control_port_61
    }

    pub fn selected_source(&self) -> TandyAudioSource {
        match (self.control_port_61 >> 5) & 0x03 {
            0 => TandyAudioSource::Timer2,
            1 => TandyAudioSource::Unspecified,
            2 => TandyAudioSource::BusAudioInput,
            _ => TandyAudioSource::ComplexSoundGenerator,
        }
    }

    /// Route normalized source samples to the Tandy external Audio Out signal.
    ///
    /// The `complex_sound_generator` sample should be the SN76496 output, including its AUDIO IN
    /// mix when the host models the original Tandy PIT channel 2 connection.
    pub fn route_sample(
        &self,
        complex_sound_generator: Sn76496Sample,
        timer2: i16,
        bus_audio_input: i16,
    ) -> Option<Sn76496Sample> {
        let mono = match self.selected_source() {
            TandyAudioSource::Timer2 => timer2,
            TandyAudioSource::Unspecified => return None,
            TandyAudioSource::BusAudioInput => bus_audio_input,
            TandyAudioSource::ComplexSoundGenerator => complex_sound_generator.mono,
        };
        Some(Sn76496Sample { mono })
    }
}
