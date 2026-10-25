#[derive(Clone, Copy, Debug)]
pub(super) struct Envelope {
    enabled: bool,
    three_bit: bool,
    pending: u8,
    active: u8,
    level: u8,
    direction_up: bool,
    stopped: bool,
    clock_count: u8,
}

impl Envelope {
    pub const fn new() -> Self {
        Self {
            enabled: false,
            three_bit: false,
            pending: 0,
            active: 0,
            level: 0,
            direction_up: true,
            stopped: false,
            clock_count: 0,
        }
    }

    pub fn write(&mut self, value: u8) {
        let was_enabled = self.enabled;
        self.enabled = value & 0x80 != 0;
        self.three_bit = value & 0x10 != 0;
        // Clock source, inversion and waveform are buffered. Enable and resolution act directly.
        self.pending = value & 0x2f;
        if !was_enabled && self.enabled {
            self.active = self.pending;
            self.start(self.mode());
        } else if self.enabled && self.stopped && self.active != self.pending {
            // One-shot shapes have already reached Fig. 5 position (3), so buffered controls
            // written afterward take effect as soon as they arrive.
            self.apply_pending();
        }
        if !self.enabled {
            self.clock_count = 0;
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn external_clock(&self) -> bool {
        self.enabled && self.active & 0x20 != 0
    }

    pub fn internal_clock_divider(&self) -> u8 {
        // Philips specifies 977 Hz in four-bit mode and 1.95 kHz in three-bit mode from a
        // 7.81 kHz frequency-generator source.
        if self.three_bit {
            4
        } else {
            8
        }
    }

    pub fn clock_generator_cycle(&mut self) {
        if !self.enabled || self.external_clock() {
            return;
        }
        self.clock_count += 1;
        if self.clock_count >= self.internal_clock_divider() {
            self.clock_count = 0;
            self.clock();
        }
    }

    pub fn clock(&mut self) {
        if !self.enabled {
            return;
        }

        let previous_mode = self.mode();
        if self.advance(previous_mode) && self.active != self.pending {
            self.apply_pending();
        }
    }

    pub fn left_level(&self) -> u8 {
        self.level.min(self.maximum_level())
    }

    pub fn right_level(&self) -> u8 {
        let level = self.left_level();
        if self.active & 1 != 0 {
            self.maximum_level() - level
        } else {
            level
        }
    }

    pub fn denominator(&self) -> u8 {
        if self.three_bit {
            8
        } else {
            16
        }
    }

    fn mode(&self) -> u8 {
        (self.active >> 1) & 7
    }

    fn apply_pending(&mut self) {
        let previous_mode = self.mode();
        self.active = self.pending;
        if self.mode() != previous_mode {
            self.start(self.mode());
        }
    }

    fn start(&mut self, mode: u8) {
        self.direction_up = true;
        self.stopped = false;
        let maximum = self.maximum_level();
        self.level = match mode {
            0 | 4 | 5 | 6 | 7 => 0,
            1 | 2 | 3 => maximum,
            _ => unreachable!(),
        };
        if mode < 2 {
            self.stopped = true;
        }
    }

    /// Advance one envelope clock. Return true at a completed shape boundary so buffered
    /// waveform, clock-source and inversion bits can become active there.
    fn advance(&mut self, mode: u8) -> bool {
        if self.stopped || mode < 2 {
            return true;
        }
        let maximum = self.maximum_level();
        match mode {
            2 => {
                if self.level > 0 {
                    self.level -= 1;
                }
                if self.level == 0 {
                    self.stopped = true;
                    true
                } else {
                    false
                }
            }
            3 => {
                if self.level > 0 {
                    self.level -= 1;
                    self.level == 0
                } else {
                    self.level = maximum;
                    false
                }
            }
            4 => self.advance_triangle(false),
            5 => self.advance_triangle(true),
            6 => {
                if self.level < maximum {
                    self.level += 1;
                }
                if self.level == maximum {
                    self.stopped = true;
                    true
                } else {
                    false
                }
            }
            7 => {
                if self.level < maximum {
                    self.level += 1;
                    self.level == maximum
                } else {
                    self.level = 0;
                    false
                }
            }
            _ => unreachable!(),
        }
    }

    fn advance_triangle(&mut self, repeat: bool) -> bool {
        let maximum = self.maximum_level();
        if self.direction_up {
            if self.level < maximum {
                self.level += 1;
            } else {
                self.direction_up = false;
                self.level -= 1;
            }
            false
        } else if self.level > 0 {
            self.level -= 1;
            if self.level == 0 {
                self.direction_up = repeat;
                self.stopped = !repeat;
                true
            } else {
                false
            }
        } else {
            unreachable!("repeating triangles leave their lower boundary ready to rise")
        }
    }

    fn maximum_level(&self) -> u8 {
        if self.three_bit {
            7
        } else {
            15
        }
    }
}
