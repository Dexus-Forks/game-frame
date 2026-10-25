/// YMZ263B uses the high nibble first. The recurrence follows Yamaha's published Y8950
/// Application Manual. YMZ280B's Yamaha datasheet states that its 4-bit format is YMZ263B
/// compatible. The YMZ263B-specific predictor limits and output rounding still need a software
/// reference comparison.
#[derive(Clone, Copy)]
pub(crate) struct AdpcmState {
    predictor: i32,
    step: i32,
}

impl AdpcmState {
    pub(crate) const fn new() -> Self {
        Self {
            predictor: 0,
            step: 127,
        }
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::new();
    }

    pub(crate) fn decode(&mut self, code: u8) -> i16 {
        const STEP_FACTOR_TENTHS: [i32; 8] = [9, 9, 9, 9, 12, 16, 20, 24];
        let code = code & 0x0f;
        let magnitude = (code & 7) as i32;
        let difference = ((2 * magnitude + 1) * self.step) / 8;
        if code & 8 == 0 {
            self.predictor += difference;
        } else {
            self.predictor -= difference;
        }

        // The Y8950 manual describes a signed 16-bit predictor domain. YMZ263's 12-bit DAC
        // receives the upper 12 bits after centering its offset-binary internal representation.
        self.predictor = self.predictor.clamp(-32768, 32767);
        self.step = ((self.step * STEP_FACTOR_TENTHS[magnitude as usize]) / 10).clamp(127, 24576);
        (self.predictor >> 4).clamp(-2048, 2047) as i16
    }

    pub(crate) fn encode(&mut self, sample: i16) -> u8 {
        const STEP_FACTOR_TENTHS: [i32; 8] = [9, 9, 9, 9, 12, 16, 20, 24];
        let target = (sample as i32) << 4;
        let difference = target - self.predictor;
        let sign = i32::from(difference < 0);
        let magnitude = ((difference.abs() * 4) / self.step).clamp(0, 7);
        let code = ((sign << 3) | magnitude) as u8;
        let delta = ((2 * magnitude + 1) * self.step) / 8;
        if sign == 0 {
            self.predictor += delta;
        } else {
            self.predictor -= delta;
        }
        self.predictor = self.predictor.clamp(-32768, 32767);
        self.step = ((self.step * STEP_FACTOR_TENTHS[magnitude as usize]) / 10).clamp(127, 24576);
        code
    }
}
