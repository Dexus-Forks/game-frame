#!/usr/bin/env python3
"""Write a probe library: a stand-in for a DLL a game imports, which logs every call it gets.

Usage: probegen.py GAME.EXE DLL OUT_DIR [--words N] [--stdcall NAME=BYTES ...] [--log FILE]

  GAME.EXE   the 32-bit Windows program (or DLL) that imports DLL
  DLL        the library to stand in for, as the import table names it (glide2x.dll, S3DTKW.DLL)
  OUT_DIR    where to write NAME.c, NAME.def and build.sh
  --words N  for an import whose argument size is unknown, log N stack words (default 4)
  --stdcall NAME=BYTES   the import NAME is __stdcall and takes BYTES of arguments (the callee
             removes them). Decorated names (_grDrawTriangle@12) say this already; undecorated
             ones (S3DTK_InitLib) need it, or the game's stack goes wrong after the call
  --log FILE the log file the probe writes, in the game's folder (default probe-NAME.log)

Then `sh OUT_DIR/build.sh` builds NAME.dll with `zig cc -target x86-windows-gnu`. Put it beside
the game in place of the real library and run the game (in Wine or a VM). Each call appends one
line to the log: the import's name, then each argument word in hex. A pointer argument shows
as an address. Every stub returns 0.

Why: before a replacement library can draw anything, you need to know which calls the game
makes, in which order, and with what arguments. A probe answers that from the running game,
with no SDK and no headers. POD settled the layouts of two card APIs this way
(docs/hardware/replacement-libraries.md, "Start with a probe").

The exports are exactly the import table's names, through the .def file, so decorated
stdcall names such as `_grGlideInit@0` match. Imports by ordinal are exported at the same
ordinal. The probe uses only kernel32 (CreateFileA, WriteFile, CloseHandle): no C runtime.
It does not read through pointers, return useful values, or keep a game running past the
first call whose result it checks. Standard library and pefile.
"""
import os
import re
import sys

import pefile

DECORATED = re.compile(r'^_?([A-Za-z_][\w]*)@(\d+)$')


def imports(path, dll):
    pe = pefile.PE(path, fast_load=True)
    pe.parse_data_directories(directories=[pefile.DIRECTORY_ENTRY['IMAGE_DIRECTORY_ENTRY_IMPORT']])
    if pe.FILE_HEADER.Machine != 0x14c:
        raise SystemExit(f'{path}: not a 32-bit x86 program')
    for entry in getattr(pe, 'DIRECTORY_ENTRY_IMPORT', []):
        if entry.dll.decode(errors='replace').lower() == dll.lower():
            return [(i.name.decode() if i.name else None, i.ordinal) for i in entry.imports]
    names = [e.dll.decode(errors='replace') for e in getattr(pe, 'DIRECTORY_ENTRY_IMPORT', [])]
    raise SystemExit(f'{path} does not import {dll}; it imports {", ".join(names) or "nothing"}')


def plan(items, words, stdcall):
    """One stub per import: (export name, C name, argument words, stdcall?, ordinal)."""
    out = []
    for k, (name, ordinal) in enumerate(items):
        if name is None:
            out.append((None, f'probe_ord{ordinal}', words, False, ordinal))
            continue
        m = DECORATED.match(name)
        if m:
            out.append((name, 'probe_' + m.group(1), int(m.group(2)) // 4, True, None))
        elif name in stdcall:
            out.append((name, 'probe_' + re.sub(r'\W', '_', name), stdcall[name] // 4, True, None))
        else:
            out.append((name, 'probe_' + re.sub(r'\W', '_', name), words, False, None))
    return out


C_HEAD = r'''/* A probe for %(dll)s, written by GameFrame's tools/pe/probegen.py from %(game)s.
 * Each stub appends "NAME arg0 arg1 ..." to %(log)s and returns 0. kernel32 only. */
#include <windows.h>

static HANDLE log_file;

static void put(const char *s, int n) {
    DWORD done;
    if (!log_file)
        log_file = CreateFileA("%(log)s", FILE_APPEND_DATA, FILE_SHARE_READ, 0, OPEN_ALWAYS, 0, 0);
    if (log_file != INVALID_HANDLE_VALUE) WriteFile(log_file, s, n, &done, 0);
}

static void log_call(const char *name, const unsigned *arg, int n) {
    char line[512];
    int k = 0;
    while (*name && k < 200) line[k++] = *name++;
    for (int i = 0; i < n && k < 500; i++) {
        line[k++] = ' '; line[k++] = '0'; line[k++] = 'x';
        for (int s = 28; s >= 0; s -= 4) line[k++] = "0123456789abcdef"[(arg[i] >> s) & 15];
    }
    line[k++] = '\n';
    put(line, k);
}
'''


def c_source(stubs, dll, game, log):
    parts = [C_HEAD % {'dll': dll, 'game': os.path.basename(game), 'log': log.replace('\\', '\\\\')}]
    for export, cname, n, std, ordinal in stubs:
        label = export if export else f'#{ordinal}'
        params = ', '.join(f'unsigned a{i}' for i in range(n)) or 'void'
        values = ', '.join(f'a{i}' for i in range(n)) or '0'
        parts.append(f'unsigned {"__stdcall" if std else "__cdecl"} {cname}({params}) {{\n'
                     f'    unsigned v[] = {{{values}}};\n'
                     f'    log_call("{label}", v, {n});\n    return 0;\n}}\n')
    return '\n'.join(parts)


def def_file(stubs, dll):
    lines = [f'LIBRARY {os.path.splitext(dll)[0]}', 'EXPORTS']
    for export, cname, n, std, ordinal in stubs:
        inner = f'{cname}@{4 * n}' if std else cname
        if export is None:
            lines.append(f'  {inner} @{ordinal} NONAME')
        else:
            lines.append(f'  {export}={inner}')
    return '\n'.join(lines) + '\n'


def main(argv):
    if len(argv) < 3 or argv[0] in ('-h', '--help'):
        print(__doc__.strip())
        return 0 if argv and argv[0] in ('-h', '--help') else 2
    game, dll, out = argv[0], argv[1], argv[2]
    words, stdcall, log = 4, {}, None
    rest = argv[3:]
    for i, a in enumerate(rest):
        if a == '--words':
            words = int(rest[i + 1])
        elif a == '--stdcall':
            name, size = rest[i + 1].split('=')
            stdcall[name] = int(size, 0)
        elif a == '--log':
            log = rest[i + 1]
    name = os.path.splitext(os.path.basename(dll))[0]
    log = log or f'probe-{name}.log'
    stubs = plan(imports(game, dll), words, stdcall)
    os.makedirs(out, exist_ok=True)
    open(os.path.join(out, f'{name}.c'), 'w').write(c_source(stubs, dll, game, log))
    open(os.path.join(out, f'{name}.def'), 'w').write(def_file(stubs, dll))
    open(os.path.join(out, 'build.sh'), 'w').write(
        '#!/bin/sh\n# Build the probe: a 32-bit DLL that exports exactly the names the game imports.\n'
        f'cd "$(dirname "$0")" && zig cc -target x86-windows-gnu -O1 -shared {name}.c {name}.def -o {dll}\n')
    known = sum(1 for s in stubs if s[3])
    print(f'{len(stubs)} imports from {dll}: {known} with a known argument size, '
          f'{len(stubs) - known} logging {words} words; wrote {name}.c, {name}.def and build.sh to {out}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
