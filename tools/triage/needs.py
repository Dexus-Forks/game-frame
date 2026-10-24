#!/usr/bin/env python3
"""List what a game needs to run: CPU features, operating system, graphics and sound APIs,
libraries, extenders, DRM and media, each with its evidence.

Usage: needs.py PATH ... [--json] [--no-cpu] [--max-files N]

  PATH         an executable or DLL (PE, NE, LE/LX or DOS MZ), a game's folder, or a disc image
               (.cue, .bin or .iso, read in place through tools/disc)
  --json       print the needs as JSON: [{"need": "api:glide2", "evidence": ["GAME.EXE imports
               glide2x.dll"]}, ...]
  --no-cpu     skip the scan of the code for MMX, 3DNow!, SSE and AVX instructions (the slow part)
  --max-files  read at most N executables (default 200)

A need is `kind:value`: cpu:mmx, os:windows-9x, os:windows-3x, os:os2, os:dos, api:glide2,
api:directdraw, api:xaudio2, lib:miles, extender:dos4gw, drm:steam, drm:steamstub,
drm:safedisc, store:gog-galaxy, media:cd-check, and so on. The signs and their needs are in
needs_signs.py. tools/live/harness.py reads this list and picks a module for each need.

It reads imports, the PE subsystem version, the NE target OS byte, section names, file names,
a few strings, and (for CPU features) every instruction of the code sections with capstone.

What it does not do: prove a need. An import shows the program can use an API, not that the
build you run does; a CPU feature found in the code may sit on a path that checks the CPU
first (re-vectoriser). Read each need as a lead, and confirm it by running the game.
"""
import json
import os
import re
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(os.path.dirname(HERE), 'disc'))
import needs_signs as signs  # noqa: E402

EXE = re.compile(r'\.(exe|dll|ovl|drv|386|vxd|com|bin)$', re.I)


class Needs:
    def __init__(self):
        self.found = {}

    def add(self, need, why):
        ev = self.found.setdefault(need, [])
        if why not in ev and len(ev) < 6:
            ev.append(why)


def scan_cpu(code, bits, name, out):
    try:
        import capstone
    except ImportError:
        return
    md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_64 if bits == 64 else capstone.CS_MODE_32)
    md.skipdata = True
    seen = {'sse'} if bits == 64 else set()     # every x86-64 CPU has SSE2: not a need
    for _, _, mnem, ops in md.disasm_lite(code, 0):
        if not mnem.startswith('p') and not mnem.startswith('f') and not mnem.startswith('v') \
                and 'mm' not in ops and not mnem.endswith(('ps', 'pd', 'ss', 'sd')):
            continue
        if signs.NOW3D.match(mnem) and '3dnow' not in seen:
            seen.add('3dnow'); out.add('cpu:3dnow', f'{name} has 3DNow! instructions ({mnem})')
        elif signs.AVX.search(ops) and 'avx' not in seen:
            seen.add('avx'); out.add('cpu:avx', f'{name} has AVX instructions ({mnem} {ops})')
        elif signs.SSE.search(ops) and 'sse' not in seen:
            seen.add('sse'); out.add('cpu:sse', f'{name} has SSE instructions ({mnem} {ops})')
        elif signs.MMX.search(ops) and 'mmx' not in seen:
            seen.add('mmx'); out.add('cpu:mmx', f'{name} has MMX instructions ({mnem} {ops})')
        if len(seen) == 5 or (len(seen) == 4 and 'avx' in seen):
            break


