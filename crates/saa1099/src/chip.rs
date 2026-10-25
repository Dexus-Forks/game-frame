use crate::envelope::Envelope;
use crate::noise::NoiseGenerator;

/// Nominal SAA1099 master clock stated by Philips.
pub const DEFAULT_CLOCK_HZ: u32 = 8_000_000;
/// The output amplitude/envelope chopping rate is clock divided by this value.
pub const OUTPUT_CLOCK_DIVIDER: u32 = 128;
const PHASE_ONE: u64 = 1_u64 << 32;
const CHANNEL_SCALE: i32 = 128;

/// One normalized left/right output sample from the two six-channel current sinks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Saa1099Sample {
    pub left: i16,
    pub right: i16,
}

/// One Philips SAA1099 programmable stereo sound generator.
#[derive(Clone, Debug)]
pub struct Saa1099 {
    clock_hz: u32,
    selected_register: u8,
    registers: [u8; 32],
    phase: [u32; 6],
    active_tone: [u8; 6],
    active_octave: [u8; 6],
    noise: [NoiseGenerator; 2],
    envelopes: [Envelope; 2],
    sound_enabled: bool,
    reset_generators: bool,
    samples_generated: u64,
}

impl Default for Saa1099 {
    fn default() -> Self {
        Self::new()
    }
}

impl Saa1099 {
    pub fn new() -> Self {
        Self::with_clock(DEFAULT_CLOCK_HZ)
    }

