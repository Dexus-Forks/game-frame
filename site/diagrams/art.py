"""Editorial illustrations for the guides: a wide scene for each family of games, and a mosaic of
all ten for the top of the taxonomy. Each one illustrates the top of a section. Every scene is
computed here (the first-person view is a real ray caster, the road a real pseudo-3D
projection), seeded so that make.py writes the same file every time. Pixel sprites get a dark
outline, blocks a bevel, and light sources a glow, so the pictures read clearly at any size.
site/diagrams/make.py writes them with the diagrams.
"""
import math
import random

W, H = 1200, 400
OUTLINE = '#0a0a12'


class Art:
    """A borderless scene with rounded corners and a soft vignette. ids are prefixed so scenes
    can be nested in one file."""

    def __init__(self, key, label):
        self.key, self.label, self.parts = key, label, []
        self.defs = [f'<filter id="{key}-glow" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="5" '
                     f'result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>',
                     f'<filter id="{key}-blur" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="18"/>'
                     f'</filter>',
                     f'<radialGradient id="{key}-vig" cx="0.5" cy="0.5" r="0.75"><stop offset="0.6" stop-color="#000" '
                     f'stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity="0.45"/></radialGradient>']

    def id(self, name):
        return f'{self.key}-{name}'

    @property
    def glow(self):
        return f' filter="url(#{self.key}-glow)"'

    @property
    def blur(self):
        return f' filter="url(#{self.key}-blur)"'

    def add(self, s):
        self.parts.append(s)

    def rect(self, x, y, w, h, fill, extra=''):
        self.add(f'<rect x="{x:.1f}" y="{y:.1f}" width="{w:.1f}" height="{h:.1f}" fill="{fill}"{extra}/>')

    def gradient(self, name, stops, vertical=True):
        x2, y2 = ('0', '1') if vertical else ('1', '0')
        st = ''.join(f'<stop offset="{o}" stop-color="{c}"/>' for o, c in stops)
        self.defs.append(f'<linearGradient id="{self.id(name)}" x1="0" y1="0" x2="{x2}" y2="{y2}">{st}</linearGradient>')
        return f'url(#{self.id(name)})'

    def radial(self, name, stops, cx=0.5, cy=0.5, r=0.5):
        st = ''.join(f'<stop offset="{o}" stop-color="{c}"/>' for o, c in stops)
        self.defs.append(f'<radialGradient id="{self.id(name)}" cx="{cx}" cy="{cy}" r="{r}">{st}</radialGradient>')
        return f'url(#{self.id(name)})'

    def text(self, x, y, s, size, fill, anchor='start', weight='700', glow=False):
        g = self.glow if glow else ''
        self.add(f'<text x="{x}" y="{y}" font-family="ui-monospace, Menlo, Consolas, monospace" font-size="{size}" '
                 f'font-weight="{weight}" fill="{fill}" text-anchor="{anchor}"{g}>{s}</text>')

    def inner(self):
        """The scene's own elements, clipped, for nesting in another SVG."""
        return (f'<defs>{"".join(self.defs)}<clipPath id="{self.id("clip")}"><rect width="{W}" height="{H}" rx="14"/>'
                f'</clipPath></defs><g clip-path="url(#{self.id("clip")})">{"".join(self.parts)}'
                f'<rect width="{W}" height="{H}" fill="url(#{self.key}-vig)"/></g>')

    def render(self):
        return (f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img" '
                f'aria-label="{self.label}">\n<title>{self.label}</title>\n{self.inner()}\n</svg>\n')


def shade(hex_colour, k):
    v = [int(hex_colour[i:i + 2], 16) for i in (1, 3, 5)]
    return '#' + ''.join(f'{max(0, min(255, int(c * k))):02x}' for c in v)


def sprite(rows, palette, x, y, px, outline=OUTLINE):
    """Pixel art from strings: one character per pixel, '.' is clear. Runs become one rect each.
    With an outline colour, every clear pixel next to a filled one is drawn in it."""
    out = []
    filled = {(r, c) for r, row in enumerate(rows) for c, ch in enumerate(row) if ch != '.'}
    if outline:
        edge = set()
        for r, c in filled:
            for dr, dc in ((-1, 0), (1, 0), (0, -1), (0, 1)):
                if (r + dr, c + dc) not in filled:
                    edge.add((r + dr, c + dc))
        for r, c in sorted(edge):
            out.append(f'<rect x="{x + c * px:.1f}" y="{y + r * px:.1f}" width="{px:.1f}" height="{px:.1f}" fill="{outline}"/>')
    for r, row in enumerate(rows):
        c = 0
        while c < len(row):
            ch = row[c]
            if ch == '.':
                c += 1
                continue
            n = 1
            while c + n < len(row) and row[c + n] == ch:
                n += 1
            out.append(f'<rect x="{x + c * px:.1f}" y="{y + r * px:.1f}" width="{n * px + 0.4:.1f}" height="{px + 0.4:.1f}" '
                       f'fill="{palette[ch]}"/>')
            c += n
    return ''.join(out)


def bevel(x, y, w, h, col, d=3):
    """A block lit from the top left: a light top and left edge, a dark bottom and right edge."""
    return (f'<rect x="{x:.1f}" y="{y:.1f}" width="{w:.1f}" height="{h:.1f}" fill="{col}"/>'
            f'<path d="M{x:.1f} {y + h:.1f}V{y:.1f}H{x + w:.1f}l{-d} {d}H{x + d:.1f}V{y + h - d:.1f}z" fill="{shade(col, 1.45)}"/>'
            f'<path d="M{x + w:.1f} {y:.1f}V{y + h:.1f}H{x:.1f}l{d} {-d}H{x + w - d:.1f}V{y + d:.1f}z" fill="{shade(col, 0.55)}"/>')


def shadow(cx, cy, rx, ry, op=0.35):
    return f'<ellipse cx="{cx:.1f}" cy="{cy:.1f}" rx="{rx:.1f}" ry="{ry:.1f}" fill="#000" opacity="{op}"/>'


def stars(rng, n, colour, size, y0=0, y1=H, x0=0, x1=W):
    return ''.join(f'<rect x="{rng.uniform(x0, x1):.0f}" y="{rng.uniform(y0, y1):.0f}" width="{size}" height="{size}" '
                   f'fill="{colour}"/>' for _ in range(n))


def twinkles(rng, n, a):
    out = []
    for k in range(n):
        x, y = rng.uniform(0, W), rng.uniform(0, H)
        out.append(f'<path d="M{x:.0f} {y - 7:.0f}V{y + 7:.0f}M{x - 7:.0f} {y:.0f}H{x + 7:.0f}" stroke="#ffffff" stroke-width="1.5" '
                   f'opacity="0.2"{a.glow}><animate attributeName="opacity" values="0.15;0.9;0.15" dur="{2 + k % 3}s" '
                   f'begin="{-k * 0.7:.1f}s" repeatCount="indefinite"/></path>')
    return ''.join(out)


def frames(bodies, dur):
    """Show each body in turn, as the frames of a loop."""
    n, out = len(bodies), []
    for f, body in enumerate(bodies):
        t0, t1 = f / n, (f + 1) / n
        if f == 0:
            kt, vals = f'0;{t1:.4f};1', '1;0;0'
        elif f == n - 1:
            kt, vals = f'0;{t0:.4f}', '0;1'
        else:
            kt, vals = f'0;{t0:.4f};{t1:.4f};1', '0;1;0;0'
        out.append(f'<g opacity="{1 if f == 0 else 0}">{body}<animate attributeName="opacity" values="{vals}" keyTimes="{kt}" '
                   f'calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/></g>')
    return ''.join(out)


# ---------------------------------------------------------------- arcade action
SHIP = ['......ccc...............', '.....cwwwc..............', '....cwwbbwcc............', '..aacwwbbbwwccccc.......',
        'ayyacwwwwwwwwwwwwccc....', 'yrryaccwwwwwwwwwwwwwwcc.', 'ayyacsssssssssssssssssss', '..aacsssssssscccc.......',
        '....cssss.cc............', '.....csss...............', '......ccc...............']
INVADER = ['...e.....e...', '....e...e....', '...ggggggg...', '..gghggghgg..', '.ggggggggggg.', '.g.ggggggg.g.', '.g.g.....g.g.',
           '....gg.gg....']


