//! Execute buffers: the vertices and the instruction stream a Direct3D 3 game writes into an
//! `IDirect3DExecuteBuffer`, laid out as the documentation's structures (`D3DTLVERTEX`,
//! `D3DINSTRUCTION`, `D3DTRIANGLE`, `D3DSTATE`, `D3DPROCESSVERTICES`), little-endian.
//!
//! [`Builder`] writes one, [`parse`] reads one back. A game's own buffer, dumped from memory,
//! parses the same way.

use crate::consts::op;

/// `D3DTLVERTEX`, 32 bytes: a vertex already transformed and lit. `sx`, `sy` are pixels,
/// `sz` the depth (0 near, 1 far), `rhw` one over the vertex's w; `color` and `specular` are
/// `D3DCOLOR` words `0xAARRGGBB` (the specular's alpha is the vertex's fog factor).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TlVertex {
    pub sx: f32,
    pub sy: f32,
    pub sz: f32,
    pub rhw: f32,
    pub color: u32,
    pub specular: u32,
    pub tu: f32,
    pub tv: f32,
}

impl TlVertex {
    pub const SIZE: usize = 32;

    pub fn to_bytes(&self) -> [u8; 32] {
        let mut b = [0u8; 32];
        let words = [
            self.sx.to_bits(),
            self.sy.to_bits(),
            self.sz.to_bits(),
            self.rhw.to_bits(),
            self.color,
            self.specular,
            self.tu.to_bits(),
            self.tv.to_bits(),
        ];
        for (i, w) in words.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }
        b
    }

    pub fn from_bytes(b: &[u8]) -> TlVertex {
        let w = |i: usize| u32::from_le_bytes([b[i * 4], b[i * 4 + 1], b[i * 4 + 2], b[i * 4 + 3]]);
        TlVertex {
            sx: f32::from_bits(w(0)),
            sy: f32::from_bits(w(1)),
            sz: f32::from_bits(w(2)),
            rhw: f32::from_bits(w(3)),
            color: w(4),
            specular: w(5),
            tu: f32::from_bits(w(6)),
            tv: f32::from_bits(w(7)),
        }
    }
}

/// `D3DEXECUTEDATA`'s offsets: where the vertices and the instructions are in the buffer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExecuteData {
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub instruction_offset: u32,
    pub instruction_length: u32,
}

/// One instruction of the stream, decoded.
#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    /// `D3DOP_TRIANGLE`: `D3DTRIANGLE`s, indices into the device's vertex list, and each
    /// one's `wFlags` (edge and strip hints, not used for drawing).
    Triangles(Vec<([u16; 3], u16)>),
    /// `D3DOP_STATERENDER`: `(D3DRENDERSTATETYPE, value)` pairs.
    RenderStates(Vec<(u32, u32)>),
    /// `D3DOP_PROCESSVERTICES`: flags, the first source vertex, the first destination vertex,
    /// the count.
    ProcessVertices { flags: u32, start: u16, dest: u16, count: u32 },
    /// `D3DOP_EXIT`.
    Exit,
    /// Any other opcode, kept as its raw records.
    Other { opcode: u8, size: u8, data: Vec<u8> },
}

