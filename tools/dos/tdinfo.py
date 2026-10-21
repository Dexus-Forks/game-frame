#!/usr/bin/env python3
"""Read Borland Turbo Debugger symbols (TDINFO, signature FB52) from a DOS or NE executable.

Usage: tdinfo.py BINARY COMMAND [ARGS]

Commands:
  summary            where the symbols are, their version and counts, and the modules (units)
  labels             TSV for labels/names.tsv: every procedure and global as seg:off, name, evidence
  procs [TEXT]       procedures: address, size, module, parameters and locals with their types
  globals [TEXT]     global variables and typed constants: address, size, type
  types [TEXT]       named types; records show each field with its offset (summed from sizes)
  lines [MODULE]     line numbers: address -> source file and line
  json OUT           everything above as one JSON file
  rust TYPE ...      Rust structs and readers for record types (and arrays of records), with a
                     comment on every field giving its Pascal type and offset
  rust-reader        the small PascalReader module the generated readers use

Borland Pascal and Borland C++ write these symbols when a program is linked with debug
information (TPC /V, BPC /V, TLINK /v), either appended to the program or in a .TDS file. Many
shipped games kept them. They give the original names of procedures, globals, parameters,
locals and types, and a map from code addresses to source lines: run this before tracing
anything by hand, and load the labels into Ghidra (scripts/ghidra-run.sh BINARY labels ...).

Addresses use Ghidra's numbering: NE segment N is selector 0x1000 + 8 * (N - 1); in an MZ
program the symbol's segment is relative to the load image, which Ghidra loads at 0x1000.

Format notes (from reading real files; Borland's own description is not public in full):
  - The block starts with the 16-bit magic 0x52FB and a version word (0x0300 = 3.0). A CodeView
    style "NBxx" trailer at the end of the file may point to it.
  - Tables follow the header and its extension, in this order: symbols (9 bytes), modules (16),
    source files (6), line numbers (4), scopes (12), segments (16), correlations (8), types (8),
    members (5, one more than the header's count), then the name pool (zero-terminated names,
    numbered from 1).
  - A type is one 8-byte slot: id, name, size, then 3 bytes whose meaning depends on the id.
    Subranges and arrays take a second slot (bounds, index type); type numbers are slot numbers.
  - Record fields (members) store a name and a type but no offset: field offsets here are the
    running sum of field sizes (Pascal records are not padded). Treat them as *inferred* and
    check one against the code before relying on it. Variant records are not decoded.
It does not read Turbo Debugger 4+ (32-bit) symbols, CodeView or Microsoft/Watcom formats.
"""
import json
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ne import MZ_LOAD, load  # noqa: E402

TYPE_NAMES = {
    0x00: 'void', 0x01: 'lstr', 0x02: 'dstr', 0x03: 'string', 0x04: 'shortint', 0x05: 'integer',
    0x06: 'longint', 0x07: 'comp', 0x08: 'byte', 0x09: 'word', 0x0A: 'dword', 0x0B: 'uquad',
    0x0C: 'char', 0x0D: 'single', 0x0E: 'real', 0x0F: 'double', 0x10: 'extended', 0x11: 'bcd4',
    0x12: 'bcd8', 0x13: 'bcd10', 0x15: 'near pointer', 0x16: 'far pointer', 0x17: 'segment',
    0x1A: 'C array', 0x1C: 'array', 0x1E: 'record', 0x1F: 'union', 0x22: 'enum', 0x23: 'procedure',
    0x24: 'label', 0x25: 'set', 0x26: 'text', 0x27: 'file', 0x28: 'boolean', 0x29: 'enum',
    0x2A: 'pword', 0x2B: 'tbyte', 0x2D: 'object', 0x36: 'wordbool', 0x37: 'longbool',
}
TWO_SLOTS = {0x04, 0x05, 0x06, 0x08, 0x09, 0x0A, 0x0C, 0x1C}   # subranges and arrays
CLASSES = {0: 'static', 1: 'absolute', 2: 'local', 3: 'var', 4: 'register', 5: 'const',
           6: 'type', 7: 'tag'}


