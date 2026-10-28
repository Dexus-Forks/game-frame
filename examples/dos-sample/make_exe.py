#!/usr/bin/env python3
"""Build the DOS sample's executables from sample.s: a 16-bit NE file and a real-mode MZ file.

Usage: make_exe.py OUT_DIR

Assembles sample.s with `zig cc` (LLVM's assembler, .code16) and then links the result itself,
so GameFrame needs no 16-bit compiler or linker:

  OUT_DIR/sample.exe   NE (16-bit protected mode), like a Borland Pascal 7 DPMI16 or Windows 3.x
                       program: segment 1 code, segment 2 data (the auto data segment), named
                       exports, a selector relocation and an import from KERNEL (ordinal 3)
  OUT_DIR/sample-mz.exe  plain real-mode MZ: the same code and data in one load image, with an
                       MZ relocation for the data segment's paragraph
  OUT_DIR/sample-it.exe  sample.exe with its data segment stored as iterated records (NE flag
                       0x0008: runs of zeros packed, as Borland's linkers do); in memory it is
                       the same program
  OUT_DIR/symbols.tsv  name, segment, offset of every routine and of main_loop's labels (the
                       "map file")
  OUT_DIR/sample-td.exe  sample.exe with Turbo Debugger symbols (TDINFO 3.0) appended inside a
                       CodeView-style NB02 wrapper, laid out as Borland Pascal 7 writes them:
                       procedures, globals, a parameter, a record type, line numbers

The TDINFO writer follows the layout tools/dos/tdinfo.py reads (learnt from a real Borland
Pascal 7 program); it proves the reader keeps working, not that the layout is Borland's own
specification. It writes no other format (no LE/LX, no overlays) and does not run anything.
"""
import os
import re
import struct
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
EXPORTS = ['rng_seed', 'rng_next', 'sum_draws', 'checksum', 'get_data_seg', 'ask_host', 'dos_version', 'field_read',
           'read_file', 'wait_ticks', 'get_key', 'write_file', 'wait_keys']
# Labels inside main_loop, written to symbols.tsv for the fragments.py test
LABELS = ['main_loop', 'frame_top', 'draw', 'draw_even_end', 'draw_odd_end', 'event', 'event_end']
# Routines nothing calls, written to symbols.tsv for the tests of ghidra-run.sh's decompile by
# address and of exportcov on an NE
UNREACHED = ['after_iret', 'after_retf', 'after_retf_n', 'far_caller', 'far_only']
DESCRIPTION = b'DOS sample for GameFrame'
MESSAGE = b'\x0cDOS SAMPLE 1'   # a Pascal string: length byte first
# Two Pascal strings back to back; the second's length byte (0x2a) is a printable '*'
ADJACENT = b'\x05FIRST' + bytes([0x2a]) + b'SECOND STRING, ITS LENGTH BYTE IS A STAR!!'


def assemble():
    with tempfile.TemporaryDirectory() as tmp:
        obj = os.path.join(tmp, 'sample.o')
        subprocess.run(['zig', 'cc', '-target', 'x86-freestanding', '-c', os.path.join(HERE, 'sample.s'),
                        '-o', obj], check=True)
        return read_elf(open(obj, 'rb').read())


