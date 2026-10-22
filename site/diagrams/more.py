"""The second set of the guides' diagrams: kinds of games, an agent hand-off, stack frames, the
fixed-step loop, unpacking, a jump table, a scene graph update, the OPL2's operators and the
consoles by generation. site/diagrams/make.py writes them with the first set.
"""
import math

from svg import AMBER, BG, BLUE, DIM, GREEN, GREY, PURPLE, RED, TEAL, TEXT, Svg


# ---------------------------------------------------------------- kinds of games
def game_families():
    s = Svg(900, 420, 'Families of games and who drives each: a stream, a simulation, an environment, or turns')
    s.heading('Who drives each family of games', 'Where the behaviour lives decides where the reverse engineering work is.')
    cols = [('a stream', 'the frame loop', TEAL), ('a simulation', 'the simulation step', AMBER),
            ('an environment', 'triggers and scripts', BLUE), ('turns', 'handlers and scripts', PURPLE)]
    rows = [('Arcade action', [0]), ('First-person and 3D action', [0, 2]), ('Racing and sports', [0]),
            ('Vehicle and space simulations', [0, 2]), ('City building and management', [1]), ('Strategy', [1, 3]),
            ('Role-playing games', [2, 3]), ('Adventure and story', [3]), ('Puzzle, board and card', [3, 0]),
            ('Online games', [2, 1])]
    x0, y0, cw, rh = 300, 110, 140, 27
    for i, (name, sub, colour) in enumerate(cols):
        cx = x0 + i * cw + cw / 2
        s.text(cx, y0 - 26, name, 13, colour, 'middle', '600')
        s.text(cx, y0 - 10, sub, 11, DIM, 'middle')
    for j, (family, drives) in enumerate(rows):
        y = y0 + j * rh
        if j % 2 == 0:
            s.rect(24, y - 4, 852, rh, '#12282a', rx=4)
        s.text(40, y + 14, family, 13, TEXT)
        for k, c in enumerate(drives):
            colour = cols[c][2]
            cx = x0 + c * cw + cw / 2
            s.add(f'<circle cx="{cx:.1f}" cy="{y + 9:.1f}" r="{7 if k == 0 else 5}" fill="{colour}" '
                  f'fill-opacity="{1 if k == 0 else 0.55}"/>')
    s.text(40, y0 + len(rows) * rh + 22, 'Large dot: the main driver. Small dot: also present. From docs/games/README.md.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- agent hand-off (animated)
def team_handoff():
    s = Svg(900, 330, 'The main session sends one agent at a time; each writes files and returns a hand-off that names the next agent')
    s.heading('A hand-off between agents', 'The main session sends a brief, reads the hand-off, checks the evidence, and sends the next agent.')
    agents = [('re-tracer', 90), ('re-measurer', 450), ('re-porter', 810)]
    my, ay, fy = 104, 190, 280
    s.rect(330, my - 22, 240, 44, TEAL, rx=8, extra=' fill-opacity="0.18"')
    s.text(450, my + 5, 'main session', 14, TEXT, 'middle', '600')
    for name, x in agents:
        s.rect(x - 75, ay - 20, 150, 40, BLUE, rx=8, extra=' fill-opacity="0.18"')
        s.text(x, ay + 5, name, 13, TEXT, 'middle', '600')
    s.rect(160, fy - 18, 580, 34, GREY, rx=6, extra=' fill-opacity="0.25"')
    s.text(450, fy + 4, 'project files: docs/, labels/, derived/, the port', 12, DIM, 'middle')
    # One token goes: main -> agent (brief), agent -> files (writes), agent -> main (hand-off), for each agent.
    legs = []
    for name, x in agents:
        legs += [((430, my + 22), (x - 18, ay - 20), AMBER, 'brief'),
                 ((x, ay + 20), (x if x != 810 else 700, fy - 18), GREY, 'writes'),
                 ((x + 18, ay - 20), (470, my + 22), GREEN, 'hand-off')]
    n = len(legs)
    dur = n * 1.2
    for i, ((x1, y1), (x2, y2), colour, label) in enumerate(legs):
        begin = i * 1.2
        anim = (f'<animate attributeName="opacity" values="0;1;1;0" keyTimes="0;0.1;0.9;1" dur="1.2s" begin="{begin}s;'
                f'loop.end+{begin}s" fill="freeze"/>')
        s.add(f'<g opacity="0"><path d="M{x1} {y1}L{x2} {y2}" stroke="{colour}" stroke-width="2.5" marker-end="url(#arrow)"/>'
              f'<circle r="6" fill="{colour}"><animateMotion dur="1.2s" begin="{begin}s;loop.end+{begin}s" '
              f'path="M{x1} {y1}L{x2} {y2}" fill="freeze"/></circle>'
              f'<text x="{(x1 + x2) / 2 + (-8 if label == "brief" else 8):.1f}" y="{(y1 + y2) / 2 - 10:.1f}" '
              f'text-anchor="{"end" if label == "brief" else "start"}" font-family="ui-sans-serif, Helvetica, Arial, sans-serif" '
              f'font-size="11" fill="{colour}">{label}</text>{anim}</g>')
    s.add(f'<rect width="1" height="1" fill="none"><animate id="loop" attributeName="x" from="0" to="0" dur="{dur}s" '
          f'begin="0s;loop.end"/></rect>')
    s.text(24, 316, 'Hand-off: Done, Evidence, Open, Next. The Next line names the agent the main session sends next.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- stack frames
def abi_stack():
    s = Svg(900, 380, 'The stack at a function\'s start: Windows x86 cdecl with three arguments, and Windows x64 with five')
    s.heading('Arguments at a function\'s first instruction', 'Higher addresses at the top. The stack grows down.')

    def column(x, title, cells):
        s.text(x, 86, title, 14, TEXT, weight='600')
        y = 100
        for label, where, colour in cells:
            s.rect(x, y, 230, 30, colour, rx=3, extra=' fill-opacity="0.22"')
            s.rect(x, y, 4, 30, colour, rx=1)
            s.text(x + 14, y + 20, label, 12, TEXT)
            s.text(x + 226, y + 20, where, 12, DIM, 'end', mono=True)
            y += 34
    column(40, 'x86 cdecl: f(a, b, c)', [
        ('c (pushed first)', '[esp+12]', AMBER), ('b', '[esp+8]', AMBER), ('a (pushed last)', '[esp+4]', AMBER),
        ('return address', '[esp]', RED)])
    s.text(40, 260, 'After push ebp; mov ebp, esp the arguments are at', 11, DIM)
    s.text(40, 276, '[ebp+8], [ebp+12], [ebp+16]. The caller removes them.', 11, DIM)
    column(330, 'Windows x64: f(a, b, c, d, e)', [
        ('e (fifth argument)', '[rsp+0x28]', AMBER), ('shadow space for d', '[rsp+0x20]', GREY),
        ('shadow space for c', '[rsp+0x18]', GREY), ('shadow space for b', '[rsp+0x10]', GREY),
        ('shadow space for a', '[rsp+0x08]', GREY), ('return address', '[rsp]', RED)])
    s.text(620, 86, 'x64: the first four in registers', 14, TEXT, weight='600')
    y = 100
    for label, reg in [('a', 'rcx'), ('b', 'rdx'), ('c', 'r8'), ('d', 'r9')]:
        s.rect(620, y, 230, 30, BLUE, rx=3, extra=' fill-opacity="0.22"')
        s.rect(620, y, 4, 30, BLUE, rx=1)
        s.text(634, y + 20, label, 12, TEXT)
        s.text(846, y + 20, reg, 12, BLUE, 'end', mono=True)
        y += 34
    s.text(620, 268, 'Floats go in xmm0 to xmm3 instead,', 12, TEXT)
    s.text(620, 286, 'by position: a float second argument', 12, TEXT)
    s.text(620, 304, 'is in xmm1, and rdx is unused.', 12, TEXT)
    s.text(620, 334, 'The caller reserves the 32 bytes of', 12, TEXT)
    s.text(620, 352, 'shadow space even for fewer arguments.', 12, TEXT)
    s.text(40, 352, 'docs/ABI.md lists every convention.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- fixed-step loop (animated)
def fixed_step():
    s = Svg(900, 330, 'A fixed-step loop: frames of different lengths add time to an accumulator, and each full step runs one update')
    s.heading('The fixed-step main loop', 'Frames take different times. The game still updates in equal steps of 1/60 second (16.7 ms).')
    frames = [12, 22, 16, 34, 9, 18, 15, 25]
    step = 1000 / 60
    x0, scale, y = 100, 4.2, 96
    acc, t = 0.0, 0.0
    steps = []
    for i, f in enumerate(frames):
        x = x0 + t * scale
        s.rect(x, y, f * scale - 2, 34, BLUE, rx=3, extra=' fill-opacity="0.3"')
        s.text(x + f * scale / 2, y + 22, f'{f} ms', 11, TEXT, 'middle')
        acc += f
        n = 0
        while acc >= step:
            acc -= step
            n += 1
        steps.append((x + f * scale, n, acc))
        t += f
    s.text(x0, y - 8, 'frames (time to draw each)', 11, DIM)
    s.text(x0, y + 62, 'updates run at the end of each frame', 11, DIM)
    for i, (x, n, rest) in enumerate(steps):
        for k in range(n):
            s.rect(x - 14 - k * 16, y + 72, 12, 26, AMBER, rx=2)
        s.text(x - 8, y + 116, str(n), 12, AMBER if n else DIM, 'middle', '600')
        s.text(x - 8, y + 134, f'{rest:.1f}', 10, DIM, 'middle')
    s.text(24, y + 116, 'steps', 11, AMBER)
    s.text(24, y + 134, 'left (ms)', 10, DIM)
    total = sum(frames) * scale
    dur = sum(frames) / 40
    s.add(f'<path d="M{x0} {y - 4}V{y + 102}" stroke="{TEXT}" stroke-width="2"><animateTransform attributeName="transform" '
          f'type="translate" from="0 0" to="{total:.1f} 0" dur="{dur:.2f}s" repeatCount="indefinite"/></path>')
    s.text(24, 300, 'One frame may run no update, another two. A port must keep the same accumulator, or its results drift.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- unpacking (animated)
def unpacking():
    s = Svg(900, 300, 'A packed program: the stub unpacks the code into memory, then jumps to the original entry point')
    s.heading('A packed program at start-up', 'The packed file holds a small stub and compressed code. The stub rebuilds the code in memory.')
    y1, y2 = 96, 196
    s.text(40, y1 - 8, 'the file on disk', 11, DIM)
    s.rect(40, y1, 90, 40, AMBER, rx=4, extra=' fill-opacity="0.85"')
    s.text(85, y1 + 25, 'stub', 12, BG, 'middle', '600')
    for i in range(30):
        shade = ['#dc1e1e', '#f0dc28', '#e88a1e'][i % 3]
        s.rect(134 + i * 9, y1, 8, 40, shade, rx=1, extra=' fill-opacity="0.75"')
    s.text(270, y1 + 58, 'compressed code: entropy near 8', 11, DIM, 'middle')
    s.text(100, y2 - 8, 'memory after the stub has run', 11, DIM)
    s.rect(40, y2, 90, 40, AMBER, rx=4, extra=' fill-opacity="0.85"')
    s.text(85, y2 + 25, 'stub', 12, BG, 'middle', '600')
    s.rect(134, y2, 600, 40, '#16302f', rx=4)
    s.add(f'<rect x="134" y="{y2}" height="40" rx="4" fill="{TEAL}" fill-opacity="0.85" width="0">'
          f'<animate attributeName="width" values="0;600;600;0" keyTimes="0;0.6;0.95;1" dur="6s" repeatCount="indefinite"/></rect>')
    s.text(434, y2 + 25, 'the original code, rebuilt', 12, BG, 'middle', '600')
    s.add(f'<path d="M85 {y1 + 42}V{y2 - 4}" stroke="{DIM}" stroke-width="2" marker-end="url(#arrow)"/>')
    s.add(f'<g opacity="0"><path d="M130 {y2 + 52}Q300 {y2 + 90} 520 {y2 + 44}" fill="none" stroke="{GREEN}" stroke-width="2.5" '
          f'marker-end="url(#arrow)"/><text x="530" y="{y2 + 74}" font-family="ui-sans-serif, Helvetica, Arial, sans-serif" '
          f'font-size="12" fill="{GREEN}">jump to the original entry point: dump the memory here</text>'
          f'<animate attributeName="opacity" values="0;0;1;1;0" keyTimes="0;0.6;0.65;0.95;1" dur="6s" repeatCount="indefinite"/></g>')
    s.add(f'<path d="M520 {y2 + 42}V{y2 + 2}" stroke="{GREEN}" stroke-width="2" opacity="0"/>')
    return s.render()


# ---------------------------------------------------------------- jump table
def jump_table():
    s = Svg(900, 330, 'A switch statement compiled to a jump table: a bounds check, then an indirect jump through a table of addresses')
    s.heading('A switch as a jump table', 'One bounds check, then one indirect jump. The table holds an address for each case.')
    code = [('cmp   eax, 4', TEXT), ('ja    default', TEXT), ('jmp   [table + eax*4]', AMBER)]
    s.rect(40, 84, 250, 96, '#12282a', rx=6)
    for i, (line, colour) in enumerate(code):
        s.text(56, 112 + i * 26, line, 13, colour, mono=True)
    s.text(40, 200, 'switch (kind) {', 12, DIM, mono=True)
    s.text(40, 216, '  case 0: ... case 4: ...', 12, DIM, mono=True)
    s.text(40, 232, '  default: ... }', 12, DIM, mono=True)
    tx, ty = 360, 84
    s.text(tx, ty - 8, 'table (read with readdata.py)', 11, DIM)
    targets = ['case_0', 'case_1', 'case_2', 'case_1', 'case_4']
    for i, t in enumerate(targets):
        s.rect(tx, ty + i * 32, 170, 28, BLUE, rx=3, extra=' fill-opacity="0.22"')
        s.text(tx + 10, ty + i * 32 + 19, f'[{i}]  0x4012{0x30 + 0x20 * i:02x}', 12, TEXT, mono=True)
    s.add(f'<path d="M292 162C320 162 330 {ty + 14} {tx - 4} {ty + 14}" fill="none" stroke="{AMBER}" stroke-width="2" marker-end="url(#arrow)"/>')
    cx = 640
    case_y = {'case_0': 0, 'case_1': 1, 'case_2': 2, 'case_4': 3}
    for name, k in case_y.items():
        s.rect(cx, ty + k * 44, 150, 32, GREEN, rx=4, extra=' fill-opacity="0.22"')
        s.text(cx + 12, ty + k * 44 + 21, name, 12, TEXT, mono=True)
    for i, t in enumerate(targets):
        k = case_y[t]
        s.add(f'<path d="M{tx + 172} {ty + i * 32 + 14}C{tx + 220} {ty + i * 32 + 14} {cx - 50} {ty + k * 44 + 16} {cx - 4} {ty + k * 44 + 16}" '
              f'fill="none" stroke="{DIM}" stroke-width="1.5" marker-end="url(#arrow)"/>')
    s.text(cx, ty + 4 * 44 + 18, 'two entries can share a case (1 and 3)', 11, DIM)
    s.text(40, 300, 'When Ghidra shows a computed jump with no cases, read the table\'s size from the bounds check and its entries as addresses.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- scene graph update (animated)
def scene_graph():
    s = Svg(900, 340, 'A scene graph update: world transforms are computed going down the tree, then bounds are merged going up')
    s.heading('Updating a scene graph', 'Down the tree: each node\'s world transform from its parent\'s. Back up: each node\'s bounds from its children.')
    nodes = {'world': (450, 100), 'town': (270, 180), 'camera': (630, 180), 'house': (170, 260), 'tree': (370, 260),
             'citizen': (560, 260), 'cart': (720, 260)}
    edges = [('world', 'town'), ('world', 'camera'), ('town', 'house'), ('town', 'tree'), ('camera', 'citizen'), ('camera', 'cart')]
    for a, b in edges:
        (x1, y1), (x2, y2) = nodes[a], nodes[b]
        s.line(x1, y1 + 16, x2, y2 - 16, DIM, 1.5)
    down = ['world', 'town', 'house', 'tree', 'camera', 'citizen', 'cart']
    up = ['house', 'tree', 'town', 'citizen', 'cart', 'camera', 'world']
    total = 2 * len(down)
    dur = total * 0.6
    for name, (x, y) in nodes.items():
        s.rect(x - 52, y - 16, 104, 32, '#16302f', rx=16)
        i, j = down.index(name), up.index(name)
        for idx, colour in ((i, TEAL), (len(down) + j, AMBER)):
            start, end = idx / total, (idx + 1) / total
            s.add(f'<rect x="{x - 52}" y="{y - 16}" width="104" height="32" rx="16" fill="{colour}" opacity="0">'
                  f'<animate attributeName="opacity" values="0;0;0.9;0.9;0" keyTimes="0;{start:.4f};{start + 0.001:.4f};{end:.4f};1" '
                  f'dur="{dur:.1f}s" repeatCount="indefinite"/></rect>')
        s.text(x, y + 5, name, 12, TEXT, 'middle', '600')
    s.rect(40, 300, 14, 14, TEAL, rx=3)
    s.text(62, 312, 'down: world transform = parent\'s world x local', 12, TEXT)
    s.rect(420, 300, 14, 14, AMBER, rx=3)
    s.text(442, 312, 'up: bounds = merge of the children\'s bounds', 12, TEXT)
    return s.render()


# ---------------------------------------------------------------- OPL2 operators (animated)
def opl2_operators():
    s = Svg(900, 330, 'An OPL2 voice: a modulator operator with feedback changes the phase of a carrier operator; each has an envelope')
    s.heading('An OPL2 voice: two operators', 'Nine voices, each a modulator and a carrier. Music is a stream of writes to their registers.')
    s.rect(40, 100, 170, 70, PURPLE, rx=8, extra=' fill-opacity="0.2"')
    s.text(125, 130, 'modulator', 14, TEXT, 'middle', '600')
    s.text(125, 150, 'operator 1', 11, DIM, 'middle')
    s.rect(300, 100, 170, 70, TEAL, rx=8, extra=' fill-opacity="0.2"')
    s.text(385, 130, 'carrier', 14, TEXT, 'middle', '600')
    s.text(385, 150, 'operator 2', 11, DIM, 'middle')
    s.add(f'<path d="M212 135H296" stroke="{DIM}" stroke-width="2" marker-end="url(#arrow)"/>')
    s.text(254, 126, 'phase', 11, DIM, 'middle')
    s.add(f'<path d="M125 98C125 70 60 70 60 98" fill="none" stroke="{PURPLE}" stroke-width="1.5" marker-end="url(#arrow)"/>')
    s.text(92, 66, 'feedback', 11, PURPLE, 'middle')
    s.add(f'<path d="M472 135H520" stroke="{DIM}" stroke-width="2" marker-end="url(#arrow)"/>')
    s.text(526, 140, 'out', 12, TEXT)
    # The carrier's output wave, modulated: drawn, then scrolled.
    pts = []
    for i in range(240):
        t = i / 240 * 6 * math.pi
        v = math.sin(t + 2.2 * math.sin(t * 2))
        pts.append(f'{560 + i * 1.3:.1f},{135 - 26 * v:.1f}')
    s.add(f'<polyline points="{" ".join(pts)}" fill="none" stroke="{TEAL}" stroke-width="1.8"/>')
    # The envelope: attack, decay, sustain, release, with a moving dot.
    ex, ey, w = 40, 210, 380
    env = [(0, 0), (40, 70), (100, 48), (260, 48), (380, 0)]
    path = 'M' + ' L'.join(f'{ex + x} {ey + 80 - y}' for x, y in env)
    s.add(f'<path d="{path}" fill="none" stroke="{AMBER}" stroke-width="2"/>')
    for (x, _), label in zip(((20, 0), (70, 0), (180, 0), (320, 0)), ('attack', 'decay', 'sustain', 'release')):
        s.text(ex + x, ey + 98, label, 11, DIM, 'middle')
    s.add(f'<circle r="5" fill="{AMBER}"><animateMotion path="{path}" dur="4s" repeatCount="indefinite"/></circle>')
    s.text(ex, ey - 6, 'each operator\'s envelope, from key on to key off', 11, DIM)
    regs = [('0x20', 'multiplier, vibrato, tremolo'), ('0x40', 'level (volume)'), ('0x60', 'attack and decay rates'),
            ('0x80', 'sustain level, release rate'), ('0xA0, 0xB0', 'frequency, octave, key on'),
            ('0xC0', 'feedback, FM or additive'), ('0xE0', 'waveform (4 on the OPL2)')]
    s.text(560, 206, 'registers (ports 388h and 389h)', 11, DIM)
    for i, (r, what) in enumerate(regs):
        s.text(560, 226 + i * 15, r, 11, TEXT, mono=True)
        s.text(640, 226 + i * 15, what, 11, TEXT)
    return s.render()


# ---------------------------------------------------------------- consoles by generation
def console_timeline():
    s = Svg(900, 380, 'Home consoles by year of first release, from the NES in 1983 to the PlayStation 5 in 2020, coloured by CPU family')
    s.heading('Home consoles by generation', 'Year of first release. The colour is the CPU family, which decides the tools.')
    cpu = {'6502': TEAL, 'Z80': GREY, '68000': BLUE, 'SH': PURPLE, 'MIPS': AMBER, 'PowerPC': GREEN, 'x86': RED, 'ARM': '#f472b6'}
    items = [(1983, 'NES', '6502'), (1985, 'Master System', 'Z80'), (1987, 'PC Engine', '6502'), (1988, 'Mega Drive', '68000'),
             (1990, 'SNES', '6502'), (1994, 'Saturn', 'SH'), (1994, 'PlayStation', 'MIPS'), (1996, 'Nintendo 64', 'MIPS'),
             (1998, 'Dreamcast', 'SH'), (2000, 'PlayStation 2', 'MIPS'), (2001, 'GameCube', 'PowerPC'), (2001, 'Xbox', 'x86'),
             (2005, 'Xbox 360', 'PowerPC'), (2006, 'PlayStation 3', 'PowerPC'), (2006, 'Wii', 'PowerPC'), (2012, 'Wii U', 'PowerPC'),
             (2013, 'PlayStation 4', 'x86'), (2013, 'Xbox One', 'x86'), (2017, 'Switch', 'ARM'), (2020, 'PlayStation 5', 'x86'),
             (2020, 'Xbox Series', 'x86')]
    lo, hi, x0, x1, ya = 1982, 2021, 50, 860, 200
    X = lambda yr: x0 + (yr - lo) / (hi - lo) * (x1 - x0)
    gens = [(1983, 1987, '3rd'), (1987, 1993, '4th'), (1993, 1998, '5th'), (1998, 2005, '6th'), (2005, 2012, '7th'),
            (2012, 2020, '8th'), (2020, 2021, '9th')]
    def halo(x, y, t, size, colour, weight='normal'):
        s.add(f'<text x="{x:.1f}" y="{y}" font-family="ui-sans-serif, Helvetica, Arial, sans-serif" font-size="{size}" '
              f'font-weight="{weight}" fill="{colour}" text-anchor="middle" stroke="{BG}" stroke-width="4" '
              f'paint-order="stroke">{t}</text>')
    for i, (a, b, g) in enumerate(gens):
        s.rect(X(a), ya - 6, X(b) - X(a) - 2, 12, '#1f3d40' if i % 2 else '#16302f', rx=3)
    placed = []  # (side, level, left, right, x)
    labels = []
    for yr, name, c in items:
        x = X(yr)
        half = len(name) * 3.4 + 6
        for level in range(1, 6):
            for side in ('up', 'down') if level <= 2 else ('up',):
                clash = any(sd == side and ((lv == level and l < x + half and x - half < r) or
                                            (lv < level and l <= x <= r) or (lv > level and x - half <= px <= x + half))
                            for sd, lv, l, r, px in placed)
                if not clash:
                    break
            else:
                continue
            break
        placed.append((side, level, x - half, x + half, x))
        y = ya - 18 - level * 20 if side == 'up' else ya + 58 + level * 18
        s.line(x, ya + (-6 if side == 'up' else 6), x, y + (4 if side == 'up' else -12), cpu[c], 1)
        s.add(f'<circle cx="{x:.1f}" cy="{ya}" r="4" fill="{cpu[c]}"/>')
        labels.append((x, y, name, cpu[c]))
    for (a, b, g) in gens:
        halo((X(a) + X(b)) / 2, ya + 24, g, 11, DIM)
    for yr in range(1985, 2021, 5):
        halo(X(yr), ya + 42, str(yr), 10, DIM)
    for x, y, name, colour in labels:
        halo(x, y, name, 11, colour, '600')
    lx = 50
    for name, colour in cpu.items():
        s.rect(lx, 352, 10, 10, colour, rx=2)
        s.text(lx + 14, 361, name, 11, TEXT)
        lx += 90
    return s.render()


MORE = {
    'game-families.svg': game_families, 'team-handoff.svg': team_handoff, 'abi-stack.svg': abi_stack,
    'fixed-step.svg': fixed_step, 'unpacking.svg': unpacking, 'jump-table.svg': jump_table,
    'scene-graph.svg': scene_graph, 'opl2-operators.svg': opl2_operators, 'console-timeline.svg': console_timeline,
}
