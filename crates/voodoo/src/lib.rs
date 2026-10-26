#![doc = include_str!("../README.md")]
#![doc = ""]
//! A software model of the 3dfx Voodoo Graphics pixel pipeline, driven by Glide 2.x state.
//!
//! - [`formats`]: Glide's texture formats (`GR_TEXFMT_*`), levels (`GR_LOD_*`) and aspect
//!   ratios (`GR_ASPECT_*`); [`ncc`]: 3dfx's YIQ compression tables.
//! - [`state`]: the render state Glide's calls set, with `grSstOpen`'s defaults.
//! - [`combine`]: the colour and alpha combine units.
//! - [`texture`]: one TMU's texture sampling (point or bilinear, wrap or clamp).
//! - [`fog`]: table fog.
//! - [`frame`]: the 565 colour buffer with its dither, the depth buffer, gamma.
//! - [`tri`]: `grDrawTriangle` through all of it.
//! - [`glide`]: a software Glide 2.x context, the calls by their names, at any output scale.
//!
//! For ports of games whose 3dfx build should be drawn the way the card drew it, at any
//! size, with no Glide library and no GPU. Every place the model is not known to be exact
//! says so in a `Deviation:` comment.

pub mod combine;
pub mod consts;
pub mod fog;
pub mod formats;
pub mod frame;
pub mod glide;
pub mod ncc;
pub mod state;
pub mod texture;
pub mod tri;

pub use frame::Frame;
pub use glide::{Glide, TexInfo};
pub use state::{Combine, State};
pub use texture::Texture;
pub use tri::{Vertex, triangle};
