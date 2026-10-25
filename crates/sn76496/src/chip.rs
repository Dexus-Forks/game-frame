/// Tandy 1000 and IBM PCjr SN76496 reference clock, in hertz.
/// Original: Tandy 1000 Service Manual, main logic schematic, sheet 9 of 12; IBM PCjr Technical
/// Reference, Sound Subsystem, pp. 2-89–2-94.
pub const TANDY_PSG_CLOCK_HZ: u32 = 3_579_545;
/// TI divides the reference clock by 16 before advancing the tone counters.
/// Original: IBM PCjr Technical Reference, Sound Subsystem, pp. 2-91–2-94.
pub const TONE_CLOCK_DIVIDER: u32 = 16;
// Original: IBM PCjr Technical Reference, Sound Subsystem, p. 2-94 (32 input clock cycles).
const READY_CLOCKS_PER_WRITE: u32 = 32;
const TONE_REGISTER_MASK: u16 = 0x03ff;
const NOISE_LFSR_MASK: u16 = 0x7fff;

/// One normalized mono output sample from the four PSG channels and its external input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sn76496Sample {
    pub mono: i16,
}

/// A deterministic SN76496 tone/noise generator.
///
/// `sample` advances one internal N/16 counter step. It produces samples at `clock_hz / 16`.
/// `advance_clock_cycles` advances chip and READY timing without producing host samples. The
/// chip's external input can be supplied with `set_audio_input`. Noise feedback taps and the
/// initial LFSR state are not specified by the TI and machine manuals; the current deterministic
/// candidate is recorded as provisional in the crate README.
#[derive(Clone, Debug)]
pub struct Sn76496 {
    clock_hz: u32,
    registers: [u16; 8],
    selected_register: u8,
    tone_counter: [u16; 3],
    tone_output: [bool; 3],
    clock_divider_phase: u8,
    noise_counter: u16,
    noise_lfsr: u16,
    noise_output: bool,
    ready_clocks_remaining: u32,
    audio_input: i16,
    samples_generated: u64,
}

impl Default for Sn76496 {
    fn default() -> Self {
        Self::new()
    }
}

impl Sn76496 {
    /// Construct the Tandy/PCjr clock profile.
    pub fn new() -> Self {
        Self::with_clock(TANDY_PSG_CLOCK_HZ)
    }