    /// Construct a chip with a board-specific master clock.
    pub fn with_clock(clock_hz: u32) -> Self {
        Self {
            clock_hz: clock_hz.max(1),
            selected_register: 0,
            registers: [0; 32],
            phase: [0; 6],
            active_tone: [0; 6],
            active_octave: [0; 6],
            noise: [NoiseGenerator::new(); 2],
            envelopes: [Envelope::new(); 2],
            sound_enabled: false,
            reset_generators: false,
            samples_generated: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::with_clock(self.clock_hz);
    }

    pub fn clock_hz(&self) -> u32 {
        self.clock_hz
    }

    pub fn sample_rate_hz(&self) -> f64 {
        self.clock_hz as f64 / OUTPUT_CLOCK_DIVIDER as f64
    }

    pub fn samples_generated(&self) -> u64 {
        self.samples_generated
    }

    /// Load the SAA1099 address buffer. Only the low five bits select a register. An address
    /// write to envelope register 18h or 19h also clocks that envelope when external clocking is
    /// active. The data byte written afterward does not affect this pulse.
    pub fn write_address(&mut self, address: u8) {
        self.selected_register = address & 0x1f;
        if let 0x18..=0x19 = self.selected_register {
            let envelope = &mut self.envelopes[(self.selected_register - 0x18) as usize];
            if envelope.external_clock() {
                envelope.clock();
            }
        }
    }

    /// Write one byte to the selected register. The address remains selected for later writes.
    pub fn write_data(&mut self, value: u8) {
        let register = self.selected_register as usize;
        self.registers[register] = value;
        match self.selected_register {
            0x18..=0x19 => {
                self.envelopes[(self.selected_register - 0x18) as usize].write(value);
            }
            0x1c => {
                self.sound_enabled = value & 1 != 0;
                self.reset_generators = value & 2 != 0;
                if self.reset_generators {
                    self.phase = [0; 6];
                }
            }
            _ => {}
        }
    }

    /// Convenience method for a single address/data register write.
    pub fn write(&mut self, register: u8, value: u8) {
        self.write_address(register);
        self.write_data(value);
    }

    /// Return a register value for host diagnostics. The chip's external data bus is write-only.
    pub fn inspect_register(&self, register: u8) -> u8 {
        self.registers[(register & 0x1f) as usize]
    }

    /// Generate one output sample at `clock_hz / 128`.
    pub fn sample(&mut self) -> Saa1099Sample {
        let phase_step: [u64; 6] =
            core::array::from_fn(|channel| self.frequency_phase_step(channel));
        let mut wrapped = [false; 6];
        if !self.reset_generators {
            for channel in 0..6 {
                let previous = self.phase[channel];
                self.phase[channel] = self.phase[channel].wrapping_add(phase_step[channel] as u32);
                wrapped[channel] = self.phase[channel] < previous;
                if (previous ^ self.phase[channel]) & 0x8000_0000 != 0 {
                    // Philips buffers tone and octave writes until this generator changes state.
                    // The sample clock limits this model's transition timing to one output step.
                    self.active_tone[channel] = self.registers[0x08 + channel];
                    self.active_octave[channel] = self.programmed_octave(channel);
                }
            }
            if wrapped[1] {
                self.envelopes[0].clock_generator_cycle();
            }
            if wrapped[4] {
                self.envelopes[1].clock_generator_cycle();
            }
        }

        let noise_clock_0 = if self.reset_generators {
            0
        } else {
            phase_step[0]
        };
        let noise_clock_1 = if self.reset_generators {
            0
        } else {
            phase_step[3]
        };
        let noise_bits = [
            self.noise[0].advance(self.noise_phase_step(0, noise_clock_0)),
            self.noise[1].advance(self.noise_phase_step(1, noise_clock_1)),
        ];
        let mut left = 0_i32;
        let mut right = 0_i32;
        if self.sound_enabled {
            for channel in 0..6 {
                let noise_generator = usize::from(channel >= 3);
                let tone_is_noise_clock =
                    self.noise_control(noise_generator) == 3 && channel == noise_generator * 3;
                let tone_enabled =
                    !tone_is_noise_clock && self.registers[0x14] & (1 << channel) != 0;
                let noise_enabled = self.registers[0x15] & (1 << channel) != 0;
                let tone_level = self.phase[channel] & 0x8000_0000 != 0;
                let signal = match (tone_enabled, noise_enabled) {
                    (false, false) => None,
                    (true, false) => Some(tone_level),
                    (false, true) => Some(noise_bits[noise_generator]),
                    // The data sheet lists a combined mixer mode but does not state its logic.
                    // AND is the current inferred gate and remains an open comparison point.
                    (true, true) => Some(tone_level && noise_bits[noise_generator]),
                };
                let Some(high) = signal else {
                    continue;
                };
                let (left_envelope, right_envelope, envelope_denominator) =
                    self.channel_envelope(channel);
                let left_amplitude =
                    self.amplitude(channel, false, left_envelope, envelope_denominator);
                let right_amplitude =
                    self.amplitude(channel, true, right_envelope, envelope_denominator);
                let polarity = if high { 1 } else { -1 };
                left += polarity * left_amplitude * CHANNEL_SCALE;
                right += polarity * right_amplitude * CHANNEL_SCALE;
            }
        }
        self.samples_generated = self.samples_generated.wrapping_add(1);
        Saa1099Sample {
            left: left.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            right: right.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        }
    }

    pub fn generate(&mut self, output: &mut [Saa1099Sample]) {
        for sample in output {
            *sample = self.sample();
        }
    }

    fn frequency_phase_step(&self, channel: usize) -> u64 {
        let tone = self.active_tone[channel] as u64;
        let octave = self.active_octave[channel] as u32;
        let numerator = PHASE_ONE * (1_u64 << octave);
        numerator / (4 * (511 - tone))
    }

    fn programmed_octave(&self, channel: usize) -> u8 {
        let octave_register = self.registers[0x10 + channel / 2];
        if channel & 1 == 0 {
            octave_register & 7
        } else {
            (octave_register >> 4) & 7
        }
    }

    fn noise_control(&self, generator: usize) -> u8 {
        let shift = if generator == 0 { 0 } else { 4 };
        (self.registers[0x16] >> shift) & 3
    }

    fn noise_phase_step(&self, generator: usize, tone_phase_step: u64) -> u64 {
        match self.noise_control(generator) {
            0 => PHASE_ONE / 2,
            1 => PHASE_ONE / 4,
            2 => PHASE_ONE / 8,
            _ => tone_phase_step * 2,
        }
    }

    fn channel_envelope(&self, channel: usize) -> (Option<u8>, Option<u8>, Option<u8>) {
        match channel {
            1 if self.envelopes[0].enabled() => (
                Some(self.envelopes[0].left_level()),
                Some(self.envelopes[0].right_level()),
                Some(self.envelopes[0].denominator()),
            ),
            4 if self.envelopes[1].enabled() => (
                Some(self.envelopes[1].left_level()),
                Some(self.envelopes[1].right_level()),
                Some(self.envelopes[1].denominator()),
            ),
            _ => (None, None, None),
        }
    }

    fn amplitude(
        &self,
        channel: usize,
        right: bool,
        envelope: Option<u8>,
        envelope_denominator: Option<u8>,
    ) -> i32 {
        let packed = self.registers[channel];
        let amplitude = if right { packed >> 4 } else { packed & 0x0f };
        match envelope {
            Some(level) => {
                i32::from(amplitude & 0x0e) * i32::from(level)
                    / i32::from(envelope_denominator.unwrap_or(16))
            }
            None => i32::from(amplitude),
        }
    }
}
