//! A software triangle rasteriser for the replacement DLLs that stand in for 1997 3D chips
//! (ATI 3D Rage, S3 ViRGE): triangles with attributes interpolated across them, written into a
//! frame buffer of 8-, 15- or 16-bit pixels through a shader that sees the pixel already
//! there (for blending).
//!
//! Rules: a pixel is drawn when its centre (`x + 0.5`, `y + 0.5`) is inside the triangle, with
//! top-left edges in and bottom-right edges out, so two triangles that share an edge never
//! both draw a pixel. Attributes are interpolated linearly in screen space (*inferred* for
//! both chips: neither build asks for perspective correction). Whether the chips round
//! exactly so is *unknown*.

mod pixel;
mod tri;

pub use pixel::{Format, Target};
pub use tri::{Vertex, triangle, polygon};
