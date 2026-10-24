#!/usr/bin/env python3
"""Keep a capture of the screen, a window or a file, under a name of its own, never overwritten.

Usage: shot.py NAME [--from FILE | --window ID | --display :N] [--text TEXT | --text-file FILE]
               [--dir DIR]

  NAME          what the capture shows: s3-setstate-error, menu-first-light (a-z, 0-9, - and _)
  --from FILE   keep a capture another tool made (a VM's screenshot copied out, a /tmp file)
  --window ID   capture one window (macOS: `screencapture -l ID`)
  --display :N  capture an X display (`import -window root` from ImageMagick, or `xwd`), as
                inside a Linux VM with Xvfb
  (none)        the whole screen (macOS: screencapture; Linux: the X display in $DISPLAY)
  --text TEXT   the dialog's message, the address, the log lines: kept beside it in NAME.txt
  --text-file   the same, from a file
  --dir DIR     where captures go (default derived/captures, which git ignores)

The capture goes to DIR/YYYY-MM-DD/NAME.EXT. When that name is taken, it becomes NAME-2.EXT,
NAME-3.EXT and so on: no capture is ever overwritten. It prints the path it wrote.

Why: the dialogs a game shows on the way to running are evidence, and they are easy to lose.
A scratch capture is overwritten by the next run, and once the cause is fixed the dialog never
comes back (docs/knowhow/context-keep-every-dialog.md). Captures of the game are game material:
keep them out of git; a few go into documents with a caption.

It does not click the dialog away, read its text, or crop. Standard library only; the screen
capture itself needs screencapture (macOS) or ImageMagick's import or xwd (X11).
"""
import datetime
import os
import re
import shutil
import subprocess
import sys

NAME = re.compile(r'[A-Za-z0-9][A-Za-z0-9_-]*')


def free_path(folder, name, ext):
    """folder/name.ext, or the first free folder/name-N.ext."""
    stem, n = os.path.join(folder, name), 2
    while os.path.exists(stem + ext) or os.path.exists(stem + '.txt'):
        stem = os.path.join(folder, f'{name}-{n}')
        n += 1
    return stem + ext


def capture(dest, window=None, display=None):
    if sys.platform == 'darwin' and not display:
        cmd = ['screencapture', '-x', '-o'] + (['-l', str(window)] if window else []) + [dest]
    else:
        env_display = display or os.environ.get('DISPLAY')
        if not env_display:
            sys.exit('shot.py: no display: give --display :N, --from FILE, or set DISPLAY')
        if shutil.which('import'):
            cmd = ['import', '-display', env_display, '-window', str(window or 'root'), dest]
        elif shutil.which('xwd'):
            dest_xwd = dest[:-4] + '.xwd'
            cmd = ['xwd', '-display', env_display, '-root' if not window else '-id', *([str(window)] if window else []),
                   '-out', dest_xwd]
            dest = dest_xwd
        else:
            sys.exit('shot.py: install ImageMagick (import) or xwd to capture an X display')
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode or not os.path.exists(dest):
        sys.exit(f'shot.py: capture failed: {" ".join(cmd)}\n{r.stderr}')
    return dest


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__.rstrip())
        return 0 if argv else 2
    opt = lambda k: argv[argv.index(k) + 1] if k in argv and argv.index(k) + 1 < len(argv) else None
    name = argv[0]
    if not NAME.fullmatch(name):
        sys.exit(f'shot.py: name {name!r}: use letters, digits, - and _ (say what it shows)')
    folder = os.path.join(opt('--dir') or os.path.join('derived', 'captures'), datetime.date.today().isoformat())
    os.makedirs(folder, exist_ok=True)
    src = opt('--from')
    ext = os.path.splitext(src)[1].lower() if src else '.png'
    dest = free_path(folder, name, ext or '.png')
    if src:
        if not os.path.isfile(src):
            sys.exit(f'shot.py: no file {src}')
        shutil.copy2(src, dest)
    else:
        dest = capture(dest, opt('--window'), opt('--display'))
    text = opt('--text')
    if opt('--text-file'):
        text = open(opt('--text-file'), encoding='utf-8', errors='replace').read()
    if text:
        with open(os.path.splitext(dest)[0] + '.txt', 'x', encoding='utf-8') as f:
            f.write(text.rstrip('\n') + '\n')
    print(dest)
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
