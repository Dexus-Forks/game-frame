"""The third set of the guides' diagrams, for the run-time and verification guides: memory
compared frame by frame, a skeleton's parent array, the bits of a handle, a pool allocator's
free list, and the layouts of the C++ standard containers. site/diagrams/make.py writes them
with the others.
"""
from svg import AMBER, BG, BLUE, DIM, GREEN, GREY, PURPLE, RED, TEAL, TEXT, Svg


def show(groups, dur):
    """Each group shown in turn for an equal share of DUR seconds, forever."""
    n = len(groups)
    out = []
    for i, body in enumerate(groups):
        times = [0] + [k / n for k in (i, i + 1) if 0 < k < n]
        vals = ([] if i == 0 else ['0']) + ['1'] + (['0'] if i < n - 1 else [])
        out.append(f'<g opacity="{vals[0]}">{body}<animate attributeName="opacity" values="{";".join(vals)}" '
                   f'keyTimes="{";".join(f"{k:.4f}" for k in times)}" dur="{dur}s" calcMode="discrete" '
                   f'repeatCount="indefinite"/></g>')
    return '\n'.join(out)


def part(fn, w=900, h=400):
    """The SVG elements FN draws on a scratch Svg, without the card: for groups that animate."""
    s = Svg(w, h, '')
    fn(s)
    return '\n'.join(s.parts)


# ---------------------------------------------------------------- memory frame by frame (animated)
def ram_compare():
    s = Svg(900, 420, 'Memory compared frame by frame: the original in an emulator and the port run the same inputs, '
                      'and the first frame and address that differ are reported')
    s.heading('Memory compared frame by frame',
              'The same input movie in the emulator and in the port. After each frame, both write the game\'s memory.')
    s.rect(24, 74, 250, 40, TEAL, rx=6, extra=' fill-opacity="0.18"')
    s.text(149, 99, 'original in the emulator', 13, TEXT, 'middle', '600')
    s.rect(300, 74, 250, 40, BLUE, rx=6, extra=' fill-opacity="0.18"')
    s.text(425, 99, 'the port, same inputs', 13, TEXT, 'middle', '600')
    s.rect(576, 74, 300, 40, AMBER, rx=6, extra=' fill-opacity="0.18"')
    s.text(726, 99, 'ramdiff.py ORIGINAL PORT --map', 13, TEXT, 'middle', '600', mono=True)
    rows = [('0x0009', 'frame counter', None), ('0x0075', 'player_x', 99), ('0x0076', 'player_y', 6),
            ('0x0100', 'score', 8), ('0x0102', 'lives', 99), ('0x0140', 'enemy_x', 9)]
    x0, y0, cw, ch = 210, 150, 64, 26
    for f in range(10):
        s.text(x0 + f * cw + cw / 2, y0 - 10, f'frame {f}', 11, DIM, 'middle')
    for r, (addr, name, bad) in enumerate(rows):
        y = y0 + r * (ch + 6)
        s.text(24, y + 17, addr, 12, DIM, mono=True)
        s.text(84, y + 17, name, 12, TEXT if bad is not None else DIM)
        for f in range(10):
            if bad is None:
                colour, op = GREY, '0.25'
            elif f >= bad:
                colour, op = RED, '0.85'
            else:
                colour, op = TEAL, '0.3'
            s.rect(x0 + f * cw + 3, y, cw - 6, ch, colour, rx=3, extra=f' fill-opacity="{op}"')
    lx, ly = x0 + 3, y0 + 6 * (ch + 6) + 8
    for colour, op, label in ((GREY, '0.25', 'left out with --ignore'), (TEAL, '0.3', 'the same'), (RED, '0.85', 'different')):
        s.rect(lx, ly, 12, 12, colour, rx=2, extra=f' fill-opacity="{op}"')
        s.text(lx + 18, ly + 10, label, 11, DIM)
        lx += 40 + len(label) * 6
    # A cursor walks the frames and stops at the first one that differs.
    h = 6 * (ch + 6) + 4
    steps = ';'.join(f'{f * cw} 0' for f in range(7))
    s.add(f'<rect x="{x0}" y="{y0 - 4}" width="{cw}" height="{h}" rx="5" fill="none" stroke="{AMBER}" stroke-width="2.5">'
          f'<animateTransform attributeName="transform" type="translate" values="{steps};{6 * cw} 0" '
          f'keyTimes="{";".join(f"{k / 10:.2f}" for k in range(7))};1" calcMode="discrete" dur="6s" '
          f'repeatCount="indefinite"/></rect>')
    s.add(f'<g opacity="0"><text x="{x0 + 6 * cw + cw / 2}" y="{y0 + 6 * (ch + 6) + 44}" font-family="ui-sans-serif, '
          f'Helvetica, Arial, sans-serif" font-size="13" font-weight="600" fill="{AMBER}" text-anchor="middle">'
          f'first difference: frame 6, 0x0076 player_y</text>'
          f'<animate attributeName="opacity" values="0;0;1;1" keyTimes="0;0.6;0.6;1" dur="6s" repeatCount="indefinite"/></g>')
    s.text(24, 404, 'Fix the first difference and run again. The later ones (score, enemy_x) are often the same error spreading.',
           12, DIM)
    return s.render()


