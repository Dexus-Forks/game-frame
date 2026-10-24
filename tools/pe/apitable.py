#!/usr/bin/env python3
"""Write the API table of a 32-bit Windows DLL: every export, its calling convention and the
bytes of arguments it takes, read from the DLL's own code.

Usage: apitable.py DLL [--header FILE ...] [--out FILE]
       apitable.py IMPORT.LIB [--header FILE ...] [--out FILE]

  DLL          a 32-bit x86 DLL, such as a 1990s card driver library (glide2x.dll, sgl.dll)
  IMPORT.LIB   or the library's import library from its SDK, when the DLL is not at hand.
               Its decorated names (_ATI3DCIF_Init@0) give the stdcall exports
  --header F   a C header of the API. Each export's prototype there gives its parameter count
               and the kind of value it returns. Give several headers to search them all
  --out FILE   write the table there (default: standard output)

Output: a tab-separated table with a header line and one row per export, sorted by ordinal:

  ordinal  export  symbol  convention  bytes  params  returns  evidence

  export      the name as exported (`_grDrawTriangle@12`, `sgl_render`)
  symbol      the plain name (`grDrawTriangle`)
  convention  stdcall (the callee removes `bytes` of arguments), cdecl (the caller does),
              data (the export is not code), or unknown
  bytes       argument bytes for stdcall, from the decorated name or the `ret N` the
              function ends with; 0 for cdecl
  params      the parameter count from the header, or - when no header has the prototype
  returns     from the prototype: float (float or double, which comes back in the x87
              register st0), void, int (anything else, in eax), or - without one
  evidence    where the convention came from: name (decorated), ret (the code), name+ret
              (both agree), lib (an import library), or a note such as `ret differs: 8`

The tool follows the code of each export from its first byte with capstone. It follows both
paths of each conditional branch. It also follows direct jumps. Each `ret` found gives the
bytes the callee removes. A single value is the answer. No value leaves `unknown`: the code
ends in a jump through a pointer, or runs past 4096 instructions. Two values are reported as
a conflict. An export inside a section that is not executable is `data`.

A replacement library must export every call the original exports. Each call must remove the
same bytes from the stack. Otherwise a game that calls one the library lacks fails to load, or
crashes after the call. This table lists the calls to stub (wrappers/stubgen.py) and then to
implement.

It does not read the DLL's imports, follow calls into other functions, or say what each
argument is. Standard library, pefile and capstone. tests/run.sh checks it against a DLL
built from our own C.
"""
import re
import struct
import sys

import pefile
from capstone import CS_ARCH_X86, CS_MODE_32, Cs

DECORATED = re.compile(r'^_?([A-Za-z_]\w*)@(\d+)$')
MAX_INSNS = 4096


def code_rets(pe, image, rva):
    """The set of `ret` immediates reachable from rva, and whether the walk was complete."""
    md = Cs(CS_ARCH_X86, CS_MODE_32)
    base = pe.OPTIONAL_HEADER.ImageBase
    todo, seen, rets, done = [rva], set(), set(), True
    count = 0
    while todo:
        at = todo.pop()
        while at not in seen and 0 <= at < len(image):
            seen.add(at)
            insn = next(md.disasm(image[at:at + 16], base + at), None)
            count += 1
            if insn is None or count > MAX_INSNS:
                return rets, False
            at += insn.size
            m = insn.mnemonic
            if m in ('ret', 'retn'):
                rets.add(int(insn.op_str, 0) if insn.op_str else 0)
                break
            if m.startswith('j') and m != 'jmp':
                target = target_of(insn, base)
                if target is not None:
                    todo.append(target)
                continue
            if m == 'jmp':
                target = target_of(insn, base)
                if target is None:
                    done = False
                else:
                    todo.append(target)
                break
            if m in ('int3', 'hlt', 'ud2'):
                break
    return rets, done


def target_of(insn, base):
    try:
        return int(insn.op_str, 16) - base
    except ValueError:
        return None


def executable(pe, rva):
    for s in pe.sections:
        if s.VirtualAddress <= rva < s.VirtualAddress + max(s.Misc_VirtualSize, s.SizeOfRawData):
            return bool(s.Characteristics & 0x20000020)  # executable, or marked as code
    return False


