"""The GameFrame home: one folder per machine, outside every clone of GameFrame, where the games,
the projects, the machines and the tools GameFrame works with live.

Usage:
  gameframe home                    where the home is, and each of its folders
  gameframe home init [--no-python] [--live] [--llm]
                                    make the folders, write config.env from what is installed,
                                    and build the shared Python environment in python/
  gameframe home doctor [--hashes]  check the folders, config.env, python/, each game and each
                                    project's links; --hashes also checks every image's SHA-256
  gameframe home adopt PROJECT --id ID [--move-project]
                                    see games.py

The home is $GAMEFRAME_HOME, or ~/gameframe. Its folders:

  config.env    machine-wide settings: GAMEFRAME_ROOT, GHIDRA_INSTALL_DIR, JAVA_HOME,
                DOSBOX_X_BINARY, the LLM endpoints. Every script and the gameframe command read
                it first, then the project's .env, which holds only what differs
  python/       the shared Python environment for the toolkit (requirements.txt)
  tools/        tools GameFrame downloads itself, one versioned folder each
  emulators/    emulator programs and their ROM sets (86box/, dosbox-x/, roms/)
  machines/     virtual machines, one folder each; machines/os/ holds operating system images
  drivers/      reference drivers and SDKs of the period
  games/ID/     one game's files, never in any repository (games.py)
  projects/ID/  one git repository per game
  worktrees/    GameFrame worktrees made by hand
  cache/        downloads and scratch files; safe to delete
  data/         what tools keep between runs

Inputs: the environment, the files in the home. Outputs: folders, config.env, python/. It
never deletes anything. It does not install Ghidra, Java or emulators: scripts/setup.sh does,
and `home init` records where they are. Standard library only.
"""
import os
import subprocess
import sys
from pathlib import Path

from paths import NAME, ROOT, SCRIPTS

FOLDERS = {
    'python': 'the shared Python environment for the toolkit',
    'tools': 'tools GameFrame downloads itself, one versioned folder each',
    'emulators': 'emulator programs and their ROM sets',
    'machines': 'virtual machines, one folder each',
    'machines/os': 'operating system install images',
    'drivers': 'reference drivers and SDKs of the period',
    'games': "each game's files: media, game, saves, analysis, derived",
    'projects': 'one git repository per game',
    'worktrees': 'GameFrame worktrees made by hand',
    'cache': 'downloads and scratch files; safe to delete',
    'data': 'what tools keep between runs',
}
CONFIG = 'config.env'
README = """\
# GameFrame home

This folder is GameFrame's home on this machine. GameFrame itself is cloned elsewhere, at
GAMEFRAME_ROOT in config.env. Run `gameframe home` to list the folders and `gameframe home
doctor` to check them.

- `config.env`: settings for every project on this machine. A project's `.env` overrides them.
- `games/ID/`: one game's files. `media/` holds the discs and installers as they came, `game/`
  the extracted or installed game, then `saves/`, `analysis/` and `derived/`. Nothing here goes
  into git. `game.json` lists each image with its SHA-256.
- `projects/ID/`: the game's git repository. Its `game`, `media`, `saves`, `analysis` and
  `derived` are links into `games/ID/`.
- `machines/`, `emulators/`, `drivers/`, `tools/`: shared by every game.
- `cache/`: safe to delete.
"""


def home():
    """The home folder: $GAMEFRAME_HOME, or ~/gameframe."""
    return Path(os.environ.get('GAMEFRAME_HOME') or '~/gameframe').expanduser()


def read_env(path):
    """KEY=value lines of an env file, as a dict. Comments and blank lines are skipped."""
    out = {}
    try:
        lines = Path(path).read_text().splitlines()
    except OSError:
        return out
    for line in lines:
        line = line.strip()
        if line and not line.startswith('#') and '=' in line:
            key, value = line.split('=', 1)
            out[key.strip().removeprefix('export ').strip()] = value.strip().strip('"\'')
    return out


