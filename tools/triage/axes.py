#!/usr/bin/env python3
"""Find which axis is "up" in a 3D game's coordinates, its sign, and whether they are mirrored.

Usage: axes.py RECORDING GEOMETRY --pos X,Y,Z [--index I] [--scale S] [--left-turn T0:T1]
               [--cell SIZE] [--json]

  RECORDING    JSON Lines of an object that moves on the level's surface (a car, a walker), as
               tools/live/memrec.py record writes: {"t": seconds, NAME: value or [values], ...}
  GEOMETRY     the level's vertices, one point a line, "x y z" or "x,y,z" (# comments), in the
               same units as the positions once --scale is applied
  --pos        the three recorded values that hold the position, in the game's own order
  --index I    for array values (eight cars, say): which element (default 0)
  --scale S    multiply the recorded values by S first (1/65536 for 16.16 fixed point)
  --left-turn  a time span, in the recording's t, in which the object is known to turn left (the
               player steered left, the track bends left): gives the handedness
  --cell SIZE  the size of the grid used to find the surface under a point (default: 1/50 of
               the level's largest extent)

For each axis and each sign it takes that axis as "up", finds the vertices under every sample
in the other two axes (the grid cell and its neighbours), and measures how far above the
highest one the sample sits. On the right axis the object rides the surface: the height is
small and nearly the same in every sample. On a wrong axis the "surface under it" is any part
of the level, and the heights scatter. It prints the six candidates, best first, and the one
it picks. With --left-turn it then looks down the up axis and takes the sign of the turn: a
left turn is anticlockwise seen from above in right-handed coordinates (as OpenGL's), and
clockwise in left-handed ones (as Direct3D's). A port or viewer that draws the level mirrored
has the handedness wrong.

Output: the table and the verdict, or JSON with --json. Exit status 1 when no axis gives a
consistent height (fewer than half the samples above the surface, or a spread larger than a
tenth of the level).

It does not read the level's file format: export the vertices with the project's own reader.
It does not tell a yaw from a pitch, or find the order of the two flat axes beyond the
handedness. Standard library only.
"""
import json
import math
import statistics
import sys
from collections import defaultdict

NAMES = 'xyz'


def load_points(path):
    pts = []
    for line in open(path, encoding='utf-8'):
        line = line.split('#')[0].replace(',', ' ').split()
        if len(line) >= 3:
            pts.append(tuple(float(v) for v in line[:3]))
    return pts


def load_path(path, names, index, scale):
    out = []
    for line in open(path, encoding='utf-8'):
        if not line.strip():
            continue
        s = json.loads(line)
        if any(n not in s for n in names):
            continue
        v = [s[n][index] if isinstance(s[n], list) else s[n] for n in names]
        out.append((s.get('t', len(out)), tuple(float(x) * scale for x in v)))
    return out


def candidate(points, path, up, sign, cell):
    """Take axis `up` with `sign` as up: the object's height over the surface in every sample."""
    flat = [a for a in range(3) if a != up]
    grid = defaultdict(list)
    for p in points:
        grid[(math.floor(p[flat[0]] / cell), math.floor(p[flat[1]] / cell))].append(p)
    heights = []
    for _, q in path:
        cx, cy = math.floor(q[flat[0]] / cell), math.floor(q[flat[1]] / cell)
        near = [(math.hypot(p[flat[0]] - q[flat[0]], p[flat[1]] - q[flat[1]]), sign * p[up])
                for dx in (-1, 0, 1) for dy in (-1, 0, 1) for p in grid.get((cx + dx, cy + dy), ())]
        if not near:
            continue
        near.sort()
        near = near[:4]
        if near[0][0] < 1e-9:
            ground = near[0][1]
        else:   # the surface under the point: its nearest vertices, weighted by closeness
            w = [1 / d for d, _ in near]
            ground = sum(wi * h for wi, (_, h) in zip(w, near)) / sum(w)
        heights.append(sign * q[up] - ground)
    spread = statistics.pstdev(heights) if len(heights) > 1 else float('inf')
    above = [h for h in heights if h >= -spread]
    return {'up': ('+' if sign > 0 else '-') + NAMES[up], 'axis': up, 'sign': sign,
            'samples': len(path), 'over_surface': len(above),
            'height': statistics.median(heights) if heights else None, 'spread': spread}


