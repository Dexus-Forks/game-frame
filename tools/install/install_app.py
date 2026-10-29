#!/usr/bin/env python3
"""Build a project's release programs and install them in the GameFrame folder, as an
application the player can open.

Usage:
  install_app.py [vanilla | extra] [--project DIR] [--dest DIR] [--data-root DIR]
                 [--no-build] [--no-menu] [--force] [--dry-run] [--system NAME]
  install_app.py --list [--dest DIR]
  gameframe install [...]      the same, from GameFrame or a project's ./gameframe; with no
                               arguments it installs the vanilla build

  vanilla     (the default) the game with the frame only. The frame's own switches turn the
              window off and on again (--no-frame, --frame), and the build's --vanilla switch,
              if it has one, still applies
  extra       both builds: vanilla, then the extra build, with the additions turned on (hooks,
              capture, input, renderers, mods), installed as "NAME Extra" beside it
  --project   the project with install.json (default: $GAMEFRAME_PROJECT, else this folder)
  --dest      the GameFrame folder for the programs (default: see Where)
  --data-root the GameFrame folder for the data (default: see Where)
  --no-build  install what the last build left, without running the build commands. After
              `extra` that is the extra build's programs, so build vanilla again first
  --no-menu   no Start menu shortcut or desktop menu entry (Windows, Linux)
  --force     replace a folder of the same name that this command did not install
  --dry-run   print the commands and the places, and change nothing
  --system    macos, windows or linux: the kind of install to write (default: this system's)
  --list      the builds installed in the GameFrame folder

install.json, at the project's root. Keys at the top apply to both builds; "vanilla" and
"extra" add to them, and "extra" starts from "vanilla":

  {"name": "Un-POD", "id": "dev.gameframe.unpod", "version": "0.1.0", "icon": "apps/icon.png",
   "vanilla": {"build": [["cargo", "build", "--release", "-p", "unpod", "-p", "pod-frame"]],
               "programs": ["target/release/unpod", "target/release/pod-frame"],
               "start": "pod-frame", "args": ["--data", "{data}"],
               "resources": ["apps/pod-frame/library.json"]},
   "extra":   {"build": [["cargo", "build", "--release", "-p", "unpod", "-p", "pod-frame",
                          "--features", "extras"]]}}

  name        the application's name; the extra build is "NAME Extra" unless it names itself
  id          a reverse-domain identifier (macOS); the extra build adds ".extra"
  build       commands run in the project's folder, in order, each a list of words
  programs    the files the build makes, copied in. On Windows ".exe" is added when the file
              without it does not exist
  start       which program starts, by file name
  args        its arguments before the player's own. {data} is the build's data folder and
              {resources} the folder that holds the resources
  resources   files and folders copied beside the programs (Contents/Resources on macOS)
  icon        a PNG or an .icns (macOS), copied in as the icon

Where (places.py): macOS /Applications/GameFrame/NAME.app, or ~/Applications/GameFrame when
/Applications is not writable; data in ~/Library/Application Support/GameFrame/NAME. Windows
%LOCALAPPDATA%\\Programs\\GameFrame\\NAME, data in %APPDATA%\\GameFrame\\NAME. Linux
~/.local/share/GameFrame/NAME, data in ~/.config/GameFrame/NAME. Each folder is made when it
is missing. The program starts in its data folder, so files it keeps in the current folder
land there. A new install replaces the programs and never touches the data.

Output: the commands as they run, then one line per installed build with its place, its data
folder and any notes (a missing icon, a signature or shortcut that failed).

What it does not do: cross-compile (build each system's install on that system), sign with a
Developer ID or notarise, make an installer package (.dmg, .msi, .deb), copy the game's own
files (the player adds them in the frame's library), or uninstall: delete the folder; the data
stays.
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import desktop  # noqa: E402
import macos  # noqa: E402
import places  # noqa: E402

MARKER = 'gameframe-install.json'


class Stop(Exception):
    pass


def builds(desc, which):
    """The builds to install: [(kind, settings)] with every key filled in."""
    base = {k: v for k, v in desc.items() if k not in ('vanilla', 'extra')}
    vanilla = {**base, **desc.get('vanilla', {})}
    out = [('vanilla', vanilla)]
    if which == 'extra':
        if 'extra' not in desc:
            raise Stop('install.json has no "extra" build')
        extra = {**vanilla, 'name': f'{vanilla.get("name", "")} Extra', 'id': f'{vanilla.get("id", "")}.extra'}
        out.append(('extra', {**extra, **desc['extra']}))
    for kind, b in out:
        missing = [k for k in ('name', 'programs', 'start') if not b.get(k)]
        if missing:
            raise Stop(f'the {kind} build in install.json has no {", ".join(missing)}')
    return out


def prepare(b, project, system, data_root):
    """Paths made absolute and checked, as the platform modules want them."""
    b = dict(b)
    b['slug'] = ''.join(c if c.isalnum() else '-' for c in b['name'].lower()).strip('-')
    b.setdefault('id', 'dev.gameframe.' + b['slug'])
    b['version'] = str(b.get('version', '0.0.0'))
    b.setdefault('args', [])
    b['data'] = data_root / b.get('data', b['name'])
    progs = []
    for p in b['programs']:
        path = project / p
        if system == 'windows' and not path.exists() and path.with_suffix('.exe').exists():
            path = path.with_suffix('.exe')
        progs.append(path)
    b['programs'] = progs
    b['resources'] = [project / r for r in b.get('resources', [])]
    b['icon'] = project / b['icon'] if b.get('icon') else None
    names = [p.name for p in progs]
    start = b['start']
    if start not in names and start + '.exe' in names:
        start += '.exe'
    if start not in names:
        raise Stop(f'"start" is {b["start"]}, which is not one of the programs: {", ".join(names)}')
    b['start'] = start
    return b


def check_files(b):
    gone = [str(p) for p in [*b['programs'], *b['resources'], *([b['icon']] if b['icon'] else [])]
            if not p.exists()]
    if gone:
        raise Stop('missing after the build: ' + ', '.join(gone))


def run_build(b, project, dry):
    for cmd in b.get('build', []):
        print('$ ' + ' '.join(cmd), flush=True)
        if not dry and subprocess.run(cmd, cwd=project).returncode:
            raise Stop(f'the build command failed: {" ".join(cmd)}')


def target(b, dest, system):
    return dest / (f'{b["name"]}.app' if system == 'macos' else b['name'])


def marker_dir(final, system):
    return final / 'Contents' / 'Resources' if system == 'macos' else final


def install(kind, b, project, dest, system, args):
    final = target(b, dest, system)
    if final.exists() and not (marker_dir(final, system) / MARKER).exists() and not args.force:
        raise Stop(f'{final} exists and was not installed by this command (--force replaces it)')
    notes = []
    if args.dry_run:
        return final, notes
    dest.mkdir(parents=True, exist_ok=True)
    b['data'].mkdir(parents=True, exist_ok=True)
    staging = dest / f'.{final.name}.new'
    shutil.rmtree(staging, ignore_errors=True)
    if system == 'macos':
        staging.mkdir()
        built = macos.make(b, staging, notes)
    elif system == 'windows':
        desktop.make_windows(b, staging)
        built = staging
    else:
        desktop.make_linux(b, staging, final)
        built = staging
    record = {'name': b['name'], 'build': kind, 'project': str(project), 'start': b['start'],
              'data': str(b['data']), 'installed': time.strftime('%Y-%m-%dT%H:%M:%S')}
    (marker_dir(built, system) / MARKER).write_text(json.dumps(record, indent=2) + '\n')
    if system == 'macos':
        if sys.platform == 'darwin':
            macos.sign(built, notes)
        else:
            notes.append('not signed: not on macOS')
    if final.exists():
        shutil.rmtree(final)
    built.rename(final)
    if system == 'macos':
        staging.rmdir()
    if not args.no_menu and system == places.system():
        if system == 'linux':
            desktop.menu_linux(b, final, notes)
        elif system == 'windows':
            desktop.menu_windows(b, final, notes)
    return final, notes


def list_installed(dest):
    if not dest.is_dir():
        print(f'nothing installed: {dest} does not exist')
        return 0
    found = 0
    for item in sorted(dest.iterdir()):
        for m in (item / 'Contents' / 'Resources' / MARKER, item / MARKER):
            if m.exists():
                r = json.loads(m.read_text())
                print(f'{r["build"]:8} {r["name"]:24} {item}  (installed {r["installed"]}, from {r["project"]})')
                found += 1
    if not found:
        print(f'nothing installed in {dest}')
    return 0


def main(argv=None):
    if argv is None and len(sys.argv) == 1:
        print(__doc__)
        return 0
    p = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    p.add_argument('which', nargs='?', choices=['vanilla', 'extra'], default='vanilla')
    p.add_argument('--project')
    p.add_argument('--dest')
    p.add_argument('--data-root')
    p.add_argument('--system', choices=['macos', 'windows', 'linux'], default=places.system())
    p.add_argument('--no-build', action='store_true')
    p.add_argument('--no-menu', action='store_true')
    p.add_argument('--force', action='store_true')
    p.add_argument('--dry-run', action='store_true')
    p.add_argument('--list', action='store_true')
    args = p.parse_args(argv)
    dest = Path(args.dest).expanduser().resolve() if args.dest else places.apps_folder(args.system)
    if args.list:
        return list_installed(dest)
    data_root = Path(args.data_root).expanduser().resolve() if args.data_root else places.data_folder(args.system)
    project = Path(args.project or os.environ.get('GAMEFRAME_PROJECT') or '.').resolve()
    try:
        path = project / 'install.json'
        if not path.exists():
            raise Stop(f'no install.json in {project} (install_app.py --help shows its keys)')
        desc = json.loads(path.read_text())
        todo = [(kind, prepare(b, project, args.system, data_root)) for kind, b in builds(desc, args.which)]
        done = []
        for kind, b in todo:
            if not args.no_build:
                run_build(b, project, args.dry_run)
            if not args.dry_run:
                check_files(b)
            done.append((kind, b, *install(kind, b, project, dest, args.system, args)))
    except Stop as e:
        print(f'install: {e}', file=sys.stderr)
        return 1
    for kind, b, final, notes in done:
        print(f'{"would install" if args.dry_run else "installed"} {kind}: {b["name"]} -> {final}')
        print(f'  data: {b["data"]}')
        for n in notes:
            print(f'  {n}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
