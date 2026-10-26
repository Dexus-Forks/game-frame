//! `grDrawPoint` and `grDrawLine`.
//!
//! *Inferred*: Glide 2.x draws points and lines with the triangle engine (the SST-1 has no
//! point or line command, SST-1 specification section 5.15), so they go through the whole pixel
//! pipeline. Deviation: how Glide shapes them is *unknown*; here a point is the square of the
//! pixel that holds its 12.4-snapped position, and a line a parallelogram one pixel across its
//! minor axis, centred on the line.

use crate::glide::Glide;
use crate::tri::Vertex;

#[allow(non_snake_case)]
impl Glide {
    /// `grDrawPoint(v)`.
    pub fn grDrawPoint(&mut self, v: &Vertex) {
        let snap = |c: f32| ((c * 16.0).round() / 16.0).floor();
        let (x, y) = (snap(v.x), snap(v.y));
        let at = |dx: f32, dy: f32| Vertex {
            x: x + dx,
            y: y + dy,
            ..*v
        };
        self.fan(&[at(0.0, 0.0), at(1.0, 0.0), at(1.0, 1.0), at(0.0, 1.0)]);
    }

    /// `grDrawLine(a, b)`.
    pub fn grDrawLine(&mut self, a: &Vertex, b: &Vertex) {
        let x_major = (b.x - a.x).abs() >= (b.y - a.y).abs();
        let off = |v: &Vertex, d: f32| {
            let (dx, dy) = if x_major { (0.0, d) } else { (d, 0.0) };
            Vertex {
                x: v.x + dx,
                y: v.y + dy,
                ..*v
            }
        };
        self.fan(&[off(a, -0.5), off(b, -0.5), off(b, 0.5), off(a, 0.5)]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn white(x: f32, y: f32) -> Vertex {
        Vertex {
            x,
            y,
            oow: 1.0,
            r: 255.0,
            g: 255.0,
            b: 255.0,
            a: 255.0,
            ..Vertex::default()
        }
    }

    #[test]
    fn a_point_is_one_pixel() {
        let mut g = Glide::open(4, 4, 1.0);
        g.grDitherMode(0);
        g.grDrawPoint(&white(2.3, 1.7));
        let lit: Vec<usize> = (0..16).filter(|&i| g.frame.colour[i] != 0).collect();
        assert_eq!(lit, [4 + 2]);
    }

    #[test]
    fn a_horizontal_line_is_one_row() {
        let mut g = Glide::open(8, 4, 1.0);
        g.grDitherMode(0);
        g.grDrawLine(&white(1.0, 2.0), &white(6.0, 2.0));
        let lit: Vec<usize> = (0..32).filter(|&i| g.frame.colour[i] != 0).collect();
        assert_eq!(lit, (17..22).collect::<Vec<_>>());
    }
}
