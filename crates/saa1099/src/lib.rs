#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

//! Philips SAA1099 programmable stereo sound generator used by the Creative Music System and
//! Game Blaster. The register model follows the Philips 1986 data handbook, pp. 150–153. The
//! card port model follows static traces of `CMSDRV.COM` and `CMS35.BIN`, documented in
//! `docs/hardware/cms-saa1099.md`.

mod chip;
mod cms;
mod envelope;
mod noise;

pub use chip::{Saa1099, Saa1099Sample, DEFAULT_CLOCK_HZ, OUTPUT_CLOCK_DIVIDER};
pub use cms::{CmsCard, CMS_DEFAULT_BASE_PORT, CMS_DEFAULT_CLOCK_HZ};
