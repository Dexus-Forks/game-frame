#!/usr/bin/env python3
"""Find which unexplained fields next to geometry are computed from it, such as bounds, centres,
normals and plane distances. An editor that moves vertices must write these fields again.

Usage: derived.py RECORDS.jsonl [--field-scale S] [--vertex-scale S] [--tol T] [--json]

  RECORDS.jsonl  one record a line, written by your decoder:
                   {"vertices": [[x, y, z], ...], "fields": [f0, f1, ...]}
                 a record is one object with its vertices (a sector, a mesh: try bounds), or
                 one face with its corners in order (try normals and planes). The fields are
                 the record's unexplained numbers, in stored order
  --field-scale  divide every field by S first: 65536 for 16.16 fixed point (default 1)
  --vertex-scale divide every coordinate by S first (default 1)
  --tol T        a field matches a candidate within T (default 0.001 after scaling)

For every field it tries every candidate on every record. It also tries each negated candidate
(a normal stored pointing the other way). It prints the candidate that matches the most records,
with its share. For a field that no candidate explains, it prints "nothing explains it".

Candidates, from each record's vertices:

  min_x min_y min_z max_x max_y max_z   the record's bounds (a box, as sectors often store)
  mid_x mid_y mid_z                     the centre of the vertices
  newell_x newell_y newell_z            the unit normal by Newell's method over all corners
  cross_x cross_y cross_z               the unit normal of the first two edges
  plane_newell plane_cross              the plane's distance: -(n . v0)

Why: level files often store data computed from the vertices. When an editor moves vertices
and does not write this data again, culling, lighting or physics use the old shape. In one
1997 game the six words after each sector were its bounds on every record. Newell's normals
matched 97.9 to 99.6 % of faces. The cross product of the first edges matched 74 % (the faces
it missed were quads that are not flat) (docs/knowhow/context-geometry-derived-data.md).

It does not decode files. Write the records from your decoder. Candidates are in the same
units as the vertices. A field stored in another unit matches only after the right scale.
Standard library only.
"""
import json
import math
import sys

AXES = 'xyz'


def unit(v):
    n = math.sqrt(sum(c * c for c in v))
    return [c / n for c in v] if n else [0.0, 0.0, 0.0]


def newell(vs):
    n = [0.0, 0.0, 0.0]
    for i, a in enumerate(vs):
        b = vs[(i + 1) % len(vs)]
        n[0] += (a[1] - b[1]) * (a[2] + b[2])
        n[1] += (a[2] - b[2]) * (a[0] + b[0])
        n[2] += (a[0] - b[0]) * (a[1] + b[1])
    return unit(n)


def cross(vs):
    if len(vs) < 3:
        return [0.0, 0.0, 0.0]
    a, b, c = vs[0], vs[1], vs[2]
    u = [b[i] - a[i] for i in range(3)]
    w = [c[i] - b[i] for i in range(3)]
    return unit([u[1] * w[2] - u[2] * w[1], u[2] * w[0] - u[0] * w[2], u[0] * w[1] - u[1] * w[0]])


def candidates(vs):
    out = {}
    for i, ax in enumerate(AXES):
        col = [v[i] for v in vs]
        out[f'min_{ax}'], out[f'max_{ax}'] = min(col), max(col)
        out[f'mid_{ax}'] = sum(col) / len(col)
    for name, n in (('newell', newell(vs)), ('cross', cross(vs))):
        for i, ax in enumerate(AXES):
            out[f'{name}_{ax}'] = n[i]
        out[f'plane_{name}'] = -sum(n[i] * vs[0][i] for i in range(3))
    return out


def find(records, fscale=1.0, vscale=1.0, tol=1e-3):
    """[(field index, best candidate or None, share)] over all records."""
    width = max(len(r['fields']) for r in records)
    hits = [dict() for _ in range(width)]
    counts = [0] * width
    for r in records:
        vs = [[c / vscale for c in v] for v in r['vertices']]
        if not vs:
            continue
        cand = candidates(vs)
        for i, f in enumerate(r['fields']):
            f /= fscale
            counts[i] += 1
            for name, value in cand.items():
                if abs(f - value) <= tol:
                    hits[i][name] = hits[i].get(name, 0) + 1
                elif value and abs(f + value) <= tol:
                    hits[i]['-' + name] = hits[i].get('-' + name, 0) + 1
    out = []
    for i in range(width):
        if not hits[i] or not counts[i]:
            out.append((i, None, 0.0))
            continue
        best = max(hits[i], key=lambda k: (hits[i][k], not k.startswith('-')))
        out.append((i, best, hits[i][best] / counts[i]))
    return out


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__.rstrip())
        return 0 if argv else 2
    opt = lambda k, d: float(argv[argv.index(k) + 1]) if k in argv else d
    records = [json.loads(l) for l in open(argv[0], encoding='utf-8') if l.strip()]
    if not records:
        raise SystemExit('derived: no records')
    rows = find(records, opt('--field-scale', 1.0), opt('--vertex-scale', 1.0), opt('--tol', 1e-3))
    if '--json' in argv:
        print(json.dumps([{'field': i, 'candidate': c, 'share': s} for i, c, s in rows], indent=2))
        return 0
    print(f'{len(records)} records')
    for i, c, s in rows:
        print(f'  field {i:<3} ' + (f'{c:<14} {100 * s:6.1f} %' if c and s >= 0.05 else 'nothing explains it'))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
