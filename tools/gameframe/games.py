"""One game's files in the GameFrame home: games/ID/, outside every git repository.

Usage:
  gameframe game add ID FILE... [--version NAME] [--move] [--no-extract]
        copy disc images, installers or an installed game's folder into games/ID/media/
        (media/NAME/ with --version), record each file's size and SHA-256 in game.json, then
        extract the first one into games/ID/game/ when that is empty. --move moves the files
        instead of copying them. A folder is copied into game/ as it is.
  gameframe game list
        every game in the home: its images and whether game/ holds files
  gameframe game link ID PROJECT
        make PROJECT's game, media, saves, analysis and derived links into games/ID/
  gameframe home adopt PROJECT --id ID [--move-project]
        move an existing project's game folders into games/ID/ and leave links in their place:
        target/ and game-data/ become game/, save-game/ becomes saves/, truth/ goes under
        derived/truth/. Each old name stays as a link to its new place, so the project's code
        keeps working. --move-project then moves the repository to projects/ID/ and leaves a
        link at its old path.

games/ID/:
  game.json   {"id", "media": [{"file", "size", "sha256", "version"}], "extracted_from"}
  media/      the discs and installers as they came; read-only after they are added
  game/       the extracted or installed game: what the tools read and the launcher runs
  saves/, analysis/, derived/

Extraction: CD images (.iso, .cue, .bin, .img) with tools/disc/discinfo.py --extract, and
installers and archives with scripts/extract-installer.sh. It never runs an installer.

It never overwrites or deletes a file: a name that is already in media/ with other bytes, or a
project folder that a link would replace and whose files have nowhere to go, stops it with a
message. Standard library only.
"""
import hashlib
import json
import os
import shutil
import stat
import subprocess
from pathlib import Path

from home import home
from paths import NAME, ROOT, SCRIPTS, python

PARTS = ('media', 'game', 'saves', 'analysis', 'derived')
LINKS = ('game', 'media', 'saves', 'analysis', 'derived')
IMAGES = {'.iso', '.cue', '.bin', '.img', '.mdf', '.nrg', '.chd'}
# Older names for a project's game folders, and where each goes.
OLD = {'target': 'game', 'game-data': 'game', 'save-game': 'saves', 'truth': 'derived/truth',
       'analysis': 'analysis', 'derived': 'derived'}


class Stop(Exception):
    pass


def folder(game_id):
    if not game_id or not all(c.isalnum() or c in '-_' for c in game_id):
        raise Stop(f'{game_id!r}: a game id is letters, digits, - and _')
    return home() / 'games' / game_id