def handedness(path, up, sign, t0, t1):
    """+1 when a left turn is anticlockwise seen from above (right-handed), -1 when clockwise."""
    flat = [a for a in range(3) if a != up]
    seg = [q for t, q in path if t0 <= t <= t1]
    turn = 0.0
    for p0, p1, p2 in zip(seg, seg[1:], seg[2:]):
        ax, ay = p1[flat[0]] - p0[flat[0]], p1[flat[1]] - p0[flat[1]]
        bx, by = p2[flat[0]] - p1[flat[0]], p2[flat[1]] - p1[flat[1]]
        turn += ax * by - ay * bx
    # The flat axes in cyclic order after "up" (x->y->z->x) make a right-handed frame with up.
    cyclic = (flat[0] - up) % 3 == 1
    seen_from_above = turn * sign * (1 if cyclic else -1)
    return (1 if seen_from_above > 0 else -1) if turn else 0, len(seg)


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__.strip())
        return 0 if argv else 2
    opts = {'--pos': None, '--index': '0', '--scale': '1', '--left-turn': None, '--cell': None}
    pos, i, as_json = [], 0, False
    while i < len(argv):
        if argv[i] == '--json':
            as_json = True
        elif argv[i] in opts and i + 1 < len(argv):
            opts[argv[i]] = argv[i + 1]
            i += 1
        else:
            pos.append(argv[i])
        i += 1
    if len(pos) != 2 or not opts['--pos']:
        print(__doc__.strip())
        return 2
    num, _, den = opts['--scale'].partition('/')
    scale = float(num) / float(den or 1)
    points = load_points(pos[1])
    path = load_path(pos[0], opts['--pos'].split(','), int(opts['--index']), scale)
    if not points or len(path) < 3:
        print('need the level\'s vertices and at least three positions', file=sys.stderr)
        return 2
    extent = max(max(p[a] for p in points) - min(p[a] for p in points) for a in range(3))
    cell = float(opts['--cell']) if opts['--cell'] else extent / 50
    cands = [candidate(points, path, a, s, cell) for a in range(3) for s in (1, -1)]
    good = lambda c: c['over_surface'] * 2 >= c['samples'] and c['spread'] < extent / 10
    cands.sort(key=lambda c: (not good(c), c['spread']))
    best = cands[0]
    result = {'candidates': cands, 'up': best['up'] if good(best) else None}
    if good(best) and opts['--left-turn']:
        t0, t1 = (float(v) for v in opts['--left-turn'].split(':'))
        h, n = handedness(path, best['axis'], best['sign'], t0, t1)
        result['handedness'] = {1: 'right-handed', -1: 'left-handed', 0: None}[h]
        result['turn_samples'] = n
    if as_json:
        print(json.dumps(result, indent=1))
    else:
        print('up   over surface   height   spread')
        for c in cands:
            h = f'{c["height"]:.3f}' if c['height'] is not None else '-'
            print(f'{c["up"]:>3}   {c["over_surface"]:5} of {c["samples"]:<5} {h:>7}   {c["spread"]:.3f}')
        if result['up']:
            print(f'up is {result["up"]}: the object rides {best["height"]:.3f} above the surface '
                  f'(spread {best["spread"]:.3f})')
            if 'handedness' in result:
                print(f'a left turn seen from above: {result["handedness"]} coordinates '
                      f'({result["turn_samples"]} samples in the turn)')
        else:
            print('no axis gives a steady height over the surface: check --pos, --scale and the geometry')
    return 0 if result['up'] else 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
