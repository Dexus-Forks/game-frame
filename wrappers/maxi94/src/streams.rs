//! The card's wave streams and the memory map that lists them (plain Rust, tested on the host).
//!
//! On the Maxi Sound 64 each `waveOutOpen` takes one of the SAM9407's wave streams, and the
//! card's memory map, read with `GetMemMapAddress` and `UpLoad`, lists the streams in use. POD
//! reads the map before and after it opens each of its two wave outputs, and takes the entry
//! that appeared as that output's stream number (*observed*, `PODMAX64.EXE`: `FUN_004b9d3c`
//! marks the streams already listed, `FUN_004b9cd4` returns the first new one, `FUN_004b9c98`
//! scans the map). The number is what it passes to `waveSetMainVolume` and `waveSetAuxVolume`.
//! With no new entry the game turns its sound off (`FUN_004ba304`: both answers must be other
//! than -1).
//!
//! The map as POD reads it (*observed*, from its scan): `UpLoad` copies `0xc0` 16-bit words,
//! 64 entries of 6 bytes. In each entry's first word the low byte, bit 7 masked off, is the
//! entry's kind, and `0x20` marks a wave stream; the high byte is the stream's number, 0 to 7
//! (the game keeps an 8-entry table by it, `0x6cd594`). The other kinds and the other four
//! bytes of each entry are *unknown*; this map leaves them zero.

/// Streams the card has (the game's table has 8 entries).
pub const STREAMS: usize = 8;
/// The 16-bit words of the map POD asks for.
pub const MAP_WORDS: usize = 0xc0;
/// An entry's kind: a wave stream in use.
pub const KIND_WAVE: u16 = 0x20;

/// One stream: the wave-out handle that holds it, and its volumes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stream {
    pub handle: usize,
    /// `waveSetMainVolume` (left, right), 0 to 0xffff.
    pub main: (u16, u16),
    /// `waveSetAuxVolume` (left, right), 0 to 0xffff.
    pub aux: (u16, u16),
}

#[derive(Default)]
pub struct Streams {
    slots: [Option<Stream>; STREAMS],
}

impl Streams {
    pub const fn new() -> Self {
        Self { slots: [None; STREAMS] }
    }

    /// A wave output was opened: it takes the lowest free stream. `None` when all are taken.
    pub fn open(&mut self, handle: usize) -> Option<usize> {
        let n = self.slots.iter().position(Option::is_none)?;
        self.slots[n] = Some(Stream { handle, main: (0xffff, 0xffff), aux: (0xffff, 0xffff) });
        Some(n)
    }

    /// A wave output was closed: its stream is free again.
    pub fn close(&mut self, handle: usize) -> Option<usize> {
        let n = self.slots.iter().position(|s| s.is_some_and(|s| s.handle == handle))?;
        self.slots[n] = None;
        Some(n)
    }

    /// The stream `handle` holds.
    pub fn stream_of(&self, handle: usize) -> Option<usize> {
        self.slots.iter().position(|s| s.is_some_and(|s| s.handle == handle))
    }

    pub fn get(&self, n: usize) -> Option<&Stream> {
        self.slots.get(n)?.as_ref()
    }

    pub fn get_mut(&mut self, n: usize) -> Option<&mut Stream> {
        self.slots.get_mut(n)?.as_mut()
    }

    /// The memory map: stream `n` in use is entry `n`, kind `KIND_WAVE`, number `n`.
    pub fn map(&self) -> [u16; MAP_WORDS] {
        let mut map = [0u16; MAP_WORDS];
        for (n, s) in self.slots.iter().enumerate() {
            if s.is_some() {
                map[n * 3] = (n as u16) << 8 | KIND_WAVE;
            }
        }
        map
    }
}

