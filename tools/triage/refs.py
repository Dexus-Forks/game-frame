#!/usr/bin/env python3
"""List every instruction that refers to an address. The search reads the raw code bytes, so it
finds hits in any function or in none. Each hit says whether the instruction reads the address,
writes it or takes it as a value.

Usage: refs.py BINARY ADDRESS [ADDRESS ...] [--near N] [--functions functions.tsv] [--json]

  BINARY       a Windows PE, 32-bit or 64-bit (its executable sections)
  ADDRESS      a virtual address, such as 0x61aad0 (a table, a global, a function)
  --near N     also find ADDRESS - N to ADDRESS + N, which loops and fields use (default 0;
               8 finds `[table + 4]` and `[table - 4]`)
  --functions  a Ghidra export's functions.tsv, to name the function each hit is in; a hit in
               no function is marked `(no function)`, which is code the export missed

Why: a search of the decompiled text finds only the code Ghidra made functions for. A linear
disassembly stops at the first data inside a code section. Neither can prove "only this
function writes that table". This tool searches the bytes themselves. In 32-bit code it finds
the address as a 32-bit immediate or displacement. In 64-bit code it also finds it as a
RIP-relative displacement. Each hit is decoded backwards, one to ten bytes, until an
instruction holds that operand, so the result does not depend on where a disassembly started.

Output: one line per instruction: its address, read, write or value (an immediate: the address
taken, pushed or compared), the instruction, and the function. Then the count.

It cannot find an address built at run time (a base plus a computed index) or held in data.
`tools/pe/relocs.py` lists the pointers in data. A hit can be data that happens to decode as an
instruction, so check each hit in Ghidra. Needs capstone (.venv).
"""
import bisect
import json
import os
import struct
import sys

import capstone
from capstone import x86

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sigscan import Image  # noqa: E402

BACK = 10      # bytes to step back from an operand to an instruction's start


def _decoder(bits):
    md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_64 if bits == 64 else capstone.CS_MODE_32)
    md.detail = True
    return md


def _kind(ins, target):
    """read, write or value for the operand of `ins` that is `target`, or None."""
    for op in ins.operands:
        if op.type == x86.X86_OP_IMM and op.imm & 0xffffffffffffffff == target:
            return 'value'
        if op.type == x86.X86_OP_MEM:
            m = op.mem
            addr = m.disp & 0xffffffffffffffff
            if m.base == x86.X86_REG_RIP:
                addr = (ins.address + ins.size + m.disp) & 0xffffffffffffffff
            if addr == target:
                if ins.mnemonic == 'lea':
                    return 'value'
                return 'write' if op.access & capstone.CS_AC_WRITE else 'read'
    return None


def _decode_at(md, data, start, at, operand, target):
    """The instruction that starts before `at` and holds `target` at byte `operand`, or None."""
    for back in range(1, BACK + 1):
        s = operand - back
        if s < 0:
            break
        for ins in md.disasm(data[s:s + 16], start + s, 1):
            if ins.size > back and s + ins.size >= operand + 4:
                kind = _kind(ins, target)
                if kind:
                    return ins, kind
    return None


def scan(img, targets):
    """[(address, kind, text, target)] for every instruction in the code that refers to a target."""
    found = {}
    for data, start, bits in img.regions:
        md = _decoder(bits)
        for target in targets:
            needle = struct.pack('<I', target & 0xffffffff)
            i = data.find(needle)
            while i >= 0:
                hit = _decode_at(md, data, start, i, i, target)
                if hit:
                    found[hit[0].address] = (hit[0], hit[1], target)
                i = data.find(needle, i + 1)
        if bits == 64:
            lo, hi = min(targets), max(targets)
            for i in range(len(data) - 3):
                disp = struct.unpack_from('<i', data, i)[0]
                base = start + i + 4 + disp            # the target if the instruction ends here
                if base > hi or base + 5 < lo:          # an immediate of up to 4 bytes may follow
                    continue
                for target in targets:
                    if 0 <= target - base <= 4 and start + i not in found:
                        hit = _decode_at(md, data, start, i, i, target)
                        if hit:
                            found[hit[0].address] = (hit[0], hit[1], target)
    rows = []
    for addr in sorted(found):
        ins, kind, target = found[addr]
        rows.append((addr, kind, f'{ins.mnemonic} {ins.op_str}'.strip(), target))
    return rows


def main(argv):
    if len(argv) < 2 or argv[0] in ('-h', '--help'):
        print(__doc__)
        return 0 if argv and argv[0] in ('-h', '--help') else 2
    opt = lambda name, default=None: argv[argv.index(name) + 1] if name in argv else default
    taken = {i + 1 for i, a in enumerate(argv) if a in ('--near', '--functions')}
    args = [a for i, a in enumerate(argv) if not a.startswith('--') and i not in taken]
    img = Image(args[0], 0)
    near = int(opt('--near', '0'), 0)
    targets = sorted({int(a, 16) + d for a in args[1:] for d in range(-near, near + 1)})
    funcs = functions(opt('--functions')) if opt('--functions') else None
    starts = [f[0] for f in funcs or []]
    rows = []
    for addr, kind, text, target in scan(img, targets):
        name = ''
        if funcs is not None:
            k = bisect.bisect_right(starts, addr) - 1
            name = funcs[k][1] if k >= 0 and _inside(funcs, k, addr) else '(no function)'
        rows.append({'address': hex(addr), 'kind': kind, 'instruction': text, 'target': hex(target), 'function': name})
    if '--json' in argv:
        print(json.dumps(rows, indent=2))
        return 0
    for r in rows:
        print(f'{r["address"]:>12s}  {r["kind"]:<5s}  {r["instruction"]:44s} {r["function"]}')
    print(f'{len(rows)} instructions')
    return 0


def functions(path):
    """[(start, name, size)] from a Ghidra export's functions.tsv (address, size, name, thunk)."""
    rows = []
    for line in open(path, encoding='utf-8', errors='replace'):
        parts = line.rstrip('\n').split('\t')
        if len(parts) >= 3:
            size = int(parts[1]) if parts[1].isdigit() else None
            rows.append((int(parts[0], 16), parts[2], size))
    return sorted(rows)


def _inside(funcs, k, addr):
    """True when addr lies in function k's body; a hit past its end is code the export missed."""
    size = funcs[k][2]
    return size is None or addr < funcs[k][0] + size


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
