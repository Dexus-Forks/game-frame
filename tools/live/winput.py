#!/usr/bin/env python3
"""Build and run winput, which sends keys, mouse moves and clicks to a game from inside Wine.

Usage: winput.py build [--out FILE]             build winput.exe (32-bit Windows) with zig cc
       winput.py host [--out FILE]              build a --dry-run copy for this machine, for tests
       winput.py run [--wine WINE] [--prefix DIR] [--exe FILE] [--dry-run] STEP ...
       winput.py keys                           the key names winput knows

STEP is one of winput's steps, in order (tools/live/winput.c has them all):

  list | window TITLE | key NAME[:MS] | down NAME | up NAME | release | type TEXT |
  move DX DY | click X Y [right] | point X Y | button [right] | wait MS

  --hold MS before the steps sets how long `key` holds a key (default 50; 500 for a game at a
  low frame rate). Keys that `down` holds are released when winput ends. For a game that draws
  its own cursor, `point X Y wait 1000 button` clicks where `click` may miss.

  winput.py run --prefix derived/wine window "POD" wait 500 key enter key down:200 type abc

Why: macOS ignores input from a process without the Accessibility permission. Tools that drive
the desktop cannot see Wine's windows. winput runs inside the same Wine prefix as the game and
calls SendInput. Keys go as scancodes, which DirectInput reads. See
docs/knowhow/wine-input-without-accessibility.md.

Inputs: winput.c next to this file. `build` writes winput.exe (default: winput.exe in the current
folder). It needs zig. `run` builds winput.exe first when --exe is missing. Then it runs WINE
(default $WINE, else `wine` on PATH), with WINEPREFIX set to --prefix when given. The exit code
is winput's. It is 1 when a window is not found or an input was not sent, and 2 for a bad step.

It does not read the screen or record input. A game that draws its own cursor from DirectInput's
relative movement ignores `click` positions. Use `move` for such a game. Standard library only.
"""
import os
import re
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
SOURCE = os.path.join(HERE, 'winput.c')


def option(argv, name, default=None):
    if name in argv:
        i = argv.index(name)
        if i + 1 >= len(argv):
            sys.exit(f'winput.py: {name} needs a value')
        value = argv[i + 1]
        del argv[i:i + 2]
        return value
    return default


def build(out, target='x86-windows-gnu'):
    zig = shutil.which('zig')
    if not zig:
        sys.exit('winput.py: zig not found (scripts/setup.sh --install installs it)')
    cmd = [zig, 'cc', '-O2', SOURCE, '-o', out]
    if target:
        cmd[2:2] = ['-target', target]
        cmd.append('-luser32')
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode:
        sys.exit(f'winput.py: build failed:\n{r.stderr}')
    print(out)
    return 0


def keys():
    names = re.findall(r'\{"(\w+)", 0x', open(SOURCE, encoding='utf-8').read())
    print(' '.join(names))
    return 0


def run(argv):
    wine = option(argv, '--wine', os.environ.get('WINE') or shutil.which('wine') or 'wine')
    prefix = option(argv, '--prefix')
    exe = option(argv, '--exe', 'winput.exe')
    if not os.path.exists(exe):
        build(exe)
    env = dict(os.environ)
    if prefix:
        env['WINEPREFIX'] = os.path.abspath(prefix)
    return subprocess.call([wine, exe, *argv], env=env)


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__.rstrip())
        return 0 if argv else 2
    cmd, rest = argv[0], argv[1:]
    if cmd == 'build':
        return build(option(rest, '--out', 'winput.exe'))
    if cmd == 'host':
        return build(option(rest, '--out', 'winput-host'), target=None)
    if cmd == 'run':
        return run(rest)
    if cmd == 'keys':
        return keys()
    print(f'winput.py: unknown command {cmd!r}; see --help', file=sys.stderr)
    return 2


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
