#!/usr/bin/env python3
"""Find the code that Ghidra's analysis did not turn into functions, and list where functions
probably start in it, ready to pass to `ghidra-run.sh BINARY decompile`.

Usage: gaps.py BINARY FUNCTIONS.tsv [--min N] [--list] [--addresses]

  BINARY         a Windows PE file (32-bit or 64-bit)
  FUNCTIONS.tsv  the export's analysis/export/BINARY/functions.tsv (address, size, name)
  --min N        the smallest gap reported (default 64 bytes)
  --list         list every gap with its size and the candidates inside it
  --addresses    print only the candidate addresses, one per line, in order (for a script)

Auto-analysis follows direct calls from the code it already knows. Code reached only through
pointers, and code Ghidra could not decode at first, is left out: in two late-1990s Watcom
games a quarter to a third of the code section lay in no function, WinMain and the main loop
among it (docs/knowhow/decompile-missed-pointer-functions.md). This tool measures that share,
then finds likely function starts inside the gaps:

  call     the target of an E8 call from anywhere in the code
  prolog   a start after a ret (and any padding) that opens like a function: push ebp; mov
           ebp, esp (55 89 E5 or 55 8B EC), sub esp, or two or more register pushes in a row
           (53 51 52 56 57 55), the way Watcom code saves the registers it uses

Each candidate is a lead. Pass them to `ghidra-run.sh BINARY decompile OUT ADDR ...` in small
batches (one bad address stops a batch), then check that each made function ends where the
next begins. A start found only as "prolog" can be data that looks like code. Coverage counts
each function's size as Ghidra gives it. Standard library only.
"""
import struct
import sys

PUSHES = set(range(0x50, 0x58))          # push eax .. push edi


def sections(data):
    """(virtual address, raw offset, raw size, is code) for each section, and the image base."""
    pe = struct.unpack_from('<I', data, 0x3C)[0]
    if data[:2] != b'MZ' or data[pe:pe + 4] != b'PE\0\0':
        sys.exit('not a PE file')
    count, opt_size = struct.unpack_from('<H', data, pe + 6)[0], struct.unpack_from('<H', data, pe + 20)[0]
    opt = pe + 24
    magic = struct.unpack_from('<H', data, opt)[0]
    base = struct.unpack_from('<Q', data, opt + 24)[0] if magic == 0x20B else struct.unpack_from('<I', data, opt + 28)[0]
    out = []
    for i in range(count):
        s = opt + opt_size + 40 * i
        vsize, va, rsize, raw = struct.unpack_from('<IIII', data, s + 8)
        flags = struct.unpack_from('<I', data, s + 36)[0]
        size = min(vsize or rsize, rsize)             # VirtualSize 0: use the raw size
        out.append((base + va, raw, size, bool(flags & 0x20000020)))
    return base, out


def load_functions(path):
    funcs = []
    for line in open(path):
        parts = line.rstrip('\n').split('\t')
        try:
            funcs.append((int(parts[0].split(':')[-1], 16), max(1, int(parts[1]))))
        except (ValueError, IndexError):
            continue
    return sorted(funcs)


def gaps_in(start, end, funcs):
    at, out = start, []
    for f, size in funcs:
        if f + size <= at or f >= end:
            continue
        if f > at:
            out.append((at, f))
        at = max(at, f + size)
    if at < end:
        out.append((at, end))
    return out


def looks_like_prologue(code, i):
    b = code[i:i + 4]
    if len(b) < 3:
        return False
    if b[0] == 0x55 and b[1:3] in (b'\x89\xe5', b'\x8b\xec'):
        return True
    if b[0] in (0x83, 0x81) and b[1] == 0xEC:
        return True
    return b[0] in PUSHES and b[1] in PUSHES and b[0] != b[1]


def candidates(code, va0, gap, calls):
    lo, hi = gap
    found = {t: 'call' for t in calls if lo <= t < hi}
    i = lo - va0
    while i < hi - va0:
        if code[i] == 0xC3 or (code[i] == 0xC2 and i + 2 < len(code)):
            j = i + (1 if code[i] == 0xC3 else 3)
            while j < hi - va0 and code[j] in (0x00, 0x90, 0xCC):
                j += 1
            if j < hi - va0 and looks_like_prologue(code, j):
                found.setdefault(va0 + j, 'prolog')
            i = j
        else:
            i += 1
    if lo - va0 >= 0 and looks_like_prologue(code, lo - va0):
        found.setdefault(lo, 'prolog')
    return sorted(found.items())


def main(argv):
    if len(argv) < 2 or argv[0] in ('-h', '--help'):
        print(__doc__)
        return 0 if argv and argv[0] in ('-h', '--help') else 2
    opt = lambda name, default=None: argv[argv.index(name) + 1] if name in argv else default
    data = open(argv[0], 'rb').read()
    funcs = load_functions(argv[1])
    minimum = int(opt('--min', '64'), 0)
    base, secs = sections(data)
    total = covered = 0
    found = []
    for va, raw, size, is_code in secs:
        if not is_code or not size:
            continue
        code = data[raw:raw + size]
        calls = set()
        for i in range(len(code) - 4):
            if code[i] == 0xE8:
                t = va + i + 5 + struct.unpack_from('<i', code, i + 1)[0]
                if va <= t < va + size:
                    calls.add(t)
        gaps = gaps_in(va, va + size, funcs)
        total += size
        covered += size - sum(e - s for s, e in gaps)
        for g in gaps:
            if g[1] - g[0] >= minimum:
                found.append((g, candidates(code, va, g, calls)))
    if '--addresses' in argv:
        for _, cands in found:
            for a, _ in cands:
                print(f'0x{a:x}')
        return 0
    big = sum(e - s for (s, e), _ in found)
    n = sum(len(c) for _, c in found)
    print(f'code {total:,} bytes; in functions {covered:,} ({100 * covered / max(1, total):.1f}%); '
          f'{len(found)} gaps of {minimum} bytes or more hold {big:,} bytes and {n} likely function starts')
    rows = sorted(found, key=lambda r: -(r[0][1] - r[0][0]))
    for (s, e), cands in (rows if '--list' in argv else rows[:10]):
        kinds = ', '.join(f'0x{a:x} {k}' for a, k in cands[:6]) + (' ...' if len(cands) > 6 else '')
        print(f'  0x{s:x}-0x{e:x}  {e - s:7,} bytes  {kinds}')
    if found and '--list' not in argv and len(rows) > 10:
        print(f'  ... {len(rows) - 10} more (--list)')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
