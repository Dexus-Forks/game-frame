#!/usr/bin/env python3
"""Line up a reference picture of a screen with a capture of the screen now: offset and scale.

Usage:
  align.py find REF NOW [--point X,Y ...] [--scales MIN:MAX] [--stretch] [--save FILE.json]
                        [--check OUT.png] [--units F]
  align.py map FILE.json X Y [--inverse] [--units F]

  REF       the screen as you know it: a frame from the port, an earlier capture, a screenshot
            whose button positions you have (PNG)
  NOW       a capture of what is shown now: the VM's window with its frame, a stream, the
            whole desktop (PNG)
  --point   a point in REF (a button from the port's probe) to give in NOW's pixels; repeat it
  --scales  the range of scales to try, REF to NOW (default 0.25:4)
  --stretch let the vertical scale differ from the horizontal one (a stretched stream)
  --save    write the result as JSON, for `map` and for the next click
  --check   write NOW with REF's outline and each point drawn on it, to look at before clicking
  --units   divide the points given out by F: 2 turns a Retina capture's pixels into the
            points that macOS input tools use
  map       give a point of REF in NOW (or, with --inverse, a point of NOW in REF) from a saved
            result

The result is `now = offset + ref * scale`, for x and y. It is found in three steps: every
scale and position on a small copy of both pictures, then the best three candidates on a
larger copy, then the best one on samples of the full pictures. The score is the normalised
correlation of the two pictures where they overlap, from -1 to 1. Below 0.5 the match is weak,
and the tool says so.

Why: a click point taken from the port, or from an earlier capture, misses when the window
has a frame, a title bar, a border or a different size. The first click lands on another
button. Align once, then map every point.

What it does not do: rotation, perspective, or a screen whose content has changed a lot since
REF (another menu, a moved window inside the game). When the pictures differ too much, ask a
model to find the element: tools/llm/locate.py. Standard library only; PNG through
tools/formats/pngio.py (8-bit, not interlaced).
"""
import json
import math
import os
import sys
from operator import mul

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'formats'))
from pngio import grey, read_png, write_png  # noqa: E402

COARSE, MEDIUM, SAMPLES = 64, 256, 4000


def integral(g, w, h):
    """Summed-area table with a zero first row and column: (w + 1) * (h + 1) values."""
    s = [0] * ((w + 1) * (h + 1))
    for y in range(h):
        row, acc, base = y * w, 0, (y + 1) * (w + 1)
        for x in range(w):
            acc += g[row + x]
            s[base + x + 1] = s[base - (w + 1) + x + 1] + acc
    return s