def arcade(a=None):
    a = a or Art('arcade', 'A side-scrolling shooter: a ship fires at a formation of aliens over a moving starfield')
    rng = random.Random(1)
    a.rect(0, 0, W, H, a.gradient('sky', [(0, '#04051a'), (1, '#1c1250')]))
    for cx, cy, rx, col in ((300, 300, 260, '#7c3aed'), (850, 120, 300, '#0e7490'), (1100, 330, 200, '#be185d')):
        a.add(f'<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{rx * 0.45:.0f}" fill="{col}" opacity="0.22"{a.blur}/>')
    for speed, n, col, size in ((70, 90, '#3b3f7a', 2), (35, 50, '#8a90e0', 3), (16, 20, '#eef2ff', 4)):
        layer = stars(rng, n, col, size)
        a.add(f'<g>{layer}<g transform="translate({W} 0)">{layer}</g><animateTransform attributeName="transform" '
              f'type="translate" from="0 0" to="-{W} 0" dur="{speed}s" repeatCount="indefinite"/></g>')
    a.add(twinkles(rng, 6, a))
    planet = a.radial('planet', [(0, '#ffc08a'), (0.55, '#d0566a'), (1, '#3d1a4a')], 0.35, 0.35, 0.7)
    a.add(f'<circle cx="1050" cy="80" r="130" fill="{planet}"/>')
    for cx, cy, r in ((1000, 60, 16), (1080, 120, 22), (1110, 40, 10), (1020, 150, 12)):
        a.add(f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="#000" opacity="0.14"/><circle cx="{cx - 2}" cy="{cy - 2}" r="{r - 3}" '
              f'fill="#fff" opacity="0.06"/>')
    ship = sprite(SHIP, {'c': '#0f766e', 'w': '#e6fffb', 'b': '#2563eb', 'a': '#9ca3af', 'y': '#fbbf24', 'r': '#f87171',
                         's': '#5eead4'}, 0, 0, 5)
    flame = (f'<path d="M-6 30 L-46 25 L-30 33 L-56 33 L-30 36 L-46 41 L-6 36Z" fill="#fbbf24"{a.glow}>'
             f'<animate attributeName="d" values="M-6 30 L-46 25 L-30 33 L-56 33 L-30 36 L-46 41 L-6 36Z;'
             f'M-6 30 L-34 27 L-24 33 L-40 33 L-24 36 L-34 39 L-6 36Z;M-6 30 L-46 25 L-30 33 L-56 33 L-30 36 L-46 41 L-6 36Z" '
             f'dur="0.2s" repeatCount="indefinite"/></path>')
    a.add(f'<g transform="translate(140 175)">{flame}{ship}<animateTransform attributeName="transform" type="translate" '
          f'values="140 175;140 195;140 160;140 175" dur="3s" repeatCount="indefinite"/></g>')
    for k in range(5):
        a.add(f'<rect x="270" y="205" width="34" height="6" rx="3" fill="#fde68a"{a.glow}><animate attributeName="x" from="270" '
              f'to="1220" dur="1.4s" begin="{k * 0.28:.2f}s" repeatCount="indefinite"/></rect>')
    colours = [('#4ade80', '#bbf7d0'), ('#c084fc', '#f5d0fe'), ('#f472b6', '#fecdd3')]
    for r in range(3):
        for c in range(5):
            inv = sprite(INVADER, {'g': colours[r][0], 'h': OUTLINE, 'e': colours[r][1]}, 0, 0, 5)
            x, y = 640 + c * 96, 84 + r * 80
            a.add(f'<g transform="translate({x} {y})">{inv}<animateTransform attributeName="transform" type="translate" '
                  f'values="{x} {y};{x - 30} {y + 18};{x} {y + 36};{x + 30} {y + 18};{x} {y}" dur="5s" '
                  f'begin="{-c * 0.3:.1f}s" repeatCount="indefinite"/></g>')
    a.add(f'<g opacity="0"{a.glow}><circle cx="880" cy="178" r="26" fill="#fde68a"/><circle cx="880" cy="178" r="14" fill="#fff"/>'
          + ''.join(f'<rect x="{880 + 40 * math.cos(t):.0f}" y="{178 + 40 * math.sin(t):.0f}" width="7" height="7" fill="#f59e0b"/>'
                    for t in [k * 0.785 for k in range(8)])
          + '<animate attributeName="opacity" values="0;1;0;0" keyTimes="0;0.06;0.3;1" dur="2.4s" repeatCount="indefinite"/></g>')
    a.text(32, 46, '1UP  004200', 24, '#fbbf24', glow=True)
    a.text(W / 2, 46, 'HI  050000', 24, '#eef2ff', 'middle', glow=True)
    for k in range(3):
        a.add(f'<g transform="translate({32 + k * 56} 352) scale(0.4)">{ship}</g>')
    return a


# ---------------------------------------------------------------- first-person (a real ray caster)
MAP = ['1111111111111111', '1..............1', '1..22..2222....1', '1..2...........1', '1..2...33.33...1',
       '1......3...3...1', '1..1...3...3...1', '1..1...33333...1', '1..............1', '1....2222..44..1',
       '1..............1', '1111111111111111']
WALLS = {'1': '#9a7350', '2': '#6a7ea0', '3': '#8a4f4f', '4': '#4f8a62'}
IMP = ['....h...h....', '....hrrrh....', '...rrrrrrr...', '...ryrrryr...', '...rrrrrrr...', '..rrrmmmrrr..', '.rr.rrrrr.rr.',
       '.r..rrrrr..r.', 'rr..rrrrr..rr', '....rr.rr....', '....rr.rr....', '...rrr.rrr...']


def first_person(a=None):
    a = a or Art('fps', 'A first-person view down a lit corridor of a maze, drawn column by column by a ray caster')
    a.rect(0, 0, W, H / 2, a.gradient('ceil', [(0, '#100e16'), (1, '#3a3442')]))
    a.rect(0, H / 2, W, H / 2, a.gradient('floor', [(0, '#2a2620'), (1, '#6a5a44')]))
    for k in range(14):
        y = H / 2 + (k / 14) ** 2 * (H / 2)
        a.rect(0, y, W, 1.5, '#000', ' opacity="0.25"')
    px, py, ang, fov = 1.5, 8.4, -0.12, math.radians(70)
    cols, cw = 240, W / 240
    for i in range(cols):
        ra = ang + (i / cols - 0.5) * fov
        dx, dy = math.cos(ra), math.sin(ra)
        mx, my = int(px), int(py)
        sx, sy = (1 if dx > 0 else -1), (1 if dy > 0 else -1)
        ddx, ddy = abs(1 / dx) if dx else 1e9, abs(1 / dy) if dy else 1e9
        tx = ((mx + 1 - px) if dx > 0 else (px - mx)) * ddx
        ty = ((my + 1 - py) if dy > 0 else (py - my)) * ddy
        side = 0
        while True:
            if tx < ty:
                tx += ddx
                mx += sx
                side = 0
            else:
                ty += ddy
                my += sy
                side = 1
            if MAP[my][mx] != '.':
                break
        raw = tx - ddx if side == 0 else ty - ddy
        dist = raw * math.cos(ra - ang)
        hit = (py + raw * dy) if side == 0 else (px + raw * dx)
        u = hit - math.floor(hit)
        h = min(H * 1.8, H * 0.9 / max(dist, 0.05))
        k = max(0.2, min(1.05, 1.6 / (0.5 + dist * 0.42))) * (0.7 if side else 1.0)
        top = H / 2 - h / 2
        a.rect(i * cw, top, cw + 0.6, h, shade(WALLS[MAP[my][mx]], k * (0.9 if int(u * 4) % 2 else 1.0)))
        if int(u * 16) % 4 == 0:
            a.rect(i * cw, top, cw + 0.6, h, '#000', ' opacity="0.18"')
        for r in range(1, 6):
            a.rect(i * cw, top + h * r / 6, cw + 0.6, max(1, h / 90), '#000', ' opacity="0.35"')
        if dist < 6:
            a.rect(i * cw, top, cw + 0.6, h * 0.08, '#fff', f' opacity="{0.05 * (6 - dist) / 6:.3f}"')
    for lx in (520, 700):
        a.add(f'<ellipse cx="{lx}" cy="100" rx="40" ry="10" fill="#fde68a" opacity="0.5"{a.glow}/>')
    imp = sprite(IMP, {'h': '#e5e7eb', 'r': '#b45309', 'y': '#fde047', 'm': '#7f1d1d'}, 0, 0, 7)
    a.add(shadow(645, 268, 46, 8))
    a.add(f'<g transform="translate(600 180)">{imp}<animateTransform attributeName="transform" type="translate" '
          f'values="600 180;606 178;600 180;594 178;600 180" dur="1.4s" repeatCount="indefinite"/></g>')
    gun = ['.....dd.....', '....dmmd....', '....dmmd....', '...dmllmd...', '...dmllmd...', '..ddmllmdd..', '.dmmmllmmmd.',
           'dmmlllllllmd', 'dmllllllllmd', 'dmllllllllmd']
    g = sprite(gun, {'d': '#1f1f24', 'm': '#4b4b56', 'l': '#8b8b98'}, 0, 0, 11)
    flash = (f'<g opacity="0"{a.glow}><circle cx="66" cy="-10" r="26" fill="#fde68a"/><circle cx="66" cy="-10" r="12" fill="#fff"/>'
             f'<animate attributeName="opacity" values="0;1;0;0" keyTimes="0;0.04;0.12;1" dur="1.6s" repeatCount="indefinite"/></g>')
    a.add(f'<g transform="translate(534 286)">{flash}{g}<animateTransform attributeName="transform" type="translate" '
          f'values="534 286;540 280;534 286;528 280;534 286" dur="1.2s" repeatCount="indefinite"/></g>')
    a.add('<path d="M590 200h20M600 190v20" stroke="#ecfeff" stroke-width="2" opacity="0.85"/>')
    a.add(bevel(0, H - 40, W, 40, '#2a221a', 3))
    for k, (lab, val, col) in enumerate((('HEALTH', '100%', '#f87171'), ('ARMOR', '45%', '#60a5fa'), ('AMMO', '50', '#f59e0b'))):
        a.text(40 + k * 240, H - 13, f'{lab} {val}', 20, col, glow=True)
    return a


