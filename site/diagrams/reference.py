"""Diagrams for the reference pages (GAME-PATTERNS.md, ABI.md, COMPILED-CODE.md, DOS.md, TYPES.md):
MSVC rand(), entities and components, an archive, float time drift, physics integrators, a
password's bits, messages from a string table, saves with ids, lockstep and client-server, a
vtable, the C runtime's start-up, signatures with wildcards, division by a constant, the flags,
segmented addresses, overlays and a structure read from its uses. The numbers in the plots are
computed here, from the same formulas the pages describe. site/diagrams/make.py writes them.
"""
import math
import struct

from svg import AMBER, BG, BLUE, DIM, GREEN, GREY, PURPLE, RED, TEAL, TEXT, Svg

SANS = 'ui-sans-serif, Helvetica, Arial, sans-serif'
MONOF = 'ui-monospace, Menlo, monospace'


def f32(x):
    return struct.unpack('<f', struct.pack('<f', x))[0]


def box(s, x, y, w, h, colour, title, sub=None, op='0.22', size=13):
    s.rect(x, y, w, h, colour, rx=6, extra=f' fill-opacity="{op}"')
    if sub:
        s.text(x + w / 2, y + h / 2 - 2, title, size, TEXT, 'middle', '600')
        s.text(x + w / 2, y + h / 2 + 15, sub, 11, DIM, 'middle')
    else:
        s.text(x + w / 2, y + h / 2 + 5, title, size, TEXT, 'middle', '600')


def arrow(s, x1, y1, x2, y2, colour=DIM):
    s.line(x1, y1, x2, y2, colour, 1.5, ' marker-end="url(#arrow)"')


def plot(s, x, y, w, h, series, xr, yr, xlabel, ylabel, ticks_x, ticks_y):
    """Lines in a box: series is [(points, colour, label)], points in data units."""
    s.rect(x, y, w, h, '#0f2427', rx=4)
    X = lambda v: x + (v - xr[0]) / (xr[1] - xr[0]) * w
    Y = lambda v: y + h - (v - yr[0]) / (yr[1] - yr[0]) * h
    for t in ticks_y:
        s.line(x, Y(t), x + w, Y(t), '#1f3d40', 1)
        s.text(x - 6, Y(t) + 4, f'{t:g}', 10, DIM, 'end')
    for t in ticks_x:
        s.text(X(t), y + h + 14, f'{t:g}', 10, DIM, 'middle')
    s.text(x + w / 2, y + h + 30, xlabel, 11, DIM, 'middle')
    s.text(x, y - 8, ylabel, 11, DIM)
    for pts, colour, _ in series:
        d = 'M' + 'L'.join(f'{X(a):.1f} {Y(min(max(b, yr[0]), yr[1])):.1f}' for a, b in pts)
        s.add(f'<path d="{d}" fill="none" stroke="{colour}" stroke-width="2"/>')
    return X, Y