def merge_env(path, values):
    """Add the keys path does not have yet, keeping its lines. Returns the keys added."""
    have = read_env(path)
    new = {k: v for k, v in values.items() if v and k not in have}
    if new:
        text = Path(path).read_text() if Path(path).exists() else '# GameFrame settings for this machine\n'
        if not text.endswith('\n'):
            text += '\n'
        text += ''.join(f'{k}={v}\n' for k, v in new.items())
        Path(path).write_text(text)
    return list(new)


def detect():
    """What scripts/lib.sh finds on this machine."""
    script = f'. "{SCRIPTS}/lib.sh"; echo "$(find_ghidra)"; echo "$(find_java21)"; echo "$(dosbox_binary 2>/dev/null || true)"'
    out = subprocess.run(['bash', '-c', script], capture_output=True, text=True).stdout.splitlines()
    out += [''] * (3 - len(out))
    return {'GAMEFRAME_ROOT': str(ROOT), 'GHIDRA_INSTALL_DIR': out[0], 'JAVA_HOME': out[1],
            'DOSBOX_X_BINARY': out[2]}


def show(_args=None):
    h = home()
    source = 'GAMEFRAME_HOME' if os.environ.get('GAMEFRAME_HOME') else 'the default'
    print(f'home  {h}  ({source}{"" if h.is_dir() else "; missing: run gameframe home init"})')
    for name, what in FOLDERS.items():
        print(f'  {name + "/":<14}{what}')
    print(f'  {CONFIG:<14}settings for every project; a project\'s .env overrides them')
    return 0


def init(args):
    h = home()
    for name in FOLDERS:
        (h / name).mkdir(parents=True, exist_ok=True)
    if not (h / 'README.md').exists():
        (h / 'README.md').write_text(README)
    added = merge_env(h / CONFIG, detect())
    print(f'home {h}')
    print(f'  {CONFIG}: ' + (f'added {", ".join(added)}' if added else 'nothing new'))
    if not args.no_python:
        venv = h / 'python'
        py = venv / 'bin' / 'python'
        if not py.exists():
            subprocess.run([sys.executable, '-m', 'venv', str(venv)], check=True)
        wants = ['requirements.txt'] + (['requirements-live.txt'] if args.live else []) + (['requirements-llm.txt'] if args.llm else [])
        for req in wants:
            subprocess.run([str(py), '-m', 'pip', 'install', '-q', '-r', str(ROOT / req)], check=True)
        print(f'  python/: {", ".join(wants)}')
    return 0


def _row(state, name, text=''):
    print(f'  {state:<8}{name:<22}{text}')


def doctor(args):
    import games
    h = home()
    bad = 0
    if not h.is_dir():
        print(f'{h} does not exist. Run `{NAME} home init`.')
        return 1
    print(f'home {h}')
    for name in FOLDERS:
        if not (h / name).is_dir():
            _row('MISSING', name + '/', f'run {NAME} home init')
            bad += 1
    env = read_env(h / CONFIG)
    for key in ('GAMEFRAME_ROOT', 'GHIDRA_INSTALL_DIR', 'JAVA_HOME'):
        value = env.get(key)
        if not value:
            _row('MISSING', key, f'not in {CONFIG}')
            bad += 1
        elif not Path(value).expanduser().exists():
            _row('GONE', key, value)
            bad += 1
        else:
            _row('ok', key, value)
    py = h / 'python' / 'bin' / 'python'
    _row('ok' if py.exists() else 'MISSING', 'python/', '' if py.exists() else f'run {NAME} home init')
    bad += not py.exists()
    for game in sorted(p for p in (h / 'games').glob('*') if p.is_dir()):
        bad += games.check_game(game, args.hashes, _row)
    for project in sorted(p for p in (h / 'projects').glob('*') if p.is_dir()):
        bad += games.check_links(project, _row)
    print('Everything is in place.' if not bad else f'{bad} problems.')
    return 1 if bad else 0
