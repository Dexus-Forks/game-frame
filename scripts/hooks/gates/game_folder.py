#!/usr/bin/env python3
"""The game folder of a GameFrame project: target/, the user's own copy of the game.

Usage:

    game_folder.py mark DIR     write target/GAME-FILES-NOT-BUILD-OUTPUT.txt in project DIR
                                (and target/ itself when it is missing)
    game_folder.py check DIR    one line on the state of DIR/target/; exit 1 when it is missing,
                                or empty while the documents name files in it, so
                                `gameframe doctor` counts it as missing

In a GameFrame project target/ holds the disc images, the installed or extracted files and the
saves. Cargo's build output has the same default name, but a project's .cargo/config.toml sends
Cargo to build/. Disk cleaners and agents that delete every target/ next to a Cargo.toml take
the game with it (docs/knowhow/game-folder-deleted-as-build-output.md). The marker says so to
anyone who lists the folder. It is not a CACHEDIR.TAG: that file tells backup tools to skip a
folder, the opposite of what a game copy needs. A real Cargo target/ has a CACHEDIR.TAG, and
a game folder has none.

is_game_folder(path) is what the delete gate (deletes.py) asks. check also lists files that
the project's documents name in target/ but that are not there. It does not restore anything
and does not read the game's files. Standard library only.
"""
import os
import re
import sys

MARKER = 'GAME-FILES-NOT-BUILD-OUTPUT.txt'
TEXT = """\
This folder is your copy of the game, not build output.

It holds the game's files for this GameFrame project: disc images, the installed or extracted
files and your saves. Git ignores it, so nothing else keeps a copy. Deleting it loses them.

Cargo builds into build/ in this project (.cargo/config.toml sets target-dir = "build"), so
`cargo clean` never touches this folder. A real Cargo target/ folder holds a CACHEDIR.TAG file;
this one has none.

Disk cleaners and scripts that delete every target/ folder next to a Cargo.toml would delete
this one. Leave it out of such cleanups, and keep a backup of the original discs and saves.
See GameFrame's docs/knowhow/game-folder-deleted-as-build-output.md.
"""
# A path in a document: target/ and a file name with an extension, no wildcards.
# The example name in GameFrame's project templates, which no game needs to have.
PLACEHOLDERS = {'GAME.EXE'}
DOC_PATH = re.compile(r'(?<![\w/.-])target/([\w./-]*\.[A-Za-z0-9]{1,4})(?![\w*?])')


def cargo_target_dir(project):
    """The target-dir that project/.cargo/config.toml sets, or None."""
    try:
        text = open(os.path.join(project, '.cargo', 'config.toml'), encoding='utf-8').read()
    except OSError:
        return None
    m = re.search(r'^\s*target-dir\s*=\s*["\']([^"\']+)["\']', text, re.M)
    return m.group(1) if m else None


def is_game_folder(path):
    """True when path is a GameFrame project's target/: it holds the marker, or the folder above
    sends Cargo elsewhere, or names GameFrame in its AGENTS.md."""
    path = os.path.realpath(os.path.expanduser(path))
    if os.path.basename(path) != 'target' or not os.path.isdir(path):
        return False
    if os.path.isfile(os.path.join(path, MARKER)):
        return True
    project = os.path.dirname(path)
    other = cargo_target_dir(project)
    if other and os.path.normpath(other) != 'target':
        return True
    try:
        return 'GameFrame' in open(os.path.join(project, 'AGENTS.md'), encoding='utf-8').read()
    except OSError:
        return False


def mark(project):
    folder = os.path.join(project, 'target')
    os.makedirs(folder, exist_ok=True)
    path = os.path.join(folder, MARKER)
    if not os.path.isfile(path) or open(path, encoding='utf-8').read() != TEXT:
        with open(path, 'w', encoding='utf-8') as f:
            f.write(TEXT)
        return True
    return False


def named_files(project):
    """target/ paths named in AGENTS.md, README.md and docs/*.md, without wildcards."""
    names, docs = set(), [os.path.join(project, f) for f in ('AGENTS.md', 'README.md')]
    root = os.path.join(project, 'docs')
    for d, subs, files in os.walk(root):
        subs[:] = [s for s in subs if not s.startswith('.')]
        docs += [os.path.join(d, f) for f in files if f.endswith('.md')]
    for doc in docs:
        try:
            text = open(doc, encoding='utf-8', errors='replace').read()
        except OSError:
            continue
        names.update(DOC_PATH.findall(text))
    return sorted(names - PLACEHOLDERS)


def check(project):
    """(status, words, ok): one row for `gameframe doctor`."""
    folder = os.path.join(project, 'target')
    see = 'see docs/knowhow/game-folder-deleted-as-build-output.md'
    if not os.path.isdir(folder):
        return 'MISSING', f'target/ (your copy of the game) is gone: restore it from the discs or a backup; {see}', False
    files = [f for f in os.listdir(folder) if f not in (MARKER, '.DS_Store')]
    missing = [n for n in named_files(project) if not os.path.exists(os.path.join(folder, n))]
    gone = f'; the documents name {len(missing)} missing, such as target/{missing[0]}' if missing else ''
    if not files and not missing:
        return 'empty', 'target/ holds no game files yet: copy the game in (re-inventory)', True
    if not files:
        return 'EMPTY', f'target/ holds no game files{gone}: restore them from the discs or a backup; {see}', False
    if os.path.isfile(os.path.join(folder, 'CACHEDIR.TAG')):
        return 'CARGO', 'target/ has a CACHEDIR.TAG: Cargo built into the game folder; set target-dir = "build" in .cargo/config.toml', True
    marked = 'marked' if not mark(project) else 'marker written'
    return ('ok' if not missing else 'check'), f'{len(files)} entries, {marked}{gone}', True


def main(argv):
    if len(argv) != 3 or argv[1] not in ('mark', 'check'):
        print(__doc__)
        return 0 if len(argv) == 1 else 2
    if argv[1] == 'mark':
        mark(argv[2])
        return 0
    status, words, ok = check(argv[2])
    print(f'  {"game folder":<14} {status:<8} {words}')
    return 0 if ok else 1


if __name__ == '__main__':
    sys.exit(main(sys.argv))
