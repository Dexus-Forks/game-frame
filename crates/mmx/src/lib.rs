#![doc = include_str!("../README.md")]
#![doc = ""]
//! Exact semantics of the Intel MMX instruction set (Pentium with MMX technology, 1997).
//!
//! An MMX register holds 64 bits read as 8 bytes, 4 words, 2 doublewords or 1 quadword,
//! lane 0 in the low bits (memory order on x86). Every function here takes and returns a
//! [`Mm`] and does what the instruction of the same name does, bit for bit: wrapping and
//! saturating arithmetic, the multiplies, compares, packs, unpacks, logic and shifts
//! (shift counts above the lane width give 0, or the sign for arithmetic shifts).
//!
//! Use it to port hand-written MMX code and to check a port against emulator ground truth.
//! Nothing here is specific to one game.
//! Reference: Intel, *Intel Architecture MMX Technology Programmer's Reference Manual*
//! (order number 243007, 1996).

mod arith;
mod compare;
mod lanes;
mod logic;
mod pack;
mod shift;

pub use arith::*;
pub use compare::*;
pub use lanes::Mm;
pub use logic::*;
pub use pack::*;
pub use shift::*;