def read_pe(data, name, out, cpu):
    import pefile
    pe = pefile.PE(data=data, fast_load=True)
    pe.parse_data_directories(directories=[pefile.DIRECTORY_ENTRY['IMAGE_DIRECTORY_ENTRY_IMPORT']])
    bits = 64 if pe.FILE_HEADER.Machine == 0x8664 else 32
    out.add('cpu:x64' if bits == 64 else 'cpu:x86', f'{name} is a {bits}-bit PE')
    oh = pe.OPTIONAL_HEADER
    out.add(signs.pe_os(oh.MajorSubsystemVersion, oh.MinorSubsystemVersion),
            f'{name} asks for subsystem {oh.MajorSubsystemVersion}.{oh.MinorSubsystemVersion}')
    funcs = set()
    for entry in getattr(pe, 'DIRECTORY_ENTRY_IMPORT', []):
        dll = entry.dll.decode('latin-1').lower()
        for pattern, need in signs.IMPORTS:
            if re.fullmatch(pattern, dll):
                out.add(need, f'{name} imports {dll}')
        funcs.update((i.name or b'').decode('latin-1') for i in entry.imports)
    if {'GetDriveTypeA', 'GetVolumeInformationA'} <= funcs:
        out.add('media:cd-check', f'{name} imports GetDriveTypeA and GetVolumeInformationA (a disc check)')
    for s in pe.sections:
        sec = s.Name.rstrip(b'\0').decode('latin-1')
        if sec in signs.SECTIONS:
            out.add(signs.SECTIONS[sec], f'{name} has a {sec} section')
        if cpu and s.Characteristics & 0x20000000:      # IMAGE_SCN_MEM_EXECUTE
            scan_cpu(s.get_data(), bits, name, out)


def read_exe(data, name, out, cpu):
    if data[:2] != b'MZ' or len(data) < 0x40:
        return
    at = struct.unpack_from('<I', data, 0x3c)[0]
    sig = data[at:at + 2] if at + 2 <= len(data) and at >= 0x40 else b''
    if sig == b'PE':
        read_pe(data, name, out, cpu)
        return
    if sig == b'NE':
        target = data[at + 0x36]
        major, minor = data[at + 0x3f], data[at + 0x3e]
        need = signs.NE_OS.get(target, 'os:windows-3x')
        out.add(need, f'{name} is a 16-bit NE for target OS {target}' + (f', version {major}.{minor}' if major else ''))
    elif sig in (b'LE', b'LX'):
        out.add('os:os2' if sig == b'LX' else 'os:dos', f'{name} is a 32-bit {sig.decode()} program')
    else:
        out.add('os:dos', f'{name} is a DOS MZ program')
    for needle, need in signs.STRINGS:
        if needle in data:
            out.add(need, f'{name} contains "{needle.decode("latin-1")}"')


def files_of(path, limit):
    """(relative name, bytes or None) for every file: a folder, a disc image, or one file."""
    if os.path.isdir(path):
        for root, _, names in os.walk(path):
            for n in sorted(names):
                full = os.path.join(root, n)
                rel = os.path.relpath(full, path).replace(os.sep, '/')
                yield rel, (lambda f=full: open(f, 'rb').read())
    elif re.search(r'\.(cue|bin|iso|img)$', path, re.I):
        from image import open_image
        from volumes import scan
        disc = open_image(path)
        for t in disc.tracks:
            for v in scan(t):
                if v.root is None or v.kind == 'Joliet':
                    continue
                for e in v.entries():
                    if not e.directory:
                        yield e.path, (lambda v=v, e=e: v.file(e))
                break
    else:
        yield os.path.basename(path), (lambda: open(path, 'rb').read())


def analyse(paths, cpu=True, limit=200):
    out, n = Needs(), 0
    for path in paths:
        for rel, read in files_of(path, limit):
            low = rel.lower()
            for pattern, need in signs.FILES:
                if re.search(pattern, low):
                    out.add(need, f'file {rel}')
            if EXE.search(low) and n < limit:
                data = read()
                if data[:2] == b'MZ':
                    n += 1
                    try:
                        read_exe(data, rel, out, cpu)
                    except Exception as e:      # a damaged or packed file: say so and go on
                        out.add('unknown:unreadable', f'{rel}: {type(e).__name__}: {e}')
    return out.found


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__); return 0
    as_json, cpu = '--json' in argv, '--no-cpu' not in argv
    limit = 200
    if '--max-files' in argv:
        i = argv.index('--max-files')
        limit = int(argv[i + 1]); del argv[i:i + 2]
    paths = [a for a in argv if not a.startswith('--')]
    found = analyse(paths, cpu, limit)
    order = ['cpu', 'os', 'extender', 'api', 'lib', 'drm', 'store', 'media', 'packer', 'unknown']
    keys = sorted(found, key=lambda k: (order.index(k.split(':')[0]) if k.split(':')[0] in order else 99, k))
    if as_json:
        print(json.dumps([{'need': k, 'evidence': found[k]} for k in keys], indent=1))
    else:
        for k in keys:
            print(f'{k:<28} {found[k][0]}' + (f' (+{len(found[k]) - 1} more)' if len(found[k]) > 1 else ''))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