def find(data):
    """The file offset of the TDINFO header, or None."""
    if len(data) > 8 and data[-8:-6] == b'NB':
        base = len(data) - struct.unpack_from('<I', data, len(data) - 4)[0]
        for at in range(max(base, 0), min(base + 64, len(data) - 2)):
            if data[at:at + 2] == b'\xfbR':
                return at
    pos = data.find(b'\xfbR')
    while pos != -1:
        if valid(data, pos):
            return pos
        pos = data.find(b'\xfbR', pos + 1)
    return None


def valid(data, at):
    if at + 0x30 > len(data):
        return False
    version, names_size = struct.unpack_from('<HI', data, at + 2)
    nsym = struct.unpack_from('<H', data, at + 0x0E)[0]
    return version >> 8 in (2, 3) and 0 < names_size < len(data) - at and at + 0x30 + 9 * nsym < len(data)


class TDInfo:
    def __init__(self, path):
        self.exe = load(path)
        d = self.data = self.exe.data
        at = self.at = find(d)
        if at is None:
            raise SystemExit(f'{path}: no Turbo Debugger symbols (FB52) found')
        (_, self.version, names_size, n_names, n_types, n_members, n_symbols, _, n_modules, _,
         n_scopes, n_lines, n_files, n_segments, n_corr) = struct.unpack_from('<HHIHHHHHHHHHHHH', d, at)
        ext, = struct.unpack_from('<H', d, at + 0x2E)
        end = len(d) - 8 if d[-8:-6] == b'NB' else self._names_end(at, names_size)
        self.names = [''] + d[end - names_size:end].decode('latin-1').split('\0')
        p = at + 0x30 + ext
        def table(fmt, count):
            nonlocal p
            size = struct.calcsize(fmt)
            rows = [struct.unpack_from(fmt, d, p + size * i) for i in range(count)]
            p += size * count
            return rows
        self.symbols = table('<HHHHB', n_symbols)            # name, type, offset, segment, class bits
        self.modules = table('<HBBHHHHHH', n_modules)          # name, language, model, ...
        self.files = table('<HI', n_files)                     # name, timestamp
        self.lines = table('<HH', n_lines)                     # line, offset
        self.scopes = table('<HHHHHH', n_scopes)
        self.segments = table('<HHHHHHHH', n_segments)         # module, segment, offset, length, ...
        self.correlations = table('<HHHH', n_corr)             # segment record, file, first line, count
        self.types = table('<BHHBH', n_types)                  # id, name, size, byte, word
        self.type_raw_at = p - 8 * n_types
        self.members = table('<BHH', n_members + 1)            # flags, name, type
        self.counts = dict(names=n_names, types=n_types, members=n_members, symbols=n_symbols,
                           modules=n_modules, scopes=n_scopes, lines=n_lines, files=n_files,
                           segments=n_segments, correlations=n_corr)

    def _names_end(self, at, names_size):
        return len(self.data)   # appended at the very end when there is no NB trailer

    # ------------------------------------------------------------------ helpers

    def name(self, i):
        return self.names[i] if 0 < i < len(self.names) else ''

    def address(self, segment, offset):
        if self.exe.kind == 'NE':
            return f'{0x1000 + 8 * (segment - 1):04x}:{offset:04x}'
        return f'{MZ_LOAD + segment:04x}:{offset:04x}'

    def type(self, t):
        """Type number t (1-based) -> (id, name, size, byte, word), or None."""
        return self.types[t - 1] if 0 < t <= len(self.types) else None

    def type_slot(self, t):
        return self.data[self.type_raw_at + 8 * (t - 1):self.type_raw_at + 8 * t]

    def type_size(self, t):
        ty = self.type(t)
        return ty[2] if ty else 0

    def type_name(self, t, depth=0):
        ty = self.type(t)
        if ty is None:
            return f'type#{t}'
        tid, name, size, b, w = ty
        if name and depth:
            return self.name(name)
        kind = TYPE_NAMES.get(tid, f'type{tid:#x}')
        if depth > 4:
            return self.name(name) or kind
        if tid in (0x04, 0x05, 0x06, 0x08, 0x09, 0x0A, 0x0C) and t < len(self.types):
            lo, hi = struct.unpack('<ii', self.type_slot(t + 1))
            base = self.name(name) or kind
            return base if name else f'{lo}..{hi}'
        if tid == 0x1C and t < len(self.types):
            index, = struct.unpack_from('<H', self.type_slot(t + 1))
            return f'array[{self.type_name(index, depth + 1)}] of {self.type_name(w, depth + 1)}'
        if tid == 0x03:
            return f'string[{b}]' if b != 255 else 'string'
        if tid in (0x15, 0x16):
            return f'^{self.type_name(w, depth + 1)}' if w else 'pointer'
        if tid == 0x23:
            return 'function: ' + self.type_name(w, depth + 1) if w > 1 else 'procedure'
        if tid == 0x1E:
            return (self.name(name) if depth else '') or f'record({size} bytes)'
        return self.name(name) or kind

    def record_fields(self, t):
        """(offset, name, type, size) for a record type; offsets are inferred from sizes."""
        ty = self.type(t)
        if ty is None or ty[0] != 0x1E or not ty[4]:
            return []
        out, off, i = [], 0, ty[4] - 1
        while 0 <= i < len(self.members):
            flags, name, mt = self.members[i]
            if not flags & 0x40:
                msize = self.type_size(mt)
                out.append((off, self.name(name), self.type_name(mt, 1), msize))
                off += msize
            if flags & 0x80:
                break
            i += 1
        return out

    def module_of_symbol(self, index):
        """The unit of a data symbol: the unit of the nearest procedure before it in the symbol
        table (procedures have code addresses, which the segment records place exactly). This is
        inferred: the module records' own symbol indexes do not line up with the units."""
        for i in range(index - 1, 0, -1):
            name, t, off, seg, bits = self.symbols[i - 1]
            ty = self.type(t)
            if bits & 7 == 0 and seg and ty and ty[0] == 0x23:
                return self.module_of(seg, off)
        return ''

    def module_of(self, segment, offset):
        for m, seg, start, length, *_ in self.segments:
            if seg == segment and start <= offset < start + length:
                return self.name(self.modules[m - 1][0]) if 0 < m <= len(self.modules) else ''
        return ''

    def procedures(self):
        """Procedures: static symbols whose type is a procedure, with sizes from the scopes."""
        out = []
        for i, (name, t, off, seg, bits) in enumerate(self.symbols, 1):
            if bits & 7 == 0 and seg and self.type(t) and self.type(t)[0] == 0x23:
                out.append(dict(index=i, name=self.name(name), segment=seg, offset=off,
                                address=self.address(seg, off), module=self.module_of(seg, off),
                                type=self.type_name(t), far=bool(bits & 0x10)))
        # Sizes: the distance to the next procedure or the end of the module's code range.
        by_seg = {}
        for p in out:
            by_seg.setdefault(p['segment'], []).append(p)
        for seg, procs in by_seg.items():
            procs.sort(key=lambda p: p['offset'])
            ends = [start + length for _, s, start, length, *_ in self.segments if s == seg]
            for a, b in zip(procs, procs[1:] + [None]):
                limit = b['offset'] if b else max(ends or [a['offset']])
                inside = [e for e in ends if a['offset'] < e <= limit]
                a['size'] = (min(inside) if inside else limit) - a['offset']
        self._attach_locals(out)
        return out

    def _attach_locals(self, procs):
        """Parameters and locals: the scope whose symbols follow a procedure belong to it."""
        by_index = {p['index']: p for p in procs}
        for first, count, _, _, off, length in self.scopes:
            owner = None
            for p in procs:
                if p['offset'] == off and length:
                    owner = p
                    break
            if owner is None:
                continue
            for i in range(first, first + count):
                if not 0 < i <= len(self.symbols):
                    continue
                name, t, o, seg, bits = self.symbols[i - 1]
                cls = bits & 7
                if cls in (2, 3, 4):
                    rel = o - 0x10000 if o >= 0x8000 else o
                    kind = 'param' if bits & 0x08 else 'local'
                    if cls == 3:
                        kind = 'var param'
                    owner.setdefault('vars', []).append(
                        dict(name=self.name(name), kind=kind, bp=rel, type=self.type_name(t, 1)))
        return by_index

    def globals(self):
        out = []
        procs = {(p['segment'], p['offset']) for p in self.procedures()}
        for i, (name, t, off, seg, bits) in enumerate(self.symbols, 1):
            if bits & 7 == 0 and seg and (seg, off) not in procs and self.type(t) \
                    and self.type(t)[0] not in (0x23, 0x24):
                out.append(dict(name=self.name(name), address=self.address(seg, off), segment=seg,
                                offset=off, size=self.type_size(t), type=self.type_name(t, 1),
                                module=self.module_of_symbol(i)))
        return out

    def named_types(self):
        out = []
        for name, t, off, seg, bits in self.symbols:
            if bits & 7 == 6 and self.type(t):
                fields = self.record_fields(t)
                out.append(dict(name=self.name(name), type=t, size=self.type_size(t),
                                kind=TYPE_NAMES.get(self.type(t)[0], hex(self.type(t)[0])),
                                definition=self.type_name(t), fields=fields))
        return out

    def line_map(self):
        """(address, file, line) for every line record, through the correlations."""
        out = []
        for segrec, file, first, count in self.correlations:
            if not 0 < segrec <= len(self.segments):
                continue
            seg = self.segments[segrec - 1][1]
            fname = self.name(self.files[file - 1][0]) if 0 < file <= len(self.files) else '?'
            for line, off in self.lines[first - 1:first - 1 + count]:
                out.append((self.address(seg, off), fname, line))
        return out