/// The volume a stereo output plays a stream at, as `waveOutSetVolume` takes it (left in the
/// low word). Deviation: the card sends the main volume to the front outputs and the aux
/// volume to the rear (*inferred*: POD sets one stream to main only and the other to aux only
/// when its setting at `0x4fdf70` is on, both to both when it is off; `FUN_004bbe0c`); with
/// two speakers both play, each at the louder of its two volumes.
pub fn stereo_volume(s: &Stream) -> u32 {
    let l = s.main.0.max(s.aux.0) as u32;
    let r = s.main.1.max(s.aux.1) as u32;
    r << 16 | l
}

#[cfg(test)]
mod tests {
    use super::*;

    /// POD's scan of the map, `FUN_004b9c98`: from entry `start` down, the first wave stream.
    fn scan(map: &[u16], mut start: i32) -> i32 {
        while start >= 0 {
            if map[start as usize * 3] & 0x7f == KIND_WAVE {
                return start;
            }
            start -= 1;
        }
        -1
    }

    /// `FUN_004b9d3c`: the table, -1 everywhere, then every listed stream marked.
    fn mark(map: &[u16]) -> [i32; 8] {
        let mut table = [-1; 8];
        let mut e = 0x3f;
        loop {
            e = scan(map, e - 1);
            if e == -1 {
                return table;
            }
            let n = (map[e as usize * 3] >> 8) as usize;
            if table[n] == -1 {
                table[n] = n as i32;
            }
        }
    }

    /// `FUN_004b9cd4`: the first listed stream the table does not have.
    fn new_stream(map: &[u16], table: &mut [i32; 8]) -> i32 {
        let mut e = 0x3f;
        loop {
            e = scan(map, e - 1);
            if e == -1 {
                return -1;
            }
            let n = (map[e as usize * 3] >> 8) as usize;
            if table[n] == -1 {
                table[n] = n as i32;
                return n as i32;
            }
        }
    }

    /// The game's start of sound, `FUN_004ba304`: two outputs, each found as a new stream.
    #[test]
    fn pod_finds_both_outputs() {
        let mut s = Streams::new();
        let mut table = mark(&s.map());
        s.open(0x1000);
        let a = new_stream(&s.map(), &mut table);
        let mut table = mark(&s.map());
        s.open(0x2000);
        let b = new_stream(&s.map(), &mut table);
        assert_eq!((a, b), (0, 1));
    }

    #[test]
    fn no_stream_opened_is_minus_one() {
        let s = Streams::new();
        let mut table = mark(&s.map());
        assert_eq!(new_stream(&s.map(), &mut table), -1);
    }

    /// The game stops its sound and starts it again (a new race): the streams are free again.
    #[test]
    fn closed_streams_are_reused() {
        let mut s = Streams::new();
        s.open(1);
        s.open(2);
        assert_eq!(s.stream_of(2), Some(1));
        assert_eq!(s.close(1), Some(0));
        assert_eq!(s.close(2), Some(1));
        assert_eq!(s.close(3), None);
        assert_eq!(s.map(), [0; MAP_WORDS]);
        assert_eq!(s.open(4), Some(0));
    }

    #[test]
    fn all_streams_taken() {
        let mut s = Streams::new();
        for h in 0..STREAMS {
            assert_eq!(s.open(h + 1), Some(h));
        }
        assert_eq!(s.open(99), None);
    }

    /// `FUN_004bbe0c` with the setting on: stream A main only, stream B aux only.
    #[test]
    fn front_and_rear_both_heard() {
        let a = Stream { handle: 1, main: (0x8000, 0x8000), aux: (0, 0) };
        let b = Stream { handle: 2, main: (0, 0), aux: (0x8000, 0x8000) };
        assert_eq!(stereo_volume(&a), 0x8000_8000);
        assert_eq!(stereo_volume(&b), 0x8000_8000);
        let quiet = Stream { handle: 3, main: (0, 0x10), aux: (0x20, 0) };
        assert_eq!(stereo_volume(&quiet), 0x0010_0020);
    }
}