def resize(g, w, h, nw, nh):
    """Area average for shrinking, nearest pixel for growing: one list of rows."""
    s = integral(g, w, h)
    out = []
    for y in range(nh):
        y0 = y * h // nh
        y1 = max(y0 + 1, (y + 1) * h // nh)
        row = []
        for x in range(nw):
            x0 = x * w // nw
            x1 = max(x0 + 1, (x + 1) * w // nw)
            total = s[y1 * (w + 1) + x1] - s[y0 * (w + 1) + x1] - s[y1 * (w + 1) + x0] + s[y0 * (w + 1) + x0]
            row.append(total / ((x1 - x0) * (y1 - y0)))
        out.append(row)
    return out


def ncc_all(rows, t, positions=None):
    """Best (score, x, y) of template rows t over image rows, at every position or the given ones."""
    th, tw = len(t), len(t[0])
    n = tw * th
    mean = sum(map(sum, t)) / n
    tz = [[v - mean for v in r] for r in t]
    tn = math.sqrt(sum(v * v for r in tz for v in r))
    if tn < 1e-6:
        return None
    h, w = len(rows), len(rows[0])
    sq = [[v * v for v in r] for r in rows]
    best = None
    if positions is None:
        positions = ((x, y) for y in range(h - th + 1) for x in range(w - tw + 1))
    for x, y in positions:
        if x < 0 or y < 0 or x + tw > w or y + th > h:
            continue
        s = s2 = num = 0.0
        for r in range(th):
            seg = rows[y + r][x:x + tw]
            s += sum(seg)
            s2 += sum(sq[y + r][x:x + tw])
            num += sum(map(mul, tz[r], seg))
        var = s2 - s * s / n
        if var <= 1e-6:
            continue
        score = num / (tn * math.sqrt(var))
        if best is None or score > best[0]:
            best = (score, x, y)
    return best


def sampled(ref, rw, rh, now, nw, nh, sx, sy, dx, dy, pts):
    """Correlation of REF's sample points with NOW under one transform."""
    a, b = [], []
    for u, v in pts:
        x, y = int(dx + u * sx + 0.5), int(dy + v * sy + 0.5)
        if not (0 <= x < nw and 0 <= y < nh):
            return -2.0
        a.append(ref[v * rw + u])
        b.append(now[y * nw + x])
    n = len(a)
    ma, mb = sum(a) / n, sum(b) / n
    num = sum((p - ma) * (q - mb) for p, q in zip(a, b))
    da = math.sqrt(sum((p - ma) ** 2 for p in a))
    db = math.sqrt(sum((q - mb) ** 2 for q in b))
    return num / (da * db) if da > 1e-6 and db > 1e-6 else -2.0


def find(ref_path, now_path, smin=0.25, smax=4.0, stretch=False):
    rw, rh, rrgb = read_png(ref_path)
    nw, nh, nrgb = read_png(now_path)
    ref, now = grey(rw, rh, rrgb), grey(nw, nh, nrgb)
    # 1. Every scale and position, small.
    kc = min(1.0, COARSE / max(nw, nh))
    cw, ch = max(1, round(nw * kc)), max(1, round(nh * kc))
    crows = resize(now, nw, nh, cw, ch)
    cands, s = [], smin
    while s <= smax:
        tw, th = round(rw * s * kc), round(rh * s * kc)
        if 6 <= tw <= cw and 6 <= th <= ch:
            hit = ncc_all(crows, resize(ref, rw, rh, tw, th))
            if hit:
                cands.append((hit[0], s, hit[1] / kc, hit[2] / kc))
        s *= 1.04
    if not cands:
        sys.exit('no scale fits: REF is larger than NOW at every scale tried (see --scales)')
    cands.sort(reverse=True)
    # 2. The best three, larger.
    km = min(1.0, MEDIUM / max(nw, nh))
    mw, mh = max(1, round(nw * km)), max(1, round(nh * km))
    mrows = resize(now, nw, nh, mw, mh)
    reach = int(math.ceil(km / kc)) + 2
    best = None
    for _, s, x, y in cands[:3]:
        for d in range(-4, 5):
            s2 = s * (1 + d / 100)
            tw, th = round(rw * s2 * km), round(rh * s2 * km)
            if tw < 6 or th < 6 or tw > mw or th > mh:
                continue
            cx, cy = round(x * km), round(y * km)
            pos = [(cx + i, cy + j) for j in range(-reach, reach + 1) for i in range(-reach, reach + 1)]
            hit = ncc_all(mrows, resize(ref, rw, rh, tw, th), pos)
            if hit and (best is None or hit[0] > best[0]):
                best = (hit[0], s2, hit[1] / km, hit[2] / km)
    if best is None:
        best = cands[0]
    # 3. Samples of the full pictures.
    _, s, x, y = best
    step = max(1, int(math.sqrt(rw * rh / SAMPLES)))
    pts = [(u, v) for v in range(step // 2, rh, step) for u in range(step // 2, rw, step)]
    reach = int(math.ceil(1 / km)) + 1
    top = (-2.0, s, s, x, y)
    for d in range(-5, 6):
        sx = s * (1 + d / 400)
        for e in (range(-8, 9) if stretch else (None,)):
            sy = sx if e is None else s * (1 + e / 400)
            for j in range(-reach, reach + 1):
                for i in range(-reach, reach + 1):
                    sc = sampled(ref, rw, rh, now, nw, nh, sx, sy, x + i, y + j, pts)
                    if sc > top[0]:
                        top = (sc, sx, sy, x + i, y + j)
    sc, sx, sy, dx, dy = top
    return {'ref': ref_path, 'now': now_path, 'ref_size': [rw, rh], 'now_size': [nw, nh],
            'scale_x': round(sx, 5), 'scale_y': round(sy, 5),
            'offset_x': round(dx, 2), 'offset_y': round(dy, 2), 'score': round(sc, 4)}


def to_now(a, x, y):
    return a['offset_x'] + x * a['scale_x'], a['offset_y'] + y * a['scale_y']


def to_ref(a, x, y):
    return (x - a['offset_x']) / a['scale_x'], (y - a['offset_y']) / a['scale_y']


def draw(a, points, out):
    w, h, rgb = read_png(a['now'])
    px = bytearray(rgb)

    def dot(x, y):
        if 0 <= x < w and 0 <= y < h:
            px[(y * w + x) * 3:(y * w + x) * 3 + 3] = b'\xff\x00\xff'
    x0, y0 = to_now(a, 0, 0)
    x1, y1 = to_now(a, *a['ref_size'])
    for x in range(int(x0), int(x1) + 1):
        dot(x, int(y0)); dot(x, int(y1))
    for y in range(int(y0), int(y1) + 1):
        dot(int(x0), y); dot(int(x1), y)
    for px_, py_ in points:
        cx, cy = (int(round(v)) for v in to_now(a, px_, py_))
        for k in range(-6, 7):
            dot(cx + k, cy); dot(cx, cy + k)
    write_png(out, w, h, bytes(px))


def option(argv, name, default=None):
    if name in argv:
        i = argv.index(name)
        v = argv[i + 1]
        del argv[i:i + 2]
        return v
    return default


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__); return 0
    units = float(option(argv, '--units', '1'))
    if argv[0] == 'map':
        inverse = '--inverse' in argv
        argv = [v for v in argv if v != '--inverse']
        a = json.load(open(argv[1]))
        x, y = float(argv[2]), float(argv[3])
        if inverse:
            mx, my = to_ref(a, x * units, y * units)
        else:
            mx, my = (v / units for v in to_now(a, x, y))
        print(f'{mx:.1f} {my:.1f}')
        return 0
    if argv[0] != 'find' or len(argv) < 3:
        sys.exit('usage: align.py find REF NOW ... (see --help)')
    points = []
    while '--point' in argv:
        points.append(tuple(float(v) for v in option(argv, '--point').split(',')))
    lo, hi = (float(v) for v in option(argv, '--scales', '0.25:4').split(':'))
    save, check = option(argv, '--save'), option(argv, '--check')
    stretch = '--stretch' in argv
    a = find(argv[1], argv[2], lo, hi, stretch)
    print(f'scale {a["scale_x"]:.4f} x {a["scale_y"]:.4f}, offset {a["offset_x"]:.1f}, {a["offset_y"]:.1f}, '
          f'score {a["score"]:.3f}')
    if a['score'] < 0.5:
        print('weak match: look at --check before clicking, or try tools/llm/locate.py')
    for x, y in points:
        mx, my = to_now(a, x, y)
        print(f'point {x:g},{y:g} in REF is {mx / units:.1f},{my / units:.1f} in NOW')
    if save:
        with open(save, 'w') as f:
            json.dump(a, f, indent=1)
    if check:
        draw(a, points, check)
        print(f'check picture: {check}')
    return 0 if a['score'] >= 0.5 else 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