def element_record(td, t):
    """The record type inside t: t itself, or the element of an array (of arrays) of records."""
    for _ in range(4):
        ty = td.type(t)
        if ty is None:
            return None
        if ty[0] == 0x1E:
            return t
        if ty[0] != 0x1C:
            return None
        t = ty[4]
    return None


def show_record(td, t, indent):
    rec = element_record(td, t)
    if rec is None:
        return
    fields = td.record_fields(rec)
    for off, name, ty, size in fields:
        print(f'{indent}+{off:#06x}  {name}: {ty}  ({size})')
    total, size = sum(f[3] for f in fields), td.type_size(rec)
    if total != size:
        print(f'{indent}(fields add up to {total} bytes but the record is {size}: a variant part or '
              f'padding; check offsets in the code)')


RUST_KEYWORDS = {'as', 'break', 'const', 'continue', 'crate', 'else', 'enum', 'extern', 'false', 'fn',
                 'for', 'if', 'impl', 'in', 'let', 'loop', 'match', 'mod', 'move', 'mut', 'pub', 'ref',
                 'return', 'self', 'static', 'struct', 'super', 'trait', 'true', 'type', 'unsafe', 'use',
                 'where', 'while', 'async', 'await', 'dyn', 'box', 'yield', 'macro', 'try', 'final'}
