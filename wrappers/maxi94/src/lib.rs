//! `94DLL32.dll`: the Guillemot Maxi Sound 64's library ("Dream thunk for 9407 products",
//! Dream S.A. 1996: a 32-bit thunk down to `94DLL16.DLL` and the driver of the card's Dream
//! SAM9407 chip). POD Gold's two "64" builds (`PODMAX64.EXE`, `PODFXM64.EXE`) import six of
//! its calls by ordinal. Under Wine there is no card and no 16-bit driver behind the thunk.
//! This stand-in answers as a card whose wave streams are Wine's wave outputs, so the game's
//! sound plays through them.
//!
//! What POD does with the card (*observed*, `PODMAX64.EXE`, `srcson\maxi64\sndplay.cxx`):
//! - It plays all its sound itself, mixed into two stereo wave outputs at 22,050 Hz, 16 bits
//!   (`waveOutOpen` on the wave mapper, `FUN_004ba304`). It sends the card no MIDI and no
//!   sample banks (it does not import `DownLoad` or the `bank*` calls); the music is CD audio.
//! - It finds each output's stream on the card in the card's memory map (`streams`), and
//!   turns its sound off when it finds none. A stand-in with an empty map is silent for that
//!   reason.
//! - It sets each stream's main and aux volume (`FUN_004bbe0c`): the setting at `0x4fdf70`
//!   on, one stream main only and the other aux only; off, both streams both.
//! - `mpuMessageAc(0x60, &b, 1, 0)` with `b` = `0x7f` when its sound starts and `0` when it
//!   stops, retried up to 100 times until the answer's low word is non-zero (`FUN_004b9be4`).
//!   What command `0x60` does on the SAM9407 is *unknown*. The routines at `0x4b9bd0`,
//!   `0x4b9c24`, `0x4b9da4` and `0x4b9e14` send other commands (`0xff`, `0x3f`, `0x68` to
//!   `0x6f`) but nothing calls them.
//!
//! - `streams`: the stream table and the memory map (plain Rust, tested on the host).
//! - `mixer`: the mixer's wave and CD lines, which the game reads its volumes from and which
//!   Wine's mixer does not have (plain Rust, tested on the host); `mixer_dll` hooks the
//!   game's mixer calls to answer for them.
//! - the calls (`dll`): the six exports, and the game's `waveOutOpen` and `waveOutClose`
//!   hooked in its import table (`shim::iat`) so the stand-in knows which streams are open.
//!
//! All six are `__stdcall` (each ends `ret 0xc` or `ret 0x10` in the original DLL).

pub mod mixer;
pub mod streams;

#[cfg(windows)]
mod dll;
#[cfg(windows)]
mod mixer_dll;
