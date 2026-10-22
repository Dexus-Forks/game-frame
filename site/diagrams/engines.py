"""Diagrams for the engine guides (docs/engines/): the layers of a game on an engine, licensees'
versions, Wolfenstein 3D's ray casting, Doom's BSP walk, WAD files and random table, Quake's
visibility sets, where id's engines keep the game code, 2.5D rendering, Build's portals and
timer, a Jedi elevator, Voxel Space, Unreal packages and natives, the DLLs of GoldSrc and
LithTech, NIF blocks, RenderWare chunks, Westwood MIX and Blizzard MPQ lookups, the Infinity
Engine's key file, middleware, SWF files, projectors, Flash frames, Director files, routes to a
port, the signs of an engine and of library code, and how to work through engine structures.
site/diagrams/make.py writes them. The other modules for the guides' sections import the small
helpers defined here.
"""
import math

from runtime import show
from svg import AMBER, BG, BLUE, DIM, EDGE, GREEN, GREY, MONO, PURPLE, RED, TEAL, TEXT, Svg, esc

PINK = '#f472b6'


# ---------------------------------------------------------------- helpers
def box(s, x, y, w, h, colour, title=None, sub=None, op=0.22, size=12, mono=False, sub_mono=False, stroke=None):
    """A tinted box with a title and an optional second line, centred."""
    s.rect(x, y, w, h, colour, stroke or 'none', rx=4, extra=f' fill-opacity="{op}"')
    if title is not None and sub is not None:
        s.text(x + w / 2, y + h / 2 - 3, title, size, TEXT, 'middle', '600', mono=mono)
        s.text(x + w / 2, y + h / 2 + 13, sub, 10.5, DIM, 'middle', mono=sub_mono)
    elif title is not None:
        s.text(x + w / 2, y + h / 2 + size * 0.35, title, size, TEXT, 'middle', '600', mono=mono)


def arrow(s, x1, y1, x2, y2, colour=DIM, width=1.5):
    s.line(x1, y1, x2, y2, colour, width, ' marker-end="url(#arrow)"')


def lines(s, x, y, rows, size=11.5, fill=DIM, gap=16, anchor='start', mono=False, weight='400'):
    for k, row in enumerate(rows):
        s.text(x, y + k * gap, row, size, fill, anchor, weight, mono=mono)


def during(t0, t1, dur, attr='opacity', on='1', off='0'):
    """An animate element that sets ATTR to ON between fractions T0 and T1 of DUR, OFF otherwise."""
    if t0 <= 0:
        return (f'<animate attributeName="{attr}" values="{on};{off}" keyTimes="0;{t1:.4f}" dur="{dur}s" '
                'calcMode="discrete" repeatCount="indefinite"/>')
    return (f'<animate attributeName="{attr}" values="{off};{on};{off}" keyTimes="0;{t0:.4f};{t1:.4f}" dur="{dur}s" '
            'calcMode="discrete" repeatCount="indefinite"/>')


def group(body, t0, t1, dur):
    """BODY shown between fractions T0 and T1 of DUR seconds."""
    return f'<g opacity="{1 if t0 <= 0 else 0}">{body}{during(t0, t1, dur)}</g>'


def scratch(fn, w=900, h=400):
    s = Svg(w, h, '')
    fn(s)
    return '\n'.join(s.parts)


def legend(s, x, y, items, gap=26):
    for colour, label in items:
        s.rect(x, y - 10, 12, 12, colour, rx=2, extra=' fill-opacity="0.6"')
        s.text(x + 18, y, label, 11, DIM)
        x += gap + len(label) * 6.2


# ---------------------------------------------------------------- README: the layers of a game on an engine
def layers():
    s = Svg(900, 370, 'The layers of a game built on an engine, from data and scripts down to the system, with the work that each layer needs')
    s.heading('The layers of a game built on an engine',
              'Each layer needs different work. Most of the work is in the layers that hold the game itself.')
    rows = [('Game data and scripts', 'QuakeC, UnrealScript, CON files, Infinity Engine scripts, rule tables', AMBER,
             'Read them first. Port the interpreter, then run the game\'s own data'),
            ('The game\'s native code', 'what the studio wrote in C or C++ on top of the engine', RED,
             'Reverse engineer it, because it holds the game'),
            ('The engine', 'renderer, file formats, script interpreter, object system', BLUE,
             'Look up its formats, its released source, its reimplementations'),
            ('Middleware', 'video (Smacker, Bink), sound (Miles, FMOD), physics (Havok)', PURPLE,
             'Identify it, label it, replace it with a library that does the same job'),
            ('System and hardware', 'DOS, Windows, DirectX, OpenGL, the sound card', GREY,
             'See the hardware guides')]
    y = 80
    for title, sub, colour, todo in rows:
        s.rect(24, y, 470, 50, colour, rx=5, extra=' fill-opacity="0.2"')
        s.text(40, y + 22, title, 13, TEXT, weight='600')
        s.text(40, y + 39, sub, 11, DIM)
        arrow(s, 500, y + 25, 530, y + 25)
        s.text(540, y + 29, todo, 11.5, TEXT)
        y += 56
    return s.render()


# ---------------------------------------------------------------- README: licensees' versions
def licensee_versions():
    s = Svg(900, 330, 'Duke Nukem 3D, Shadow Warrior and Blood each changed the Build engine. Deus Ex runs Unreal Engine 1 with Ion Storm\'s changes')
    s.heading('Licensed engines in several versions',
              'Each licensee changed the code it bought. A released source shows only one of these versions.')

    def tree(x, root, sub, kids, colour, first=None):
        box(s, x, 84, 360, 44, colour, root, sub, op=0.25)
        n = len(kids)
        for k, (name, note) in enumerate(kids):
            kx = x + k * (360 / n) + 4
            kw = 360 / n - 8
            s.line(x + 180, 128, kx + kw / 2, 170, DIM, 1.2)
            box(s, kx, 172, kw, 50, colour, name, None, op=0.12)
            s.text(kx + kw / 2, 240, note, 10.5, AMBER, 'middle')
            s.text(kx + kw / 2, 216, 'the first game' if name == first else 'changed', 10, DIM, 'middle')
    tree(24, 'Build engine', 'source released in 2000', [('Duke Nukem 3D', 'its own CON scripts'),
                                                          ('Shadow Warrior', 'its own C code'),
                                                          ('Blood', 'heavily changed')], BLUE)
    tree(500, 'Unreal Engine 1', 'licensed with source', [('Unreal', 'Epic\'s own game'), ('Deus Ex', 'Ion Storm\'s changes')],
         PURPLE, first='Unreal')
    s.rect(24, 270, 852, 44, AMBER, rx=5, extra=' fill-opacity="0.12"')
    s.text(40, 297, 'Confirm every detail against the binary you study before you rely on the source. '
                    'Record which version the source was.', 12, TEXT)
    return s.render()


# ---------------------------------------------------------------- id: Wolfenstein 3D ray casting (animated)
WOLF_MAP = ['############',
            '#..........#',
            '#..##......#',
            '#..##...#..#',
            '#.......#..#',
            '#..........#',
            '#....###...#',
            '#..........#',
            '#..........#',
            '############']


def wolf_raycast():
    s = Svg(900, 380, 'Wolfenstein 3D casts one ray per screen column against the tile grid. The distance to the '
                      'wall it hits sets the height of that column\'s wall slice')
    s.heading('Ray casting in Wolfenstein 3D',
              'The world is a grid of tiles. Each screen column casts one ray. The height of its slice depends on the distance to the wall.')
    cell, mx, my = 20, 40, 92
    for r, row in enumerate(WOLF_MAP):
        for c, ch in enumerate(row):
            if ch == '#':
                s.rect(mx + c * cell, my + r * cell, cell - 1, cell - 1, BLUE, rx=1, extra=' fill-opacity="0.45"')
            else:
                s.rect(mx + c * cell, my + r * cell, cell - 1, cell - 1, GREY, rx=1, extra=' fill-opacity="0.12"')
    px, py, heading, fov, n = 2.5, 7.5, -math.pi / 4.4, math.pi / 3, 40
    sx, sy, sw, sh = 330, 92, 540, 200
    s.rect(sx, sy, sw, sh / 2, GREY, rx=2, extra=' fill-opacity="0.18"')
    s.rect(sx, sy + sh / 2, sw, sh / 2, GREY, rx=2, extra=' fill-opacity="0.32"')
    dur, cw = 6.0, sw / n
    rays = []
    for i in range(n):
        ang = heading - fov / 2 + fov * (i + 0.5) / n
        dx, dy = math.cos(ang), math.sin(ang)
        # A digital differential analyser through the grid, as such renderers do.
        mxi, myi = int(px), int(py)
        ddx, ddy = abs(1 / dx) if dx else 1e9, abs(1 / dy) if dy else 1e9
        stx, sty = (1, (mxi + 1 - px) * ddx) if dx > 0 else (-1, (px - mxi) * ddx)
        sty_, sdy = (1, (myi + 1 - py) * ddy) if dy > 0 else (-1, (py - myi) * ddy)
        sdx, side = sty, 0
        while True:
            if sdx < sdy:
                sdx += ddx
                mxi += stx
                side = 0
            else:
                sdy += ddy
                myi += sty_
                side = 1
            if WOLF_MAP[myi][mxi] == '#':
                break
        dist = (sdx - ddx) if side == 0 else (sdy - ddy)
        perp = dist * math.cos(ang - heading)
        hx, hy = px + dx * dist, py + dy * dist
        h = min(sh, sh * 0.9 / perp)
        t0 = 0.85 * i / n
        colour = '#3b82c4' if side == 0 else '#2a5f96'
        s.add(f'<rect x="{sx + i * cw:.1f}" y="{sy + (sh - h) / 2:.1f}" width="{cw + 0.4:.1f}" height="{h:.1f}" '
              f'fill="{colour}" opacity="0">{during(t0, 0.97, dur)}</rect>')
        rays.append((t0, 0.85 * (i + 1) / n,
                     f'<path d="M{mx + px * cell:.1f} {my + py * cell:.1f}L{mx + hx * cell:.1f} {my + hy * cell:.1f}" '
                     f'stroke="{AMBER}" stroke-width="2" fill="none"/>'
                     f'<rect x="{sx + i * cw:.1f}" y="{sy - 6}" width="{cw:.1f}" height="4" fill="{AMBER}"/>'))
    for t0, t1, body in rays:
        s.add(group(body, t0, t1, dur))
    # The field of view, faintly, and the player.
    for a in (heading - fov / 2, heading + fov / 2):
        s.line(mx + px * cell, my + py * cell, mx + (px + math.cos(a) * 3) * cell, my + (py + math.sin(a) * 3) * cell,
               AMBER, 1, ' stroke-dasharray="3 3" opacity="0.6"')
    s.add(f'<circle cx="{mx + px * cell}" cy="{my + py * cell}" r="5" fill="{AMBER}"/>')
    s.text(mx, my + 10 * cell + 20, 'a part of the 64 x 64 tile map, from above', 11, DIM)
    s.text(sx, sy + sh + 22, 'the screen, with one slice per column, drawn from left to right', 11, DIM)
    s.text(sx, sy + sh + 44, 'Light and dark faces show whether the ray crossed a vertical or a horizontal grid line.', 11, DIM)
    s.text(24, 364, 'Sprites are drawn after the walls, clipped against the distance stored for each column. '
                    'Game time runs at 70 ticks a second.', 12, TEXT)
    return s.render()