RUST_SCALARS = {0x04: ('i8', 'i8'), 0x05: ('i16', 'i16'), 0x06: ('i32', 'i32'), 0x08: ('u8', 'u8'),
                0x09: ('u16', 'u16'), 0x0A: ('u32', 'u32'), 0x0C: ('u8', 'u8'), 0x28: ('bool', 'boolean'),
                0x0E: ('Real48', 'real48'), 0x0D: ('f32', 'f32'), 0x0F: ('f64', 'f64')}

RUST_READER = """//! Reading Borland Pascal data as it is laid out in memory and in files written with BlockWrite:
//! little-endian integers, one-byte Boolean, 6-byte Real, string[N] as a length byte plus N
//! bytes. Generated by tools/dos/tdinfo.py rust-reader (reverse-engineer-games).

/// A Turbo Pascal 6-byte Real, kept as its bytes so values stay exact; `to_f64` converts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Real48(pub [u8; 6]);

impl Real48 {
    pub fn to_f64(self) -> f64 {
        let b = self.0;
        if b[0] == 0 {
            return 0.0;
        }
        let mut mant: u64 = 0;
        for i in (1..6).rev() {
            mant = (mant << 8) | b[i] as u64;
        }
        mant &= (1u64 << 39) - 1;
        let sign = if b[5] & 0x80 != 0 { -1.0 } else { 1.0 };
        sign * (1.0 + mant as f64 / (1u64 << 39) as f64) * 2f64.powi(b[0] as i32 - 129)
    }
}

/// A Pascal string[N]: `len` used bytes out of a fixed capacity. Bytes after `len` are kept, as
/// the original keeps them in memory.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PStr<const N: usize> {
    pub len: u8,
    pub bytes: [u8; N],
}

impl<const N: usize> PStr<N> {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..(self.len as usize).min(N)]
    }
    /// The text, reading each byte as one character (code page 437 is not mapped).
    pub fn to_string_lossy(&self) -> String {
        self.as_bytes().iter().map(|&b| b as char).collect()
    }
}

impl<const N: usize> Default for PStr<N> {
    fn default() -> Self {
        PStr { len: 0, bytes: [0; N] }
    }
}

impl<const N: usize> std::fmt::Debug for PStr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_string_lossy())
    }
}

/// An error: the data ended before a read finished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortRead {
    pub at: usize,
    pub wanted: usize,
}

/// A cursor over bytes laid out as Borland Pascal stores them.
pub struct PascalReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> PascalReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        PascalReader { data, pos: 0 }
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }
    pub fn seek(&mut self, pos: usize) {
        self.pos = pos;
    }
    pub fn bytes(&mut self, n: usize) -> Result<&'a [u8], ShortRead> {
        if self.pos + n > self.data.len() {
            return Err(ShortRead { at: self.pos, wanted: n });
        }
        let out = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(out)
    }
    pub fn array<const N: usize>(&mut self) -> Result<[u8; N], ShortRead> {
        let mut out = [0u8; N];
        out.copy_from_slice(self.bytes(N)?);
        Ok(out)
    }
    pub fn u8(&mut self) -> Result<u8, ShortRead> {
        Ok(self.bytes(1)?[0])
    }
    pub fn i8(&mut self) -> Result<i8, ShortRead> {
        Ok(self.u8()? as i8)
    }
    pub fn u16(&mut self) -> Result<u16, ShortRead> {
        Ok(u16::from_le_bytes(self.array()?))
    }
    pub fn i16(&mut self) -> Result<i16, ShortRead> {
        Ok(i16::from_le_bytes(self.array()?))
    }
    pub fn u32(&mut self) -> Result<u32, ShortRead> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    pub fn i32(&mut self) -> Result<i32, ShortRead> {
        Ok(i32::from_le_bytes(self.array()?))
    }
    pub fn f32(&mut self) -> Result<f32, ShortRead> {
        Ok(f32::from_le_bytes(self.array()?))
    }
    pub fn f64(&mut self) -> Result<f64, ShortRead> {
        Ok(f64::from_le_bytes(self.array()?))
    }
    /// Pascal Boolean: one byte; any value other than 0 is true (as `if b then` tests it).
    pub fn boolean(&mut self) -> Result<bool, ShortRead> {
        Ok(self.u8()? != 0)
    }
    pub fn real48(&mut self) -> Result<Real48, ShortRead> {
        Ok(Real48(self.array()?))
    }
    pub fn pstr<const N: usize>(&mut self) -> Result<PStr<N>, ShortRead> {
        let len = self.u8()?;
        Ok(PStr { len, bytes: self.array()? })
    }
    /// A fixed array of N values, each read by `f` (Pascal `array[a..b] of T`).
    pub fn arr<T, const N: usize>(
        &mut self,
        mut f: impl FnMut(&mut Self) -> Result<T, ShortRead>,
    ) -> Result<[T; N], ShortRead> {
        let mut err = None;
        let a: [Option<T>; N] = std::array::from_fn(|_| {
            if err.is_some() {
                return None;
            }
            match f(self) {
                Ok(v) => Some(v),
                Err(e) => {
                    err = Some(e);
                    None
                }
            }
        });
        match err {
            Some(e) => Err(e),
            None => Ok(a.map(|x| x.expect("read"))),
        }
    }
}
"""


