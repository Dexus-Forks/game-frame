#!/usr/bin/env python3
"""How much of a program's code a Ghidra export covers, where the gaps are, and where the
functions it missed probably start.

Usage: exportcov.py BINARY functions.tsv [--bodies FILE] [--gaps N] [--starts FILE] [--min PERCENT]

  BINARY         the Windows PE that was exported (32-bit or 64-bit)
  functions.tsv  the export's function list (address, size, name), from scripts/ghidra-export.sh
  --bodies FILE  each piece of each function (start, end, entry; bodies.tsv from the same
                 export, read by default when it sits beside functions.tsv). Without it, each
                 function counts as one run of its size from its entry, which undercounts a
                 function whose body is in several pieces
  --gaps N       list the N largest gaps (default 10)
  --starts FILE  write the likely function starts in the gaps, one address a line, ready for
                 `scripts/ghidra-run.sh BINARY decompile OUT.txt $(cat FILE)`
  --min PERCENT  exit 1 when the coverage is below PERCENT (default 90)

It prints the bytes of the executable sections that lie in some function, as a share of all
code bytes; the largest runs of code in no function; and, for those gaps, the likely starts of
functions Ghidra missed:

- call targets: every `E8 rel32` call in the code whose target lies in a gap (a direct call
  from anywhere, even from other missed code);
- gap starts: the first instruction of a gap after its padding (`nop` of any length, `int3`),
  when it is on a 16-byte boundary, as compilers align functions, or begins a common prologue
  (push ebp / mov ebp, esp; push of a saved register; sub esp; on x64, sub rsp or a mov to the
  shadow space);
- starts after a return: in a gap, the first instruction after a `ret` (C3, or C2 nn nn) and
  its padding, by the same rule.

Why: Ghidra's analysis follows direct calls from code it knows. Functions reached only through
pointers (per-object step functions, handler tables, callbacks) are missed, and so is code
between them. In one 1997 game a quarter of the code section was outside every function after
the first export (docs/knowhow/decompile-missed-pointer-functions.md). Run this after every
export.

The starts are candidates, not proof: a byte pattern in data can look like a prologue. The
batch decompile skips an address that nothing refers to unless it follows a ret or padding.
Single-byte padding (`int3`, `nop` and zeros) at the ends of a gap is not counted as missed
code. Needs no Ghidra; reads the PE with pefile and decodes with capstone (.venv).
"""
import bisect
import os
import struct
import sys

import capstone

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sigscan import Image  # noqa: E402

PROLOGUES = (b'\x55\x8b\xec', b'\x55\x89\xe5', b'\x53\x56', b'\x56\x57', b'\x83\xec', b'\x81\xec',
             b'\x48\x83\xec', b'\x48\x81\xec', b'\x48\x89\x5c\x24', b'\x40\x53', b'\x48\x89\x4c\x24')
PAD = b'\xcc\x90\x00'


def load_functions(path):
    rows = []
    for line in open(path, encoding='utf-8', errors='replace'):
        parts = line.rstrip('\n').split('\t')
        if len(parts) >= 2 and parts[1].isdigit():
            rows.append((int(parts[0], 16), int(parts[1])))
    return sorted(rows)


def load_bodies(path):
    """(start, size) of every piece in bodies.tsv: start, end (exclusive), entry, in hex."""
    rows = []
    for line in open(path, encoding='utf-8', errors='replace'):
        parts = line.rstrip('\n').split('\t')
        try:
            s, e = int(parts[0], 16), int(parts[1], 16)
        except (IndexError, ValueError):
            continue
        if e > s:
            rows.append((s, e - s))
    return sorted(rows)


def covered_ranges(funcs, lo, hi):
    """Merged [start, end) ranges of functions, clipped to [lo, hi)."""
    out = []
    for start, size in funcs:
        s, e = max(start, lo), min(start + size, hi)
        if s >= e:
            continue
        if out and s <= out[-1][1]:
            out[-1][1] = max(out[-1][1], e)
        else:
            out.append([s, e])
    return out


