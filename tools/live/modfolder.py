#!/usr/bin/env python3
"""Put a folder of changed files into a game's install folder, list what is there, take it out.

Usage: modfolder.py install MOD_DIR INSTALL_DIR
       modfolder.py list INSTALL_DIR
       modfolder.py clear INSTALL_DIR

  MOD_DIR      a folder laid out like the game's data tree: Data/Binary/Voitures/CAR.BIN
  INSTALL_DIR  the game's install folder: under Wine, drive_c/GAME inside the prefix
               (derived/wine/drive_c/POD); in a VM, the same path there (run this inside it)

install copies every file of MOD_DIR to the same relative path under INSTALL_DIR, after
removing the files an earlier install put there, so a second install replaces the first and no
stale file stays behind. It refuses to replace a file the game already had: those are the
player's own, and a mod that needs it changed is a patch. It writes a manifest,
INSTALL_DIR/.gameframe-mod.json, with every file it added. list prints the manifest; clear
removes those files, then any folder left empty, then the manifest.

Why: many CD-era games open each data file under the install folder first and fall back to the
CD, so a changed file placed there replaces the disc's copy with no patch to the game
(docs/knowhow/context-mods-install-folder-first.md). Before recording a reference run, `list`
says whether any mod is in place (docs/knowhow/mods-loaded-in-a-reference-recording.md).

Under Wine the folder names match in any case; on a case-sensitive file system the files are
copied with MOD_DIR's spelling. It does not edit ini files or the registry, does not change the
game's own files, and does not touch the disc. Standard library only.
"""
import json
import os
import shutil
import sys

MANIFEST = '.gameframe-mod.json'


def manifest(install):
    path = os.path.join(install, MANIFEST)
    if not os.path.exists(path):
        return None
    with open(path, encoding='utf-8') as f:
        return json.load(f)


def clear(install, quiet=False):
    m = manifest(install)
    if not m:
        if not quiet:
            print(f'no mod installed in {install}')
        return 0
    for rel in m['files']:
        p = os.path.join(install, rel)
        if os.path.isfile(p):
            os.remove(p)
    for rel in sorted({os.path.dirname(r) for r in m['files']}, key=len, reverse=True):
        while rel:
            d = os.path.join(install, rel)
            if os.path.isdir(d) and not os.listdir(d):
                os.rmdir(d)
            rel = os.path.dirname(rel)
    os.remove(os.path.join(install, MANIFEST))
    if not quiet:
        print(f'removed {len(m["files"])} files of {m["from"]} from {install}')
    return 0


def install(mod, install_dir):
    if not os.path.isdir(mod):
        sys.exit(f'modfolder: no folder {mod}')
    os.makedirs(install_dir, exist_ok=True)
    clear(install_dir, quiet=True)
    files = sorted(os.path.relpath(os.path.join(r, f), mod)
                   for r, _, fs in os.walk(mod) for f in fs if not f.startswith('.'))
    clash = [f for f in files if os.path.exists(os.path.join(install_dir, f))]
    if clash:
        sys.exit('modfolder: these files are the game\'s own; a mod that replaces them is a patch:\n  '
                 + '\n  '.join(clash[:10]))
    for rel in files:
        dest = os.path.join(install_dir, rel)
        os.makedirs(os.path.dirname(dest), exist_ok=True)
        shutil.copyfile(os.path.join(mod, rel), dest)
        os.chmod(dest, 0o644)              # files from a read-only disc mount stay writable
    with open(os.path.join(install_dir, MANIFEST), 'w', encoding='utf-8') as f:
        json.dump({'from': os.path.abspath(mod), 'files': files}, f, indent=2)
    print(f'installed {len(files)} files from {mod} into {install_dir}')
    return 0


def main(argv):
    if len(argv) < 2 or argv[0] in ('-h', '--help') or argv[0] not in ('install', 'list', 'clear'):
        print(__doc__.rstrip())
        return 0 if argv and argv[0] in ('-h', '--help') else 2
    if argv[0] == 'install':
        if len(argv) != 3:
            sys.exit('usage: modfolder.py install MOD_DIR INSTALL_DIR')
        return install(argv[1], argv[2])
    if argv[0] == 'clear':
        return clear(argv[1])
    m = manifest(argv[1])
    if not m:
        print(f'no mod installed in {argv[1]}')
        return 1
    print(f'{len(m["files"])} files from {m["from"]}:')
    for rel in m['files']:
        print(f'  {rel}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