def rust_ident(name):
    n = name.lower()
    return n + '_' if n in RUST_KEYWORDS else n


RUST_PRELUDE = {'Option', 'Result', 'Box', 'String', 'Vec', 'Some', 'None', 'Ok', 'Err', 'Self',
                'Copy', 'Clone', 'Default', 'Drop', 'Fn', 'Iterator', 'Send', 'Sized', 'Sync', 'Debug'}


def rust_struct_name(name):
    base = name[:-4] if name.lower().endswith('type') and len(name) > 4 else name
    base = base[:1].upper() + base[1:]
    return base + 'Rec' if base in RUST_PRELUDE else base   # Optiontype -> OptionRec


def rust_type(td, t, out_structs, hint):
    """(rust type, read expression) for type t; nested anonymous records become structs."""
    ty = td.type(t)
    tid, name, size, b, w = ty
    if tid in RUST_SCALARS:
        rt, method = RUST_SCALARS[tid]
        return rt, f'r.{method}()?'
    if tid == 0x03:
        return f'PStr<{size - 1}>', 'r.pstr()?'
    if tid == 0x1C:
        index, = struct.unpack_from('<H', td.type_slot(t + 1))
        lo = hi = None
        it = td.type(index)
        if it and it[0] in (0x04, 0x05, 0x06, 0x08, 0x09, 0x0A, 0x0C):
            lo, hi = struct.unpack('<ii', td.type_slot(index + 1))
        count = size // max(td.type_size(w), 1)
        et, er = rust_type(td, w, out_structs, hint)
        if et == 'u8':
            return f'[u8; {count}]', 'r.array()?'
        return f'[{et}; {count}]', f'r.arr(|r| Ok({er}))?'
    if tid == 0x1E:
        sname = rust_struct_name(td.name(name)) if name else hint
        if sname not in out_structs:
            out_structs[sname] = None   # reserve, then fill (recursion)
            out_structs[sname] = rust_record(td, t, sname, out_structs)
        return sname, f'{sname}::read(r)?'
    return f'[u8; {size}]', 'r.array()?'   # unknown kinds stay raw bytes


