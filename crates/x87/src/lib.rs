#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![no_std]

mod arith;
mod control;
mod ext;
mod int;
mod round;

pub use control::{Control, Precision, Rounding};
pub use ext::Ext;
