#!/usr/bin/env python3
"""Read a recording made by memrec.py: a summary, a CSV table, or one sample by time.

Usage: memlog.py summary FILE
       memlog.py csv FILE [--fields NAME,NAME] [--fixed NAME,NAME] [--out FILE]
       memlog.py at FILE SECONDS

  FILE      a JSON Lines recording: one sample a line, {"t": seconds, "n": sample, NAME: value}
            (tools/live/memrec.py record)
  summary   the samples, the time they span and the rate, then each value: its type, the
            smallest and largest value, and how many times it changed
  csv       a table with one column per value; an array NAME becomes NAME[0], NAME[1] ...
  --fields  only these values (default: all, in the recording's order)
  --fixed   print these values as 16.16 fixed point (value / 65536, six decimals); the
            recording itself keeps the raw integers
  at        the sample nearest to SECONDS, as JSON

Why: a recording is the reference run a port is checked against
(docs/knowhow/record-a-run-from-memory.md). Spreadsheets and plots want CSV; a quick look
wants the ranges, and a stuck or never-changing value shows at once in the summary.

It does not print values the game's own way (a game's time format, say): write that in the
project, beside the port, and test it against the game's screens. Standard library only.
"""
import csv
import json
import sys


def samples(path):
    with open(path, encoding='utf-8') as f:
        for n, line in enumerate(f, 1):
            line = line.strip()
            if line:
                try:
                    yield json.loads(line)
                except ValueError:
                    raise SystemExit(f'memlog: {path}:{n} is not JSON')


def columns(rows, fields=None):
    names = [k for k in rows[0] if k not in ('t', 'n')] if rows else []
    if fields:
        missing = [f for f in fields if f not in names]
        if missing:
            raise SystemExit(f'memlog: no value named {", ".join(missing)} (have {", ".join(names)})')
        names = fields
    cols = []
    for k in names:
        v = rows[0][k]
        cols += [(k, i) for i in range(len(v))] if isinstance(v, list) else [(k, None)]
    return cols


def cell(row, k, i, fixed):
    v = row.get(k)
    v = v[i] if i is not None and isinstance(v, list) else v
    return f'{v / 65536:.6f}' if k in fixed and isinstance(v, int) else v


def summary(rows):
    if not rows:
        print('0 samples')
        return 1
    span = rows[-1]['t'] - rows[0]['t']
    rate = (len(rows) - 1) / span if span > 0 else 0
    print(f'{len(rows)} samples over {span:.2f} s ({rate:.1f} a second)')
    for k, i in columns(rows):
        vals = [cell(r, k, i, ()) for r in rows]
        changes = sum(1 for a, b in zip(vals, vals[1:]) if a != b)
        name = f'{k}[{i}]' if i is not None else k
        kind = type(vals[0]).__name__
        print(f'  {name:<20} {kind:<6} {min(vals)} .. {max(vals)}   {changes} changes')
    return 0


def main(argv):
    if len(argv) < 2 or argv[0] in ('-h', '--help') or argv[0] not in ('summary', 'csv', 'at'):
        print(__doc__.rstrip())
        return 0 if argv and argv[0] in ('-h', '--help') else 2
    opt = lambda k: argv[argv.index(k) + 1] if k in argv and argv.index(k) + 1 < len(argv) else None
    rows = list(samples(argv[1]))
    if argv[0] == 'summary':
        return summary(rows)
    if argv[0] == 'at':
        if not rows:
            return 1
        t = float(argv[2])
        print(json.dumps(min(rows, key=lambda r: abs(r['t'] - t))))
        return 0
    fields = opt('--fields').split(',') if opt('--fields') else None
    fixed = set(opt('--fixed').split(',')) if opt('--fixed') else set()
    cols = columns(rows, fields)
    out = open(opt('--out'), 'w', newline='', encoding='utf-8') if opt('--out') else sys.stdout
    w = csv.writer(out, lineterminator='\n')
    w.writerow(['t', 'n'] + [f'{k}[{i}]' if i is not None else k for k, i in cols])
    for r in rows:
        w.writerow([r['t'], r['n']] + [cell(r, k, i, fixed) for k, i in cols])
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