def dll_rows(path):
    pe = pefile.PE(path)
    if pe.FILE_HEADER.Machine != 0x14c:
        raise SystemExit(f'{path}: not a 32-bit x86 file')
    image = pe.get_memory_mapped_image()
    rows = []
    for e in (pe.DIRECTORY_ENTRY_EXPORT.symbols if hasattr(pe, 'DIRECTORY_ENTRY_EXPORT') else []):
        name = e.name.decode() if e.name else f'@{e.ordinal}'
        m = DECORATED.match(name)
        symbol = m.group(1) if m else name
        if e.forwarder or not executable(pe, e.address):
            rows.append([e.ordinal, name, symbol, 'data', 0, 'name' if m else '-'])
            continue
        rets, complete = code_rets(pe, image, e.address)
        if m:
            want = int(m.group(2))
            if rets and rets != {want}:
                ev = f'ret differs: {",".join(map(str, sorted(rets)))}'
            else:
                ev = 'name+ret' if rets else 'name'
            rows.append([e.ordinal, name, symbol, 'stdcall', want, ev])
        elif len(rets) == 1:
            n = rets.pop()
            rows.append([e.ordinal, name, symbol, 'stdcall' if n else 'cdecl', n, 'ret'])
        elif rets:
            rows.append([e.ordinal, name, symbol, 'unknown', 0, f'rets conflict: {",".join(map(str, sorted(rets)))}'])
        else:
            rows.append([e.ordinal, name, symbol, 'unknown', 0, 'no ret' + ('' if complete else ' (jump through a pointer)')])
    return sorted(rows, key=lambda r: r[0])


def lib_rows(path):
    """Decorated names from an import library's members (short import headers name them)."""
    data = open(path, 'rb').read()
    if not data.startswith(b'!<arch>\n'):
        raise SystemExit(f'{path}: not a COFF archive (.lib)')
    names = sorted(set(re.findall(rb'(?<![\w@])(_[A-Za-z_]\w*@\d+)(?=\x00)', data)))
    names = [n.decode() for n in names if not n.startswith(b'__imp_')]
    rows = []
    for i, name in enumerate(names, 1):
        m = DECORATED.match(name)
        rows.append([i, name, m.group(1), 'stdcall', int(m.group(2)), 'lib'])
    return rows


def kind(ret):
    """The kind of a return type: float (in st0), void, or int (in eax)."""
    if re.search(r'\b(float|double|FxFloat)\b', ret):
        return 'float'
    if re.search(r'\bvoid\b', ret) and '*' not in ret:
        return 'void'
    return 'int'


def prototypes(headers):
    """name -> (parameter count, return kind) from C prototypes."""
    out = {}
    for h in headers:
        text = open(h, encoding='latin-1').read()
        text = re.sub(r'/\*.*?\*/', ' ', text, flags=re.S)
        text = re.sub(r'//[^\n]*', ' ', text)
        text = re.sub(r'#[^\n]*', ' ', text)
        for m in re.finditer(r'([\w\s\*]+?)\b(\w+)\s*\(([^()]*(?:\([^()]*\)[^()]*)*)\)\s*;', text):
            ret, name, args = m.group(1), m.group(2), m.group(3).strip()
            if name in out:
                continue
            words = ret.split()
            if not words or words[-1] in ('return', 'if', 'while'):
                continue
            n = 0 if args in ('', 'void') else args.count(',') + 1
            out[name] = (n, kind(' '.join(words[-3:])))
        # ATI's SDK declares each call through a macro: PROTOTYPE (name, return type, (args)).
        for m in re.finditer(r'PROTOTYPE\s*\(\s*(\w+)\s*,\s*([\w\s\*]+?)\s*,\s*\(([^()]*)\)\s*\)', text):
            name, ret, args = m.group(1), m.group(2), m.group(3).strip()
            n = 0 if args in ('', 'void') else args.count(',') + 1
            out.setdefault(name, (n, kind(ret)))
    return out


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__)
        return 0 if argv else 2
    path, headers, out = argv[0], [], None
    i = 1
    while i < len(argv):
        if argv[i] == '--header' and i + 1 < len(argv):
            headers.append(argv[i + 1])
            i += 2
        elif argv[i] == '--out' and i + 1 < len(argv):
            out = argv[i + 1]
            i += 2
        else:
            raise SystemExit(f'unknown argument {argv[i]} (see --help)')
    with open(path, 'rb') as f:
        magic = f.read(8)
    rows = lib_rows(path) if magic == b'!<arch>\n' else dll_rows(path)
    protos = prototypes(headers)
    lines = ['ordinal\texport\tsymbol\tconvention\tbytes\tparams\treturns\tevidence']
    for ordinal, name, symbol, conv, n, ev in rows:
        p = protos.get(symbol)
        params = str(p[0]) if p else '-'
        returns = p[1] if p else '-'
        lines.append(f'{ordinal}\t{name}\t{symbol}\t{conv}\t{n}\t{params}\t{returns}\t{ev}')
    text = '\n'.join(lines) + '\n'
    if out:
        open(out, 'w').write(text)
    else:
        sys.stdout.write(text)
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
