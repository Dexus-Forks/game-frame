#!/usr/bin/env python3
"""The game folder of a GameFrame project: game/, the user's own copy of the game.

Usage:

    game_folder.py mark DIR     write GAME-FILES-NOT-BUILD-OUTPUT.txt in project DIR's game folder
                                (and the folder itself when it is missing)
    game_folder.py check DIR    one line on the state of DIR's game folder; exit 1 when it is
                                missing, or empty while the documents name files in it, so
                                `gameframe doctor` counts it as missing

A project's game folder is game/, a link into the GameFrame home (games/ID/game/, made by
`gameframe game link` or `home adopt`), or a real folder. Older projects have target/ instead,
and the same rules hold for it. target/ is also Cargo's default build folder, so disk cleaners
and agents that delete every target/ next to a Cargo.toml take the game with it
(docs/knowhow/game-folder-deleted-as-build-output.md). The marker says what the folder is to
anyone who lists it. It is not a CACHEDIR.TAG: that file tells backup tools to skip a folder, the
opposite of what a game copy needs. A real Cargo target/ has a CACHEDIR.TAG, and a game folder
has none.

is_game_folder(path) is what the delete gate (deletes.py) asks. It also knows the home's
games/ID/game/ and games/ID/media/, beside their game.json. check also lists files that the
project's documents name in the game folder but that are not there. It does not restore
anything and does not read the game's files. Standard library only.
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
DOC_PATH = re.compile(r'(?<![\w/.-])(?:target|game)/([\w./-]*\.[A-Za-z0-9]{1,4})(?![\w*?])')


def cargo_target_dir(project):
    """The target-dir that project/.cargo/config.toml sets, or None."""
    try:
        text = open(os.path.join(project, '.cargo', 'config.toml'), encoding='utf-8').read()
    except OSError:
        return None
    m = re.search(r'^\s*target-dir\s*=\s*["\']([^"\']+)["\']', text, re.M)
    return m.group(1) if m else None


def folder_name(project):
    """The project's game folder: game/ (a link into the GameFrame home, or a folder), or the
    older target/ when only that exists."""
    if not os.path.lexists(os.path.join(project, 'game')) and os.path.isdir(os.path.join(project, 'target')):
        return 'target'
    return 'game'


def is_game_folder(path):
    """True when path is a game's files: game/ or media/ in the GameFrame home (beside a
    game.json), or a project's game/ or older target/ that holds the marker, sends Cargo
    elsewhere, or names GameFrame in its AGENTS.md."""
    path = os.path.realpath(os.path.expanduser(path))
    base = os.path.basename(path)
    if base not in ('target', 'game', 'media') or not os.path.isdir(path):
        return False
    if base in ('game', 'media') and os.path.isfile(os.path.join(os.path.dirname(path), 'game.json')):
        return True
    if base == 'media':
        return False
    if os.path.isfile(os.path.join(path, MARKER)):
        return True
    project = os.path.dirname(path)
    if base == 'game':
        try:
            return 'GameFrame' in open(os.path.join(project, 'AGENTS.md'), encoding='utf-8').read()
        except OSError:
            return False
    other = cargo_target_dir(project)
    if other and os.path.normpath(other) != 'target':
        return True
    try:
        return 'GameFrame' in open(os.path.join(project, 'AGENTS.md'), encoding='utf-8').read()
    except OSError:
        return False


def mark(project):
    folder = os.path.join(project, folder_name(project))
    os.makedirs(folder, exist_ok=True)
    path = os.path.join(folder, MARKER)
    if not os.path.isfile(path) or open(path, encoding='utf-8').read() != TEXT:
        with open(path, 'w', encoding='utf-8') as f:
            f.write(TEXT)
        return True
    return False


def named_files(project):
    """game/ and target/ paths named in AGENTS.md, README.md and docs/*.md, without wildcards."""
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
    name = folder_name(project)
    folder = os.path.join(project, name)
    see = 'see docs/knowhow/game-folder-deleted-as-build-output.md'
    if os.path.islink(folder) and not os.path.isdir(folder):
        return 'MISSING', f'game/ links to {os.readlink(folder)}, which is gone: restore it, or run gameframe home adopt; {see}', False
    if not os.path.isdir(folder):
        return 'MISSING', f'{name}/ (your copy of the game) is gone: restore it from the discs or a backup; {see}', False
    files = [f for f in os.listdir(folder) if f not in (MARKER, '.DS_Store')]
    missing = [n for n in named_files(project) if not os.path.exists(os.path.join(folder, n))]
    gone = f'; the documents name {len(missing)} missing, such as {name}/{missing[0]}' if missing else ''
    if not files and not missing:
        return 'empty', f'{name}/ holds no game files yet: gameframe game add ID DISC.iso (re-inventory)', True
    if not files:
        return 'EMPTY', f'{name}/ holds no game files{gone}: restore them from the discs or a backup; {see}', False
    if os.path.isfile(os.path.join(folder, 'CACHEDIR.TAG')):
        return 'CARGO', f'{name}/ has a CACHEDIR.TAG: Cargo built into the game folder; set target-dir = "build" in .cargo/config.toml', True
    marked = 'marked' if not mark(project) else 'marker written'
    where = f' in {os.path.realpath(folder)}' if os.path.islink(folder) else ''
    return ('ok' if not missing else 'check'), f'{len(files)} entries{where}, {marked}{gone}', True


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
