#!/usr/bin/env python3
"""Draw the guides' diagrams as SVG files in docs/assets/diagrams/: static pictures and
animated ones, in the site's dark card style, so they show the same on GitHub and on the site.

Usage: python3 site/diagrams/make.py            write every diagram
       python3 site/diagrams/make.py --check    exit 1 if a written file differs (for tests)

Each diagram is one function below that returns SVG text. The numbers in them come from the
standards and the tools they illustrate (a CD sector's layout, the VGA's planes, a 16.16 fixed
point number), so the picture and the text say the same thing. Animation uses SVG's own
<animate> elements, which run in an <img> on GitHub and in the browser, with no script.
Standard library only.
"""
import math
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, 'docs', 'assets', 'diagrams')

BG, EDGE = '#0d1b1e', '#1f3d40'
TEXT, DIM = '#e2ece9', '#8fa9a5'
TEAL, AMBER, BLUE, RED, GREEN, PURPLE, GREY = '#2dd4bf', '#f59e0b', '#60a5fa', '#f87171', '#4ade80', '#c084fc', '#64748b'
FONT = "ui-sans-serif, -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif"
MONO = "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace"


def esc(s):
    return str(s).replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')


class Svg:
    def __init__(self, w, h, label):
        self.w, self.h, self.parts = w, h, []
        self.label = label

    def add(self, s):
        self.parts.append(s)
        return self

    def text(self, x, y, s, size=13, fill=TEXT, anchor='start', weight='400', mono=False, extra=''):
        return self.add(f'<text x="{x:.1f}" y="{y:.1f}" font-family="{MONO if mono else FONT}" font-size="{size}" '
                        f'font-weight="{weight}" fill="{fill}" text-anchor="{anchor}"{extra}>{esc(s)}</text>')

    def rect(self, x, y, w, h, fill, stroke='none', rx=4, extra=''):
        return self.add(f'<rect x="{x:.1f}" y="{y:.1f}" width="{w:.1f}" height="{h:.1f}" rx="{rx}" fill="{fill}" '
                        f'stroke="{stroke}"{extra}/>')

    def line(self, x1, y1, x2, y2, stroke=DIM, width=1.5, extra=''):
        return self.add(f'<path d="M{x1:.1f} {y1:.1f}L{x2:.1f} {y2:.1f}" stroke="{stroke}" stroke-width="{width}" '
                        f'fill="none"{extra}/>')

    def heading(self, s, sub=None):
        self.text(24, 34, s.upper(), 12, TEAL, weight='600', extra=' letter-spacing="0.08em"')
        if sub:
            self.text(24, 54, sub, 13, DIM)

    def render(self):
        head = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{self.w}" height="{self.h}" '
                f'viewBox="0 0 {self.w} {self.h}" role="img" aria-label="{esc(self.label)}">\n'
                f'<title>{esc(self.label)}</title>\n'
                f'<rect x="0.5" y="0.5" width="{self.w - 1}" height="{self.h - 1}" rx="10" fill="{BG}" stroke="{EDGE}"/>\n'
                '<defs><marker id="arrow" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" '
                f'orient="auto"><path d="M0 0L8 4L0 8z" fill="{DIM}"/></marker></defs>\n')
        return head + '\n'.join(self.parts) + '\n</svg>\n'


