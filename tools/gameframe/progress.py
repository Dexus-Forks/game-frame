"""Progress logs: what each running agent has done so far, readable without its transcript.

An agent writes one line when it starts, one after each step that changes a file or takes more
than a few minutes, and one when it hands off:

    gameframe progress re-porter --add "loadconfiguration ported; 150 draws match"

The lines go to derived/progress/<agent>.log in the project (GAMEFRAME_PROJECT, else the current
folder), each with its date and time. derived/ is ignored by git. The session that sent the
agent reads them:

    gameframe progress                  every log, newest first, its last 5 lines
    gameframe progress re-porter        one log in full
    gameframe progress --lines N        the last N lines of each log

It shows what agents wrote. It does not know whether an agent is still running.
"""
import json
import os
import re
import sys
import time

NAME_RE = re.compile(r'^[A-Za-z0-9][A-Za-z0-9._-]*$')


def folder():
    return os.path.join(os.environ.get('GAMEFRAME_PROJECT') or os.getcwd(), 'derived', 'progress')


def add(name, text):
    os.makedirs(folder(), exist_ok=True)
    line = ' '.join(text.split())
    with open(os.path.join(folder(), name + '.log'), 'a', encoding='utf-8') as f:
        f.write(f'{time.strftime("%Y-%m-%d %H:%M:%S")} {line}\n')
    return 0


def ago(seconds):
    if seconds < 90:
        return f'{int(seconds)} s ago'
    if seconds < 5400:
        return f'{int(seconds // 60)} min ago'
    return f'{seconds / 3600:.1f} h ago'


def logs():
    d = folder()
    if not os.path.isdir(d):
        return []
    found = [os.path.join(d, f) for f in os.listdir(d) if f.endswith('.log')]
    return sorted(found, key=os.path.getmtime, reverse=True)


def show(args):
    if args.add is not None:
        if not args.name or not NAME_RE.match(args.name):
            print('progress --add needs an agent name first, such as re-porter', file=sys.stderr)
            return 2
        return add(args.name, args.add)
    paths = logs()
    if args.name:
        paths = [p for p in paths if os.path.basename(p) == args.name + '.log']
        if not paths:
            print(f'no progress log for {args.name} in {folder()}', file=sys.stderr)
            return 1
    out = []
    for p in paths:
        lines = open(p, encoding='utf-8').read().splitlines()
        keep = lines if args.name else lines[-args.lines:]
        out.append({'agent': os.path.basename(p)[:-4], 'updated': ago(time.time() - os.path.getmtime(p)),
                    'lines': keep})
    if args.json:
        print(json.dumps(out, indent=2))
        return 0
    if not out:
        print(f'no progress logs in {folder()}')
        return 0
    for o in out:
        print(f'{o["agent"]} (updated {o["updated"]})')
        for line in o['lines']:
            print(f'  {line}')
    return 0