# ---------------------------------------------------------------- a skeleton's parent array (animated)
def skeleton_parents():
    s = Svg(900, 420, 'A skeleton stored as an array of parent indices, parents before children, and the tree it '
                      'makes; one pass from the start computes every joint\'s world pose')
    s.heading('A skeleton as an array of parents',
              'Each joint stores its parent\'s index. Parents come first, so one pass from the start poses every joint.')
    joints = [('pelvis', -1), ('spine', 0), ('chest', 1), ('neck', 2), ('head', 3), ('l_arm', 2), ('l_hand', 5),
              ('r_arm', 2), ('r_hand', 7), ('l_leg', 0), ('r_leg', 0)]
    x0, w, y = 40, 74, 86
    s.text(x0, y - 8, 'index', 11, DIM)
    for i, (name, p) in enumerate(joints):
        x = x0 + i * w
        s.text(x + w / 2 - 2, y + 10, str(i), 11, DIM, 'middle', mono=True)
        s.rect(x, y + 16, w - 4, 30, BLUE, rx=3, extra=' fill-opacity="0.22"')
        s.text(x + w / 2 - 2, y + 36, str(p), 13, TEXT, 'middle', '600', mono=True)
        s.text(x + w / 2 - 2, y + 62, name, 11, DIM, 'middle')
    s.text(x0 + 11 * w + 4, y + 36, 'parent', 11, DIM)
    pos = {0: (450, 196), 1: (450, 246), 2: (450, 296), 3: (450, 346), 4: (450, 390), 5: (330, 330), 6: (230, 376),
           7: (570, 330), 8: (670, 376), 9: (300, 246), 10: (600, 246)}
    for i, (_, p) in enumerate(joints):
        if p >= 0:
            s.line(*pos[p], *pos[i], DIM, 1.5)
    dur = 8.8
    for i, (name, _) in enumerate(joints):
        x, yy = pos[i]
        s.rect(x - 38, yy - 13, 76, 26, GREY, rx=13, extra=' fill-opacity="0.35"')
        t0 = i / len(joints)
        s.add(f'<rect x="{x - 38}" y="{yy - 13}" width="76" height="26" rx="13" fill="{TEAL}" opacity="0">'
              f'<animate attributeName="opacity" values="0;0;0.85;0.85;0" keyTimes="0;{t0:.3f};{t0:.3f};0.95;1" '
              f'dur="{dur}s" repeatCount="indefinite"/></rect>')
        s.text(x, yy + 4, name, 11, TEXT, 'middle', '600')
        ax = x0 + i * w
        s.add(f'<rect x="{ax - 2}" y="{y + 14}" width="{w}" height="34" rx="4" fill="none" stroke="{AMBER}" '
              f'stroke-width="2" opacity="0"><animate attributeName="opacity" values="0;0;1;1;0;0" '
              f'keyTimes="0;{t0:.3f};{t0:.3f};{t0 + 1 / len(joints):.3f};{t0 + 1 / len(joints):.3f};1" dur="{dur}s" '
              f'repeatCount="indefinite"/></rect>')
    s.text(760, 196, 'world[i] =', 12, TEXT, mono=True)
    s.text(760, 214, '  world[parent[i]]', 12, TEXT, mono=True)
    s.text(760, 232, '  x local[i]', 12, TEXT, mono=True)
    s.text(760, 262, 'The root\'s parent', 11, DIM)
    s.text(760, 278, 'is -1 (all bits set).', 11, DIM)
    s.text(760, 302, 'skeleton.py finds', 11, DIM)
    s.text(760, 318, 'these arrays in files.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- the bits of a handle
def handle_bits():
    s = Svg(900, 360, 'A 32-bit handle: 12 bits of generation and 20 bits of index; a lookup compares the generation '
                      'in the handle with the one in the slot')
    s.heading('A handle: an index and a generation',
              'The index picks a slot. The generation must match the slot\'s, or the object is gone.')
    x0, bw, y = 40, 25, 86
    value = 0x00300042
    for b in range(32):
        bit = 31 - b
        on = (value >> bit) & 1
        colour = AMBER if bit >= 20 else BLUE
        s.rect(x0 + b * bw, y, bw - 2, 30, colour, rx=2, extra=f' fill-opacity="{0.85 if on else 0.18}"')
        s.text(x0 + b * bw + bw / 2 - 1, y + 20, str(on), 12, BG if on else DIM, 'middle', '600', mono=True)
        if bit in (31, 20, 19):
            s.text(x0 + b * bw + bw / 2 - 1, y - 6, str(bit), 10, DIM, 'middle', mono=True)
    s.line(x0, y + 40, x0 + 12 * bw - 2, y + 40, AMBER, 2)
    s.text(x0 + 6 * bw, y + 58, 'generation = handle >> 20  (3)', 12, AMBER, 'middle', mono=True)
    s.line(x0 + 12 * bw, y + 40, x0 + 32 * bw - 2, y + 40, BLUE, 2)
    s.text(x0 + 22 * bw, y + 58, 'index = handle & 0xFFFFF  (0x42 = 66)', 12, BLUE, 'middle', mono=True)
    s.text(x0 + 32 * bw - 2, y - 6, 'handle = 0x00300042', 12, TEXT, 'end', mono=True)
    # The slot table.
    ty = 196
    s.text(x0, ty - 10, 'the table of slots', 11, DIM)
    for k, (idx, gen, obj) in enumerate([(64, 1, 'tree'), (65, 7, 'cart'), (66, 3, 'citizen'), (67, 2, 'house')]):
        x = x0 + k * 130
        hl = idx == 66
        s.rect(x, ty, 120, 54, TEAL if hl else GREY, rx=4, extra=f' fill-opacity="{0.3 if hl else 0.2}"')
        s.text(x + 10, ty + 20, f'slot {idx}', 12, TEXT, weight='600')
        s.text(x + 10, ty + 40, f'gen {gen}  {obj}', 12, DIM, mono=True)
    s.text(x0, ty + 86, '0x00300042 -> slot 66, generation 3 = 3: the citizen', 12, GREEN, mono=True)
    s.text(x0, ty + 108, '0x00200042 -> slot 66, generation 2 != 3: the old object was freed; the lookup returns null',
           12, RED, mono=True)
    s.text(x0, ty + 140, 'In the code: a mask (and eax, 0xFFFFF) for the index, a shift (shr eax, 20) for the generation, '
                         'then a compare with the slot.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- a pool's free list (animated)
def pool_freelist():
    s = Svg(900, 330, 'A pool allocator: equal slots, with the free ones linked through their first bytes; '
                      'allocating takes the head and freeing pushes the slot back, so the last freed slot is reused first')
    s.heading('A pool allocator\'s free list',
              'Free slots hold a pointer to the next free slot. Allocate takes the head. Free puts a slot back as the head.')
    states = [
        ('the start: free list 2 -> 5 -> 7', {0, 1, 3, 4, 6}, [2, 5, 7]),
        ('allocate: takes slot 2, the head', {0, 1, 2, 3, 4, 6}, [5, 7]),
        ('free slot 4: it becomes the head, 4 -> 5 -> 7', {0, 1, 2, 3, 6}, [4, 5, 7]),
        ('allocate: takes slot 4, the one freed last', {0, 1, 2, 3, 4, 6}, [5, 7]),
    ]
    x0, w, y = 120, 90, 170

    def draw(used, free, caption):
        def fn(g):
            for i in range(8):
                x = x0 + i * w
                u = i in used
                g.rect(x, y, w - 8, 56, TEAL if u else GREY, rx=4, extra=f' fill-opacity="{0.55 if u else 0.2}"')
                g.text(x + (w - 8) / 2, y + 24, f'slot {i}', 12, TEXT if u else DIM, 'middle', '600')
                nxt = 'next: null' if free and i == free[-1] else 'next ->'
                g.text(x + (w - 8) / 2, y + 44, 'in use' if u else nxt, 11, TEXT if u else DIM, 'middle')
            g.text(40, y + 34, 'head', 13, AMBER, weight='600')
            if free:
                hx = x0 + free[0] * w + (w - 8) / 2
                g.add(f'<path d="M78 {y + 30}C 100 {y - 50} {hx - 30} {y - 50} {hx} {y - 4}" fill="none" stroke="{AMBER}" '
                      f'stroke-width="2.5" marker-end="url(#arrow)"/>')
            for a, b in zip(free, free[1:]):
                ax, bx = x0 + a * w + (w - 8) / 2, x0 + b * w + (w - 8) / 2
                g.add(f'<path d="M{ax} {y + 60}C {ax} {y + 100} {bx} {y + 100} {bx} {y + 62}" fill="none" '
                      f'stroke="{DIM}" stroke-width="2" marker-end="url(#arrow)"/>')
            g.text(450, 300, caption, 14, TEXT, 'middle', '600')
        return part(fn)
    s.add(show([draw(u, f, c) for c, u, f in states], 10))
    return s.render()


# ---------------------------------------------------------------- C++ container layouts
def cpp_containers():
    s = Svg(900, 470, 'Layouts of std::vector, std::string, std::list and a std::map node in 64-bit MSVC release builds')
    s.heading('C++ containers in memory (64-bit MSVC, release)',
              'The same few shapes appear inside every structure that holds a container.')

    def px(field):
        return field[2] if len(field) > 2 else field[1] * 15

    def cells(x, y, fields, colour=BLUE):
        for field in fields:
            name, cw = field[0], px(field)
            s.rect(x, y, cw - 3, 30, colour, rx=3, extra=' fill-opacity="0.22"')
            s.text(x + (cw - 3) / 2, y + 19, name, 11, TEXT, 'middle', mono=True)
            x += cw
        return x

    def offs(x, y, fields):
        o = 0
        for field in fields:
            s.text(x + 2, y - 4, f'+{o:#x}' if o else '+0', 9, DIM, mono=True)
            x += px(field)
            o += field[1]

    y = 96
    s.text(24, y + 19, 'vector<T>', 13, TEAL, weight='600', mono=True)
    f = [('first', 8), ('last', 8), ('end', 8)]
    offs(170, y, f)
    end = cells(170, y, f)
    s.text(end + 10, y + 19, 'count = (last - first) / sizeof(T)', 11, DIM, mono=True)
    s.rect(560, y + 38, 300, 22, GREEN, rx=3, extra=' fill-opacity="0.2"')
    s.rect(560, y + 38, 200, 22, GREEN, rx=3, extra=' fill-opacity="0.45"')
    s.text(660, y + 53, 'elements', 10, TEXT, 'middle')
    s.text(810, y + 53, 'spare', 10, DIM, 'middle')
    for x1, x2 in ((185, 560), (305, 760), (425, 860)):
        s.add(f'<path d="M{x1} {y + 30}C {x1} {y + 49} {x2 - 40} {y + 49} {x2} {y + 40}" fill="none" stroke="{DIM}" '
              f'stroke-width="1.2" marker-end="url(#arrow)"/>')

    y = 196
    s.text(24, y + 19, 'string', 13, TEAL, weight='600', mono=True)
    f = [('"short text" (15 or fewer)', 16), ('size', 8), ('capacity', 8)]
    offs(170, y, f)
    end = cells(170, y, f)
    s.text(end + 10, y + 14, 'capacity <= 15: the text is inline', 11, DIM)
    s.text(end + 10, y + 30, 'capacity > 15: +0 is a pointer to it', 11, DIM)

    y = 276
    s.text(24, y + 19, 'list<T>', 13, TEAL, weight='600', mono=True)
    f = [('head', 8), ('size', 8)]
    offs(170, y, f)
    cells(170, y, f)
    nf = [('next', 8, 64), ('prev', 8, 64), ('value', 8, 64)]
    for k in range(2):
        nx = 450 + k * 220
        offs(nx, y, nf)
        cells(nx, y, nf, PURPLE)
        s.text(nx + 96, y + 46, 'sentinel node' if k == 0 else 'node', 10, DIM, 'middle')
    s.add(f'<path d="M200 {y + 30}C 220 {y + 60} 420 {y + 60} 450 {y + 24}" fill="none" stroke="{DIM}" '
          f'stroke-width="1.2" marker-end="url(#arrow)"/>')
    s.add(f'<path d="M482 {y}C 500 {y - 22} 650 {y - 22} 670 {y + 4}" fill="none" stroke="{DIM}" stroke-width="1.2" '
          f'marker-end="url(#arrow)"/>')

    y = 376
    s.text(24, y + 19, 'map node', 13, TEAL, weight='600', mono=True)
    f = [('left', 8, 104), ('parent', 8, 104), ('right', 8, 104), ('colour', 1, 60), ('nil', 1, 44), ('(pad)', 6, 60),
         ('key, value', 12, 180)]
    offs(170, y, f)
    cells(170, y, f, PURPLE)
    s.text(170, y + 54, 'std::map and std::set hold a pointer to a head node and the count. A colour byte next to three '
                        'pointers is a red-black tree.', 11, DIM)
    s.text(24, 456, 'Debug builds add a pointer at +0 of each container. libstdc++ differs: docs/TYPES.md has both.', 11, DIM)
    return s.render()


RUNTIME = {
    'ram-compare.svg': ram_compare, 'skeleton-parents.svg': skeleton_parents, 'handle-bits.svg': handle_bits,
    'pool-freelist.svg': pool_freelist, 'cpp-containers.svg': cpp_containers,
}