def read_elf(d):
    """The .text bytes and {symbol: offset} from an ELF32 object (no relocations allowed)."""
    shoff, = struct.unpack_from('<I', d, 0x20)
    shentsize, shnum, shstrndx = struct.unpack_from('<HHH', d, 0x2e)
    secs = [struct.unpack_from('<IIIIIIIIII', d, shoff + i * shentsize) for i in range(shnum)]
    names = secs[shstrndx]
    def secname(s):
        o = names[4] + s[0]
        return d[o:d.index(b'\0', o)].decode()
    by = {secname(s): s for s in secs}
    if any(n.startswith('.rel') and 'text' in n for n in by):
        sys.exit('sample.s produced relocations: keep every label local (no .globl)')
    text = by['.text']; code = d[text[4]:text[4] + text[5]]
    symtab = by['.symtab']; strtab = secs[symtab[6]]
    syms = {}
    for i in range(symtab[5] // 16):
        nm, value, size, info, other, shndx = struct.unpack_from('<IIIBBH', d, symtab[4] + 16 * i)
        name = d[strtab[4] + nm:d.index(b'\0', strtab[4] + nm)].decode()
        if name and shndx < len(secs) and secs[shndx] is text:
            syms[name] = value
    return code, syms


def data_segment():
    """The data segment: the variables sample.s uses, then Pascal strings and a C string."""
    d = bytearray(0x10)
    d[0:4] = struct.pack('<I', 0x1234)       # RandSeed's initial value
    d += MESSAGE + b'\0' + DESCRIPTION + b'\0' + ADJACENT + b'\0'
    d += bytes(0x80 - len(d))
    d += b'sample.txt\0'                     # FileName at 0x80 (read_file)
    d += bytes(0x180 - len(d))               # FileBuf at 0x90; main_loop's words at 0xA0;
                                             # KeyLog at 0xB0, KeyDown at 0x100 (wait_keys)
    return bytes(d)


def pstr(s):
    return bytes([len(s)]) + s


def iterated_records(data):
    """The data segment as NE iterated records (u16 count, u16 length, bytes), as Borland's
    linkers pack it: each run of eight or more zeros is one record of a zero byte repeated,
    the bytes between runs are records of count 1."""
    out, i = b'', 0
    for m in re.finditer(rb'\0{8,}', data):
        if m.start() > i:
            out += struct.pack('<HH', 1, m.start() - i) + data[i:m.start()]
        out += struct.pack('<HH', m.end() - m.start(), 1) + b'\0'
        i = m.end()
    if i < len(data):
        out += struct.pack('<HH', 1, len(data) - i) + data[i:]
    return out


def build_ne(code, syms, data, iterated=False):
    code = bytearray(code)
    align = 4  # segment data aligned to 16 bytes
    # Relocations of segment 1 (8 bytes each). The word at each location is 0xffff (end of chain).
    relocs = [
        struct.pack('<BBHBBH', 2, 0, syms['fix_tick_ds'], 2, 0, 0),         # selector of segment 2
        struct.pack('<BBHBBH', 2, 0, syms['fix_entry_ds'] + 1, 2, 0, 0),   # selector of segment 2
        struct.pack('<BBHBBH', 2, 0, syms['fix_dataseg'] + 1, 2, 0, 0),    # selector of segment 2
        struct.pack('<BBHHH', 3, 1, syms['fix_import'], 1, 3),            # far pointer: KERNEL.3
        struct.pack('<BBHBBH', 2, 0, syms['fix_far_only'], 1, 0, 0),       # selector of segment 1
    ]
    stub = mz_stub()
    ne_off = len(stub)
    # Tables after the 64-byte NE header, in the order Windows' linker writes them.
    segtab_off = 0x40
    nseg = 2
    restab = pstr(b'DOSSAMPL') + struct.pack('<H', 0)
    for i, name in enumerate(EXPORTS, 1):
        restab += pstr(name.upper().encode()) + struct.pack('<H', i)
    restab += b'\0'
    rsrc_off = segtab_off + 8 * nseg          # no resources: resource table = resident names
    restab_off = rsrc_off
    modref_off = restab_off + len(restab)
    impnames = b'\0' + pstr(b'KERNEL')
    modref = struct.pack('<H', 1)
    imp_off = modref_off + len(modref)
    entry = bytes([len(EXPORTS), 1]) + b''.join(struct.pack('<BH', 1, syms[n]) for n in EXPORTS) + b'\0'
    entry_off = imp_off + len(impnames)
    nonres = pstr(DESCRIPTION) + struct.pack('<H', 0) + b'\0'
    tables_end = entry_off + len(entry)
    nonres_file = ne_off + tables_end
    seg1_file = (nonres_file + len(nonres) + 15) & ~15
    reloc_blob = struct.pack('<H', len(relocs)) + b''.join(relocs)
    seg2_file = (seg1_file + len(code) + len(reloc_blob) + 15) & ~15
    for off in (syms['fix_entry_ds'] + 1, syms['fix_dataseg'] + 1, syms['fix_import'], syms['fix_tick_ds']):
        code[off:off + 2] = b'\xff\xff'
    segtab = struct.pack('<HHHH', seg1_file >> align, len(code), 0x0100 | 0x0c00, len(code))
    if iterated:
        data = iterated_records(data)
    segtab += struct.pack('<HHHH', seg2_file >> align, len(data), 0x0001 | 0x0c00 | (0x0008 if iterated else 0), 0x4000)
    hdr = bytearray(0x40)
    struct.pack_into('<2sBBHH', hdr, 0, b'NE', 5, 10, entry_off, len(entry))
    struct.pack_into('<HH', hdr, 0x0c, 0x0302, 2)      # flags: one data segment per instance, PM app; autodata = 2
    struct.pack_into('<HH', hdr, 0x10, 0x0400, 0x2000)  # heap and stack sizes
    struct.pack_into('<HHHH', hdr, 0x14, syms['entry'], 1, 0, 2)   # CS:IP = 1:entry, SS:SP = 2:0
    struct.pack_into('<HHH', hdr, 0x1c, nseg, 1, len(nonres))
    struct.pack_into('<HHHHHI', hdr, 0x22, segtab_off, rsrc_off, restab_off, modref_off, imp_off, nonres_file)
    struct.pack_into('<HHH', hdr, 0x30, 0, align, 0)
    hdr[0x36] = 2                                       # target OS: Windows
    struct.pack_into('<H', hdr, 0x3e, 0x030a)          # expected Windows version 3.10
    out = bytearray(stub) + hdr + segtab + restab + modref + impnames + entry
    assert len(out) == ne_off + tables_end
    out += nonres
    out += b'\0' * (seg1_file - len(out)) + code + reloc_blob
    out += b'\0' * (seg2_file - len(out)) + data
    return bytes(out)


def mz_stub():
    """A 128-byte MZ header and stub that prints a message and exits, with e_lfanew = 0x80."""
    msg = b'This program needs a DOS extender or Windows.\r\n$'
    code = b'\x0e\x1f\xba\x0e\x00\xb4\x09\xcd\x21\xb8\x01\x4c\xcd\x21' + msg   # push cs; pop ds; print; exit
    h = bytearray(0x40)
    size = 0x80
    struct.pack_into('<2sHHHHHHHHHHHHH', h, 0, b'MZ', size % 512, (size + 511) // 512, 0, 4, 0, 0xffff,
                     0, 0xb8, 0, 0, 0, 0x40, 0)
    struct.pack_into('<I', h, 0x3c, 0x80)
    out = h + code
    assert len(out) <= 0x80
    return bytes(out + b'\0' * (0x80 - len(out)))


def build_mz(code, syms, data):
    """A real-mode MZ: code at paragraph 0 of the image, data after it, one relocation per
    reference to the data segment (DOS adds the load segment to each)."""
    code = bytearray(code)
    code_paras = (len(code) + 15) // 16
    relocs = []
    for off in (syms['fix_entry_ds'] + 1, syms['fix_dataseg'] + 1, syms['fix_tick_ds']):
        code[off:off + 2] = struct.pack('<H', code_paras)
        relocs.append(struct.pack('<HH', off, 0))
    code[syms['fix_import']:syms['fix_import'] + 4] = b'\0\0\0\0'   # no imports in real mode
    image = bytes(code) + b'\0' * (code_paras * 16 - len(code)) + data
    hdr_paras = (0x1c + 4 * len(relocs) + 15) // 16
    h = bytearray(hdr_paras * 16)
    total = len(h) + len(image)
    stack_seg = (len(image) + 15) // 16
    struct.pack_into('<2sHHHHHHHHHHHHH', h, 0, b'MZ', total % 512, (total + 511) // 512, len(relocs),
                     hdr_paras, 0x1000, 0xffff, stack_seg, 0x1000, 0, syms['entry'], 0, 0x1c, 0)
    h[0x1c:0x1c + 4 * len(relocs)] = b''.join(relocs)
    return bytes(h) + image


def tdinfo(syms, code_len):
    """A TDINFO 3.0 block for sample.exe (see tools/dos/tdinfo.py for the layout)."""
    names = []
    def name(text):
        names.append(text)
        return len(names)
    # Types, numbered from 1; subranges take two slots.
    types = [bytes(8)]                                                           # 1 void
    types.append(struct.pack('<BHHBH', 0x23, 0, 4, 0x41, 1))                     # 2 far procedure
    types.append(struct.pack('<BHHBH', 0x23, 0, 4, 0x41, 5))                     # 3 far function: word
    types.append(struct.pack('<BHHBH', 0x06, name('Longint'), 4, 0, 0))          # 4 longint
    types.append(struct.pack('<ii', -2 ** 31, 2 ** 31 - 1))
    types.append(struct.pack('<BHHBH', 0x09, name('Word'), 2, 0, 0))            # 6 word
    types.append(struct.pack('<ii', 0, 65535))
    rec_name = name('SampleState')
    types.append(struct.pack('<BHHBH', 0x1E, rec_name, 8, 0, 1))                 # 8 record, member 1
    members = [struct.pack('<BHH', 0, name('seed'), 4), struct.pack('<BHH', 0, name('total'), 6),
               struct.pack('<BHH', 0x80, name('calls'), 6)]
    # Anonymous records: 9 is `record x, y: Word end`, 10 is Player's own `record lives: Word;
    # pos: <9> end`, 11 is the named type Spot = record kind: Word; at: <9> end.
    types.append(struct.pack('<BHHBH', 0x1E, 0, 4, 0, 4))                       # 9, members 4-5
    types.append(struct.pack('<BHHBH', 0x1E, 0, 6, 0, 6))                       # 10, members 6-7
    spot_name = name('Spot')
    types.append(struct.pack('<BHHBH', 0x1E, spot_name, 6, 0, 8))               # 11, members 8-9
    members += [struct.pack('<BHH', 0, name('x'), 6), struct.pack('<BHH', 0x80, name('y'), 6),
                struct.pack('<BHH', 0, name('lives'), 6), struct.pack('<BHH', 0x80, name('pos'), 9),
                struct.pack('<BHH', 0, name('kind'), 6), struct.pack('<BHH', 0x80, name('at'), 9)]
    procs = [('rng_seed', 2), ('rng_next', 3), ('sum_draws', 3), ('checksum', 3),
             ('get_data_seg', 3), ('ask_host', 3), ('dos_version', 3), ('field_read', 3)]
    # An imported procedure: Borland gives it a segment number past the program's segments, and
    # here the same offset as rng_next, so a scope matched by offset alone would pick it.
    symbols, scopes = [struct.pack('<HHHHB', name('host_call'), 3, syms['rng_next'], 0xC00F, 0x18)], []
    for proc, t in procs:
        symbols.append(struct.pack('<HHHHB', name(proc), t, syms[proc], 1, 0x18))
        if proc == 'rng_next':      # its parameter, at [bp+6]
            symbols.append(struct.pack('<HHHHB', name('range'), 6, 6, 0, 0x0A))
            scopes.append(struct.pack('<HHHHHH', len(symbols), 1, 0, 0xFFFF, syms[proc], syms['sum_draws'] - syms[proc]))
    for var, off, t in (('RandSeed', 0, 4), ('Total', 4, 6), ('Calls', 6, 6), ('Player', 0x0A, 10)):
        symbols.append(struct.pack('<HHHHB', name(var), t, off, 2, 0))
    symbols.append(struct.pack('<HHHHB', rec_name, 8, 0, 0, 6))                  # type SampleState
    symbols.append(struct.pack('<HHHHB', spot_name, 11, 0, 0, 6))                # type Spot
    module = struct.pack('<HBBHHHHHH', name('sample'), 2, 4, 1, 0, 1, 1, 1, 1)
    files = struct.pack('<HI', name('SAMPLE.S'), 0)
    lines = [(10 + i, syms[p]) for i, (p, _) in enumerate(procs)]
    line_tab = b''.join(struct.pack('<HH', l, o) for l, o in lines)
    segment = struct.pack('<HHHHHHHH', 1, 1, 0, code_len, 1, len(scopes), 1, 1)
    corr = struct.pack('<HHHH', 1, 1, 1, len(lines))
    pool = b''.join(n.encode() + b'\0' for n in names)
    header = struct.pack('<HHIHHHHHHHHHHHHIIBHHBH', 0x52FB, 0x0300, len(pool), len(names), len(types),
                         len(members), len(symbols), 0, 1, 0, len(scopes), len(lines), 1, 1, 1,
                         0, 0, 0, 0, 0, 0, 16)
    assert len(header) == 0x30
    body = (header + bytes(16) + b''.join(symbols) + module + files + line_tab + b''.join(scopes)
            + segment + corr + b''.join(types) + b''.join(members) + bytes(5) + pool)
    return body


def with_tdinfo(exe, td):
    """Append TD symbols as Borland's linker does for a protected-mode program: an NB02 header
    (16 bytes, at a 16-byte boundary), the symbols, and an NB02 trailer giving their distance."""
    out = bytearray(exe)
    out += bytes(-len(out) % 16)
    base = len(out)
    out += b'NB02' + b'\xff' * 12 + td
    out += b'NB02' + struct.pack('<I', len(out) + 8 - base)
    return bytes(out)


def main(out_dir):
    os.makedirs(out_dir, exist_ok=True)
    code, syms = assemble()
    data = data_segment()
    ne = build_ne(code, syms, data)
    open(os.path.join(out_dir, 'sample.exe'), 'wb').write(ne)
    open(os.path.join(out_dir, 'sample-td.exe'), 'wb').write(with_tdinfo(ne, tdinfo(syms, len(code))))
    open(os.path.join(out_dir, 'sample-mz.exe'), 'wb').write(build_mz(code, syms, data))
    open(os.path.join(out_dir, 'sample-it.exe'), 'wb').write(build_ne(code, syms, data, iterated=True))
    with open(os.path.join(out_dir, 'symbols.tsv'), 'w') as f:
        for name in ['entry'] + EXPORTS + LABELS + UNREACHED:
            f.write(f'{name}\t1\t{syms[name]:#06x}\n')
    print(f'built {out_dir}/sample.exe (NE), sample-td.exe (NE + TD symbols), sample-mz.exe (MZ), sample-it.exe (NE, iterated data), symbols.tsv')


if __name__ == '__main__':
    if len(sys.argv) != 2:
        print(__doc__)
        sys.exit(2)
    main(sys.argv[1])