def sha256(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for block in iter(lambda: f.read(1 << 20), b''):
            h.update(block)
    return h.hexdigest()


def load(game):
    try:
        return json.loads((game / 'game.json').read_text())
    except (OSError, ValueError):
        return {'id': game.name, 'media': []}


def save(game, data):
    (game / 'game.json').write_text(json.dumps(data, indent=2) + '\n')


def make(game):
    for part in PARTS:
        (game / part).mkdir(parents=True, exist_ok=True)
    # The delete gate knows a game's folders by this file beside them.
    if not (game / 'game.json').exists():
        save(game, {'id': game.name, 'media': []})


def empty(path):
    return not path.exists() or not any(p for p in path.iterdir() if p.name not in ('.DS_Store', 'GAME-FILES-NOT-BUILD-OUTPUT.txt'))


def extract(source, into):
    """Extract a disc image or an installer into a folder. Returns True when it worked."""
    if source.suffix.lower() in IMAGES:
        py, _ = python()
        cmd = [py, str(ROOT / 'tools' / 'disc' / 'discinfo.py'), str(source), '--extract', str(into)]
    else:
        cmd = ['bash', str(SCRIPTS / 'extract-installer.sh'), str(source), str(into)]
    run = subprocess.run(cmd, stdout=subprocess.DEVNULL)
    return run.returncode == 0


def add(args):
    game = folder(args.id)
    make(game)
    data = load(game)
    media = game / 'media' / (args.version or '')
    media.mkdir(parents=True, exist_ok=True)
    added = []
    for name in args.files:
        src = Path(name).expanduser()
        if src.is_dir():
            if not empty(game / 'game'):
                raise Stop(f'{game / "game"} already holds files: copy {src} by hand if you want both')
            shutil.copytree(src, game / 'game', dirs_exist_ok=True)
            data['extracted_from'] = str(src)
            print(f'copied the folder {src} into game/')
            continue
        if not src.is_file():
            raise Stop(f'{src} is not a file or a folder')
        dest = media / src.name
        digest = sha256(src)
        if dest.exists():
            if sha256(dest) != digest:
                raise Stop(f'{dest} exists with other bytes: give --version a name to keep both')
        elif args.move:
            shutil.move(str(src), dest)
        else:
            shutil.copy2(src, dest)
        rel = str(dest.relative_to(game))
        data['media'] = [m for m in data.get('media', []) if m['file'] != rel]
        data['media'].append({'file': rel, 'size': dest.stat().st_size, 'sha256': digest, 'version': args.version})
        added.append(dest)
        print(f'{rel}  {digest[:16]}...')
    save(game, data)
    first = next((p for p in added if p.suffix.lower() != '.bin' or not p.with_suffix('.cue').exists()), None)
    if first and not args.no_extract and empty(game / 'game'):
        if extract(first, game / 'game'):
            data['extracted_from'] = str(first.relative_to(game))
            save(game, data)
            print(f'extracted {first.name} into game/')
        else:
            print(f'could not extract {first.name}: extract it into {game / "game"} by hand')
            return 1
    return 0


def list_games(_args):
    root = home() / 'games'
    games = sorted(p for p in root.glob('*') if p.is_dir()) if root.is_dir() else []
    if not games:
        print(f'No games in {root}. Add one with `{NAME} game add ID FILE`.')
    for g in games:
        data = load(g)
        state = 'empty' if empty(g / 'game') else 'ready'
        print(f'{g.name:<20}game/ {state:<6} {len(data.get("media", []))} media files')
        for m in data.get('media', []):
            print(f'    {m["file"]}  {m["size"]:,} bytes')
    return 0


def link(game_id, project):
    """Make project's links into games/ID/. A real folder in the way must be empty."""
    game = folder(game_id)
    make(game)
    project = Path(project).expanduser().resolve()
    for name in LINKS:
        p = project / name
        if p.is_symlink():
            if p.resolve() == (game / name).resolve():
                continue
            p.unlink()
        elif p.exists():
            if not empty(p):
                raise Stop(f'{p} holds files: run `{NAME} home adopt {project} --id {game_id}` to move them')
            shutil.rmtree(p)
        p.symlink_to(game / name, target_is_directory=True)
    return game


def link_cmd(args):
    link(args.id, args.project)
    print(f'linked {", ".join(LINKS)} in {args.project} to {folder(args.id)}')
    return 0


def _move_into(src, dest):
    """Move src's files into dest. Stops before moving anything if a name is in both."""
    dest.mkdir(parents=True, exist_ok=True)
    clash = [p.name for p in src.iterdir() if (dest / p.name).exists() and p.name != '.DS_Store']
    if clash:
        raise Stop(f'{dest} already has {", ".join(clash[:5])}: move {src} by hand')
    for p in src.iterdir():
        if p.name == '.DS_Store':
            p.unlink()
        elif p.is_dir() and not p.is_symlink() and not os.access(p, os.W_OK):
            # macOS needs a folder writable to give it a new parent. A read-only game copy stays
            # read-only afterwards.
            mode = p.stat().st_mode
            p.chmod(mode | stat.S_IWUSR)
            p.rename(dest / p.name)
            (dest / p.name).chmod(mode)
        else:
            shutil.move(str(p), dest / p.name)
    src.rmdir()


def adopt(args):
    project = Path(args.project).expanduser().resolve()
    if not (project / '.git').exists():
        raise Stop(f'{project} is not a git repository')
    game = folder(args.id)
    make(game)
    moved = []
    for old, new in OLD.items():
        src = project / old
        if src.is_symlink() or not src.is_dir():
            continue
        # A folder with files in git belongs to the repository: it stays.
        tracked = subprocess.run(['git', '-C', str(project), 'ls-files', '--', old],
                                 capture_output=True, text=True).stdout.split()
        if tracked:
            print(f'kept {old}/: git tracks {len(tracked)} files in it')
            continue
        _move_into(src, game / new)
        print(f'moved {old}/ to {game / new}')
        moved.append((old, new))
    link(args.id, project)
    # The old names stay as links, so the project's own code and documents keep working.
    for old, new in moved:
        if old not in LINKS:
            (project / old).symlink_to(game / new, target_is_directory=True)
            print(f'left {old} -> {game / new}')
    ignore = project / '.gitignore'
    text = ignore.read_text() if ignore.exists() else ''
    names = list(LINKS) + [old for old, _ in moved if old not in LINKS]
    lines = [f'/{n}' for n in names if f'/{n}\n' not in text]
    if lines:
        ignore.write_text(text + ('' if text.endswith('\n') or not text else '\n')
                          + '# Links into the GameFrame home (games/ID/): never commit them\n' + '\n'.join(lines) + '\n')
    print(f'linked {", ".join(LINKS)} to {game}')
    if args.move_project:
        dest = home() / 'projects' / args.id
        if dest.exists():
            raise Stop(f'{dest} exists: move the repository by hand')
        shutil.move(str(project), dest)
        project.symlink_to(dest, target_is_directory=True)
        print(f'moved the repository to {dest}, with a link at {project}')
    return 0


def check_game(game, hashes, row):
    data = load(game)
    bad = 0
    if not (game / 'game.json').exists():
        row('note', f'games/{game.name}', 'no game.json: add its media with game add')
    for m in data.get('media', []):
        f = game / m['file']
        if not f.exists():
            row('GONE', f'games/{game.name}', m['file'])
            bad += 1
        elif f.stat().st_size != m['size'] or (hashes and sha256(f) != m['sha256']):
            row('CHANGED', f'games/{game.name}', m['file'])
            bad += 1
    state = 'empty' if empty(game / 'game') else 'ready'
    row('ok' if not bad else 'BAD', f'games/{game.name}', f'game/ {state}, {len(data.get("media", []))} media files')
    return bad


def check_links(project, row):
    bad = 0
    for name in LINKS:
        p = project / name
        if p.is_symlink() and not p.exists():
            row('BROKEN', f'projects/{project.name}', f'{name} -> {os.readlink(p)}')
            bad += 1
        elif p.exists() and not p.is_symlink() and not empty(p):
            row('HERE', f'projects/{project.name}', f'{name}/ is a real folder: {NAME} home adopt')
            bad += 1
    if not bad:
        row('ok', f'projects/{project.name}', 'links in place')
    return bad
