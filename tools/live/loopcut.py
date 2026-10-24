#!/usr/bin/env python3
"""Cut a recorded path into one closed loop: a racing line, a patrol, a lap of a level.

Usage: loopcut.py FILE --x NAME --y NAME [--z NAME] [--index I] [--gap D] [--out FILE]

  FILE     a memrec.py recording (JSON Lines), longer than one loop
  --x/--y  the values that hold the position (--z for a third axis); an array value takes
           --index I (default 0), such as the player's car in an array of eight
  --gap D  how close the path must come back to its first point to close (default: 2% of the
           path's largest extent)
  --out    write the kept samples as JSON Lines (default: print only the summary)

It walks the path from its first sample, waits until the path has gone well away from the
start (more than four gaps), and cuts at the first sample that comes back within the gap. It
prints how many samples it kept, the loop's length, and the distance between its ends. It
exits 1 when the path never comes back: the recording is shorter than a loop.

Why: a line-following driver that aims along a recorded path wedges at the path's end when
the path is less than a loop, because the next aim point is the start, far across the level
(docs/knowhow/racing-line-must-be-a-closed-lap.md). Record more than a loop and cut it here.

It does not smooth the path, resample it, or find a loop that starts elsewhere than the first
sample. Standard library only.
"""
import json
import math
import sys


def point(row, names, index):
    out = []
    for k in names:
        v = row[k]
        out.append(v[index] if isinstance(v, list) else v)
    return out


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__.rstrip())
        return 0 if argv else 2
    opt = lambda k: argv[argv.index(k) + 1] if k in argv and argv.index(k) + 1 < len(argv) else None
    if not opt('--x') or not opt('--y'):
        raise SystemExit('loopcut: give --x NAME and --y NAME')
    names = [opt('--x'), opt('--y')] + ([opt('--z')] if opt('--z') else [])
    index = int(opt('--index') or 0)
    rows = [json.loads(l) for l in open(argv[0], encoding='utf-8') if l.strip()]
    pts = [point(r, names, index) for r in rows]
    if len(pts) < 3:
        raise SystemExit('loopcut: too few samples')
    extent = max(max(p[a] for p in pts) - min(p[a] for p in pts) for a in range(len(names)))
    gap = float(opt('--gap')) if opt('--gap') else 0.02 * extent
    start, left, cut = pts[0], False, None
    for i, p in enumerate(pts[1:], 1):
        d = math.dist(p, start)
        if d > 4 * gap:
            left = True
        elif left and d <= gap:
            cut = i
            break
    if cut is None:
        print(f'loopcut: the path never comes back within {gap:g} of its start: record more than a loop',
              file=sys.stderr)
        return 1
    length = sum(math.dist(a, b) for a, b in zip(pts[:cut], pts[1:cut + 1]))
    print(f'kept {cut} of {len(rows)} samples: a loop of length {length:.1f}, '
          f'ends {math.dist(pts[cut], start):.2f} apart (gap {gap:g})')
    if opt('--out'):
        with open(opt('--out'), 'w', encoding='utf-8') as f:
            f.writelines(json.dumps(r) + '\n' for r in rows[:cut])
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
