//! The two source lines of the card's mixer that POD asks for, kept in memory when the
//! system's mixer does not give them as the card did (plain Rust, tested on the host).
//!
//! POD's "64" builds keep their sound-effect and music volumes in the mixer (*observed*,
//! `PODMAX64.EXE` `FUN_004b9710`): they open mixer 0, ask for the source line of component
//! type WAVEOUT (`0x1008`) and of type COMPACTDISC (`0x1005`), each line's VOLUME control
//! (`0x50030001`), and later read and set its value (`FUN_004b9ad4`, `FUN_004b9b40`). The
//! effect volume, read back from the wave line, is what the game gives each card stream
//! (`FUN_004bbe0c`). Without a wave line the game reads 0 and every stream is silent. The
//! card's driver had both lines. Wine's mixer has no CD line, and its wave line's volume
//! reads 0 (*measured*, Wine 11 on macOS, before the game sets it): the game then takes 0 as
//! its effect volume, writes it back and plays silent streams. So the stand-in answers for
//! both lines itself, as the card's driver did, and the system's mixer answers the rest.
//!
//! Deviation: the CD line's volume is kept and read back, and changes nothing: the CD's music
//! plays at the system's volume.

/// MIXERLINE_COMPONENTTYPE_SRC_COMPACTDISC.
pub const SRC_COMPACTDISC: u32 = 0x1005;
/// MIXERLINE_COMPONENTTYPE_SRC_WAVEOUT.
pub const SRC_WAVEOUT: u32 = 0x1008;
/// MIXERCONTROL_CONTROLTYPE_VOLUME.
pub const CONTROL_VOLUME: u32 = 0x5003_0001;
/// The volume control's range.
pub const VOLUME_MAX: u32 = 0xffff;

/// Line and control IDs this mixer gives out: kept apart from the system mixer's own.
const LINE_ID: u32 = 0x9407_0000;
const CONTROL_ID: u32 = 0x9407_8000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    pub component: u32,
    pub name: &'static str,
    pub volume: u32,
}

impl Line {
    pub fn line_id(&self) -> u32 {
        LINE_ID | self.component
    }
    pub fn control_id(&self) -> u32 {
        CONTROL_ID | self.component
    }
}

pub struct Mixer {
    lines: [Line; 2],
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}

impl Mixer {
    /// Both lines at full volume until the game sets them (*unknown*: what the card's driver
    /// started at; the game sets its own levels from its settings).
    pub const fn new() -> Self {
        Self {
            lines: [
                Line { component: SRC_WAVEOUT, name: "Wave", volume: VOLUME_MAX },
                Line { component: SRC_COMPACTDISC, name: "CD Audio", volume: VOLUME_MAX },
            ],
        }
    }

    pub fn by_component(&self, component: u32) -> Option<&Line> {
        self.lines.iter().find(|l| l.component == component)
    }

    pub fn by_line(&self, id: u32) -> Option<&Line> {
        self.lines.iter().find(|l| l.line_id() == id)
    }

    pub fn by_control_id(&self, id: u32) -> Option<&Line> {
        self.lines.iter().find(|l| l.control_id() == id)
    }

    pub fn by_control(&mut self, id: u32) -> Option<&mut Line> {
        self.lines.iter_mut().find(|l| l.control_id() == id)
    }
}

/// `name` as a NUL-terminated field of `N` bytes.
pub fn name_field<const N: usize>(name: &str) -> [u8; N] {
    let mut f = [0u8; N];
    let n = name.len().min(N - 1);
    f[..n].copy_from_slice(&name.as_bytes()[..n]);
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_found_by_type_and_id() {
        let mut m = Mixer::new();
        let wave = *m.by_component(SRC_WAVEOUT).unwrap();
        assert_eq!(m.by_line(wave.line_id()), Some(&wave));
        assert!(m.by_component(0x1004).is_none());
        let cd = m.by_component(SRC_COMPACTDISC).unwrap().control_id();
        m.by_control(cd).unwrap().volume = 0x1234;
        assert_eq!(m.by_component(SRC_COMPACTDISC).unwrap().volume, 0x1234);
        assert_eq!(m.by_component(SRC_WAVEOUT).unwrap().volume, VOLUME_MAX);
        assert!(m.by_control(wave.line_id()).is_none());
        assert_eq!(m.by_control_id(wave.control_id()), Some(&wave));
    }

    /// POD's percentage of the wave line, `FUN_004b9ad4`: (value - min) / ((max - min) / 100).
    #[test]
    fn full_volume_reads_as_100_percent() {
        let m = Mixer::new();
        let v = m.by_component(SRC_WAVEOUT).unwrap().volume;
        assert_eq!(v / (VOLUME_MAX / 100), 100);
    }

    #[test]
    fn names_are_terminated() {
        assert_eq!(&name_field::<4>("Wave"), b"Wav\0");
        assert_eq!(&name_field::<6>("CD")[..3], b"CD\0");
    }
}
