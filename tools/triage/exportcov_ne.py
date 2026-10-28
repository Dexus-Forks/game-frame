"""exportcov.py for 16-bit NE programs (Windows 3.x, OS/2 1.x): how much of the code segments a
Ghidra export covers, the gaps, and the likely starts of the functions it missed. exportcov.py
calls this when BINARY is an NE; see its help for the options.

Addresses are seg:off as Ghidra's NE loader numbers the segments (segment N is selector
0x1000 + 8 * (N - 1)), in functions.tsv, bodies.tsv and the starts file alike. Two sources of
starts:

- far targets: every internal relocation into a code segment. A linker patches only the
  selector of a far call or far pointer and chains the sites (each site holds the next one,
  0xffff ends the chain), so the target offset is the word before each site. A far call
  (9A off sel) always counts; a far pointer counts when its target opens like code (ENTER,
  push bp, or follows a return), since code segments also hold pointers to data;
- prologues in gaps: ENTER nn nn 00 or push bp / mov bp, sp (55 89 E5, as Borland's compilers
  write it, or 55 8B EC) in bytes no function covers, when
  they follow a return (C3, CB, C2 nn nn, CA nn nn), a covered byte, the segment start, or two
  printable bytes (Borland puts a procedure's string constants just before its code).

Segments stored as iterated records (flag 0x0008) are read expanded, as the loader does, so the
relocation chains through a packed data segment (far pointers to methods) count too.

The rest of a gap is usually data kept in code segments: jump tables, and in Delphi programs
the class tables (VMTs, RTTI, class names). Standard library only, with tools/dos/ne.py.
"""
import os
import re
import struct
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'dos'))
from ne import load  # noqa: E402

PAD = b'\x00\x90\xcc'
RETURNS, RETURNS_N = (0xc3, 0xcb), (0xc2, 0xca)
ENTER_OR_BP = re.compile(rb'\xc8..\x00|\x55\x8b\xec|\x55\x89\xe5', re.S)
CHAIN_LIMIT = 0x10000   # a chain longer than a segment's words is broken


def is_ne(path):
    """True when the file is a 16-bit NE executable."""
    try:
        return load(path).kind == 'NE'
    except (OSError, ValueError, struct.error):
        return False


def seg_off(text):
    s, o = text.split(':')
    return int(s, 16), int(o, 16)


def pieces(functions, bodies):
    """((selector, start), (selector, end)) of every piece: bodies.tsv, else functions.tsv's
    entry and size (one run, which undercounts a function in several pieces)."""
    out = []
    source = bodies or functions
    for line in open(source, encoding='utf-8', errors='replace'):
        parts = line.rstrip('\n').split('\t')
        try:
            start = seg_off(parts[0])
            end = seg_off(parts[1]) if bodies else (start[0], start[1] + int(parts[1]))
        except (IndexError, ValueError):
            continue
        out.append((start, end))
    return out


def image(exe, s):
    """A segment's bytes as they are in memory (an iterated segment's records expanded)."""
    return exe.image(s.selector) or b''


def coverage(exe, rows):
    """{selector: bytearray}, 1 for each byte of a code segment some function covers."""
    marks = {s.selector: bytearray(len(image(exe, s))) for s in exe.segments if not s.is_data and s.offset}
    for (sel, lo), (end_sel, hi) in rows:
        m = marks.get(sel)
        if m is None:
            continue
        hi = min(hi if end_sel == sel else len(m), len(m))
        if hi > lo:
            m[lo:hi] = b'\x01' * (hi - lo)
    return marks


def gaps_of(exe, marks):
    """(selector, start, end) of the runs of code in no function, padding trimmed off both ends."""
    gaps = []
    for s in exe.segments:
        m = marks.get(s.selector)
        if m is None:
            continue
        code, i = image(exe, s), 0
        while i < len(m):
            if m[i]:
                i += 1
                continue
            j = i
            while j < len(m) and not m[j]:
                j += 1
            a, b = i, j
            while a < b and code[a] in PAD:
                a += 1
            while b > a and code[b - 1] in PAD:
                b -= 1
            if b > a:
                gaps.append((s.selector, a, b))
            i = j
    return gaps


def opens_like_code(code, off):
    return code[off] in (0xc8, 0x55) or (off > 0 and code[off - 1] in RETURNS)