def rust_record(td, t, sname, out_structs):
    fields, reads = [], []
    for off, fname, ftype, fsize in td.record_fields(t):
        rt, rexpr = rust_type(td, _member_type(td, t, fname), out_structs, sname + fname[:1].upper() + fname[1:])
        ident = rust_ident(fname)
        fields.append(f'    /// +{off:#06x} {fname}: {ftype}\n    pub {ident}: {rt},')
        reads.append(f'            {ident}: {rexpr},')
    size = td.type_size(t)
    return (f'#[derive(Debug, Clone, Copy, PartialEq)]\npub struct {sname} {{\n' + '\n'.join(fields) + '\n}\n\n'
            f'impl {sname} {{\n    /// Bytes in the original\'s memory and files.\n    pub const SIZE: usize = {size};\n\n'
            f'    pub fn read(r: &mut PascalReader) -> Result<Self, ShortRead> {{\n'
            f'        let start = r.pos();\n        let v = {sname} {{\n' + '\n'.join(reads) + '\n        };\n'
            f'        debug_assert_eq!(r.pos() - start, Self::SIZE);\n        Ok(v)\n    }}\n}}\n')


def _member_type(td, t, fname):
    first = td.type(t)[4]
    i = first - 1
    while 0 <= i < len(td.members):
        flags, name, mt = td.members[i]
        if td.name(name) == fname and not flags & 0x40:
            return mt
        if flags & 0x80:
            break
        i += 1
    raise KeyError(fname)


