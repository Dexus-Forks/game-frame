"""Diagrams for the console and handheld guides (docs/consoles/, docs/handhelds/): the frame loop,
bank switching, the video and maths chips, the CPUs that share a game, containers and keys, the
routes to a port, the tools, and the handhelds' memory maps, screens, saves and executables.
site/diagrams/make.py writes them with the other sets.
"""
import re

from svg import AMBER, BG, BLUE, DIM, FONT, GREEN, GREY, MONO, PURPLE, RED, TEAL, TEXT, Svg, esc

PINK = '#f472b6'


# ---------------------------------------------------------------- helpers
def para(s, x, y, text, width, size=12, fill=DIM, lh=None, weight='400', anchor='start'):
    """TEXT wrapped to about WIDTH pixels from (x, y). Returns the y below the last line."""
    lh = lh or size + 5
    per = max(8, int(width / (size * 0.54)))
    out, line = [], ''
    for w in text.split():
        if line and len(line) + 1 + len(w) > per:
            out.append(line)
            line = w
        else:
            line = f'{line} {w}' if line else w
    if line:
        out.append(line)
    for i, t in enumerate(out):
        s.text(x, y + i * lh, t, size, fill, anchor, weight)
    return y + len(out) * lh


def box(s, x, y, w, h, colour, title='', lines=(), size=13, op=0.2, mono=False):
    """A tinted box with a centred title and smaller lines under it."""
    s.rect(x, y, w, h, colour, colour, rx=5, extra=f' fill-opacity="{op}" stroke-opacity="0.65"')
    lines = [lines] if isinstance(lines, str) else list(lines)
    if not title and not lines:
        return
    block = (size + 3 if title else 0) + 15 * len(lines) - (0 if lines else 3)
    ty = y + (h - block) / 2 + (size - 2 if title else 10)
    if title:
        s.text(x + w / 2, ty, title, size, TEXT, 'middle', '600')
        ty += size + 5
    for t in lines:
        s.text(x + w / 2, ty, t, 11, DIM, 'middle', mono=mono)
        ty += 15


def arrow(s, x1, y1, x2, y2, colour=DIM, width=1.5, dash=False):
    s.line(x1, y1, x2, y2, colour, width, ' marker-end="url(#arrow)"' + (' stroke-dasharray="4 3"' if dash else ''))


def phase(n, k, dur, attr='opacity'):
    """An animate element that shows its parent only in step K of N equal steps."""
    vals = ';'.join('1' if i == k else '0' for i in range(n))
    keys = ';'.join(f'{i / n:.4f}' for i in range(n))
    return (f'<animate attributeName="{attr}" values="{vals}" keyTimes="{keys}" dur="{dur}s" calcMode="discrete" '
            'repeatCount="indefinite"/>')


def group(body, n, k, dur):
    """BODY (SVG elements) shown only in step K of N."""
    return f'<g opacity="{1 if k == 0 else 0}">{body}{phase(n, k, dur)}</g>'


def part(fn):
    """The elements FN draws on a scratch card, without the card itself: for groups that animate."""
    t = Svg(900, 400, '')
    fn(t)
    return '\n'.join(t.parts)


def dot(s, path, dur, colour=AMBER, r=5, begin=0.0):
    s.add(f'<circle r="{r}" fill="{colour}"><animateMotion path="{path}" dur="{dur}s" begin="{-begin}s" '
          'repeatCount="indefinite"/></circle>')


def legend(s, x, y, items):
    for colour, label in items:
        s.rect(x, y - 10, 12, 12, colour, rx=2, extra=' fill-opacity="0.6"')
        s.text(x + 18, y, label, 11, DIM)
        x += 34 + len(label) * 6.2


def memmap(s, x, y, w, rows, scale=1.0, addr_w=58):
    """A vertical memory map. ROWS: (start address, label, height, colour). Returns the bottom y."""
    for start, label, h, colour in rows:
        h *= scale
        s.rect(x + addr_w, y, w, h - 2, colour, colour, rx=3, extra=' fill-opacity="0.2" stroke-opacity="0.5"')
        s.text(x + addr_w - 6, y + 12, start, 11, DIM, 'end', mono=True)
        s.text(x + addr_w + 10, y + min(h / 2 + 4, 16), label, 12, TEXT)
        y += h
    return y


def bitrow(s, x, y, fields, unit, h=30, size=11):
    """A row of bit fields, left to right: (label, width in bits, colour). Returns the end x."""
    for label, n, colour in fields:
        w = n * unit
        s.rect(x, y, w - 2, h, colour, rx=3, extra=' fill-opacity="0.3"')
        s.text(x + w / 2 - 1, y + h / 2 + 4, label, size, TEXT, 'middle', '600')
        x += w
    return x


def flow(s, x, y, w, h, gap, steps, colour=TEAL, size=12):
    """Boxes left to right joined by arrows. STEPS: (title, lines) or (title, lines, colour)."""
    for i, st in enumerate(steps):
        c = st[2] if len(st) > 2 else colour
        box(s, x, y, w, h, c, st[0], st[1], size)
        if i:
            arrow(s, x - gap + 2, y + h / 2, x - 3, y + h / 2)
        x += w + gap
    return x


def timeline(s, items, lo, hi, x0, x1, ya, colours, step=5):
    """Dots on a year axis with labels stacked so they do not touch. ITEMS: (year, label, key)."""
    X = lambda yr: x0 + (yr - lo) / (hi - lo) * (x1 - x0)
    s.line(x0, ya, x1, ya, GREY, 2)
    for yr in range(int(lo) + (step - int(lo) % step) % step, int(hi) + 1, step):
        s.line(X(yr), ya - 4, X(yr), ya + 4, GREY, 1)
        s.text(X(yr), ya + 20, str(yr), 11, DIM, 'middle', mono=True)
    placed, labels = [], []
    for yr, label, key in items:
        x = X(yr)
        half = len(label) * 3.3 + 6
        level = 0
        while any(lv == level and l < x + half and x - half < r for lv, l, r in placed):
            level += 1
        placed.append((level, x - half, x + half))
        y = ya - 22 - level * 22
        s.line(x, ya - 5, x, y + 5, colours[key], 1)
        s.add(f'<circle cx="{x:.1f}" cy="{ya}" r="4.5" fill="{colours[key]}"/>')
        labels.append(f'<rect x="{x - half + 4:.1f}" y="{y - 12:.1f}" width="{2 * half - 8:.1f}" height="16" fill="{BG}"/>')
        labels.append(f'<text x="{x:.1f}" y="{y:.1f}" font-family="{FONT}" font-size="12" fill="{TEXT}" text-anchor="middle" '
              f'stroke="{BG}" stroke-width="4" paint-order="stroke">{esc(label)}</text>')
    for lab in labels:
        s.add(lab)
    return X


def screen(s, x, y, w, h, label, colour, sub=None):
    s.rect(x, y, w, h, colour, colour, rx=2, extra=' fill-opacity="0.15" stroke-opacity="0.8"')
    s.text(x + 6, y + 15, label, 11, TEXT, weight='600')
    if sub:
        s.text(x + 6, y + 29, sub, 10, DIM, mono=True)


# ================================================================ docs/consoles/README.md
def console_frame_loop():
    s = Svg(900, 360, 'The console frame loop: the reset vector starts the game, the main loop runs the logic, and the '
                      'vertical blank interrupt writes the video chip, starts DMA and reads the controller')
    s.heading('The frame on a console', 'No operating system. The game drives the hardware itself, in time with the display.')
    # The display's time: visible lines, then the vertical blank, twice.
    x0, y0, fw = 40, 92, 400
    for f in range(2):
        x = x0 + f * fw
        s.rect(x, y0, fw * 0.84 - 2, 26, BLUE, rx=3, extra=' fill-opacity="0.22"')
        s.text(x + fw * 0.42, y0 + 17, 'visible lines: the chip draws the picture', 11, TEXT, 'middle')
        s.rect(x + fw * 0.84, y0, fw * 0.16 - 2, 26, AMBER, rx=3, extra=' fill-opacity="0.5"')
        s.text(x + fw * 0.92, y0 + 17, 'vblank', 11, TEXT, 'middle', '600')
    s.add(f'<rect x="{x0}" y="{y0 - 5}" width="3" height="36" fill="{TEXT}"><animate attributeName="x" '
          f'values="{x0};{x0 + 2 * fw - 3}" dur="3.2s" repeatCount="indefinite"/></rect>')
    s.text(x0, y0 + 46, 'One frame lasts 1/60 s (NTSC) or 1/50 s (PAL). Almost every game runs its logic once a frame.', 12, DIM)
    # What runs when.
    y = 186
    box(s, 40, y, 150, 64, GREY, 'Reset vector', ['set up the hardware', 'and clear RAM'])
    arrow(s, 192, y + 32, 236, y + 32)
    box(s, 240, y, 230, 64, BLUE, 'Main loop', ['game logic for the next frame,', 'then wait for the vblank'])
    box(s, 560, y, 300, 64, AMBER, 'Vertical blank interrupt (NMI)', ['write video registers, start sprite DMA,',
                                                                      'read the controller into RAM'])
    s.add(f'<path d="M472 {y + 20}C 510 {y - 6} 520 {y - 6} 556 {y + 20}" fill="none" stroke="{DIM}" stroke-width="1.5" '
          'marker-end="url(#arrow)"/>')
    s.add(f'<path d="M556 {y + 46}C 520 {y + 72} 510 {y + 72} 474 {y + 46}" fill="none" stroke="{DIM}" stroke-width="1.5" '
          'marker-end="url(#arrow)"/>')
    s.text(515, y - 8, 'interrupt', 11, DIM, 'middle')
    s.text(515, y + 82, 'return', 11, DIM, 'middle')
    para(s, 40, 300, 'Find these three places first. The reset vector leads to the start-up code. The vertical blank '
                     'handler gives the frame timing. The controller read gives the input to log each frame.', 820)
    return s.render()