    /// Construct a PSG with the board's reference clock.
    pub fn with_clock(clock_hz: u32) -> Self {
        let mut registers = [0_u16; 8];
        registers[1] = 0x0f;
        registers[3] = 0x0f;
        registers[5] = 0x0f;
        registers[7] = 0x0f;
        Self {
            clock_hz: clock_hz.max(1),
            registers,
            selected_register: 0,
            tone_counter: [1; 3],
            tone_output: [true; 3],
            clock_divider_phase: 0,
            noise_counter: 0,
            noise_lfsr: 0,
            noise_output: false,
            ready_clocks_remaining: 0,
            audio_input: 0,
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
        f64::from(self.clock_hz) / f64::from(TONE_CLOCK_DIVIDER)
    }

    pub fn samples_generated(&self) -> u64 {
        self.samples_generated
    }

    /// True when the TI READY line would allow another data transfer.
    pub fn ready(&self) -> bool {
        self.ready_clocks_remaining == 0
    }

    /// Remaining input-clock cycles before the TI READY line returns high.
    pub fn ready_clocks_remaining(&self) -> u32 {
        self.ready_clocks_remaining
    }

    /// Supply the normalized signal presented to the SN76496 AUDIO IN pin.
    pub fn set_audio_input(&mut self, sample: i16) {
        self.audio_input = sample;
    }

    /// Advance the PSG by input-clock cycles without generating host audio samples.
    ///
    /// This advances both the tone/noise counters and the READY interval. It lets a machine
    /// model keep bus timing independent from its audio renderer.
    pub fn advance_clock_cycles(&mut self, clock_cycles: u32) {
        self.ready_clocks_remaining = self.ready_clocks_remaining.saturating_sub(clock_cycles);
        let accumulated = u64::from(self.clock_divider_phase) + u64::from(clock_cycles);
        let ticks = accumulated / u64::from(TONE_CLOCK_DIVIDER);
        self.clock_divider_phase = (accumulated % u64::from(TONE_CLOCK_DIVIDER)) as u8;
        for _ in 0..ticks {
            self.advance_psg_tick();
        }
    }

    /// Write one serial register byte. A byte with bit 7 set selects a register and supplies its
    /// low data bits. A continuation byte updates the six high bits only when a tone frequency
    /// register is selected. The 32-cycle READY interval advances with input-clock time.
    pub fn write_byte(&mut self, value: u8) {
        if value & 0x80 != 0 {
            self.selected_register = (value >> 4) & 7;
            let index = self.selected_register as usize;
            if index & 1 == 0 && index != 6 {
                self.registers[index] = (self.registers[index] & 0x03f0) | u16::from(value & 0x0f);
                self.normalize_tone(index / 2);
            } else {
                self.registers[index] = u16::from(value & 0x0f);
                if index == 6 {
                    self.reset_noise();
                }
            }
        } else if self.selected_register & 1 == 0 && self.selected_register != 6 {
            let index = self.selected_register as usize;
            self.registers[index] =
                (self.registers[index] & 0x000f) | (u16::from(value & 0x3f) << 4);
            self.normalize_tone(index / 2);
        }
        self.ready_clocks_remaining = READY_CLOCKS_PER_WRITE;
    }

    /// Return a register value for host diagnostics. Hardware reads from the sound port are not
    /// supported by the Tandy/PCjr interface.
    pub fn inspect_register(&self, register: u8) -> u16 {
        self.registers[(register & 7) as usize]
    }

    /// Generate one normalized mono sample after advancing one N/16 tone-counter clock.
    pub fn sample(&mut self) -> Sn76496Sample {
        self.advance_clock_cycles(TONE_CLOCK_DIVIDER);
        self.samples_generated = self.samples_generated.wrapping_add(1);
        self.current_sample()
    }

    /// Read the current normalized mono output without advancing the chip.
    pub fn current_sample(&self) -> Sn76496Sample {
        let mut channel_high = [false; 4];
        channel_high[..3].copy_from_slice(&self.tone_output);
        channel_high[3] = self.noise_output;

        let mut mixed = 0_i32;
        for (channel, high) in channel_high.into_iter().enumerate() {
            let attenuation = (self.registers[channel * 2 + 1] & 0x0f) as usize;
            let amplitude = i32::from(ATTENUATION_Q15[attenuation]) / 4;
            if amplitude != 0 {
                mixed += if high { amplitude } else { -amplitude };
            }
        }
        mixed += i32::from(self.audio_input);
        Sn76496Sample {
            mono: mixed.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        }
    }

    fn advance_psg_tick(&mut self) {
        let mut tone_3_rising = false;
        for channel in 0..3 {
            let period = self.registers[channel * 2] & TONE_REGISTER_MASK;
            if period == 0 {
                self.tone_output[channel] = true;
            } else if self.tone_counter[channel] <= 1 {
                self.tone_counter[channel] = period;
                self.tone_output[channel] = !self.tone_output[channel];
                if channel == 2 && self.tone_output[channel] {
                    tone_3_rising = true;
                }
            } else {
                self.tone_counter[channel] -= 1;
            }
        }

        self.advance_noise(tone_3_rising);
    }

    pub fn generate(&mut self, output: &mut [Sn76496Sample]) {
        for sample in output {
            *sample = self.sample();
        }
    }

    fn normalize_tone(&mut self, channel: usize) {
        if self.registers[channel * 2] == 0 {
            self.tone_counter[channel] = 0;
            self.tone_output[channel] = true;
        }
    }

    fn reset_noise(&mut self) {
        self.noise_lfsr = 0;
        self.noise_output = false;
        self.noise_counter = 0;
    }

    fn advance_noise(&mut self, tone_3_rising: bool) {
        let rate = self.registers[6] & 3;
        if rate == 3 {
            if tone_3_rising {
                self.shift_noise();
            }
            return;
        }

        let period = 32_u16 << rate;
        self.noise_counter += 1;
        if self.noise_counter >= period {
            self.noise_counter = 0;
            self.shift_noise();
        }
    }

    fn shift_noise(&mut self) {
        if self.noise_lfsr == 0 {
            // A non-zero seed prevents the documented feedback network from locking at zero.
            self.noise_lfsr = 1 << 14;
        }
        let output = self.noise_lfsr & 1 != 0;
        let white_noise = self.registers[6] & 4 != 0;
        let feedback = if white_noise {
            output ^ (self.noise_lfsr & 2 != 0)
        } else {
            output
        };
        self.noise_lfsr = (self.noise_lfsr >> 1) | (u16::from(feedback) << 14);
        self.noise_lfsr &= NOISE_LFSR_MASK;
        self.noise_output = output;
    }
}

// The data sheet specifies 2 dB attenuation steps, a maximum 28 dB setting, and OFF at 0xF.
const ATTENUATION_Q15: [i16; 16] = [
    32767, 26028, 20675, 16422, 13045, 10362, 8229, 6537, 5193, 4125, 3277, 2602, 2067, 1642, 1304,
    0,
];
