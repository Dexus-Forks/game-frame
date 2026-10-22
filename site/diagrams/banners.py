"""Editorial banners for the top of each section of the guides, and the home page's hero
picture. They are illustrations: the machines, media and tools of each section, drawn in the
same pixel style as the game family scenes in art.py. site/diagrams/make.py writes them.
"""
import math
import random

from art import Art, H, W, bevel, frames, shade, shadow, sprite, stars

MONO = 'ui-monospace, Menlo, Consolas, monospace'


def mono(a, x, y, s, size, fill, anchor='start', weight='700', extra=''):
    a.add(f'<text x="{x}" y="{y}" font-family="{MONO}" font-size="{size}" font-weight="{weight}" fill="{fill}" '
          f'text-anchor="{anchor}"{extra}>{s}</text>')


def crt(a, x, y, w, h, inner, bezel='#c9c2b0'):
    """A CRT monitor: a bezel, a dark glass and a scene clipped to the glass."""
    a.add(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="18" fill="{bezel}"/>'
          f'<rect x="{x + 6}" y="{y + 6}" width="{w - 12}" height="{h - 12}" rx="14" fill="{shade(bezel, 0.8)}"/>')
    gx, gy, gw, gh = x + 26, y + 24, w - 52, h - 58
    cid = a.id(f'glass{x}')
    a.defs.append(f'<clipPath id="{cid}"><rect x="{gx}" y="{gy}" width="{gw}" height="{gh}" rx="16"/></clipPath>')
    a.add(f'<rect x="{gx}" y="{gy}" width="{gw}" height="{gh}" rx="16" fill="#0a0f0c"/>'
          f'<g clip-path="url(#{cid})">{inner(gx, gy, gw, gh)}'
          + ''.join(f'<rect x="{gx}" y="{gy + k}" width="{gw}" height="1" fill="#000" opacity="0.18"/>' for k in range(0, int(gh), 3))
          + '</g>')
    a.add(f'<circle cx="{x + w - 40}" cy="{y + h - 18}" r="5" fill="#4ade80"/>')