# ---------------------------------------------------------------- CDs and DVDs
def cd_sectors():
    s = Svg(900, 400, 'A CD sector of 2352 bytes in four kinds: audio, Mode 1, Mode 2 Form 1 and Mode 2 Form 2')
    s.heading('A CD sector: 2352 bytes', 'Where the user data sits in each kind of sector. Widths are to scale.')
    rows = [
        ('Audio', [(2352, 'audio samples: 588 stereo pairs, 16-bit', BLUE)]),
        ('Mode 1', [(12, 'sync', GREY), (4, 'hdr', PURPLE), (2048, 'user data: 2048', TEAL), (4, 'EDC', AMBER), (8, '', GREY), (276, 'ECC 276', RED)]),
        ('Mode 2 Form 1', [(12, 'sync', GREY), (4, 'hdr', PURPLE), (8, 'sub', GREEN), (2048, 'user data: 2048', TEAL), (4, 'EDC', AMBER), (276, 'ECC 276', RED)]),
        ('Mode 2 Form 2', [(12, 'sync', GREY), (4, 'hdr', PURPLE), (8, 'sub', GREEN), (2324, 'user data: 2324 (video, streamed sound)', TEAL), (4, 'EDC', AMBER)]),
    ]
    x0, width, y = 150, 720, 80
    for name, parts in rows:
        s.text(x0 - 12, y + 24, name, 13, TEXT, 'end', '600')
        x = x0
        for size, label, colour in parts:
            w = size / 2352 * width
            s.rect(x, y, max(w - 1, 1), 36, colour, rx=2, extra=' fill-opacity="0.85"')
            if label and w > 60:
                s.text(x + 8, y + 23, label, 12, BG, weight='600')
            x += w
        y += 50
    # The first 24 bytes of a Mode 2 sector, enlarged.
    s.text(x0 - 12, y + 24, 'first 24 bytes', 12, DIM, 'end')
    ex = x0
    for size, label, colour in ((12, 'sync: 00 FF x10 00', GREY), (4, 'header', PURPLE), (8, 'XA subheader (Mode 2)', GREEN)):
        w = size * 30
        s.rect(ex, y, w - 2, 30, colour, rx=2, extra=' fill-opacity="0.85"')
        s.text(ex + 8, y + 20, label, 12, BG, weight='600')
        ex += w
    s.text(x0, y + 54, 'EDC and ECC: error detection and correction, which audio sectors lack. Bit 5 of the subheader marks Form 2.', 11, DIM)
    s.text(x0, y + 72, 'A cooked .iso keeps only the 2048 user bytes of Mode 1 and Form 1 sectors. A raw .bin keeps all 2352.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- floppy image levels
def floppy_levels():
    s = Svg(900, 280, 'Three levels of floppy disk image: sectors in order, sectors with their layout, and the flux')
    s.heading('Three levels of disk image', 'Each level keeps more of what was on the disk.')
    levels = [
        ('Sectors in order', '.img .adf .st .dsk .d64', 'the data of each standard sector', TEAL, 1),
        ('Sectors with their layout', '.imd .td0 .stx .g64', 'IDs, sizes, gaps and errors of every track', AMBER, 2),
        ('Flux', 'KryoFlux, .scp, Greaseweazle, .ipf, .woz', 'the signal as the drive read it, with timing', RED, 3),
    ]
    y = 78
    for name, examples, holds, colour, depth in levels:
        s.rect(24, y, 320, 52, colour, rx=6, extra=' fill-opacity="0.18"')
        s.rect(24, y, 6, 52, colour, rx=2)
        s.text(44, y + 22, name, 14, TEXT, weight='600')
        s.text(44, y + 40, examples, 11, DIM, mono=True)
        s.text(362, y + 31, holds, 13, TEXT)
        # A track drawn three ways: blocks, blocks with gaps and marks, a wave.
        x0 = 660
        if depth == 1:
            for i in range(9):
                s.rect(x0 + i * 26, y + 14, 22, 24, colour, rx=2, extra=' fill-opacity="0.8"')
        elif depth == 2:
            for i, w in enumerate((18, 18, 30, 18, 12, 18, 18, 24)):
                xx = x0 + sum((18, 18, 30, 18, 12, 18, 18, 24)[:i]) + i * 6
                s.rect(xx, y + 14, w, 24, colour if i != 4 else RED, rx=2, extra=' fill-opacity="0.8"')
                s.rect(xx, y + 10, w, 3, PURPLE, rx=1)
        else:
            pts = ' '.join(f'{x0 + i * 2:.1f},{y + 26 + 11 * math.sin(i * 0.9) * (1 if i % 17 else 0.3):.1f}' for i in range(118))
            s.add(f'<polyline points="{pts}" fill="none" stroke="{colour}" stroke-width="1.6"/>')
        y += 64
    return s.render()


# ---------------------------------------------------------------- entropy scale
def entropy_scale():
    s = Svg(900, 220, 'The entropy scale from 0 to 8 bits per byte, with what each range usually is')
    s.heading('Entropy, in bits per byte', 'What a file or a block usually is, by its entropy.')
    x0, w, y = 40, 820, 110
    stops = [(0, '#14145a'), (0.35, '#1ea0c8'), (0.6, '#3cb43c'), (0.85, '#f0dc28'), (1, '#dc1e1e')]
    grad = ''.join(f'<stop offset="{o}" stop-color="{c}"/>' for o, c in stops)
    s.add(f'<defs><linearGradient id="eg">{grad}</linearGradient></defs>')
    s.rect(x0, y, w, 26, 'url(#eg)', rx=4)
    for v in range(9):
        x = x0 + v / 8 * w
        s.line(x, y + 26, x, y + 34, DIM, 1)
        s.text(x, y + 48, str(v), 12, DIM, 'middle')
    zones = [(0, 1, 'padding, empty tables'), (4, 6, 'text, code, small numbers'), (6, 7.5, 'code and data mixed, sound'),
             (7.5, 8, 'compressed or encrypted')]
    for i, (a, b, label) in enumerate(zones):
        xa, xb = x0 + a / 8 * w, x0 + b / 8 * w
        lift = 22 if i == 3 else 0
        s.add(f'<path d="M{xa:.1f} {y - 6}V{y - 12 - lift}H{xb:.1f}V{y - 6}" fill="none" stroke="{TEXT}" stroke-opacity="0.5"/>')
        s.text((xa + xb) / 2 if i < 3 else xb, y - 18 - lift, label, 11, TEXT, 'middle' if i < 3 else 'end')
    s.text(x0, y + 74, 'tools/triage/filetypes.py and binview.py measure it; the browser lab draws it along a file.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- Mode X planes (animated)
def modex_planes():
    s = Svg(900, 330, 'Mode X: pixel x of a row goes to plane x mod 4, at byte x divided by 4')
    s.heading('Mode X: four planes', 'Pixel x is stored in plane x mod 4, at offset x / 4. The highlight follows each pixel to its plane.')
    colours = [TEAL, AMBER, BLUE, RED]
    x0, y0, cw = 60, 86, 44
    s.text(x0, y0 - 8, 'the screen row', 11, DIM)
    for x in range(16):
        s.rect(x0 + x * (cw + 4), y0, cw, 34, colours[x % 4], rx=3, extra=' fill-opacity="0.85"')
        s.text(x0 + x * (cw + 4) + cw / 2, y0 + 22, str(x), 12, BG, 'middle', '600')
    py = 190
    for p in range(4):
        px = x0 + p * 200
        s.text(px, py - 10, f'plane {p}', 12, colours[p], weight='600')
        for k in range(4):
            s.rect(px + k * 44, py, 40, 30, colours[p], rx=3, extra=' fill-opacity="0.25"')
            s.text(px + k * 44 + 20, py + 20, str(p + 4 * k), 12, TEXT, 'middle')
        s.text(px, py + 50, 'offsets 0, 1, 2, 3', 11, DIM)
    # The moving highlight: one marker that visits each pixel and then its plane slot, 16 steps.
    n, step = 16, 0.6
    keys = ';'.join(f'{i / n:.4f}' for i in range(n)) + ';1'
    xs = [x0 + x * (cw + 4) + cw / 2 for x in range(n)] + [x0 + cw / 2]
    tx = [x0 + (x % 4) * 200 + (x // 4) * 44 + 20 for x in range(n)] + [x0 + 20]
    dur = n * step
    s.add(f'<circle r="7" fill="none" stroke="{TEXT}" stroke-width="2"><animate attributeName="cx" values="{";".join(f"{v:.1f}" for v in xs)}" '
          f'keyTimes="{keys}" calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/>'
          f'<animate attributeName="cy" values="{";".join(["103"] * (n + 1))}" dur="{dur}s" repeatCount="indefinite"/></circle>')
    s.add(f'<circle r="7" fill="none" stroke="{TEXT}" stroke-width="2"><animate attributeName="cx" values="{";".join(f"{v:.1f}" for v in tx)}" '
          f'keyTimes="{keys}" calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/>'
          f'<animate attributeName="cy" values="{";".join(["205"] * (n + 1))}" dur="{dur}s" repeatCount="indefinite"/></circle>')
    s.text(x0, 300, 'With all four planes enabled, one byte written fills four neighbouring pixels at once.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- fixed point
def fixed_point():
    s = Svg(900, 230, 'A 16.16 fixed-point number: 16 bits of whole part and 16 bits of fraction, with an example')
    s.heading('16.16 fixed point', 'A 32-bit integer read as a number with 16 bits after the point.')
    x0, y, bw = 40, 84, 25
    value = int(3.25 * 65536) | 0
    bits = f'{value & 0xFFFFFFFF:032b}'
    for i, b in enumerate(bits):
        colour = AMBER if i < 16 else TEAL
        s.rect(x0 + i * bw, y, bw - 2, 30, colour, rx=2, extra=f' fill-opacity="{0.9 if b == "1" else 0.2}"')
        s.text(x0 + i * bw + (bw - 2) / 2, y + 20, b, 12, BG if b == '1' else TEXT, 'middle', '600', mono=True)
    s.text(x0, y + 52, 'whole part: 3', 13, AMBER, weight='600')
    s.text(x0 + 16 * bw, y + 52, 'fraction: 0x4000 / 0x10000 = 0.25', 13, TEAL, weight='600')
    s.text(x0, y + 84, 'The value 0x00034000 is 3.25. Multiplying two of them needs 64 bits before the shift back (imul, then shrd 16).', 12, DIM)
    s.text(x0, y + 102, 'A port that keeps only 32 bits overflows where the original did not.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- fill rule
def fill_rule():
    s = Svg(900, 360, 'A fill rule: pixels whose centres lie on a left or top edge are drawn, on a right or bottom edge they are not')
    s.heading('Which pixels a polygon covers', 'Two triangles share an edge. With a top-left rule, each pixel on the shared edge is drawn once.')
    gx, gy, c = 60, 80, 26
    n = 10
    A, B, C, D = (1, 1), (9, 1), (1, 9), (9, 9)

    def inside(px, py, tri):
        (x1, y1), (x2, y2), (x3, y3) = tri

        def edge(ax, ay, bx, by):
            return (bx - ax) * (py - ay) - (by - ay) * (px - ax)
        e = [edge(x1, y1, x2, y2), edge(x2, y2, x3, y3), edge(x3, y3, x1, y1)]
        return all(v >= 0 for v in e) or all(v <= 0 for v in e), e

    t1, t2 = (A, B, D), (A, D, C)
    for gxi in range(n):
        for gyi in range(n):
            px, py = gxi + 0.5, gyi + 0.5
            in1, e1 = inside(px, py, t1)
            in2, e2 = inside(px, py, t2)
            on_shared = abs((py - 1) - (px - 1)) < 1e-9 and 1 <= px <= 9
            colour = None
            if on_shared:
                colour = AMBER                      # the shared diagonal: drawn by one triangle only
            elif in1:
                colour = TEAL
            elif in2:
                colour = BLUE
            s.rect(gx + gxi * c, gy + gyi * c, c - 2, c - 2, colour or '#16302f', rx=2,
                   extra=' fill-opacity="0.85"' if colour else '')
    pts = lambda tri: ' '.join(f'{gx + x * c:.1f},{gy + y * c:.1f}' for x, y in tri)
    s.add(f'<polygon points="{pts(t1)}" fill="none" stroke="{TEXT}" stroke-width="1.5"/>')
    s.add(f'<polygon points="{pts(t2)}" fill="none" stroke="{TEXT}" stroke-width="1.5" stroke-dasharray="4 3"/>')
    lx = gx + n * c + 50
    for i, (colour, label) in enumerate(((TEAL, 'drawn by the first triangle'), (BLUE, 'drawn by the second'),
                                        (AMBER, 'centre on the shared edge: drawn once, by the triangle'),
                                        (None, 'for which it is a left or top edge'))):
        if colour:
            s.rect(lx, 98 + i * 30, 18, 18, colour, rx=2)
        s.text(lx + (28 if colour else 28), 112 + i * 30 - (8 if not colour else 0), label, 13, TEXT)
    s.text(lx, 250, 'Without a rule, both triangles draw that diagonal (a seam of', 12, DIM)
    s.text(lx, 268, 'double-drawn pixels) or neither does (a crack). Games of the era', 12, DIM)
    s.text(lx, 286, 'chose their own rule. Measure it, then copy it.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- pointer chain (animated)
def pointer_chain():
    s = Svg(900, 250, 'A pointer chain: a global holds the world, the world at +0x18 holds the player, the player at +0x24 holds the gold')
    s.heading('A path to a value that moves', 'The global stays put; the objects move each run. Following the chain finds the gold in any run.')
    boxes = [('global', '0x4a1f30', 'static', TEAL), ('world', '+0x18', 'heap', BLUE), ('player', '+0x24', 'heap', BLUE),
             ('gold', '500', 'the value', AMBER)]
    x, y = 40, 100
    centres = []
    for i, (name, field, where, colour) in enumerate(boxes):
        s.rect(x, y, 170, 70, colour, rx=8, extra=' fill-opacity="0.16"')
        s.rect(x, y, 170, 4, colour, rx=2)
        s.text(x + 14, y + 30, name, 15, TEXT, weight='600')
        s.text(x + 14, y + 52, f'{field}  ({where})', 12, DIM, mono=True)
        centres.append((x, x + 170))
        if i < len(boxes) - 1:
            s.add(f'<path d="M{x + 172} {y + 35}H{x + 218}" stroke="{DIM}" stroke-width="2" marker-end="url(#arrow)"/>')
        x += 220
    path = f'M{centres[0][0] + 85} {y + 35}' + ''.join(f'L{a + 85} {y + 35}' for a, _ in centres[1:])
    s.add(f'<circle r="6" fill="{AMBER}"><animateMotion dur="4s" repeatCount="indefinite" path="{path}" '
          f'keyPoints="0;0.33;0.66;1;1" keyTimes="0;0.25;0.5;0.75;1" calcMode="linear"/></circle>')
    s.text(40, 210, '[0x4a1f30] -> +0x18 -> +0x24', 13, TEXT, mono=True)
    s.text(300, 210, 'tools/triage/pointerpaths.py finds it from dumps of two runs', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- coverage diff
def coverage_diff():
    s = Svg(900, 300, 'Code coverage of two runs: the blocks that ran only with the action are the action\'s code')
    s.heading('Coverage: what ran only with the action', 'Two runs alike in all but one action. Most code runs in both.')
    cx1, cx2, cy, r = 330, 470, 185, 90
    s.add(f'<circle cx="{cx1}" cy="{cy}" r="{r}" fill="{BLUE}" fill-opacity="0.25" stroke="{BLUE}"/>')
    s.add(f'<circle cx="{cx2}" cy="{cy}" r="{r}" fill="{AMBER}" fill-opacity="0.25" stroke="{AMBER}"/>')
    s.text(cx1 - 70, cy - 96, 'baseline run', 13, BLUE, 'middle', '600')
    s.text(cx2 + 70, cy - 96, 'run with the action', 13, AMBER, 'middle', '600')
    s.text(400, cy - 4, 'main loop, drawing,', 12, TEXT, 'middle')
    s.text(400, cy + 14, 'input, sound...', 12, TEXT, 'middle')
    s.text(cx1 - 40, cy + 4, 'start-up only', 11, DIM, 'middle')
    s.text(cx2 + 62, cy - 4, 'buy_unit', 13, AMBER, 'middle', '600')
    s.text(cx2 + 62, cy + 14, 'spend_gold', 13, AMBER, 'middle', '600')
    s.text(620, 140, 'coverdiff.py BASELINE ACTION', 12, TEXT, mono=True)
    s.text(620, 160, '  --functions functions.tsv', 12, TEXT, mono=True)
    s.text(620, 190, 'lists the right-hand crescent by', 12, DIM)
    s.text(620, 208, 'function, a handful out of thousands.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- Hilbert curve (animated)
def hilbert_curve():
    order = 4
    side = 1 << order
    s = Svg(900, 330, 'A Hilbert curve of order 4: it visits every cell, and neighbouring bytes stay close')
    s.heading('The Hilbert layout', 'binview.py --layout hilbert walks this curve, so each region of a file becomes a patch.')

    def h(d):
        x = y = 0
        t, sz = d, 1
        while sz < side:
            rx = 1 & (t // 2)
            ry = 1 & (t ^ rx)
            if ry == 0:
                if rx == 1:
                    x, y = sz - 1 - x, sz - 1 - y
                x, y = y, x
            x += sz * rx
            y += sz * ry
            t //= 4
            sz *= 2
        return x, y
    cell, x0, y0 = 15, 60, 80
    pts = [h(d) for d in range(side * side)]
    path = ' '.join(f'{"M" if i == 0 else "L"}{x0 + x * cell + cell / 2:.1f} {y0 + y * cell + cell / 2:.1f}' for i, (x, y) in enumerate(pts))
    length = (side * side - 1) * cell
    s.add(f'<path d="{path}" fill="none" stroke="{TEAL}" stroke-opacity="0.18" stroke-width="3"/>')
    s.add(f'<path d="{path}" fill="none" stroke="{TEAL}" stroke-width="3" stroke-linecap="round" stroke-dasharray="{length}" '
          f'stroke-dashoffset="{length}"><animate attributeName="stroke-dashoffset" values="{length};0;0" keyTimes="0;0.8;1" '
          f'dur="8s" repeatCount="indefinite"/></path>')
    # The same bytes in rows: a file region of 64 bytes highlighted in both layouts.
    rx0 = 380
    s.text(rx0, y0 - 8, 'rows of 16: bytes 96 to 159 form a band', 11, DIM)
    s.text(rx0 + 300, y0 - 8, 'Hilbert: the same bytes form a patch', 11, DIM)
    for d in range(256):
        x, y = d % 16, d // 16
        hot = 96 <= d < 160
        s.rect(rx0 + x * 15, y0 + y * 15, 13, 13, AMBER if hot else '#16302f', rx=1)
        hx, hy = pts[d]
        s.rect(rx0 + 300 + hx * 15, y0 + hy * 15, 13, 13, AMBER if hot else '#16302f', rx=1)
    return s.render()


# ---------------------------------------------------------------- message anatomy
def message_anatomy():
    s = Svg(900, 330, 'A game message laid out byte by byte: magic, type, sequence number, length and body, and many messages as rows')
    s.heading('Reading a protocol', 'One message, and forty of them side by side: the fields line up as columns.')
    fields = [('magic', 2, GREY, '47 46'), ('type', 1, PURPLE, '02'), ('sequence', 2, GREEN, '65 00'),
              ('length', 2, BLUE, '13 00'), ('body', 12, TEAL, '21 06 f0 84 77 62 f0 f3 ...')]
    x, y, bw = 40, 84, 36
    for name, size, colour, sample in fields:
        w = size * bw
        s.rect(x, y, w - 3, 34, colour, rx=3, extra=' fill-opacity="0.85"')
        s.text(x + 6, y + 22, sample if size < 12 else sample, 12, BG, weight='600', mono=True)
        s.text(x + 2, y + 52, name, 12, colour, weight='600')
        x += w
    # Forty messages as rows: constant columns, a counter ramp, a type column, noisy body.
    import random
    r = random.Random(5)
    gx, gy, c = 40, 170, 9
    for row in range(14):
        kind = (1, 2, 3)[row % 3]
        cols = [0x47, 0x46, kind * 70, 100 + row * 9, 0, 7 + {1: 4, 2: 12, 3: 20}[kind], 0] + [r.randrange(256) for _ in range(12)]
        for col, v in enumerate(cols):
            hue = v / 256 * 6
            k = int(hue) % 6
            f = hue - int(hue)
            q, t = int(255 * (1 - f)), int(255 * f)
            rgb = [(255, t, 0), (q, 255, 0), (0, 255, t), (0, q, 255), (t, 0, 255), (255, 0, q)][k]
            s.rect(gx + col * (c + 1) * 2, gy + row * (c + 1), c * 2, c, f'rgb{rgb}', rx=1, extra=' fill-opacity="0.85"')
    s.text(gx + 420, 190, 'flat columns: constants (the magic)', 12, TEXT)
    s.text(gx + 420, 210, 'a smooth ramp: the sequence counter', 12, TEXT)
    s.text(gx + 420, 230, 'three repeating values: the message type', 12, TEXT)
    s.text(gx + 420, 250, 'noise: the body, or compressed data', 12, TEXT)
    s.text(gx + 420, 280, 'tools/formats/fields.py and the records explorer', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- OAIS packages
def oais_packages():
    s = Svg(900, 260, 'Three packages: the copy as received, the copy kept with checksums and findings, and what is given to players')
    s.heading('Keeping a game: three packages', 'The words come from the OAIS reference model that archives use.')
    boxes = [('Received', 'the installer or disc image,', 'as bought, and where from', GREY),
             ('Kept', 'a bag with checksums, provenance,', 'hashes and the findings', TEAL),
             ('Given to players', 'the port and its documents,', 'run with the player\'s own copy', AMBER)]
    x = 40
    for i, (name, a, b, colour) in enumerate(boxes):
        s.rect(x, 90, 240, 100, colour, rx=10, extra=' fill-opacity="0.16"')
        s.rect(x, 90, 240, 5, colour, rx=2)
        s.text(x + 18, 124, name, 16, TEXT, weight='600')
        s.text(x + 18, 150, a, 12, DIM)
        s.text(x + 18, 168, b, 12, DIM)
        if i < 2:
            s.add(f'<path d="M{x + 244} 140H{x + 290}" stroke="{DIM}" stroke-width="2" marker-end="url(#arrow)"/>')
        x += 300
    s.add(f'<path d="M370 196V222H160V196" fill="none" stroke="{TEAL}" stroke-dasharray="4 3" marker-end="url(#arrow)"/>')
    s.text(265, 240, 'fixity.py verify: checked again over the years', 11, TEAL, 'middle')
    return s.render()


# ---------------------------------------------------------------- PC hardware timeline
def pc_timeline():
    s = Svg(900, 350, 'A timeline of PC graphics, sound and CPU features from 1981 to 1999')
    s.heading('The PC games hardware, 1981 to 1999', 'When each standard arrived. A game targets what was common when it shipped.')
    y0, x0, x1, y_axis = 0, 60, 860, 160
    lo, hi = 1981, 1999
    X = lambda yr: x0 + (yr - lo) / (hi - lo) * (x1 - x0)
    s.line(x0, y_axis, x1, y_axis, DIM, 1.5)
    for yr in range(lo, hi + 1):
        s.line(X(yr), y_axis - 4, X(yr), y_axis + 4, DIM, 1)
        if yr % 2 == 1:
            s.text(X(yr), y_axis + 22, str(yr), 11, DIM, 'middle')
    events = [  # (year, label, track, colour): track -1 above (graphics), +1 below (sound and CPU)
        (1981, 'CGA', -1, TEAL), (1984, 'EGA', -1, TEAL), (1987, 'VGA', -1, TEAL), (1991, 'Mode X (Abrash)', -2, TEAL),
        (1995, 'DirectX', -1, BLUE), (1996, '3dfx Voodoo', -2, BLUE), (1992, 'OpenGL 1.0', -1, BLUE),
        (1987, 'AdLib', 1, AMBER), (1989, 'Sound Blaster', 2, AMBER), (1992, 'Gravis Ultrasound', 1, AMBER),
        (1997, 'MMX', 1, RED), (1998, '3DNow!', 2, RED), (1999, 'SSE', 3, RED),
    ]
    for yr, label, track, colour in events:
        x = X(yr)
        y = y_axis + track * 28 + (26 if track > 0 else -8)
        if track > 0:
            s.line(x, y_axis + 30, x, y - 10, colour, 1)
        else:
            s.line(x, y_axis, x, y + 4, colour, 1)
        s.add(f'<circle cx="{x:.1f}" cy="{y_axis}" r="4" fill="{colour}"/>')
        s.text(x, y + (6 if track > 0 else 0), label, 12, colour, 'middle', '600')
    s.text(x0, 326, 'graphics', 11, TEAL)
    s.text(x0 + 70, 326, '3D APIs and cards', 11, BLUE)
    s.text(x0 + 200, 326, 'sound', 11, AMBER)
    s.text(x0 + 260, 326, 'CPU instructions', 11, RED)
    return s.render()


DIAGRAMS = {
    'cd-sectors.svg': cd_sectors, 'floppy-levels.svg': floppy_levels, 'entropy-scale.svg': entropy_scale,
    'modex-planes.svg': modex_planes, 'fixed-point.svg': fixed_point, 'fill-rule.svg': fill_rule,
    'pointer-chain.svg': pointer_chain, 'coverage-diff.svg': coverage_diff, 'hilbert.svg': hilbert_curve,
    'message-anatomy.svg': message_anatomy, 'oais-packages.svg': oais_packages, 'pc-timeline.svg': pc_timeline,
}


def main(argv):
    os.makedirs(OUT, exist_ok=True)
    stale = []
    for name, fn in DIAGRAMS.items():
        path = os.path.join(OUT, name)
        svg = fn()
        old = open(path, encoding='utf-8').read() if os.path.exists(path) else None
        if '--check' in argv:
            if old != svg:
                stale.append(name)
            continue
        if old != svg:
            with open(path, 'w', encoding='utf-8') as f:
                f.write(svg)
            print(f'wrote {os.path.relpath(path, ROOT)}')
    if stale:
        print('diagrams differ from site/diagrams/make.py: ' + ', '.join(stale))
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