# ---------------------------------------------------------------- racing (pseudo-3D road)
CAR = ['......kkkkkkkkkkkkkk......', '.....kggggggggggggggggk.....'[:26], '....kgggggggggggggggggk....'[:26],
       '...rrrrrrrrrrrrrrrrrrrr...', '..rrhhhhhhhhhhhhhhhhhhrr..', '.rrrrrrrrrrrrrrrrrrrrrrrr.', 'rrttttrrrrrrrrrrrrrrrttttr'[:26],
       'rrttttrrrppppppppprrrttttr'[:26], 'rrrrrrrrrrrrrrrrrrrrrrrrrr', 'rddrrrrrrrrrrrrrrrrrrrrddr', 'kkkkk................kkkkk',
       'kkkkk................kkkkk']
PALM = ['....gg.gg....', '..ggggggggg..', '.gg..ggg..gg.', 'g....gtg....g', '.....btb.....', '.....btb.....', '.....btb.....',
        '.....btb.....', '.....btb.....', '....bbtbb....']


def racing(a=None):
    a = a or Art('race', 'A racing game seen from behind the car, with the road curving away to a sunset')
    hz = 170
    a.rect(0, 0, W, hz, a.gradient('sky', [(0, '#160f3a'), (0.55, '#a33d5e'), (1, '#fbbf24')]))
    a.add(f'<circle cx="760" cy="{hz - 10}" r="100" fill="{a.radial("sun", [(0, "#fff7cc"), (1, "#fbbf24")])}"{a.glow}/>')
    for k in range(6):
        a.rect(650, hz - 76 + k * 13, 220, 4 + k, '#a33d5e')
    for cx, cy, w in ((200, 60, 160), (420, 40, 110), (1000, 70, 180)):
        a.add(f'<ellipse cx="{cx}" cy="{cy}" rx="{w}" ry="14" fill="#f9a8d4" opacity="0.35"/>'
              f'<ellipse cx="{cx + 30}" cy="{cy - 8}" rx="{w * 0.5:.0f}" ry="10" fill="#fce7f3" opacity="0.3"/>')
    rng = random.Random(4)
    for layer, col, amp in ((0, '#3b1d52', 70), (1, '#24123a', 40)):
        pts = ' '.join(f'{x},{hz - 20 - amp * abs(math.sin(x / (160 - layer * 40) + layer)) - rng.uniform(0, 14):.0f}'
                       for x in range(0, W + 41, 40))
        a.add(f'<polygon points="0,{hz} {pts} {W},{hz}" fill="{col}"/>')
    a.rect(0, hz - 2, W, 3, '#fbbf24', ' opacity="0.6"')
    bodies = []
    for f in range(4):
        rows, props = [], []
        last = None
        for y in range(hz + 1, H + 1, 3):
            p = (y - hz) / (H - hz)
            half = 18 + 560 * p
            cx = W / 2 + 300 * (1 - p) ** 2.2 - 40 * p
            z = 1 / (p + 0.015)
            seg = int(z * 1.1 + f * 0.5)
            stripe = seg % 2
            rows.append(f'<rect x="0" y="{y}" width="{W}" height="3.4" fill="{"#2f855a" if stripe else "#276749"}"/>'
                        f'<rect x="{cx - half * 1.1:.1f}" y="{y}" width="{half * 2.2:.1f}" height="3.4" '
                        f'fill="{"#f87171" if stripe else "#f1f5f9"}"/>'
                        f'<rect x="{cx - half:.1f}" y="{y}" width="{half * 2:.1f}" height="3.4" fill="{"#4d4d5a" if stripe else "#46465a"}"/>')
            if stripe:
                rows.append(f'<rect x="{cx - half * 0.025:.1f}" y="{y}" width="{half * 0.05:.1f}" height="3.4" fill="#f1f5f9"/>')
            if last is not None and seg != last and seg % 6 == 0 and 0.08 < p < 0.9:
                props.append((y, p, cx, half))
            last = seg
        for y, p, cx, half in props:
            s = 1.2 + 9 * p
            for side in (-1, 1):
                rows.append(sprite(PALM, {'g': '#15803d', 't': '#5a3e22', 'b': '#3f2a16'},
                                   cx + side * half * 1.5 - 6.5 * s, y - 10 * s, s))
        bodies.append(''.join(rows))
    a.add(frames(bodies, 0.36))
    rival = sprite(CAR, {'r': '#2563eb', 'k': '#111827', 'g': '#93c5fd', 'h': '#1e3a8a', 't': '#fca5a5', 'p': '#e5e7eb',
                         'd': '#1e3a8a'}, 0, 0, 3)
    a.add(f'<g transform="translate(686 192)">{rival}<animateTransform attributeName="transform" type="translate" '
          f'values="686 192;700 191;686 192" dur="2s" repeatCount="indefinite"/></g>')
    car = sprite(CAR, {'r': '#dc2626', 'k': '#111827', 'g': '#7dd3fc', 'h': '#7f1d1d', 't': '#fecaca', 'p': '#f1f5f9',
                       'd': '#7f1d1d'}, 0, 0, 10)
    lights = (f'<rect x="20" y="62" width="40" height="18" fill="#f87171" opacity="0.8"{a.glow}/>'
              f'<rect x="200" y="62" width="40" height="18" fill="#f87171" opacity="0.8"{a.glow}/>')
    a.add(shadow(600, 392, 150, 14, 0.5))
    a.add(f'<g transform="translate(470 270)">{car}{lights}<animateTransform attributeName="transform" type="translate" '
          f'values="470 270;465 271;470 270;475 271;470 270" dur="0.5s" repeatCount="indefinite"/></g>')
    a.text(32, 46, 'TIME 58', 26, '#fde68a', glow=True)
    a.text(W - 32, 46, '183 km/h', 26, '#fde68a', 'end', glow=True)
    return a