def rust_for(td, type_names):
    named = {n['name'].lower(): n for n in td.named_types()}
    out_structs, tables = {}, []
    for tn in type_names:
        nt = named.get(tn.lower())
        if nt is None:
            raise SystemExit(f'{tn}: no such named type')
        t = nt['type']
        ty = td.type(t)
        if ty[0] == 0x1C:     # array of records: a table type plus its element struct
            elem = element_record(td, t)
            ename = rust_struct_name(nt['name'])
            rust_type(td, elem, out_structs, ename) if td.type(elem)[1] else out_structs.setdefault(ename, rust_record(td, elem, ename, out_structs))
            index, = struct.unpack_from('<H', td.type_slot(t + 1))
            lo, hi = struct.unpack('<ii', td.type_slot(index + 1))
            ename = rust_struct_name(td.name(td.type(elem)[1])) if td.type(elem)[1] else ename
            tables.append(f'/// Pascal `{nt["name"]} = array[{lo}..{hi}] of {ename}` ({nt["size"]} bytes).\n'
                          f'pub const {nt["name"].upper()}_LOW: i32 = {lo};\npub const {nt["name"].upper()}_COUNT: usize = {hi - lo + 1};\n')
        elif ty[0] == 0x1E:
            rust_type(td, t, out_structs, rust_struct_name(nt['name']))
        else:
            raise SystemExit(f'{tn}: not a record or an array of records')
    head = ('// Generated by tools/dos/tdinfo.py rust (reverse-engineer-games) from the original program\'s\n'
            '// debug symbols. Field order, types and offsets are the original\'s; do not edit by hand.\n'
            '#![allow(clippy::all)]\n#[allow(unused_imports)]\nuse crate::pascal::{PascalReader, PStr, Real48, ShortRead};\n\n')
    return head + '\n'.join(tables) + '\n' + '\n'.join(v for v in out_structs.values() if v)


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        sys.exit(2)
    path, cmd, args = argv[0], argv[1], argv[2:]
    if cmd == 'rust-reader':
        print(RUST_READER, end='')
        return
    td = TDInfo(path)
    needle = (args[0] if args else '').lower()
    if cmd == 'summary':
        print(f'{path}: Turbo Debugger symbols version {td.version >> 8}.{td.version & 0xFF} at file+{td.at:#x}')
        print('  ' + ', '.join(f'{v} {k}' for k, v in td.counts.items()))
        print(f'  {len(td.procedures())} procedures, {len(td.globals())} globals, {len(td.named_types())} named types')
        print('  modules (unit, source file, code ranges):')
        for i, m in enumerate(td.modules, 1):
            files = [td.name(td.files[f - 1][0]) for f in range(m[5], m[5] + m[6]) if 0 < f <= len(td.files)]
            ranges = [f'{td.address(s, o)}+{n:#x}' for mod, s, o, n, *_ in td.segments if mod == i and n]
            print(f'    {td.name(m[0]):<12} {", ".join(files):<16} {" ".join(ranges)}')
    elif cmd == 'labels':
        # A name used in two units (unit-private procedures) gets its unit: drawlines_fonthand.
        rows = [(p['address'], p['name'], p['module'], f'{p["size"]} bytes') for p in td.procedures()]
        rows += [(g['address'], g['name'], g['module'], g['type']) for g in td.globals()]
        seen = {}
        for _, name, *_ in rows:
            seen[name.lower()] = seen.get(name.lower(), 0) + 1
        print('# address\tlabel\tevidence (from tdinfo.py: the original debug symbols)')
        printed = set()
        for addr, name, module, what in rows:
            label = f'{name}_{module}' if seen[name.lower()] > 1 and module else name
            if label in printed:
                label = f'{label}_{addr.replace(":", "_")}'
            printed.add(label)
            print(f'{addr}\t{label}\tobserved: TD symbol {name}, unit {module or "?"}, {what}')
    elif cmd == 'procs':
        for p in td.procedures():
            if needle in p['name'].lower() or needle in p['module'].lower():
                print(f'{p["address"]}  {p["size"]:6d} bytes  {p["module"]:<10} {p["name"]}'
                      f'{"" if p["far"] else "  (near)"}')
                for v in p.get('vars', []):
                    print(f'        {v["kind"]:<9} [bp{v["bp"]:+d}] {v["name"]}: {v["type"]}')
    elif cmd == 'globals':
        for g in td.globals():
            if needle in g['name'].lower() or needle in g['module'].lower():
                print(f'{g["address"]}  {g["size"]:6d}  {g["module"]:<10} {g["name"]}: {g["type"]}')
    elif cmd == 'types':
        for t in td.named_types():
            if needle in t['name'].lower():
                print(f'type {t["name"]} = {t["definition"]}  ({t["size"]} bytes)')
                show_record(td, t['type'], '    ')
    elif cmd == 'lines':
        for addr, fname, line in td.line_map():
            if needle in fname.lower():
                print(f'{addr}\t{fname}:{line}')
    elif cmd == 'rust':
        print(rust_for(td, args), end='')
    elif cmd == 'rust-reader':
        print(RUST_READER, end='')
    elif cmd == 'json':
        out = dict(version=td.version, counts=td.counts, procedures=td.procedures(), globals=td.globals(),
                   types=td.named_types(), lines=td.line_map())
        with open(args[0], 'w') as f:
            json.dump(out, f, indent=1)
        print(f'wrote {args[0]}')
    else:
        print(__doc__)
        sys.exit(2)


if __name__ == '__main__':
    main(sys.argv[1:])
