"""Diagrams for the hardware guides (docs/hardware/): the PC video standards, Mode 13h addresses,
the VGA palette, the vertical retrace, a text-mode cell, pixel formats, floating-point formats,
rounding, self-modifying code, Sound Blaster DMA, MIDI messages, the DOS memory map, a game with
several renderers, and the forms shaders ship in. site/diagrams/make.py writes them.
"""
from svg import AMBER, BG, BLUE, DIM, GREEN, GREY, PURPLE, RED, TEAL, TEXT, Svg

PINK = '#f472b6'
SANS = 'ui-sans-serif, Helvetica, Arial, sans-serif'

# The 16 colours of CGA text and EGA, as the VGA shows them by default.
RGBI = ['#000000', '#0000aa', '#00aa00', '#00aaaa', '#aa0000', '#aa00aa', '#aa5500', '#aaaaaa',
        '#555555', '#5555ff', '#55ff55', '#55ffff', '#ff5555', '#ff55ff', '#ffff55', '#ffffff']


def bits(s, x, y, fields, unit, h=30):
    """A row of bit fields, left to right: (label, width in bits, colour). Returns the end x."""
    for label, n, colour in fields:
        w = n * unit
        s.rect(x, y, w - 2, h, colour, rx=3, extra=' fill-opacity="0.3"')
        s.text(x + w / 2 - 1, y + h / 2 + 4, label, 11, TEXT, 'middle', '600')
        x += w
    return x


