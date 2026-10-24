#!/usr/bin/env python3
"""Read a shade, fog or remap table of a palette game: for each level, the colour that each
palette colour turns into. Says which level is black, which leaves every colour unchanged,
which is white, and what colour the levels fade toward.

Usage: shadetable.py FILE --palette PAL [--offset N] [--levels N] [--colours N]
                     [--layout level|colour] [--palette-offset N] [--dac shift|scale]
                     [--png OUT.png] [--json]

  FILE          the file that holds the table (a remap file, the executable, a memory dump)
  --palette     the palette the table belongs to: 768 bytes of RGB, 6-bit or 8-bit, or a JASC
                .pal; --palette-offset N reads it from inside a bigger file (after a header)
  --offset N    where the table starts in FILE (default 0)
  --levels N    the number of levels (default 64; 32 and 16 are common too)
  --colours N   the colours per level (default 256)
  --layout      level: one row of N colours per level, table[level * colours + colour], the
                way most renderers look it up. colour: one run of levels per colour,
                table[colour * levels + level], the way some files store it. Default: the
                one that has a level leaving every colour as it is, else level
  --dac         how 6-bit palette values become 8 bits (rawimage.py --dac)
  --png OUT     a picture of the table: one row per level, one column per colour, each in the
                colour it maps to, 4 pixels wide (look at it: a fade to black or to a haze
                colour is obvious)
  --json        the measurements per level

A game with 256 colours cannot multiply a colour by a light value. It looks the answer up in
a table instead: row `level`, column `colour`. The renderer turns an intensity into a level
with a shift, so a 2.14 light of 0x4000 becomes level 32 with `>> 9`. Knowing which level
leaves colours unchanged tells you the intensity the game treats as normal light, and so how
the lighting code's numbers map to what is drawn (docs/hardware/software-rendering.md).

For each level the tool reports how many colours map to themselves, how bright the result is
compared with the input (measured on the palette's luminance), and, when most colours move
toward one colour, that colour: fog and haze tables fade toward the fog's colour, light
tables toward black or white. It does not find the table in a file: give its offset (the code
that loads it, re-trace) or try candidate offsets. Standard library only.
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rawimage import load_palette, png  # noqa: E402


def luma(rgb):
    r, g, b = rgb
    return 0.299 * r + 0.587 * g + 0.114 * b


def read(data, offset, levels, colours, layout):
    if offset + levels * colours > len(data):
        sys.exit(f'the table needs {levels * colours} bytes at {offset}; the file has {len(data) - offset}')
    t = data[offset:offset + levels * colours]
    if layout == 'colour':
        return [bytes(t[c * levels + lv] for c in range(colours)) for lv in range(levels)]
    return [t[lv * colours:(lv + 1) * colours] for lv in range(levels)]


def measure(rows, pal):
    colours = len(rows[0])
    lit = [c for c in range(colours) if c < len(pal) and luma(pal[c]) > 8]
    base = sum(luma(pal[c]) for c in lit) or 1
    out = []
    for lv, row in enumerate(rows):
        same = sum(row[c] == c for c in range(colours))
        ratio = sum(luma(pal[row[c]]) for c in lit if row[c] < len(pal)) / base
        counts = {}
        for c in range(colours):
            counts[row[c]] = counts.get(row[c], 0) + 1
        top = max(counts, key=counts.get)
        out.append({'level': lv, 'unchanged': same, 'brightness': round(ratio, 3),
                    'most_common': top, 'most_common_share': round(counts[top] / colours, 3),
                    'most_common_rgb': list(pal[top]) if top < len(pal) else None})
    return out


def describe(m, colours):
    """A label for one level; neighbouring levels with the same label are printed as one span."""
    if m['unchanged'] >= colours - 8:
        return 'unchanged'
    if m['brightness'] < 0.05:
        return 'black'
    if m['most_common_share'] >= 0.5:
        rgb = m['most_common_rgb']
        return f'mostly colour {m["most_common"]} {tuple(rgb) if rgb else ""}'
    return 'darker' if m['brightness'] < 1 else 'brighter'


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__)
        return 0 if argv else 2
    opt = lambda name, default=None: argv[argv.index(name) + 1] if name in argv else default
    if '--palette' not in argv:
        sys.exit('--palette is needed: the table maps colours of a palette')
    pal = load_palette(opt('--palette'), int(opt('--palette-offset', '0'), 0), opt('--dac', 'scale'))
    data = open(argv[0], 'rb').read()
    offset, levels, colours = (int(opt(n, d), 0) for n, d in (('--offset', '0'), ('--levels', '64'), ('--colours', '256')))
    layout = opt('--layout')
    if layout is None:
        def has_identity(lay):
            return any(sum(r[c] == c for c in range(colours)) >= colours - 8
                       for r in read(data, offset, levels, colours, lay))
        layout = 'colour' if not has_identity('level') and has_identity('colour') else 'level'
    rows = read(data, offset, levels, colours, layout)
    ms = measure(rows, pal)
    if '--png' in argv:
        w = 4
        png(opt('--png'), colours * w, levels * w,
            [b''.join(bytes(pal[i] if i < len(pal) else (255, 0, 255)) * w for i in row) for row in rows for _ in range(w)])
    if '--json' in argv:
        print(json.dumps({'layout': layout, 'levels': ms}, indent=1))
        return 0
    print(f'{levels} levels x {colours} colours, layout {layout}, at offset {offset}')
    start, last = 0, None
    for lv, m in enumerate(ms + [None]):
        text = describe(m, colours) if m else None
        if text != last:
            if last is not None:
                span = f'level {start}' if start == lv - 1 else f'levels {start}-{lv - 1}'
                b = [x['brightness'] for x in ms[start:lv]]
                same = [x['unchanged'] for x in ms[start:lv]]
                print(f'  {span:14s} {last:24s} brightness {min(b):.2f}-{max(b):.2f}, '
                      f'{min(same)}-{max(same)} of {colours} colours unchanged')
            start, last = lv, text
    same = [m['level'] for m in ms if m['unchanged'] >= colours - 8]
    if same:
        shift = None
        for s in range(8, 16):
            if (0x4000 >> s) in same:
                shift = s
        hint = f'; a 2.14 intensity of 0x4000 >> {shift} lands there' if shift else ''
        print(f'normal light: level {same[0]}' + (f' to {same[-1]}' if len(same) > 1 else '') + hint)
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
