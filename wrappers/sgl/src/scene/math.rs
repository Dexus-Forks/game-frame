//! 3 × 4 affine matrices for SGL's transforms. A point is a column vector and `p' = M p`.
//! `sgl_translate`, `sgl_rotate` and `sgl_scale` multiply the transform being edited on the
//! right, as OpenGL does, so the last one named is the first applied to a point (*inferred*:
//! POD names a translation before the rotations of the same object, `FUN_004868c0`).

pub type V3 = [f32; 3];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat {
    /// Rows: `m[r] = [a, b, c, t]`.
    pub m: [[f32; 4]; 3],
}

impl Default for Mat {
    fn default() -> Self {
        Mat::IDENTITY
    }
}

impl Mat {
    pub const IDENTITY: Mat = Mat { m: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]] };

    pub fn translate(v: V3) -> Mat {
        let mut m = Mat::IDENTITY;
        for r in 0..3 {
            m.m[r][3] = v[r];
        }
        m
    }

    pub fn scale(v: V3) -> Mat {
        let mut m = Mat::IDENTITY;
        for r in 0..3 {
            m.m[r][r] = v[r];
        }
        m
    }

    /// A rotation by `angle` radians about `axis`, counter-clockwise looking down the axis
    /// towards the origin in a right-handed frame (*inferred*: SGL's sign is to be checked
    /// against the game's picture).
    pub fn rotate(axis: V3, angle: f32) -> Mat {
        let [x, y, z] = normalize(axis);
        let (s, c) = angle.sin_cos();
        let t = 1.0 - c;
        Mat {
            m: [
                [t * x * x + c, t * x * y - s * z, t * x * z + s * y, 0.0],
                [t * x * y + s * z, t * y * y + c, t * y * z - s * x, 0.0],
                [t * x * z - s * y, t * y * z + s * x, t * z * z + c, 0.0],
            ],
        }
    }

    /// `self * o`.
    pub fn mul(&self, o: &Mat) -> Mat {
        let mut r = [[0.0f32; 4]; 3];
        for (i, row) in r.iter_mut().enumerate() {
            for (j, v) in row.iter_mut().enumerate() {
                *v = (0..3).map(|k| self.m[i][k] * o.m[k][j]).sum::<f32>() + if j == 3 { self.m[i][3] } else { 0.0 };
            }
        }
        Mat { m: r }
    }

    pub fn point(&self, p: V3) -> V3 {
        let m = &self.m;
        [0, 1, 2].map(|r| m[r][0] * p[0] + m[r][1] * p[1] + m[r][2] * p[2] + m[r][3])
    }

    pub fn vector(&self, v: V3) -> V3 {
        let m = &self.m;
        [0, 1, 2].map(|r| m[r][0] * v[0] + m[r][1] * v[1] + m[r][2] * v[2])
    }

    /// The inverse of an affine matrix (the camera's view from its placement). A singular
    /// matrix gives the identity.
    pub fn inverse(&self) -> Mat {
        let m = &self.m;
        let a = [[m[0][0], m[0][1], m[0][2]], [m[1][0], m[1][1], m[1][2]], [m[2][0], m[2][1], m[2][2]]];
        let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1]) - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
        if det.abs() < 1e-20 {
            return Mat::IDENTITY;
        }
        let d = 1.0 / det;
        let inv = [
            [(a[1][1] * a[2][2] - a[1][2] * a[2][1]) * d, (a[0][2] * a[2][1] - a[0][1] * a[2][2]) * d, (a[0][1] * a[1][2] - a[0][2] * a[1][1]) * d],
            [(a[1][2] * a[2][0] - a[1][0] * a[2][2]) * d, (a[0][0] * a[2][2] - a[0][2] * a[2][0]) * d, (a[0][2] * a[1][0] - a[0][0] * a[1][2]) * d],
            [(a[1][0] * a[2][1] - a[1][1] * a[2][0]) * d, (a[0][1] * a[2][0] - a[0][0] * a[2][1]) * d, (a[0][0] * a[1][1] - a[0][1] * a[1][0]) * d],
        ];
        let t = [m[0][3], m[1][3], m[2][3]];
        let mut r = [[0.0f32; 4]; 3];
        for i in 0..3 {
            for j in 0..3 {
                r[i][j] = inv[i][j];
            }
            r[i][3] = -(inv[i][0] * t[0] + inv[i][1] * t[1] + inv[i][2] * t[2]);
        }
        Mat { m: r }
    }

    /// The matrix for normals: the inverse transpose of the 3 × 3 part, so a scaled object
    /// keeps its normals at right angles to its faces.
    pub fn normal_matrix(&self) -> Mat {
        let i = self.inverse();
        let mut r = Mat::IDENTITY;
        for a in 0..3 {
            for b in 0..3 {
                r.m[a][b] = i.m[b][a];
            }
        }
        r
    }
}

pub fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub fn normalize(v: V3) -> V3 {
    let l = dot(v, v).sqrt();
    if l > 1e-20 { [v[0] / l, v[1] / l, v[2] / l] } else { [0.0, 0.0, 1.0] }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: V3, b: V3) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < 1e-5)
    }

    #[test]
    fn inverse_undoes() {
        let m = Mat::translate([1.0, 2.0, 3.0]).mul(&Mat::rotate([0.3, 1.0, 0.2], 0.7)).mul(&Mat::scale([2.0, 2.0, 0.5]));
        let p = [0.4, -1.5, 9.0];
        assert!(close(m.inverse().point(m.point(p)), p));
    }

    #[test]
    fn last_named_applies_first() {
        // Translate then scale: the scale acts on the point before the translation.
        let m = Mat::translate([10.0, 0.0, 0.0]).mul(&Mat::scale([2.0, 2.0, 2.0]));
        assert!(close(m.point([1.0, 0.0, 0.0]), [12.0, 0.0, 0.0]));
    }
}