/// Writes an execute buffer: the vertices first, then the instructions, as most games and
/// the SDK samples laid it out.
#[derive(Clone, Debug, Default)]
pub struct Builder {
    vertices: Vec<u8>,
    instructions: Vec<u8>,
    triangles_at: Option<usize>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder::default()
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / TlVertex::SIZE
    }

    /// Adds a vertex and returns its index.
    pub fn vertex(&mut self, v: TlVertex) -> u16 {
        let i = self.vertex_count() as u16;
        self.vertices.extend_from_slice(&v.to_bytes());
        i
    }

    fn header(&mut self, opcode: u8, size: u8, count: u16) {
        self.instructions.extend_from_slice(&[opcode, size]);
        self.instructions.extend_from_slice(&count.to_le_bytes());
    }

    fn close_triangles(&mut self) {
        if let Some(at) = self.triangles_at.take() {
            let n = (self.instructions.len() - at - 4) / 8;
            self.instructions[at + 2..at + 4].copy_from_slice(&(n as u16).to_le_bytes());
        }
    }

    /// `D3DOP_PROCESSVERTICES`.
    pub fn process_vertices(&mut self, flags: u32, start: u16, dest: u16, count: u32) {
        self.close_triangles();
        self.header(op::PROCESSVERTICES, 16, 1);
        for w in [flags, start as u32 | (dest as u32) << 16, count, 0] {
            self.instructions.extend_from_slice(&w.to_le_bytes());
        }
    }

    /// One `D3DOP_STATERENDER` with these states, in order.
    pub fn render_states(&mut self, states: &[(u32, u32)]) {
        if states.is_empty() {
            return;
        }
        self.close_triangles();
        self.header(op::STATERENDER, 8, states.len() as u16);
        for (t, v) in states {
            self.instructions.extend_from_slice(&t.to_le_bytes());
            self.instructions.extend_from_slice(&v.to_le_bytes());
        }
    }

    /// A triangle, added to the open `D3DOP_TRIANGLE` or starting one.
    pub fn triangle(&mut self, v: [u16; 3]) {
        let open = self
            .triangles_at
            .is_some_and(|at| (self.instructions.len() - at - 4) / 8 < u16::MAX as usize);
        if !open {
            self.close_triangles();
            self.triangles_at = Some(self.instructions.len());
            self.header(op::TRIANGLE, 8, 0);
        }
        for i in v {
            self.instructions.extend_from_slice(&i.to_le_bytes());
        }
        self.instructions.extend_from_slice(&0u16.to_le_bytes());
    }

    /// A convex polygon as a fan of triangles from its first vertex.
    pub fn fan(&mut self, v: &[u16]) {
        for i in 1..v.len().saturating_sub(1) {
            self.triangle([v[0], v[i], v[i + 1]]);
        }
    }

    /// The buffer, closed with `D3DOP_EXIT`, and its `D3DEXECUTEDATA`. The instructions start
    /// on a four-byte boundary after the vertices.
    pub fn finish(mut self) -> (Vec<u8>, ExecuteData) {
        self.close_triangles();
        self.header(op::EXIT, 0, 0);
        let data = ExecuteData {
            vertex_offset: 0,
            vertex_count: self.vertex_count() as u32,
            instruction_offset: self.vertices.len() as u32,
            instruction_length: self.instructions.len() as u32,
        };
        let mut buf = self.vertices;
        buf.extend_from_slice(&self.instructions);
        (buf, data)
    }
}

/// Reads the instruction stream `data` names in `buf`, up to `D3DOP_EXIT` or its end.
/// A record cut short by the end of the stream ends it.
pub fn parse(buf: &[u8], data: &ExecuteData) -> Vec<Instruction> {
    let start = data.instruction_offset as usize;
    let end = (start + data.instruction_length as usize).min(buf.len());
    let mut at = start;
    let mut out = Vec::new();
    let u16_at = |o: usize| u16::from_le_bytes([buf[o], buf[o + 1]]);
    let u32_at = |o: usize| u32::from_le_bytes([buf[o], buf[o + 1], buf[o + 2], buf[o + 3]]);
    while at + 4 <= end {
        let (opcode, size, count) = (buf[at], buf[at + 1] as usize, u16_at(at + 2) as usize);
        let body = at + 4;
        let next = body + size * count;
        if next > end {
            break;
        }
        let rec = |i: usize| body + i * size;
        out.push(match opcode {
            op::TRIANGLE if size >= 8 => Instruction::Triangles(
                (0..count)
                    .map(|i| ([u16_at(rec(i)), u16_at(rec(i) + 2), u16_at(rec(i) + 4)], u16_at(rec(i) + 6)))
                    .collect(),
            ),
            op::STATERENDER if size >= 8 => {
                Instruction::RenderStates((0..count).map(|i| (u32_at(rec(i)), u32_at(rec(i) + 4))).collect())
            }
            op::PROCESSVERTICES if size >= 16 && count >= 1 => {
                // One record is the documented use; further ones are kept as `Other`.
                let r = rec(0);
                Instruction::ProcessVertices {
                    flags: u32_at(r),
                    start: u16_at(r + 4),
                    dest: u16_at(r + 6),
                    count: u32_at(r + 8),
                }
            }
            op::EXIT => {
                out.push(Instruction::Exit);
                break;
            }
            _ => Instruction::Other {
                opcode,
                size: size as u8,
                data: buf[body..next].to_vec(),
            },
        });
        at = next;
    }
    out
}

/// The vertices `data` names in `buf`.
pub fn vertices(buf: &[u8], data: &ExecuteData) -> Vec<TlVertex> {
    let at = data.vertex_offset as usize;
    (0..data.vertex_count as usize)
        .map(|i| at + i * TlVertex::SIZE)
        .take_while(|o| o + TlVertex::SIZE <= buf.len())
        .map(|o| TlVertex::from_bytes(&buf[o..o + TlVertex::SIZE]))
        .collect()
}