# ---------------------------------------------------------------- vehicle and space simulations
def space_sim(a=None):
    a = a or Art('space', 'A ship flying past a ringed planet and its moon, with a radar scanner below')
    rng = random.Random(7)
    a.rect(0, 0, W, H, '#02030a')
    for cx, cy, rx, col in ((900, 120, 340, '#4c1d95'), (1050, 300, 220, '#0e7490'), (200, 60, 200, '#1e3a8a')):
        a.add(f'<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{rx * 0.5:.0f}" fill="{col}" opacity="0.28"{a.blur}/>')
    a.add(stars(rng, 220, '#8a93c8', 2))
    a.add(stars(rng, 40, '#ffffff', 3))
    a.add(twinkles(rng, 8, a))
    cx, cy, r, tilt = 330, 230, 150, -14
    rings = ((300, 62, 16, '#d6b48a'), (262, 54, 8, '#a3825a'), (330, 68, 6, '#e8d2b0'))
    # The far half of the ring passes behind the planet.
    for rx, ry, w, col in rings:
        a.add(f'<path d="M{cx - rx} {cy} A{rx} {ry} 0 0 1 {cx + rx} {cy}" fill="none" stroke="{col}" stroke-width="{w}" '
              f'opacity="0.5" transform="rotate({tilt} {cx} {cy})"/>')
    a.add(f'<circle cx="{cx}" cy="{cy}" r="{r + 14}" fill="#60a5fa" opacity="0.35"{a.blur}/>')
    body = a.radial('planet', [(0, '#7ff0e0'), (0.45, '#14b8a6'), (0.85, '#0b4f4a'), (1, '#062a28')], 0.68, 0.32, 0.8)
    a.add(f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="{body}"/>')
    cid = a.id('pclip')
    a.defs.append(f'<clipPath id="{cid}"><circle cx="{cx}" cy="{cy}" r="{r}"/></clipPath>')
    bands = ''.join(f'<path d="M{cx - r} {cy - 90 + k * 30} C {cx - 60} {cy - 110 + k * 32} {cx + 60} {cy - 70 + k * 30} '
                    f'{cx + r} {cy - 92 + k * 31}" stroke="{["#a7f3d0", "#0f766e", "#5eead4"][k % 3]}" '
                    f'stroke-width="{10 + k % 3 * 6}" fill="none" opacity="0.22"/>' for k in range(7))
    storm = f'<ellipse cx="{cx + 50}" cy="{cy + 34}" rx="30" ry="14" fill="#fde68a" opacity="0.35"/>'
    a.add(f'<g clip-path="url(#{cid})">{bands}{storm}<circle cx="{cx - 120}" cy="{cy + 110}" r="{r * 1.25:.0f}" fill="#000" '
          f'opacity="0.55"{a.blur}/></g>')
    # The near half passes in front, with its shadow on the planet.
    a.add(f'<g clip-path="url(#{cid})"><path d="M{cx - 300} {cy} A300 62 0 0 0 {cx + 300} {cy}" fill="none" stroke="#000" '
          f'stroke-width="18" opacity="0.3" transform="rotate({tilt} {cx} {cy}) translate(0 -14)"/></g>')
    for rx, ry, w, col in rings:
        a.add(f'<path d="M{cx - rx} {cy} A{rx} {ry} 0 0 0 {cx + rx} {cy}" fill="none" stroke="{col}" stroke-width="{w}" '
              f'opacity="0.9" transform="rotate({tilt} {cx} {cy})"/>')
    moon = a.radial('moon', [(0, '#f1f5f9'), (0.7, '#94a3b8'), (1, '#334155')], 0.65, 0.35, 0.7)
    a.add(f'<circle cx="620" cy="80" r="34" fill="{moon}"/>'
          + ''.join(f'<circle cx="{x}" cy="{y}" r="{rr}" fill="#475569" opacity="0.5"/>' for x, y, rr in
                    ((610, 72, 7), (630, 92, 5), (602, 92, 4))))
    ship = ('<path d="M0 0 L120 -26 L176 0 L120 26 Z" fill="#334155" stroke="#e2e8f0" stroke-width="2"/>'
            '<path d="M0 0 L120 -26 L104 0 Z" fill="#475569"/><path d="M0 0 L104 0 L120 26Z" fill="#1e293b"/>'
            '<path d="M120 -26 L176 0 L120 26 L104 0Z" fill="#64748b"/>'
            '<path d="M60 -10 L90 -14 L96 -4 L64 -2Z" fill="#7dd3fc"/>'
            '<path d="M40 -40 L104 -12 L80 -8Z" fill="#94a3b8" stroke="#e2e8f0" stroke-width="1.5"/>'
            '<path d="M40 40 L104 12 L80 8Z" fill="#475569" stroke="#e2e8f0" stroke-width="1.5"/>'
            f'<ellipse cx="-4" cy="0" rx="14" ry="10" fill="#f59e0b"{a.glow}><animate attributeName="rx" values="14;22;14" '
            f'dur="0.3s" repeatCount="indefinite"/></ellipse>'
            '<circle cx="176" cy="0" r="3" fill="#f87171"><animate attributeName="opacity" values="1;0;1" dur="1s" '
            'repeatCount="indefinite"/></circle>')
    a.add(f'<g><g transform="rotate(-8)">{ship}</g><animateTransform attributeName="transform" type="translate" '
          f'values="690 150;712 140;690 150;668 160;690 150" dur="8s" repeatCount="indefinite"/></g>')
    a.add('<ellipse cx="950" cy="335" rx="220" ry="50" fill="#04221f" stroke="#2dd4bf" stroke-width="2" opacity="0.95"/>'
          '<ellipse cx="950" cy="335" rx="150" ry="32" fill="none" stroke="#2dd4bf" opacity="0.4"/>'
          '<ellipse cx="950" cy="335" rx="75" ry="16" fill="none" stroke="#2dd4bf" opacity="0.3"/>'
          '<path d="M730 335H1170M950 285V385" stroke="#2dd4bf" opacity="0.3"/>')
    for x, y, h, col in ((870, 335, -26, '#f59e0b'), (1010, 345, -18, '#f87171'), (1060, 322, 14, '#4ade80'),
                         (910, 356, -10, '#e2e8f0')):
        a.add(f'<path d="M{x} {y}V{y + h}" stroke="{col}" stroke-width="2"/><rect x="{x - 4}" y="{y + h - 4}" width="8" height="8" '
              f'fill="{col}"{a.glow}/>')
    a.text(1160, 48, 'ALT 412 km', 22, '#5eead4', 'end', glow=True)
    a.text(1160, 80, 'SPEED 7.66 km/s', 22, '#5eead4', 'end', glow=True)
    return a


# ---------------------------------------------------------------- city building and management
PERSON = ['.h.', 'sss', '.p.', 'p.p']


def management(a=None):
    a = a or Art('city', 'An isometric town of houses, fields, trees and a pond, with people walking along the roads')
    a.rect(0, 0, W, H, a.gradient('sky', [(0, '#1e3a5f'), (1, '#5b8db0')]))
    tw, th, ox, oy = 80, 40, W / 2, 10
    rng = random.Random(3)
    grid = 13
    roads = {(i, 6) for i in range(grid)} | {(6, j) for j in range(grid)}
    houses = {(2, 3), (3, 3), (4, 2), (8, 3), (9, 4), (3, 8), (4, 9), (8, 8), (9, 9), (10, 8), (7, 4), (4, 7)}
    fields = {(i, j) for i in range(9, 12) for j in range(0, 3)}
    pond = {(1, 10), (2, 10), (1, 11), (2, 11)}
    trees = {(rng.randrange(grid), rng.randrange(grid)) for _ in range(30)} - roads - houses - fields - pond

    def iso(i, j):
        return ox + (i - j) * tw / 2, oy + (i + j) * th / 2

    def order():
        for s in range(2 * grid):
            for i in range(grid):
                j = s - i
                if 0 <= j < grid:
                    yield i, j
    for i, j in order():
        x, y = iso(i, j)
        if (i, j) in roads:
            col = '#9a8a70'
        elif (i, j) in fields:
            col = '#d4a72c'
        elif (i, j) in pond:
            col = '#2563eb'
        else:
            col = ['#3f7d3a', '#468a40', '#3a7535'][(i * 7 + j * 3) % 3]
        a.add(f'<path d="M{x} {y}l{tw / 2} {th / 2}l{-tw / 2} {th / 2}l{-tw / 2} {-th / 2}z" fill="{col}" stroke="#00000030"/>')
        if (i, j) in fields:
            for k in range(1, 4):
                a.add(f'<path d="M{x - tw / 2 + k * tw / 8} {y + th / 2 - k * th / 8}l{tw / 2} {th / 2}" stroke="#a8861f" '
                      f'stroke-width="2"/>')
        if (i, j) in pond:
            a.add(f'<path d="M{x - 14} {y + th / 2}h28" stroke="#93c5fd" stroke-width="2" opacity="0.6">'
                  f'<animate attributeName="opacity" values="0.2;0.8;0.2" dur="{2 + (i + j) % 3}s" repeatCount="indefinite"/></path>')
        if (i, j) in roads and i != 6:
            a.add(f'<path d="M{x - 8} {y + th / 2 - 4}l16 8" stroke="#f1f5f9" stroke-width="2" opacity="0.5"/>')
    for i, j in order():
        x, y = iso(i, j)
        cy = y + th / 2
        if (i, j) in houses:
            wall = ['#e3d2b0', '#d4b892', '#c9c9c9'][(i + j) % 3]
            hgt = 30
            a.add(f'<path d="M{x} {cy + 14}l40 -20l20 10l-40 20z" fill="#000" opacity="0.2"/>')
            a.add(f'<path d="M{x - 26} {cy}l26 13l26 -13v{-hgt}l-26 13l-26 -13z" fill="{shade(wall, 0.78)}"/>'
                  f'<path d="M{x} {cy + 13}l26 -13v{-hgt}l-26 13z" fill="{wall}"/>'
                  f'<path d="M{x - 18} {cy - 10}l8 4v10l-8 -4z" fill="#5a3e22"/>'
                  f'<path d="M{x + 8} {cy - 6}l8 -4v8l-8 4z" fill="#fde68a"/>'
                  f'<path d="M{x - 30} {cy - hgt}l30 -22l30 22l-30 15z" fill="#8f3527"/>'
                  f'<path d="M{x} {cy - hgt - 22}l30 22l-30 15z" fill="#c2552f"/>'
                  f'<rect x="{x + 12}" y="{cy - hgt - 20}" width="6" height="12" fill="#5a3e22"/>')
            if (i + j) % 3 == 0:
                a.add(f'<circle cx="{x + 15}" cy="{cy - hgt - 26}" r="5" fill="#e5e7eb" opacity="0.6">'
                      f'<animate attributeName="cy" values="{cy - hgt - 24};{cy - hgt - 60}" dur="3s" repeatCount="indefinite"/>'
                      f'<animate attributeName="opacity" values="0.6;0" dur="3s" repeatCount="indefinite"/>'
                      f'<animate attributeName="r" values="4;10" dur="3s" repeatCount="indefinite"/></circle>')
        elif (i, j) in trees:
            a.add(shadow(x + 8, cy + 2, 14, 5, 0.25))
            a.add(f'<rect x="{x - 2}" y="{cy - 14}" width="5" height="14" fill="#5a3e22"/>'
                  f'<circle cx="{x}" cy="{cy - 24}" r="14" fill="#265a22"/><circle cx="{x - 4}" cy="{cy - 28}" r="9" fill="#3f8a36"/>'
                  f'<circle cx="{x - 6}" cy="{cy - 31}" r="4" fill="#6abf5a"/>')
    path = ' '.join(f'{"M" if k == 0 else "L"}{iso(i, 6)[0]:.0f} {iso(i, 6)[1] + th / 2 - 8:.0f}' for k, i in enumerate(range(grid)))
    path2 = ' '.join(f'{"M" if k == 0 else "L"}{iso(6, j)[0]:.0f} {iso(6, j)[1] + th / 2 - 8:.0f}' for k, j in enumerate(range(grid)))
    for k, (p, col) in enumerate(((path, '#f59e0b'), (path, '#60a5fa'), (path2, '#f472b6'), (path2, '#e2e8f0'),
                                  (path, '#4ade80'), (path2, '#c084fc'))):
        person = sprite(PERSON, {'h': '#3a2210', 's': col, 'p': '#1f2937'}, -4.5, -6, 3)
        a.add(f'<g>{person}<animateMotion dur="{16 + k * 3}s" begin="{-k * 4}s" repeatCount="indefinite" path="{p}"/></g>')
    return a


# ---------------------------------------------------------------- strategy
SOLDIER = ['..hh..', '.hffh.', '..ff..', 'cbbbbc', 'cbwwbc', '.bbbb.', '.b..b.', '.k..k.']
CASTLE = ['s.s.s.....s.s.s', 'sssss.....sssss', 'swsws.....swsws', 'sssssssssssssss', 'ssswsssssssswss', 'sssssssddssssss',
          'sssssssddssssss']


def strategy(a=None):
    a = a or Art('rts', 'A strategy map: two armies and their castles on grass, forest and a river, a selected group and '
                        'fog of war')
    rng = random.Random(9)
    t = 24
    cols, rows = W // t, H // t + 1
    for r in range(rows):
        for c in range(cols):
            rc = 25 + 6 * math.sin(r / 3)
            river = abs(c - rc) < 1.5
            shore = abs(c - rc) < 2.4 and not river
            n = math.sin(c / 5) + math.cos(r / 3.5) + rng.uniform(-0.6, 0.6)
            if river:
                col = '#1d5fb4'
            elif shore:
                col = '#c8b27a'
            elif n < -1.1:
                col = '#8a7550'
            else:
                col = ['#3a7a34', '#418540', '#367331'][(r * 5 + c) % 3]
            a.rect(c * t, r * t, t + 0.5, t + 0.5, col)
            if river and (r + c) % 3 == 0:
                a.add(f'<path d="M{c * t + 4} {r * t + 12}q4 -4 8 0t8 0" stroke="#93c5fd" fill="none" stroke-width="2" opacity="0.6">'
                      f'<animate attributeName="opacity" values="0.2;0.8;0.2" dur="{2 + (r % 3)}s" repeatCount="indefinite"/></path>')
            elif not river and not shore and n > 1.15:
                a.add(f'<circle cx="{c * t + 13}" cy="{r * t + 14}" r="11" fill="#163d1a"/>'
                      f'<circle cx="{c * t + 11}" cy="{r * t + 11}" r="9" fill="#24602a"/>'
                      f'<circle cx="{c * t + 9}" cy="{r * t + 8}" r="4" fill="#3f8a36"/>')
            elif col.startswith('#3') and rng.random() < 0.08:
                a.add(f'<path d="M{c * t + 8} {r * t + 18}l2 -6l2 6m2 0l2 -5l2 5" stroke="#6abf5a" fill="none" stroke-width="1.5"/>')
    a.add(bevel(24 * t, 7 * t - 4, 4 * t, t + 8, '#8a6a3a', 3))
    for team, col, cx, cy, kx in (('b', '#2563eb', 250, 160, 60), ('r', '#dc2626', 880, 200, 1050)):
        a.add(shadow(kx + 44, 120, 60, 10))
        a.add(sprite(CASTLE, {'s': '#9ca3af', 'w': '#1f2937', 'd': '#5a3e22'}, kx, 60, 6))
        a.add(f'<path d="M{kx + 44} 30V60" stroke="#e5e7eb" stroke-width="2"/><path d="M{kx + 46} 30h22l-6 7l6 7h-22z" '
              f'fill="{col}"><animate attributeName="d" values="M{kx + 46} 30h22l-6 7l6 7h-22z;M{kx + 46} 31h20l-4 6l6 8h-22z;'
              f'M{kx + 46} 30h22l-6 7l6 7h-22z" dur="1s" repeatCount="indefinite"/></path>')
        for k in range(9):
            x, y = cx + (k % 3) * 36, cy + (k // 3) * 38
            a.add(shadow(x + 15, y + 40, 14, 4, 0.3))
            if team == 'b':
                a.add(f'<ellipse cx="{x + 15}" cy="{y + 40}" rx="17" ry="6" fill="none" stroke="#4ade80" stroke-width="2"/>')
            a.add(sprite(SOLDIER, {'h': '#3a2a1a', 'f': '#e5c09a', 'b': col, 'c': shade(col, 0.6), 'w': '#e5e7eb',
                                   'k': '#1f2937'}, x, y, 5))
    a.add('<rect x="240" y="148" width="132" height="132" fill="#4ade80" fill-opacity="0.06" stroke="#4ade80" stroke-width="2" '
          'stroke-dasharray="8 6"><animate attributeName="stroke-dashoffset" from="0" to="28" dur="1s" repeatCount="indefinite"/>'
          '</rect>')
    a.add(f'<path d="M380 210 C 520 120 640 140 760 220" fill="none" stroke="#fde68a" stroke-width="4" stroke-dasharray="14 10"'
          f'{a.glow}><animate attributeName="stroke-dashoffset" from="48" to="0" dur="1s" repeatCount="indefinite"/></path>'
          '<path d="M748 204l18 18l-24 6z" fill="#fde68a"/>')
    a.add(f'<path d="M1010 0 Q 960 120 1020 200 T 1000 400 H{W} V0Z" fill="#000" opacity="0.6"/>')
    a.add(bevel(0, H - 44, W, 44, '#1f2937', 3))
    for k, (val, col) in enumerate(((1500, '#fbbf24'), (820, '#a16207'), (1210, '#4ade80'))):
        a.add(f'<circle cx="{40 + k * 190}" cy="{H - 22}" r="9" fill="{col}"/>')
        a.text(58 + k * 190, H - 15, f'{val}', 20, '#f1f5f9')
    a.add(bevel(1040, H - 120, 140, 100, '#111827', 3))
    a.rect(1050, H - 110, 120, 80, '#24602a')
    a.rect(1100, H - 110, 10, 80, '#1d5fb4')
    a.add('<rect x="1060" y="300" width="4" height="4" fill="#60a5fa"/><rect x="1150" y="310" width="4" height="4" fill="#f87171"/>'
          '<rect x="1056" y="296" width="30" height="20" fill="none" stroke="#f1f5f9" stroke-width="1.5"/>')
    return a


# ---------------------------------------------------------------- RPG
HEROES = [
    ('ARIN', ['...hhhh...', '..hhhhhh..', '..hffffh..', '..ffeffe..', '..ffffff..', '.aammmmaa.', 'aammmmmmaa', 'a.mmmmmm.a',
              'a.mmmmmm.a', '..mm..mm..', '..bb..bb..'], {'h': '#9ca3af', 'f': '#f2c79e', 'e': '#1f2937', 'a': '#d1d5db',
                                                       'm': '#64748b', 'b': '#3f2a1a'}, 0.9, 0.2),
    ('MIRA', ['....pp....', '...pppp...', '..pppppp..', '.pppppppp.', '..ffffff..', '..feffef..', '..ffffff..', '.pppppppp.',
              '.pppyyppp.', '..pppppp..', '..pp..pp..'], {'p': '#7c3aed', 'f': '#f2c79e', 'e': '#1f2937', 'y': '#fbbf24'}, 0.5, 0.8),
    ('TOV', ['..gggggg..', '.gggggggg.', '..ffffff..', '..feffef..', '..ffffff..', '..gggggg..', '.gggbbggg.', '.gggggggg.',
             '..gggggg..', '..gg..gg..', '..bb..bb..'], {'g': '#15803d', 'f': '#f2c79e', 'e': '#1f2937', 'b': '#3f2a1a'}, 0.75, 0.4),
    ('SELA', ['..wwwwww..', '.wwwwwwww.', '..ffffff..', '..feffef..', '..ffffff..', '.wwwyywww.', '.wwyyyyww.', '.wwwyywww.',
              '.wwwwwwww.', '..ww..ww..', '..bb..bb..'], {'w': '#f1f5f9', 'f': '#f2c79e', 'e': '#1f2937', 'y': '#f59e0b',
                                                       'b': '#3f2a1a'}, 0.6, 1.0)]
DRAGON = ['...............ww.....', '..............wwww....', '.....rr......wwwwww...', '....rrrr....wwwwwwww..',
          '...rrerrr..wwwwwwwww..', '..rrrrrrrrrrrwwwwww...', 'rrrrrrrrrrrrrrrrww....', '.ttrrrrrrrrrrrrrrr....',
          '..ttbbbbbbbbbrrrrrrr..', '....bbbbbbbbbrrrrrrrrr', '....rrr....rrr.....rrr', '....rrr....rrr........',
          '...rrrr...rrrr........']


def rpg(a=None):
    a = a or Art('rpg', 'An RPG battle in a torch-lit dungeon: a party of four faces a dragon, with health bars below')
    a.rect(0, 0, W, H, '#14131a')
    t = 40
    for r in range(3):
        for c in range(W // t + 1):
            a.add(bevel(c * t - (20 if r % 2 else 0), r * 30, t - 2, 28, shade('#3b3540', 0.9 + ((r * 7 + c * 13) % 4) * 0.06), 2))
    for r in range(3, H // t + 1):
        for c in range(W // t):
            a.add(bevel(c * t, r * t - 30, t - 2, t - 2, shade('#4b4652', 0.82 + ((r * 7 + c * 13) % 5) * 0.06), 3))
    for x in (160, 600, 1040):
        a.add(f'<ellipse cx="{x + 7}" cy="70" rx="90" ry="60" fill="#f59e0b" opacity="0.12"{a.blur}/>')
        a.add(bevel(x, 54, 14, 40, '#6b4a2a', 2))
        a.add(f'<path d="M{x + 7} 22 C{x + 22} 40 {x + 18} 54 {x + 7} 56 C{x - 4} 54 {x - 8} 40 {x + 7} 22Z" fill="#f59e0b"{a.glow}>'
              f'<animate attributeName="d" values="M{x + 7} 22 C{x + 22} 40 {x + 18} 54 {x + 7} 56 C{x - 4} 54 {x - 8} 40 {x + 7} 22Z;'
              f'M{x + 9} 26 C{x + 20} 40 {x + 18} 54 {x + 7} 56 C{x - 4} 54 {x - 6} 42 {x + 9} 26Z;'
              f'M{x + 7} 22 C{x + 22} 40 {x + 18} 54 {x + 7} 56 C{x - 4} 54 {x - 8} 40 {x + 7} 22Z" dur="0.6s" '
              f'repeatCount="indefinite"/></path><circle cx="{x + 7}" cy="46" r="6" fill="#fef3c7"/>')
    for k, (name, rows, pal, hp, mp) in enumerate(HEROES):
        x, y = 110 + (k % 2) * 110, 120 + k * 36
        a.add(shadow(x + 30, y + 74, 30, 7))
        a.add(f'<g transform="translate({x} {y})">{sprite(rows, pal, 0, 0, 6)}<animateTransform attributeName="transform" '
              f'type="translate" values="{x} {y};{x} {y - 4};{x} {y}" dur="{0.8 + k * 0.1:.1f}s" repeatCount="indefinite"/></g>')
    a.add(shadow(910, 290, 170, 16, 0.45))
    a.add(sprite(DRAGON, {'r': '#b91c1c', 'e': '#fde047', 't': '#f8fafc', 'b': '#f59e0b', 'w': '#7f1d1d'}, 740, 80, 14))
    a.add(f'<g opacity="0"{a.glow}>' + ''.join(f'<circle cx="{720 - k * 60}" cy="{200 + (k % 2) * 10}" r="{26 - k * 3}" '
                                              f'fill="{["#fde68a", "#f59e0b", "#f97316", "#dc2626"][k % 4]}"/>' for k in range(6))
          + '<animate attributeName="opacity" values="0;0;1;1;0" keyTimes="0;0.5;0.55;0.85;1" dur="3s" repeatCount="indefinite"/></g>')
    a.add('<g><text x="300" y="130" font-family="ui-monospace, Menlo, monospace" font-size="36" font-weight="800" fill="#fde68a" '
          'stroke="#0a0a12" stroke-width="5" paint-order="stroke" text-anchor="middle">24<animate attributeName="y" '
          'values="160;110" dur="3s" repeatCount="indefinite"/><animate attributeName="opacity" values="0;0;1;0" '
          'keyTimes="0;0.6;0.7;1" dur="3s" repeatCount="indefinite"/></text></g>')
    a.add(f'<rect x="20" y="298" width="1160" height="92" rx="10" fill="{a.gradient("ui", [(0, "#2a3a8a"), (1, "#141c4a")])}" '
          f'stroke="#e2e8f0" stroke-width="4"/><rect x="28" y="306" width="1144" height="76" rx="6" fill="none" stroke="#6b7bd6" '
          f'stroke-width="2"/>')
    for k, (name, _, _, hp, mp) in enumerate(HEROES):
        x = 50 + k * 285
        a.text(x, 332, name, 22, '#f8fafc')
        a.text(x + 218, 332, f'{int(hp * 240)}', 18, '#cbd5e1', 'end', '600')
        a.add(bevel(x, 342, 220, 12, '#0b1020', 2))
        a.rect(x + 2, 344, 216 * hp, 8, '#4ade80' if hp > 0.55 else '#f59e0b')
        a.add(bevel(x, 360, 220, 10, '#0b1020', 2))
        a.rect(x + 2, 362, 216 * mp, 6, '#60a5fa')
    return a


# ---------------------------------------------------------------- adventure
GUY = ['...hhhh...', '..hhhhhh..', '..hffffh..', '..fefeff..', '..ffffff..', '...ffff...', '.cccccccc.', 'cccccccccc',
       'cc.cccc.cc', 'ff.cccc.ff', '...pppp...', '...pp.pp..', '...pp.pp..', '..bbb.bbb.']


def adventure(a=None):
    a = a or Art('adv', 'A point-and-click adventure: a moonlit room with a window, a bookshelf, a door and a table, and a '
                        'verb bar below')
    a.rect(0, 0, W, 270, a.gradient('wall', [(0, '#3a2440'), (1, '#5e3c5a')]))
    for x in range(0, W, 40):
        for y in range(0, 210, 40):
            a.add(f'<path d="M{x + 20} {y + 12}l6 8l-6 8l-6 -8z" fill="#7a5070" opacity="0.25"/>')
    a.rect(0, 206, W, 64, a.gradient('floor', [(0, '#6a4630'), (1, '#4a2e20')]))
    for x in range(0, W, 60):
        a.rect(x, 206, 2, 64, '#3a2418')
    a.rect(0, 202, W, 6, '#2a1a10')
    a.add('<path d="M120 182 L340 182 L520 268 L200 268Z" fill="#cbd5ff" opacity="0.12"/>')
    a.rect(120, 50, 220, 132, a.gradient('night', [(0, '#0b1230'), (1, '#1e2a5a')]))
    a.add(f'<circle cx="290" cy="90" r="22" fill="#fef3c7"{a.glow}/>')
    rng = random.Random(2)
    a.add(''.join(f'<rect x="{rng.uniform(125, 335):.0f}" y="{rng.uniform(55, 175):.0f}" width="3" height="3" fill="#e2e8f0"/>'
                  for _ in range(18)))
    a.add('<rect x="118" y="48" width="224" height="136" fill="none" stroke="#2a1a10" stroke-width="10"/>'
          '<path d="M230 48V184M118 116H342" stroke="#2a1a10" stroke-width="6"/>'
          '<path d="M100 40 Q 120 120 106 200 L 92 200 L 92 40Z" fill="#7f1d1d"/><path d="M360 40 Q 340 120 354 200 L 368 200 L 368 40Z" '
          'fill="#7f1d1d"/><rect x="88" y="34" width="284" height="8" rx="4" fill="#5a3e22"/>')
    a.add(bevel(780, 40, 90, 166, '#5a3a1e', 3))
    for s in range(4):
        y = 48 + s * 40
        a.rect(786, y + 34, 78, 4, '#3a2418')
        x = 788
        for b in range(6):
            w = 10 + (b * 7 + s * 3) % 6
            col = ['#b91c1c', '#1d4ed8', '#15803d', '#a16207', '#7c3aed', '#0e7490'][(b + s) % 6]
            a.add(bevel(x, y + 8 + (b % 2) * 4, w, 26 - (b % 2) * 4, col, 1.5))
            x += w + 1
    a.add(bevel(920, 60, 140, 210, '#6b4422', 4))
    a.add('<rect x="934" y="74" width="112" height="80" fill="none" stroke="#4a2e16" stroke-width="4"/>'
          '<rect x="934" y="166" width="112" height="90" fill="none" stroke="#4a2e16" stroke-width="4"/>'
          '<circle cx="1040" cy="170" r="7" fill="#fbbf24"/>')
    a.add(bevel(500, 60, 120, 86, '#c9a86a', 4))
    a.add('<rect x="512" y="72" width="96" height="62" fill="#2f6f62"/><circle cx="560" cy="102" r="16" fill="#f59e0b"/>'
          '<path d="M512 134 L540 110 L560 124 L585 100 L608 134Z" fill="#1e4a3a"/>')
    a.add(shadow(660, 266, 110, 6, 0.4))
    a.add(bevel(560, 196, 200, 14, '#7a5230', 2))
    a.add(bevel(574, 210, 10, 52, '#4a3220', 1.5) + bevel(736, 210, 10, 52, '#4a3220', 1.5))
    a.add(f'<ellipse cx="656" cy="180" rx="60" ry="40" fill="#fde68a" opacity="0.15"{a.blur}/>')
    a.rect(650, 172, 12, 24, '#f8fafc')
    a.add(f'<path d="M656 172 q-7 -10 0 -20 q7 10 0 20z" fill="#fde68a"{a.glow}><animate attributeName="opacity" values="1;0.6;1" '
          f'dur="0.7s" repeatCount="indefinite"/></path>')
    a.add(shadow(430, 266, 40, 6))
    a.add(sprite(GUY, {'h': '#3a2210', 'f': '#f2c79e', 'e': '#1f2937', 'c': '#e2e8f0', 'p': '#1e3a8a', 'b': '#111827'},
                 400, 102, 12))
    a.rect(0, 274, W, 126, '#0a0a12')
    a.rect(0, 274, W, 3, '#3f3a5a')
    a.text(W / 2, 302, 'Look at candle', 22, '#c084fc', 'middle', glow=True)
    verbs = ['Walk to', 'Pick up', 'Use', 'Open', 'Close', 'Talk to', 'Give', 'Push', 'Look at']
    for k, v in enumerate(verbs):
        a.text(60 + (k // 3) * 170, 338 + (k % 3) * 24, v, 20, '#4ade80' if v == 'Look at' else '#2f8a50', weight='600')
    items = [('<path d="M20 20 h18 v6 h-6 v4 h-4 v-4 h-8z" fill="#fbbf24"/><circle cx="16" cy="23" r="7" fill="none" '
              'stroke="#fbbf24" stroke-width="3"/>'),
             '<rect x="14" y="8" width="26" height="22" fill="#f1f5f9"/><path d="M18 14h18M18 19h18M18 24h12" stroke="#94a3b8" '
             'stroke-width="2"/>', '<circle cx="27" cy="19" r="10" fill="#f59e0b"/><circle cx="27" cy="19" r="6" fill="#fbbf24"/>']
    for k in range(8):
        x, y = 620 + (k % 4) * 140, 318 + (k // 4) * 40
        a.add(bevel(x, y, 128, 36, '#1e1b2e', 2))
        if k < 3:
            a.add(f'<g transform="translate({x + 36} {y})">{items[k]}</g>')
    a.add('<g><path d="M0 -14V14M-14 0H14" stroke="#ffffff" stroke-width="3"/><animateMotion dur="5s" repeatCount="indefinite" '
          'path="M985 160 L 656 168 L 560 102 L 985 160" keyPoints="0;0.4;0.4;0.7;1" keyTimes="0;0.3;0.5;0.7;1" calcMode="linear"/></g>')
    return a


# ---------------------------------------------------------------- puzzle
PIECE = {'I': '#2dd4bf', 'O': '#fde047', 'T': '#c084fc', 'S': '#4ade80', 'Z': '#f87171', 'L': '#fb923c', 'J': '#60a5fa'}


def puzzle(a=None):
    a = a or Art('puzzle', 'A falling-block puzzle: a well of stacked pieces, a full line about to clear, and the next piece')
    a.rect(0, 0, W, H, a.gradient('bg', [(0, '#0b1020'), (1, '#2a1b5b')]))
    for x in range(0, W, 40):
        for y in range(0, H, 40):
            a.rect(x + 18, y + 18, 4, 4, '#ffffff', ' opacity="0.05"')
    c, wx, wy = 18, 510, 20
    a.add(f'<rect x="{wx - 8}" y="{wy - 4}" width="{10 * c + 16}" height="{20 * c + 10}" rx="4" fill="#05050c" stroke="#7c3aed" '
          f'stroke-width="3"{a.glow}/>')
    for k in range(1, 10):
        a.rect(wx + k * c, wy, 1, 20 * c, '#ffffff', ' opacity="0.04"')
    rng = random.Random(6)
    keys = list(PIECE)
    for r in range(12, 20):
        hole = rng.randrange(10) if r != 17 else -1
        for k in range(10):
            if k == hole or (r < 14 and rng.random() < 0.45):
                continue
            col = PIECE[keys[(r * 3 + k) % 7]]
            cell = bevel(wx + k * c, wy + r * c, c - 1, c - 1, col, 3)
            if r == 17:
                cell = (f'<g>{cell}<rect x="{wx + k * c}" y="{wy + r * c}" width="{c - 1}" height="{c - 1}" fill="#fff" opacity="0">'
                        f'<animate attributeName="opacity" values="0;0.9;0" dur="1.2s" repeatCount="indefinite"/></rect></g>')
            a.add(cell)
    t = [(0, 0), (1, 0), (2, 0), (1, 1)]
    a.add(''.join(f'<rect x="{wx + (3 + dx) * c + 1}" y="{wy + (11 + dy) * c + 1}" width="{c - 3}" height="{c - 3}" fill="none" '
                  f'stroke="{PIECE["T"]}" stroke-width="2" opacity="0.6"/>' for dx, dy in t))
    cells = ''.join(bevel(wx + (3 + dx) * c, wy + dy * c, c - 1, c - 1, PIECE['T'], 3) for dx, dy in t)
    steps = ';'.join(f'0 {k * c}' for k in range(11))
    a.add(f'<g>{cells}<animateTransform attributeName="transform" type="translate" values="{steps}" calcMode="discrete" '
          f'dur="5.5s" repeatCount="indefinite"/></g>')
    a.add(bevel(740, 40, 150, 100, '#141432', 3))
    a.text(756, 70, 'NEXT', 20, '#e2e8f0')
    for dx in range(4):
        a.add(bevel(764 + dx * 26, 92, 25, 25, PIECE['I'], 3))
    a.add(bevel(740, 160, 150, 90, '#141432', 3))
    a.text(756, 196, 'LINES 42', 20, '#fde68a', glow=True)
    a.text(756, 228, 'LEVEL 5', 20, '#fde68a', glow=True)
    a.add(bevel(310, 40, 160, 90, '#141432', 3))
    a.text(326, 72, 'SCORE', 20, '#e2e8f0')
    a.text(326, 110, '018220', 28, '#fde68a', glow=True)
    for k, (key, (x, y)) in enumerate(zip('SZLJOI', ((90, 120), (180, 260), (1000, 300), (1080, 110), (300, 320), (1000, 40)))):
        a.add(f'<g transform="rotate({k * 17} {x + 20} {y + 20})" opacity="0.3">{bevel(x, y, 40, 40, PIECE[key], 4)}</g>')
    return a


# ---------------------------------------------------------------- online
BLOBS = [(230, 120, 120, 48), (300, 175, 70, 40), (330, 260, 50, 70), (360, 330, 30, 40), (560, 95, 70, 30),
         (610, 130, 60, 40), (620, 230, 60, 80), (660, 300, 35, 40), (800, 110, 170, 50), (880, 170, 90, 40),
         (830, 210, 40, 30), (980, 300, 60, 34), (1010, 140, 40, 40), (450, 70, 40, 20)]


def online(a=None):
    a = a or Art('online', 'Players around the world connected to a server, with messages travelling between them')
    a.rect(0, 0, W, H, a.gradient('bg', [(0, '#020617'), (1, '#0b2a3a')]))
    a.add(f'<ellipse cx="600" cy="200" rx="560" ry="200" fill="#0e7490" opacity="0.18"{a.blur}/>')
    for gx in range(14, W, 12):
        for gy in range(14, H, 12):
            d = min(((gx - x) / w) ** 2 + ((gy - y) / h) ** 2 for x, y, w, h in BLOBS)
            if d < 1:
                a.add(f'<circle cx="{gx}" cy="{gy}" r="{2.6 if d < 0.6 else 1.8}" fill="#2dd4bf" '
                      f'opacity="{0.7 if d < 0.6 else 0.4}"/>')
    for gy in range(40, H, 60):
        a.add(f'<path d="M0 {gy} H{W}" stroke="#2dd4bf" opacity="0.06"/>')
    nodes = [(220, 120), (330, 260), (600, 120), (640, 250), (860, 110), (990, 300), (300, 170), (880, 180)]
    server = (600, 120)
    for k, (x, y) in enumerate(nodes):
        if (x, y) == server:
            continue
        mx, my = (x + server[0]) / 2, min(y, server[1]) - 90
        d = f'M{x} {y} Q{mx} {my} {server[0]} {server[1]}'
        a.add(f'<path d="{d}" fill="none" stroke="#5eead4" stroke-width="2" opacity="0.45"/>')
        a.add(f'<circle r="5" fill="#fde68a"{a.glow}><animateMotion dur="{2.2 + k * 0.3:.1f}s" begin="{-k * 0.4:.1f}s" '
              f'repeatCount="indefinite" path="{d}"/></circle>')
        a.add(f'<circle cx="{x}" cy="{y}" r="8" fill="#f59e0b"{a.glow}/><circle cx="{x}" cy="{y}" r="8" fill="none" stroke="#f59e0b" '
              f'stroke-width="2"><animate attributeName="r" values="8;26" dur="2s" begin="{k * 0.3:.1f}s" repeatCount="indefinite"/>'
              f'<animate attributeName="opacity" values="1;0" dur="2s" begin="{k * 0.3:.1f}s" repeatCount="indefinite"/></circle>')
    a.add(bevel(server[0] - 22, server[1] - 30, 44, 60, '#1e293b', 3))
    for k in range(4):
        a.rect(server[0] - 15, server[1] - 23 + k * 13, 30, 8, '#0f766e')
        a.add(f'<circle cx="{server[0] + 10}" cy="{server[1] - 19 + k * 13}" r="2" fill="#4ade80"><animate attributeName="opacity" '
              f'values="1;0.2;1" dur="{0.5 + k * 0.2:.1f}s" repeatCount="indefinite"/></circle>')
    a.add(bevel(970, 20, 210, 150, '#0b1a26', 3))
    a.text(985, 50, 'PLAYERS', 16, '#5eead4')
    for k, (name, ping) in enumerate((('aki', 34), ('maren', 71), ('dz', 112), ('olu', 168))):
        col = '#4ade80' if ping < 80 else '#fbbf24' if ping < 150 else '#f87171'
        a.text(985, 80 + k * 24, name, 17, '#e2e8f0', weight='600')
        a.text(1165, 80 + k * 24, f'{ping} ms', 17, col, 'end', '600')
    return a


FAMILIES = [('arcade', 'Arcade action', arcade), ('fps', 'First-person', first_person), ('race', 'Racing', racing),
            ('space', 'Vehicle and space', space_sim), ('city', 'Management', management), ('rts', 'Strategy', strategy),
            ('rpg', 'RPG', rpg), ('adv', 'Adventure', adventure), ('puzzle', 'Puzzle', puzzle), ('online', 'Online', online)]


def mosaic():
    """All ten families in a grid of five by two, each tile a crop from the middle of its scene."""
    out = Art('mosaic', 'Ten families of games: arcade action, first-person, racing, vehicle and space simulations, '
                        'management, strategy, RPG, adventure, puzzle and online games')
    tw, th, gap = 228, 190, 12
    for k, (key, name, fn) in enumerate(FAMILIES):
        scene = fn(Art('m-' + key, name))
        x, y = 6 + (k % 5) * (tw + gap), 6 + (k // 5) * (th + gap)
        out.add(f'<svg x="{x}" y="{y}" width="{tw}" height="{th}" viewBox="260 0 680 400" preserveAspectRatio="xMidYMid slice">'
                f'{scene.inner()}</svg>')
        out.add(f'<rect x="{x}" y="{y + th - 36}" width="{tw}" height="36" fill="#000" opacity="0.6"/>')
        out.add(f'<rect x="{x}" y="{y}" width="{tw}" height="{th}" rx="10" fill="none" stroke="#ffffff" stroke-opacity="0.12" '
                f'stroke-width="2"/>')
        out.add(f'<text x="{x + 12}" y="{y + th - 12}" font-family="ui-sans-serif, Helvetica, Arial, sans-serif" font-size="17" '
                f'font-weight="700" fill="#f8fafc">{name}</text>')
    out.parts.insert(0, f'<rect width="{W}" height="{H}" fill="#0d1b1e"/>')
    return out.render()


DIAGRAMS = {'art-games.svg': mosaic}
for _key, _name, _fn in FAMILIES:
    DIAGRAMS[f'art-{_key}.svg'] = (lambda fn=_fn: fn().render())
