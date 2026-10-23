#![doc = include_str!("../README.md")]
#![doc = ""]
//! Texture and pixel formats of the PC 3D chips of 1997, and the API constants that name them.
//!
//! - [`texel`]: one decoder for every texel layout those chips used, to 8-bit RGBA.
//! - [`ncc`]: 3dfx's "narrow channel compression" (YIQ 4:2:2) tables.
//! - [`glide`]: 3dfx Glide 2.x formats, mip levels (LOD) and aspect ratios.
//! - [`ati`]: ATI 3D Rage ATI3DCIF texture formats, pixel formats and render-state ids.
//! - [`s3`]: S3 ViRGE S3D Toolkit texture and video formats, render types and state keys.
//! - [`card`]: what each chip family accepts, for tools that compare renderers.
//! - [`resample`]: the 2:1 point-sampled reduction loaders used to fit texture memory.
//!
//! Nothing here is specific to one game. Colour channels narrower than 8 bits widen by bit
//! replication (5 bits `abcde` become `abcdeabc`), as the chips' own colour paths did.

pub mod ati;
pub mod card;
pub mod glide;
pub mod ncc;
pub mod resample;
pub mod s3;
pub mod texel;

pub use texel::{Rgba, TexelFormat};