# ---------------------------------------------------------------- the video standards
def video_standards():
    s = Svg(900, 400, 'The PC video standards drawn to scale by resolution, with the colours each could show')
    s.heading('PC video modes, to scale', 'Each box is a mode\'s resolution, to scale. Below it, the colours it shows at once.')
    modes = [('CGA', '320 x 200', 320, 200, 'CGA'), ('EGA', '640 x 350', 640, 350, 'EGA'),
             ('VGA Mode 13h', '320 x 200', 320, 200, 'VGA'), ('Mode X', '320 x 240', 320, 240, 'VGA'),
             ('VESA', '640 x 480', 640, 480, 'VGA')]
    x = 24
    for name, res, w, h, kind in modes:
        w2, h2 = w / 2.8, h / 2.8
        y = 92
        s.rect(x, y, w2, h2, BLUE, BLUE, rx=2, extra=' fill-opacity="0.15" stroke-opacity="0.6"')
        s.text(x + 6, y + 16, name, 12, TEXT, weight='600')
        s.text(x + 6, y + 32, res, 11, DIM, mono=True)
        sy = y + 220
        if kind == 'CGA':
            cols, label = ['#000000', '#55ffff', '#ff55ff', '#ffffff'], '4 colours, fixed palettes'
        elif kind == 'EGA':
            cols, label = RGBI, '16 colours from 64'
        else:
            cols, label = [f'hsl({i * 360 // 32}, 70%, {35 + (i % 4) * 10}%)' for i in range(32)], '256 from 262,144'
        cw = min(14, (w2 - 4) / len(cols)) if kind != 'CGA' else 14
        for i, c in enumerate(cols):
            s.rect(x + i * cw, sy, cw - 1, 14, c, rx=1)
        s.text(x, sy + 32, label, 11, DIM)
        x += w2 + 12
    s.text(24, 384, 'Most DOS games from 1990 to 1996 use Mode 13h or Mode X. Later games use VESA modes.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- Mode 13h addresses
def mode13h_address():
    s = Svg(900, 330, 'Mode 13h: one byte per pixel, 320 bytes per line, so the pixel at x, y is at offset y times 320 plus x')
    s.heading('Mode 13h: a pixel is a byte', 'Offset = y x 320 + x, from A000:0000. The byte is an index into the palette.')
    x0, y0, c = 40, 90, 22
    for yy in range(6):
        for xx in range(14):
            hit = (xx, yy) == (5, 2)
            s.rect(x0 + xx * c, y0 + yy * c, c - 2, c - 2, AMBER if hit else BLUE, rx=2,
                   extra=f' fill-opacity="{0.9 if hit else 0.18}"')
    for xx in range(0, 14, 2):
        s.text(x0 + xx * c + c / 2 - 1, y0 - 6, str(xx), 10, DIM, 'middle', mono=True)
    for yy in range(6):
        s.text(x0 - 8, y0 + yy * c + 15, str(yy), 10, DIM, 'end', mono=True)
    s.text(x0, y0 + 6 * c + 18, 'the top-left corner of the screen (x 0 to 13, y 0 to 5)', 11, DIM)
    s.text(400, 104, 'pixel (5, 2)', 14, AMBER, weight='600')
    s.text(400, 132, '2 x 320 + 5 = 645 = 0x285', 14, TEXT, mono=True)
    s.text(400, 156, 'A000:0285 holds its palette index', 13, DIM, mono=True)
    # The memory as one line of bytes.
    my = 220
    s.text(400, my + 50, 'video memory, one byte after another', 11, DIM)
    labels = ['...', '0x283', '0x284', '0x285', '0x286', '0x287', '...']
    for i, lab in enumerate(labels):
        hit = lab == '0x285'
        s.rect(400 + i * 66, my, 62, 30, AMBER if hit else GREY, rx=3, extra=f' fill-opacity="{0.9 if hit else 0.22}"')
        s.text(400 + i * 66 + 31, my + 20, lab, 11, BG if hit else DIM, 'middle', mono=True)
    s.add(f'<path d="M{x0 + 5 * c + 10} {y0 + 2 * c + 10}C 300 {y0 + 140} 520 {my - 40} {400 + 3 * 66 + 31} {my - 2}" '
          f'fill="none" stroke="{AMBER}" stroke-width="2" marker-end="url(#arrow)"/>')
    s.text(40, 306, '64,000 bytes in all. Many games draw into a buffer in normal memory and copy it here once a frame.',
           12, DIM)
    return s.render()


# ---------------------------------------------------------------- the VGA palette (animated colour cycling)
def vga_palette():
    s = Svg(900, 380, 'The VGA palette: a pixel holds an index, the palette maps it to 6-bit red, green and blue; '
                      'two common 6-to-8-bit conversions differ, and colour cycling changes the palette, not the pixels')
    s.heading('The VGA palette', 'Pixels hold an index. The palette gives each index 6 bits of red, green and blue (0 to 63).')
    s.rect(40, 90, 120, 44, GREY, rx=4, extra=' fill-opacity="0.25"')
    s.text(100, 110, 'pixel byte', 11, DIM, 'middle')
    s.text(100, 126, '0x2A = 42', 13, TEXT, 'middle', '600', mono=True)
    s.line(164, 112, 206, 112, DIM, 1.5, ' marker-end="url(#arrow)"')
    s.rect(210, 90, 210, 44, PURPLE, rx=4, extra=' fill-opacity="0.22"')
    s.text(315, 110, 'palette[42]', 11, DIM, 'middle', mono=True)
    s.text(315, 126, 'R 12  G 40  B 63', 13, TEXT, 'middle', '600', mono=True)
    s.line(424, 112, 466, 112, DIM, 1.5, ' marker-end="url(#arrow)"')
    s.rect(470, 90, 44, 44, '#3ea1ff', rx=4)
    s.text(530, 108, 'Written through port 3C8h (the index)', 11, DIM)
    s.text(530, 124, 'and 3C9h (red, green, blue in turn).', 11, DIM)
    # The two conversions.
    a = [v * 255 // 63 for v in range(64)]
    b = [(v << 2) | (v >> 4) for v in range(64)]
    diff = [v for v in range(64) if a[v] != b[v]]
    y = 170
    s.text(40, y, f'6 bits to 8 bits: two formulas, different on {len(diff)} of 64 values', 13, TEXT, weight='600')
    s.text(40, y + 22, 'v x 255 / 63', 12, TEAL, mono=True)
    s.text(40, y + 44, '(v << 2) | (v >> 4)', 12, AMBER, mono=True)
    for v in range(64):
        x = 210 + v * 10
        s.rect(x, y + 10, 9, 16, f'#{a[v]:02x}{a[v]:02x}{a[v]:02x}', rx=1)
        s.rect(x, y + 32, 9, 16, f'#{b[v]:02x}{b[v]:02x}{b[v]:02x}', rx=1)
        if a[v] != b[v]:
            s.rect(x + 2, y + 52, 5, 5, RED, rx=1)
    s.text(210, y + 74, f'red marks: v = 33 gives {a[33]} by the first and {b[33]} by the second. Compare with captures of '
                        'the original.', 11, DIM)
    # Colour cycling: the same pixels, a rotating palette.
    y = 280
    s.text(40, y, 'Colour cycling: the pixels stay, the palette entries rotate', 13, TEXT, weight='600')
    blues = ['#0b2a6b', '#12408f', '#1b5ab5', '#2a78d6', '#46a0f0', '#7cc4ff', '#46a0f0', '#2a78d6']
    for i in range(24):
        x = 40 + i * 34
        vals = ';'.join(blues[(i + k) % 8] for k in range(8)) + ';' + blues[i % 8]
        s.add(f'<rect x="{x}" y="{y + 12}" width="32" height="40" rx="2" fill="{blues[i % 8]}">'
              f'<animate attributeName="fill" values="{vals}" dur="1.6s" calcMode="discrete" repeatCount="indefinite"/>'
              f'</rect>')
    s.text(40, y + 74, 'A capture of video memory alone misses fades, flashes and cycling. Capture the palette every frame.',
           11, DIM)
    return s.render()


# ---------------------------------------------------------------- the vertical retrace (animated)
def vga_retrace():
    s = Svg(900, 340, 'The VGA beam draws the visible lines, then the vertical retrace returns it to the top; port 3DAh '
                      'bit 3 is set during the retrace, and games wait for it')
    s.heading('The vertical retrace', 'The beam draws 400 visible lines, then returns to the top. Mode 13h does this 70 times a second.')
    sx, sy, sw, sh = 40, 84, 300, 200
    s.rect(sx, sy, sw, sh, BLUE, BLUE, rx=3, extra=' fill-opacity="0.12" stroke-opacity="0.5"')
    s.rect(sx, sy + sh, sw, 30, GREY, rx=3, extra=' fill-opacity="0.25"')
    s.text(sx + sw / 2, sy + sh + 20, 'blanking and retrace', 11, DIM, 'middle')
    dur = 2.0
    # The beam: down the visible area, then through the retrace band, then back to the top.
    s.add(f'<rect x="{sx}" y="{sy}" width="{sw}" height="3" fill="{AMBER}"><animate attributeName="y" '
          f'values="{sy};{sy + sh};{sy + sh + 27};{sy}" keyTimes="0;0.85;0.99;1" dur="{dur}s" repeatCount="indefinite"/></rect>')
    # The signal of bit 3 over the same time.
    gx, gy, gw = 400, 110, 460
    s.text(gx, gy - 16, 'port 3DAh bit 3 (vertical retrace)', 12, TEXT, weight='600')
    pts = f'M{gx} {gy + 40}H{gx + gw * 0.85 + 6}V{gy}H{gx + gw * 0.99}V{gy + 40}H{gx + gw}'
    s.add(f'<path d="{pts}" fill="none" stroke="{TEAL}" stroke-width="2"/>')
    s.text(gx + gw * 0.88, gy - 4, 'set', 11, TEAL)
    s.add(f'<rect x="{gx}" y="{gy - 6}" width="2" height="54" fill="{AMBER}"><animate attributeName="x" '
          f'values="{gx};{gx + gw}" dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.text(gx, gy + 70, 'A game waits here: read the port until bit 3 is set, then', 12, DIM)
    s.text(gx, gy + 88, 'flip pages or write the palette, with no tearing.', 12, DIM)
    s.text(gx, gy + 122, 'If the game waits every frame, the refresh rate is its speed:', 12, TEXT)
    s.text(gx, gy + 140, '70 Hz in Mode 13h, 60 Hz in Mode X 320 x 240.', 12, TEXT)
    s.text(gx, gy + 166, 'Bit 0 is set during either blanking, for raster effects.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- a text-mode cell
def text_cell():
    s = Svg(900, 330, 'A text-mode cell: two bytes at B800, the character code and the attribute, whose bits give '
                      'blink, background colour, bright and foreground colour')
    s.heading('A text-mode cell: two bytes', 'Mode 3 is 80 x 25 cells at B800:0000. Even bytes are characters, odd bytes are attributes.')
    s.text(40, 100, 'B800:0000', 11, DIM, mono=True)
    s.rect(40, 108, 110, 40, GREY, rx=3, extra=' fill-opacity="0.25"')
    s.text(95, 133, "0x41 'A'", 13, TEXT, 'middle', '600', mono=True)
    s.text(160, 100, 'B800:0001', 11, DIM, mono=True)
    s.rect(160, 108, 110, 40, AMBER, rx=3, extra=' fill-opacity="0.3"')
    s.text(215, 133, '0x1E', 13, TEXT, 'middle', '600', mono=True)
    s.line(215, 150, 330, 218, DIM, 1.5, ' marker-end="url(#arrow)"')
    end = bits(s, 340, 206, [('blink', 1, RED), ('background', 3, BLUE), ('bright', 1, AMBER), ('foreground', 3, TEAL)], 56)
    for k, b in enumerate('00011110'):
        s.text(340 + k * 56 + 27, 258, b, 13, TEXT, 'middle', '600', mono=True)
        s.text(340 + k * 56 + 27, 198, str(7 - k), 10, DIM, 'middle', mono=True)
    s.text(340, 282, 'background 1 (blue), bright + foreground 6 = 14 (yellow)', 11, DIM)
    # The cell as it shows.
    s.rect(560, 84, 64, 80, RGBI[1], rx=2)
    s.text(592, 142, 'A', 54, RGBI[14], 'middle', '700', mono=True)
    s.text(640, 104, 'as drawn: the', 11, DIM)
    s.text(640, 120, 'card\'s 8 x 16 font,', 11, DIM)
    s.text(640, 136, 'code page 437,', 11, DIM)
    s.text(640, 152, 'the 16 colours below', 11, DIM)
    for i, c in enumerate(RGBI):
        s.rect(640 + i * 14, 160, 13, 13, c, rx=1)
    s.text(340, 304, 'Bit 7 may mean a bright background instead, if the game turns blinking off.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- pitch and 16-bit pixel formats
def pixel_formats():
    s = Svg(900, 360, 'A DirectDraw surface whose pitch is wider than its row of pixels, and the 5-6-5 and 5-5-5 '
                      '16-bit pixel formats')
    s.heading('Surfaces: pitch and pixel formats', 'Lock gives a pointer and a pitch. The next row starts a pitch later, not a width later.')
    x0, y0 = 40, 90
    for r in range(5):
        s.rect(x0, y0 + r * 26, 300, 22, TEAL, rx=2, extra=' fill-opacity="0.3"')
        s.rect(x0 + 302, y0 + r * 26, 80, 22, GREY, rx=2, extra=' fill-opacity="0.25"')
    s.text(x0 + 150, y0 + 15, 'pixels of row 0 (width x bytes per pixel)', 11, TEXT, 'middle')
    s.text(x0 + 342, y0 + 15, 'padding', 10, DIM, 'middle')
    s.line(x0, y0 + 140, x0 + 382, y0 + 140, AMBER, 2)
    s.text(x0 + 191, y0 + 158, 'pitch: the bytes from one row to the next', 12, AMBER, 'middle')
    s.text(x0, y0 + 186, 'row y starts at pointer + y x pitch', 12, TEXT, mono=True)
    s.text(x0, y0 + 206, 'A port must never assume pitch = width x bytes.', 11, DIM)
    gx = 470
    s.text(gx, 100, '16-bit 5-6-5', 13, TEXT, weight='600')
    bits(s, gx, 110, [('red', 5, RED), ('green', 6, GREEN), ('blue', 5, BLUE)], 24)
    s.text(gx, 170, '16-bit 5-5-5', 13, TEXT, weight='600')
    bits(s, gx, 180, [('x', 1, GREY), ('red', 5, RED), ('green', 5, GREEN), ('blue', 5, BLUE)], 24)
    s.text(gx, 240, 'bit 15 on the left, bit 0 on the right', 11, DIM)
    s.text(gx, 266, 'The game reads the pixel format from the surface', 12, DIM)
    s.text(gx, 284, 'and may have a code path for each. Port the one', 12, DIM)
    s.text(gx, 302, 'the reference captures used, or both.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- floating-point formats
def float_formats():
    s = Svg(900, 330, 'The x87 80-bit extended format beside the 64-bit double and 32-bit float formats, '
                      'and rounding when a value is stored to memory')
    s.heading('x87 registers keep more bits than memory', 'A value is rounded only when it is stored. Widths are to scale.')
    unit = 10.5
    rows = [('80-bit extended (x87 registers)', [('s', 1, RED), ('exponent 15', 15, AMBER), ('i', 1, PURPLE),
                                                 ('fraction 63', 63, BLUE)]),
            ('64-bit double (memory)', [('s', 1, RED), ('exponent 11', 11, AMBER), ('fraction 52', 52, BLUE)]),
            ('32-bit float (memory)', [('s', 1, RED), ('exponent 8', 8, AMBER), ('fraction 23', 23, BLUE)])]
    y = 96
    for label, fields in rows:
        s.text(24, y - 6, label, 12, TEXT, weight='600')
        bits(s, 24, y, fields, unit, 28)
        y += 66
    s.text(24, 300, 'i: the integer bit, explicit only in the 80-bit format. FST and FSTP round to the size of the memory '
                    'operand. The precision', 11, DIM)
    s.text(24, 316, 'control word can make every operation round to 24 or 53 bits. Direct3D sets 24 bits unless the game '
                    'asks it not to.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- rounding
def rounding_table():
    s = Svg(900, 390, 'Float to integer conversion and signed division by two, rounded three ways: toward zero, '
                      'to nearest even, and toward minus infinity')
    s.heading('One value, three roundings', 'The same source can compile to any of these. Each differs from the others somewhere.')
    xs = [-2.5, -1.5, -1.0, -0.5, 0.5, 1.0, 1.5, 2.5]

    def trunc(v):
        return int(v)

    def even(v):
        return round(v)

    def floor(v):
        import math
        return math.floor(v)
    cols = [('x', None, DIM), ('C cast, _ftol: toward zero', trunc, TEAL), ('FISTP, 1.5 x 2^52 trick: nearest even', even, AMBER),
            ('floor: toward minus infinity', floor, PURPLE)]
    y0, x0 = 100, 24
    for r, (label, fn, colour) in enumerate(cols):
        y = y0 + r * 34
        s.text(x0, y + 18, label, 12, colour if fn else TEXT, weight='600')
        for k, v in enumerate(xs):
            val = v if fn is None else fn(v)
            base = trunc(v)
            odd = fn is not None and val != base
            s.rect(330 + k * 66, y, 60, 26, colour if fn else GREY, rx=3, extra=f' fill-opacity="{0.55 if odd else 0.18}"')
            s.text(330 + k * 66 + 30, y + 18, f'{val:g}', 12, TEXT, 'middle', '600', mono=True)
    y = y0 + 4 * 34 + 26
    s.text(x0, y, 'Signed division by 2', 13, TEXT, weight='600')
    ns = [-7, -5, -3, -1, 1, 3, 5, 7]
    for r, (label, fn, colour) in enumerate([('n', None, DIM), ('IDIV: toward zero', lambda n: int(n / 2), TEAL),
                                             ('SAR n, 1: toward minus infinity', lambda n: n >> 1, PURPLE)]):
        yy = y + 12 + r * 30
        s.text(x0, yy + 17, label, 12, colour if fn else TEXT, weight='600')
        for k, n in enumerate(ns):
            val = n if fn is None else fn(n)
            odd = fn is not None and val != int(n / 2)
            s.rect(330 + k * 66, yy, 60, 24, colour if fn else GREY, rx=3, extra=f' fill-opacity="{0.55 if odd else 0.18}"')
            s.text(330 + k * 66 + 30, yy + 17, str(val), 12, TEXT, 'middle', '600', mono=True)
    return s.render()


# ---------------------------------------------------------------- self-modifying code (animated)
def self_modifying():
    s = Svg(900, 300, 'Self-modifying code: before an inner loop runs, the game writes a value into the immediate '
                      'operand of an instruction in the loop')
    s.heading('Code that changes itself', 'Before the loop runs, the game writes the texture width into the loop\'s own instruction.')
    s.text(40, 98, 'the loop as Ghidra shows it', 11, DIM)
    s.text(40, 124, 'add esi, 0x12345678', 15, TEXT, mono=True)
    s.text(40, 150, '81 C6 78 56 34 12', 13, DIM, mono=True)
    s.text(40, 192, 'the setup code', 11, DIM)
    s.text(40, 218, 'mov dword [loop + 2], ecx    ; ecx = width', 14, AMBER, mono=True)
    bx, by = 470, 104
    s.text(bx, by - 10, 'the instruction\'s bytes in memory', 11, DIM)
    before = ['81', 'C6', '78', '56', '34', '12']
    after = ['81', 'C6', '40', '01', '00', '00']
    for i in range(6):
        x = bx + i * 62
        imm = i >= 2
        s.rect(x, by, 56, 40, AMBER if imm else GREY, rx=3, extra=f' fill-opacity="{0.3 if imm else 0.2}"')
        if imm:
            s.add(f'<text x="{x + 28}" y="{by + 26}" font-family="ui-monospace, Menlo, monospace" font-size="15" '
                  f'font-weight="600" fill="{TEXT}" text-anchor="middle">{before[i]}<animate attributeName="opacity" '
                  f'values="1;1;0;0" keyTimes="0;0.4;0.45;1" dur="5s" repeatCount="indefinite"/></text>')
            s.add(f'<text x="{x + 28}" y="{by + 26}" font-family="ui-monospace, Menlo, monospace" font-size="15" '
                  f'font-weight="600" fill="{AMBER}" text-anchor="middle" opacity="0">{after[i]}<animate '
                  f'attributeName="opacity" values="0;0;1;1" keyTimes="0;0.4;0.45;1" dur="5s" repeatCount="indefinite"/></text>')
        else:
            s.text(x + 28, by + 26, before[i], 15, DIM, 'middle', '600', mono=True)
    s.text(bx, by + 64, 'opcode and register', 11, DIM)
    s.text(bx + 124 + 120, by + 64, 'the 32-bit immediate', 11, AMBER, 'middle')
    s.add(f'<g opacity="0"><text x="{bx}" y="{by + 100}" font-family="ui-monospace, Menlo, monospace" font-size="15" '
          f'fill="{TEXT}">add esi, 0x140   ; 320</text><animate attributeName="opacity" values="0;0;1;1" '
          f'keyTimes="0;0.45;0.5;1" dur="5s" repeatCount="indefinite"/></g>')
    s.text(40, 280, 'Find stores whose target is a code address. In the port, the patched value is an ordinary variable.',
           12, DIM)
    return s.render()


# ---------------------------------------------------------------- Sound Blaster DMA (animated)
def sb_dma():
    s = Svg(900, 330, 'Sound Blaster auto-initialised DMA: the card plays one half of a buffer while the game fills the '
                      'other, with an interrupt at the end of each half')
    s.heading('Sound Blaster: double buffering with DMA', 'The card reads one half while the game fills the other. An interrupt marks each half\'s end.')
    x0, y0, w = 60, 110, 760
    s.rect(x0, y0, w / 2 - 2, 60, TEAL, rx=4, extra=' fill-opacity="0.18"')
    s.rect(x0 + w / 2 + 2, y0, w / 2 - 2, 60, BLUE, rx=4, extra=' fill-opacity="0.18"')
    s.text(x0 + w / 4, y0 - 10, 'half A', 12, TEXT, 'middle', '600')
    s.text(x0 + 3 * w / 4, y0 - 10, 'half B', 12, TEXT, 'middle', '600')
    dur = 4
    s.add(f'<rect x="{x0}" y="{y0 - 4}" width="3" height="68" fill="{AMBER}"><animate attributeName="x" '
          f'values="{x0};{x0 + w}" dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.text(x0, y0 + 90, 'amber: the DMA controller reading samples to the card, at the sample rate', 11, AMBER)
    for half, (label, beg) in enumerate((('IRQ: half A played. The game refills A while B plays', 0.5),
                                         ('IRQ: half B played. The game refills B while A plays', 0.0))):
        cx = x0 + (w / 2 if half == 0 else w) - 2
        s.add(f'<g opacity="0"><circle cx="{cx}" cy="{y0 + 30}" r="12" fill="{RED}"/>'
              f'<text x="{x0 + w / 2}" y="{y0 + 130}" font-family="{SANS}" font-size="13" font-weight="600" fill="{RED}" '
              f'text-anchor="middle">{label}</text>'
              f'<animate attributeName="opacity" values="0;1;1;0;0" keyTimes="0;0.01;0.3;0.31;1" dur="{dur}s" '
              f'begin="{beg * dur}s" repeatCount="indefinite"/></g>')
    s.text(60, 288, 'The block size set on the card is half the buffer. Its length, and the sample rate, set the delay between '
                    'a sound', 11, DIM)
    s.text(60, 304, 'starting in the game and reaching the speaker. Record them: a port\'s mixer is free, but the timing of '
                    'events is not.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- MIDI messages
def midi_messages():
    s = Svg(900, 330, 'MIDI channel messages: a status byte with the message kind and channel, then one or two data bytes; '
                      'running status leaves out repeated status bytes')
    s.heading('MIDI messages', 'A status byte (high bit set) gives the kind and the channel. Data bytes are 0 to 127.')
    msgs = [('Note on, channel 1', ['90', '3C', '64'], ['status 9n', 'key 60 (C4)', 'velocity 100']),
            ('Note off', ['80', '3C', '40'], ['status 8n', 'key', 'velocity']),
            ('Program change', ['C0', '13'], ['status Cn', 'instrument 19']),
            ('Controller', ['B0', '07', '50'], ['status Bn', 'volume (7)', 'value 80'])]
    y = 92
    for name, bs, labels in msgs:
        s.text(40, y + 20, name, 12, TEXT, weight='600')
        for k, (b, lab) in enumerate(zip(bs, labels)):
            x = 230 + k * 140
            st = k == 0
            s.rect(x, y, 130, 30, AMBER if st else BLUE, rx=3, extra=' fill-opacity="0.28"')
            s.text(x + 30, y + 20, b, 13, TEXT, 'middle', '600', mono=True)
            s.text(x + 54, y + 20, lab, 10, DIM)
        y += 42
    s.text(40, y + 20, 'Running status', 12, TEXT, weight='600')
    for k, (b, st) in enumerate([('90', 1), ('3C', 0), ('64', 0), ('40', 0), ('64', 0), ('43', 0), ('00', 0)]):
        x = 230 + k * 70
        s.rect(x, y, 64, 30, AMBER if st else BLUE, rx=3, extra=' fill-opacity="0.28"')
        s.text(x + 32, y + 20, b, 13, TEXT, 'middle', '600', mono=True)
    s.text(230, y + 50, 'three notes on with one status byte. Velocity 0 (the last) is a note off.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- the DOS memory map
def dos_memory():
    s = Svg(900, 480, 'The memory map of a DOS PC: conventional memory to 640 KB, video memory and ROMs in the upper '
                      'area, the EMS page frame, the high memory area, and extended memory above 1 MB')
    s.heading('Memory in a DOS PC', 'Real-mode programs see the first megabyte. Extended memory needs XMS, EMS or a DOS extender.')
    regions = [
        ('00000', 'interrupt vectors, BIOS data, DOS, drivers', 20, GREY),
        ('', 'conventional memory: the program, its data and stack', 120, TEAL),
        ('A0000', 'video memory: A000 graphics, B000 mono text, B800 colour text', 40, AMBER),
        ('C0000', 'video BIOS', 14, GREY),
        ('C8000', 'upper memory blocks; the EMS page frame (64 KB, often at D000 or E000)', 34, PURPLE),
        ('F0000', 'system BIOS', 14, GREY),
        ('100000', 'high memory area: 64 KB minus 16 bytes, reached from real mode', 22, BLUE),
        ('10FFF0', 'extended memory: XMS blocks, or a DOS extender\'s flat 32-bit memory', 110, GREEN),
    ]
    x, y = 120, 84
    for addr, label, h, colour in regions:
        s.rect(x, y, 260, h - 2, colour, rx=2, extra=' fill-opacity="0.3"')
        if addr:
            s.text(x - 8, y + 11, addr, 11, DIM, 'end', mono=True)
        if addr in ('A0000', '100000'):
            s.text(24, y + 11, '640 KB' if addr == 'A0000' else '1 MB', 11, AMBER, weight='600')
        s.text(x + 272, y + h / 2 + 4, label, 12, TEXT)
        y += h
    s.text(24, 464, 'Not to scale. A game that needs more than 640 KB uses EMS (pages swapped into the frame), XMS (copies), '
                    'or a DOS extender (DOS/4GW).', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- several renderers
def renderer_paths():
    s = Svg(900, 330, 'One game with several renderers: the game code calls a renderer interface implemented for Glide, '
                      'Direct3D, OpenGL and software; each looks slightly different, and the port chooses one reference')
    s.heading('One game, several renderers', 'The rules are shared. The pictures differ: pick one renderer as the reference.')
    s.rect(330, 82, 240, 44, TEAL, rx=6, extra=' fill-opacity="0.22"')
    s.text(450, 109, 'game code', 14, TEXT, 'middle', '600')
    s.rect(330, 150, 240, 40, GREY, rx=6, extra=' fill-opacity="0.25"')
    s.text(450, 175, 'renderer interface (a vtable)', 12, TEXT, 'middle')
    s.line(450, 126, 450, 148, DIM, 1.5, ' marker-end="url(#arrow)"')
    rs = [('Glide (3dfx)', '16-bit colour with dithering, the card\'s own fog table'), ('Direct3D', 'fog and filtering vary by '
          'card and driver'), ('OpenGL', 'often the MiniGL subset Quake used'), ('software', 'exact and slow; often 8-bit '
          'with lookup tables')]
    for k, (name, note) in enumerate(rs):
        x = 30 + k * 214
        s.rect(x, 230, 200, 40, BLUE if k < 3 else PURPLE, rx=6, extra=' fill-opacity="0.22"')
        s.text(x + 100, 255, name, 13, TEXT, 'middle', '600')
        s.line(450, 190, x + 100, 228, DIM, 1.2, ' marker-end="url(#arrow)"')
        line, n = '', 0
        for word in note.split() + ['']:
            if word and len(line) + len(word) < 34:
                line = (line + ' ' + word).strip()
                continue
            s.text(x + 100, 290 + n * 14, line, 10, DIM, 'middle')
            line, n = word, n + 1
    return s.render()


# ---------------------------------------------------------------- the forms shaders ship in
def shader_forms():
    s = Svg(900, 330, 'The forms in which games shipped shaders, by year: fixed-function state, Direct3D 8 and 9 '
                      'bytecode, ARB assembly, GLSL, DXBC, DXIL and SPIR-V')
    s.heading('How shaders ship, by year', 'Each bar runs from the form\'s first games to about when new games stopped using it.')
    lo, hi, x0, x1 = 1996, 2024, 200, 870
    X = lambda yr: x0 + (yr - lo) / (hi - lo) * (x1 - x0)
    for yr in range(1996, 2025, 4):
        s.line(X(yr), 80, X(yr), 296, '#1f3d40', 1)
        s.text(X(yr), 312, str(yr), 10, DIM, 'middle')
    forms = [('fixed-function state', 1996, 2008, GREY), ('Direct3D 8/9 bytecode', 2000, 2014, AMBER),
             ('ARB assembly (OpenGL)', 2002, 2008, PURPLE), ('GLSL source', 2004, 2024, GREEN),
             ('DXBC (Direct3D 10, 11)', 2006, 2024, BLUE), ('DXIL (Direct3D 12)', 2016, 2024, TEAL),
             ('SPIR-V (Vulkan)', 2016, 2024, PINK)]
    for k, (name, a, b, colour) in enumerate(forms):
        y = 86 + k * 30
        s.text(24, y + 15, name, 12, TEXT)
        s.rect(X(a), y, X(b) - X(a), 20, colour, rx=4, extra=' fill-opacity="0.55"')
    return s.render()


DIAGRAMS = {
    'video-standards.svg': video_standards, 'mode13h-address.svg': mode13h_address, 'vga-palette.svg': vga_palette,
    'vga-retrace.svg': vga_retrace, 'text-cell.svg': text_cell, 'pixel-formats.svg': pixel_formats,
    'float-formats.svg': float_formats, 'rounding-table.svg': rounding_table, 'self-modifying.svg': self_modifying,
    'sb-dma.svg': sb_dma, 'midi-messages.svg': midi_messages, 'dos-memory.svg': dos_memory,
    'renderer-paths.svg': renderer_paths, 'shader-forms.svg': shader_forms,
}
