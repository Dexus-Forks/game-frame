"""The small start scripts an installed build runs: make the data folder, go into it, and start
the build's program with its arguments and the player's own.

A program started from the Finder, the Start menu or a desktop menu starts in a folder it cannot
write to (/ on macOS). The script moves to the build's data folder first, so a program that
keeps its files in the current folder keeps them there.

In the arguments, {data} is the data folder and {resources} the folder that holds the build's
resources (Contents/Resources on macOS, the program's own folder elsewhere).
"""
import re
from pathlib import Path

PLACE = re.compile(r'\{(data|resources)\}')


def _sh_literal(text):
    return re.sub(r'([\\"$`])', r'\\\1', text)


def _sh_word(arg):
    parts, last = [], 0
    for m in PLACE.finditer(arg):
        parts.append(_sh_literal(arg[last:m.start()]))
        parts.append('${data}' if m.group(1) == 'data' else '${res}')
        last = m.end()
    parts.append(_sh_literal(arg[last:]))
    return '"' + ''.join(parts) + '"'


def _sh_path(path):
    """A path for a shell script, under "$HOME" when it is in the home folder."""
    home = str(Path.home())
    text = str(path)
    if text == home or text.startswith(home + '/'):
        return '"$HOME' + _sh_literal(text[len(home):]) + '"'
    return '"' + _sh_literal(text) + '"'


def sh_script(name, start, args, data, resources):
    """A POSIX shell script. `resources` is a shell expression relative to $here, such as
    "$here/../Resources"."""
    command = ' '.join([f'"$here/{_sh_literal(start)}"', *(_sh_word(a) for a in args), '"$@"'])
    return f'''#!/bin/sh
# {name}, installed by GameFrame (gameframe install). Starts {start} in its data folder.
here="$(cd "$(dirname "$(readlink -f "$0" 2>/dev/null || echo "$0")")" && pwd)"
res={resources}
data={_sh_path(data)}
mkdir -p "$data" && cd "$data" || exit 1
exec {command}
'''


def _cmd_word(arg):
    parts, last = [], 0
    for m in PLACE.finditer(arg):
        parts.append(arg[last:m.start()].replace('%', '%%'))
        parts.append('%data%' if m.group(1) == 'data' else '%~dp0.')
        last = m.end()
    parts.append(arg[last:].replace('%', '%%'))
    return '"' + ''.join(parts) + '"'


def cmd_script(name, start, args, data):
    """A Windows batch file, for starting the build from a terminal."""
    words = ''.join(' ' + _cmd_word(a) for a in args)
    return (f'@echo off\r\n'
            f'rem {name}, installed by GameFrame (gameframe install). Starts {start} in its data folder.\r\n'
            f'set "data={data}"\r\n'
            f'if not exist "%data%" mkdir "%data%"\r\n'
            f'cd /d "%data%"\r\n'
            f'"%~dp0{start}"{words} %*\r\n')


def resolve(args, data, resources):
    """The arguments with {data} and {resources} filled in, for a shortcut."""
    return [PLACE.sub(lambda m: str(data if m.group(1) == 'data' else resources), a) for a in args]