# ---------------------------------------------------------------- hardware: a motherboard
def chip(a, x, y, w, h, label, sub, pins=True):
    """A chip in a black package: pins on two sides, a glossy top, a notch and printed markings."""
    if pins:
        for k in range(int((w - 16) // 14) + 1):
            a.add(bevel(x + 8 + k * 14, y - 10, 7, 10, '#c0c4cc', 1.5) + bevel(x + 8 + k * 14, y + h, 7, 10, '#c0c4cc', 1.5))
    a.add(shadow(x + w / 2 + 6, y + h + 6, w / 2, 8, 0.4))
    a.add(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="5" fill="{a.gradient(f"chip{x}", [(0, "#2c2c34"), (1, "#121216")])}"/>'
          f'<rect x="{x + 1}" y="{y + 1}" width="{w - 2}" height="{h * 0.45:.0f}" rx="4" fill="#ffffff" opacity="0.05"/>'
          f'<circle cx="{x + 14}" cy="{y + h / 2}" r="5" fill="#0a0a0c"/>')
    mono(a, x + w / 2, y + h / 2 + 2, label, 22, '#e5e7eb', 'middle')
    mono(a, x + w / 2, y + h / 2 + 24, sub, 13, '#9ca3af', 'middle', '600')


def hardware():
    a = Art('b-hw', 'A circuit board with a CPU, a video chip, a sound chip, memory and a timer, with signals moving along '
                    'its copper traces')
    a.rect(0, 0, W, H, a.gradient('pcb', [(0, '#0d4a34'), (1, '#05261a')]))
    for x in range(0, W, 8):
        a.rect(x, 0, 1, H, '#ffffff', ' opacity="0.015"')
    rng = random.Random(11)
    traces = []
    for k in range(40):
        x, y = rng.randrange(0, W, 20), rng.randrange(0, H, 20)
        pts = [(x, y)]
        for _ in range(4):
            if rng.random() < 0.5:
                x += rng.choice([-1, 1]) * rng.randrange(60, 260, 20)
            else:
                y += rng.choice([-1, 1]) * rng.randrange(40, 160, 20)
            pts.append((x, y))
        d = 'M' + ' L'.join(f'{px} {py}' for px, py in pts)
        traces.append(d)
        a.add(f'<path d="{d}" fill="none" stroke="#062a1c" stroke-width="5" opacity="0.6"/>'
              f'<path d="{d}" fill="none" stroke="#c9a227" stroke-width="2.5" opacity="0.55"/>')
        for px, py in (pts[0], pts[-1]):
            a.add(f'<circle cx="{px}" cy="{py}" r="5" fill="#d4af37"/><circle cx="{px}" cy="{py}" r="2" fill="#05261a"/>')
    for k, d in enumerate(traces[:16]):
        a.add(f'<circle r="4" fill="#fef3c7"{a.glow}><animateMotion dur="{2 + k % 5}s" begin="{-k * 0.7:.1f}s" '
              f'repeatCount="indefinite" path="{d}"/></circle>')
    chip(a, 110, 120, 240, 150, 'CPU', '80486DX2-66')
    a.add(f'<rect x="150" y="150" width="160" height="90" fill="#2dd4bf" opacity="0.05"{a.glow}><animate attributeName="opacity" '
          f'values="0.03;0.12;0.03" dur="2s" repeatCount="indefinite"/></rect>')
    chip(a, 460, 80, 180, 100, 'VGA', '256 KB')
    chip(a, 460, 250, 180, 90, 'OPL3', 'FM synthesis')
    for k in range(4):
        x = 740 + k * 40
        a.add(bevel(x, 50, 26, 300, '#1e293b', 2))
        for j in range(9):
            a.add(bevel(x + 4, 62 + j * 31, 18, 24, '#0f172a', 2))
        a.rect(x + 2, 344, 22, 10, '#d4af37')
    chip(a, 960, 150, 160, 110, 'PIT', '8253')
    for x, y, col in ((400, 60, '#1d4ed8'), (420, 330, '#1d4ed8'), (690, 200, '#111827'), (930, 300, '#1d4ed8')):
        a.add(shadow(x + 4, y + 34, 16, 5, 0.4))
        a.add(f'<rect x="{x - 14}" y="{y}" width="28" height="34" rx="4" fill="{a.gradient(f"cap{x}", [(0, shade(col, 1.6)), (0.5, col), (1, shade(col, 0.5))], False)}"/>'
              f'<ellipse cx="{x}" cy="{y}" rx="14" ry="5" fill="#9ca3af"/><path d="M{x - 6} {y - 2}l12 4M{x - 6} {y + 2}l12 -4" '
              f'stroke="#4b5563" stroke-width="1.5"/>')
    for x in range(1000, 1160, 22):
        a.add(bevel(x, 300, 16, 6, '#d6c3a0', 1) + f'<rect x="{x + 4}" y="300" width="2" height="6" fill="#b91c1c"/>'
              f'<rect x="{x + 9}" y="300" width="2" height="6" fill="#1d4ed8"/>')
    for x in range(40, 340, 12):
        a.rect(x, 372, 8, 28, '#d4af37')
    return a


# ---------------------------------------------------------------- consoles: a TV, a pad and a cartridge
def platform_scene(a, gx, gy, gw, gh, t=26):
    """A side-scrolling platform game: sky, hills, bricks, a question block, a pipe and a hero."""
    out = [f'<rect x="{gx}" y="{gy}" width="{gw}" height="{gh}" fill="{a.gradient(f"psky{gx}", [(0, "#3b82f6"), (1, "#93c5fd")])}"/>']
    for hx, hr in ((gx + 60, 90), (gx + 260, 120)):
        out.append(f'<circle cx="{hx}" cy="{gy + gh - 10}" r="{hr}" fill="#15803d" opacity="0.6"/>')
    out.append(f'<g><ellipse cx="{gx + 90}" cy="{gy + 44}" rx="30" ry="12" fill="#fff"/><ellipse cx="{gx + 110}" cy="{gy + 36}" rx="20" '
               f'ry="12" fill="#fff"/><animateTransform attributeName="transform" type="translate" values="0 0;40 0;0 0" dur="10s" '
               f'repeatCount="indefinite"/></g>')
    for c in range(int(gw // t) + 1):
        out.append(bevel(gx + c * t, gy + gh - t, t - 1, t - 1, '#16a34a', 3) + bevel(gx + c * t, gy + gh - 2, t - 1, 4, '#8a4a20', 1))
    for c in (4, 5, 6, 7):
        col = '#f59e0b' if c == 5 else '#c2410c'
        out.append(bevel(gx + c * t, gy + gh - 5 * t, t - 1, t - 1, col, 3))
    out.append(f'<text x="{gx + 5.5 * t}" y="{gy + gh - 4.25 * t}" font-family="{MONO}" font-size="16" font-weight="800" '
               f'fill="#7c2d12" text-anchor="middle">?</text>')
    px = gx + gw - 110
    out.append(bevel(px - 4, gy + gh - 3 * t, 2 * t + 8, 16, '#22c55e', 3) + bevel(px, gy + gh - 3 * t + 16, 2 * t, 2 * t - 16, '#15803d', 3))
    hero = sprite(['.rrrr.', 'rrrrrr', '.ssss.', '.sess.', 'tttttt', 'tt..tt', 'bb..bb'],
                  {'r': '#ef4444', 's': '#fcd5b4', 'e': '#1f2937', 't': '#2563eb', 'b': '#7c2d12'}, 0, 0, 5)
    out.append(f'<g>{hero}<animateTransform attributeName="transform" type="translate" '
               f'values="{gx + 40} {gy + gh - t - 36};{gx + 150} {gy + gh - 5 * t - 40};{gx + 260} {gy + gh - t - 36};'
               f'{gx + 40} {gy + gh - t - 36}" dur="3s" repeatCount="indefinite"/></g>')
    return ''.join(out)


def consoles():
    a = Art('b-con', 'A television showing a platform game, with a game pad and a cartridge beside it')
    a.rect(0, 0, W, H, a.gradient('room', [(0, '#1f1630'), (1, '#3a2340')]))
    a.rect(0, 326, W, 74, a.gradient('shelf', [(0, '#5a3a2a'), (1, '#3a2418')]))
    a.rect(0, 324, W, 4, '#2a1a12')
    a.add(f'<ellipse cx="600" cy="190" rx="300" ry="160" fill="#60a5fa" opacity="0.12"{a.blur}/>')
    a.add(shadow(600, 334, 260, 10, 0.5))
    a.add(f'<rect x="350" y="26" width="500" height="300" rx="22" fill="{a.gradient("tv", [(0, "#4a4a52"), (1, "#26262c")])}"/>'
          '<rect x="360" y="36" width="480" height="280" rx="16" fill="#1a1a1f"/>')
    cid = a.id('tvglass')
    a.defs.append(f'<clipPath id="{cid}"><rect x="378" y="50" width="444" height="232" rx="20"/></clipPath>')
    a.add(f'<g clip-path="url(#{cid})">{platform_scene(a, 378, 50, 444, 232)}'
          + ''.join(f'<rect x="378" y="{50 + k}" width="444" height="1" fill="#000" opacity="0.15"/>' for k in range(0, 232, 3))
          + '<path d="M378 50 H822 V110 Q600 90 378 120Z" fill="#fff" opacity="0.07"/></g>')
    a.add('<circle cx="808" cy="300" r="5" fill="#ef4444"/><rect x="560" y="320" width="80" height="10" fill="#1a1a1f"/>')
    # The game pad.
    a.add(shadow(180, 330, 130, 10, 0.5))
    a.add(f'<path d="M70 240 Q70 222 92 222 H268 Q290 222 290 240 V300 Q290 322 268 322 H92 Q70 322 70 300Z" '
          f'fill="{a.gradient("pad", [(0, "#e5e7eb"), (1, "#a1a1aa")])}"/>'
          '<rect x="84" y="236" width="192" height="72" rx="6" fill="#1f2937"/>')
    a.add(bevel(110, 252, 18, 50, '#111827', 2) + bevel(94, 268, 50, 18, '#111827', 2))
    for cx in (226, 258):
        a.add(f'<circle cx="{cx}" cy="286" r="13" fill="#7f1d1d"/><circle cx="{cx}" cy="284" r="12" fill="#dc2626"/>'
              f'<circle cx="{cx - 4}" cy="280" r="4" fill="#fca5a5" opacity="0.7"/>')
    a.add('<rect x="158" y="282" width="20" height="7" rx="3.5" fill="#6b7280"/><rect x="184" y="282" width="20" height="7" rx="3.5" '
          'fill="#6b7280"/><path d="M180 222 C 180 160 330 160 350 196" stroke="#111827" stroke-width="5" fill="none"/>')
    # The cartridge.
    a.add(shadow(1030, 332, 100, 8, 0.5))
    a.add(f'<path d="M940 118 H1120 Q1128 118 1128 126 V324 H932 V126 Q932 118 940 118Z" '
          f'fill="{a.gradient("cart", [(0, "#b6bac2"), (1, "#80848c")])}"/>')
    for k in range(6):
        a.rect(946, 130 + k * 6, 168, 2, '#6b7280', ' opacity="0.5"')
    a.add(bevel(950, 170, 160, 118, '#1e3a8a', 3))
    a.add(f'<g><clipPath id="{a.id("label")}"><rect x="954" y="174" width="152" height="110"/></clipPath>'
          f'<g clip-path="url(#{a.id("label")})">{platform_scene(a, 954, 174, 152, 110, 14)}</g></g>')
    a.rect(940, 300, 180, 24, '#6b7280')
    for k in range(12):
        a.rect(948 + k * 14, 306, 8, 18, '#d4af37')
    return a


# ---------------------------------------------------------------- handhelds
def handhelds():
    a = Art('b-hand', 'Two handheld game machines on a desk, one with a green monochrome screen and one with a colour screen')
    a.rect(0, 0, W, H, a.gradient('bg', [(0, '#13212b'), (1, '#24363f')]))
    a.add(f'<ellipse cx="600" cy="200" rx="420" ry="160" fill="#2dd4bf" opacity="0.08"{a.blur}/>')

    def device(x, y, w, h, top, bottom, screen_fn, glass, label):
        a.add(shadow(x + w / 2 + 10, y + h + 4, w / 2 + 10, 12, 0.5))
        a.add(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="22" fill="{a.gradient(f"body{x}", [(0, top), (1, bottom)])}"/>'
              f'<rect x="{x + 3}" y="{y + 3}" width="{w - 6}" height="{h * 0.4:.0f}" rx="20" fill="#fff" opacity="0.07"/>')
        a.add(f'<rect x="{x + 18}" y="{y + 22}" width="{w - 36}" height="{w - 60}" rx="10" fill="#33333d"/>'
              f'<rect x="{x + 18}" y="{y + 22}" width="{w - 36}" height="{w - 60}" rx="10" fill="none" stroke="#000" stroke-opacity="0.4"/>')
        mono(a, x + 30, y + 38, label, 9, '#a1a1aa', weight='600')
        a.add(f'<circle cx="{x + 30}" cy="{y + 80}" r="4" fill="#ef4444"{a.glow}/>')
        sx, sy, sw, sh = x + 44, y + 44, w - 88, w - 110
        cid = a.id(f'scr{x}')
        a.defs.append(f'<clipPath id="{cid}"><rect x="{sx}" y="{sy}" width="{sw}" height="{sh}"/></clipPath>')
        a.add(f'<rect x="{sx}" y="{sy}" width="{sw}" height="{sh}" fill="{glass}"/><g clip-path="url(#{cid})">{screen_fn(sx, sy, sw, sh)}'
              f'<path d="M{sx} {sy} H{sx + sw} V{sy + sh * 0.3:.0f} Q{sx + sw / 2} {sy + sh * 0.2:.0f} {sx} {sy + sh * 0.4:.0f}Z" '
              f'fill="#fff" opacity="0.08"/></g>')
        dy = y + w - 10
        a.add(bevel(x + 34, dy + 10, 18, 54, '#1f2937', 2) + bevel(x + 16, dy + 28, 54, 18, '#1f2937', 2))
        for cx, cy in ((x + w - 70, dy + 46), (x + w - 34, dy + 28)):
            a.add(f'<circle cx="{cx}" cy="{cy + 2}" r="15" fill="#5b0f2e"/><circle cx="{cx}" cy="{cy}" r="15" fill="#9d174d"/>'
                  f'<circle cx="{cx - 5}" cy="{cy - 5}" r="5" fill="#f9a8d4" opacity="0.5"/>')
        for k in range(2):
            a.add(f'<rect x="{x + w / 2 - 40 + k * 44}" y="{dy + 86}" width="32" height="8" rx="4" fill="#71717a" '
                  f'transform="rotate(-25 {x + w / 2 - 24 + k * 44} {dy + 90})"/>')
        for k in range(6):
            a.add(f'<rect x="{x + w - 60 + k * 8}" y="{y + h - 50}" width="4" height="34" rx="2" fill="#000" opacity="0.25" '
                  f'transform="rotate(-25 {x + w - 40} {y + h - 33})"/>')

    def mono_scene(sx, sy, sw, sh):
        pal = ['#0f380f', '#306230', '#8bac0f', '#9bbc0f']
        out = [f'<rect x="{sx}" y="{sy}" width="{sw}" height="{sh}" fill="{pal[3]}"/>']
        for r in range(3):
            for c in range(10):
                if (r, c) not in ((0, 3), (0, 4), (1, 7)):
                    out.append(bevel(sx + 6 + c * 12, sy + sh - 14 - r * 12, 11, 11, pal[1], 2))
        piece = ''.join(bevel(sx + 54 + dx * 12, dy * 12, 11, 11, pal[0], 2) for dx, dy in ((0, 0), (1, 0), (2, 0), (1, 1)))
        out.append(f'<g>{piece}<animateTransform attributeName="transform" type="translate" '
                   f'values="0 {sy + 4};0 {sy + 16};0 {sy + 28};0 {sy + 40};0 {sy + 52}" calcMode="discrete" dur="2.5s" '
                   f'repeatCount="indefinite"/></g>')
        out.append(f'<rect x="{sx}" y="{sy}" width="{sw}" height="{sh}" fill="url(#{a.id("dots")})" opacity="0.25"/>')
        return ''.join(out)
    a.defs.append(f'<pattern id="{a.id("dots")}" width="3" height="3" patternUnits="userSpaceOnUse"><rect width="1" height="3" '
                  f'fill="#0f380f"/><rect width="3" height="1" fill="#0f380f"/></pattern>')

    def colour_scene(sx, sy, sw, sh):
        return platform_scene(a, sx, sy, sw, sh, 14)
    device(340, 14, 228, 368, '#d6d2c8', '#b4afa4', mono_scene, '#9bbc0f', 'DOT MATRIX')
    device(650, 26, 236, 356, '#4f46e5', '#312e81', colour_scene, '#38bdf8', 'COLOUR LCD')
    return a


# ---------------------------------------------------------------- home computers: a breadbin, its monitor and drives
def computers():
    a = Art('b-comp', 'An 8-bit home computer in a wedge-shaped case with the keyboard built in, connected to a monitor '
                      'showing its blue BASIC screen, with a cassette deck on one side and a disk drive on the other')
    a.rect(0, 0, W, H, a.gradient('wall', [(0, '#2b2420'), (1, '#3d332b')]))
    a.rect(0, 296, W, 104, a.gradient('desk', [(0, '#6b4f36'), (1, '#4a3424')]))
    a.rect(0, 294, W, 4, '#2a1d14')
    # Cables run behind everything.
    a.add('<path d="M240 270 C 300 250 330 262 360 268" stroke="#141010" stroke-width="5" fill="none"/>'
          '<path d="M960 262 C 920 250 880 262 850 268" stroke="#141010" stroke-width="5" fill="none"/>')
    # The monitor, behind the computer, with its screen clear of it.
    a.add(shadow(600, 268, 190, 10, 0.4))
    a.add(f'<rect x="430" y="18" width="340" height="236" rx="16" fill="{a.gradient("mon", [(0, "#a08a6c"), (1, "#76624c")])}"/>')
    a.add('<rect x="444" y="30" width="312" height="196" rx="10" fill="#5a4a38"/>')
    a.rect(454, 38, 292, 180, '#7c70da', ' rx="12"')
    a.rect(476, 54, 248, 146, '#40318d')
    lines = ['64K RAM SYSTEM  38911', 'BASIC BYTES FREE', '', 'READY.', '10 PRINT "HELLO ";', '20 GOTO 10', 'RUN',
             'HELLO HELLO HELLO']
    for k, line in enumerate(lines):
        mono(a, 482, 70 + k * 16, line.replace(' ', '&#160;'), 12.5, '#a5a0ff', weight='700')
    a.add('<rect x="482" y="188" width="9" height="12" fill="#a5a0ff"><animate attributeName="opacity" values="1;0;1" dur="1s" '
          'calcMode="discrete" repeatCount="indefinite"/></rect>')
    for k in range(0, 180, 3):
        a.rect(454, 38 + k, 292, 1, '#000', ' opacity="0.12"')
    a.add('<path d="M462 46 Q 600 38 738 46 L 734 92 Q 600 80 466 92Z" fill="#fff" opacity="0.06"/>')
    a.add('<circle cx="740" cy="238" r="4" fill="#4ade80"/><rect x="700" y="234" width="28" height="7" rx="2" fill="#3a2e22"/>')
    # The cassette deck, on the left: a long low box with a smoked lid over the cassette, a tape
    # counter, and a row of piano keys along its front.
    tbl, tbr, tfl, tfr, tby, tfy = 52, 238, 36, 254, 226, 262

    def tape_at(u, v):
        left = tbl + (tfl - tbl) * v
        right = tbr + (tfr - tbr) * v
        return left + (right - left) * u, tby + (tfy - tby) * v

    def quad(u0, v0, u1, v1, fill, extra=''):
        p0, p1, p2, p3 = tape_at(u0, v0), tape_at(u1, v0), tape_at(u1, v1), tape_at(u0, v1)
        return (f'<path d="M{p0[0]:.1f} {p0[1]:.1f} L{p1[0]:.1f} {p1[1]:.1f} L{p2[0]:.1f} {p2[1]:.1f} L{p3[0]:.1f} {p3[1]:.1f}Z" '
                f'fill="{fill}"{extra}/>')
    a.add(shadow(146, 298, 112, 7, 0.5))
    a.add(f'<path d="M{tfl} {tfy} V{tfy + 26} Q{tfl} {tfy + 32} {tfl + 6} {tfy + 32} H{tfr - 6} Q{tfr} {tfy + 32} {tfr} {tfy + 26} V{tfy}Z" '
          f'fill="#8c7a5e"/>')
    a.add(f'<path d="M{tbl + 6} {tby} H{tbr - 6} Q{tbr} {tby} {tbr + 2} {tby + 4} L{tfr} {tfy} H{tfl} L{tbl - 2} {tby + 4} '
          f'Q{tbl} {tby} {tbl + 6} {tby}Z" fill="{a.gradient("dtop", [(0, "#cbbc9c"), (1, "#b2a282")])}"/>')
    a.add(quad(0.06, 0.08, 0.7, 0.62, '#2a2420'))
    a.add(quad(0.1, 0.16, 0.66, 0.54, '#5a4e44'))
    lab0, lab1 = tape_at(0.2, 0.2), tape_at(0.56, 0.28)
    a.add(f'<rect x="{lab0[0]:.1f}" y="{lab0[1]:.1f}" width="{lab1[0] - lab0[0]:.1f}" height="4" fill="#e7dcc4"/>')
    for u in (0.24, 0.52):
        cx, cy = tape_at(u, 0.38)
        a.add(f'<ellipse cx="{cx:.1f}" cy="{cy:.1f}" rx="9" ry="4" fill="#3f2a1a"/><ellipse cx="{cx:.1f}" cy="{cy:.1f}" rx="3.5" ry="1.8" '
              f'fill="#e5e7eb"/>')
    a.add(quad(0.06, 0.08, 0.7, 0.62, '#1a1410', ' opacity="0.35"'))
    a.add(quad(0.08, 0.1, 0.4, 0.3, '#ffffff', ' opacity="0.12"'))
    a.add(quad(0.76, 0.14, 0.96, 0.42, '#1a1410'))
    cx, cy = tape_at(0.86, 0.29)
    mono(a, cx, cy + 3, '023', 9, '#f1f5f9', 'middle')
    cx, cy = tape_at(0.86, 0.52)
    a.add(f'<ellipse cx="{cx:.1f}" cy="{cy:.1f}" rx="5" ry="2.5" fill="#5a4636"/>')
    keys = ['rec', 'play', 'rew', 'ff', 'stop', 'eject']
    for k, name in enumerate(keys):
        u0, u1 = 0.04 + k * 0.16, 0.04 + k * 0.16 + 0.145
        a.add(quad(u0, 0.66, u1, 1.0, '#4a3c30'))
        a.add(quad(u0 + 0.01, 0.68, u1 - 0.01, 0.8, '#6a5646'))
        f0, f1 = tape_at(u0, 1.0), tape_at(u1, 1.0)
        a.add(bevel(f0[0], tfy, f1[0] - f0[0], 14, '#3a2e24', 1.5))
        mx, my = (f0[0] + f1[0]) / 2, tfy + 7
        if name == 'rec':
            a.add(f'<circle cx="{mx:.1f}" cy="{my}" r="3" fill="#ef4444"/>')
        elif name == 'play':
            a.add(f'<path d="M{mx - 3:.1f} {my - 3.5} L{mx + 4:.1f} {my} L{mx - 3:.1f} {my + 3.5}Z" fill="#e7dcc4"/>')
        elif name == 'rew':
            a.add(f'<path d="M{mx + 4:.1f} {my - 3} L{mx:.1f} {my} L{mx + 4:.1f} {my + 3}Z M{mx:.1f} {my - 3} L{mx - 4:.1f} {my} '
                  f'L{mx:.1f} {my + 3}Z" fill="#e7dcc4"/>')
        elif name == 'ff':
            a.add(f'<path d="M{mx - 4:.1f} {my - 3} L{mx:.1f} {my} L{mx - 4:.1f} {my + 3}Z M{mx:.1f} {my - 3} L{mx + 4:.1f} {my} '
                  f'L{mx:.1f} {my + 3}Z" fill="#e7dcc4"/>')
        elif name == 'stop':
            a.add(f'<rect x="{mx - 3:.1f}" y="{my - 3}" width="6" height="6" fill="#e7dcc4"/>')
        else:
            a.add(f'<path d="M{mx - 4:.1f} {my + 1} L{mx:.1f} {my - 3.5} L{mx + 4:.1f} {my + 1}Z" fill="#e7dcc4"/>'
                  f'<rect x="{mx - 4:.1f}" y="{my + 2.5}" width="8" height="1.6" fill="#e7dcc4"/>')
    # The disk drive, on the right.
    a.add(shadow(1065, 300, 112, 7, 0.45))
    a.add(bevel(960, 192, 210, 106, '#b9a888', 3))
    a.add(bevel(960, 192, 210, 16, '#cbbb9c', 2))
    a.add('<rect x="976" y="236" width="140" height="12" rx="3" fill="#221c18"/>'
          '<rect x="1030" y="229" width="30" height="26" rx="4" fill="#6f604a"/><rect x="1036" y="236" width="18" height="12" rx="2" '
          'fill="#4a3e30"/>')
    a.add('<rect x="1124" y="272" width="14" height="6" rx="2" fill="#22c55e"/><rect x="1144" y="272" width="14" height="6" rx="2" '
          'fill="#ef4444"><animate attributeName="opacity" values="1;0.2;1;1;0.2;1" dur="1.4s" repeatCount="indefinite"/></rect>')
    # The computer: one wedge-shaped case. Its top slopes down towards the viewer, and the keyboard
    # is set into that slope.
    bl, br, fl, fr, by, fy = 300, 900, 268, 932, 262, 352
    a.add(shadow(600, 386, 360, 10, 0.55))
    a.add(f'<path d="M{fl} {fy} L{fl + 4} {fy + 22} Q{fl + 6} {fy + 30} {fl + 16} {fy + 30} H{fr - 16} Q{fr - 6} {fy + 30} {fr - 4} {fy + 22} '
          f'L{fr} {fy}Z" fill="#7a6a52"/>')
    top = a.gradient('top', [(0, '#c4b494'), (1, '#a8977a')])
    a.add(f'<path d="M{bl + 14} {by} H{br - 14} Q{br} {by} {br + 4} {by + 10} L{fr} {fy - 6} Q{fr + 2} {fy} {fr - 8} {fy} '
          f'H{fl + 8} Q{fl - 2} {fy} {fl} {fy - 6} L{bl - 4} {by + 10} Q{bl} {by} {bl + 14} {by}Z" fill="{top}"/>')
    a.add(f'<path d="M{bl + 14} {by} H{br - 14} Q{br} {by} {br + 4} {by + 10} L{br + 5} {by + 13} H{bl - 5} L{bl - 4} {by + 10} '
          f'Q{bl} {by} {bl + 14} {by}Z" fill="#d6c8aa"/>')

    def at(u, v):
        """A point on the sloped top: u from left (0) to right (1), v from back (0) to front (1)."""
        left = bl + (fl - bl) * v
        right = br + (fr - br) * v
        return left + (right - left) * u, by + (fy - by) * v

    # The recess the keyboard sits in.
    r0, r1 = at(0.05, 0.22), at(0.83, 0.22)
    r2, r3 = at(0.84, 0.94), at(0.04, 0.94)
    a.add(f'<path d="M{r0[0]:.1f} {r0[1]:.1f} L{r1[0]:.1f} {r1[1]:.1f} L{r2[0]:.1f} {r2[1]:.1f} L{r3[0]:.1f} {r3[1]:.1f}Z" '
          f'fill="#4a3c30"/>')
    rows = [(17, 0.0), (17, 0.35), (16, 0.6), (15, 0.85)]
    for r, (n, indent) in enumerate(rows):
        v0, v1 = 0.25 + r * 0.155, 0.25 + r * 0.155 + 0.13
        for c in range(n):
            u0 = 0.065 + (indent * 0.042) + c * 0.0435
            u1 = u0 + 0.039
            p0, p1, p2, p3 = at(u0, v0), at(u1, v0), at(u1, v1), at(u0, v1)
            a.add(f'<path d="M{p0[0]:.1f} {p0[1]:.1f} L{p1[0]:.1f} {p1[1]:.1f} L{p2[0]:.1f} {p2[1]:.1f} L{p3[0]:.1f} {p3[1]:.1f}Z" '
                  f'fill="#5e4a3c"/><path d="M{p0[0] + 1:.1f} {p0[1] + 1:.1f} L{p1[0] - 1:.1f} {p1[1] + 1:.1f} L{p1[0] - 2:.1f} '
                  f'{p1[1] + 4:.1f} L{p0[0] + 2:.1f} {p0[1] + 4:.1f}Z" fill="#7a6452"/>')
    s0, s1, s2, s3 = at(0.3, 0.87), at(0.62, 0.87), at(0.62, 0.92), at(0.3, 0.92)
    a.add(f'<path d="M{s0[0]:.1f} {s0[1]:.1f} L{s1[0]:.1f} {s1[1]:.1f} L{s2[0]:.1f} {s2[1]:.1f} L{s3[0]:.1f} {s3[1]:.1f}Z" fill="#5e4a3c"/>')
    for k in range(4):
        v0, v1 = 0.25 + k * 0.17, 0.25 + k * 0.17 + 0.14
        p0, p1, p2, p3 = at(0.87, v0), at(0.95, v0), at(0.95, v1), at(0.87, v1)
        a.add(f'<path d="M{p0[0]:.1f} {p0[1]:.1f} L{p1[0]:.1f} {p1[1]:.1f} L{p2[0]:.1f} {p2[1]:.1f} L{p3[0]:.1f} {p3[1]:.1f}Z" '
              f'fill="#d9cdb2" stroke="#8c7c62"/>')
        mono(a, (p0[0] + p2[0]) / 2, (p0[1] + p2[1]) / 2 + 4, f'f{2 * k + 1}', 10, '#5a4636', 'middle')
    for k, col in enumerate(('#ef4444', '#f59e0b', '#fde047', '#22c55e', '#3b82f6')):
        x, y = at(0.86 + k * 0.012, 0.08)
        a.rect(x, y, 6, 8, col)
    led = at(0.97, 0.9)
    a.add(f'<rect x="{led[0] - 8:.1f}" y="{led[1] - 2:.1f}" width="10" height="4" rx="2" fill="#ef4444"/>')
    return a


# ---------------------------------------------------------------- arcade cabinets
INV2 = ['..g.....g..', '...g...g...', '..ggggggg..', '.gg.ggg.gg.', 'ggggggggggg', 'g.ggggggg.g', 'g.g.....g.g', '...gg.gg...']


def cabinet_screen(a, kind, x, y, w, h):
    out = [f'<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="#020617"/>']
    rng = random.Random(len(kind))
    out.append(stars(rng, 20, '#475569', 2, y, y + h, x, x + w))
    if kind == 'invaders':
        for r in range(3):
            for c in range(4):
                out.append(sprite(INV2, {'g': ['#4ade80', '#c084fc', '#f472b6'][r]}, x + 10 + c * 26, y + 14 + r * 22, 2, None))
        out.append(f'<rect x="{x + w / 2 - 10}" y="{y + h - 18}" width="20" height="8" fill="#5eead4"/>')
    elif kind == 'racer':
        out.append(f'<rect x="{x}" y="{y + h * 0.4:.0f}" width="{w}" height="{h * 0.6:.0f}" fill="#166534"/>'
                   f'<path d="M{x + w * 0.45:.0f} {y + h * 0.4:.0f} L{x + w * 0.55:.0f} {y + h * 0.4:.0f} L{x + w * 0.9:.0f} {y + h} '
                   f'L{x + w * 0.1:.0f} {y + h}Z" fill="#52525b"/><rect x="{x + w / 2 - 12}" y="{y + h - 26}" width="24" height="14" '
                   f'fill="#dc2626"/><rect x="{x}" y="{y}" width="{w}" height="{h * 0.4:.0f}" fill="#fb923c" opacity="0.6"/>')
    elif kind == 'maze':
        for k in range(6):
            out.append(f'<rect x="{x + 8}" y="{y + 10 + k * 18}" width="{w - 16}" height="3" fill="#2563eb"/>')
        for k in range(10):
            out.append(f'<circle cx="{x + 14 + k * 10}" cy="{y + 20}" r="2" fill="#fde68a"/>')
        out.append(f'<path d="M{x + 40} {y + 56} a10 10 0 1 1 0.1 0 l0 -10z" fill="#fde047"/>'
                   f'<rect x="{x + 70}" y="{y + 48}" width="16" height="16" rx="8" fill="#f87171"/>')
    else:
        out.append(f'<rect x="{x}" y="{y + h - 20}" width="{w}" height="20" fill="#78350f"/>')
        out.append(sprite(['.rr.', 'rffr', '.bb.', 'b..b'], {'r': '#ef4444', 'f': '#fcd5b4', 'b': '#2563eb'}, x + 20, y + h - 40, 5))
        out.append(sprite(['.gg.', 'gffg', '.kk.', 'k..k'], {'g': '#16a34a', 'f': '#fcd5b4', 'k': '#111827'}, x + w - 44, y + h - 40, 5))
    return ''.join(out)


def arcade():
    a = Art('b-arc', 'A row of arcade cabinets in a dark room, their screens and marquees glowing')
    a.rect(0, 0, W, H, '#0b0716')
    for k in range(24):
        for j in range(3):
            a.rect(k * 50, 330 + j * 24, 50, 24, '#1d1233' if (k + j) % 2 else '#120a24')
    cabinets = [('#f472b6', 'GALAXY RAID', 'invaders'), ('#2dd4bf', 'TURBO RUN', 'racer'), ('#f59e0b', 'MAZE MUNCH', 'maze'),
                ('#60a5fa', 'DUEL', 'fighter')]
    for k, (col, title, kind) in enumerate(cabinets):
        x = 80 + k * 270
        a.add(f'<ellipse cx="{x + 90}" cy="380" rx="140" ry="24" fill="{col}" opacity="0.18"{a.blur}/>')
        body = a.gradient(f'cab{k}', [(0, '#2a1f40'), (0.5, '#1a1230'), (1, '#0d0820')], False)
        a.add(f'<path d="M{x} 32 h180 v70 l-22 30 v140 l22 34 v84 h-180 v-84 l22 -34 v-140 l-22 -30z" fill="{body}" '
              f'stroke="{col}" stroke-width="2" stroke-opacity="0.5"/>')
        a.add(f'<path d="M{x + 4} 110 L{x + 22} 136 V270 L{x + 4} 302Z" fill="{col}" opacity="0.25"/>')
        a.add(f'<rect x="{x + 10}" y="40" width="160" height="54" rx="5" fill="{col}"{a.glow}><animate attributeName="opacity" '
              f'values="1;0.75;1" dur="{1.6 + k * 0.3:.1f}s" repeatCount="indefinite"/></rect>')
        a.add(f'<text x="{x + 90}" y="74" font-family="ui-sans-serif, Helvetica, Arial, sans-serif" font-size="17" font-weight="900" '
              f'fill="#0b0716" text-anchor="middle" letter-spacing="1">{title}</text>')
        a.add(f'<rect x="{x + 28}" y="134" width="124" height="128" rx="8" fill="#000"/>')
        cid = a.id(f'scr{k}')
        a.defs.append(f'<clipPath id="{cid}"><rect x="{x + 36}" y="142" width="108" height="112" rx="6"/></clipPath>')
        a.add(f'<g clip-path="url(#{cid})">{cabinet_screen(a, kind, x + 36, 142, 108, 112)}'
              f'<rect x="{x + 36}" y="142" width="108" height="112" fill="{col}" opacity="0.08"/></g>')
        a.add(f'<path d="M{x + 22} 272 H{x + 158} L{x + 176} 300 H{x + 4}Z" fill="#2e2448"/>')
        a.add(f'<rect x="{x + 54}" y="270" width="6" height="18" fill="#111"/><circle cx="{x + 57}" cy="268" r="9" fill="#dc2626"/>'
              f'<circle cx="{x + 54}" cy="265" r="3" fill="#fca5a5"/>')
        for j, bc in enumerate(('#fde68a', '#60a5fa', '#4ade80')):
            a.add(f'<circle cx="{x + 100 + j * 22}" cy="287" r="7" fill="{shade(bc, 0.6)}"/><circle cx="{x + 100 + j * 22}" cy="285" '
                  f'r="6" fill="{bc}"/>')
        a.add(bevel(x + 60, 320, 60, 54, '#231a36', 2))
        for j in range(2):
            a.add(f'<rect x="{x + 70 + j * 24}" y="332" width="14" height="20" rx="2" fill="#f97316"{a.glow}/>')
    return a


# ---------------------------------------------------------------- engines: a wireframe world
def engines():
    a = Art('b-eng', 'A wireframe landscape on a perspective grid under a striped sun, with a spinning wireframe shape above')
    a.rect(0, 0, W, H, a.gradient('sky', [(0, '#05030f'), (1, '#2a0a40')]))
    hz = 210
    sun = a.gradient('sun', [(0, '#fde047'), (1, '#db2777')])
    a.add(f'<circle cx="600" cy="{hz - 10}" r="130" fill="{sun}"{a.glow}/>')
    for k in range(7):
        a.rect(460, hz - 70 + k * 12, 280, 3 + k, '#2a0a40')
    a.rect(0, hz, W, H - hz, a.gradient('ground', [(0, '#1a0630'), (1, '#05010c')]))
    for k in range(-24, 25):
        a.add(f'<path d="M{W / 2 + k * 20} {hz} L{W / 2 + k * 180} {H}" stroke="#c084fc" stroke-width="1.5" opacity="0.6"/>')
    lines = []
    for f in range(6):
        g = []
        for k in range(12):
            t = ((k + f / 6) / 12) ** 2.2
            g.append(f'<path d="M0 {hz + t * (H - hz):.1f} H{W}" stroke="#c084fc" stroke-width="1.5" opacity="0.6"/>')
        lines.append(''.join(g))
    a.add(f'<g{a.glow}>{frames(lines, 0.9)}</g>')
    rng = random.Random(5)
    for layer, col in ((0, '#0d9488'), (1, '#2dd4bf')):
        pts = [(x, hz - 6 - (40 + 60 * layer) * abs(math.sin(x / (170 - layer * 50) + layer)) * rng.uniform(0.6, 1))
               for x in range(0, W + 1, 50)]
        a.add(f'<polygon points="0,{hz} ' + ' '.join(f'{x},{y:.0f}' for x, y in pts) + f' {W},{hz}" fill="#0b0618" stroke="{col}" '
              f'stroke-width="2"/>')
        for x, y in pts[::2]:
            a.add(f'<path d="M{x} {y:.0f} L{x + 50} {hz}" stroke="{col}" stroke-width="1" opacity="0.5"/>')
    phi = (1 + 5 ** 0.5) / 2
    verts = [(-1, phi, 0), (1, phi, 0), (-1, -phi, 0), (1, -phi, 0), (0, -1, phi), (0, 1, phi), (0, -1, -phi), (0, 1, -phi),
             (phi, 0, -1), (phi, 0, 1), (-phi, 0, -1), (-phi, 0, 1)]
    edges = [(i, j) for i in range(12) for j in range(i + 1, 12)
             if abs(sum((verts[i][k] - verts[j][k]) ** 2 for k in range(3)) - 4) < 1e-6]
    bodies = []
    for f in range(24):
        ang = f / 24 * 2 * math.pi / 5
        proj = []
        for x, y, z in verts:
            x2, z2 = x * math.cos(ang) + z * math.sin(ang), -x * math.sin(ang) + z * math.cos(ang)
            y2, z3 = y * math.cos(0.4) - z2 * math.sin(0.4), y * math.sin(0.4) + z2 * math.cos(0.4)
            s = 240 / (z3 + 6)
            proj.append((W / 2 + x2 * s, 110 + y2 * s, z3))
        bodies.append(''.join(f'<path d="M{proj[i][0]:.1f} {proj[i][1]:.1f}L{proj[j][0]:.1f} {proj[j][1]:.1f}" stroke="#fde68a" '
                              f'stroke-width="{2.6 if proj[i][2] + proj[j][2] < 0 else 1.4}" '
                              f'opacity="{1 if proj[i][2] + proj[j][2] < 0 else 0.5}"/>' for i, j in edges))
    a.add(f'<g{a.glow}>{frames(bodies, 2.4)}</g>')
    return a


# ---------------------------------------------------------------- media: floppies, a CD and a tape
def floppy(a, x, y, col, rot, label_lines):
    body = a.gradient(f'fl{x}', [(0, shade(col, 1.3)), (1, shade(col, 0.8))])
    lines = ''.join(f'<rect x="{x + 36}" y="{y + 104 + j * 16}" width="{108 - j * 16}" height="4" rx="2" fill="#94a3b8"/>'
                    for j in range(label_lines))
    return (f'<g transform="rotate({rot} {x + 90} {y + 90})">{shadow(x + 96, y + 186, 90, 8, 0.4)}'
            f'<path d="M{x} {y + 4} Q{x} {y} {x + 4} {y} H{x + 168} L{x + 180} {y + 12} V{y + 176} Q{x + 180} {y + 180} {x + 176} {y + 180} '
            f'H{x + 4} Q{x} {y + 180} {x} {y + 176}Z" fill="{body}"/>'
            f'<rect x="{x + 40}" y="{y}" width="104" height="58" fill="{a.gradient(f"sh{x}", [(0, "#e5e7eb"), (1, "#9ca3af")], False)}"/>'
            f'<rect x="{x + 104}" y="{y + 8}" width="24" height="42" rx="2" fill="{shade(col, 0.6)}"/>'
            f'<rect x="{x + 8}" y="{y + 160}" width="10" height="12" fill="#0f172a"/>'
            f'<rect x="{x + 22}" y="{y + 82}" width="136" height="92" rx="4" fill="#f8fafc"/>'
            f'<rect x="{x + 22}" y="{y + 82}" width="136" height="12" rx="4" fill="{shade(col, 1.5)}"/>{lines}</g>')


def media():
    a = Art('b-media', 'Three floppy disks, a spinning compact disc in its open case and a cassette tape')
    a.rect(0, 0, W, H, a.gradient('bg', [(0, '#1a1f2e'), (1, '#2b3346')]))
    a.add(f'<ellipse cx="600" cy="220" rx="500" ry="170" fill="#60a5fa" opacity="0.07"{a.blur}/>')
    a.add(floppy(a, 70, 80, '#1f2937', -8, 3) + floppy(a, 210, 120, '#1e40af', 6, 4) + floppy(a, 130, 200, '#991b1b', -3, 2))
    a.add(shadow(640, 380, 220, 12, 0.5))
    a.add(bevel(440, 30, 380, 340, '#0f172a', 4))
    a.add('<rect x="452" y="42" width="356" height="316" fill="#1e293b" opacity="0.6"/>')
    cx, cy, r = 630, 200, 150
    a.add(f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="{a.radial("disc", [(0, "#f1f5f9"), (0.7, "#cbd5e1"), (1, "#94a3b8")])}"/>')
    hues = ['#f472b6', '#c084fc', '#60a5fa', '#2dd4bf', '#4ade80', '#fde047', '#fb923c']
    wedges = ''.join(f'<path d="M{cx} {cy} L{cx + r * math.cos(t):.1f} {cy + r * math.sin(t):.1f} A{r} {r} 0 0 1 '
                     f'{cx + r * math.cos(t + 0.22):.1f} {cy + r * math.sin(t + 0.22):.1f}Z" fill="{hues[k % 7]}" opacity="0.32"/>'
                     for k, t in enumerate(x * 0.22 for x in range(29)))
    a.add(f'<g>{wedges}<animateTransform attributeName="transform" type="rotate" from="0 {cx} {cy}" to="360 {cx} {cy}" dur="5s" '
          f'repeatCount="indefinite"/></g>')
    a.add(f'<path d="M{cx - r} {cy} A{r} {r} 0 0 1 {cx} {cy - r} L{cx} {cy - 60} A60 60 0 0 0 {cx - 60} {cy}Z" fill="#fff" opacity="0.18"/>')
    a.add(f'<circle cx="{cx}" cy="{cy}" r="56" fill="#e2e8f0"/><circle cx="{cx}" cy="{cy}" r="56" fill="none" stroke="#94a3b8" stroke-width="2"/>'
          f'<circle cx="{cx}" cy="{cy}" r="34" fill="#cbd5e1" opacity="0.5"/><circle cx="{cx}" cy="{cy}" r="18" fill="#0f172a"/>')
    a.add(shadow(1010, 320, 140, 10, 0.5))
    a.add(f'<rect x="870" y="140" width="270" height="170" rx="12" fill="{a.gradient("tape", [(0, "#2b3240"), (1, "#111827")])}"/>')
    for sx, sy in ((882, 152), (1124, 152), (882, 294), (1124, 294)):
        a.add(f'<circle cx="{sx + 2}" cy="{sy + 2}" r="5" fill="#9ca3af"/><path d="M{sx - 1} {sy + 2}h6" stroke="#374151" stroke-width="1.5"/>')
    a.add(bevel(892, 160, 226, 86, '#f59e0b', 3))
    a.add('<rect x="900" y="168" width="210" height="16" fill="#fef3c7"/>')
    mono(a, 1005, 181, 'SIDE A', 12, '#78350f', 'middle')
    a.add('<rect x="928" y="194" width="154" height="44" rx="22" fill="#111827"/><rect x="964" y="204" width="82" height="24" '
          'fill="#3f2a1a"/>')
    for x, rr in ((952, 22), (1058, 15)):
        a.add(f'<circle cx="{x}" cy="216" r="{rr}" fill="#5a3e22"/>')
        a.add(f'<g><circle cx="{x}" cy="216" r="10" fill="#e5e7eb"/>'
              + ''.join(f'<rect x="{x - 2}" y="207" width="4" height="9" fill="#111827" transform="rotate({rot} {x} 216)"/>'
                        for rot in (0, 120, 240))
              + f'<animateTransform attributeName="transform" type="rotate" from="0 {x} 216" to="360 {x} 216" dur="2s" '
                f'repeatCount="indefinite"/></g>')
    a.add('<path d="M912 310 L936 270 H1074 L1098 310Z" fill="#374151"/><circle cx="960" cy="296" r="6" fill="#111827"/>'
          '<circle cx="1050" cy="296" r="6" fill="#111827"/>')
    return a


# ---------------------------------------------------------------- observe: a lens that magnifies memory (animated)
def observe():
    a = Art('b-obs', 'A magnifying glass over a hex dump of memory, enlarging the bytes beneath it, with one value found')
    a.rect(0, 0, W, H, '#0b1418')
    rng = random.Random(21)
    rows = [[rng.randrange(256) for _ in range(28)] for _ in range(15)]
    rows[6][12], rows[6][13] = 0xe8, 0x03

    def dump(highlight_op):
        out = []
        for r, row in enumerate(rows):
            y = 34 + r * 25
            out.append(f'<text x="24" y="{y}" font-family="{MONO}" font-size="15" font-weight="600" fill="#2f5f5a">'
                       f'{0x4a1f00 + r * 28:06x}</text>')
            for c, b in enumerate(row):
                hit = r == 6 and c in (12, 13)
                if hit:
                    out.append(f'<rect x="{117 + c * 37}" y="{y - 15}" width="30" height="20" rx="3" fill="#f59e0b" '
                               f'opacity="{highlight_op}"><animate attributeName="opacity" values="0.25;0.7;0.25" dur="1.6s" '
                               f'repeatCount="indefinite"/></rect>')
                out.append(f'<text x="{120 + c * 37}" y="{y}" font-family="{MONO}" font-size="15" font-weight="{"800" if hit else "500"}" '
                           f'fill="{"#fde68a" if hit else "#3f8a80"}">{b:02x}</text>')
        return ''.join(out)
    a.add(dump(0.4))
    a.add(f'<rect x="0" y="0" width="{W}" height="26" fill="#2dd4bf" opacity="0.08"><animate attributeName="y" values="0;380;0" '
          f'dur="8s" repeatCount="indefinite"/></rect>')
    lx, ly, lr, zoom = 600, 168, 104, 1.9
    lid = a.id('lens')
    a.defs.append(f'<clipPath id="{lid}"><circle cx="{lx}" cy="{ly}" r="{lr}"/></clipPath>')
    a.add(f'<g clip-path="url(#{lid})"><rect x="{lx - lr}" y="{ly - lr}" width="{2 * lr}" height="{2 * lr}" fill="#0f2a2e"/>'
          f'<g transform="translate({lx} {ly}) scale({zoom}) translate({-lx} {-ly})">{dump(0.5)}</g>'
          f'<circle cx="{lx - 30}" cy="{ly - 40}" r="{lr}" fill="#fff" opacity="0.05"/></g>')
    a.add(f'<circle cx="{lx}" cy="{ly}" r="{lr + 6}" fill="none" stroke="#111827" stroke-width="6"/>'
          f'<circle cx="{lx}" cy="{ly}" r="{lr}" fill="none" stroke="{a.gradient("rim", [(0, "#f1f5f9"), (1, "#6b7280")])}" '
          f'stroke-width="12"/>'
          f'<path d="M{lx + 78} {ly + 78} L{lx + 160} {ly + 160}" stroke="#3f2a1a" stroke-width="28" stroke-linecap="round"/>'
          f'<path d="M{lx + 78} {ly + 78} L{lx + 160} {ly + 160}" stroke="#7c5a3a" stroke-width="18" stroke-linecap="round"/>'
          f'<path d="M{lx + 72} {ly + 72} L{lx + 92} {ly + 92}" stroke="#9ca3af" stroke-width="30"/>')
    a.add(bevel(860, 290, 300, 70, '#111f24', 3))
    mono(a, 880, 320, f'0x{0x4a1f00 + 6 * 28 + 12:x}: e8 03', 18, '#fde68a')
    mono(a, 880, 346, 'gold = 1000', 18, '#5eead4')
    return a


# ---------------------------------------------------------------- working: a terminal and an agent at work (animated)
def working():
    a = Art('b-work', 'A terminal where an agent works through the steps of the method, line by line')
    a.rect(0, 0, W, H, a.gradient('bg', [(0, '#0f172a'), (1, '#1e293b')]))
    a.add('<rect x="60" y="30" width="700" height="340" rx="12" fill="#020617" stroke="#334155" stroke-width="2"/>'
          '<circle cx="86" cy="54" r="7" fill="#f87171"/><circle cx="108" cy="54" r="7" fill="#fbbf24"/>'
          '<circle cx="130" cy="54" r="7" fill="#4ade80"/>')
    lines = [('$ ', 'tools/pe/peinfo.py target/GAME.EXE', '#e2e8f0'), ('  ', 'MSVC 6.0, DirectDraw, 1 PDB path', '#64748b'),
             ('> ', 're-tracer: rng_next at 0x413a10', '#5eead4'), ('> ', 're-measurer: 1000 draws captured', '#5eead4'),
             ('> ', 're-porter: crates/game/src/rng.rs', '#5eead4'), ('$ ', 'cargo test rng', '#e2e8f0'),
             ('  ', 'test rng_matches_original ... ok', '#4ade80'), ('> ', 're-lander: committed 3 files', '#fde68a')]
    n, dur = len(lines), 9
    for k, (p, t, col) in enumerate(lines):
        y = 96 + k * 34
        t0 = k / (n + 1)
        a.add(f'<g opacity="0"><text x="88" y="{y}" font-family="{MONO}" font-size="18" font-weight="600" fill="#94a3b8">{p}'
              f'<tspan fill="{col}">{t}</tspan></text><animate attributeName="opacity" values="0;0;1;1" '
              f'keyTimes="0;{t0:.3f};{t0:.3f};1" dur="{dur}s" repeatCount="indefinite"/></g>')
    steps = ['inventory', 'trace', 'measure', 'port', 'verify', 'land']
    for k, name in enumerate(steps):
        y = 50 + k * 54
        col = ['#64748b', '#60a5fa', '#f59e0b', '#2dd4bf', '#4ade80', '#c084fc'][k]
        a.add(f'<rect x="830" y="{y}" width="300" height="40" rx="8" fill="{col}" fill-opacity="0.2" stroke="{col}"/>')
        mono(a, 980, y + 26, name, 18, '#e2e8f0', 'middle')
        if k:
            a.add(f'<path d="M980 {y - 14} V{y - 2}" stroke="#64748b" stroke-width="2"/>')
    return a


# ---------------------------------------------------------------- preservation: an archive shelf
def preservation():
    a = Art('b-pres', 'Archive shelves holding big-box games, CDs in jewel cases, a box of floppy disks, cartridges and '
                      'labelled archive boxes')
    a.rect(0, 0, W, H, a.gradient('wall', [(0, '#1c1917'), (1, '#2a2521')]))
    rng = random.Random(31)
    palette = ['#b91c1c', '#1d4ed8', '#047857', '#a16207', '#7c3aed', '#0e7490', '#be185d', '#4b5563', '#c2410c']
    titles = ['STARFALL', 'DUNGEON', 'ISLAND', 'ORBIT', 'TOWNS', 'RAIDER', 'SANDS', 'WASTES', 'LEGION', 'EMPIRE', 'BURROW', 'ARENA']

    def big_box(x, base, h):
        w = rng.randrange(30, 44)
        col = rng.choice(palette)
        g = a.gradient(f'bb{x}{base}', [(0, shade(col, 0.7)), (0.4, shade(col, 1.15)), (1, shade(col, 0.6))], False)
        title = rng.choice(titles)
        a.add(f'<rect x="{x}" y="{base - h}" width="{w}" height="{h}" fill="{g}"/>'
              f'<rect x="{x + 3}" y="{base - h + 8}" width="{w - 6}" height="18" fill="#f8fafc" opacity="0.85"/>'
              f'<text x="{x + w / 2}" y="{base - 26}" font-family="ui-sans-serif, Helvetica, Arial, sans-serif" font-size="11" '
              f'font-weight="800" fill="#f8fafc" text-anchor="middle" transform="rotate(-90 {x + w / 2} {base - 26})">{title}</text>'
              f'<rect x="{x}" y="{base - h}" width="2" height="{h}" fill="#fff" opacity="0.15"/>')
        return w

    def jewel_cases(x, base, n):
        for k in range(n):
            col = rng.choice(palette)
            a.add(f'<rect x="{x + k * 11}" y="{base - 126}" width="10" height="126" fill="#e5e7eb" opacity="0.35"/>'
                  f'<rect x="{x + k * 11 + 2}" y="{base - 124}" width="6" height="122" fill="{col}"/>'
                  f'<rect x="{x + k * 11 + 2}" y="{base - 112}" width="6" height="30" fill="#f8fafc" opacity="0.7"/>'
                  f'<rect x="{x + k * 11}" y="{base - 126}" width="1.5" height="126" fill="#fff" opacity="0.6"/>')
        return n * 11

    def jewel_front(x, base):
        a.add(shadow(x + 64, base, 64, 5, 0.4))
        a.add(f'<rect x="{x}" y="{base - 128}" width="128" height="124" fill="#e5e7eb" opacity="0.3"/>'
              f'<rect x="{x + 10}" y="{base - 124}" width="114" height="116" fill="{a.gradient(f"cover{x}", [(0, "#1e1b4b"), (1, "#7c3aed")])}"/>'
              f'<circle cx="{x + 67}" cy="{base - 76}" r="30" fill="#fde047" opacity="0.8"/>'
              f'<text x="{x + 67}" y="{base - 22}" font-family="ui-sans-serif, Helvetica, Arial, sans-serif" font-size="14" '
              f'font-weight="900" fill="#f8fafc" text-anchor="middle">ODYSSEY</text>'
              f'<rect x="{x}" y="{base - 128}" width="10" height="124" fill="#9ca3af" opacity="0.5"/>'
              f'<path d="M{x + 10} {base - 124} L{x + 60} {base - 124} L{x + 10} {base - 60}Z" fill="#fff" opacity="0.12"/>'
              f'<rect x="{x - 4}" y="{base - 6}" width="136" height="6" fill="#3f2a1a"/>')
        return 136

    def floppy_box(x, base):
        a.add(f'<path d="M{x} {base} V{base - 70} L{x + 14} {base - 84} H{x + 140} V{base} Z" fill="#374151"/>')
        for k in range(10):
            col = rng.choice(['#111827', '#1d4ed8', '#991b1b', '#f8fafc'])
            a.add(f'<rect x="{x + 6 + k * 13}" y="{base - 112 + (k % 3) * 4}" width="11" height="44" fill="{col}"/>'
                  f'<rect x="{x + 6 + k * 13}" y="{base - 112 + (k % 3) * 4}" width="11" height="8" fill="#f8fafc" opacity="0.8"/>')
        a.add(f'<rect x="{x}" y="{base - 70}" width="140" height="70" fill="{a.gradient(f"fb{x}", [(0, "#4b5563"), (1, "#1f2937")])}"/>'
              f'<rect x="{x + 20}" y="{base - 52}" width="100" height="24" fill="#f8fafc" opacity="0.85"/>'
              f'<text x="{x + 70}" y="{base - 35}" font-family="{MONO}" font-size="11" font-weight="700" fill="#111827" '
              f'text-anchor="middle">3.5" DISKS</text>')
        return 148

    def archive_box(x, base, label):
        a.add(bevel(x, base - 96, 150, 96, '#a8a29e', 3))
        a.add(f'<rect x="{x + 30}" y="{base - 74}" width="90" height="40" fill="#f8fafc"/>'
              f'<text x="{x + 75}" y="{base - 58}" font-family="{MONO}" font-size="11" font-weight="700" fill="#111827" '
              f'text-anchor="middle">{label}</text><text x="{x + 75}" y="{base - 42}" font-family="{MONO}" font-size="10" '
              f'fill="#4b5563" text-anchor="middle">SHA-256 OK</text>'
              f'<rect x="{x + 66}" y="{base - 22}" width="18" height="10" rx="5" fill="#57534e"/>')
        return 158

    def cartridges(x, base, n):
        for k in range(n):
            col = rng.choice(palette)
            a.add(bevel(x + k * 30, base - 78, 26, 78, '#6b7280', 2))
            a.add(f'<rect x="{x + k * 30 + 4}" y="{base - 66}" width="18" height="44" fill="{col}"/>')
        return n * 30 + 8

    for shelf in range(3):
        base = 128 + shelf * 132
        a.add(f'<rect x="0" y="{base - 132}" width="{W}" height="132" fill="#000" opacity="0.12"/>')
        x = 40
        plan = [['big', 'big', 'big', 'jewel', 'front', 'big', 'big', 'archive', 'big', 'big', 'cart'],
                ['archive', 'big', 'big', 'big', 'jewel', 'floppy', 'big', 'big', 'jewel', 'big'],
                ['big', 'big', 'cart', 'big', 'big', 'front', 'jewel', 'big', 'big', 'archive', 'big']][shelf]
        for item in plan:
            if x > 1140:
                break
            if item == 'big':
                x += big_box(x, base, rng.randrange(100, 124)) + 3
            elif item == 'jewel':
                x += jewel_cases(x, base, rng.randrange(6, 11)) + 8
            elif item == 'front':
                x += jewel_front(x, base) + 8
            elif item == 'floppy':
                x += floppy_box(x, base) + 8
            elif item == 'archive':
                x += archive_box(x, base, rng.choice(['GAME 1997', 'MASTERS', 'BETA 0.9', 'MANUALS']))
            else:
                x += cartridges(x, base, 4)
        a.add(f'<rect x="0" y="{base}" width="{W}" height="14" fill="{a.gradient(f"board{shelf}", [(0, "#7a5a3c"), (1, "#4a3424")])}"/>'
              f'<rect x="0" y="{base}" width="{W}" height="2" fill="#9a7a5a"/>{shadow(600, base + 18, 600, 6, 0.4)}')
    a.add(f'<rect x="0" y="0" width="{W}" height="{H}" fill="{a.radial("light", [(0, "#fde68a"), (1, "#000000")], 0.5, 0, 1)}" '
          f'opacity="0.18"/>')
    return a


# ---------------------------------------------------------------- the home page hero (animated)
def hero():
    a = Art('b-hero', 'Bytes from a game\'s executable flow through decompiling, tracing and porting, and a game scene '
                      'is rebuilt tile by tile')
    a.rect(0, 0, W, H, a.gradient('bg', [(0, '#06121a'), (1, '#0d1b1e')]))
    rng = random.Random(41)
    for c in range(8):
        col = ''.join(f'<text x="{40 + c * 38}" y="{k * 26}" font-family="{MONO}" font-size="17" font-weight="600" '
                      f'fill="{"#5eead4" if rng.random() < 0.15 else "#2f5f5a"}">{rng.randrange(256):02x}</text>'
                      for k in range(32))
        dur = 6 + c % 3 * 2
        a.add(f'<g>{col}<animateTransform attributeName="transform" type="translate" from="0 -416" to="0 0" dur="{dur}s" '
              f'repeatCount="indefinite"/></g>')
    a.add(f'<rect x="0" y="0" width="360" height="{H}" fill="url(#b-hero-fade)"/>')
    a.defs.append('<linearGradient id="b-hero-fade" x1="0" x2="1"><stop offset="0.6" stop-color="#0d1b1e" stop-opacity="0"/>'
                  '<stop offset="1" stop-color="#0d1b1e" stop-opacity="1"/></linearGradient>')
    steps = [('decompile', '#60a5fa'), ('trace', '#f59e0b'), ('port', '#2dd4bf')]
    for k, (name, col) in enumerate(steps):
        y = 100 + k * 80
        a.add(f'<rect x="400" y="{y}" width="190" height="52" rx="10" fill="{col}" fill-opacity="0.16" stroke="{col}" '
              f'stroke-width="2"/>')
        mono(a, 495, y + 33, name, 20, '#e2e8f0', 'middle')
        a.add(f'<circle r="5" fill="{col}"><animateMotion dur="2s" begin="{k * 0.6:.1f}s" repeatCount="indefinite" '
              f'path="M360 {y + 26} H400 M590 {y + 26} H650"/></circle>')
        a.add(f'<path d="M360 {y + 26} H398 M592 {y + 26} H650" stroke="{col}" stroke-width="2" stroke-dasharray="4 6" '
              f'opacity="0.6"/>')
    gx, gy, t = 666, 30, 33
    cid = a.id('game')
    a.defs.append(f'<clipPath id="{cid}"><rect x="{gx}" y="{gy}" width="{15 * t}" height="{10 * t}" rx="12"/></clipPath>')
    game = [f'<rect x="{gx}" y="{gy}" width="{15 * t}" height="{10 * t}" fill="{a.gradient("gsky", [(0, "#3b82f6"), (1, "#93c5fd")])}"/>']
    for hx, hr, col in ((gx + 80, 120, '#16a34a'), (gx + 330, 160, '#15803d'), (gx + 470, 100, '#16a34a')):
        game.append(f'<circle cx="{hx}" cy="{gy + 10 * t - 40}" r="{hr}" fill="{col}" opacity="0.55"/>')
    for cx, cy in ((gx + 90, gy + 50), (gx + 330, gy + 36)):
        game.append(f'<g><ellipse cx="{cx}" cy="{cy}" rx="34" ry="14" fill="#fff"/><ellipse cx="{cx + 22}" cy="{cy - 8}" rx="22" ry="14" '
                    f'fill="#fff"/><animateTransform attributeName="transform" type="translate" values="0 0;30 0;0 0" dur="12s" '
                    f'repeatCount="indefinite"/></g>')
    level = ['...............', '...............', '..........ccc..', '....?.?........', '...............',
             '.........BBBB..', '..BB?BB........', '..........P....', 'GGGGGGG..GGGGGG', 'DDDDDDD..DDDDDD']
    pal = {'G': '#4ade80', 'D': '#a0522d', 'B': '#c2410c', '?': '#f59e0b', 'P': '#15803d'}
    cells = [(r, c) for r in range(10) for c in range(15) if level[r][c] != '.']
    rng.shuffle(cells)
    n, dur = len(cells), 9
    for k, (r, c) in enumerate(cells):
        ch, x, y = level[r][c], gx + c * t, gy + r * t
        if ch == 'c':
            body = f'<ellipse cx="{x + t / 2}" cy="{y + t / 2}" rx="8" ry="12" fill="#fde047" stroke="#a16207" stroke-width="3"/>'
        elif ch == 'P':
            body = (bevel(x - 4, y, t + 8, 14, '#22c55e', 3) + bevel(x, y + 14, t, t - 14, '#15803d', 3)
                    + bevel(x, y + t, t, t, '#15803d', 3))
        else:
            body = bevel(x, y, t - 1, t - 1, pal[ch], 4)
            if ch == 'B':
                body += f'<path d="M{x} {y + t / 2}h{t}M{x + t / 2} {y}v{t / 2}M{x + t / 4} {y + t / 2}v{t / 2}" stroke="#7c2d12" stroke-width="2"/>'
            elif ch == '?':
                body += f'<text x="{x + t / 2}" y="{y + t - 9}" font-family="{MONO}" font-size="20" font-weight="800" fill="#7c2d12" text-anchor="middle">?</text>'
        t0 = 0.03 + 0.3 * k / n
        game.append(f'<g opacity="1">{body}<animate attributeName="opacity" values="1;0;0;1;1" '
                    f'keyTimes="0;0.01;{t0:.3f};{t0 + 0.01:.3f};1" calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/></g>')
    # The hero runs the level once the blocks are in: onto both platforms, a jump at the pole, then a slide to the
    # ground with the flag. Times are seconds in the blocks' cycle; the sprite stands 36 above what it stands on.
    pole, slide = gx + 13 * t + 16, (6.2, 7.2)
    stand = lambda top: top - 36
    path = [(0, gx + 10, stand(gy + 8 * t)), (3.2, gx + 10, stand(gy + 8 * t)), (3.5, gx + 24, stand(gy + 8 * t))]
    def jump(t1, x1, y1, h):
        t0, x0, y0 = path[-1]
        for i in range(1, 9):
            f = i / 8
            path.append((t0 + (t1 - t0) * f, x0 + (x1 - x0) * f, y0 + (y1 - y0) * f - 4 * h * f * (1 - f)))
    jump(4.1, gx + 84, stand(gy + 6 * t), 60)
    path.append((4.8, gx + 196, stand(gy + 6 * t)))
    jump(5.4, gx + 312, stand(gy + 5 * t), 50)
    path.append((5.7, gx + 356, stand(gy + 5 * t)))
    jump(slide[0], pole - 28, gy + 2 * t + 16, 40)
    path += [(slide[1], pole - 28, stand(gy + 8 * t)), (dur, pole - 28, stand(gy + 8 * t))]
    drop = 6 * t - 24
    game.append(f'<path d="M{pole} {gy + 2 * t}V{gy + 8 * t}" stroke="#e5e7eb" stroke-width="4"/>'
                f'<g><path d="M{pole + 2} {gy + 2 * t}h30l-10 10l10 10h-30z" fill="#ef4444"/><animateTransform '
                f'attributeName="transform" type="translate" values="0 0;0 0;0 {drop};0 {drop}" '
                f'keyTimes="0;{slide[0] / dur:.3f};{slide[1] / dur:.3f};1" dur="{dur}s" repeatCount="indefinite"/></g>')
    hero_sprite = sprite(['.rrrr.', 'rrrrrr', '.ssss.', '.sesss'[:6], 'tttttt', 'tt..tt', 'bb..bb'],
                         {'r': '#ef4444', 's': '#fcd5b4', 'e': '#1f2937', 't': '#2563eb', 'b': '#7c2d12'}, 0, 0, 5)
    game.append(f'<g opacity="1">{hero_sprite}<animateTransform attributeName="transform" type="translate" '
                f'values="{";".join(f"{x:.0f} {y:.0f}" for _, x, y in path)}" '
                f'keyTimes="{";".join(f"{k / dur:.4f}" for k, _, _ in path)}" dur="{dur}s" repeatCount="indefinite"/>'
                f'<animate attributeName="opacity" values="1;0;0;1;1" keyTimes="0;0.01;0.34;0.35;1" calcMode="discrete" '
                f'dur="{dur}s" repeatCount="indefinite"/></g>')
    a.add(f'<g clip-path="url(#{cid})">{"".join(game)}</g>')
    a.add(f'<rect x="{gx}" y="{gy}" width="{15 * t}" height="{10 * t}" rx="12" fill="none" stroke="#2dd4bf" stroke-width="3"{a.glow}/>')
    mono(a, gx + 15 * t / 2, gy + 10 * t + 30, 'the rebuilt game', 16, '#5eead4', 'middle', '600')
    a.add('<rect x="70" y="364" width="220" height="28" rx="6" fill="#0d1b1e" opacity="0.92"/>')
    mono(a, 180, H - 14, "the original's bytes", 16, '#5eead4', 'middle', '600')
    return a


BANNERS = {'hardware': hardware, 'consoles': consoles, 'handhelds': handhelds, 'computers': computers, 'arcade': arcade,
           'engines': engines, 'media': media, 'observe': observe, 'working': working, 'preservation': preservation,
           'hero': hero}
DIAGRAMS = {f'banner-{name}.svg': (lambda fn=fn: fn().render()) for name, fn in BANNERS.items()}