def console_first_steps():
    s = Svg(900, 300, 'Six first steps on a console game, from the exact revision of the ROM to the map of the main '
                      'variables in RAM')
    s.heading('Where to start on a console game', 'Each step gives the next one what it needs.')
    steps = [('1. Identify', ['console, region, revision', 'and the checksum'], GREY),
             ('2. Emulator', ['an accurate one', 'with a debugger'], TEAL),
             ('3. Ghidra', ['right processor and', 'memory map, bank by bank'], BLUE),
             ('4. Entry points', ['the reset vector and', 'the vblank handler'], PURPLE),
             ('5. Controller read', ['read once a frame,', 'log it as the input record'], AMBER),
             ('6. Map the RAM', ['player, enemies, level,', 'random state into labels/'], GREEN)]
    for i, (t, lines, c) in enumerate(steps):
        x, y = 40 + (i % 3) * 280, 84 + (i // 3) * 100
        box(s, x, y, 240, 70, c, t, lines)
        if i % 3:
            arrow(s, x - 38, y + 35, x - 3, y + 35)
    s.add(f'<path d="M800 {84 + 72}C 800 172 160 160 160 {184 - 3}" fill="none" stroke="{DIM}" stroke-width="1.5" '
          'marker-end="url(#arrow)"/>')
    s.text(40, 284, 'Then the usual loop: trace, emulate, port, verify.', 12, DIM)
    return s.render()


# ================================================================ docs/consoles/platforms.md
def nes_bank_switching():
    dur = 6
    s = Svg(900, 400, 'NES bank switching: the mapper puts one 16 KB bank of the cartridge at 8000 to BFFF while the '
                      'last bank stays at C000 to FFFF, so the same address holds different code at different times')
    s.heading('Bank switching on the NES', 'The CPU sees 32 KB of cartridge ROM. A mapper chip such as MMC1 chooses which banks are there.')
    s.text(40, 92, 'what the CPU sees', 12, TEXT, weight='600')
    s.text(40, 114, '$8000', 11, DIM, mono=True)
    s.text(40, 236, '$C000', 11, DIM, mono=True)
    s.text(40, 342, '$FFFF', 11, DIM, mono=True)
    box(s, 92, 104, 170, 116, AMBER, '', op=0.15)
    s.text(177, 128, 'switchable window', 12, TEXT, 'middle', '600')
    for k, b in enumerate((2, 5, 0)):
        s.add(f'<text x="177" y="172" font-family="{MONO}" font-size="20" font-weight="600" fill="{AMBER}" '
              f'text-anchor="middle" opacity="{1 if k == 0 else 0}">bank {b}{phase(3, k, dur)}</text>')
    s.text(177, 200, '16 KB', 11, DIM, 'middle')
    box(s, 92, 226, 170, 116, TEAL, 'fixed: bank 7', ['16 KB, the last bank:', 'reset code and vectors'], op=0.15)
    # The cartridge's eight banks.
    s.text(520, 92, 'the cartridge ROM: 8 banks of 16 KB (128 KB)', 12, TEXT, weight='600')
    bx, by, bw, bh = 520, 104, 40, 236
    for b in range(8):
        x = bx + b * (bw + 4)
        colour = TEAL if b == 7 else BLUE
        s.rect(x, by, bw, bh, colour, rx=3, extra=' fill-opacity="0.18"')
        s.text(x + bw / 2, by + bh + 16, str(b), 11, DIM, 'middle', mono=True)
    for k, b in enumerate((2, 5, 0)):
        x = bx + b * (bw + 4)
        s.add(f'<g opacity="{1 if k == 0 else 0}"><rect x="{x}" y="{by}" width="{bw}" height="{bh}" rx="3" fill="{AMBER}" '
              f'fill-opacity="0.75"/><path d="M264 162C 380 162 {x - 60} {by + 60} {x - 2} {by + 60}" fill="none" '
              f'stroke="{AMBER}" stroke-width="2" marker-end="url(#arrow)"/>{phase(3, k, dur)}</g>')
    s.add(f'<path d="M264 284C 400 284 {bx + 7 * 44 - 40} {by + 200} {bx + 7 * 44 - 2} {by + 200}" fill="none" '
          f'stroke="{TEAL}" stroke-width="2" marker-end="url(#arrow)"/>')
    s.text(40, 384, 'Address $8123 runs different code in bank 2 and bank 5. Name functions 02:8123 and 05:8123.', 12, DIM)
    return s.render()


def n64_rcp():
    s = Svg(900, 330, 'Nintendo 64 graphics: the game on the VR4300 builds display lists, the RSP runs microcode on '
                      'them, and the RDP draws the triangles into the frame buffer; a port interprets the display lists')
    s.heading('Nintendo 64: graphics are microcode', 'The game builds a list of commands. The RCP\'s two halves turn it into pixels.')
    y, h = 96, 76
    steps = [('VR4300 (game)', ['builds a display list', 'in RDRAM'], BLUE),
             ('Display list', ['matrices, vertices,', 'triangles, textures'], GREY),
             ('RSP microcode', ['Fast3D, F3DEX ...', 'transform and light'], PURPLE),
             ('RDP', ['rasterises the', 'triangles'], AMBER),
             ('Frame buffer', ['in RDRAM,', 'shown by the VI'], TEAL)]
    flow(s, 30, y, 148, h, 30, steps)
    # The port's route.
    py = 236
    box(s, 208, py, 340, 52, GREEN, 'A port: read the display lists', ['draw them with a modern graphics API'])
    s.add(f'<path d="M282 {y + h + 2}L282 {py - 3}" stroke="{GREEN}" stroke-width="2" fill="none" marker-end="url(#arrow)"/>')
    para(s, 580, py + 8, 'The SM64 PC port works this way. Audio is also a microcode program on the RSP.', 290)
    return s.render()


def psx_gte():
    s = Svg(900, 340, 'The PlayStation GTE: a vertex of 16-bit integers is rotated by a fixed-point matrix where 4096 '
                      'means 1.0, translated, then divided by depth to give screen coordinates; a flag register records '
                      'overflow')
    s.heading('PlayStation: the GTE does 3D in fixed point', 'Coprocessor 2. No floating point anywhere on the console.')
    y = 96
    box(s, 30, y, 150, 80, BLUE, 'Vertex V', ['X, Y, Z: 16-bit', 'integers'], mono=False)
    s.text(196, y + 46, 'x', 18, DIM, 'middle')
    box(s, 212, y, 170, 80, PURPLE, 'Rotation R', ['3 x 3, 16-bit, 1.3.12', '4096 means 1.0'])
    s.text(398, y + 46, '+', 18, DIM, 'middle')
    box(s, 414, y, 120, 80, PURPLE, 'TR', ['translation,', '32-bit'])
    arrow(s, 538, y + 40, 574, y + 40)
    box(s, 578, y, 140, 80, AMBER, 'Divide by Z', ['H / Z: the', 'projection'])
    arrow(s, 722, y + 40, 752, y + 40)
    box(s, 756, y, 114, 80, TEAL, 'SX, SY', ['screen', 'coordinates'])
    s.text(30, y + 112, 'Example: one matrix entry of 2048 halves a coordinate. 2048 / 4096 = 0.5.', 12, TEXT, mono=False)
    box(s, 30, 240, 400, 66, RED, 'FLAG register', ['overflow and saturation of each step,', 'with the GTE\'s own rounding'])
    para(s, 460, 254, 'Some games use GTE results for collision and positions. There a port must give the same '
                      'integers bit for bit. psx-spx describes every GTE operation.', 410)
    return s.render()


def ps2_ee_iop():
    s = Svg(900, 380, 'The PlayStation 2: the Emotion Engine with its R5900 core and two vector units feeds the '
                      'Graphics Synthesizer; the IOP handles input, sound and discs; its floats differ from IEEE 754')
    s.heading('PlayStation 2: three kinds of processor', 'Game code runs on the R5900 and the vector units. The IOP runs the input, sound and disc side.')
    box(s, 30, 84, 450, 150, BLUE, '', op=0.08)
    s.text(46, 104, 'Emotion Engine', 13, TEXT, weight='600')
    box(s, 46, 116, 150, 100, BLUE, 'R5900 core', ['MIPS with 128-bit', 'integer instructions', 'and an FPU'])
    box(s, 210, 116, 120, 100, PURPLE, 'VU0', ['vector unit,', 'close to the core'])
    box(s, 344, 116, 120, 100, PURPLE, 'VU1', ['vector unit,', 'runs geometry', 'microprograms'])
    arrow(s, 482, 166, 528, 166)
    box(s, 532, 126, 160, 80, AMBER, 'Graphics', ['Synthesizer'])
    box(s, 720, 84, 150, 150, TEAL, 'IOP', ['a PlayStation CPU:', 'controllers,', 'sound, discs'])
    s.line(694, 166, 718, 166, DIM, 1.5)
    # The float difference.
    y = 262
    s.text(30, y, 'The same float operation, two results', 13, TEXT, weight='600')
    rows = [('operation', 'IEEE 754 (PC)', 'PS2 FPU and VUs'), ('result too large', 'infinity', 'clamps to the largest value'),
            ('0 / 0', 'NaN', 'never NaN')]
    for r, (a, b, c) in enumerate(rows):
        yy = y + 24 + r * 24
        col = DIM if r == 0 else TEXT
        s.text(30, yy, a, 12, col, mono=r > 0)
        s.text(260, yy, b, 12, TEAL if r else DIM)
        s.text(460, yy, c, 12, AMBER if r else DIM)
    s.text(30, y + 104, 'Rounding also differs. A port with normal floats differs at the edges. PCSX2 documents each case.', 12, DIM)
    return s.render()


# ================================================================ docs/consoles/early-consoles.md
def early_console_timeline():
    s = Svg(900, 300, 'The early consoles by year of release, from the Magnavox Odyssey in 1972 to the Atari 7800 in '
                      '1986, coloured by CPU family')
    s.heading('Early consoles', 'Year of release, coloured by CPU family. Ghidra builds in all four CPU families.')
    cols = {'none': GREY, '6502': TEAL, 'CP1600': PURPLE, 'Z80': BLUE, '6809': AMBER}
    items = [(1972.7, 'Odyssey', 'none'), (1977.7, 'Atari 2600', '6502'), (1980, 'Intellivision', 'CP1600'),
             (1982.6, 'ColecoVision', 'Z80'), (1982.85, 'Atari 5200', '6502'), (1982.85, 'Vectrex', '6809'),
             (1986.4, 'Atari 7800', '6502')]
    timeline(s, items, 1971, 1988, 50, 850, 210, cols)
    legend(s, 50, 272, [(GREY, 'no CPU'), (TEAL, '6502 family'), (PURPLE, 'CP1600'), (BLUE, 'Z80'), (AMBER, '6809')])
    return s.render()


def atari2600_beam():
    s = Svg(900, 380, 'Racing the beam on the Atari 2600: a scanline is 228 colour clocks or 76 CPU cycles, and the '
                      'program writes the TIA registers in time with each line of the frame')
    s.heading('Atari 2600: racing the beam', 'No frame buffer. The program writes the TIA while the beam moves, every line.')
    # One scanline.
    x0, y0, w = 40, 104, 560
    hb = w * 68 / 228
    s.text(x0, y0 - 10, 'one scanline: 228 colour clocks = 76 CPU cycles', 12, TEXT, weight='600')
    s.rect(x0, y0, hb - 2, 34, GREY, rx=3, extra=' fill-opacity="0.35"')
    s.text(x0 + hb / 2, y0 + 21, 'blank: 68', 11, TEXT, 'middle')
    s.rect(x0 + hb, y0, w - hb, 34, BLUE, rx=3, extra=' fill-opacity="0.22"')
    s.text(x0 + hb + (w - hb) / 2, y0 + 21, 'visible: 160 pixels', 11, TEXT, 'middle')
    s.add(f'<rect x="{x0}" y="{y0 - 4}" width="3" height="42" fill="{AMBER}"><animate attributeName="x" '
          f'values="{x0};{x0 + w - 3}" dur="1.2s" repeatCount="indefinite"/></rect>')
    # The CPU's writes along the line.
    s.text(x0 + 4, y0 + 74, 'STA WSYNC waits for the next line.', 11, DIM)
    s.text(x0 + 4, y0 + 90, 'Writes to GRP0 must land before the beam reaches the sprite.', 11, DIM)
    s.text(x0 + 4, y0 + 106, 'A write in the middle of the line changes the rest of that line.', 11, DIM)
    # The frame.
    fx, fy = 660, 84
    s.text(fx, fy + 8, 'one NTSC frame: 262 lines', 12, TEXT, weight='600')
    parts = [('VSYNC', 3, AMBER), ('VBLANK', 37, GREY), ('visible', 192, BLUE), ('overscan', 30, GREY)]
    y = fy + 20
    for label, n, c in parts:
        h = max(n * 0.9, 12)
        s.rect(fx, y, 110, h - 2, c, rx=2, extra=' fill-opacity="0.3"')
        s.text(fx + 118, y + min(h / 2 + 4, 11), f'{label} {n}', 11, DIM)
        y += h
    s.add(f'<rect x="{fx}" y="{fy + 20}" width="110" height="2" fill="{AMBER}"><animate attributeName="y" '
          f'values="{fy + 20};{y - 2}" dur="4s" repeatCount="indefinite"/></rect>')
    para(s, 40, 300, 'Game logic runs in the VBLANK and overscan lines. During the visible lines the code is the '
                     'drawing. A port must model the CPU cycles in each line, so take ground truth from cycle-exact '
                     'emulator traces.', 580)
    return s.render()


def cp1610_words():
    s = Svg(900, 340, 'The Intellivision CP1610: ROM words are 10 bits wide, and the SDBD prefix builds a 16-bit '
                      'operand from the low 8 bits of two ROM words; R6 is the stack pointer and R7 the program counter')
    s.heading('Intellivision: 10-bit ROM words', 'The CP1610 addresses 16-bit words. Cartridge ROM gives 10 bits of each.')
    s.text(40, 96, 'one ROM word', 12, TEXT, weight='600')
    bitrow(s, 40, 106, [('unused', 6, GREY), ('10 bits of data', 10, BLUE)], 22)
    s.text(40, 172, 'SDBD, then an instruction with an immediate operand', 12, TEXT, weight='600')
    bitrow(s, 40, 182, [('x', 2, GREY), ('low byte', 8, AMBER)], 22)
    s.text(270, 202, 'first word', 11, DIM)
    bitrow(s, 40, 220, [('x', 2, GREY), ('high byte', 8, TEAL)], 22)
    s.text(270, 240, 'second word', 11, DIM)
    arrow(s, 340, 220, 384, 220)
    bitrow(s, 390, 205, [('high byte', 8, TEAL), ('low byte', 8, AMBER)], 22)
    s.text(390, 260, 'one 16-bit operand', 11, DIM)
    # The registers.
    rx = 760
    s.text(rx, 96, 'registers', 12, TEXT, weight='600')
    for i in range(8):
        c = PURPLE if i == 6 else (AMBER if i == 7 else GREY)
        name = {6: 'R6  stack', 7: 'R7  PC'}.get(i, f'R{i}')
        s.rect(rx, 106 + i * 24, 110, 20, c, rx=3, extra=' fill-opacity="0.3"')
        s.text(rx + 8, 120 + i * 24, name, 11, TEXT, mono=True)
    para(s, 40, 296, 'Game code also calls into the Executive ROM. A port must reproduce what that ROM does, so trace '
                     'those calls too.', 680)
    return s.render()


def colecovision_blocks():
    s = Svg(900, 280, 'The ColecoVision: a Z80A with 1 KB of RAM and an 8 KB BIOS, a TMS9928A video chip with its own '
                      '16 KB of VRAM, an SN76489A for sound and the cartridge')
    s.heading('ColecoVision: the parts', 'A Z80A at 3.58 MHz. The video chip has its own memory, reached through ports.')
    s.line(40, 150, 860, 150, GREY, 3)
    s.text(44, 142, 'CPU bus', 11, DIM)
    parts = [(50, 'Z80A', ['3.58 MHz'], BLUE), (190, 'RAM', ['1 KB'], GREY), (330, 'BIOS', ['8 KB'], PURPLE),
             (470, 'Cartridge', ['the game'], TEAL), (610, 'SN76489A', ['sound'], AMBER), (750, 'TMS9928A', ['video'], GREEN)]
    for x, t, lines, c in parts:
        box(s, x, 170, 110, 56, c, t, lines)
        s.line(x + 55, 152, x + 55, 168, GREY, 2)
    box(s, 750, 76, 110, 48, GREEN, 'VRAM', ['16 KB'])
    s.line(805, 126, 805, 168, GREEN, 2)
    s.text(40, 256, 'Unicorn and Capstone have no Z80. Take ground truth from a small core such as iz80, or from MAME.', 12, DIM)
    return s.render()


def atari7800_clock():
    s = Svg(900, 330, 'The Atari 7800: the CPU runs at 1.79 MHz and slows to 1.19 MHz when it touches the TIA or the '
                      'RIOT; NTSC consoles check the cartridge signature and PAL consoles skip the check')
    s.heading('Atari 7800: a clock that changes', 'SALLY runs at 1.79 MHz. Each access to the TIA or the RIOT runs at 1.19 MHz.')
    x, y = 40, 104
    seq = [('RAM', 1.79), ('ROM', 1.79), ('TIA', 1.19), ('RAM', 1.79), ('RIOT', 1.19), ('ROM', 1.79), ('ROM', 1.79), ('TIA', 1.19)]
    for name, mhz in seq:
        w = 60 * 1.79 / mhz
        c = AMBER if mhz < 1.5 else BLUE
        s.rect(x, y, w - 3, 34, c, rx=3, extra=' fill-opacity="0.3"')
        s.text(x + w / 2 - 1, y + 21, name, 11, TEXT, 'middle', '600')
        x += w
    s.text(40, y + 54, 'Each box is one CPU cycle. Cycles on the TIA and the RIOT take 1.5 times as long.', 12, DIM)
    s.text(40, y + 72, 'A port that counts cycles must count these.', 12, DIM)
    # The signature check.
    y = 218
    box(s, 40, y, 160, 56, GREY, 'Cartridge', ['inserted'])
    arrow(s, 202, y + 28, 246, y + 28)
    box(s, 250, y, 200, 56, BLUE, 'NTSC console', ['checks the signature'])
    arrow(s, 452, y + 28, 496, y + 28)
    box(s, 500, y, 130, 56, TEAL, 'Game runs', ['if it is valid'])
    s.text(655, y + 33, 'or', 12, DIM, 'middle')
    box(s, 680, y, 190, 56, PURPLE, 'PAL console', ['skips the check'])
    return s.render()


def vectrex_beam():
    s = Svg(900, 330, 'The Vectrex: the 6809 sets the beam position through a DAC and the beam draws lines; the 6809 '
                      'joins A and B into the 16-bit D register and multiplies A by B into D')
    s.heading('Vectrex: the CPU draws', 'No video memory. The 68A09 steers the beam through a DAC, line by line.')
    box(s, 40, 96, 110, 56, BLUE, '68A09', ['1.5 MHz'])
    arrow(s, 152, 124, 196, 124)
    box(s, 200, 96, 110, 56, AMBER, 'DAC', ['X, Y, brightness'])
    arrow(s, 312, 124, 356, 124)
    s.rect(360, 76, 200, 230, GREY, rx=6, extra=' fill-opacity="0.12"')
    ship = 'M460 120L500 220L460 196L420 220Z'
    s.add(f'<path d="{ship}" fill="none" stroke="{TEXT}" stroke-width="2.5" stroke-dasharray="360" stroke-dashoffset="360">'
          '<animate attributeName="stroke-dashoffset" values="360;0;0" keyTimes="0;0.7;1" dur="2.5s" repeatCount="indefinite"/></path>')
    s.text(460, 290, 'the beam draws each line', 11, DIM, 'middle')
    # The registers.
    rx = 610
    s.text(rx, 96, '6809 registers', 12, TEXT, weight='600')
    bitrow(s, rx, 108, [('A', 8, AMBER), ('B', 8, TEAL)], 12)
    s.text(rx + 202, 128, '= D, 16-bit', 13, TEXT, weight='600', mono=True)
    for i, r in enumerate(('X', 'Y', 'U', 'S')):
        s.rect(rx, 150 + i * 24, 240, 20, GREY, rx=3, extra=' fill-opacity="0.3"')
        s.text(rx + 8, 164 + i * 24, f'{r}  16-bit', 11, TEXT, mono=True)
    s.text(rx, 260, 'MUL: A x B gives D, in hardware', 12, TEXT)
    para(s, rx, 284, 'Game code does the drawing. Trace it like the logic.', 250)
    return s.render()


# ================================================================ docs/consoles/cd-add-ons.md
def console_code_upload():
    s = Svg(900, 300, 'Code for a second CPU: the main CPU copies it from the disc or cartridge into the second CPU\'s '
                      'RAM, so take it from a RAM dump and load the dump into Ghidra at its address')
    s.heading('Code that only exists in RAM', 'On the Sega CD, the 32X and the Jaguar, the main CPU uploads code for another CPU.')
    y = 100
    box(s, 40, y, 150, 70, GREY, 'Disc or cartridge', ['the code, maybe packed'])
    box(s, 260, y, 150, 70, BLUE, 'Main CPU', ['copies it'])
    box(s, 480, y, 170, 70, PURPLE, 'Second CPU\'s RAM', ['the code at its', 'run address'])
    box(s, 720, y, 150, 70, TEAL, 'Second CPU', ['runs it'])
    arrow(s, 192, y + 35, 256, y + 35)
    arrow(s, 412, y + 35, 476, y + 35)
    arrow(s, 652, y + 35, 716, y + 35)
    for k in range(2):
        dot(s, f'M190 {y + 35}L480 {y + 35}', 2.4, AMBER, 5, k * 1.2)
    y2 = 214
    box(s, 480, y2, 170, 56, AMBER, 'RAM dump', ['from the emulator'])
    arrow(s, 565, y + 72, 565, y2 - 3)
    box(s, 720, y2, 150, 56, GREEN, 'Ghidra', ['load at the address'])
    arrow(s, 652, y2 + 28, 716, y2 + 28)
    para(s, 40, y2 + 18, 'Take the dump after the upload. Export each CPU\'s code as its own program.', 400)
    return s.render()


def segacd_word_ram():
    s = Svg(900, 330, 'The Mega-CD: a second 68000 at 12.5 MHz with 512 KB of program RAM, and 256 KB of word RAM '
                      'shared with the Mega Drive\'s 68000')
    s.heading('Mega-CD: two 68000s share word RAM', 'The add-on brings its own 68000, its own RAM, a PCM chip and the BIOS.')
    box(s, 30, 84, 280, 210, BLUE, '', op=0.07)
    s.text(46, 106, 'Mega Drive', 13, TEXT, weight='600')
    box(s, 46, 120, 120, 60, BLUE, 'Main 68000', ['the game\'s', 'main code'])
    box(s, 176, 120, 120, 60, GREY, 'Z80', ['sound'])
    box(s, 46, 196, 250, 80, GREY, 'Cartridge port', ['the add-on connects here'], op=0.12)
    box(s, 560, 84, 310, 210, PURPLE, '', op=0.07)
    s.text(576, 106, 'Mega-CD add-on', 13, TEXT, weight='600')
    box(s, 576, 120, 130, 60, PURPLE, 'Sub 68000', ['12.5 MHz'])
    box(s, 720, 120, 136, 60, GREY, 'Program RAM', ['512 KB'])
    box(s, 576, 196, 130, 80, AMBER, 'RF5C164', ['PCM sound'])
    box(s, 720, 196, 136, 80, GREY, 'BIOS', ['in the add-on'])
    box(s, 340, 150, 190, 70, TEAL, 'Word RAM', ['256 KB, shared'])
    arrow(s, 312, 185, 336, 185)
    arrow(s, 558, 185, 534, 185)
    dot(s, 'M560 175L330 175', 2.2, AMBER, 5)
    s.text(435, 248, 'data passes both ways', 11, DIM, 'middle')
    s.text(30, 316, 'Trace both 68000s. Map which code runs on which CPU and how they share word RAM.', 12, DIM)
    return s.render()


def sega_32x_cpus():
    s = Svg(900, 300, 'The Sega 32X: two SH-2s at 23 MHz with 256 KB of SDRAM, a frame buffer and PWM sound, beside the '
                      'Mega Drive\'s 68000 and Z80, so a game can run code on four CPUs')
    s.heading('Sega 32X: four CPUs, four programs', 'The Mega Drive keeps running. The 32X adds two SH-2s beside it.')
    cpus = [('SH-2 master', '23 MHz', PURPLE), ('SH-2 slave', '23 MHz', PURPLE), ('68000', 'Mega Drive', BLUE), ('Z80', 'Mega Drive sound', GREY)]
    for i, (t, sub, c) in enumerate(cpus):
        x = 40 + i * 210
        box(s, x, 90, 180, 60, c, t, [sub])
        arrow(s, x + 90, 152, x + 90, 190)
        box(s, x, 194, 180, 40, GREEN, '', [f'Ghidra program {i + 1}'])
    s.text(40, 270, 'Shared by the SH-2s: 256 KB of SDRAM, the frame buffer, PWM sound. Export each CPU\'s code separately.', 12, DIM)
    return s.render()


def huc6280_mmu():
    dur = 5
    s = Svg(900, 420, 'The HuC6280 MMU: eight mapping registers each put one 8 KB bank of a 2 MB space into one page of '
                      'the CPU\'s 64 KB; the TAM instruction changes a register')
    s.heading('PC Engine: the HuC6280 maps 64 KB onto 2 MB', 'Eight registers, MPR0 to MPR7, each choose one of 256 banks of 8 KB.')
    x0, y0, ph = 40, 90, 32
    s.text(x0, y0 - 6, 'the CPU\'s 64 KB', 12, TEXT, weight='600')
    mpr = ['$FF  I/O', '$F8  RAM', '...', '...', None, '...', '...', '$00  ROM']
    for i in range(8):
        y = y0 + 4 + i * ph
        s.text(x0, y + 20, f'${i * 0x2000:04X}', 11, DIM, mono=True)
        s.rect(x0 + 52, y, 90, ph - 3, BLUE, rx=3, extra=' fill-opacity="0.2"')
        s.text(x0 + 60, y + 19, f'MPR{i}', 11, TEXT, mono=True)
        s.rect(x0 + 150, y, 100, ph - 3, AMBER if i == 4 else GREY, rx=3, extra=' fill-opacity="0.25"')
        if mpr[i]:
            s.text(x0 + 158, y + 19, mpr[i], 11, TEXT, mono=True)
    for k, b in enumerate((0x10, 0x11, 0x12)):
        s.add(f'<text x="{x0 + 158}" y="{y0 + 4 + 4 * ph + 19}" font-family="{MONO}" font-size="11" fill="{AMBER}" '
              f'font-weight="600" opacity="{1 if k == 0 else 0}">${b:02X}  ROM{phase(3, k, dur)}</text>')
    # The physical space.
    px, pw = 520, 340
    s.text(px, y0 - 6, 'the 2 MB physical space: 256 banks of 8 KB', 12, TEXT, weight='600')
    s.rect(px, y0 + 4, pw, 256, GREY, rx=3, extra=' fill-opacity="0.12"')
    for b, label, c, dy, anchor in ((0x00, '$00 ROM start', TEAL, 14, 'end'), (0xF8, '$F8 RAM', GREEN, -4, 'start'),
                                    (0xFF, '$FF hardware I/O', PURPLE, 14, 'end')):
        yy = y0 + 4 + b
        s.rect(px, yy, pw, 2, c)
        s.text(px + 6 if anchor == 'start' else px + pw - 6, yy + dy, label, 11, c, anchor)
    for k, b in enumerate((0x10, 0x11, 0x12)):
        yy = y0 + 4 + b * 1.0
        s.add(f'<g opacity="{1 if k == 0 else 0}"><rect x="{px}" y="{yy + 20}" width="{pw}" height="8" fill="{AMBER}" '
              f'fill-opacity="0.8"/><path d="M292 {y0 + 4 + 4 * ph + 14}C 400 {y0 + 4 + 4 * ph + 14} 440 {yy + 24} '
              f'{px - 3} {yy + 24}" fill="none" stroke="{AMBER}" stroke-width="2" marker-end="url(#arrow)"/>'
              f'{phase(3, k, dur)}</g>')
    s.text(px, y0 + 292, 'TAM writes a register: MPR4 = $10, then $11, then $12.', 11, DIM)
    s.text(40, 404, 'Physical address = bank x 8 KB + offset. Ghidra\'s 65C02 mis-reads TAM and TMA. The HuC6280 extension fixes them.', 12, DIM)
    return s.render()


def cdi_os9():
    s = Svg(900, 270, 'The Philips CD-i: programs are OS-9 modules on CD-RTOS; load each module with Ghidra\'s 68000 '
                      'language and run the programs in MAME or CeDImu')
    s.heading('CD-i: programs are OS-9 modules', 'The player runs CD-RTOS, built on Microware OS-9, on an SCC68070.')
    box(s, 40, 90, 180, 130, GREY, 'CD-i disc', ['files on the disc'])
    arrow(s, 222, 155, 262, 155)
    box(s, 266, 90, 300, 130, BLUE, '', op=0.08)
    s.text(282, 110, 'OS-9 modules', 13, TEXT, weight='600')
    for i, name in enumerate(('the game program', 'a shared library', 'a data module')):
        bitrow(s, 282, 122 + i * 30, [('header', 6, PURPLE), (name, 14, BLUE)], 13, 24, 10)
    arrow(s, 568, 155, 608, 155)
    box(s, 612, 90, 250, 60, GREEN, 'Ghidra, 68000', ['one program per module'])
    box(s, 612, 160, 250, 60, TEAL, 'MAME cdimono1, CeDImu', ['to run and debug'])
    s.text(40, 250, 'Each module header names the module and gives its type and entry point.', 12, DIM)
    return s.render()


# ================================================================ docs/consoles/32-bit-oddities.md
def jaguar_cpus():
    s = Svg(900, 340, 'The Atari Jaguar: the 68000 runs the main loop from 2 MB of RAM and uploads code into the local '
                      'RAM of Tom and Jerry, two RISC cores at 26.59 MHz with 4 KB and 8 KB')
    s.heading('Atari Jaguar: one 68000, two RISC cores', 'RISC code runs badly from main RAM, so it is copied into each core\'s own small RAM.')
    box(s, 40, 96, 200, 80, BLUE, '68000', ['13.295 MHz', 'runs the main loop'])
    box(s, 40, 200, 200, 80, GREY, 'Main RAM', ['2 MB'])
    s.line(140, 178, 140, 198, DIM, 1.5)
    box(s, 380, 96, 220, 80, PURPLE, 'Tom (GPU)', ['RISC at 26.59 MHz', '4 KB local RAM'])
    box(s, 380, 200, 220, 80, AMBER, 'Jerry (DSP)', ['RISC at 26.59 MHz', '8 KB local RAM'])
    arrow(s, 242, 236, 376, 136)
    arrow(s, 242, 244, 376, 240)
    dot(s, 'M242 236L376 136', 1.8, PURPLE, 4)
    dot(s, 'M242 244L376 240', 1.8, AMBER, 4, 0.9)
    s.text(330, 226, 'upload', 11, DIM, 'middle')
    box(s, 650, 96, 210, 184, GREEN, 'For the port', ['export the 68000 code', 'from the cartridge.', '',
                                                    'take Tom and Jerry code', 'from a RAM dump after', 'the upload'])
    s.text(40, 320, 'Ghidra builds in the 68000. No Ghidra module is known for Tom and Jerry.', 12, DIM)
    return s.render()


def arm_versions_3do():
    s = Svg(900, 300, 'The 3DO\'s ARM60 is ARMv3, one version older than the first ARM language Ghidra builds in; ARMv4 '
                      'added halfword and signed byte loads and stores')
    s.heading('3DO: an ARM older than Ghidra\'s', 'Ghidra\'s ARM languages start at v4. The ARM60 is v3.')
    vers = [('ARMv3', ['ARM60 (3DO)', '26-bit and 32-bit modes'], AMBER), ('ARMv4', ['adds LDRH, STRH,', 'LDRSB, LDRSH'], BLUE),
            ('ARMv4T', ['adds Thumb', 'ARM7TDMI (GBA)'], BLUE), ('ARMv5 and later', ['ARM9 (DS) onward'], BLUE)]
    x = 40
    for i, (t, lines, c) in enumerate(vers):
        box(s, x, 96, 190, 74, c, t, lines)
        if i:
            arrow(s, x - 22, 133, x - 3, 133)
        x += 212
    s.rect(252, 186, 608, 10, TEAL, rx=3, extra=' fill-opacity="0.6"')
    s.text(252, 214, 'Ghidra\'s ARM languages', 12, TEAL)
    s.rect(40, 186, 190, 10, RED, rx=3, extra=' fill-opacity="0.6"')
    s.text(40, 214, 'no exact match', 12, RED)
    para(s, 40, 250, 'Most v3 instructions decode the same under v4. v4 adds instructions and makes the 26-bit mode '
                     'optional. Check code that uses the 26-bit mode by hand.', 820)
    return s.render()


# ================================================================ docs/consoles/matching-decomp.md
def matching_loop():
    s = Svg(900, 400, 'A matching decompilation: split the program, write C for one function, compile it with the '
                      'original compiler, compare, and when it matches replace the assembly; the build stays identical')
    s.heading('Matching decompilation, one function at a time', 'The build must give the identical program after every change.')
    box(s, 30, 90, 170, 60, GREY, '1. Compiler', ['exact version', 'and settings'])
    arrow(s, 202, 120, 240, 120)
    box(s, 244, 90, 170, 60, BLUE, '2. Split', ['splat, decomp-toolkit:', 'code, data, assets'])
    arrow(s, 416, 120, 454, 120)
    # The inner loop.
    cx, cy = 630, 190
    nodes = [(cx, cy - 92, 'Write C', 'm2c draft, decomp.me', PURPLE), (cx + 160, cy, 'Compile', 'original compiler', AMBER),
             (cx, cy + 92, 'Compare', 'asm-differ, objdiff', TEAL), (cx - 160, cy, 'Edit', 'order, types, temps', PURPLE)]
    s.add(f'<ellipse cx="{cx}" cy="{cy}" rx="160" ry="92" fill="none" stroke="{GREY}" stroke-width="1.5" stroke-dasharray="5 4"/>')
    dot(s, f'M{cx} {cy - 92}A160 92 0 1 1 {cx - 0.1} {cy - 92}', 4, AMBER, 5)
    for x, y, t, sub, c in nodes:
        s.rect(x - 75, y - 24, 150, 48, BG, rx=5)
        box(s, x - 75, y - 24, 150, 48, c, t, [sub], 12, op=0.35)
    s.text(cx, cy + 4, '3. one function', 12, DIM, 'middle')
    arrow(s, cx + 40, cy + 118, cx + 40, cy + 140)
    box(s, cx - 60, cy + 142, 260, 34, GREEN, '4. Match: C replaces the assembly', [], 12)
    para(s, 30, 190, 'The build still gives the identical program, which proves the match. Then 5: name the '
                     'variables, structures and functions.', 360)
    para(s, 30, 270, 'Old compilers are sensitive to statement order, temporary types and register use. Some '
                     'functions take days.', 360)
    return s.render()


def matching_choice():
    s = Svg(900, 366, 'Choosing a route: an existing decompilation, a matching decompilation, or disassembly and '
                      'reimplementation, by what the game is written in and what you need to publish')
    s.heading('When to choose a matching decompilation', 'Three questions decide the route.')
    q = [(40, 92, 'Is the game mostly hand-written assembly?'), (40, 182, 'Has a community already started a decompilation?'),
         (40, 272, 'Must the port be exactly the original, bugs included?')]
    for x, y, t in q:
        box(s, x, y, 380, 50, BLUE, t, [], 12)
    ans = [(500, 92, 'Disassemble, document, reimplement', 'matching proves less for assembly', GREY),
           (500, 182, 'Use or join it', 'its documents help any route', TEAL),
           (500, 272, 'Matching decompilation', 'if you accept the legal position', AMBER)]
    for x, y, t, sub, c in ans:
        box(s, x, y, 360, 50, c, t, [sub], 12)
        arrow(s, 422, y + 25, x - 3, y + 25)
        s.text(460, y + 18, 'yes', 11, GREEN, 'middle')
    for y in (142, 232):
        arrow(s, 230, y + 2, 230, y + 38)
        s.text(240, y + 26, 'no', 11, RED)
    s.text(500, 348, 'If no: reimplement from documented behaviour.', 11, DIM)
    return s.render()


# ================================================================ docs/consoles/modern.md
def modern_containers():
    s = Svg(900, 376, 'Signed and encrypted containers from the Xbox 360 on: XEX2 wraps a PE, SELF wraps an ELF, Wii '
                      'discs are AES-encrypted, RPX is an ELF variant; each needs a decrypted dump and a loader')
    s.heading('Seventh generation onward: unwrap first', 'Each executable sits in a signed vendor container. Most are encrypted.')
    rows = [('Xbox 360', 'XEX2, signed, usually encrypted', 'PE, PowerPC', 'XEXLoaderWV'),
            ('PlayStation 3', 'SELF / SPRX, signed, encrypted', 'ELF: PPU and SPU code', 'GhidraSPU for the SPU'),
            ('Wii', 'disc, AES-encrypted', 'DOL and REL', 'Ghidra-GameCube-Loader'),
            ('Wii U', 'RPX and RPL', 'ELF variant', 'GhidraRPXLoader'),
            ('PlayStation 4', 'SELF with a Sony header', 'x86-64 ELF', 'GhidraPS4Loader, GhidraOrbis')]
    s.text(170, 84, 'container', 11, DIM)
    s.text(536, 84, 'after decryption', 11, DIM)
    s.text(716, 84, 'loader', 11, DIM)
    for i, (p, wrap, inner, loader) in enumerate(rows):
        y = 94 + i * 50
        s.text(40, y + 26, p, 12, TEXT, weight='600')
        s.rect(170, y, 330, 40, RED, RED, rx=4, extra=' fill-opacity="0.12" stroke-opacity="0.6"')
        s.text(180, y + 25, wrap, 11, DIM)
        s.rect(390, y + 6, 102, 28, BLUE, rx=3, extra=' fill-opacity="0.3"')
        s.text(441, y + 24, 'code', 11, TEXT, 'middle')
        arrow(s, 502, y + 20, 532, y + 20)
        s.text(536, y + 25, inner, 11, TEXT)
        s.text(716, y + 25, loader, 11, TEAL)
    s.text(40, 362, 'You need a decrypted dump of your own copy first. GameFrame keeps all game material out of the repository.', 12, DIM)
    return s.render()


def modern_status():
    s = Svg(900, 376, 'Tool status from the Xbox 360 to the Series consoles: which have a Ghidra loader and which have '
                      'an emulator that runs commercial games')
    s.heading('Seventh generation onward: what exists', 'From the table on this page. Pin an exact emulator build and keep a copy.')
    rows = [('Xbox 360', 'XEXLoaderWV', 'Xenia Canary'), ('PlayStation 3', 'GhidraSPU (SPU only)', 'RPCS3'),
            ('Wii', 'GameCube loader', 'Dolphin'), ('Wii U', 'GhidraRPXLoader', 'Cemu'), ('PlayStation 4', 'GhidraPS4Loader', 'shadPS4'),
            ('Xbox One', None, None), ('PlayStation 5', None, None), ('Xbox Series', None, None)]
    s.text(220, 86, 'Ghidra loader', 11, DIM)
    s.text(520, 86, 'emulator', 11, DIM)
    for i, (p, ld, emu) in enumerate(rows):
        y = 96 + i * 30
        s.text(40, y + 17, p, 12, TEXT, weight='600')
        for x, v in ((220, ld), (520, emu)):
            c = GREEN if v else RED
            if v and 'only' in v:
                c = AMBER
            s.rect(x, y, 270, 24, c, rx=3, extra=' fill-opacity="0.22"')
            s.text(x + 10, y + 16, v or 'none known', 11, TEXT if v else DIM)
    s.text(40, 362, 'The PS4, Xbox One, PS5 and Series are x86-64. Many of their games also have PC releases.', 12, DIM)
    return s.render()


def ps3_cell():
    s = Svg(900, 360, 'The PlayStation 3 Cell: one PPE and six SPEs for games; each SPE has 256 KiB of local store and '
                      'reaches main memory only by DMA; SPU floats differ from IEEE')
    s.heading('PlayStation 3: the Cell', 'The PPE runs the main code. Each SPU works only on its own local store.')
    box(s, 40, 92, 180, 90, BLUE, 'PPE', ['PowerPC core', 'the main program'])
    box(s, 40, 206, 180, 64, GREY, 'Main memory', [])
    s.line(130, 184, 130, 204, DIM, 1.5)
    for i in range(6):
        x = 270 + (i % 3) * 200
        y = 92 + (i // 3) * 100
        box(s, x, y, 180, 80, PURPLE, f'SPE {i}', ['own 128-bit ISA', '256 KiB local store'])
    s.add(f'<path d="M222 238H245V182H850" fill="none" stroke="{AMBER}" stroke-width="2" stroke-opacity="0.6"/>')
    for i in range(3):
        dot(s, 'M222 238H245V182H850', 3.0, AMBER, 4, i)
    s.text(270, 300, 'DMA is the only way between main memory and a local store.', 12, DIM)
    s.text(40, 330, 'SPU single precision rounds toward zero, has no INF or NaN, treats denormals as zero (not confirmed).', 12, TEXT)
    s.text(40, 348, 'A port of SPU float code needs a software float model.', 12, DIM)
    return s.render()


def xenon_recomp():
    s = Svg(900, 300, 'Static recompilation of an Xbox 360 game: XenonRecomp turns the PowerPC code into C++ and '
                      'XenosRecomp turns the shaders into HLSL; the player\'s own game files complete the PC port')
    s.heading('Xbox 360: static recompilation', 'How Unleashed Recompiled was made. The project ships no game files.')
    box(s, 40, 100, 160, 60, RED, 'XEX2', ['the player\'s copy'])
    arrow(s, 202, 120, 246, 105)
    arrow(s, 202, 140, 246, 175)
    box(s, 250, 80, 180, 50, BLUE, 'PowerPC code', [])
    box(s, 250, 150, 180, 50, PURPLE, 'Xenos shaders', [])
    arrow(s, 432, 105, 476, 105)
    arrow(s, 432, 175, 476, 175)
    box(s, 480, 80, 180, 50, BLUE, 'XenonRecomp', ['to C++'])
    box(s, 480, 150, 180, 50, PURPLE, 'XenosRecomp', ['to HLSL'])
    arrow(s, 662, 105, 706, 130)
    arrow(s, 662, 175, 706, 150)
    box(s, 710, 100, 150, 80, GREEN, 'Native PC port', ['with a new', 'hardware layer'])
    para(s, 40, 240, 'ReXGlue (2026) is an open SDK built on XenonRecomp and Xenia. It is in early development.', 820)
    return s.render()


def gekko_paired_singles():
    s = Svg(900, 340, 'GameCube and Wii paired singles: one register holds two floats, and a quantised load turns '
                      'small integers into floats with a type and a scale set in a GQR register')
    s.heading('Gekko and Broadway: paired singles', 'One floating-point register holds two single floats, ps0 and ps1. One instruction works on both.')
    bitrow(s, 40, 92, [('ps0', 16, BLUE), ('ps1', 16, TEAL)], 12)
    s.text(440, 112, 'one 64-bit register', 11, DIM)
    s.text(40, 166, 'A quantised load (psq_l) reads compact values and converts them', 12, TEXT, weight='600')
    box(s, 40, 180, 170, 60, GREY, 'Memory', ['u8 value 200'], mono=False)
    arrow(s, 212, 210, 256, 210)
    box(s, 260, 180, 230, 60, AMBER, 'GQR register', ['load type: u8', 'load scale: 7'])
    arrow(s, 492, 210, 536, 210)
    box(s, 540, 180, 200, 60, BLUE, 'ps0 = 1.5625', ['200 / 2^7 = 200 / 128'])
    s.text(40, 270, 'Types: u8, u16, s8, s16 or float. The value is divided by 2 to the power of the scale.', 12, DIM)
    para(s, 40, 296, 'So the same code gives different results under a different GQR setup. Confirm the setup '
                     'against Dolphin\'s code.', 820)
    return s.render()


# ================================================================ docs/consoles/porting.md
def port_routes():
    s = Svg(900, 330, 'Four routes to a port compared: emulation, static recompilation, matching decompilation and '
                      'reimplementation, by effort and by what the project can publish')
    s.heading('Four routes to a port', 'They differ in effort and in what the project may publish.')
    rows = [('Emulation', 1, 'the emulator', 'the game', TEAL), ('Static recompilation', 2, 'recompiler, hardware layer', 'the ROM', BLUE),
            ('Matching decomp, then port', 4, 'contested: derived C', 'the ROM', AMBER), ('Reimplementation', 3, 'the new code', 'the data', GREEN)]
    s.text(250, 88, 'effort', 11, DIM)
    s.text(470, 88, 'what can be published', 11, DIM)
    s.text(700, 88, 'the player brings', 11, DIM)
    for i, (t, eff, pub, brings, c) in enumerate(rows):
        y = 100 + i * 46
        s.text(40, y + 22, t, 12, TEXT, weight='600')
        for k in range(4):
            s.rect(250 + k * 46, y + 6, 42, 22, c if k < eff else GREY, rx=3, extra=f' fill-opacity="{0.7 if k < eff else 0.15}"')
        s.text(470, y + 22, pub, 12, TEXT)
        s.text(700, y + 22, brings, 12, TEXT)
    s.text(40, 300, 'GameFrame is built for reimplementation. The other routes share most of its steps. Write the choice into AGENTS.md.', 12, DIM)
    return s.render()


def port_fixed_rate():
    s = Svg(900, 360, 'The logic keeps the original rate: on a faster display the port shows the latest logic frame '
                      'again, and a lag frame is a frame where the original did no logic update')
    s.heading('The frame and the clock', 'Run the logic at the original rate, such as 60.0988 Hz for an NTSC NES. Only the picture follows the display.')
    x0, w = 160, 700
    s.text(40, 108, 'logic updates', 12, TEXT, weight='600')
    s.text(40, 168, 'a faster display', 12, TEXT, weight='600')
    n, m = 6, 10
    for i in range(n):
        s.rect(x0 + i * w / n, 92, w / n - 4, 26, BLUE, rx=3, extra=' fill-opacity="0.3"')
        s.text(x0 + i * w / n + 8, 110, f'update {i + 1}', 11, TEXT)
    for j in range(m):
        x = x0 + j * w / m
        shown = int(j * (w / m) // (w / n)) + 1
        s.rect(x, 152, w / m - 4, 26, TEAL, rx=3, extra=' fill-opacity="0.22"')
        s.text(x + (w / m - 4) / 2, 170, f'shows {shown}', 10, DIM, 'middle')
    s.add(f'<rect x="{x0}" y="86" width="2" height="98" fill="{AMBER}"><animate attributeName="x" values="{x0};{x0 + w}" '
          f'dur="5s" repeatCount="indefinite"/></rect>')
    # Lag frames.
    s.text(40, 238, 'a lag frame', 12, TEXT, weight='600')
    frames = ['1', '2', 'lag', '3', '4', '5']
    for i, f in enumerate(frames):
        lag = f == 'lag'
        s.rect(x0 + i * w / n, 222, w / n - 4, 26, RED if lag else BLUE, rx=3, extra=f' fill-opacity="{0.5 if lag else 0.3}"')
        s.text(x0 + i * w / n + 8, 240, 'no update' if lag else f'update {f}', 11, TEXT)
    para(s, 40, 286, 'The original took two frames for one update. Lag frames change the timing. A port that must match '
                     'speedruns reproduces them. Most ports do not. Record the decision. Pick one region as the reference.', 820)
    return s.render()


def input_mask():
    s = Svg(900, 300, 'The controller read: the buttons pressed this frame are the buttons held now that were not held '
                      'in the previous frame')
    s.heading('Input: held and pressed this frame', 'Many games keep last frame\'s buttons to find the ones that were just pressed.')
    names = ['A', 'B', 'Sel', 'Start', 'Up', 'Down', 'Left', 'Right']
    rows = [('previous frame', '10000010', GREY), ('this frame (held)', '11000100', BLUE),
            ('pressed = held AND NOT previous', '01000100', AMBER)]
    x0 = 320
    for k, n in enumerate(names):
        s.text(x0 + k * 62 + 29, 92, n, 11, DIM, 'middle')
    for r, (label, bitsv, c) in enumerate(rows):
        y = 104 + r * 46
        s.text(40, y + 22, label, 12, TEXT, weight='600')
        for k, b in enumerate(bitsv):
            on = b == '1'
            s.rect(x0 + k * 62, y, 58, 32, c if on else GREY, rx=3, extra=f' fill-opacity="{0.65 if on else 0.12}"')
            s.text(x0 + k * 62 + 29, y + 21, b, 13, TEXT if on else DIM, 'middle', '600', mono=True)
    para(s, 40, 260, 'The NES order of the eight buttons is shown. Log this RAM each frame as the input record. Find every read: '
                     'some games read the controller twice a frame or in an interrupt.', 820)
    return s.render()


def video_kinds():
    s = Svg(900, 330, 'Three kinds of console video and how a port draws each: tile and sprite chips, frame buffers '
                      'with triangles, and display lists for a programmable GPU')
    s.heading('Three kinds of console video', 'Each kind decides how the port draws the picture.')
    cols = [('Tiles, sprites, layers', 'NES to GBA, Mega Drive, Saturn VDP2', BLUE,
             ['Reimplement the video chip.', 'Each frame, read the tile,', 'map, sprite and palette', 'memory and draw it.', 'Exact output is possible.']),
            ('Frame buffer, triangles', 'PlayStation, Saturn VDP1', AMBER,
             ['Draw the game\'s commands', 'with a modern API.', 'Decide whether to keep', 'the wobble and the missing', 'perspective correction.']),
            ('Display lists', 'N64 and later consoles', PURPLE,
             ['Interpret the lists the', 'game builds and translate', 'them to a modern API.'])]
    for i, (t, sub, c, lines) in enumerate(cols):
        x = 40 + i * 280
        box(s, x, 84, 260, 56, c, t, [sub], 13, op=0.3)
        for k, l in enumerate(lines):
            s.text(x + 10, 166 + k * 18, l, 12, TEXT)
    # Small pictures of each.
    for k in range(6):
        s.rect(40 + k * 18, 270, 16, 16, BLUE, rx=1, extra=f' fill-opacity="{0.2 + 0.1 * (k % 3)}"')
    s.rect(160, 262, 24, 24, PINK, rx=2, extra=' fill-opacity="0.7"')
    s.add(f'<path d="M330 290L380 258L420 288Z" fill="{AMBER}" fill-opacity="0.4" stroke="{AMBER}"/>')
    s.add(f'<path d="M400 290L450 266L480 292Z" fill="{AMBER}" fill-opacity="0.25" stroke="{AMBER}"/>')
    for k, t in enumerate(('matrix', 'vertices', 'triangles', 'end')):
        s.rect(600 + k * 64, 266, 60, 20, PURPLE, rx=3, extra=' fill-opacity="0.3"')
        s.text(630 + k * 64, 280, t, 10, TEXT, 'middle')
    s.text(40, 318, 'Keep the original resolution and aspect ratio as the reference. Widescreen is a deviation to record.', 12, DIM)
    return s.render()


def raster_split():
    s = Svg(900, 340, 'A raster effect: the playfield scrolls while the status bar stays still, because an interrupt '
                      'on one scanline rewrites the scroll register while the beam is drawing')
    s.heading('Raster effects: registers changed mid-frame', 'An interrupt fires on a chosen line. The game writes a video register there.')
    x0, y0, w, h, split = 40, 84, 360, 230, 170
    s.add(f'<clipPath id="pf"><rect x="{x0}" y="{y0}" width="{w}" height="{split}"/></clipPath>')
    hills = ''.join(f'<rect x="{x0 + k * 40}" y="{y0 + 120 - (k * 37) % 70}" width="30" height="{50 + (k * 37) % 70}" '
                    f'fill="{GREEN}" fill-opacity="0.45"/>' for k in range(-2, 20))
    s.add(f'<g clip-path="url(#pf)"><rect x="{x0}" y="{y0}" width="{w}" height="{split}" fill="{BLUE}" fill-opacity="0.18"/>'
          f'<g>{hills}<animateTransform attributeName="transform" type="translate" values="0 0;-80 0" dur="2s" '
          'repeatCount="indefinite"/></g></g>')
    s.rect(x0, y0 + split, w, h - split, GREY, rx=0, extra=' fill-opacity="0.35"')
    s.text(x0 + 14, y0 + split + 26, 'SCORE 001200    LIVES 3', 13, TEXT, weight='600', mono=True)
    s.line(x0 - 6, y0 + split, x0 + w + 6, y0 + split, AMBER, 2, ' stroke-dasharray="5 3"')
    s.add(f'<rect x="{x0}" y="{y0}" width="{w}" height="2" fill="{TEXT}"><animate attributeName="y" values="{y0};{y0 + h - 2}" '
          'dur="1.6s" repeatCount="indefinite"/></rect>')
    tx = 440
    s.text(tx, 120, f'line {split}: the interrupt fires', 13, AMBER, weight='600')
    s.text(tx, 142, 'The handler sets the scroll back to 0. The lines below are drawn unscrolled.', 12, DIM)
    s.text(tx, 182, 'Line interrupts', 13, TEXT, weight='600')
    s.text(tx, 204, 'SNES: H-IRQ.  Game Boy: LYC.', 12, DIM, mono=True)
    para(s, tx, 244, 'Apply the register change at the same line in the port. Drawing the whole frame from the final '
                     'register values loses the effect.', 420)
    return s.render()


def sound_cpu():
    s = Svg(900, 300, 'Sound on a separate CPU: the game sends commands to a sound CPU that runs its own program and '
                      'drives the sound chip; a port emulates that CPU and chip or ports the sound program')
    s.heading('Sound with its own CPU', 'SNES: SPC700. Mega Drive: Z80. PS2: the IOP. The game carries the sound program.')
    box(s, 40, 96, 160, 60, BLUE, 'Main CPU', ['the game'])
    arrow(s, 202, 126, 266, 126)
    s.text(234, 116, 'commands', 10, DIM, 'middle')
    dot(s, 'M202 126L266 126', 1.0, AMBER, 4)
    box(s, 270, 96, 200, 60, PURPLE, 'Sound CPU', ['its own program', 'from the game'])
    arrow(s, 472, 126, 516, 126)
    box(s, 520, 96, 170, 60, AMBER, 'Sound chip', ['YM2612, S-DSP, APU'])
    arrow(s, 692, 126, 736, 126)
    box(s, 740, 96, 120, 60, GREY, 'Audio', [])
    box(s, 40, 200, 400, 60, GREEN, 'A: emulate the sound CPU and chip', ['usually less work, gives exact sound'])
    box(s, 460, 200, 400, 60, TEAL, 'B: trace and port the sound program', ['new code for the new system'])
    s.text(40, 286, 'Either way the program and samples come from the player\'s copy. Use an accurate chip emulator.', 12, DIM)
    return s.render()


def console_saves():
    s = Svg(900, 300, 'Three kinds of console save and how a port keeps each: battery RAM mapped to a file with the '
                      'same layout, memory card calls turned into file calls, and passwords encoded exactly')
    s.heading('Saves: keep the original format', 'Then saves move between the original, through an emulator, and the port.')
    rows = [('Battery RAM (SRAM)', 'in the cartridge', 'a file with the same bytes', BLUE),
            ('Memory card', 'PlayStation, PS2, GameCube', 'card API calls become file calls', PURPLE),
            ('Password', 'the state as text', 'the same encoding, exactly', AMBER)]
    for i, (t, sub, port, c) in enumerate(rows):
        y = 88 + i * 60
        box(s, 40, y, 300, 48, c, t, [sub], 12)
        arrow(s, 342, y + 24, 396, y + 24)
        box(s, 400, y, 460, 48, GREEN, port, [], 12)
    s.text(40, 284, 'Players still have old passwords and old save files.', 12, DIM)
    return s.render()


def port_check():
    s = Svg(900, 320, 'Checking the port: one input movie drives the original in an emulator and the port; both dump '
                      'RAM every frame, and the comparison stops at the first frame that differs')
    s.heading('Checking the port frame by frame', 'One input movie, two runs, one comparison.')
    box(s, 40, 120, 150, 60, AMBER, 'Input movie', ['per-frame input'])
    arrow(s, 192, 140, 246, 108)
    arrow(s, 192, 160, 246, 192)
    box(s, 250, 80, 180, 50, TEAL, 'Original', ['in its emulator'])
    box(s, 250, 170, 180, 50, BLUE, 'The port', [])
    x0 = 480
    for f in range(10):
        x = x0 + f * 38
        s.text(x + 17, 80, str(f), 10, DIM, 'middle', mono=True)
        bad = f >= 7
        for y, c in ((90, TEAL), (180, BLUE)):
            s.rect(x, y, 34, 30, RED if bad and c == BLUE else c, rx=3, extra=f' fill-opacity="{0.6 if bad and c == BLUE else 0.25}"')
        s.text(x + 17, 154, 'x' if bad else '=', 13, RED if bad else GREEN, 'middle', '600', mono=True)
    s.text(480, 236, 'RAM dumps of each frame', 11, DIM)
    s.add(f'<rect x="{x0 - 2}" y="86" width="38" height="128" rx="4" fill="none" stroke="{AMBER}" stroke-width="2">'
          f'<animateTransform attributeName="transform" type="translate" values="{";".join(f"{k * 38} 0" for k in range(8))}" '
          f'keyTimes="{";".join(f"{k / 8:.3f}" for k in range(8))}" calcMode="discrete" dur="5s" repeatCount="indefinite"/></rect>')
    s.text(40, 278, 'Stop at the first difference (frame 7 here) and fix it. Tile-based video can also be compared pixel for pixel.', 12, DIM)
    return s.render()


# ================================================================ docs/consoles/tools.md
def nes_memory_map():
    s = Svg(900, 420, 'The NES CPU memory map: 2 KB of RAM and its mirrors, the PPU registers from 2000, the APU and '
                      'I/O registers from 4000, cartridge RAM at 6000 and cartridge ROM from 8000 with the vectors at the top')
    s.heading('Set the memory map first: the NES', 'Give Ghidra each region and name the registers. Then the code shows PPUCTRL in place of 0x2000.')
    rows = [('$0000', 'RAM, 2 KB (mirrored to $1FFF)', 48, GREEN), ('$2000', 'PPU registers (mirrored to $3FFF)', 48, PURPLE),
            ('$4000', 'APU and I/O registers', 30, AMBER), ('$4020', 'cartridge expansion', 30, GREY),
            ('$6000', 'cartridge RAM (battery saves)', 40, TEAL), ('$8000', 'cartridge ROM (banked by the mapper)', 130, BLUE)]
    end = memmap(s, 40, 84, 320, rows, 0.85)
    s.text(92, end + 10, '$FFFF', 11, DIM, 'end', mono=True)
    s.text(460, 100, 'Named registers', 13, TEXT, weight='600')
    regs = [('$2000', 'PPUCTRL'), ('$2001', 'PPUMASK'), ('$2002', 'PPUSTATUS'), ('$2006', 'PPUADDR'), ('$2007', 'PPUDATA'),
            ('$4014', 'OAMDMA'), ('$4016', 'JOY1')]
    for i, (a, n) in enumerate(regs):
        s.text(460, 126 + i * 20, a, 12, DIM, mono=True)
        s.text(520, 126 + i * 20, n, 12, PURPLE if a < '$4000' else AMBER, mono=True)
    s.text(460, 290, 'Vectors at the top of ROM', 13, TEXT, weight='600')
    for i, (a, n) in enumerate((('$FFFA', 'NMI: the vertical blank'), ('$FFFC', 'reset'), ('$FFFE', 'IRQ and BRK'))):
        s.text(460, 314 + i * 20, a, 12, DIM, mono=True)
        s.text(520, 314 + i * 20, n, 12, BLUE)
    s.text(40, 404, 'Most console communities publish register lists. Load them as labels before analysis.', 12, DIM)
    return s.render()


def ghidra_adc():
    s = Svg(900, 330, 'Two cases where Ghidra\'s 6502 ADC p-code gives the wrong flags: signed overflow, which it copies '
                      'from the carry, and an incoming carry it leaves out of the carry test')
    s.heading('Why Ghidra\'s 6502 p-code is not ground truth', 'Its ADC has no decimal mode and computes two flags wrongly. Two worked cases follow.')
    hdr = ['', 'A + operand + C', 'result', 'real 6502', 'Ghidra p-code']
    xs = [40, 190, 390, 500, 680]
    for x, t in zip(xs, hdr):
        s.text(x, 98, t, 11, DIM)
    cases = [('signed overflow', '$50 + $50 + 0', '$A0', 'V = 1, C = 0', 'V = 0 (copied from C)'),
             ('incoming carry', '$FF + $00 + 1', '$00', 'C = 1', 'C = 0 (carry-in ignored)')]
    for i, (name, op, res, real, ghidra) in enumerate(cases):
        y = 112 + i * 56
        s.rect(30, y, 840, 44, GREY, rx=4, extra=' fill-opacity="0.1"')
        s.text(40, y + 27, name, 12, TEXT, weight='600')
        s.text(190, y + 27, op, 13, TEXT, mono=True)
        s.text(390, y + 27, res, 13, TEXT, mono=True)
        s.text(500, y + 27, real, 13, GREEN, mono=True)
        s.text(680, y + 27, ghidra, 12, RED, mono=True)
    para(s, 40, 254, '80 + 80 = 160 does not fit a signed byte (-128 to 127), so the real V flag is set. Take 6502 '
                     'ground truth from a tested core: py65, mre/mos6502, floooh/chips or MAME.', 820)
    return s.render()


def cpu_support():
    s = Svg(900, 520, 'Console CPUs and their support in Ghidra and in Unicorn: built in, by extension or none, as the '
                      'tools page lists them')
    s.heading('Console CPUs: Ghidra and Unicorn', 'Without a Unicorn backend, take ground truth from an emulator or a small core.')
    rows = [('6502, 65C02', 'built in', 'no'), ('HuC6280', 'extension', 'no'), ('CP1600', 'built in', 'no'), ('6809', 'built in', 'no'),
            ('65C816', 'extension', 'no'), ('Z80', 'built in', 'no'), ('SM83', 'extension', 'no'), ('68000', 'built in', 'yes'),
            ('MIPS R3000, R4300', 'built in', 'yes'), ('MIPS R5900', 'extension', 'no'), ('SuperH', 'built in', 'no'),
            ('PowerPC', 'built in', 'yes'), ('ARM7', 'built in', 'yes'), ('x86, V30MZ', 'built in', 'yes'),
            ('V810', 'extension', '-'), ('TLCS-900/H', 'none', 'no'), ('Allegrex', 'extension', '-'), ('Cell SPU', 'extension', '-'),
            ('Jaguar RISC', 'none', '-'), ('ARMv3', 'no exact match', '-')]
    colour = {'built in': GREEN, 'extension': AMBER, 'none': RED, 'no exact match': RED, 'yes': GREEN, 'no': RED, '-': GREY}
    for i, (cpu, g, u) in enumerate(rows):
        col, r = divmod(i, 10)
        x, y = 40 + col * 430, 94 + r * 38
        if r == 0:
            s.text(x, 84, 'CPU', 11, DIM)
            s.text(x + 160, 84, 'Ghidra', 11, DIM)
            s.text(x + 290, 84, 'Unicorn', 11, DIM)
        s.text(x, y + 19, cpu, 12, TEXT, weight='600')
        s.rect(x + 160, y + 4, 120, 22, colour[g], rx=3, extra=' fill-opacity="0.25"')
        s.text(x + 166, y + 19, g, 11, TEXT)
        s.rect(x + 290, y + 4, 100, 22, colour[u], rx=3, extra=' fill-opacity="0.25"')
        s.text(x + 296, y + 19, u if u != '-' else 'not stated', 11, TEXT if u != '-' else DIM)
    s.text(40, 484, 'Unicorn runs ARM, ARM64, M68K, MIPS, PowerPC and x86. It has no paired singles, R5900 or VMX128.', 12, DIM)
    s.text(40, 504, 'It does not emulate coprocessors such as the GTE or the PS2 vector units.', 12, DIM)
    return s.render()


def ram_search():
    dur = 6
    s = Svg(900, 300, 'RAM search: start with every address, keep those equal to 3 while the player has 3 lives, then '
                      'those equal to 2 after a life is lost, until one address is left')
    s.heading('RAM search finds a variable', 'Filter the candidates each time the value you know changes.')
    steps = [('all of RAM', None), ('equal to 3 (3 lives)', 3), ('now equal to 2', 2), ('one address left: lives', 1)]
    keep = [set(range(64)), {3, 9, 17, 22, 30, 41, 46, 52, 60}, {17, 41, 52}, {41}]
    for k, (label, _) in enumerate(steps):
        def draw(t, k=k, label=label):
            for c in range(64):
                x, y = 40 + (c % 16) * 30, 96 + (c // 16) * 30
                on = c in keep[k]
                fill = AMBER if (on and k == 3) else (BLUE if on else GREY)
                t.rect(x, y, 26, 26, fill, rx=3, extra=f' fill-opacity="{0.7 if on else 0.08}"')
            t.text(560, 130, f'step {k + 1}', 12, DIM)
            t.text(560, 154, label, 14, AMBER if k == 3 else TEXT, weight='600')
            t.text(560, 178, f'{len(keep[k])} candidates' if k < 3 else 'name it in labels/', 12, DIM)
        s.add(group(part(draw), 4, k, dur))
    s.text(40, 250, 'Then put a breakpoint on writes to that address to find the code that changes it.', 12, DIM)
    s.text(40, 270, 'A small grid stands for the whole RAM here.', 11, DIM)
    return s.render()


def unicorn_run():
    s = Svg(900, 300, 'Running one console function under Unicorn: load the code at its address, fill RAM from a dump, '
                      'set the registers, stub the hardware registers, call it and read the results')
    s.heading('One original function, headless', 'For the CPUs Unicorn supports, the same method as pe_emu.py for Windows code.')
    steps = [('1. Load code', ['ROM or segments', 'at their addresses'], BLUE), ('2. Map RAM', ['fill it from an', 'emulator dump'], GREEN),
             ('3. Set registers', ['the function\'s', 'arguments'], PURPLE), ('4. Stub hardware', ['the registers', 'it touches'], AMBER),
             ('5. Call, read', ['the results:', 'ground truth'], TEAL)]
    flow(s, 30, 100, 152, 76, 20, steps)
    para(s, 40, 220, 'Compare the results with the port\'s function on many inputs. Coprocessors such as the GTE and '
                     'the PS2 vector units need a full emulator instead.', 820)
    return s.render()


def splat_flow():
    s = Svg(900, 280, 'Splitting and rebuilding: splat or decomp-toolkit split the ROM by a configuration into code, data '
                      'and assets, and the project builds them back into an identical ROM')
    s.heading('Split, then rebuild the same bytes', 'The first build is all disassembly. It must already give the identical program.')
    box(s, 40, 100, 140, 70, GREY, 'ROM or EXE', ['the original'])
    arrow(s, 182, 135, 222, 135)
    box(s, 226, 100, 160, 70, BLUE, 'splat, dtk', ['a config of', 'addresses'])
    for i, (t, c) in enumerate((('code: assembly', PURPLE), ('data', AMBER), ('assets', TEAL))):
        y = 84 + i * 36
        arrow(s, 388, 135, 436, y + 15)
        box(s, 440, y, 150, 30, c, t, [], 12)
    for i in range(3):
        arrow(s, 592, 99 + i * 36, 636, 135)
    box(s, 640, 100, 100, 70, GREEN, 'build', [])
    arrow(s, 742, 135, 772, 135)
    box(s, 776, 100, 94, 70, GREEN, 'same', ['hash'])
    s.text(40, 230, 'Then C replaces the assembly one function at a time. The m2c tool drafts each function.', 12, DIM)
    s.text(40, 250, 'The decomp.me site, asm-differ and objdiff (GameCube and Wii) check it against the original.', 12, DIM)
    return s.render()


# ================================================================ docs/handhelds/README.md
def handheld_groups():
    s = Svg(900, 330, 'The four groups of handhelds: 4-bit LCD games with the program in the CPU, 8-bit cartridge '
                      'systems, ARM machines with an operating system, and modern ARM64 and x86-64 machines')
    s.heading('Handhelds in four groups', 'The group decides where the program lives and which tools read it.')
    groups = [('4-bit LCD games', '1979 on', ['mask ROM inside the MCU', 'lit LCD segments', 'Game & Watch, Microvision'], GREY),
              ('8-bit and 16-bit carts', '1989 on', ['ROM in a cartridge', 'tiles and sprites', 'Game Boy, Lynx, Game Gear'], TEAL),
              ('ARM on an OS', '2001 on', ['files on cards or carts', 'OS calls to stub', 'GBA, DS, N-Gage, PSP'], BLUE),
              ('ARM64 and x86-64', '2017 on', ['encrypted containers', 'or PC games', 'Switch, Steam Deck'], PURPLE)]
    for i, (t, yrs, lines, c) in enumerate(groups):
        x = 40 + i * 210
        box(s, x, 90, 190, 60, c, t, [yrs], 13, op=0.3)
        for k, l in enumerate(lines):
            s.text(x + 8, 176 + k * 20, l, 12, TEXT if k < 2 else DIM)
        if i:
            arrow(s, x - 18, 120, x - 3, 120)
    para(s, 40, 268, 'In every group after the first, games come on removable media. The first group needs a decap or '
                     'a MAME set to read the program.', 820)
    return s.render()


# ================================================================ docs/handhelds/practices.md
def mbc3_rtc():
    s = Svg(900, 360, 'The Game Boy MBC3 clock: writing 08 to 0C into 4000 to 5FFF selects a clock register, writing 00 '
                      'then 01 into 6000 to 7FFF latches the time, and the DH register holds day bit 8, halt and carry')
    s.heading('MBC3: a real-time clock in the cartridge', 'A 32.768 kHz crystal and a battery keep it running with the power off.')
    regs = [('$08', 'seconds'), ('$09', 'minutes'), ('$0A', 'hours'), ('$0B', 'day, low 8 bits'), ('$0C', 'DH')]
    s.text(40, 96, 'write to $4000-$5FFF to select', 12, TEXT, weight='600')
    for i, (v, n) in enumerate(regs):
        s.rect(40, 108 + i * 30, 230, 26, AMBER if v == '$0C' else BLUE, rx=3, extra=' fill-opacity="0.25"')
        s.text(50, 126 + i * 30, v, 12, TEXT, mono=True)
        s.text(96, 126 + i * 30, n, 12, TEXT)
    s.text(330, 96, 'DH, the clock\'s top register', 12, TEXT, weight='600')
    bitrow(s, 330, 108, [('carry', 1, RED), ('halt', 1, AMBER), ('unused', 5, GREY), ('day 8', 1, BLUE)], 66)
    for k in range(8):
        s.text(330 + k * 66 + 32, 154, str(7 - k), 10, DIM, 'middle', mono=True)
    para(s, 330, 180, 'The day counter is 9 bits, 0 to 511. Carry stays set until software clears it.', 520)
    s.text(330, 230, 'latch: write $00, then $01, to $6000-$7FFF', 12, TEXT, weight='600')
    box(s, 330, 242, 110, 34, GREY, '$00', [], 12)
    arrow(s, 442, 259, 470, 259)
    box(s, 474, 242, 110, 34, GREY, '$01', [], 12)
    arrow(s, 586, 259, 614, 259)
    box(s, 618, 242, 240, 34, TEAL, 'the time is frozen for reading', [], 12)
    s.text(40, 330, 'A save file needs the clock state beside the RAM. That is the latched values, the halt flag and the carry flag.', 12, DIM)
    return s.render()


def outside_inputs():
    s = Svg(900, 300, 'Outside inputs a handheld test records and replays each frame beside the buttons: link cables, '
                      'infrared, the real-time clock, sensors, touch and cartridge presence')
    s.heading('Outside inputs to record', 'A test replays all of them, frame by frame, beside the buttons.')
    ins = [('buttons', BLUE), ('link cable', TEAL), ('infrared', TEAL), ('real-time clock', AMBER), ('shock, tilt', PURPLE),
           ('touchscreen', PURPLE), ('GPS, camera', PURPLE), ('Game-Key Card', GREY), ('mouse (Joy-Con 2)', GREY)]
    for i, (t, c) in enumerate(ins):
        x, y = 40 + (i % 3) * 150, 90 + (i // 3) * 54
        box(s, x, y, 136, 40, c, t, [], 12, op=0.3)
    s.rect(30, 80, 466, 170, 'none', GREY, rx=8, extra=' stroke-dasharray="5 4"')
    arrow(s, 498, 165, 520, 165)
    box(s, 524, 120, 160, 90, GREEN, 'Input record', ['one entry per frame'])
    arrow(s, 686, 165, 726, 165)
    box(s, 730, 120, 130, 90, TEAL, 'Replay', ['the original', 'and the port'])
    s.text(40, 270, 'Examples: GBC infrared, the Pokémon mini shock sensor, the Game.com touchscreen, the Gizmondo GPS.', 12, DIM)
    return s.render()


def lcd_ghosting():
    s = Svg(900, 300, 'Sprite flicker and a slow LCD: the game draws two sprites on alternate frames and the slow LCD '
                      'blends them; tests compare the raw frames and leave blending to the presentation')
    s.heading('LCD behaviour: compare the raw frames', 'Some games flicker sprites and rely on the slow LCD to blend them.')
    for k, (label, ax, bx) in enumerate((('frame 1: sprite A', 1, 0), ('frame 2: sprite B', 0, 1))):
        x = 40 + k * 200
        s.rect(x, 96, 170, 130, GREEN, rx=4, extra=' fill-opacity="0.12"')
        s.rect(x + 30, 140, 40, 40, TEXT, rx=3, extra=f' fill-opacity="{0.9 * ax}"')
        s.rect(x + 100, 140, 40, 40, TEXT, rx=3, extra=f' fill-opacity="{0.9 * bx}"')
        s.text(x, 246, label, 12, TEXT)
    # What the eye sees on the LCD.
    x = 480
    s.rect(x, 96, 170, 130, GREEN, rx=4, extra=' fill-opacity="0.12"')
    for dx, vals in ((30, '0.9;0.25;0.9'), (100, '0.25;0.9;0.25')):
        s.add(f'<rect x="{x + dx}" y="140" width="40" height="40" rx="3" fill="{TEXT}" fill-opacity="0.6">'
              f'<animate attributeName="fill-opacity" values="{vals}" dur="0.5s" repeatCount="indefinite"/></rect>')
    s.text(x, 246, 'on the LCD: both, half bright', 12, TEXT)
    para(s, 690, 110, 'The test compares frame 1 and frame 2 as drawn. Blend only in the presentation layer.', 180)
    s.text(40, 284, 'On a Game & Watch, the test compares which LCD segments are lit.', 12, DIM)
    return s.render()


# ================================================================ docs/handhelds/game-boy.md
def gb_memory_map():
    dur = 6
    s = Svg(900, 420, 'The Game Boy memory map: ROM bank 0 at 0000, a switchable ROM bank at 4000, VRAM, cartridge RAM, '
                      'work RAM, sprite attributes, I/O registers and high RAM')
    s.heading('Game Boy: the memory map', 'The mapper changes which ROM bank sits at $4000-$7FFF. Name functions bank:address.')
    rows = [('$0000', 'ROM bank 0 (fixed)', 54, TEAL), ('$4000', 'ROM bank n (switchable)', 54, AMBER), ('$8000', 'VRAM: tiles and maps', 30, PURPLE),
            ('$A000', 'cartridge RAM (battery)', 30, GREEN), ('$C000', 'work RAM', 30, GREY), ('$E000', 'echo of work RAM', 22, GREY),
            ('$FE00', 'OAM: 40 sprites', 22, PURPLE), ('$FF00', 'I/O registers', 22, BLUE), ('$FF80', 'high RAM', 22, GREY)]
    end = memmap(s, 40, 84, 280, rows)
    s.text(92, end + 14, '$FFFF', 11, DIM, 'end', mono=True)
    s.text(420, 96, 'the cartridge ROM, 16 KB banks', 12, TEXT, weight='600')
    for b in range(8):
        x = 420 + b * 54
        s.rect(x, 108, 48, 80, TEAL if b == 0 else BLUE, rx=3, extra=' fill-opacity="0.18"')
    for k, b in enumerate((1, 4, 6)):
        x = 420 + b * 54
        s.add(f'<g opacity="{1 if k == 0 else 0}"><rect x="{x}" y="108" width="48" height="80" rx="3" fill="{AMBER}" '
              f'fill-opacity="0.75"/><path d="M380 165C 400 228 {x + 24} 228 {x + 24} 192" fill="none" stroke="{AMBER}" '
              f'stroke-width="2" marker-end="url(#arrow)"/><text x="420" y="250" font-family="{MONO}" font-size="12" '
              f'fill="{AMBER}">write {b} to $2000: bank {b} at $4000</text>{phase(3, k, dur)}</g>')
    for b in range(8):
        s.text(420 + b * 54 + 24, 180, str(b), 11, TEXT, 'middle', mono=True)
    para(s, 420, 290, 'The write goes to ROM space, where the mapper catches it. The CPU is a Sharp SM83, close to the Z80 '
                      'and the 8080. Unicorn has none, so take ground truth from SameBoy.', 440)
    return s.render()


def gbc_modes():
    s = Svg(900, 330, 'Game Boy Color modes: the header byte at 0143 decides whether a cartridge runs in colour, in '
                      'both modes, or in the monochrome mode; double speed doubles the CPU clock')
    s.heading('Game Boy Color: the header byte picks the mode', 'The boot ROM reads $0143 and sets the mode before the game starts.')
    rows = [('$80', 'supports both systems', 'black "dual mode" cartridge', TEAL), ('$C0', 'CGB only', 'clear cartridge', BLUE),
            ('other', 'monochrome "Non CGB" mode', 'grey DMG cartridge', GREY)]
    for i, (v, mode, cart, c) in enumerate(rows):
        y = 90 + i * 50
        box(s, 40, y, 90, 40, AMBER, v, [], 13, op=0.3)
        arrow(s, 132, y + 20, 172, y + 20)
        box(s, 176, y, 260, 40, c, mode, [], 12)
        s.text(452, y + 25, cart, 12, DIM)
    s.text(40, 266, 'Speed', 13, TEXT, weight='600')
    bitrow(s, 120, 250, [('normal: 4.194304 MHz', 10, BLUE)], 22, 26)
    bitrow(s, 120, 282, [('double: 8.388608 MHz', 20, AMBER)], 22, 26)
    s.text(580, 270, 'A dual-mode game has two code paths.', 12, DIM)
    s.text(580, 300, 'A port needs both.', 12, DIM)
    return s.render()


def gbc_hdma():
    s = Svg(900, 360, 'Game Boy Color VRAM DMA: general-purpose DMA copies everything at once and halts the CPU, '
                      'HBlank DMA copies 16 bytes in each horizontal blank while the game runs')
    s.heading('CGB VRAM DMA: two modes', 'HDMA5 bit 7 picks the mode. Each moves $10 to $800 bytes into VRAM.')
    # General purpose.
    s.text(40, 98, 'bit 7 clear: general-purpose DMA', 12, TEXT, weight='600')
    s.rect(40, 110, 520, 28, BLUE, rx=3, extra=' fill-opacity="0.2"')
    s.add(f'<rect x="40" y="110" width="0" height="28" rx="3" fill="{AMBER}" fill-opacity="0.7"><animate attributeName="width" '
          'values="0;520;520" keyTimes="0;0.25;1" dur="4s" repeatCount="indefinite"/></rect>')
    s.text(580, 129, 'all at once, with the CPU halted', 12, DIM)
    # HBlank.
    s.text(40, 178, 'bit 7 set: HBlank DMA, $10 bytes per line', 12, TEXT, weight='600')
    n = 13
    for i in range(n):
        x = 40 + i * 40
        s.rect(x, 190, 30, 28, BLUE, rx=2, extra=' fill-opacity="0.2"')
        s.rect(x + 30, 190, 8, 28, AMBER, rx=1, extra=' fill-opacity="0.25"')
        s.add(f'<rect x="{x + 30}" y="190" width="8" height="28" rx="1" fill="{AMBER}" opacity="0">'
              f'<animate attributeName="opacity" values="0;0;1;1" keyTimes="0;{i / n:.3f};{i / n:.3f};1" dur="4s" '
              'repeatCount="indefinite"/></rect>')
    s.text(40, 236, 'Blue is a line being drawn while the game runs. Amber is one block copied in its HBlank.', 11, DIM)
    s.text(40, 274, 'Double speed', 13, TEXT, weight='600')
    s.text(40, 296, 'speeds up: the CPU, timers, serial port, OAM DMA', 12, AMBER)
    s.text(40, 316, 'stays the same: the LCD, VRAM DMA, sound', 12, TEAL)
    s.text(420, 296, 'KEY1 ($FF4D): set bit 0, execute stop.', 12, DIM, mono=False)
    s.text(420, 316, 'The switch pauses the CPU for 2050 M-cycles.', 12, DIM)
    return s.render()


def sgb_layout():
    s = Svg(900, 380, 'The Super Game Boy picture: the 160 by 144 Game Boy screen inside a 256 by 224 SNES picture with a '
                      'border, palettes per cell of a 20 by 18 grid, and a clock taken from the SNES that runs fast')
    s.heading('Super Game Boy: a Game Boy inside a SNES', 'Drawn to scale. The border is SNES tiles. Palettes can change per cell.')
    k = 1.2
    x0, y0 = 40, 84
    s.rect(x0, y0, 256 * k, 224 * k, PURPLE, PURPLE, rx=3, extra=' fill-opacity="0.2" stroke-opacity="0.7"')
    s.text(x0 + 8, y0 + 16, 'SNES picture 256 x 224, border in 4 bpp tiles', 11, TEXT)
    gx, gy = x0 + 48 * k, y0 + 40 * k
    s.rect(gx, gy, 160 * k, 144 * k, GREEN, rx=1, extra=' fill-opacity="0.2"')
    for c in range(21):
        s.line(gx + c * 8 * k, gy, gx + c * 8 * k, gy + 144 * k, GREEN, 0.4, ' stroke-opacity="0.5"')
    for r in range(19):
        s.line(gx, gy + r * 8 * k, gx + 160 * k, gy + r * 8 * k, GREEN, 0.4, ' stroke-opacity="0.5"')
    s.rect(gx + 5 * 8 * k, gy + 6 * 8 * k, 4 * 8 * k, 3 * 8 * k, AMBER, rx=1, extra=' fill-opacity="0.5"')
    s.text(gx + 4, gy + 144 * k - 6, 'Game Boy 160 x 144: a 20 x 18 grid of cells', 10, TEXT)
    tx = 400
    s.text(tx, 104, 'What a game can add', 13, TEXT, weight='600')
    for i, t in enumerate(('a palette per cell (amber: one area)', 'up to 24 SNES sprites', 'four joypads, read separately',
                           'SNES sound effects', 'code uploaded and run on the SNES CPU')):
        s.text(tx, 128 + i * 20, t, 12, TEXT)
    s.text(tx, 248, 'The clock comes from the SNES', 13, TEXT, weight='600')
    for i, (t, pct) in enumerate((('NTSC: 4.2955 MHz', 2.41), ('PAL: 4.2563 MHz', 1.48), ('SGB2: correct speed', 0))):
        y = 262 + i * 30
        s.rect(tx, y, 300, 22, GREY, rx=3, extra=' fill-opacity="0.2"')
        s.rect(tx, y, 300 + 300 * pct / 20, 22, AMBER if pct else TEAL, rx=3, extra=' fill-opacity="0.35"')
        s.text(tx + 8, y + 15, t, 11, TEXT)
        s.text(tx + 380, y + 15, f'+{pct}%' if pct else 'own crystal', 11, DIM, 'end')
    s.text(tx, 362, 'Bars exaggerate the difference 20 times.', 10, DIM)
    return s.render()


def gba_blocks():
    s = Svg(900, 280, 'The Game Boy Advance: an ARM7TDMI runs GBA games and an SM83 runs Game Boy games, with tile and '
                      'bitmap video modes and four Game Boy sound channels plus two sample channels')
    s.heading('Game Boy Advance: two CPUs, one at a time', 'The ARM7TDMI runs GBA games. The SM83 is there for Game Boy games.')
    box(s, 40, 90, 200, 70, BLUE, 'ARM7TDMI', ['ARM and Thumb', 'GBA games'])
    box(s, 40, 176, 200, 60, GREY, 'SM83', ['Game Boy games'])
    box(s, 300, 90, 260, 70, PURPLE, 'Video', ['tile modes and', 'bitmap modes'])
    box(s, 300, 176, 260, 60, AMBER, 'Sound', ['4 Game Boy channels + 2 sample'])
    box(s, 620, 90, 240, 146, GREEN, 'Tools', ['Ghidra: ARM built in', 'gba-ghidra-loader', 'Unicorn ARM runs', 'single functions',
                                              'mGBA, no$gba'])
    s.text(40, 266, 'GBATEK is the community document for the GBA and the DS.', 12, DIM)
    return s.render()


# ================================================================ docs/handhelds/lcd-games.md
def gw_segments():
    s = Svg(900, 330, 'A Game & Watch: the 4-bit MCU holds the ROM, the RAM and the LCD driver; bits in its LCD RAM '
                      'light fixed printed segments, and a test compares which segments are lit each frame')
    s.heading('Game & Watch: lit segments', 'Every image is a fixed printed segment. The program turns segments on and off.')
    box(s, 40, 90, 260, 190, GREY, '', op=0.08)
    s.text(56, 112, 'Sharp SM5xx MCU, 4-bit', 13, TEXT, weight='600')
    box(s, 56, 124, 110, 50, BLUE, 'mask ROM', ['1,792 to 4,032 B'])
    box(s, 176, 124, 110, 50, GREEN, 'RAM', ['80-128 nibbles'])
    box(s, 56, 186, 230, 80, AMBER, '', op=0.2)
    s.text(171, 208, 'LCD driver and LCD RAM', 13, TEXT, 'middle', '600')
    s.text(171, 226, 'one bit per segment', 11, DIM, 'middle')
    bits = '1010011010'
    for i, b in enumerate(bits):
        s.rect(80 + i * 18, 240, 14, 12, AMBER if b == '1' else GREY, rx=1, extra=' fill-opacity="0.8"')
    arrow(s, 302, 200, 356, 200)
    # A scene of segments: a figure in three positions, one lit at a time.
    s.rect(360, 90, 300, 180, GREEN, rx=6, extra=' fill-opacity="0.08"')
    for k, x in enumerate((400, 490, 580)):
        fig = (f'<g><circle cx="{x}" cy="150" r="12" fill="{TEXT}"/><rect x="{x - 10}" y="166" width="20" height="40" rx="4" '
               f'fill="{TEXT}"/><rect x="{x - 14}" y="210" width="8" height="30" fill="{TEXT}"/><rect x="{x + 6}" y="210" '
               f'width="8" height="30" fill="{TEXT}"/></g>')
        s.add(f'<g opacity="0.08">{fig}</g>')
        s.add(f'<g opacity="{1 if k == 0 else 0}">{fig}{phase(3, k, 2.4)}</g>')
    s.text(510, 290, 'one position lit at a time, the others faint', 11, DIM, 'middle')
    para(s, 690, 110, 'The test compares the set of lit segments each frame. A pixel comparison against MAME\'s SVG '
                      'adds nothing.', 180)
    return s.render()


def microvision():
    s = Svg(900, 260, 'The Microvision: the base unit has the LCD, keypad and paddle but no CPU; each cartridge holds its '
                      'own processor with the program masked inside it')
    s.heading('Microvision: the CPU is in the cartridge', 'The cartridge connector gives no normal ROM dump. The program is inside the chip.')
    box(s, 40, 90, 360, 130, GREY, '', op=0.1)
    s.text(56, 112, 'Base unit: no CPU', 13, TEXT, weight='600')
    box(s, 56, 126, 110, 80, GREEN, 'LCD', ['16 x 16 pixels'])
    box(s, 176, 126, 100, 80, BLUE, 'keypad', ['12 buttons'])
    box(s, 286, 126, 100, 80, BLUE, 'paddle', ['one'])
    box(s, 480, 90, 380, 130, AMBER, '', op=0.1)
    s.text(496, 112, 'Cartridge', 13, TEXT, weight='600')
    box(s, 496, 126, 170, 80, AMBER, 'Intel 8021', ['1K ROM inside', '100 kHz'])
    box(s, 676, 126, 170, 80, AMBER, 'TI TMS1100', ['2K ROM inside', '100 kHz'])
    s.line(402, 166, 478, 166, DIM, 2)
    s.text(40, 246, 'Take the program from a decap or a MAME set. A static discharge can destroy an early cartridge\'s CPU and program.', 12, DIM)
    return s.render()


# ================================================================ docs/handhelds/sega-atari-snk.md and others
def handheld_screens():
    s = Svg(900, 470, 'Handheld screens drawn to scale, from the Pokemon mini at 96 by 64 to the Virtual Boy at 384 by '
                      '224 for each eye')
    s.heading('Handheld screens, to scale', 'Pixels of the picture each machine draws. The Game Gear shows part of a Master System picture.')
    k = 0.75
    items = [('Pokémon mini', 96, 64, GREY), ('Lynx', 160, 102, AMBER), ('Game Boy, GBC', 160, 144, GREEN),
             ('Game Gear', 160, 144, BLUE), ('Neo Geo Pocket', 160, 152, PURPLE), ('Game.com', 200, 160, TEAL),
             ('WonderSwan', 224, 144, PINK), ('Virtual Boy, each eye', 384, 224, RED)]
    x, y, rowh = 40, 84, 0
    for name, w, h, c in items:
        if x + w * k > 860:
            x, y = 40, y + rowh + 56
            rowh = 0
        s.rect(x, y, w * k, h * k, c, c, rx=2, extra=' fill-opacity="0.15" stroke-opacity="0.8"')
        s.text(x, y + h * k + 16, name, 11, TEXT, weight='600')
        s.text(x, y + h * k + 32, f'{w} x {h}', 11, DIM, mono=True)
        x += max(w * k, len(name) * 6.4) + 24
        rowh = max(rowh, h * k)
    return s.render()


def lynx_blocks():
    s = Svg(900, 280, 'The Atari Lynx: Mikey holds the 65SC02 at 4 MHz, Suzy is a blitter that scales sprites with a '
                      'maths unit, and the boot ROM decrypts the cartridge loader')
    s.heading('Atari Lynx: Mikey and Suzy', 'The CPU sits inside Mikey. Suzy draws and does maths.')
    box(s, 40, 90, 240, 80, BLUE, 'Mikey', ['65SC02 at 4 MHz', 'timers, sound, display'])
    box(s, 320, 90, 240, 80, PURPLE, 'Suzy', ['blitter: scaled sprites', 'maths unit'])
    s.line(282, 130, 318, 130, DIM, 1.5)
    box(s, 40, 196, 160, 56, GREY, 'Cartridge', ['encrypted loader'])
    arrow(s, 202, 224, 246, 224)
    box(s, 250, 196, 160, 56, AMBER, 'Boot ROM', ['decrypts it'])
    arrow(s, 412, 224, 456, 224)
    box(s, 460, 196, 100, 56, TEAL, 'Game', ['runs'])
    para(s, 600, 100, 'Load the code with Ghidra\'s 65C02 language. Take ground truth from a tested 6502 core, because '
                      'Ghidra\'s ADC p-code is not exact. An emulator needs the boot ROM.', 270)
    return s.render()


def wonderswan_path():
    s = Svg(900, 250, 'The WonderSwan path: the V30MZ is a 16-bit x86, so Ghidra\'s x86 real-mode language reads it and '
                      'Unicorn runs its functions, as for DOS code')
    s.heading('WonderSwan: x86 tools apply', 'The NEC V30MZ at 3.072 MHz runs 16-bit x86 code.')
    steps = [('Cartridge', ['V30MZ code'], GREY), ('Ghidra', ['x86 real mode', '+ ghidra-wonderswan'], BLUE),
             ('Unicorn', ['x86 16-bit', 'one function'], PURPLE), ('Ground truth', ['as for DOS games'], TEAL)]
    flow(s, 40, 96, 186, 70, 26, steps)
    s.text(40, 206, 'The extension has few users. Test it on a known binary first.', 12, DIM)
    return s.render()


def virtualboy_eyes():
    s = Svg(900, 260, 'The Virtual Boy: the V810 at 20 MHz drives two red LED displays of 384 by 224 pixels, one for '
                      'each eye')
    s.heading('Virtual Boy: one picture per eye', 'Two red displays, 384 x 224 each. A test compares both pictures.')
    k = 0.55
    for i, eye in enumerate(('left eye', 'right eye')):
        x = 40 + i * (384 * k + 30)
        s.rect(x, 90, 384 * k, 224 * k, RED, RED, rx=3, extra=' fill-opacity="0.15"')
        s.text(x + 8, 108, eye, 11, TEXT, weight='600')
        s.add(f'<rect x="{x + 70 + i * 8}" y="140" width="50" height="30" fill="{RED}" fill-opacity="0.7"/>')
    s.text(40, 238, 'The object is drawn a little apart in each eye to give depth.', 12, DIM)
    box(s, 520, 106, 340, 80, BLUE, 'NEC V810, 20 MHz', ['Ghidra\'s V850 differs:', 'use Ghidra_v810_v830'])
    return s.render()


def pokemini_blocks():
    s = Svg(900, 300, 'The Pokemon mini: an Epson S1C88 at 4 MHz with 4 KB of SRAM shared with video, a 4 KB BIOS, '
                      'cartridges up to 2 MB, a 96 by 64 LCD at 30 frames a second, and sensors')
    s.heading('Pokémon mini: small, with outside inputs', 'An 8-bit core with 16-bit instructions. No Ghidra module, no Unicorn.')
    box(s, 40, 90, 180, 70, BLUE, 'Epson S1C88', ['4 MHz'])
    box(s, 240, 90, 180, 70, GREEN, 'SRAM 4 KB', ['shared with video'])
    box(s, 440, 90, 180, 70, GREY, 'BIOS 4 KB', ['start-up, power'])
    box(s, 640, 90, 220, 70, AMBER, 'Cartridge', ['up to 2 MB', 'every game under 500 KB'])
    screen(s, 40, 186, 96 * 1.2, 64 * 1.2, 'LCD', TEAL, '96 x 64, 30 fps')
    for i, t in enumerate(('shock sensor', 'infrared, 6 players', 'rumble', 'real-time clock')):
        box(s, 200 + i * 168, 196, 150, 40, PURPLE, t, [], 12)
    s.text(200, 268, 'Record each of these in a test. Take ground truth from PokeMini\'s trace.', 12, DIM)
    return s.render()


def gamecom_touch():
    s = Svg(900, 300, 'The Game.com screen: 200 by 160 pixels in 4 grey levels, with a touchscreen of 12 by 10 cells over '
                      'it')
    s.heading('Game.com: a coarse touchscreen', 'A Sharp SM8521 at 10 MHz. The touch grid is much coarser than the pixels.')
    k = 1.2
    x0, y0 = 40, 84
    greys = ['#1b2a28', '#4c6461', '#8fa9a5', '#d8e4e1']
    for i, g in enumerate(greys):
        s.rect(x0 + i * 50 * k, y0, 50 * k, 160 * k, g, rx=0)
    s.rect(x0, y0, 200 * k, 160 * k, 'none', GREY, rx=2)
    for c in range(13):
        s.line(x0 + c * 200 * k / 12, y0, x0 + c * 200 * k / 12, y0 + 160 * k, AMBER, 0.8, ' stroke-opacity="0.7"')
    for r in range(11):
        s.line(x0, y0 + r * 160 * k / 10, x0 + 200 * k, y0 + r * 160 * k / 10, AMBER, 0.8, ' stroke-opacity="0.7"')
    s.add(f'<rect x="{x0 + 7 * 200 * k / 12}" y="{y0 + 4 * 16 * k}" width="{200 * k / 12}" height="{16 * k}" fill="{AMBER}" '
          'fill-opacity="0.8"><animate attributeName="opacity" values="1;0.2;1" dur="1.2s" repeatCount="indefinite"/></rect>')
    tx = 320
    s.text(tx, 104, 'screen: 200 x 160, 4 grey levels', 12, TEXT, weight='600')
    s.text(tx, 126, 'touch: 12 x 10 cells (amber lines)', 12, AMBER, weight='600')
    para(s, tx, 162, 'A touch reports a cell, about 17 x 16 pixels. Record the touched cell in the input record each '
                     'frame. The modem cartridge is another outside input.', 540)
    return s.render()


# ================================================================ docs/handhelds/ds-and-3ds.md
def ds_cartridge():
    s = Svg(900, 300, 'A Nintendo DS cartridge holds an ARM9 binary, an ARM7 binary, overlays and the NitroFS file '
                      'system; load the ARM9 and ARM7 code into Ghidra as separate programs')
    s.heading('Nintendo DS: one game, two CPUs', 'The ARM946E-S at 67 MHz and the ARM7TDMI at 33 MHz each run their own binary.')
    parts = [('header', 70, GREY), ('ARM9 binary', 170, BLUE), ('ARM7 binary', 120, PURPLE), ('overlays', 140, AMBER),
             ('NitroFS files', 260, TEAL)]
    x = 40
    for t, w, c in parts:
        s.rect(x, 96, w - 3, 44, c, rx=3, extra=' fill-opacity="0.3"')
        s.text(x + w / 2, 123, t, 12, TEXT, 'middle', '600')
        x += w
    box(s, 110, 196, 200, 60, BLUE, 'Ghidra program 1', ['ARM9 at its address'])
    box(s, 330, 196, 200, 60, PURPLE, 'Ghidra program 2', ['ARM7 at its address'])
    arrow(s, 195, 142, 200, 192)
    arrow(s, 340, 142, 400, 192)
    para(s, 570, 200, 'Overlays load into ARM9 memory while the game runs. Each one shares addresses with others.', 300)
    return s.render()


def n3ds_containers():
    s = Svg(900, 300, 'Nintendo 3DS containers: an NCSD or CCI card image holds NCCH partitions, and an NCCH holds the '
                      'ExeFS with the code; the contents are AES-encrypted with keys in the ARM9 boot ROM')
    s.heading('Nintendo 3DS: containers inside containers', 'The code is in the ExeFS of an NCCH. Everything is AES-encrypted.')
    box(s, 40, 86, 820, 150, RED, '', op=0.06)
    s.text(56, 106, 'NCSD / CCI: the card image (CIA for installed titles)', 12, TEXT, weight='600')
    box(s, 56, 118, 520, 104, AMBER, '', op=0.08)
    s.text(72, 138, 'NCCH: one partition, the game', 12, TEXT, weight='600')
    box(s, 72, 150, 230, 58, BLUE, 'ExeFS', ['the code, icon, banner'])
    box(s, 316, 150, 240, 58, TEAL, 'RomFS', ['the game\'s files'])
    box(s, 592, 118, 252, 104, GREY, 'more NCCH partitions', ['manual, updates'])
    s.text(40, 262, 'The keys are in the ARM9 boot ROM. The user decides how to obtain and use their own keys and dumps.', 12, DIM)
    s.text(40, 282, 'ghidra-3ds loads the result. Its analysis is described as "pretty broken".', 12, DIM)
    return s.render()


def n3ds_lineage():
    s = Svg(900, 260, 'The 3DS emulators: Citra was taken down in March 2024, and Lime3DS and PabloMK7\'s fork merged into '
                      'Azahar in late 2024, with first releases in early 2025')
    s.heading('3DS emulators since 2024', 'Repositories can disappear. Pin an exact build and keep a local copy.')
    box(s, 40, 100, 160, 60, RED, 'Citra', ['taken down March 2024'])
    box(s, 300, 80, 200, 44, BLUE, 'Lime3DS', [], 12)
    box(s, 300, 136, 200, 44, BLUE, 'PabloMK7\'s fork', [], 12)
    arrow(s, 202, 125, 296, 102, dash=True)
    arrow(s, 202, 135, 296, 158, dash=True)
    arrow(s, 502, 102, 596, 125)
    arrow(s, 502, 158, 596, 135)
    box(s, 600, 100, 260, 60, GREEN, 'Azahar', ['merged late 2024, releases early 2025'])
    s.text(40, 220, 'Citra went at the same time as the yuzu settlement.', 12, DIM)
    return s.render()


# ================================================================ docs/handhelds/psp-and-vita.md
def psp_vita_exec():
    s = Svg(900, 280, 'PSP and PS Vita executables: an encrypted ~PSP ELF in EBOOT.BIN and PRX files, and a SELF wrapping '
                      'an ARM ELF with SUPRX modules; each needs a decrypted file and a Ghidra extension')
    s.heading('PSP and PS Vita: signed and encrypted', 'Decrypt your own copy first. Then each needs a Ghidra extension.')
    rows = [('PSP', 'EBOOT.BIN, PRX: "~PSP" encrypted', 'MIPS32 + VFPU ELF', 'ghidra-allegrex', 'PPSSPP'),
            ('PS Vita', 'SELF, SUPRX modules', 'ARM ELF (Cortex-A9)', 'VitaLoaderRedux', 'Vita3K')]
    for i, (p, wrap, inner, ld, emu) in enumerate(rows):
        y = 96 + i * 76
        s.text(40, y + 34, p, 13, TEXT, weight='600')
        box(s, 130, y, 260, 56, RED, wrap, [], 11, op=0.15)
        arrow(s, 392, y + 28, 426, y + 28)
        box(s, 430, y, 170, 56, BLUE, inner, [], 11)
        arrow(s, 602, y + 28, 636, y + 28)
        box(s, 640, y, 220, 56, GREEN, ld, [f'run in {emu}'], 12)
    s.text(40, 264, 'splat supports PSP binaries. Capstone has no Allegrex mode.', 12, DIM)
    return s.render()


# ================================================================ docs/handhelds/switch-family.md
def switch_nca():
    s = Svg(900, 300, 'A Switch game: the NSO executable sits in the ExeFS of an encrypted NCA; the keys come from the '
                      'user\'s own console and Ghidra-Switch-Loader loads the NSO')
    s.heading('Nintendo Switch: NSO inside an NCA', 'Four Cortex-A57 cores at 1.02 GHz. AArch64 code.')
    box(s, 40, 86, 520, 150, RED, '', op=0.06)
    s.text(56, 106, 'NCA, encrypted', 12, TEXT, weight='600')
    box(s, 56, 118, 300, 104, AMBER, '', op=0.08)
    s.text(72, 138, 'ExeFS', 12, TEXT, weight='600')
    box(s, 72, 150, 130, 58, BLUE, 'main', ['the NSO'])
    box(s, 212, 150, 130, 58, GREY, 'sdk, subsdk', ['more NSOs'])
    box(s, 370, 118, 174, 104, TEAL, 'RomFS', ['the game\'s files'])
    arrow(s, 562, 160, 606, 160)
    box(s, 610, 86, 250, 64, PURPLE, 'prod.keys, title keys', ['from the user\'s console'])
    box(s, 610, 170, 250, 66, GREEN, 'Ghidra-Switch-Loader', ['loads the NSO'])
    s.text(40, 264, 'Homebrew uses NRO files. The Switch Lite runs the same games, so the same tools apply.', 12, DIM)
    return s.render()


def switch_timeline():
    s = Svg(900, 290, 'Switch emulators from 2024 to 2026: the yuzu settlement in March 2024, Ryubing in maintenance in '
                      'February 2026, takedown notices to at least 13 projects, and Eden and Citron as the active forks')
    s.heading('Switch emulators since 2024', 'From secondary 2026 coverage. Pin an exact build and keep a local copy.')
    cols = {'end': RED, 'fork': BLUE, 'event': AMBER}
    items = [(2024.17, 'March 2024: yuzu settles, Citra goes', 'end'), (2024.4, 'Suyu, an early fork', 'fork'),
             (2025.2, 'Eden and Citron, the active forks', 'fork'), (2026.1, 'Feb 2026: Ryubing in maintenance', 'end'),
             (2026.12, 'takedown notices to 13+ projects', 'event')]
    timeline(s, items, 2023.8, 2026.6, 60, 840, 220, cols, step=1)
    s.text(60, 274, 'The 2024 to 2026 cases ended in settlements. No court ruling on console emulators is known from that period.', 12, DIM)
    return s.render()


def switch2_rate():
    s = Svg(900, 300, 'A fixed 60 Hz display shows a frame every 16.7 ms, while the Switch 2 display with VRR shows a '
                      'frame when the game finishes it, up to 120 Hz, so a test records the rate')
    s.heading('Switch 2: the game sets its own frame rate', 'With VRR, a frame shows when it is ready, up to 120 Hz.')
    x0, w = 200, 640
    s.text(40, 112, 'fixed 60 Hz', 12, TEXT, weight='600')
    for i in range(8):
        s.rect(x0 + i * w / 8, 96, w / 8 - 4, 26, BLUE, rx=3, extra=' fill-opacity="0.3"')
    s.text(40, 172, 'VRR, up to 120 Hz', 12, TEXT, weight='600')
    widths = [1, 0.7, 1.3, 0.55, 0.55, 0.9, 1.2, 0.6, 0.6, 0.6]
    total = sum(widths)
    x = x0
    for wd in widths:
        ww = wd / total * w
        s.rect(x, 156, ww - 4, 26, TEAL, rx=3, extra=' fill-opacity="0.3"')
        x += ww
    s.add(f'<rect x="{x0}" y="90" width="2" height="98" fill="{AMBER}"><animate attributeName="x" values="{x0};{x0 + w}" '
          'dur="4s" repeatCount="indefinite"/></rect>')
    s.text(x0, 208, 'Each box is one frame on the display. Both rows cover the same span of time.', 11, DIM)
    para(s, 40, 246, 'Record the frame rate in each test. The Game-Key Card and the Joy-Con 2 mouse are outside inputs.', 820)
    return s.render()


# ================================================================ the other handheld pages
def arm_os_stubs():
    s = Svg(900, 280, 'An ARM-OS handheld game calls the operating system; to run one function under Unicorn, replace '
                      'each OS call it makes with a stub that returns what the OS would')
    s.heading('ARM on an OS: stub the OS calls', 'Study the imports and OS calls as closely as the CPU code, as for a Windows game.')
    box(s, 40, 90, 220, 120, BLUE, 'Game function', ['ARM code'])
    oss = [('N-Gage', 'Symbian OS 6.1'), ('Gizmondo', 'Windows CE'), ('GP2X', 'Linux')]
    for i, (dev, os_) in enumerate(oss):
        y = 90 + i * 42
        arrow(s, 262, 150, 336, y + 17)
        box(s, 340, y, 220, 34, GREY, f'{dev}: {os_}', [], 11)
    arrow(s, 562, 150, 616, 150)
    box(s, 620, 90, 240, 120, GREEN, 'Unicorn stubs', ['one per OS call,', 'returning what', 'the OS would'])
    s.text(40, 244, 'EKA2L1 reimplements Symbian at high level, so code that needs exact OS behaviour may differ.', 12, DIM)
    return s.render()


def steamdeck_path():
    s = Svg(900, 274, 'The Steam Deck runs Windows games through Proton on SteamOS, so GameFrame\'s Windows PE path '
                      'applies: Ghidra, Unicorn x86-64 and Wine')
    s.heading('PC handhelds: the PC path', 'An x86-64 PC with a built-in screen. The games are PC games.')
    steps = [('Windows game', ['a PE file'], BLUE), ('Proton', ['on SteamOS 3'], GREY), ('GameFrame', ['Ghidra, Unicorn', 'x86-64, Wine'], TEAL)]
    flow(s, 40, 96, 220, 66, 40, steps)
    box(s, 40, 190, 380, 44, AMBER, 'LCD model: 60 Hz panel', [], 12)
    box(s, 440, 190, 380, 44, PURPLE, 'OLED model: 90 Hz panel', [], 12)
    s.text(40, 260, 'Record which panel a test ran on.', 12, DIM)
    return s.render()


def j2me_builds():
    s = Svg(900, 344, 'One J2ME game often shipped as many builds for different handsets, each a separate .jar to treat '
                      'as its own target')
    s.heading('Phones: one game, many builds', 'Each handset got its own .jar. Treat each build as a separate target.')
    box(s, 375, 80, 150, 44, GREY, 'One game', [])
    s.line(450, 126, 450, 142, DIM, 1.5)
    s.line(130, 142, 740, 142, DIM, 1.5)
    sizes = [(52, 52), (60, 72), (72, 96), (96, 72)]
    for i, (w, h) in enumerate(sizes):
        cx = 130 + i * 203
        arrow(s, cx, 142, cx, 158)
        screen(s, cx - w / 2, 162, w, h, chr(65 + i), [BLUE, TEAL, PURPLE, AMBER][i])
        s.text(cx, 162 + 96 + 18, f'build {chr(65 + i)}: its own .jar', 11, DIM, 'middle')
    s.text(40, 306, 'Builds differ in screen size, keys and libraries. Record which build a test runs.', 12, DIM)
    s.text(40, 326, 'Ghidra\'s built-in JVM module reads the bytecode.', 12, DIM)
    return s.render()


def brew_signing():
    s = Svg(900, 240, 'A downloaded BREW game is signed for one handset, so a copy does not run on another handset and '
                      'most were never dumped')
    s.heading('BREW: signed for one handset', 'Downloaded apps (.mif and .mod files) carry a signature tied to the phone.')
    box(s, 40, 100, 170, 60, GREY, 'Download', ['.mif, .mod'])
    arrow(s, 212, 130, 256, 130)
    box(s, 260, 100, 200, 60, GREEN, 'Handset A', ['the signature matches'])
    arrow(s, 462, 130, 506, 130, dash=True)
    s.text(484, 120, 'copy', 10, DIM, 'middle')
    box(s, 510, 100, 200, 60, RED, 'Handset B', ['does not run'])
    para(s, 730, 112, 'So most BREW games were never dumped.', 140)
    s.text(40, 212, 'Melange (Android) is in early development. Zeebo games became playable through @Tuxality\'s work in 2024.', 12, DIM)
    return s.render()


DIAGRAMS = {
    'console-frame-loop.svg': console_frame_loop,
    'console-first-steps.svg': console_first_steps,
    'nes-bank-switching.svg': nes_bank_switching,
    'n64-rcp.svg': n64_rcp,
    'psx-gte.svg': psx_gte,
    'ps2-ee-iop.svg': ps2_ee_iop,
    'console-early-timeline.svg': early_console_timeline,
    'atari2600-beam.svg': atari2600_beam,
    'intellivision-words.svg': cp1610_words,
    'colecovision-parts.svg': colecovision_blocks,
    'atari7800-clock.svg': atari7800_clock,
    'vectrex-beam.svg': vectrex_beam,
    'console-code-upload.svg': console_code_upload,
    'segacd-word-ram.svg': segacd_word_ram,
    'sega32x-cpus.svg': sega_32x_cpus,
    'pcengine-mmu.svg': huc6280_mmu,
    'cdi-os9.svg': cdi_os9,
    'jaguar-cpus.svg': jaguar_cpus,
    '3do-arm-versions.svg': arm_versions_3do,
    'console-matching-loop.svg': matching_loop,
    'console-matching-choice.svg': matching_choice,
    'console-modern-containers.svg': modern_containers,
    'console-modern-status.svg': modern_status,
    'ps3-cell.svg': ps3_cell,
    'xbox360-recomp.svg': xenon_recomp,
    'gamecube-paired-singles.svg': gekko_paired_singles,
    'console-port-routes.svg': port_routes,
    'console-fixed-rate.svg': port_fixed_rate,
    'console-input-mask.svg': input_mask,
    'console-video-kinds.svg': video_kinds,
    'console-raster-split.svg': raster_split,
    'console-sound-cpu.svg': sound_cpu,
    'console-saves.svg': console_saves,
    'console-port-check.svg': port_check,
    'nes-memory-map.svg': nes_memory_map,
    'console-6502-adc.svg': ghidra_adc,
    'console-cpu-support.svg': cpu_support,
    'console-ram-search.svg': ram_search,
    'console-unicorn-run.svg': unicorn_run,
    'console-splat-flow.svg': splat_flow,
    'handheld-groups.svg': handheld_groups,
    'gameboy-mbc3-rtc.svg': mbc3_rtc,
    'handheld-outside-inputs.svg': outside_inputs,
    'handheld-lcd-ghosting.svg': lcd_ghosting,
    'gameboy-memory-map.svg': gb_memory_map,
    'gbc-modes.svg': gbc_modes,
    'gbc-hdma.svg': gbc_hdma,
    'sgb-layout.svg': sgb_layout,
    'gba-parts.svg': gba_blocks,
    'gamewatch-segments.svg': gw_segments,
    'microvision-cartridge.svg': microvision,
    'handheld-screens.svg': handheld_screens,
    'lynx-parts.svg': lynx_blocks,
    'wonderswan-path.svg': wonderswan_path,
    'virtualboy-eyes.svg': virtualboy_eyes,
    'pokemini-parts.svg': pokemini_blocks,
    'gamecom-touch.svg': gamecom_touch,
    'ds-cartridge.svg': ds_cartridge,
    '3ds-containers.svg': n3ds_containers,
    '3ds-emulators.svg': n3ds_lineage,
    'psp-vita-executables.svg': psp_vita_exec,
    'switch-nca.svg': switch_nca,
    'switch-emulators.svg': switch_timeline,
    'switch2-frame-rate.svg': switch2_rate,
    'handheld-arm-os-stubs.svg': arm_os_stubs,
    'steamdeck-pc-path.svg': steamdeck_path,
    'phone-j2me-builds.svg': j2me_builds,
    'phone-brew-signing.svg': brew_signing,
}


# The label of each card is the alt text the guides give it, in plain sentences.
LABELS = {
    'console-frame-loop.svg': 'The reset vector, the main loop and the vertical blank interrupt of a console game',
    'console-first-steps.svg': 'Six first steps on a console game, from the ROM revision to the RAM map',
    'nes-bank-switching.svg': 'NES bank switching, where a mapper puts one 16 KB bank at 8000 while the last bank stays fixed',
    'n64-rcp.svg': 'Nintendo 64 graphics from display lists through RSP microcode and the RDP to the frame buffer',
    'psx-gte.svg': 'The PlayStation GTE rotates, translates and projects vertices in fixed point',
    'ps2-ee-iop.svg': 'The PlayStation 2 Emotion Engine, vector units and IOP, and how its floats differ from IEEE 754',
    'console-early-timeline.svg': 'The early consoles by year of release, coloured by CPU family',
    'atari2600-beam.svg': 'Racing the beam on the Atari 2600, with one scanline of 76 CPU cycles and one NTSC frame of 262 lines',
    'intellivision-words.svg': 'Intellivision ROM words are 10 bits wide. SDBD builds a 16-bit operand from two of them',
    'colecovision-parts.svg': 'The ColecoVision parts on the CPU bus, with the video chip and its own VRAM',
    'atari7800-clock.svg': 'The Atari 7800 clock slows for TIA and RIOT access. NTSC consoles check the cartridge signature',
    'vectrex-beam.svg': 'The Vectrex CPU steers the beam through a DAC, beside the 6809 registers',
    'console-code-upload.svg': 'The main CPU uploads code for a second CPU, so the code is taken from a RAM dump',
    'segacd-word-ram.svg': 'The 68000 of the Mega Drive and the 68000 of the Mega-CD share word RAM',
    'sega32x-cpus.svg': 'A 32X game runs code on four CPUs, each exported as its own program',
    'pcengine-mmu.svg': 'The HuC6280 MMU maps eight 8 KB pages of the CPU onto 256 banks of a 2 MB space',
    'cdi-os9.svg': 'CD-i programs are OS-9 modules, each loaded with the 68000 language',
    'jaguar-cpus.svg': 'The Jaguar 68000 uploads code into the local RAM of the Tom and Jerry RISC cores',
    '3do-arm-versions.svg': 'The ARM60 is ARMv3, one version older than the first ARM language in Ghidra',
    'console-matching-loop.svg': 'The matching loop of writing C, compiling, comparing and editing until one function matches',
    'console-matching-choice.svg': 'Three questions that decide whether to choose a matching decompilation',
    'console-modern-containers.svg': 'Signed and encrypted containers around each executable, with the loader for each',
    'console-modern-status.svg': 'Which platforms from the Xbox 360 onward have a Ghidra loader and an emulator',
    'ps3-cell.svg': 'The Cell with one PPE and six SPEs, each with 256 KiB of local store reached by DMA',
    'xbox360-recomp.svg': 'Static recompilation of an Xbox 360 game into a native PC port',
    'gamecube-paired-singles.svg': 'Paired singles hold two floats in one register. A GQR register sets the quantised load',
    'console-port-routes.svg': 'Four routes to a port, by effort and by what can be published',
    'console-fixed-rate.svg': 'The logic keeps the original rate on a faster display. A lag frame has no update',
    'console-input-mask.svg': 'Buttons pressed this frame are the held buttons that were not held in the previous frame',
    'console-video-kinds.svg': 'Three kinds of console video and how a port draws each',
    'console-raster-split.svg': 'A raster effect, where an interrupt on one line rewrites the scroll register',
    'console-sound-cpu.svg': 'Sound on its own CPU, and the two ways to port it',
    'console-saves.svg': 'Battery RAM, memory cards and passwords, and how a port keeps each format',
    'console-port-check.svg': 'One input movie drives the original and the port. The comparison stops at the first difference',
    'nes-memory-map.svg': 'The NES CPU memory map with its named registers and vectors',
    'console-6502-adc.svg': 'Two cases where the 6502 ADC p-code in Ghidra gives the wrong flags',
    'console-cpu-support.svg': 'Console CPUs and their support in Ghidra and Unicorn',
    'console-ram-search.svg': 'RAM search narrows the candidates until one address is left',
    'console-unicorn-run.svg': 'Five steps to run one original console function under Unicorn',
    'console-splat-flow.svg': 'Splitting a ROM into code, data and assets, then building the same bytes again',
    'handheld-groups.svg': 'The four groups of handhelds and where each keeps its program',
    'gameboy-mbc3-rtc.svg': 'The MBC3 clock registers, the latch and the bits of the DH register',
    'handheld-outside-inputs.svg': 'Outside inputs a test records each frame beside the buttons',
    'handheld-lcd-ghosting.svg': 'Flickered sprites blend on a slow LCD. The test compares the raw frames',
    'gameboy-memory-map.svg': 'The Game Boy memory map, with the switchable ROM bank at 4000',
    'gbc-modes.svg': 'The header byte at 0143 picks the Game Boy Color mode. Double speed doubles the clock',
    'gbc-hdma.svg': 'General-purpose DMA and HBlank DMA on the Game Boy Color, and what double speed changes',
    'sgb-layout.svg': 'The Game Boy screen inside the SNES picture, drawn to scale, and the fast SGB clock',
    'gba-parts.svg': 'The Game Boy Advance CPUs, video, sound and tools',
    'gamewatch-segments.svg': 'A Game & Watch MCU lights fixed printed segments from its LCD RAM',
    'microvision-cartridge.svg': 'The Microvision base unit has no CPU. Each cartridge holds its own',
    'handheld-screens.svg': 'Handheld screens drawn to scale',
    'lynx-parts.svg': 'The Lynx chips Mikey and Suzy, and the boot ROM that decrypts the loader',
    'wonderswan-path.svg': 'The WonderSwan path through Ghidra x86 real mode and Unicorn',
    'virtualboy-eyes.svg': 'The Virtual Boy draws one 384 by 224 picture for each eye',
    'pokemini-parts.svg': 'The Pokémon mini parts and its outside inputs',
    'gamecom-touch.svg': 'The Game.com screen with its 12 by 10 touch grid',
    'ds-cartridge.svg': 'A DS cartridge holds ARM9 and ARM7 binaries, overlays and NitroFS',
    '3ds-containers.svg': 'The 3DS containers NCSD or CCI, NCCH, ExeFS and RomFS',
    '3ds-emulators.svg': 'The 3DS emulators since Citra was taken down in 2024',
    'psp-vita-executables.svg': 'The PSP and PS Vita executables and the Ghidra extension for each',
    'switch-nca.svg': 'The NSO of a Switch game inside the ExeFS of an encrypted NCA',
    'switch-emulators.svg': 'Switch emulators from the yuzu settlement in 2024 to the takedowns of 2026',
    'switch2-frame-rate.svg': 'A fixed 60 Hz display compared with VRR up to 120 Hz',
    'handheld-arm-os-stubs.svg': 'Game code calls the OS, so Unicorn needs a stub for each call',
    'steamdeck-pc-path.svg': 'The Steam Deck runs Windows games through Proton, so the PC path applies',
    'phone-j2me-builds.svg': 'One J2ME game shipped as many builds, each its own target',
    'phone-brew-signing.svg': 'A downloaded BREW game is signed for one handset',
}


def _labelled(fn, label):
    def draw():
        svg = fn()
        svg = re.sub(r'aria-label="[^"]*"', f'aria-label="{esc(label)}"', svg, count=1)
        return re.sub(r'<title>[^<]*</title>', f'<title>{esc(label)}</title>', svg, count=1)
    return draw


DIAGRAMS = {name: _labelled(fn, LABELS[name]) for name, fn in DIAGRAMS.items()}
