//! A Direct3D 3 device in software: an RGB565 render target with a depth buffer, the render
//! states, the texture handles and the vertex list that execute buffers draw from.

use crate::consts::*;
use crate::exec::{self, ExecuteData, Instruction, TlVertex};
use crate::pixel;
use crate::raster::{self, Point};
use crate::state::States;
use crate::texture::Texture;

/// What `execute` could not draw, counted, so a port can report it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Skipped {
    /// `D3DOP_PROCESSVERTICES` with transform or lighting (only `COPY` of TL vertices draws).
    pub transform: u32,
    /// Points, lines, spans, matrices, lights and the other opcodes not drawn.
    pub opcodes: u32,
    /// Triangles drawn in point or wireframe fill mode (drawn solid).
    pub fill_mode: u32,
    /// Triangles with table fog asked for (`FOGTABLEMODE`), drawn without it.
    pub table_fog: u32,
}

pub struct Device {
    /// The game's resolution (the viewport the vertices are in).
    pub width: usize,
    pub height: usize,
    /// The drawing's multiple of it; the target is `width × scale` by `height × scale`.
    pub scale: f32,
    pub target_w: usize,
    pub target_h: usize,
    /// RGB565 pixels.
    pub colour: Vec<u16>,
    pub depth: Vec<f32>,
    pub states: States,
    /// The device's vertex list, filled by `D3DOP_PROCESSVERTICES`.
    pub vertices: Vec<TlVertex>,
    textures: Vec<Option<Texture>>,
    /// The card's `D3DPTEXTURECAPS_TRANSPARENCY`: colour-keyed texels are not drawn.
    pub transparency: bool,
    pub skipped: Skipped,
}

impl Device {
    pub fn new(width: usize, height: usize, scale: f32) -> Device {
        let scale = scale.max(0.25);
        let (tw, th) = (
            ((width as f32 * scale).round() as usize).max(1),
            ((height as f32 * scale).round() as usize).max(1),
        );
        Device {
            width,
            height,
            scale,
            target_w: tw,
            target_h: th,
            colour: vec![0; tw * th],
            depth: vec![1.0; tw * th],
            states: States::default(),
            vertices: Vec::new(),
            textures: Vec::new(),
            transparency: false,
            skipped: Skipped::default(),
        }
    }

    /// A texture's handle (`IDirect3DTexture::GetHandle`), never 0.
    pub fn add_texture(&mut self, t: Texture) -> u32 {
        self.textures.push(Some(t));
        self.textures.len() as u32
    }

    /// Replaces the texture behind a handle (`IDirect3DTexture::Load`, or a locked surface
    /// written again).
    pub fn put_texture(&mut self, handle: u32, t: Texture) {
        if let Some(slot) = self.textures.get_mut(handle.wrapping_sub(1) as usize) {
            *slot = Some(t);
        }
    }

    pub fn texture(&self, handle: u32) -> Option<&Texture> {
        self.textures.get(handle.wrapping_sub(1) as usize)?.as_ref()
    }

    /// `IDirect3DViewport::Clear` (or a colour-fill and depth-fill `Blt`): the target to a
    /// 565 colour, the depth buffer to `z`.
    pub fn clear(&mut self, colour: u16, z: f32) {
        self.colour.fill(colour);
        self.depth.fill(z);
    }

    /// `IDirect3DDevice::Execute`: runs the buffer's instructions.
    pub fn execute(&mut self, buf: &[u8], data: &ExecuteData) {
        let source = exec::vertices(buf, data);
        for i in exec::parse(buf, data) {
            match i {
                Instruction::ProcessVertices { flags, start, dest, count } => {
                    if flags & PROCESS_OPMASK != PROCESS_COPY {
                        self.skipped.transform += 1;
                        continue;
                    }
                    let (s, d) = (start as usize, dest as usize);
                    let n = (count as usize).min(source.len().saturating_sub(s));
                    if self.vertices.len() < d + n {
                        self.vertices.resize(d + n, TlVertex::default());
                    }
                    self.vertices[d..d + n].copy_from_slice(&source[s..s + n]);
                }
                Instruction::RenderStates(list) => {
                    for (t, v) in list {
                        self.states.set(t, v);
                    }
                }
                Instruction::Triangles(list) => {
                    for (t, _) in list {
                        self.triangle(t);
                    }
                }
                Instruction::Exit => break,
                Instruction::Other { .. } => self.skipped.opcodes += 1,
            }
        }
    }

    fn point(&self, v: &TlVertex) -> Point {
        let c = |w: u32, s: u32| (w >> s & 0xff) as f32;
        let mut a = [0f32; 12];
        a[0] = v.tu;
        a[1] = v.tv;
        a[2..6].copy_from_slice(&[c(v.color, 16), c(v.color, 8), c(v.color, 0), c(v.color, 24)]);
        a[6..10].copy_from_slice(&[c(v.specular, 16), c(v.specular, 8), c(v.specular, 0), c(v.specular, 24)]);
        Point { x: v.sx * self.scale, y: v.sy * self.scale, z: v.sz, rhw: v.rhw, a }
    }

    /// One triangle of the vertex list, by indices; indices past the list are skipped.
    pub fn triangle(&mut self, t: [u16; 3]) {
        let Some(vs) = t.iter().map(|&i| self.vertices.get(i as usize).copied()).collect::<Option<Vec<_>>>() else {
            return;
        };
        let mut p = [self.point(&vs[0]), self.point(&vs[1]), self.point(&vs[2])];
        let area = raster::area(&p);
        let cull = self.states.get(rs::CULLMODE);
        if (cull == CULL_CCW && area < 0.0) || (cull == CULL_CW && area > 0.0) {
            return;
        }
        if self.states.get(rs::FILLMODE) != FILL_SOLID {
            self.skipped.fill_mode += 1;
        }
        if self.states.get(rs::FOGTABLEMODE) != FOG_NONE {
            self.skipped.table_fog += 1;
        }
        // Flat shading: the first vertex's colour and specular for the whole triangle.
        if self.states.get(rs::SHADEMODE) == SHADE_FLAT {
            let first = p[0].a;
            for q in p.iter_mut().skip(1) {
                q.a[2..10].copy_from_slice(&first[2..10]);
            }
        }
        let perspective = self.states.on(rs::TEXTUREPERSPECTIVE).then_some(2);
        let (tw, th) = (self.target_w, self.target_h);
        let handle = self.states.get(rs::TEXTUREHANDLE).wrapping_sub(1) as usize;
        let texture = self.textures.get(handle).and_then(Option::as_ref);
        let pipe = pixel::Pipe::new(&self.states, texture, self.transparency, self.scale);
        let (colour, depth) = (&mut self.colour, &mut self.depth);
        raster::triangle(p, tw, th, perspective, &mut |f| {
            let o = f.y * tw + f.x;
            if let Some((c, write_z)) = pipe.shade(&f, colour[o], depth[o]) {
                colour[o] = c;
                if write_z {
                    depth[o] = f.z;
                }
            }
        });
    }

    /// The target as RGBA8, each 565 field widened by repeating its top bits.
    pub fn picture(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.colour.len() * 4);
        for &p in &self.colour {
            let [r, g, b, _] = crate::texture::Format::Rgb565.unpack(p);
            out.extend_from_slice(&[r as u8, g as u8, b as u8, 255]);
        }
        out
    }
}