def gaps_of(data, base, ranges):
    """[(start, end)] of code in no function, with padding trimmed from both ends."""
    gaps, at = [], base
    for s, e in ranges + [[base + len(data), base + len(data)]]:
        if s > at:
            a, b = at - base, s - base
            while a < b and data[a] in PAD:
                a += 1
            while b > a and data[b - 1] in PAD:
                b -= 1
            if b > a:
                gaps.append((base + a, base + b))
        at = max(at, e)
    return gaps


def in_gap(gaps, starts, addr):
    i = bisect.bisect_right(starts, addr) - 1
    return i >= 0 and addr < gaps[i][1]


def skip_padding(md, data, base, at, end):
    """The address of the first instruction from `at` that is not padding (nop, int3)."""
    while at < end:
        ins = next(md.disasm(data[at - base:at - base + 16], at, 1), None)
        if ins is None or ins.mnemonic not in ('nop', 'int3') and ins.bytes != b'\x00\x00':
            return at
        at += ins.size
    return end


def candidates(regions, gaps):
    starts = [g[0] for g in gaps]
    found = {}
    for data, base, bits in regions:
        i = data.find(b'\xe8')
        while 0 <= i < len(data) - 4:
            target = base + i + 5 + struct.unpack_from('<i', data, i + 1)[0]
            if in_gap(gaps, starts, target):
                found.setdefault(target, 'call target')
            i = data.find(b'\xe8', i + 1)
        md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_64 if bits == 64 else capstone.CS_MODE_32)
        for g0, g1 in gaps:
            if not (base <= g0 < base + len(data)):
                continue
            first = skip_padding(md, data, base, g0, g1)
            if first < g1 and (first % 16 == 0 or data[first - base:first - base + 4].startswith(PROLOGUES)):
                found.setdefault(first, 'gap start')
            a = g0 - base
            for j in range(a, g1 - base):
                ret = data[j] == 0xc3 or (data[j] == 0xc2 and j + 2 < len(data))
                if not ret:
                    continue
                k = skip_padding(md, data, base, base + j + (1 if data[j] == 0xc3 else 3), g1) - base
                if k < g1 - base and (k + base > base + j + 1 and (k + base) % 16 == 0
                                      or data[k:k + 4].startswith(PROLOGUES)):
                    found.setdefault(base + k, 'after a return')
    return found


def main(argv):
    if len(argv) < 2 or argv[0] in ('-h', '--help'):
        print(__doc__.rstrip())
        return 0 if argv and argv[0] in ('-h', '--help') else 2
    opt = lambda k, d=None: argv[argv.index(k) + 1] if k in argv else d
    img = Image(argv[0], 0)
    funcs = load_functions(argv[1])
    bodies = opt('--bodies', os.path.join(os.path.dirname(os.path.abspath(argv[1])), 'bodies.tsv'))
    pieces = load_bodies(bodies) if os.path.exists(bodies) else funcs
    if pieces is not funcs:
        print(f'function bodies from {bodies} ({len(pieces)} pieces)')
    total = done = 0
    gaps = []
    for data, base, _ in img.regions:
        ranges = covered_ranges(pieces, base, base + len(data))
        total += len(data)
        done += sum(e - s for s, e in ranges)
        gaps += gaps_of(data, base, ranges)
    missed = sum(e - s for s, e in gaps)
    pct = 100.0 * done / total if total else 100.0
    print(f'{len(funcs)} functions cover {done} of {total} code bytes ({pct:.1f}%); '
          f'{missed} bytes of code lie in {len(gaps)} gaps (padding left out)')
    for s, e in sorted(gaps, key=lambda g: g[0] - g[1])[:int(opt('--gaps', '10'))]:
        print(f'  gap {s:#x}-{e:#x}  {e - s} bytes')
    found = candidates(img.regions, gaps)
    print(f'{len(found)} likely function starts in the gaps')
    for addr in sorted(found)[:40]:
        print(f'  {addr:#x}  {found[addr]}')
    if opt('--starts'):
        with open(opt('--starts'), 'w', encoding='utf-8') as f:
            f.write(''.join(f'{a:#x}\n' for a in sorted(found)))
        print(f'wrote {len(found)} addresses to {opt("--starts")}')
    return 1 if pct < float(opt('--min', '90')) else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