def relocations(exe, seg):
    """(source type, flags, site, target segment, target offset) of each record of a segment."""
    d, at = exe.data, seg.offset + seg.length + 2
    for i in range(seg.relocs):
        if at + 8 * i + 8 > len(d):
            return
        src, flags, site, a, _, c = struct.unpack_from('<BBHBBH', d, at + 8 * i)
        if flags & 3:                                   # imports and OS fixups
            continue
        yield (src, flags, site) + (exe.entries.get(c, (0, 0)) if a == 0xff else (a, c))


def far_targets(exe):
    """{(selector, offset): kind} of every far call or far pointer into a code segment."""
    found, segs = {}, exe.segments
    for seg in segs:
        code = image(exe, seg)
        for src, flags, site, tseg, toff in relocations(exe, seg):
            if not 0 < tseg <= len(segs) or segs[tseg - 1].is_data or not segs[tseg - 1].offset:
                continue
            target = segs[tseg - 1]
            tcode = image(exe, target)
            if src == 3:                                # a far pointer with its offset in the record
                if toff < len(tcode) and opens_like_code(tcode, toff):
                    found.setdefault((target.selector, toff), 'far pointer target')
                continue
            if src != 2 or flags & 4:                   # only chained selectors carry an offset
                continue
            for _ in range(CHAIN_LIMIT):
                if site == 0xffff or not 2 <= site <= len(code) - 2:
                    break
                off = struct.unpack_from('<H', code, site - 2)[0]
                call = site >= 3 and code[site - 3] == 0x9a
                if off < len(tcode) and (call or opens_like_code(tcode, off)):
                    found.setdefault((target.selector, off), 'far call target' if call else 'far pointer target')
                site = struct.unpack_from('<H', code, site)[0]
    return found


def prologue_starts(exe, marks):
    """{(selector, offset): kind} of the prologues in no function that may open a routine."""
    found = {}
    for s in exe.segments:
        m = marks.get(s.selector)
        if m is None:
            continue
        code = image(exe, s)
        for hit in ENTER_OR_BP.finditer(code):
            k = hit.start()
            if m[k]:
                continue
            before = code[max(0, k - 3):k]
            text = len(before) >= 2 and all(32 <= c < 127 for c in before[-2:])
            if (k == 0 or m[k - 1] or before[-1] in RETURNS or text
                    or (len(before) == 3 and before[0] in RETURNS_N)):
                found[(s.selector, k)] = 'prologue'
    return found


def main(argv, opt):
    exe = load(argv[0])
    bodies = opt('--bodies', os.path.join(os.path.dirname(os.path.abspath(argv[1])), 'bodies.tsv'))
    bodies = bodies if os.path.exists(bodies) else None
    if bodies:
        print(f'function bodies from {bodies}')
    funcs = sum(1 for _ in pieces(argv[1], None))
    marks = coverage(exe, pieces(argv[1], bodies))
    total, done = sum(len(m) for m in marks.values()), sum(sum(m) for m in marks.values())
    gaps = gaps_of(exe, marks)
    pct = 100.0 * done / total if total else 100.0
    print(f'{funcs} functions cover {done} of {total} code bytes ({pct:.1f}%); '
          f'{sum(b - a for _, a, b in gaps)} bytes of code lie in {len(gaps)} gaps (padding left out)')
    for sel, a, b in sorted(gaps, key=lambda g: g[1] - g[2])[:int(opt('--gaps', '10'))]:
        print(f'  gap {sel:04x}:{a:04x}-{sel:04x}:{b:04x}  {b - a} bytes')
    found = {t: k for t, k in far_targets(exe).items() if t[0] in marks and not marks[t[0]][t[1]]}
    for t, k in prologue_starts(exe, marks).items():
        found.setdefault(t, k)
    print(f'{len(found)} likely function starts in the gaps')
    for (sel, off) in sorted(found)[:40]:
        print(f'  {sel:04x}:{off:04x}  {found[(sel, off)]}')
    if opt('--starts'):
        with open(opt('--starts'), 'w', encoding='utf-8') as f:
            f.write(''.join(f'{sel:04x}:{off:04x}\n' for sel, off in sorted(found)))
        print(f'wrote {len(found)} addresses to {opt("--starts")}')
    return 1 if pct < float(opt('--min', '90')) else 0
