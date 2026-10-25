const TIMER_TICK_CLOCKS: u64 = 32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TimerFlags {
    pub(crate) timer0: bool,
    pub(crate) timer1: bool,
    pub(crate) timer2: bool,
}

pub(crate) struct MmaTimers {
    control: u8,
    timer0_reload: u16,
    base_reload: u16,
    timer1_reload: u8,
    timer2_reload: u16,
    timer1_remaining: u8,
    timer2_remaining: u16,
    timer0_next: Option<u64>,
    base_next: Option<u64>,
    flags: TimerFlags,
    timer2_latch: Option<u16>,
}

impl MmaTimers {
    pub(crate) const fn new() -> Self {
        Self {
            control: 0,
            timer0_reload: 0,
            base_reload: 0,
            timer1_reload: 0,
            timer2_reload: 0,
            timer1_remaining: 0,
            timer2_remaining: 0,
            timer0_next: None,
            base_next: None,
            flags: TimerFlags {
                timer0: false,
                timer1: false,
                timer2: false,
            },
            timer2_latch: None,
        }
    }

    pub(crate) fn write_counter(&mut self, register: u8, value: u8) {
        match register {
            2 => self.timer0_reload = (self.timer0_reload & 0xff00) | value as u16,
            3 => self.timer0_reload = (self.timer0_reload & 0x00ff) | ((value as u16) << 8),
            4 => self.base_reload = (self.base_reload & 0x0f00) | value as u16,
            5 => {
                self.base_reload = (self.base_reload & 0x00ff) | (((value & 0x0f) as u16) << 8);
                self.timer1_reload = value >> 4;
            }
            6 => self.timer2_reload = (self.timer2_reload & 0xff00) | value as u16,
            7 => self.timer2_reload = (self.timer2_reload & 0x00ff) | ((value as u16) << 8),
            _ => {}
        }
    }

    pub(crate) fn read_counter(&mut self, register: u8) -> Option<u8> {
        match register {
            6 => Some(self.latch_timer2()),
            7 => Some(self.read_timer2_high()),
            _ => None,
        }
    }

    pub(crate) fn write_control(&mut self, value: u8, now: u64) {
        let old = self.control;
        self.control = value;
        for (mask, flag) in [
            (1, &mut self.flags.timer0),
            (2, &mut self.flags.timer1),
            (4, &mut self.flags.timer2),
        ] {
            if value & mask != 0 && old & mask == 0 {
                *flag = false;
            }
        }

        if value & 0x01 != 0 && old & 0x01 == 0 {
            self.timer0_next = Some(now.saturating_add(self.timer0_period()));
        } else if value & 0x01 == 0 {
            self.timer0_next = None;
        }

        if value & 0x02 != 0 && old & 0x02 == 0 {
            self.timer1_remaining = self.timer1_reload.max(1);
        }
        if value & 0x04 != 0 && old & 0x04 == 0 {
            self.timer2_remaining = self.timer2_reload.max(1);
        }

        if value & 0x08 != 0 && old & 0x08 == 0 {
            self.base_next = Some(now.saturating_add(self.base_period()));
        } else if value & 0x08 == 0 {
            self.base_next = None;
        }
    }

    pub(crate) fn next_event(&self) -> Option<u64> {
        match (self.timer0_next, self.base_next) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        }
    }

    pub(crate) fn advance_event(&mut self, clock: u64) {
        if self.timer0_next == Some(clock) {
            self.flags.timer0 = true;
            self.timer0_next = Some(clock.saturating_add(self.timer0_period()));
        }
        if self.base_next == Some(clock) {
            let period = self.base_period();
            self.base_next = Some(clock.saturating_add(period));
            if self.control & 0x02 != 0 {
                if self.timer1_remaining > 1 {
                    self.timer1_remaining -= 1;
                } else {
                    self.flags.timer1 = true;
                    self.timer1_remaining = self.timer1_reload.max(1);
                }
            }
            if self.control & 0x04 != 0 {
                if self.timer2_remaining > 1 {
                    self.timer2_remaining -= 1;
                } else {
                    self.flags.timer2 = true;
                    self.timer2_remaining = self.timer2_reload.max(1);
                }
            }
        }
    }

    pub(crate) fn flags(&self) -> TimerFlags {
        self.flags
    }

    pub(crate) fn control(&self) -> u8 {
        self.control
    }

    pub(crate) fn latch_timer2(&mut self) -> u8 {
        let value = self.timer2_remaining;
        self.timer2_latch = Some(value);
        value as u8
    }

    pub(crate) fn read_timer2_high(&mut self) -> u8 {
        let value = self.timer2_latch.take().unwrap_or(self.timer2_remaining);
        (value >> 8) as u8
    }

    fn timer0_period(&self) -> u64 {
        self.timer0_reload.max(1) as u64 * TIMER_TICK_CLOCKS
    }

    fn base_period(&self) -> u64 {
        self.base_reload.max(1) as u64 * TIMER_TICK_CLOCKS
    }
}