# ---------------------------------------------------------------- id: Doom's BSP walk (animated)
def doom_bsp():
    s = Svg(900, 440, 'Doom\'s renderer walks the BSP tree and visits the player\'s side of each split first, so the subsectors come out from front to back')
    s.heading('Walking Doom\'s BSP tree front to back',
              'At each split, the renderer visits the player\'s side first. The subsectors come out in order from nearest to farthest.')
    x0, y0, w, h = 40, 92, 320, 220
    regions = {'A': (x0, y0, 160, 100), 'B': (x0, y0 + 100, 160, 120), 'C': (x0 + 160, y0, 160, 130),
               'D': (x0 + 160, y0 + 130, 160, 90)}
    order = ['B', 'A', 'D', 'C']
    dur = 8.0
    for name, (x, y, ww, hh) in regions.items():
        s.rect(x + 2, y + 2, ww - 4, hh - 4, GREY, rx=3, extra=' fill-opacity="0.12"')
        k = order.index(name)
        t0 = 0.12 + k * 0.2
        s.add(f'<rect x="{x + 2}" y="{y + 2}" width="{ww - 4}" height="{hh - 4}" rx="3" fill="{TEAL}" opacity="0">'
              f'{during(t0, 0.96, dur, on="0.35")}</rect>')
        s.text(x + ww / 2, y + hh / 2 - 4, name, 18, TEXT, 'middle', '700')
        s.add(f'<text x="{x + ww / 2}" y="{y + hh / 2 + 16}" font-family="{MONO}" font-size="12" fill="{AMBER}" '
              f'text-anchor="middle" opacity="0">drawn {k + 1}{during(t0, 0.96, dur)}</text>')
    s.line(x0 + 160, y0, x0 + 160, y0 + h, RED, 2.5)
    s.line(x0, y0 + 100, x0 + 160, y0 + 100, PURPLE, 2.5)
    s.line(x0 + 160, y0 + 130, x0 + w, y0 + 130, BLUE, 2.5)
    s.text(x0 + 164, y0 + h + 16, 'split 1', 11, RED)
    s.text(x0 - 4, y0 + 96, 'split 2', 11, PURPLE, 'end' if x0 > 60 else 'start')
    s.text(x0 + w - 44, y0 + 126, 'split 3', 11, BLUE)
    s.add(f'<circle cx="{x0 + 30}" cy="{y0 + 196}" r="6" fill="{AMBER}"/>')
    s.text(x0 + 42, y0 + 200, 'player', 11, AMBER)
    # The tree.
    nodes = {'n1': (640, 100, 'split 1', RED), 'n2': (540, 190, 'split 2', PURPLE), 'n3': (760, 190, 'split 3', BLUE)}
    leaves = {'A': 585, 'B': 495, 'D': 715, 'C': 805}
    s.line(640, 116, 540, 174, DIM, 1.5)
    s.line(640, 116, 760, 174, DIM, 1.5)
    for name, lx in leaves.items():
        parent = nodes['n2'] if name in 'AB' else nodes['n3']
        s.line(parent[0], parent[1] + 16, lx, 266, DIM, 1.5)
    for key, (x, y, label, colour) in nodes.items():
        box(s, x - 50, y - 16, 100, 32, colour, label, op=0.3)
    for name, lx in leaves.items():
        k = order.index(name)
        box(s, lx - 26, 266, 52, 34, GREY, None, op=0.25)
        s.add(f'<rect x="{lx - 26}" y="266" width="52" height="34" rx="4" fill="{TEAL}" opacity="0">'
              f'{during(0.12 + k * 0.2, 0.96, dur, on="0.55")}</rect>')
        s.text(lx, 288, name, 14, TEXT, 'middle', '600')
        s.text(lx, 322, f'{k + 1}', 13, AMBER, 'middle', '600')
    # The visiting marker.
    path = [(640, 100), (540, 190), (495, 283), (585, 283), (760, 190), (715, 283), (805, 283)]
    times = [0, 0.06, 0.12, 0.32, 0.46, 0.52, 0.72]
    xs = ';'.join(f'{x - 640} {y - 100}' for x, y in path)
    s.add(f'<circle cx="640" cy="100" r="22" fill="none" stroke="{AMBER}" stroke-width="2.5">'
          f'<animateTransform attributeName="transform" type="translate" values="{xs}" '
          f'keyTimes="{";".join(f"{t:.2f}" for t in times)}" calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/></circle>')
    lines(s, 450, 350, ['The player is left of split 1, so the left half comes first.', 'The player is below split 2, so B comes before A.',
                        'Each screen column is drawn once, so later walls fill', 'only the columns that are still empty.'], 11.5, DIM, gap=17)
    s.text(40, 350, 'An illustrative map of four subsectors.', 11, DIM)
    s.text(40, 424, 'World positions are 16.16 fixed point. The game state changes 35 times a second.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- id: a WAD file
def doom_wad():
    s = Svg(900, 360, 'A WAD file holds a 12-byte header, the lumps and a directory of 16-byte entries with the offset, size and name of each lump. A level is a marker lump followed by ten lumps')
    s.heading('The layout of a Doom WAD file',
              'Every piece of data is a named lump. The header gives the offset of the directory, which lists every lump.')
    y = 84
    box(s, 24, y, 150, 56, AMBER, 'header, 12 bytes', None, op=0.3)
    box(s, 180, y, 470, 56, BLUE, 'lump data, one lump after another', None, op=0.2)
    box(s, 656, y, 220, 56, TEAL, 'directory', None, op=0.3)
    lines(s, 24, y + 80, ['"IWAD" or "PWAD"', 'number of lumps', 'offset of the directory'], 11.5, TEXT, mono=True)
    s.text(24, y + 132, '4 bytes each, little-endian', 10.5, DIM)
    s.add(f'<path d="M100 {y}C 100 {y - 22} 766 {y - 22} 766 {y - 2}" fill="none" stroke="{AMBER}" '
          f'stroke-width="1.5" marker-end="url(#arrow)"/>')
    s.text(656, y + 80, 'each entry, 16 bytes:', 11.5, TEXT)
    lines(s, 656, y + 98, ['offset of the lump (4)', 'size of the lump (4)', 'name, 8 characters'], 11.5, DIM, mono=True)
    s.text(24, 262, 'A level is a marker lump of size 0, followed by these lumps in this order', 12, TEXT, weight='600')
    level = ['E1M1', 'THINGS', 'LINEDEFS', 'SIDEDEFS', 'VERTEXES', 'SEGS', 'SSECTORS', 'NODES', 'SECTORS', 'REJECT',
             'BLOCKMAP']
    x = 24
    for k, name in enumerate(level):
        w = 26 + len(name) * 7.4
        box(s, x, 276, w, 30, AMBER if k == 0 else BLUE, name, op=0.3 if k == 0 else 0.18, size=11, mono=True)
        x += w + 4
    s.text(24, 330, 'NODES holds the BSP tree, SSECTORS and SEGS its leaves. REJECT and BLOCKMAP speed up sight and '
                    'collision tests. A PWAD replaces lumps by name.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- id: Doom's random table (animated)
def doom_random():
    s = Svg(900, 400, 'Doom\'s P_Random reads the next entry of a fixed table of 256 bytes through an index that wraps around, so one extra call shifts every later number and breaks a demo')
    s.heading('Random numbers in Doom',
              'P_Random returns table[index] and adds one to the index. The index goes back to 0 after 255.')
    x0, y0, cw, ch = 40, 84, 25.5, 17
    for i in range(256):
        r, c = divmod(i, 32)
        s.rect(x0 + c * cw, y0 + r * ch, cw - 2, ch - 2, BLUE, rx=2, extra=' fill-opacity="0.18"')
        if c % 8 == 0:
            s.text(x0 + c * cw + 3, y0 + r * ch + 11, str(i), 8.5, DIM, mono=True)
    seq = list(range(248, 256)) + list(range(0, 8))
    vals_x = ';'.join(f'{x0 + (i % 32) * cw:.1f}' for i in seq)
    vals_y = ';'.join(f'{y0 + (i // 32) * ch:.1f}' for i in seq)
    kt = ';'.join(f'{k / len(seq):.4f}' for k in range(len(seq)))
    s.add(f'<rect width="{cw - 2}" height="{ch - 2}" rx="2" fill="{AMBER}">'
          f'<animate attributeName="x" values="{vals_x}" keyTimes="{kt}" calcMode="discrete" dur="6s" repeatCount="indefinite"/>'
          f'<animate attributeName="y" values="{vals_y}" keyTimes="{kt}" calcMode="discrete" dur="6s" repeatCount="indefinite"/></rect>')
    s.text(x0, y0 + 8 * ch + 18, 'the 256 entries, numbered 0 to 255; the index runs off the end and starts again at 0', 11, DIM)
    y = 270
    s.text(40, y, 'The original, playing a demo', 12, TEAL, weight='600')
    s.text(40, y + 54, 'A port with one extra call', 12, RED, weight='600')
    calls = [('imp attacks', 'a'), ('door', 'b'), ('pistol', 'c'), ('imp moves', 'd'), ('barrel', 'e')]
    for k, (name, _) in enumerate(calls):
        box(s, 250 + k * 122, y - 18, 116, 34, TEAL, name, f'index {100 + k}', op=0.2, size=11)
    port = [('imp attacks', 100, TEAL), ('extra call', 101, RED), ('door', 102, RED), ('pistol', 103, RED),
            ('imp moves', 104, RED)]
    for k, (name, idx, colour) in enumerate(port):
        box(s, 250 + k * 122, y + 36, 116, 34, colour, name, f'index {idx}', op=0.2, size=11)
    s.text(40, 384, 'Every call after the extra one reads a different number, so the demo goes out of sync. '
                    'The port must keep the order of calls.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- id: Quake's visibility (animated)
def quake_pvs():
    s = Svg(900, 360, 'For each leaf of its BSP tree, Quake stores a precomputed list of the leaves that can be seen from it. The renderer draws only those leaves')
    s.heading('Precomputed visibility in Quake',
              'For every leaf of the BSP tree, the compiler stores which leaves can be seen from it.')
    rooms = [(40, 90, 120, 80), (170, 90, 120, 80), (300, 90, 120, 80), (40, 180, 120, 80), (170, 180, 120, 80),
             (300, 180, 120, 80)]
    states = [(0, [1, 1, 1, 1, 0, 0]), (4, [0, 1, 0, 1, 1, 1])]
    dur = 6.0

    def draw(cam, bits):
        def f(t):
            for k, (x, y, w, h) in enumerate(rooms):
                colour = AMBER if k == cam else (TEAL if bits[k] else GREY)
                op = 0.55 if k == cam else (0.3 if bits[k] else 0.1)
                t.rect(x, y, w, h, colour, rx=3, extra=f' fill-opacity="{op}"')
                t.text(x + 8, y + 18, f'leaf {k}', 11, TEXT if bits[k] else DIM, mono=True)
                if not bits[k]:
                    t.text(x + w / 2, y + h / 2 + 10, 'not drawn', 10.5, DIM, 'middle')
            x, y, w, h = rooms[cam]
            t.add(f'<circle cx="{x + w / 2}" cy="{y + h / 2 + 6}" r="7" fill="{AMBER}"/>')
            t.text(470, 120, f'camera in leaf {cam}', 13, AMBER, weight='600')
            t.text(470, 148, f'visible set of leaf {cam}:', 12, TEXT)
            for k, b in enumerate(bits):
                t.rect(470 + k * 34, 158, 30, 30, TEAL if b else GREY, rx=3, extra=f' fill-opacity="{0.45 if b else 0.15}"')
                t.text(485 + k * 34, 178, str(b), 13, TEXT, 'middle', '600', mono=True)
                t.text(485 + k * 34, 204, str(k), 10, DIM, 'middle', mono=True)
            t.text(470, 226, 'one bit per leaf', 11, DIM)
        return scratch(f)
    s.add(show([draw(c, b) for c, b in states], dur))
    lines(s, 470, 262, ['The bits are stored run-length compressed in the BSP file.',
                        'Walls hide whole rooms, so most leaves are skipped at once.',
                        'The light on each surface is also precomputed, in light maps.'], 11.5, DIM, gap=18)
    s.text(40, 290, 'An illustrative map of six leaves.', 11, DIM)
    s.text(40, 340, 'The test is done before drawing anything, so the cost of a hidden room is almost nothing.', 12, TEXT)
    return s.render()


# ---------------------------------------------------------------- id: where the game code lives
def id_game_code():
    s = Svg(900, 380, 'Quake interprets QuakeC byte code in progs.dat. Quake II calls a native game DLL through tables. Quake III runs QVM byte code in three ways')
    s.heading('Where the game code lives in id\'s engines',
              'The engine changes little between games. Quake uses byte code, Quake II uses a DLL, and Quake III can use either.')
    cols = [('Quake (1996)', 24, BLUE), ('Quake II (1997)', 316, PURPLE), ('Quake III (1999)', 608, TEAL)]
    for title, x, colour in cols:
        s.text(x, 90, title, 13, colour, weight='600')
        box(s, x, 102, 268, 44, GREY, 'the engine', 'one executable', op=0.25)
    # Quake
    arrow(s, 158, 148, 158, 178)
    box(s, 24, 180, 268, 44, BLUE, 'progs.dat', 'QuakeC compiled to byte code', op=0.25, mono=True)
    lines(s, 24, 246, ['The engine interprets it.', 'Decompile progs.dat before you', 'read any machine code.',
                       'Client and server run in one process,', 'even in single player.'], 11.5)
    # Quake II
    box(s, 316, 180, 268, 44, PURPLE, 'gamex86.dll', 'the game, native C code', op=0.25, mono=True)
    s.line(400, 148, 400, 178, AMBER, 1.5, ' marker-end="url(#arrow)"')
    s.line(500, 178, 500, 148, TEAL, 1.5, ' marker-end="url(#arrow)"')
    s.text(392, 168, 'engine functions', 9.5, AMBER, 'end')
    s.text(508, 168, 'game functions', 9.5, TEAL)
    lines(s, 316, 246, ['Start at its one export, GetGameAPI.', 'It takes the engine\'s table of functions',
                        'and returns the game\'s table.', 'Licensees replaced the DLL.'], 11.5)
    # Quake III
    arrow(s, 742, 148, 742, 178)
    box(s, 608, 180, 268, 44, TEAL, 'qagame, cgame, ui', 'C compiled to QVM by a changed LCC', op=0.25)
    ways = ['interpreted', 'compiled on load', 'native DLL']
    for k, w in enumerate(ways):
        box(s, 608 + k * 91, 240, 86, 30, GREY, w, op=0.3, size=10.5)
    lines(s, 608, 296, ['The same code can run in three ways.', 'Data is in .pk3 files, which are ZIP', 'archives.'], 11.5)
    s.text(24, 360, 'In each case, read the game code first, because most of a licensee\'s changes are in it.', 12, TEXT)
    return s.render()


# ---------------------------------------------------------------- id: using the released source
def released_source():
    s = Svg(900, 350, 'Functions in a game\'s binary are matched to the released engine source. The functions that differ are the licensee\'s changes')
    s.heading('Using the released source',
              'Name each function in the binary from the source. The functions that do not match are the licensee\'s changes.')
    names = ['R_RenderView', 'P_MobjThinker', 'W_CacheLump', '(no match)', 'S_StartSound', '(changed)', 'Z_Malloc',
             '(no match)', 'M_Random']
    states = ['same', 'same', 'same', 'new', 'same', 'changed', 'same', 'new', 'same']
    s.text(40, 90, 'functions in the binary', 12, TEXT, weight='600')
    s.text(560, 90, 'the released source', 12, TEXT, weight='600')
    for k, (name, st) in enumerate(zip(names, states)):
        y = 100 + k * 23
        colour = {'same': TEAL, 'new': RED, 'changed': AMBER}[st]
        s.rect(40, y, 200, 19, colour, rx=3, extra=f' fill-opacity="{0.4 if st != "same" else 0.2}"')
        s.text(48, y + 14, f'FUN_{0x401000 + k * 0x2c0:06x}', 10.5, DIM, mono=True)
        if st == 'same':
            s.rect(560, y, 200, 19, TEAL, rx=3, extra=' fill-opacity="0.2"')
            s.text(568, y + 14, name, 10.5, TEXT, mono=True)
            s.line(242, y + 10, 558, y + 10, TEAL, 1, ' opacity="0.6"')
        elif st == 'changed':
            s.rect(560, y, 200, 19, AMBER, rx=3, extra=' fill-opacity="0.15"')
            s.text(568, y + 14, 'P_DamageMobj', 10.5, TEXT, mono=True)
            s.line(242, y + 10, 558, y + 10, AMBER, 1, ' stroke-dasharray="4 3"')
            s.text(400, y + 6, 'differs', 10, AMBER, 'middle')
        else:
            s.text(250, y + 14, 'the licensee\'s own code', 10.5, RED)
    legend(s, 40, 336, [(TEAL, 'same as the source, so name it'), (AMBER, 'changed by the licensee'),
                        (RED, 'new code, where most of the work is')])
    s.text(790, 140, 'Strings, error', 11, DIM)
    s.text(790, 156, 'messages and the', 11, DIM)
    s.text(790, 172, 'order of functions', 11, DIM)
    s.text(790, 188, 'help to match.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- 2.5D: columns and rows (animated)
def columns_rows():
    s = Svg(900, 330, 'A 2.5D renderer draws walls as vertical columns, one screen column at a time, then floors and '
                      'ceilings as horizontal rows')
    s.heading('How a 2.5D renderer draws walls and floors',
              'A wall has one depth down each screen column, so walls are drawn column by column. Floors are drawn row by row.')
    sx, sy, sw, sh = 40, 84, 400, 220
    dur = 6.0
    s.rect(sx, sy, sw, sh, GREY, rx=3, extra=' fill-opacity="0.12"')
    n = 40
    cw = sw / n
    tops = []
    for i in range(n):
        x = sx + i * cw
        u = (i + 0.5) / n
        # Two walls meeting in a corner: heights fall away from the middle.
        h = 60 + 110 * (1 - abs(u - 0.45) * 1.6)
        top = sy + sh / 2 - h / 2
        tops.append(top)
        colour = '#3b82c4' if u < 0.45 else '#2a5f96'
        s.add(f'<rect x="{x:.1f}" y="{top:.1f}" width="{cw + 0.3:.1f}" height="{h:.1f}" fill="{colour}" opacity="0">'
              f'{during(0.5 * i / n, 0.97, dur)}</rect>')
    rows = 14
    for j in range(rows):
        frac = j / rows
        # Floor rows below the walls, ceiling rows above, each row one shade.
        yb = sy + sh - (j + 1) * 7
        yt = sy + j * 7
        shade = 0.15 + 0.35 * (1 - frac)
        t0 = 0.5 + 0.4 * j / rows
        for y, colour in ((yb, AMBER), (yt, PURPLE)):
            # Only the columns where this row is clear of the wall.
            free = [i for i in range(n) if (y + 6 <= tops[i]) or (y >= sy + sh - (tops[i] - sy))]
            runs, start = [], None
            for i in range(n + 1):
                if i < n and i in free:
                    start = i if start is None else start
                elif start is not None:
                    runs.append((start, i))
                    start = None
            body = ''.join(f'<rect x="{sx + a * cw:.1f}" y="{y:.1f}" width="{(b - a) * cw:.1f}" height="6" fill="{colour}"/>'
                           for a, b in runs)
            s.add(f'<g opacity="0">{body}{during(t0, 0.97, dur, on=f"{shade:.2f}")}</g>')
    s.add(f'<rect y="{sy - 8}" width="{cw:.1f}" height="4" fill="{TEAL}"><animate attributeName="x" values="{sx};{sx + sw};{sx + sw}" '
          f'keyTimes="0;0.5;1" dur="{dur}s" calcMode="linear" repeatCount="indefinite"/></rect>')
    lines(s, 480, 100, ['', 'For each screen column, find the wall it sees and its',
                        'distance. Draw one vertical strip, scaled by that distance.'], 12, DIM, gap=18)
    s.text(480, 100, '1. Walls, column by column', 12, TEAL, weight='600')
    lines(s, 480, 176, ['', 'Each screen row of floor is at one distance, so one',
                        'texture step serves the whole row.'], 12, DIM, gap=18)
    s.text(480, 176, '2. Floors and ceilings, row by row', 12, AMBER, weight='600')
    lines(s, 480, 252, ['The map is flat polygons (sectors) with a floor and a', 'ceiling height. The view cannot look up or down far.'],
          11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- Build: sectors and portals (animated)
def build_portals():
    s = Svg(900, 400, 'In the Build engine, each wall points to the next wall of its sector. A portal wall also points to the sector next to it. The renderer walks from the player\'s sector through the portals')
    s.heading('Sectors joined by portals in the Build engine',
              'The walls of each sector form a loop. A red wall is a portal into the sector next to it.')
    sectors = {
        0: [(60, 220), (200, 220), (200, 330), (60, 330)],
        1: [(200, 220), (330, 180), (360, 300), (200, 330)],
        2: [(330, 180), (440, 120), (470, 250), (360, 300)],
        3: [(60, 100), (200, 100), (200, 220), (60, 220)],
    }
    portals = [((200, 220), (200, 330)), ((330, 180), (360, 300)), ((60, 220), (200, 220))]
    order = [0, 1, 3, 2]
    dur = 7.0
    for k, pts in sectors.items():
        d = 'M' + 'L'.join(f'{x} {y}' for x, y in pts) + 'Z'
        s.add(f'<path d="{d}" fill="{GREY}" fill-opacity="0.12" stroke="{BLUE}" stroke-width="2"/>')
        t0 = 0.08 + order.index(k) * 0.18
        s.add(f'<path d="{d}" fill="{TEAL}" opacity="0">{during(t0, 0.95, dur, on="0.3")}</path>')
        cx = sum(p[0] for p in pts) / len(pts)
        cy = sum(p[1] for p in pts) / len(pts)
        s.text(cx, cy - 4, f'sector {k}', 12, TEXT, 'middle', '600')
        s.add(f'<text x="{cx}" y="{cy + 14}" font-family="{MONO}" font-size="11" fill="{AMBER}" text-anchor="middle" '
              f'opacity="0">step {order.index(k) + 1}{during(t0, 0.95, dur)}</text>')
    for a, b in portals:
        s.line(a[0], a[1], b[0], b[1], RED, 3.5)
    # The wall loop of sector 0, with next-wall arrows.
    pts = sectors[0]
    for k in range(4):
        (x1, y1), (x2, y2) = pts[k], pts[(k + 1) % 4]
        mxp, myp = (x1 + x2) / 2, (y1 + y2) / 2
        dx, dy = (x2 - x1), (y2 - y1)
        ln = math.hypot(dx, dy)
        ux, uy = dx / ln * 12, dy / ln * 12
        s.line(mxp - ux, myp - uy, mxp + ux, myp + uy, AMBER, 1.8, ' marker-end="url(#arrow)"')
    s.add(f'<circle cx="90" cy="314" r="6" fill="{AMBER}"/>')
    s.text(102, 318, 'player', 11, AMBER)
    # Overlapping sectors.
    s.add(f'<path d="M520 150L640 150L640 260L520 260Z" fill="{PURPLE}" fill-opacity="0.2" stroke="{PURPLE}" stroke-width="2"/>')
    s.add(f'<path d="M570 190L700 190L700 300L570 300Z" fill="{BLUE}" fill-opacity="0.2" stroke="{BLUE}" '
          f'stroke-width="2" stroke-dasharray="5 3"/>')
    s.text(526, 170, 'sector 7', 11, TEXT)
    s.text(640, 292, 'sector 9', 11, TEXT)
    lines(s, 520, 326, ['Two sectors can share the same place', 'on the map. The engine does not check',
                        'this. The player never sees both at once.'], 11.5, DIM, gap=16)
    lines(s, 720, 120, ['Each wall stores:', '- the next wall of', '  its sector', '- the wall on the', '  other side, if a',
                        '  portal', '- the sector there'], 11.5, DIM, gap=17, mono=False)
    s.text(40, 384, 'The renderer starts in the player\'s sector and goes through portals it can see, drawing as it goes.', 12, TEXT)
    return s.render()


# ---------------------------------------------------------------- Build: units and the timer (animated)
def build_timing():
    s = Svg(900, 320, 'In Duke Nukem 3D, heights use a scale 16 times finer than horizontal positions. The timer runs at 120 ticks a second. The game logic runs on every fourth tick, 30 times a second')
    s.heading('Scales and rates in Duke Nukem 3D',
              'These values come from Duke Nukem 3D. Find the exact values in the code of your own game.')
    # Scales.
    s.text(40, 92, 'Positions are integers', 12, TEXT, weight='600')
    s.rect(40, 110, 160, 14, BLUE, rx=2, extra=' fill-opacity="0.45"')
    s.text(40, 142, 'x and y, 1024 units', 11.5, DIM, mono=True)
    s.rect(330, 110, 14, 160, AMBER, rx=2, extra=' fill-opacity="0.45"')
    s.text(352, 120, 'the same', 11.5, DIM)
    s.text(352, 138, 'distance in z', 11.5, DIM)
    s.text(352, 156, '16384 units', 11.5, DIM, mono=True)
    lines(s, 40, 176, ['Code that mixes them shifts by 4.', 'z >> 4 compares with x and y,', 'x << 4 compares with z.',
                       'Watch the shifts in maths code.'], 11.5, DIM, mono=False, gap=17)
    # The timer.
    tx, ty = 480, 100
    s.text(tx, 92, 'The timer runs at 120 ticks a second', 12, TEXT, weight='600')
    n = 24
    for k in range(n):
        x = tx + k * 16
        logic = k % 4 == 0
        s.rect(x, ty + 10, 12, 24, TEAL if logic else GREY, rx=2, extra=f' fill-opacity="{0.6 if logic else 0.25}"')
    s.add(f'<rect y="{ty + 4}" width="12" height="36" rx="2" fill="none" stroke="{AMBER}" stroke-width="2">'
          f'<animate attributeName="x" values="{";".join(str(tx + k * 16) for k in range(n))}" '
          f'keyTimes="{";".join(f"{k / n:.4f}" for k in range(n))}" calcMode="discrete" dur="4.8s" repeatCount="indefinite"/></rect>')
    s.text(tx, ty + 60, 'one tick lasts 1/120 s', 11.5, DIM)
    s.rect(tx, ty + 72, 12, 12, TEAL, rx=2, extra=' fill-opacity="0.6"')
    s.text(tx + 18, ty + 82, 'the game logic runs on every 4th tick, 30 times a second', 11.5, DIM)
    lines(s, tx, ty + 116, ['Frames are drawn as often as the machine allows.', 'Only the logic steps change the game.',
                            'Find the rate in the timer code of your game.'], 11.5, DIM, gap=17)
    s.text(40, 298, 'Files are in .GRP archives. Art is in .ART files with a shared palette and shade tables.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- Jedi: an INF elevator (animated)
def jedi_elevator():
    s = Svg(900, 330, 'A Jedi engine elevator from an INF file. The floor of a sector moves between two stops when a switch is used. The INF file describes this as a small state machine')
    s.heading('Level logic in the Jedi engine',
              'INF files describe elevators, doors, switches and triggers as small state machines. This example is a lift with two stops.')
    dur = 6.0
    # The side view.
    s.rect(40, 90, 300, 210, GREY, rx=3, extra=' fill-opacity="0.1"')
    s.line(40, 280, 340, 280, DIM, 1)
    s.text(48, 274, 'stop 0', 10.5, DIM)
    s.line(140, 160, 240, 160, DIM, 1, ' stroke-dasharray="3 3"')
    s.text(244, 164, 'stop 1', 10.5, DIM)
    s.add(f'<rect x="140" width="100" height="10" fill="{AMBER}"><animate attributeName="y" '
          f'values="270;270;150;150;270" keyTimes="0;0.2;0.45;0.75;1" dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.add(f'<rect x="60" y="230" width="20" height="30" rx="3" fill="{RED}">'
          f'<animate attributeName="fill-opacity" values="0.3;1;0.3;0.3" keyTimes="0;0.12;0.25;1" dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.text(70, 252, '', 10)
    s.text(60, 222, 'switch', 10.5, DIM)
    s.text(190, 310, 'the lift\'s sector, from the side', 10.5, DIM, 'middle')
    # The state machine.
    states = [('at stop 0', 420, 120), ('moving up', 640, 120), ('at stop 1', 640, 230), ('moving down', 420, 230)]
    times = [(0, 0.2), (0.2, 0.45), (0.45, 0.75), (0.75, 1.0)]
    for (name, x, y), (t0, t1) in zip(states, times):
        box(s, x, y, 160, 44, BLUE, name, op=0.2)
        s.add(f'<rect x="{x}" y="{y}" width="160" height="44" rx="4" fill="none" stroke="{AMBER}" stroke-width="2.5" '
              f'opacity="{1 if t0 == 0 else 0}">{during(t0, t1, dur)}</rect>')
    arrow(s, 582, 142, 636, 142)
    s.text(609, 134, 'switch', 10.5, AMBER, 'middle')
    arrow(s, 720, 166, 720, 226)
    s.text(728, 200, 'reached', 10.5, DIM)
    arrow(s, 636, 252, 584, 252)
    s.text(609, 276, 'delay', 10.5, DIM, 'middle')
    arrow(s, 500, 226, 500, 168)
    s.text(508, 200, 'reached', 10.5, DIM)
    s.text(420, 310, 'Dark Forces keeps INF files in .GOB archives. The Force Engine documents the language.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- Voxel Space (animated)
def _height(x, y):
    return (math.sin(x * 0.09) * math.cos(y * 0.07) * 0.5 + math.sin((x + y) * 0.045) * 0.35 +
            math.cos(x * 0.21 - y * 0.13) * 0.15) * 0.5 + 0.5


def voxel_space():
    s = Svg(900, 380, 'Voxel Space draws terrain from a height map and a colour map, one screen column at a time, '
                      'front to back, each column drawn only where it rises above what is already drawn')
    s.heading('How Voxel Space draws terrain',
              'For each screen column, the renderer steps away from the camera. It draws only what rises above the parts already drawn.')
    # The two maps, small.
    g = 12
    for name, ox, kind in (('height map', 40, 'h'), ('colour map', 40, 'c')):
        pass
    for j in range(g):
        for i in range(g):
            hv = _height(i * 10, j * 10)
            grey = int(40 + 200 * hv)
            s.rect(40 + i * 10, 90 + j * 10, 10, 10, f'#{grey:02x}{grey:02x}{grey:02x}', rx=0)
            r, gg, b = (int(60 + 100 * hv), int(110 + 90 * hv), int(50 + 40 * hv)) if hv > 0.35 else (40, 90, 170)
            s.rect(40 + i * 10, 230 + j * 10, 10, 10, f'#{r:02x}{gg:02x}{b:02x}', rx=0)
    s.text(40, 84, 'height map', 11, DIM)
    s.text(40, 224, 'colour map', 11, DIM)
    # The render: per column, march front to back.
    sx, sy, sw, sh, n = 200, 90, 660, 220, 66
    s.rect(sx, sy, sw, sh, '#16324a', rx=3)
    cw = sw / n
    dur = 6.0
    for i in range(n):
        ang = (i / n - 0.5) * 1.2
        ybuf = sh
        spans = []
        z = 4.0
        while z < 160:
            wx, wy = 60 + math.sin(ang) * z * 1.0, 20 + math.cos(ang) * z * 1.0
            hv = _height(wx, wy)
            water = hv <= 0.35
            hv = max(hv, 0.35)
            proj = (25 - hv * 30) * 200 / z + sh * 0.25
            if proj < ybuf:
                if water:
                    col = (40, 90, 170)
                else:
                    col = (int(60 + 100 * hv), int(110 + 90 * hv), int(50 + 40 * hv))
                fade = max(0.35, 1 - z / 190)
                col = tuple(int(c * fade + 22 * (1 - fade)) for c in col)
                spans.append((max(proj, 0), ybuf, col))
                ybuf = max(proj, 0)
            z += 1 + z * 0.04
        body = ''.join(f'<rect x="{sx + i * cw:.1f}" y="{sy + a:.1f}" width="{cw + 0.4:.1f}" height="{b - a:.1f}" '
                       f'fill="#{c[0]:02x}{c[1]:02x}{c[2]:02x}"/>' for a, b, c in spans if b - a > 0.4)
        s.add(f'<g opacity="0">{body}{during(0.7 * i / n, 0.97, dur)}</g>')
    s.text(sx, sy + sh + 20, 'Each column steps away from the camera through the maps and keeps the highest point drawn so far.', 11, DIM)
    s.text(40, 364, 'NovaLogic used it in Comanche (1992) and Delta Force (1998). Maps are images, so the terrain has no polygons.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- Unreal: packages
def unreal_package():
    s = Svg(900, 416, 'An Unreal package has a header that points at a name table, an import table and an export table. Objects refer to each other by index, positive for exports and negative for imports')
    s.heading('The layout of an Unreal package',
              'Code, textures, sounds and maps are all stored in packages. Every reference is an index into one of the tables.')
    box(s, 24, 84, 200, 128, AMBER, None, op=0.22)
    s.text(36, 104, 'header', 12, TEXT, weight='600')
    lines(s, 36, 124, ['tag  C1 83 2A 9E', 'version', 'name count, offset', 'export count, offset', 'import count, offset'],
          10.5, DIM, mono=True, gap=17)
    tables = [('name table', 260, TEAL, ['0  None', '1  Engine', '2  Weapon', '3  Class', '4  MyWeapon', '5  Fire']),
              ('import table', 470, PURPLE, ['1  Class Engine.Weapon', '2  Package Engine', '3  Class Core.Class']),
              ('export table', 680, BLUE, ['1  MyWeapon, class -3', '   parent -1', '2  Fire, inside 1'])]
    for title, x, colour, rows in tables:
        s.rect(x, 84, 196, 160, colour, rx=4, extra=' fill-opacity="0.16"')
        s.text(x + 10, 104, title, 12, TEXT, weight='600')
        lines(s, x + 10, 126, rows, 10.5, TEXT, mono=True, gap=18)
    arrow(s, 226, 130, 256, 130)
    s.text(24, 280, 'An object index in the file', 12, TEXT, weight='600')
    rows = [('above 0', 'an export, entry number index in the export table', BLUE),
            ('below 0', 'an import, entry number -index in the import table', PURPLE),
            ('0', 'no object', GREY)]
    for k, (v, meaning, colour) in enumerate(rows):
        s.rect(24, 292 + k * 28, 80, 22, colour, rx=3, extra=' fill-opacity="0.3"')
        s.text(64, 307 + k * 28, v, 11, TEXT, 'middle', '600')
        s.text(116, 307 + k * 28, meaning, 11.5, DIM)
    lines(s, 560, 286, ['MyWeapon\'s class is -3, which is import 3,', 'Core.Class. Its parent is -1, which is', 'import 1, Engine.Weapon. Names are',
                        'indices into the name table too.'], 11.5, DIM, gap=18)
    s.text(24, 400, 'Licensees changed the format. Read the version in each header and check which fields your version has.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- Unreal: script calls native code (animated)
def unreal_natives():
    s = Svg(900, 340, 'UnrealScript byte code in a .u package runs in the interpreter in Core.dll. A native call goes by its fixed number through a table to C++ code in an engine DLL')
    s.heading('How UnrealScript calls native functions',
              'The interpreter reads byte code tokens. A native call goes by a fixed number through a table to C++ code.')
    dur = 5.0
    box(s, 24, 90, 210, 54, AMBER, 'Weapon.u', 'compiled UnrealScript', op=0.25)
    toks = ['tok', 'tok', 'native n', 'tok', 'tok']
    for k, t in enumerate(toks):
        box(s, 24 + k * 42, 160, 40, 26, RED if 'native' in t else GREY, t if 'native' not in t else 'n', op=0.3, size=10.5)
    s.text(24, 206, 'byte code tokens', 10.5, DIM)
    box(s, 300, 90, 230, 96, TEAL, None, op=0.18)
    s.text(415, 114, 'Core.dll', 12, TEXT, 'middle', '600')
    lines(s, 415, 136, ['the object system, names', 'and the script interpreter, which', 'reads a token and runs its handler'], 11, DIM,
          anchor='middle', gap=16)
    box(s, 600, 90, 276, 96, PURPLE, None, op=0.16)
    s.text(612, 112, 'table of native functions', 12, TEXT, weight='600')
    for k in range(4):
        y = 122 + k * 15
        s.text(612, y + 10, f'[{"n" if k == 2 else "."}]', 10, AMBER if k == 2 else DIM, mono=True)
        s.text(640, y + 10, 'AActor::execSpawn' if k == 2 else '...', 10.5, TEXT if k == 2 else DIM, mono=True)
    box(s, 600, 220, 276, 54, BLUE, 'Engine.dll', 'C++ code, exported with mangled names', op=0.25)
    arrow(s, 236, 117, 296, 117)
    arrow(s, 532, 138, 596, 138)
    arrow(s, 738, 188, 738, 216)
    # A token travelling.
    s.add(f'<circle r="7" fill="{AMBER}"><animateMotion path="M128 173L330 150L620 162L738 240" dur="{dur}s" '
          f'keyPoints="0;0.33;0.7;1;1" keyTimes="0;0.3;0.6;0.85;1" calcMode="linear" repeatCount="indefinite"/></circle>')
    lines(s, 24, 246, ['Native functions have fixed numbers, which the script', 'calls. Map the numbers to names early.',
                       'Demangle the DLL exports to name the C++ side.'], 11.5, DIM, gap=18)
    s.text(24, 324, 'The tick takes the frame time, a variable step. Replays need the frame times recorded.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- GoldSrc and LithTech: game DLLs
def game_dlls():
    s = Svg(900, 350, 'GoldSrc and LithTech load the engine and then the game code from separate DLLs. Half-Life uses hl.dll and client.dll. LithTech games use object.lto and cshell.dll')
    s.heading('Game code in separate DLLs',
              'Other games share the engine. The DLLs hold the code of this game, so study them first.')
    # GoldSrc row.
    s.text(24, 92, 'GoldSrc (Half-Life, 1998)', 13, TEAL, weight='600')
    box(s, 24, 104, 120, 50, GREY, 'hl.exe', 'the launcher', op=0.3, mono=True)
    arrow(s, 146, 129, 176, 129)
    box(s, 180, 104, 190, 50, BLUE, 'hw.dll or sw.dll', 'hardware or software engine', op=0.25, mono=True)
    arrow(s, 372, 118, 420, 104)
    arrow(s, 372, 140, 420, 156)
    box(s, 424, 80, 240, 46, RED, 'hl.dll  (server)', 'monsters, weapons, rules', op=0.25, mono=True)
    box(s, 424, 134, 240, 46, AMBER, 'client.dll  (client)', 'HUD, prediction, effects', op=0.25, mono=True)
    lines(s, 684, 100, ['Their source is in the', 'Half-Life SDK. Maps are BSP', 'version 30 with WAD3 textures.'],
          11.5, DIM, gap=17)
    # LithTech row.
    s.text(24, 222, 'LithTech (Shogo, No One Lives Forever)', 13, TEAL, weight='600')
    box(s, 24, 234, 120, 50, GREY, 'lithtech.exe', 'the engine', op=0.3, mono=True)
    arrow(s, 146, 248, 420, 236)
    arrow(s, 146, 272, 420, 288)
    box(s, 424, 212, 240, 46, RED, 'object.lto  (server)', 'the game objects', op=0.25, mono=True)
    box(s, 424, 266, 240, 46, AMBER, 'cshell.dll  (client)', 'the client side', op=0.25, mono=True)
    lines(s, 684, 232, ['Ordinary Windows DLLs', 'under other names. Data is', 'in .REZ archives.'], 11.5, DIM, gap=17)
    s.text(24, 336, 'Mods replace these DLLs. A compatible engine (Xash3D FWGS for GoldSrc) can run the original game DLLs.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- NetImmerse: NIF blocks
def nif_blocks():
    s = Svg(900, 360, 'A NIF file is a list of blocks, each one of a class such as NiNode or NiTriShape, which refer to '
                      'each other by block number and so make a scene graph')
    s.heading('The blocks of a NIF file',
              'Each block is one object of a class. Blocks refer to each other by block number.')
    blocks = [('0', 'NiNode', 'children: 1, 3'), ('1', 'NiTriShape', 'data: 2, properties: 4'),
              ('2', 'NiTriShapeData', 'vertices, triangles'), ('3', 'NiTriShape', 'data: 5, properties: 4'),
              ('4', 'NiTexturingProperty', 'texture: 6'), ('5', 'NiTriShapeData', 'vertices, triangles'),
              ('6', 'NiSourceTexture', 'file name')]
    s.text(24, 92, 'The header says "NetImmerse File Format" or "Gamebryo File Format", then gives the version.', 11.5, DIM)
    for k, (n, cls, refs) in enumerate(blocks):
        y = 104 + k * 30
        s.rect(24, y, 34, 26, AMBER, rx=3, extra=' fill-opacity="0.3"')
        s.text(41, y + 18, n, 12, TEXT, 'middle', '600', mono=True)
        s.rect(62, y, 330, 26, BLUE, rx=3, extra=' fill-opacity="0.16"')
        s.text(72, y + 18, cls, 11.5, TEXT, mono=True)
        s.text(240, y + 18, refs, 10.5, DIM)
    # The tree.
    pos = {'0': (640, 110), '1': (540, 180), '3': (740, 180), '2': (480, 260), '4': (640, 260), '5': (800, 260),
           '6': (640, 310)}
    edges = [('0', '1'), ('0', '3'), ('1', '2'), ('1', '4'), ('3', '4'), ('3', '5'), ('4', '6')]
    for a, b in edges:
        s.line(pos[a][0], pos[a][1] + 12, pos[b][0], pos[b][1] - 12, DIM, 1.3)
    names = {'0': 'NiNode', '1': 'NiTriShape', '3': 'NiTriShape', '2': 'data', '4': 'texturing', '5': 'data',
             '6': 'texture'}
    for k, (x, y) in pos.items():
        box(s, x - 52, y - 12, 104, 24, PURPLE if k in '13' else (TEAL if k == '0' else GREY), f'{k} {names[k]}',
            op=0.3, size=10.5)
    s.text(830, 114, 'block 4 is', 10.5, DIM, 'middle')
    s.text(830, 128, 'shared', 10.5, DIM, 'middle')
    s.text(24, 346, 'NifTools documents every version.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- RenderWare: chunks (animated)
def rw_chunks():
    s = Svg(900, 360, 'A RenderWare file is made of chunks. Each chunk has a 12-byte header of type, size and library version, so a reader can skip a chunk or step into it')
    s.heading('The chunks of a RenderWare file',
              'Every chunk starts with its type, its size and the library version. A reader moves from one header to the next.')
    s.text(24, 92, 'chunk header, 12 bytes', 12, TEXT, weight='600')
    for k, (name, colour) in enumerate((('type', AMBER), ('size', TEAL), ('version', PURPLE))):
        box(s, 24 + k * 92, 102, 88, 30, colour, name, '4 bytes', op=0.3, size=11)
    s.text(310, 122, 'then size bytes of content, which may be more chunks', 11.5, DIM)
    rows = [(0, 'Clump', '0x10'), (1, 'Struct', '0x01'), (1, 'Frame List', '0x0E'), (1, 'Geometry List', '0x1A'),
            (2, 'Geometry', '0x0F'), (2, 'Geometry', '0x0F'), (1, 'Atomic', '0x14'), (1, 'Extension', '0x03')]
    y0 = 158
    for k, (depth, name, code) in enumerate(rows):
        y = y0 + k * 22
        x = 40 + depth * 34
        s.rect(x, y, 330 - depth * 34, 19, BLUE, rx=3, extra=f' fill-opacity="{0.12 + depth * 0.08}"')
        s.text(x + 8, y + 14, name, 11, TEXT)
        s.text(360, y + 14, code, 10.5, AMBER, 'end', mono=True)
    s.text(40, y0 + len(rows) * 22 + 18, 'a model (.dff), chunk by chunk, with the code of each chunk type', 10.5, DIM)
    s.add(f'<rect x="34" width="340" height="21" rx="3" fill="none" stroke="{AMBER}" stroke-width="2">'
          f'<animate attributeName="y" values="{";".join(str(y0 - 1 + k * 22) for k in range(len(rows)))}" '
          f'keyTimes="{";".join(f"{k / len(rows):.4f}" for k in range(len(rows)))}" calcMode="discrete" dur="5s" '
          f'repeatCount="indefinite"/></rect>')
    lines(s, 440, 170, ['A reader can skip a chunk that it does not know', 'by jumping ahead by its size.',
                        '', 'The same chunk format holds PC and PlayStation 2', 'data, with platform data inside. A PC reader',
                        'helps with the console version.', '', 'Models are .dff, texture dictionaries .txd,',
                        'streams .rws.'], 11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- Westwood: MIX archives
def mix_archive():
    s = Svg(900, 300, 'A Westwood MIX archive names its entries by a hash of the file name. The index holds the hash, the offset and the size of each entry. The names themselves are not stored')
    s.heading('File names stored as hashes in Westwood MIX archives',
              'From Command & Conquer on, the index stores a hash of each file name. The name itself is not stored.')
    box(s, 24, 92, 150, 40, AMBER, '"RULES.INI"', op=0.25, mono=True)
    arrow(s, 176, 112, 216, 112)
    box(s, 220, 92, 140, 40, PURPLE, 'hash function', op=0.25)
    arrow(s, 362, 112, 402, 112)
    box(s, 406, 92, 120, 40, TEAL, 'an id', '32 bits', op=0.25, size=11)
    s.text(560, 104, 'The game hashes the name it wants,', 11.5, DIM)
    s.text(560, 122, 'then finds that id in the index.', 11.5, DIM)
    s.rect(24, 160, 852, 30, GREY, rx=3, extra=' fill-opacity="0.15"')
    parts = [('count, body size', 24, 150, GREY), ('index of id, offset and size per entry', 178, 300, TEAL),
             ('the files\' data', 482, 394, BLUE)]
    for label, x, w, colour in parts:
        box(s, x, 160, w - 4, 30, colour, label, op=0.25, size=11)
    lines(s, 24, 222, ['A listing of a MIX shows only ids. Names come from the game\'s strings, from other tools\'',
                       'name lists, or from guessing and hashing. Dune II and Lands of Lore used .PAK archives with names.'],
          11.5, DIM, gap=18)
    s.text(24, 280, 'Rules for units and buildings are in INI files, so read them before the code.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- Blizzard: MPQ lookup (animated)
def mpq_lookup():
    s = Svg(900, 390, 'To find a file in a Blizzard MPQ archive, the game hashes the name to a start slot in the hash table. Two more hashes confirm the entry, which leads to the block table and the data')
    s.heading('Finding a file in a Blizzard MPQ archive',
              'The game computes three hashes of the file name. The first picks a slot in the table. The other two confirm the entry.')
    box(s, 24, 92, 210, 40, AMBER, '"units\\human\\marine.grp"', op=0.25, size=10.5, mono=True)
    lines(s, 24, 156, ['hash 0 mod the table size gives the start slot', 'hash A and hash B are stored to confirm it'], 11, DIM, gap=17)
    x0, y0 = 280, 92
    slots = [('empty', GREY), ('other', BLUE), ('other', BLUE), ('match', TEAL), ('empty', GREY), ('other', BLUE),
             ('empty', GREY), ('other', BLUE)]
    s.text(x0, y0 - 6, 'hash table', 11, DIM)
    for k, (st, colour) in enumerate(slots):
        s.rect(x0, y0 + k * 26, 190, 22, colour, rx=3, extra=f' fill-opacity="{0.35 if st == "match" else 0.15}"')
        s.text(x0 + 8, y0 + k * 26 + 15, f'{k}', 10, DIM, mono=True)
        s.text(x0 + 30, y0 + k * 26 + 15, {'empty': 'empty', 'other': 'A, B of another name', 'match': 'A, B match'}[st],
               10.5, TEXT if st == 'match' else DIM)
    # The probe: start at 1, step to 2, then 3 (match).
    s.add(f'<rect x="{x0 - 3}" width="196" height="26" rx="4" fill="none" stroke="{AMBER}" stroke-width="2.5">'
          f'<animate attributeName="y" values="{y0 + 26 - 2};{y0 + 52 - 2};{y0 + 78 - 2}" keyTimes="0;0.25;0.5" '
          f'calcMode="discrete" dur="5s" repeatCount="indefinite"/></rect>')
    s.add(group(f'<text x="{x0 + 204}" y="{y0 + 82}" font-family="ui-sans-serif, Helvetica, Arial, sans-serif" '
                f'font-size="11" fill="{TEAL}">found</text>', 0.5, 1, 5))
    s.text(x0, y0 + 8 * 26 + 18, 'if the slot does not match, try the next one', 10.5, DIM)
    s.text(x0, y0 + 8 * 26 + 34, 'an empty slot means that the file is absent', 10.5, DIM)
    arrow(s, 474, y0 + 89, 560, y0 + 89)
    box(s, 564, 150, 150, 60, PURPLE, 'block table', 'offset, sizes, flags', op=0.22)
    arrow(s, 716, 180, 746, 180)
    box(s, 750, 150, 126, 60, BLUE, 'file data', 'in sectors', op=0.22)
    lines(s, 564, 236, ['Flags say if the file is compressed', '(several methods) or encrypted.',
                        'The key comes from the file name.'], 11.5, DIM, gap=17)
    s.text(24, 374, 'StormLib reads MPQ archives. storm.dll exports its functions by ordinal; community lists name many.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- Infinity Engine: chitin.key
def infinity_key():
    s = Svg(900, 320, 'The Infinity Engine\'s chitin.key lists every BIF archive and every resource, with the archive '
                      'and the place in it where the resource is')
    s.heading('The key file and the BIF archives of the Infinity Engine',
              'chitin.key lists every resource. Each entry says which .bif archive holds the resource.')
    s.rect(24, 86, 330, 200, AMBER, rx=4, extra=' fill-opacity="0.12"')
    s.text(36, 106, 'chitin.key', 12, TEXT, weight='600', mono=True)
    s.text(36, 128, 'BIF list', 11, DIM)
    lines(s, 48, 146, ['0  data\\AREAS.bif', '1  data\\CREATURES.bif', '2  data\\ITEMS.bif'], 10.5, TEXT, mono=True, gap=16)
    s.text(36, 206, 'resources with name, type and locator (examples)', 11, DIM)
    lines(s, 48, 224, ['AREA01   .are   bif 0, entry 3', 'GUARD01  .cre   bif 1, entry 41', 'SWORD01  .itm   bif 2, entry 7'],
          10.5, TEXT, mono=True, gap=16)
    for k, (name, colour) in enumerate((('AREAS.bif', TEAL), ('CREATURES.bif', BLUE), ('ITEMS.bif', PURPLE))):
        y = 92 + k * 66
        box(s, 470, y, 190, 52, colour, name, 'numbered entries', op=0.22, mono=True)
        s.add(f'<path d="M356 {220 + k * 16}C 410 {220 + k * 16} 420 {y + 26} 466 {y + 26}" fill="none" stroke="{DIM}" '
              f'stroke-width="1.3" marker-end="url(#arrow)"/>')
    lines(s, 690, 104, ['Areas (.are), creatures', '(.cre), items (.itm), spells', '(.spl), dialogues (.dlg)',
                        'and 2DA tables hold', 'almost all the rules.', '', 'IESDP documents every', 'format.'],
          11.5, DIM, gap=17)
    s.text(24, 306, 'Scripts (.bcs) are trigger and action blocks, run in a fixed order for each creature and area.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- studio engines: how to approach
def studio_approach():
    s = Svg(900, 290, 'Four steps for a studio engine. List the archives, read the rule tables and text, list the opcodes of the script interpreter, then trace the native code for what remains')
    s.heading('Four steps for a studio engine',
              'In these games, many rules are stored as data. Read the data before the code.')
    steps = [('1. Archives', 'list every file', 'usually documented', AMBER),
             ('2. Rule tables', 'INI, 2DA, records', 'many rules are here', TEAL),
             ('3. Scripts', 'list the opcodes', 'mission and quest logic', PURPLE),
             ('4. Native code', 'what remains', 'simulation, combat, paths, AI', RED)]
    for k, (title, sub, note, colour) in enumerate(steps):
        x = 24 + k * 216
        box(s, x, 92, 196, 60, colour, title, sub, op=0.25)
        s.text(x + 98, 174, note, 11, DIM, 'middle')
        if k < 3:
            arrow(s, x + 198, 122, x + 214, 122)
    s.text(24, 216, 'Examples', 12, TEXT, weight='600')
    lines(s, 24, 236, ['Westwood uses .MIX and RULES.INI. Blizzard uses .MPQ. The Infinity Engine uses chitin.key, .2da and .bcs.',
                       'Fallout uses .DAT archives, .PRO prototypes and scripts in SSL compiled to .INT byte code.'], 11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- middleware: map
def middleware_map():
    s = Svg(900, 430, 'Middleware libraries around a game for video, sound, physics, animation, scripting, compression, online play and protection, with the signs of each and how the port treats it')
    s.heading('Middleware libraries around a game',
              'Identify each library, label it and replace it. A few libraries need their exact behaviour in the port.')
    box(s, 350, 170, 200, 70, GREY, 'game.exe', 'the game and its engine', op=0.3, mono=True)
    libs = [('Video', 'Smacker, Bink', 'SMK2, BIK, binkw32.dll', 24, 84, BLUE),
            ('Sound', 'Miles, FMOD, DIGPAK', 'mss32.dll, fmod.dll', 24, 170, BLUE),
            ('Animation', 'Granny 3D', '.gr2, granny2.dll', 24, 256, BLUE),
            ('Physics', 'Havok, PhysX', 'hk prefixes in RTTI', 626, 84, AMBER),
            ('Scripting', 'Lua, Python', 'lua_pcall, "Lua 5.0"', 626, 170, AMBER),
            ('Compression', 'zlib, LZSS, implode', 'CRC table 0x77073096', 626, 256, BLUE),
            ('Online', 'GameSpy, DirectPlay', 'dplayx.dll', 252, 336, GREY),
            ('Protection', 'SafeDisc, SecuROM', 'secdrv.sys, .icd', 470, 336, GREY)]
    for title, names, sign, x, y, colour in libs:
        s.rect(x, y, 250 if x != 252 and x != 470 else 178, 62, colour, rx=4, extra=' fill-opacity="0.16"')
        s.text(x + 10, y + 19, title, 12, TEXT, weight='600')
        s.text(x + 10, y + 36, names, 11, TEXT)
        s.text(x + 10, y + 52, sign, 10.5, DIM, mono=True)
    for x, y in ((276, 115), (276, 201), (276, 270)):
        s.line(x, y, 348, 205, DIM, 1, ' opacity="0.5"')
    for x, y in ((624, 115), (624, 201), (624, 270)):
        s.line(x, y, 552, 205, DIM, 1, ' opacity="0.5"')
    s.line(340, 336, 420, 242, DIM, 1, ' opacity="0.5"')
    s.line(560, 336, 480, 242, DIM, 1, ' opacity="0.5"')
    legend(s, 24, 420, [(BLUE, 'replace with a library that does the same job'),
                        (AMBER, 'may set game results, so check before replacing'), (GREY, 'leave out of the port')])
    return s.render()


# ---------------------------------------------------------------- middleware: a shared rand (animated)
def shared_rand():
    s = Svg(900, 330, 'When a library also calls the C runtime\'s rand, its calls take numbers from the game\'s sequence; '
                      'a port without those calls gives the game different numbers')
    s.heading('A library that shares the game\'s rand()',
              'A library that calls the same generator uses up numbers from the game\'s sequence. Without it, every later draw changes.')
    dur = 6.0
    rows = [('the original', [('game', 1), ('game', 2), ('sound library', 3), ('game', 4), ('game', 5), ('sound library', 6),
                              ('game', 7)], 92),
            ('a port with its own sound', [('game', 1), ('game', 2), ('game', 3), ('game', 4), ('game', 5), ('game', 6),
                                            ('game', 7)], 192)]
    for title, calls, y in rows:
        s.text(24, y, title, 12, TEXT, weight='600')
        for k, (who, n) in enumerate(calls):
            x = 24 + k * 122
            colour = PURPLE if who != 'game' else TEAL
            body = scratch(lambda t, x=x, y=y, who=who, n=n, colour=colour: box(t, x, y + 10, 116, 44, colour, who, f'rand() #{n}', op=0.25, size=11))
            s.add(group(body, 0.1 * k, 0.97, dur))
    marks = [(2, 'same'), (3, 'differs'), (4, 'differs')]
    s.text(24, 286, 'The game\'s third draw is #4 in the original and #3 in the port. From there on, every result differs.', 12, TEXT)
    s.text(24, 308, 'Trace every caller of the random function, including library code.', 11.5, DIM)
    return s.render()


# ---------------------------------------------------------------- Flash: the SWF header and tags
def swf_layout():
    s = Svg(900, 380, 'An SWF file starts with a signature of FWS, CWS or ZWS, a version and a length. The stage size, frame rate and frame count follow, then tags that each have a code and a length')
    s.heading('The layout of an SWF file',
              'A short header comes first, followed by a list of tags. CWS files are compressed with zlib from byte 8. ZWS files use LZMA.')
    hdr = [('F/C/Z', 3, AMBER), ('W S', 0, AMBER), ('ver', 1, PURPLE), ('length', 4, TEAL)]
    x = 24
    for label, n, colour in (('F/C/Z', 1, AMBER), ('W', 1, AMBER), ('S', 1, AMBER), ('ver', 1, PURPLE),
                             ('file length', 4, TEAL)):
        w = n * 42
        box(s, x, 90, w - 3, 34, colour, label, op=0.3, size=11)
        x += w
    s.text(24, 142, 'bytes 0 to 7, never compressed', 10.5, DIM)
    rest = [('stage size, bit-packed, in twips', 270, BLUE), ('frame rate, 8.8', 110, PURPLE),
            ('frame count', 100, PURPLE)]
    x = 370
    for label, w, colour in rest:
        box(s, x, 90, w - 3, 34, colour, label, op=0.25, size=10.5)
        x += w
    s.text(370, 142, 'compressed in CWS and ZWS', 10.5, DIM)
    s.text(24, 182, 'Each tag header, 16 bits', 12, TEXT, weight='600')
    x = 24
    for label, n, colour in (('tag code', 10, AMBER), ('length', 6, TEAL)):
        box(s, x, 192, n * 22 - 3, 30, colour, f'{label}, {n} bits', op=0.3, size=10.5)
        x += n * 22
    s.text(380, 212, 'length 63 means a 32-bit length follows', 11, DIM)
    tags = [('9', 'SetBackgroundColor'), ('2', 'DefineShape'), ('21', 'DefineBitsJPEG2'), ('39', 'DefineSprite'),
            ('82', 'DoABC'), ('76', 'SymbolClass'), ('12', 'DoAction'), ('1', 'ShowFrame'), ('0', 'End')]
    s.text(24, 256, 'Tags in a file, in order, with code and name', 12, TEXT, weight='600')
    x, y = 24, 268
    for code, name in tags:
        w = 36 + len(name) * 7
        if x + w > 880:
            x, y = 24, y + 38
        colour = RED if name in ('DoABC', 'DoAction') else (AMBER if name == 'ShowFrame' else BLUE)
        s.rect(x, y, w, 30, colour, rx=3, extra=' fill-opacity="0.22"')
        s.text(x + 8, y + 19, code, 10.5, DIM, mono=True)
        s.text(x + 30, y + 19, name, 11, TEXT, mono=True)
        x += w + 6
    legend(s, 24, 366, [(RED, 'scripts, DoAction for AVM1 and DoABC for AVM2'), (AMBER, 'ShowFrame ends a frame'),
                        (BLUE, 'shapes, images, sprites')])
    return s.render()


# ---------------------------------------------------------------- Flash: a projector
def flash_projector():
    s = Svg(900, 260, 'A Windows Flash projector is the Flash Player program with the SWF added at the end, followed by 8 bytes that hold the marker 0xFA123456 and the length of the movie')
    s.heading('The layout of a Flash projector',
              'The file holds the player program, then the movie, then 8 bytes that give the position of the movie.')
    box(s, 24, 94, 420, 50, GREY, 'Flash Player program', 'an ordinary .exe', op=0.25)
    box(s, 448, 94, 300, 50, BLUE, 'the SWF movie', 'FWS or CWS ...', op=0.28)
    box(s, 752, 94, 60, 50, AMBER, '56 34 12 FA', None, op=0.35, size=9.5, mono=True)
    box(s, 816, 94, 60, 50, TEAL, 'length', None, op=0.35, size=10.5)
    s.text(782, 162, 'marker', 10.5, DIM, 'middle')
    s.text(846, 162, '4 bytes', 10.5, DIM, 'middle')
    s.add(f'<path d="M846 146C 846 200 600 200 452 150" fill="none" stroke="{TEAL}" stroke-width="1.5" marker-end="url(#arrow)"/>')
    s.text(24, 190, 'movie start = file size - 8 - length', 12, TEXT, mono=True)
    s.text(24, 214, 'The marker 0xFA123456 is stored little-endian. swfinfo.py --extract copies the movie out.', 11.5, DIM)
    s.text(24, 240, 'A Director projector works the same way, with the Director player and the movie inside.', 11.5, DIM)
    return s.render()


# ---------------------------------------------------------------- Flash: frames run late (animated)
def flash_frames():
    s = Svg(900, 330, 'Flash games run on frames at the rate in the header; when a frame takes too long the next one '
                      'starts late, so a game that counts frames runs slower in real time')
    s.heading('How Flash games run on frames',
              'enterFrame runs once a frame. A frame that takes too long delays the next one. A game that counts frames then runs slower.')
    dur = 6.0
    x0, w = 160, 700
    for k in range(13):
        s.line(x0 + k * (w / 12), 84, x0 + k * (w / 12), 212, EDGE, 1)
        s.text(x0 + k * (w / 12), 226, f'{k * 1 / 30 * 1000:.0f}', 9.5, DIM, 'middle', mono=True)
    s.text(x0 + w, 242, 'milliseconds, at 30 frames a second', 10.5, DIM, 'end')
    s.text(24, 120, 'as the header asks', 11.5, TEXT)
    for k in range(12):
        s.rect(x0 + k * (w / 12) + 2, 104, w / 12 - 4, 26, TEAL, rx=3, extra=' fill-opacity="0.35"')
        s.text(x0 + k * (w / 12) + w / 24, 122, f'{k + 1}', 10.5, TEXT, 'middle', mono=True)
    s.text(24, 190, 'a busy frame', 11.5, TEXT)
    x = x0
    lengths = [1, 1, 1, 2.5, 1, 1, 2, 1, 1, 1]
    for k, ln in enumerate(lengths):
        wpx = ln * w / 12
        if x + wpx > x0 + w + 0.01:
            break
        colour = RED if ln > 1 else TEAL
        s.rect(x + 2, 174, wpx - 4, 26, colour, rx=3, extra=' fill-opacity="0.35"')
        s.text(x + wpx / 2, 192, f'{k + 1}', 10.5, TEXT, 'middle', mono=True)
        x += wpx
    s.add(f'<rect y="84" width="2" height="128" fill="{AMBER}"><animate attributeName="x" values="{x0};{x0 + w}" '
          f'dur="{dur}s" repeatCount="indefinite"/></rect>')
    lines(s, 24, 274, ['After 12 frames of time, the busy run has finished only 9 frames. An enemy that moves',
                       '2 pixels a frame has moved only 18 of the expected 24 pixels. A port must decide which to copy.'], 11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- Director files
def director_files():
    s = Svg(900, 300, 'Director files come as editable movies and casts, protected versions without the script source, compressed Shockwave versions and projectors. All of them are RIFX or XFIR containers')
    s.heading('Director and Shockwave files',
              'All forms use one container format, RIFX on a Mac (big-endian) or XFIR on Windows (little-endian).')
    forms = [('.dir  .cst', 'editable movie and cast', 'Lingo source and byte code', BLUE),
             ('.dxr  .cxt', 'protected', 'source text removed, byte code kept', PURPLE),
             ('.dcr  .cct', 'Shockwave, for the web', 'compressed ("Afterburner")', AMBER),
             ('projector', 'the player with the movie inside', '.exe or Mac application', GREY)]
    for k, (ext, what, note, colour) in enumerate(forms):
        x = 24 + k * 216
        box(s, x, 92, 200, 70, colour, ext, what, op=0.22, mono=True)
        s.text(x + 100, 182, note, 10.5, DIM, 'middle')
        if k < 2:
            arrow(s, x + 202, 127, x + 214, 127)
    lines(s, 24, 222, ['ProjectorRays decompiles Lingo byte code back to source and unpacks projectors.',
                       'ScummVM\'s Director engine runs a growing number of these games.',
                       'Xtras (.x32, .xtr) are native plug-ins in ordinary machine code.'], 11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- Flash: routes to a port
def flash_routes():
    s = Svg(900, 300, 'A Flash game can run unchanged in Ruffle, be reimplemented from the decompiled scripts, or be rebuilt from the decompiled project with the AIR SDK')
    s.heading('Routes for a Flash game',
              'A Flash game can run in Ruffle, be reimplemented or be rebuilt from the decompiled project.')
    box(s, 24, 150, 130, 56, AMBER, 'game.swf', 'and what it loads', op=0.3)
    routes = [('Run it in Ruffle', 'the game stays as it was, with no port', 'gaps are fixed in Ruffle itself', TEAL, 90),
              ('Reimplement it', 'translate the frame loop and clips', 'check against captures; mark deviations', BLUE, 158),
              ('Rebuild the decompiled project', 'FFDec exports, the AIR SDK compiles', 'the original\'s code is in it, '
               'so take legal care', RED, 226)]
    for title, a, b, colour, y in routes:
        s.add(f'<path d="M156 178C 200 178 200 {y + 26} 240 {y + 26}" fill="none" stroke="{DIM}" stroke-width="1.5" '
              f'marker-end="url(#arrow)"/>')
        s.rect(244, y, 632, 54, colour, rx=4, extra=' fill-opacity="0.16"')
        s.text(258, y + 22, title, 12.5, TEXT, weight='600')
        s.text(258, y + 41, a, 11, DIM)
        s.text(560, y + 41, b, 11, DIM)
    return s.render()


# ---------------------------------------------------------------- recognising: where to look
def recognise_clues():
    s = Svg(900, 360, 'The places that name an engine are file extensions and magic numbers, strings, DLLs beside the executable, the PDB path, RTTI class names, configuration files and the credits')
    s.heading('Where an engine shows itself',
              'Check these places during inventory, before you decompile. Record each one as evidence.')
    clues = [('archive magic', '"PACK", "IWAD", "KenSilverman"', AMBER),
             ('strings', 'versions, copyrights, errors', AMBER),
             ('DLLs beside it', 'Engine.dll, glide2x.dll, mss32.dll', BLUE),
             ('the PDB path', 'the studio\'s folder names', BLUE),
             ('RTTI names', '.?AVUObject@@  .?AVNiNode@@', PURPLE),
             ('configuration', '[Engine.Engine], config.cfg', PURPLE),
             ('credits, manual', '"Uses Bink Video"', GREY)]
    for k, (title, ex, colour) in enumerate(clues):
        y = 84 + k * 38
        s.rect(24, y, 380, 32, colour, rx=4, extra=' fill-opacity="0.18"')
        s.text(36, y + 21, title, 12, TEXT, weight='600')
        s.text(160, y + 21, ex, 10.5, DIM, mono=True)
        s.add(f'<path d="M406 {y + 16}C 470 {y + 16} 470 200 530 200" fill="none" stroke="{DIM}" stroke-width="1" opacity="0.6"/>')
    box(s, 534, 150, 342, 100, TEAL, None, op=0.18)
    s.text(550, 176, 'docs/FINDINGS.md', 12, TEXT, weight='600', mono=True)
    lines(s, 550, 198, ['engine: Unreal Engine 1', 'evidence: Core.dll, Engine.dll,', '.u packages starting C1 83 2A 9E'],
          11, DIM, mono=False, gap=17)
    s.text(534, 280, 'A changed version number or one extra', 11.5, DIM)
    s.text(534, 298, 'field is common. Compare the header with', 11.5, DIM)
    s.text(534, 316, 'the engine\'s documented layout.', 11.5, DIM)
    return s.render()


# ---------------------------------------------------------------- recognising: library code (animated)
def library_code():
    s = Svg(900, 330, 'An executable\'s functions, with the C runtime, the C++ library, zlib, middleware and compiler helpers labelled first. The functions left unlabelled are the game and its engine')
    s.heading('Label the libraries first',
              'Ghidra cannot tell the game\'s functions from a library\'s. Once the libraries are labelled, the rest is the game and its engine.')
    import random
    rnd = random.Random(7)
    kinds = [('crt', TEAL, 0.22), ('cpp', BLUE, 0.1), ('zlib', PURPLE, 0.05), ('mw', AMBER, 0.12), ('helpers', GREY, 0.06)]
    cells = []
    for i in range(240):
        r = rnd.random()
        acc, kind = 0, 'game'
        for name, colour, p in kinds:
            acc += p
            if r < acc:
                kind = name
                break
        cells.append(kind)
    colours = dict((n, c) for n, c, _ in kinds)
    order = ['crt', 'cpp', 'zlib', 'mw', 'helpers']
    dur = 7.0
    for i, kind in enumerate(cells):
        r, c = divmod(i, 30)
        x, y = 24 + c * 28.4, 86 + r * 22
        s.rect(x, y, 26, 18, GREY, rx=2, extra=' fill-opacity="0.2"')
        if kind == 'game':
            s.add(f'<rect x="{x:.1f}" y="{y}" width="26" height="18" rx="2" fill="{RED}" opacity="0">'
                  f'{during(0.78, 0.97, dur, on="0.6")}</rect>')
        else:
            t0 = 0.1 + order.index(kind) * 0.13
            s.add(f'<rect x="{x:.1f}" y="{y}" width="26" height="18" rx="2" fill="{colours[kind]}" opacity="0">'
                  f'{during(t0, 0.97, dur, on="0.55")}</rect>')
    legend(s, 24, 286, [(TEAL, 'C runtime'), (BLUE, 'C++ library'), (PURPLE, 'zlib'), (AMBER, 'middleware'),
                        (GREY, 'compiler helpers'), (RED, 'what is left, the game and its engine')], gap=18)
    s.text(24, 314, 'Function signatures, copyright strings and constant tables (zlib\'s CRC table 0x77073096) find most of them.', 11.5, DIM)
    return s.render()


# ---------------------------------------------------------------- engine structures: how to work with them
def structures_steps():
    s = Svg(900, 360, 'Working through an engine\'s structures. Class names lead to type records. The factory table leads to each class\'s creation function and vtable. The stream loader gives the file format, and the main loop gives the update order')
    s.heading('Engine structures, in order',
              'Each step names a set of functions and leads to the next step.')
    steps = [('1. Type records', 'class names in the strings,', 'parent pointers make the tree', TEAL),
             ('2. Factory table', 'name or hash + create function,', 'each leads to a vtable', BLUE),
             ('3. Stream loader', 'calls the factory per object;', 'its reads are the file format', PURPLE),
             ('4. Update order', 'scene graph, then controllers,', 'or the other way round', AMBER)]
    for k, (title, a, b, colour) in enumerate(steps):
        x = 24 + k * 216
        s.rect(x, 86, 200, 74, colour, rx=5, extra=' fill-opacity="0.2"')
        s.text(x + 12, 108, title, 12.5, TEXT, weight='600')
        s.text(x + 12, 128, a, 11, DIM)
        s.text(x + 12, 145, b, 11, DIM)
        if k < 3:
            arrow(s, x + 202, 123, x + 214, 123)
    # A small class tree.
    tree = [('Object', 140, 200), ('Spatial', 90, 250), ('Controller', 190, 250), ('Node', 64, 300), ('Geometry', 146, 300)]
    for (name, x, y), parent in zip(tree, [None, 0, 0, 1, 1]):
        if parent is not None:
            px, py = tree[parent][1], tree[parent][2]
            s.line(x, y - 10, px, py + 10, TEAL, 1.2)
        box(s, x - 38, y - 10, 76, 20, TEAL, name, op=0.25, size=10)
    s.text(24, 336, 'IsA walks the parent pointers', 10.5, DIM)
    # Factory table.
    for k, (n, f) in enumerate((('"Node"', 'Node::Create'), ('"Geometry"', 'Geometry::Create'), ('"Light"', 'Light::Create'))):
        s.text(240, 214 + k * 22, n, 10.5, TEXT, mono=True)
        s.text(330, 214 + k * 22, f, 10.5, DIM, mono=True)
    # Streaming two passes.
    s.text(456, 210, 'pass 1 creates every object', 10.5, TEXT)
    s.text(456, 232, 'pass 2 turns ids into pointers', 10.5, TEXT)
    s.text(456, 262, 'the order of pass 1 sets the', 10.5, DIM)
    s.text(456, 278, 'order of lists after loading', 10.5, DIM)
    lines(s, 672, 210, ['frame:', '  update scene graph', '  update controllers', '  draw'], 10.5, TEXT, mono=True, gap=17)
    return s.render()


DIAGRAMS = {
    'engine-layers.svg': layers, 'engine-licensee-versions.svg': licensee_versions,
    'engine-wolf3d-raycast.svg': wolf_raycast, 'engine-doom-bsp.svg': doom_bsp, 'engine-doom-wad.svg': doom_wad,
    'engine-doom-random.svg': doom_random, 'engine-quake-pvs.svg': quake_pvs, 'engine-id-game-code.svg': id_game_code,
    'engine-released-source.svg': released_source, 'engine-columns-rows.svg': columns_rows,
    'engine-build-portals.svg': build_portals, 'engine-build-timing.svg': build_timing,
    'engine-jedi-elevator.svg': jedi_elevator, 'engine-voxel-space.svg': voxel_space,
    'engine-unreal-package.svg': unreal_package, 'engine-unreal-natives.svg': unreal_natives,
    'engine-game-dlls.svg': game_dlls, 'engine-nif-blocks.svg': nif_blocks, 'engine-rw-chunks.svg': rw_chunks,
    'engine-mix-archive.svg': mix_archive, 'engine-mpq-lookup.svg': mpq_lookup, 'engine-infinity-key.svg': infinity_key,
    'engine-studio-approach.svg': studio_approach, 'engine-middleware-map.svg': middleware_map,
    'engine-shared-rand.svg': shared_rand, 'engine-swf-layout.svg': swf_layout,
    'engine-flash-projector.svg': flash_projector, 'engine-flash-frames.svg': flash_frames,
    'engine-director-files.svg': director_files, 'engine-flash-routes.svg': flash_routes,
    'engine-recognise-clues.svg': recognise_clues, 'engine-library-code.svg': library_code,
    'engine-structures-steps.svg': structures_steps,
}