# ---------------------------------------------------------------- MSVC rand (animated)
def msvc_rand():
    s = Svg(900, 330, 'MSVC rand(): the state is multiplied by 214013 and 2531011 is added; the draw is bits 16 to 30 '
                      'of the new state. From srand(1), the first draws are 41, 18467, 6334, 26500 and 19169')
    s.heading('The MSVC rand() generator', 'state = state x 214013 + 2531011, then the draw is bits 16 to 30. Seeded with 1.')
    state, rows = 1, []
    for _ in range(5):
        state = (state * 214013 + 2531011) & 0xFFFFFFFF
        rows.append((state, (state >> 16) & 0x7FFF))
    n, dur = len(rows), 7.5
    x0, y0, bw = 40, 120, 22
    s.text(x0, y0 - 14, 'the 32-bit state after each step (bit 31 on the left)', 11, DIM)
    for i, (st, out) in enumerate(rows):
        g = []
        for b in range(32):
            bit = 31 - b
            on = (st >> bit) & 1
            colour = AMBER if 16 <= bit <= 30 else GREY
            g.append(f'<rect x="{x0 + b * bw}" y="{y0}" width="{bw - 2}" height="30" rx="2" fill="{colour}" '
                     f'fill-opacity="{0.85 if on else 0.18}"/>')
            g.append(f'<text x="{x0 + b * bw + bw / 2 - 1}" y="{y0 + 20}" font-family="{MONOF}" font-size="12" '
                     f'font-weight="600" fill="{BG if on else DIM}" text-anchor="middle">{on}</text>')
        g.append(f'<text x="{x0}" y="{y0 + 84}" font-family="{MONOF}" font-size="14" fill="{TEXT}">draw {i + 1}: '
                 f'state = 0x{st:08x}   (state &gt;&gt; 16) &amp; 0x7fff = {out}</text>')
        t0, t1 = i / n, (i + 1) / n
        kt = f'0;{t0:.3f};{t0:.3f};{t1:.3f};{t1:.3f};1' if i else f'0;{t1:.3f};{t1:.3f};1'
        vals = '0;0;1;1;0;0' if i else '1;1;0;0'
        if i == n - 1:
            kt, vals = f'0;{t0:.3f};{t0:.3f};1', '0;0;1;1'
        s.add(f'<g opacity="{vals[0]}">{"".join(g)}<animate attributeName="opacity" values="{vals}" keyTimes="{kt}" '
              f'dur="{dur}s" repeatCount="indefinite"/></g>')
    s.text(x0 + bw, y0 + 46, 'bits 30 to 16: the draw', 11, AMBER)
    s.text(x0, 250, 'Draws from srand(1): ' + ', '.join(str(o) for _, o in rows), 13, TEXT, weight='600')
    s.text(x0, 276, 'A port must copy the formula, the seed, and the order of every draw. One extra draw shifts all the '
                    'ones after it.', 12, DIM)
    s.text(x0, 300, 'tools/triage/constants.py finds 214013 and the other generators\' constants in a binary.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- entities and components
def entity_components():
    s = Svg(900, 330, 'An entity-component system: entities are ids, each component type keeps its own array, and each '
                      'system walks one array in a fixed order')
    s.heading('Entities and components', 'An entity is an id. Each kind of component has its own array. Systems walk the arrays in order.')
    ents = ['#1 farmer', '#2 house', '#3 tree', '#4 cart']
    for k, e in enumerate(ents):
        box(s, 40, 90 + k * 50, 130, 38, TEAL, e, size=12)
    comps = [('Position', [1, 2, 3, 4], BLUE), ('Health', [1, 3], RED), ('Mover', [1, 4], AMBER),
             ('Storage', [2, 4], PURPLE)]
    for c, (name, owners, colour) in enumerate(comps):
        x = 250 + c * 150
        s.text(x, 84, name, 12, TEXT, weight='600')
        for k, o in enumerate(owners):
            s.rect(x, 92 + k * 34, 130, 28, colour, rx=3, extra=' fill-opacity="0.3"')
            s.text(x + 10, 92 + k * 34 + 19, f'entity #{o}', 11, TEXT, mono=True)
    s.text(250, 260, 'type id: often a hash of the name ("Position" -> 0x8f3a...)', 11, DIM)
    s.text(40, 296, 'Each frame: Mover system, then Health system, then Storage system, each from the first element to the '
                    'last. Keep that order.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- an archive
def package_archive():
    s = Svg(900, 330, 'A game archive: a header, a table of entries with a name or name hash, offset, size and a '
                      'compression flag, then the data')
    s.heading('An archive file', 'A header, a table of entries, then the data. Names may be stored as hashes.')
    x = 40
    for name, w, colour in (('header', 110, GREY), ('entry table', 230, AMBER), ('data', 470, BLUE)):
        s.rect(x, 90, w - 4, 40, colour, rx=4, extra=' fill-opacity="0.3"')
        s.text(x + w / 2 - 2, 115, name, 12, TEXT, 'middle', '600')
        x += w
    s.text(40, 152, 'magic, version, entry count, table offset', 11, DIM)
    cols = ['name hash', 'offset', 'size', 'packed size', 'flags']
    rows = [('0x8c1f22a0', '0x0400', '18 432', '6 210', 'zlib'), ('0x1b7e9c41', '0x1c42', '2 048', '2 048', 'stored'),
            ('0x5d03aa17', '0x2442', '96 000', '31 877', 'zlib')]
    ty = 180
    for c, h in enumerate(cols):
        s.text(150 + c * 110, ty, h, 11, AMBER, weight='600')
    for r, row in enumerate(rows):
        for c, v in enumerate(row):
            s.text(150 + c * 110, ty + 24 + r * 22, v, 12, TEXT, mono=True)
    for r, (x2, w2) in enumerate(((380, 120), (504, 60), (568, 300))):
        s.rect(x2, 90, w2 - 4, 40, BLUE, rx=3, extra=' fill-opacity="0.35"')
        s.add(f'<path d="M700 {ty + 20 + r * 22}C 760 {ty + 20 + r * 22} {x2 + w2 / 2} 170 {x2 + w2 / 2} 134" fill="none" '
              f'stroke="{DIM}" stroke-width="1.2" marker-end="url(#arrow)"/>')
    s.text(40, 300, 'An example layout. The lookup code hashes a file name and searches the table. hashnames.py finds the '
                    'hash function.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- float time drift (plot)
def float_time():
    s = Svg(900, 360, 'The clock error from adding 1/60 of a second to a 32-bit float once per frame for an hour, '
                      'computed: the error reaches about 2.8 seconds')
    s.heading('Float time drifts', 'A clock kept as a 32-bit float, adding 1/60 s each frame, compared with the true time. The values are computed.')
    t, step, pts = 0.0, f32(1 / 60), [(0, 0)]
    for i in range(1, 216001):
        t = f32(t + step)
        if i % 1800 == 0:
            pts.append((i / 3600, t - i / 60))
    plot(s, 90, 90, 560, 210, [(pts, AMBER, 'float')], (0, 60), (-4, 4), 'minutes of play at 60 frames a second',
         'error (seconds)', range(0, 61, 10), (-4, -2, 0, 2, 4))
    s.text(680, 110, 'The step 1/60 is not exact', 12, TEXT)
    s.text(680, 128, 'as a float, and each add', 12, TEXT)
    s.text(680, 146, 'rounds to the clock\'s', 12, TEXT)
    s.text(680, 164, 'precision, which falls as', 12, TEXT)
    s.text(680, 182, 'the value grows.', 12, TEXT)
    s.text(680, 214, 'Integer ticks do not drift.', 12, GREEN)
    s.text(680, 240, 'If the original keeps a', 12, DIM)
    s.text(680, 258, 'float clock, the port must', 12, DIM)
    s.text(680, 276, 'keep the same one.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- integrators (plot)
def integrators():
    s = Svg(900, 360, 'A spring simulated with explicit Euler, semi-implicit Euler and the exact solution, computed '
                      'with a step of 0.1: explicit Euler gains energy and the swing grows')
    s.heading('Euler integration of a spring', 'x\'\' = -x from x = 1 with a step of 0.1. The two methods differ only in the order of two lines. The values are computed.')
    dt, steps = 0.1, 200
    ex, sm, exact = [(0, 1)], [(0, 1)], []
    x, v = 1.0, 0.0
    for i in range(1, steps + 1):
        x, v = x + v * dt, v - x * dt
        ex.append((i * dt, x))
    x, v = 1.0, 0.0
    for i in range(1, steps + 1):
        v = v - x * dt
        x = x + v * dt
        sm.append((i * dt, x))
    exact = [(i * dt, math.cos(i * dt)) for i in range(steps + 1)]
    plot(s, 70, 90, 600, 210, [(exact, GREY, ''), (sm, TEAL, ''), (ex, RED, '')], (0, 20), (-3, 3), 'time',
         'position x', range(0, 21, 5), (-3, -2, -1, 0, 1, 2, 3))
    for k, (colour, name, code) in enumerate(((RED, 'explicit Euler', 'x += v*dt; v -= x*dt'),
                                              (TEAL, 'semi-implicit Euler', 'v -= x*dt; x += v*dt'),
                                              (GREY, 'exact', 'cos t'))):
        y = 110 + k * 56
        s.rect(700, y - 10, 14, 4, colour)
        s.text(722, y - 4, name, 12, TEXT, weight='600')
        s.text(700, y + 14, code, 11, DIM, mono=True)
    s.text(700, 290, 'The port keeps the', 11, DIM)
    s.text(700, 306, 'original\'s order.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- a password's bits
def password_bits():
    s = Svg(900, 330, 'A level password decoded: each character is an index into the game\'s alphabet, each index '
                      'gives five bits, and the bits are packed fields with a checksum')
    s.heading('The bits of a password', 'Each letter is an index into the game\'s alphabet. The bits hold the state and a checksum.')
    alphabet = 'BCDFGHJKLMNPQRSTVWXYZ0123456789!'
    s.text(40, 92, 'alphabet:', 11, DIM)
    s.text(110, 92, alphabet, 13, TEXT, mono=True)
    word = 'KQ7B'
    idx = [alphabet.index(c) for c in word]
    for k, (c, i) in enumerate(zip(word, idx)):
        x = 40 + k * 205
        s.rect(x, 110, 60, 44, TEAL, rx=4, extra=' fill-opacity="0.3"')
        s.text(x + 30, 140, c, 20, TEXT, 'middle', '700', mono=True)
        s.text(x + 72, 128, f'index {i}', 11, DIM)
        s.text(x + 72, 146, f'{i:05b}', 14, AMBER, weight='600', mono=True)
    allbits = ''.join(f'{i:05b}' for i in idx)
    fields = [('level', 5, BLUE), ('lives', 3, GREEN), ('items', 6, PURPLE), ('checksum', 6, RED)]
    x, pos = 40, 0
    for name, n, colour in fields:
        w = n * 40
        s.rect(x, 200, w - 4, 40, colour, rx=3, extra=' fill-opacity="0.3"')
        s.text(x + (w - 4) / 2, 216, name, 11, TEXT, 'middle', '600')
        s.text(x + (w - 4) / 2, 233, allbits[pos:pos + n], 13, TEXT, 'middle', '600', mono=True)
        x += w
        pos += n
    s.text(40, 270, 'An example layout. The game\'s checker shows the real layout in its alphabet string, its shifts and its '
                    'checksum.', 11, DIM)
    s.text(40, 290, 'Port the encoder and the decoder, and test them on passwords the original accepts.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- messages from a string table
def message_format():
    s = Svg(900, 300, 'A message stored as a type and arguments, formatted through the string table of the player\'s '
                      'language when it is shown')
    s.heading('Messages from a string table', 'The text is made when the message is shown, from the language\'s string table.')
    box(s, 40, 100, 220, 70, AMBER, 'message 0x12', 'person 41, building 7', op='0.25')
    box(s, 330, 100, 240, 70, PURPLE, 'string 0x12 (English)', '"%1 has moved into %2."', op='0.25')
    box(s, 630, 100, 240, 70, TEAL, '"Ana has moved into', 'the Stone House."', op='0.25', size=12)
    arrow(s, 262, 135, 326, 135)
    arrow(s, 572, 135, 636, 135)
    s.text(40, 210, 'A save may keep the arguments (formatted again on load) or only the text (shown as it is).', 12, DIM)
    s.text(40, 232, 'Never hard-code English where the original reads a table.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- saves with ids (animated)
def save_fixup():
    s = Svg(900, 330, 'Saving and loading objects that point at each other: pointers are written as ids, loading '
                      'creates every object, then a second pass turns ids back into pointers')
    s.heading('Pointers in a save', 'Writing turns each pointer into the target\'s id. Loading reads all objects, then fixes the ids.')
    objs = [('#1 farmer', 'home: #2'), ('#2 house', 'owner: #1'), ('#3 field', 'worker: #1')]
    for k, (name, ref) in enumerate(objs):
        x = 60 + k * 270
        s.rect(x, 110, 200, 60, BLUE, rx=6, extra=' fill-opacity="0.22"')
        s.text(x + 100, 134, name, 13, TEXT, 'middle', '600')
        s.text(x + 100, 156, ref, 12, AMBER, 'middle', mono=True)
    s.text(60, 96, 'pass 1: create every object, with the ids as read', 12, TEXT, weight='600')
    links = [(0, 1), (1, 0), (2, 0)]
    paths = []
    for a, b in links:
        ax, bx = 60 + a * 270 + 100, 60 + b * 270 + 100
        bend = 230 if a < b else 260 if a - b == 1 else 290
        paths.append(f'<path d="M{ax} 172C {ax} {bend} {bx} {bend} {bx} 174" fill="none" stroke="{GREEN}" '
                     f'stroke-width="2" marker-end="url(#arrow)"/>')
    s.add(f'<g opacity="0">{"".join(paths)}<text x="60" y="310" font-family="{SANS}" font-size="12" font-weight="600" '
          f'fill="{GREEN}">pass 2: each id replaced by a pointer to that object</text>'
          f'<animate attributeName="opacity" values="0;0;1;1" keyTimes="0;0.45;0.5;1" dur="5s" repeatCount="indefinite"/></g>')
    return s.render()


# ---------------------------------------------------------------- lockstep and client-server
def net_models():
    s = Svg(900, 330, 'Two network models: lockstep sends only inputs and needs exact determinism; client-server sends '
                      'the state from the server')
    s.heading('Lockstep and client-server', 'Lockstep games exchange the players\' inputs. In client-server games, the server sends the state to each client.')
    s.text(40, 92, 'Lockstep', 14, TEAL, weight='600')
    for k, name in enumerate(('player A', 'player B')):
        box(s, 40 + k * 260, 110, 160, 60, TEAL, name, 'runs the whole game')
    s.add(f'<path d="M202 130H296" stroke="{AMBER}" stroke-width="2" marker-end="url(#arrow)"/>'
          f'<path d="M298 150H204" stroke="{AMBER}" stroke-width="2" marker-end="url(#arrow)"/>')
    s.text(250, 124, 'inputs', 11, AMBER, 'middle')
    s.text(40, 200, 'Each machine computes the same game', 12, DIM)
    s.text(40, 218, 'from the same inputs. A difference of one bit', 12, DIM)
    s.text(40, 236, 'makes the games diverge.', 12, DIM)
    s.text(500, 92, 'Client-server', 14, BLUE, weight='600')
    box(s, 500, 110, 160, 60, BLUE, 'server', 'runs the game')
    box(s, 740, 110, 130, 60, GREY, 'client', 'draws it')
    s.add(f'<path d="M662 130H736" stroke="{AMBER}" stroke-width="2" marker-end="url(#arrow)"/>'
          f'<path d="M738 150H664" stroke="{AMBER}" stroke-width="2" marker-end="url(#arrow)"/>')
    s.text(700, 124, 'state', 11, AMBER, 'middle')
    s.text(700, 166, 'inputs', 11, AMBER, 'middle')
    s.text(500, 200, 'The server must be exact. Clients show', 12, DIM)
    s.text(500, 218, 'a prediction until it arrives. A port must use', 12, DIM)
    s.text(500, 236, 'the same protocol to play with original clients.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- a vtable
def vtable_layout():
    s = Svg(900, 330, 'An MSVC object with virtual methods: its first field points to the vtable, and the slot before '
                      'the vtable points to RTTI that names the class')
    s.heading('An object, its vtable and its RTTI (MSVC)', 'The object\'s first field points to the vtable. The slot before it leads to the class name.')
    s.text(40, 92, 'the object', 11, DIM)
    for k, (f, c) in enumerate((('vftable *', AMBER), ('field +0x08', BLUE), ('field +0x10', BLUE))):
        s.rect(40, 100 + k * 34, 170, 30, c, rx=3, extra=' fill-opacity="0.3"')
        s.text(50, 120 + k * 34, f, 12, TEXT, mono=True)
    s.text(330, 92, 'the vtable', 11, DIM)
    slots = [('-0x08', 'complete object locator', PURPLE), ('+0x00', 'scalar deleting destructor', GREEN),
             ('+0x08', 'Update', GREEN), ('+0x10', 'Draw', GREEN), ('+0x18', 'Save', GREEN)]
    for k, (off, f, c) in enumerate(slots):
        y = 100 + k * 34
        s.rect(330, y, 230, 30, c, rx=3, extra=' fill-opacity="0.3"')
        s.text(340, y + 20, f, 12, TEXT)
        s.text(552, y + 20, off, 11, DIM, 'end', mono=True)
    arrow(s, 212, 115, 326, 149, AMBER)
    s.text(640, 92, 'RTTI', 11, DIM)
    box(s, 640, 100, 220, 50, PURPLE, 'type descriptor', '.?AVCitizen@@')
    box(s, 640, 170, 220, 50, PURPLE, 'class hierarchy', 'base classes')
    arrow(s, 562, 115, 636, 125, PURPLE)
    arrow(s, 750, 152, 750, 166, PURPLE)
    s.text(40, 300, 'A call through slot +0x18 reads (**(code **)(*obj + 0x18))(obj). Base classes share their early slots.',
           12, DIM)
    return s.render()


# ---------------------------------------------------------------- the C runtime's start-up
def crt_startup():
    s = Svg(900, 260, 'An executable starts in the C runtime: the entry point initialises the runtime and the static '
                      'constructors, then calls WinMain or main, where the game begins')
    s.heading('From the entry point to the game', 'The C runtime runs first. The game\'s own code starts where main or WinMain is called.')
    steps = [('entry point', 'from the PE header', GREY), ('security cookie', 'VC++ 2005 and later', GREY),
             ('runtime set-up', 'heap, I/O, locale', GREY), ('static constructors', '_initterm table', PURPLE),
             ('WinMain / main', 'the game', TEAL)]
    for k, (a, b, c) in enumerate(steps):
        x = 30 + k * 172
        box(s, x, 100, 150, 64, c, a, b, op='0.3')
        if k:
            arrow(s, x - 20, 132, x - 2, 132)
    s.text(30, 210, 'GetCommandLine and GetStartupInfo are called just before WinMain. Runtime error strings ("R6002") '
                    'sit in the runtime.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- signatures with wildcards
def signature_match():
    s = Svg(900, 320, 'A library function turned into a byte signature, with the bytes the linker changes as wildcards, '
                      'and the signature found in a game')
    s.heading('A signature with wildcards', 'The bytes the linker fills in (addresses) become ??. Every other byte must match.')
    lib = ['55', '8B', 'EC', 'FF', '76', '04', '9A', '00', '00', '00', '00', '59', '5D', 'C3']
    wild = {7, 8, 9, 10}
    game = ['55', '8B', 'EC', 'FF', '76', '04', '9A', '3C', '01', 'A2', '1B', '59', '5D', 'C3']
    for row, (label, data) in enumerate((('library (.LIB)', lib), ('signature', None), ('game', game))):
        y = 96 + row * 56
        s.text(40, y + 20, label, 12, TEXT, weight='600')
        for k in range(14):
            x = 170 + k * 50
            w = k in wild
            if data is None:
                v = '??' if w else lib[k]
            else:
                v = data[k]
            colour = AMBER if w else (GREEN if row == 2 else BLUE)
            s.rect(x, y, 46, 30, colour, rx=3, extra=f' fill-opacity="{0.35 if w else 0.22}"')
            s.text(x + 23, y + 20, v, 12, TEXT, 'middle', '600', mono=True)
    s.text(170 + 7 * 50, 266, 'the fixup for a far call\'s address', 11, AMBER)
    s.text(40, 300, 'omf.py makes signatures from a library. sigscan.py finds them in the game and writes labels.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- division by a constant
def magic_division():
    s = Svg(900, 320, 'Unsigned division by 10 compiled as a multiplication by 0xCCCCCCCD and a shift by 35; '
                      'the results equal x / 10 for every 32-bit x')
    s.heading('Division by a constant', 'The compiler turns x / 10 into a multiply and a shift, which give the same result.')
    s.text(40, 100, 'mov eax, 0xCCCCCCCD', 14, TEXT, mono=True)
    s.text(40, 122, 'mul ecx            ; edx:eax = x * 0xCCCCCCCD', 14, TEXT, mono=True)
    s.text(40, 144, 'shr edx, 3         ; edx = x / 10', 14, TEXT, mono=True)
    s.text(40, 176, 'The divisor is 2^35 / 0xCCCCCCCD = 9.99999999942, so the constant is 10.', 12, DIM)
    xs = [7, 10, 99, 12345, 4000000000, 4294967295]
    s.text(40, 210, 'x', 12, AMBER, weight='600')
    s.text(240, 210, '(x * 0xCCCCCCCD) >> 35', 12, AMBER, weight='600')
    s.text(500, 210, 'x / 10', 12, AMBER, weight='600')
    for k, x in enumerate(xs):
        y = 232 + k * 14
        s.text(40, y, str(x), 11, TEXT, mono=True)
        s.text(240, y, str((x * 0xCCCCCCCD) >> 35), 11, TEXT, mono=True)
        s.text(500, y, str(x // 10), 11, GREEN, mono=True)
    return s.render()


# ---------------------------------------------------------------- the flags
def cpu_flags():
    s = Svg(900, 300, 'cmp eax, 1 with eax = -1 sets the flags once; jl reads them as signed and jumps, jb reads them '
                      'as unsigned and does not')
    s.heading('Signed and unsigned jumps', 'jl and jb read the same flags after one cmp. jl compares signed numbers. jb compares unsigned numbers.')
    s.text(40, 104, 'eax = 0xFFFFFFFF', 14, TEXT, mono=True)
    s.text(40, 128, 'cmp eax, 1', 14, TEXT, mono=True)
    for k, (f, v, c) in enumerate((('ZF', 0, GREY), ('SF', 1, AMBER), ('CF', 0, BLUE), ('OF', 0, GREY))):
        x = 300 + k * 70
        s.rect(x, 96, 60, 40, c, rx=4, extra=f' fill-opacity="{0.4 if v else 0.2}"')
        s.text(x + 30, 114, f, 11, DIM, 'middle')
        s.text(x + 30, 130, str(v), 14, TEXT, 'middle', '600', mono=True)
    box(s, 40, 180, 380, 60, AMBER, 'jl: signed, SF != OF', '-1 < 1: the jump is taken', op='0.25')
    box(s, 470, 180, 380, 60, BLUE, 'jb: unsigned, CF = 1', '4294967295 < 1 is false: not taken', op='0.25')
    s.text(40, 276, 'Use the signedness of the jump, whatever type Ghidra declares.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- segmented addresses
def segment_address():
    s = Svg(900, 300, 'A real-mode address: the segment times 16 plus the offset, so 1234:0010 and 1235:0000 are the '
                      'same byte at 12350h')
    s.heading('Segment and offset addresses', 'linear = segment x 16 + offset. Different pairs can name the same byte.')
    rows = [('1234:0010', 0x1234, 0x10), ('1235:0000', 0x1235, 0), ('1000:2350', 0x1000, 0x2350)]
    for k, (txt, seg, off) in enumerate(rows):
        y = 100 + k * 44
        s.text(40, y + 20, txt, 15, TEXT, mono=True)
        s.text(180, y + 20, f'= {seg:04X}0 + {off:04X} = {seg * 16 + off:05X}', 14, AMBER, mono=True)
    s.rect(520, 92, 340, 130, GREY, rx=4, extra=' fill-opacity="0.15"')
    for k, (seg, colour) in enumerate(((0x1000, BLUE), (0x1234, TEAL), (0x1235, PURPLE))):
        x = 520 + (seg - 0x1000) * 16 / 0x10000 * 340 * 4
        s.rect(x, 100 + k * 36, 340 - (x - 520), 28, colour, rx=3, extra=' fill-opacity="0.3"')
        s.text(x + 6, 118 + k * 36, f'segment {seg:04X} (64 KB)', 11, TEXT)
    s.line(520 + 0x2350 / 0x10000 * 340 * 4, 92, 520 + 0x2350 / 0x10000 * 340 * 4, 222, AMBER, 2)
    s.text(520, 244, 'segments overlap every 16 bytes', 11, DIM)
    s.text(40, 270, 'Ghidra numbers 16-bit code by its own segment selectors. Always quote Ghidra\'s address.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- overlays (animated)
def overlays():
    s = Svg(900, 300, 'Overlays: code segments that share one area of memory; the overlay manager loads each from the '
                      'file when it is called')
    s.heading('Overlays', 'Parts of the program take turns in one area of memory. The overlay manager loads each when called.')
    box(s, 40, 100, 180, 120, GREY, 'resident code', 'always in memory', op='0.25')
    s.rect(260, 100, 220, 120, BLUE, rx=6, extra=' fill-opacity="0.12" stroke-dasharray="4 4"', stroke='#60a5fa')
    s.text(370, 92, 'overlay area', 11, DIM, 'middle')
    ovl = [('overlay 1: menus', TEAL), ('overlay 2: battle', AMBER), ('overlay 3: map editor', PURPLE)]
    for k, (name, colour) in enumerate(ovl):
        x = 560
        y = 92 + k * 46
        s.rect(x, y, 300, 38, colour, rx=4, extra=' fill-opacity="0.25"')
        s.text(x + 10, y + 24, name + '  (in the .EXE or .OVL file)', 12, TEXT)
        t0, t1 = k / 3, (k + 1) / 3
        kt = f'0;{t0:.3f};{t0:.3f};{t1:.3f};{t1:.3f};1' if k else f'0;{t1:.3f};{t1:.3f};1'
        vals = '0;0;1;1;0;0' if k else '1;1;0;0'
        if k == 2:
            kt, vals = f'0;{t0:.3f};{t0:.3f};1', '0;0;1;1'
        s.add(f'<g opacity="{vals[0]}"><rect x="266" y="106" width="208" height="108" rx="5" fill="{colour}" '
              f'fill-opacity="0.45"/><text x="370" y="165" font-family="{SANS}" font-size="13" font-weight="600" '
              f'fill="{TEXT}" text-anchor="middle">{name}</text><path d="M558 {y + 19}C 520 {y + 19} 520 160 478 160" '
              f'fill="none" stroke="{colour}" stroke-width="2" marker-end="url(#arrow)"/>'
              f'<animate attributeName="opacity" values="{vals}" keyTimes="{kt}" dur="6s" repeatCount="indefinite"/></g>')
    s.text(40, 266, 'Ghidra needs each overlay loaded at the address it runs from. DOS.md shows how.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- a structure from its uses
def struct_from_uses():
    s = Svg(900, 320, 'Several functions each touch some offsets of one structure; merged, their accesses give the '
                      'structure\'s fields, sizes and types')
    s.heading('A structure read from its uses', 'Each function uses a few fields. Merging the accesses of several functions gives the layout.')
    fns = [('constructor', [(0, 8, 'vtable'), (8, 4, 'int'), (16, 4, 'float'), (24, 8, 'ptr')], TEAL),
           ('Save', [(8, 4, 'int'), (12, 4, 'int'), (16, 4, 'float')], AMBER),
           ('Update', [(16, 4, 'float'), (20, 1, 'byte'), (24, 8, 'ptr')], PURPLE)]
    unit = 18
    for r, (name, acc, colour) in enumerate(fns):
        y = 96 + r * 40
        s.text(40, y + 19, name, 12, TEXT, weight='600')
        for off, size, t in acc:
            s.rect(200 + off * unit, y, size * unit - 3, 28, colour, rx=3, extra=' fill-opacity="0.4"')
            s.text(200 + off * unit + (size * unit - 3) / 2, y + 18, t, 10, TEXT, 'middle')
    y = 236
    s.text(40, y + 19, 'merged', 12, GREEN, weight='600')
    for off, size, t in ((0, 8, 'vtable'), (8, 4, 'int'), (12, 4, 'int'), (16, 4, 'float'), (20, 1, 'byte'),
                         (21, 3, 'gap'), (24, 8, 'ptr')):
        s.rect(200 + off * unit, y, size * unit - 3, 28, GREY if t == 'gap' else GREEN, rx=3,
               extra=f' fill-opacity="{0.15 if t == "gap" else 0.35}"')
        s.text(200 + off * unit + (size * unit - 3) / 2, y + 18, t, 10, TEXT, 'middle')
    for off in range(0, 33, 4):
        s.text(200 + off * unit, 88, f'+{off:x}', 9, DIM, mono=True)
    s.text(40, 300, 'decomp_query.py layout CONSTRUCTOR SAVE UPDATE prints this table and a draft struct.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- what each compiler stage removes
def compile_stages():
    s = Svg(900, 300, 'The stages of a compiler from source to machine code, and what each stage removes from the program')
    s.heading('What each stage removes', 'A decompiler gets back the structure. The removed parts come from other evidence.')
    stages = [('scanner', 'text to tokens', 'comments, macros', GREY), ('parser', 'tokens to a tree', 'parentheses', BLUE),
              ('semantic analysis', 'names and types', 'names, types', PURPLE), ('optimiser', 'a simpler tree', 'constant sums',
              AMBER), ('code generator', 'machine code', 'open orders', TEAL)]
    s.text(30, 108, 'source', 12, TEXT, weight='600')
    for k, (name, does, loses, colour) in enumerate(stages):
        x = 96 + k * 160
        arrow(s, x - 22, 128, x - 2, 128)
        box(s, x, 96, 140, 64, colour, name, does, op='0.3')
        s.line(x + 70, 162, x + 70, 196, RED, 1.5, ' stroke-dasharray="3 3"')
        s.rect(x + 8, 198, 124, 30, RED, rx=4, extra=' fill-opacity="0.16"')
        s.text(x + 70, 218, loses, 12, TEXT, 'middle')
    s.text(30, 270, 'Strings, symbols, RTTI, debug builds and save formats keep some of what the stages remove.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- order of evaluation
def eval_order():
    s = Svg(900, 300, 'Two random draws as the arguments of one call: the binary makes the second argument\'s draw first, '
                      'so the port makes its calls in the same order')
    s.heading('Order of evaluation', 'The language leaves the order of arguments open. The binary shows the order this build used.')
    s.rect(30, 84, 260, 40, '#12282a', rx=6)
    s.text(46, 109, 'spawn(rand() % w, rand() % h);', 13, TEXT, mono=True)
    s.text(30, 146, 'the source (not available)', 11, DIM)
    s.rect(330, 84, 250, 132, '#12282a', rx=6)
    asm = [('call  rand', AMBER), ('...   % h', DIM), ('push  eax', TEXT), ('call  rand', AMBER), ('...   % w', DIM),
           ('push  eax', TEXT)]
    for i, (line, colour) in enumerate(asm):
        s.text(346, 106 + i * 19, line, 12, colour, mono=True)
    s.text(330, 236, 'the binary: the height draw comes first', 11, DIM)
    s.rect(620, 84, 250, 132, '#12282a', rx=6)
    port = [('let y = rand() % h;', TEXT), ('let x = rand() % w;', TEXT), ('spawn(x, y);', TEXT)]
    for i, (line, colour) in enumerate(port):
        s.text(636, 116 + i * 30, line, 12, colour, mono=True)
    s.text(620, 236, 'the port: one statement per draw', 11, DIM)
    arrow(s, 292, 104, 326, 104)
    arrow(s, 582, 150, 616, 150)
    s.text(30, 276, 'A port that computes arguments from left to right without this would give every later draw a different value.', 12, DIM)
    return s.render()


DIAGRAMS = {
    'msvc-rand.svg': msvc_rand, 'entity-components.svg': entity_components, 'package-archive.svg': package_archive,
    'float-time.svg': float_time, 'integrators.svg': integrators, 'password-bits.svg': password_bits,
    'message-format.svg': message_format, 'save-fixup.svg': save_fixup, 'net-models.svg': net_models,
    'vtable-layout.svg': vtable_layout, 'crt-startup.svg': crt_startup, 'signature-match.svg': signature_match,
    'magic-division.svg': magic_division, 'cpu-flags.svg': cpu_flags, 'segment-address.svg': segment_address,
    'overlays.svg': overlays, 'struct-from-uses.svg': struct_from_uses,
    'compile-stages.svg': compile_stages, 'eval-order.svg': eval_order,
}
