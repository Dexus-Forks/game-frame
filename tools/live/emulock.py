#!/usr/bin/env python3
"""Say who is using a shared emulator or VM, since when and for what, so two sessions do not
drive one game at once.

Usage: emulock.py take NAME --who WHO --why TEXT [--dir DIR] [--force]
       emulock.py free NAME --who WHO [--dir DIR]
       emulock.py show [NAME] [--dir DIR]

  NAME     the emulator or VM: gf-vm, dosbox, the-windows-pc
  --who    the session or person: a session's name, an agent's name, a person's name
  --why    what for: "recording beltane-3", "testing a grip mod"
  --dir    where the locks live (default derived/locks). For a VM several machines use, put
           them inside the VM (run this there over SSH), so every session sees the same ones
  --force  take a lock another holder has, after asking them; the old note is printed

take writes DIR/NAME.lock with the holder, the time and the reason, and fails with exit 1,
naming the holder, when someone else holds it. Taking a lock you already hold renews it. free
removes your own lock (exit 1 when another holds it). show prints every lock, or one.

Why: two sessions in one emulated PC restarted each other's game, and a reference recording
was spoiled by another session's mods (docs/knowhow/mods-loaded-in-a-reference-recording.md,
docs/knowhow/context-emulated-pc.md). A lock note turns that into a question asked first.

It is a note between honest users, not a guard: it does not stop a process, and a script that
never calls take is not held back. Writes are atomic (a new file renamed into place).
Standard library only.
"""
import datetime
import json
import os
import sys


def path(d, name):
    if not name or '/' in name or name.startswith('.'):
        raise SystemExit(f'emulock: bad name {name!r}')
    return os.path.join(d, name + '.lock')


def read(p):
    try:
        with open(p, encoding='utf-8') as f:
            return json.load(f)
    except (OSError, ValueError):
        return None


def describe(name, lock):
    return f'{name}: held by {lock["who"]} since {lock["since"]} for "{lock["why"]}"'


def main(argv):
    if not argv or argv[0] in ('-h', '--help') or argv[0] not in ('take', 'free', 'show'):
        print(__doc__.rstrip())
        return 0 if argv and argv[0] in ('-h', '--help') else 2
    opt = lambda k: argv[argv.index(k) + 1] if k in argv and argv.index(k) + 1 < len(argv) else None
    d = opt('--dir') or os.path.join('derived', 'locks')
    taken = {i + 1 for i, a in enumerate(argv) if a in ('--who', '--why', '--dir')}
    args = [a for i, a in enumerate(argv[1:], 1) if not a.startswith('--') and i not in taken]
    cmd = argv[0]
    if cmd == 'show':
        names = args or sorted(f[:-5] for f in os.listdir(d) if f.endswith('.lock')) if os.path.isdir(d) else args
        held = [(n, read(path(d, n))) for n in names]
        for n, lock in held:
            print(describe(n, lock) if lock else f'{n}: free')
        if not held:
            print('no locks')
        return 0
    if not args or not opt('--who'):
        raise SystemExit(f'usage: emulock.py {cmd} NAME --who WHO' + (' --why TEXT' if cmd == 'take' else ''))
    p, who = path(d, args[0]), opt('--who')
    lock = read(p)
    if cmd == 'free':
        if lock and lock['who'] != who:
            print(describe(args[0], lock), file=sys.stderr)
            return 1
        if lock:
            os.remove(p)
        print(f'{args[0]}: free')
        return 0
    if lock and lock['who'] != who and '--force' not in argv:
        print(describe(args[0], lock) + '. Ask them, or use --force.', file=sys.stderr)
        return 1
    if lock and lock['who'] != who:
        print('taking over from: ' + describe(args[0], lock))
    os.makedirs(d, exist_ok=True)
    new = {'who': who, 'why': opt('--why') or '', 'since': datetime.datetime.now().isoformat(timespec='seconds'),
           'host': os.uname().nodename if hasattr(os, 'uname') else ''}
    tmp = p + f'.{os.getpid()}'
    with open(tmp, 'w', encoding='utf-8') as f:
        json.dump(new, f)
    if lock is None and os.path.exists(p) and '--force' not in argv:
        os.remove(tmp)                       # someone took it between our read and our write
        print(describe(args[0], read(p)), file=sys.stderr)
        return 1
    os.replace(tmp, p)
    print(describe(args[0], new))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
