"""Diagrams for the home computer guides (docs/computers/): memory maps, video timing, custom
chips, disk and tape formats, CPU families, executable formats and the Mac's code formats.
site/diagrams/make.py writes them. arcade.py uses the drawing helpers defined here.
"""
from runtime import show, part
from svg import AMBER, BG, BLUE, DIM, GREEN, GREY, PURPLE, RED, TEAL, TEXT, Svg

PINK = '#f472b6'
SANS = 'ui-sans-serif, Helvetica, Arial, sans-serif'

DIAGRAMS = {}


def diagram(name):
    """Register the function under NAME in DIAGRAMS."""
    def wrap(fn):
        DIAGRAMS[name] = fn
        return fn
    return wrap


# ---------------------------------------------------------------- drawing helpers
def box(s, x, y, w, h, colour, title, sub=None, op=0.2, size=13, sub_size=11, mono=False, anchor='middle'):
    """A filled box with a title and an optional second line."""
    s.rect(x, y, w, h, colour, rx=6, extra=f' fill-opacity="{op}"')
    tx = x + w / 2 if anchor == 'middle' else x + 10
    if sub:
        s.text(tx, y + h / 2 - 3, title, size, TEXT, anchor, '600')
        s.text(tx, y + h / 2 + 13, sub, sub_size, DIM, anchor, mono=mono)
    else:
        s.text(tx, y + h / 2 + 5, title, size, TEXT, anchor, '600')


def arrow(s, x1, y1, x2, y2, colour=DIM, width=1.5):
    s.line(x1, y1, x2, y2, colour, width, ' marker-end="url(#arrow)"')


def lines(s, x, y, rows, size=12, fill=DIM, gap=18, anchor='start', mono=False):
    """Several lines of text, one under another. Returns the y after the last line."""
    for row in rows:
        s.text(x, y, row, size, fill, anchor, mono=mono)
        y += gap
    return y


def memmap(s, x, y, w, regions, size=12):
    """A memory map drawn as a column, low addresses at the top: (address, label, height, colour)."""
    for addr, label, h, colour in regions:
        s.rect(x, y, w, h - 2, colour, rx=3, extra=' fill-opacity="0.25"')
        if h >= 16:
            s.text(x + 10, y + h / 2 + 3, label, size if h >= 20 else 10, TEXT)
        if addr:
            s.text(x - 8, y + 10, addr, 11, DIM, 'end', mono=True)
        y += h
    return y


def grid_cell(s, x, y, w, h, colour, op):
    s.rect(x, y, w, h, colour, rx=2, extra=f' fill-opacity="{op}"')


def fade(body, times, dur):
    """BODY shown only between the given fractions of DUR, as a list of (start, end)."""
    keys, vals = [0.0], ['0']
    for a, b in times:
        keys += [a, b]
        vals += ['1', '0']
    keys.append(1.0)
    vals.append('0')
    pairs = sorted(zip(keys, vals))
    return (f'<g opacity="0">{body}<animate attributeName="opacity" values="{";".join(v for _, v in pairs)}" '
            f'keyTimes="{";".join(f"{k:.4f}" for k, _ in pairs)}" dur="{dur}s" calcMode="discrete" '
            f'repeatCount="indefinite"/></g>')


# ================================================================ the computers README
@diagram('computer-groups.svg')
def computer_groups():
    s = Svg(900, 330, 'The machines fall into four groups. The first step is to get past the loader, to learn the '
                      'custom chips, to identify the executable format, or to read the console guides.')
    s.heading('Four groups of machines', 'Each group starts with a different first step.')
    groups = [
        ('8-bit home computers', 'Sinclair, Acorn, Commodore,', 'Amstrad, MSX, Atari, Apple II', TEAL,
         ['Get past the loader.', 'Then learn the frame timing.']),
        ('16 and 32-bit computers', 'Amiga, Atari ST, Archimedes,', 'X68000, PC-98, FM Towns', BLUE,
         ['Learn the custom chips.', 'Many games bypass the OS.']),
        ('The PC and the Mac', 'DOS, Windows, 68k, PowerPC,', 'Intel and ARM Macs, engines', AMBER,
         ['Identify the executable format.', 'The tools depend on it.']),
        ('Consoles', 'The console guides', 'cover these machines', GREY,
         ['See Console games.', '']),
    ]
    x = 24
    for title, a, b, colour, step in groups:
        s.rect(x, 80, 202, 108, colour, rx=8, extra=' fill-opacity="0.16"')
        s.text(x + 14, 106, title, 14, TEXT, weight='600')
        s.text(x + 14, 130, a, 11, DIM)
        s.text(x + 14, 146, b, 11, DIM)
        arrow(s, x + 101, 192, x + 101, 222, colour)
        s.rect(x, 226, 202, 62, colour, rx=8, extra=' fill-opacity="0.32"')
        s.text(x + 14, 250, step[0], 12, TEXT, weight='600')
        s.text(x + 14, 268, step[1], 12, TEXT)
        x += 216
    s.text(24, 314, 'First step', 11, DIM)
    return s.render()


@diagram('computer-loader-snapshot.svg')
def loader_snapshot():
    s = Svg(900, 330, 'A tape or disk game runs through its loader and protection in an emulator. Once the game code '
                      'is in memory, a snapshot is taken. The lossless media image is kept as the record.')
    s.heading('Snapshot after the loader', 'Run the loader in an emulator. Take the snapshot once the game code sits in memory.')
    steps = [('media image', 'TZX, G64, WOZ, IPF', GREY), ('loader', 'BASIC stub, turbo', BLUE),
             ('protection', 'decrypts the next stage', RED), ('game in memory', 'decrypted code', TEAL),
             ('snapshot', 'the port starts here', GREEN)]
    dur = 6.0
    for i, (title, sub, colour) in enumerate(steps):
        x = 24 + i * 174
        box(s, x, 100, 150, 56, colour, title, sub, op=0.22)
        if i:
            arrow(s, x - 22, 128, x - 3, 128)
        s.add(f'<rect x="{x}" y="100" width="150" height="56" rx="6" fill="none" stroke="{AMBER}" stroke-width="2.5" '
              f'opacity="0"><animate attributeName="opacity" values="0;1;0;0" '
              f'keyTimes="0;{i / 5:.3f};{(i + 1) / 5:.3f};1" calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/></rect>')
    # Memory filling up as the loader runs.
    s.text(24, 196, 'memory while it loads', 12, TEXT, weight='600')
    for k in range(32):
        x = 24 + k * 27
        t = min(0.79, 0.2 + k / 32 * 0.4)
        s.add(f'<rect x="{x}" y="206" width="25" height="22" rx="2" fill="{GREY}" fill-opacity="0.25">'
              f'<animate attributeName="fill" values="{GREY};{RED};{TEAL};{GREY}" keyTimes="0;{t:.3f};{t + 0.2:.3f};1" '
              f'calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.rect(24, 246, 12, 12, RED, rx=2, extra=' fill-opacity="0.6"')
    s.text(42, 256, 'encrypted, as loaded', 11, DIM)
    s.rect(200, 246, 12, 12, TEAL, rx=2, extra=' fill-opacity="0.6"')
    s.text(218, 256, 'decrypted game code', 11, DIM)
    lines(s, 24, 290, ['The port does not need to copy the protection code. Keep the lossless image (TZX, G64, WOZ, EDSK, ATX, '
                       'IPF, STX)', 'as the record of what shipped.'], 12)
    return s.render()


@diagram('computer-8bit-timing.svg')
def eight_bit_timing():
    s = Svg(900, 340, 'On four 8-bit machines the game logic depends on exact timing. They are the Spectrum with '
                      'contended memory, the C64 with badlines, the Atari with ANTIC DMA and the CPC, which rounds '
                      'every access to 4 cycles.')
    s.heading('Machines where video timing affects the game', 'On these four machines the video hardware takes CPU time. The CPU time left depends on the screen.')
    cards = [
        ('ZX Spectrum', 'contended memory', RED,
         ['The ULA delays the CPU when both', 'use 0x4000-0x7FFF.', 'The delay depends on where the', 'beam is in the frame.']),
        ('Commodore 64', 'badlines', BLUE,
         ['On every eighth screen line the', 'VIC-II takes about 40 of the', '63 cycles (PAL).', 'Raster interrupts time effects.']),
        ('Atari 400/800', 'ANTIC DMA', PURPLE,
         ['ANTIC fetches the display list', 'and screen data each line.', 'Display list interrupts change', 'the picture part way down.']),
        ('Amstrad CPC', 'wait states', AMBER,
         ['Every instruction is stretched', 'to a multiple of 4 T-states.', 'Time is counted in NOPs', 'of 1 microsecond.']),
    ]
    x = 24
    for title, sub, colour, rows in cards:
        s.rect(x, 78, 202, 180, colour, rx=8, extra=' fill-opacity="0.13"')
        s.text(x + 14, 104, title, 14, TEXT, weight='600')
        s.text(x + 14, 122, sub, 12, colour, weight='600')
        lines(s, x + 14, 152, rows, 12, TEXT, 20)
        x += 216
    lines(s, 24, 290, ['A port that runs the game logic per frame must give it the CPU time the real frame left over.',
                       'Each machine\'s page gives the numbers.'], 12)
    return s.render()


# ================================================================ Sinclair
@diagram('zx81-display.svg')
def zx81_display():
    s = Svg(900, 320, 'On the ZX81 the CPU generates the display. In SLOW mode the picture shows and the game runs in '
                      'the border time. In FAST mode the screen goes blank and the game gets all the CPU time.')
    s.heading('The ZX81 CPU draws the picture', 'The ZX81 has a Z80A at 3.25 MHz. The Ferranti ULA needs the CPU to feed it every visible line.')
    for i, (mode, colour, rows) in enumerate((
            ('SLOW mode', TEAL, ['The picture shows.', 'The CPU spends the visible lines', 'drawing it. The game runs in the', 'border and blanking time.']),
            ('FAST mode', AMBER, ['The screen blanks while the', 'program runs.', 'The game gets all the CPU time.', '']))):
        x = 40 + i * 430
        s.text(x, 96, mode, 14, colour, weight='600')
        s.rect(x, 108, 200, 150, GREY, rx=4, extra=' fill-opacity="0.25"')
        if i == 0:
            s.rect(x + 24, 130, 152, 106, BLUE, rx=2, extra=' fill-opacity="0.35"')
            s.text(x + 100, 188, 'picture', 12, TEXT, 'middle')
            s.add(f'<rect x="{x + 24}" y="130" width="152" height="3" fill="{AMBER}"><animate attributeName="y" '
                  f'values="130;233" dur="1.5s" repeatCount="indefinite"/></rect>')
        else:
            s.rect(x + 24, 130, 152, 106, '#000000', rx=2)
            s.text(x + 100, 188, 'blank', 12, DIM, 'middle')
        lines(s, x + 214, 132, rows, 12, TEXT)
    s.text(40, 292, 'Record the mode the game runs in. The CPU time left for the game logic depends on it.', 12, DIM)
    return s.render()


@diagram('spectrum-memory-map.svg')
def spectrum_memory_map():
    s = Svg(900, 440, 'The ZX Spectrum memory map holds ROM, the screen bitmap and the attributes. On 128K models port '
                      '0x7FFD pages one of eight 16 KB RAM pages into 0xC000.')
    s.heading('Spectrum memory and paging', 'This is the 48K memory map. On the 128K, any of eight 16 KB pages can appear in the top quarter.')
    x, y = 110, 84
    memmap(s, x, y, 230, [('0x0000', 'ROM (16 KB)', 64, GREY), ('0x4000', 'screen bitmap, 6144 bytes', 44, BLUE),
                          ('0x5800', 'attributes, 768 bytes', 22, AMBER), ('0x5B00', 'RAM (contended to 0x7FFF)', 28, RED),
                          ('0x8000', 'RAM', 64, TEAL), ('0xC000', 'RAM, or a 128K page', 64, GREEN)])
    s.text(x, y + 306, '0x4000 to 0x7FFF is contended memory on the 48K.', 11, RED)
    # The 128K pages.
    px = 440
    s.text(px, 100, 'port 0x7FFD on the 128K, +2, +2A and +3', 12, TEXT, weight='600')
    fields = [('lock', 1, RED), ('ROM', 1, GREY), ('screen', 1, BLUE), ('RAM page at 0xC000', 3, GREEN)]
    bx = px
    for label, n, colour in [('-', 2, GREY)] + fields:
        w = n * 54
        s.rect(bx, 112, w - 2, 30, colour, rx=3, extra=' fill-opacity="0.3"')
        s.text(bx + w / 2 - 1, 131, label, 11 if n > 1 else 10, TEXT, 'middle', '600')
        bx += w
    s.text(px, 158, 'bit 7', 10, DIM, mono=True)
    s.text(px + 432, 158, 'bit 0', 10, DIM, 'end', mono=True)
    s.text(px + 216, 158, 'screen: bit 3 picks page 5 or page 7 for display', 10, DIM, 'middle')
    contended = {1, 3, 5, 7}
    s.text(px, 196, 'pages 0 to 7 (128K and +2 contention shown)', 12, TEXT, weight='600')
    dur = 8.0
    for p in range(8):
        cx = px + p * 54
        s.rect(cx, 206, 46, 40, RED if p in contended else TEAL, rx=3, extra=' fill-opacity="0.25"')
        s.text(cx + 23, 230, str(p), 13, TEXT, 'middle', '600', mono=True)
        s.add(f'<rect x="{cx}" y="206" width="46" height="40" rx="3" fill="none" stroke="{AMBER}" stroke-width="2.5" '
              f'opacity="0"><animate attributeName="opacity" values="0;1;0;0" keyTimes="0;{p / 8:.3f};{(p + 1) / 8:.3f};1" '
              f'calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.text(px, 270, 'page 5 is always at 0x4000 and page 2 at 0x8000', 11, DIM)
    s.text(px, 288, 'page 7 holds the shadow screen', 11, DIM)
    s.rect(px, 304, 12, 12, RED, rx=2, extra=' fill-opacity="0.5"')
    s.text(px + 18, 314, 'contended: pages 1, 3, 5, 7 (128K, +2); 4 to 7 (+2A, +3)', 11, DIM)
    lines(s, px, 350, ['The +2A and +3 add port 0x1FFD for all-RAM modes.', 'Record both ports for every code region',
                       'that runs from a paged bank.'], 12, TEXT)
    return s.render()


@diagram('spectrum-screen.svg')
def spectrum_screen():
    s = Svg(900, 400, 'The Spectrum screen is a 256 by 192 bitmap whose lines are stored out of order, with one '
                      'attribute byte for each 8 by 8 cell. A sprite that crosses a cell changes the colour of the '
                      'whole cell.')
    s.heading('Spectrum screen and attribute clash', 'Each 8 x 8 cell has one attribute byte. It sets one ink and one paper colour for all 64 pixels.')
    # The attribute byte.
    x0 = 24
    s.text(x0, 96, 'attribute byte (0x5800 + row x 32 + column)', 12, TEXT, weight='600')
    bx = x0
    for label, n, colour in (('flash', 1, RED), ('bright', 1, AMBER), ('paper', 3, BLUE), ('ink', 3, TEAL)):
        w = n * 46
        s.rect(bx, 106, w - 2, 30, colour, rx=3, extra=' fill-opacity="0.3"')
        s.text(bx + w / 2 - 1, 125, label, 11, TEXT, 'middle', '600')
        bx += w
    s.text(x0, 156, 'bitmap address of pixel line y, character column x', 12, TEXT, weight='600')
    bx = x0
    for label, colour in (('0', GREY), ('1', GREY), ('0', GREY), ('y7', BLUE), ('y6', BLUE), ('y2', AMBER), ('y1', AMBER),
                          ('y0', AMBER), ('y5', TEAL), ('y4', TEAL), ('y3', TEAL), ('x4', PURPLE), ('x3', PURPLE),
                          ('x2', PURPLE), ('x1', PURPLE), ('x0', PURPLE)):
        s.rect(bx, 166, 22, 26, colour, rx=2, extra=' fill-opacity="0.3"')
        s.text(bx + 11, 183, label, 9, TEXT, 'middle', mono=True)
        bx += 23
    lines(s, x0, 214, ['The bitmap is in three thirds of 64 lines.', 'Inside a third, line 1 of every character row comes',
                       'first, then line 2, and so on.', 'A screen loading from tape fills in that order.'], 12)
    # Attribute clash, animated.
    gx, gy, c = 480, 90, 34
    cols, rows = 10, 6
    ink_blue = '#2b2bd7'
    for r in range(rows):
        for k in range(cols):
            s.rect(gx + k * c, gy + r * c, c - 1, c - 1, '#d7d7d7', rx=0)
    dur = 5.0
    sprite = (f'<g><rect x="0" y="0" width="16" height="16" fill="{ink_blue}"/>'
              f'<animateTransform attributeName="transform" type="translate" values="{gx + 9} {gy + 2 * c + 9};'
              f'{gx + (cols - 1) * c + 9} {gy + 2 * c + 9}" dur="{dur}s" repeatCount="indefinite"/></g>')
    # Background graphics in red ink: the sprite turns their whole cell blue.
    for k in range(cols):
        a, b = (k - 1) / (cols - 1), (k + 1) / (cols - 1)
        if a <= 0:
            vals, kt = f'{ink_blue};#d72b2b', f'0;{b:.3f}'
        elif b >= 1:
            vals, kt = f'#d72b2b;{ink_blue}', f'0;{a:.3f}'
        else:
            vals, kt = f'#d72b2b;{ink_blue};#d72b2b', f'0;{a:.3f};{b:.3f}'
        s.add(f'<rect x="{gx + k * c + 4}" y="{gy + 2 * c + 22}" width="{c - 9}" height="4" fill="#d72b2b">'
              f'<animate attributeName="fill" values="{vals}" keyTimes="{kt}" calcMode="discrete" '
              f'dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.add(sprite)
    lines(s, gx, gy + rows * c + 24, ['A blue sprite crosses a red line. Each cell it enters', 'takes the sprite\'s ink, so the line turns blue too.',
                                     'This is attribute clash.'], 12)
    return s.render()


@diagram('spectrum-contention.svg')
def spectrum_contention():
    s = Svg(900, 420, 'A Spectrum 48K frame is 312 lines of 224 T-states. On the 192 screen lines the ULA reads the '
                      'screen for 128 T-states and delays the CPU by 6, 5, 4, 3, 2, 1, 0 and 0 T-states.')
    s.heading('Spectrum contended memory (48K)', 'A frame is 69888 T-states: 312 lines of 224. The ULA reads the screen on 192 of them.')
    fx, fy, fw, fh = 40, 86, 250, 260
    line_h = fh / 312
    s.rect(fx, fy, fw, fh, GREY, rx=3, extra=' fill-opacity="0.2"')
    sy = fy + 64 * line_h
    s.rect(fx, sy, fw * 128 / 224, 192 * line_h, RED, rx=2, extra=' fill-opacity="0.35"')
    s.rect(fx + fw * 128 / 224, sy, fw * 96 / 224, 192 * line_h, TEAL, rx=2, extra=' fill-opacity="0.2"')
    s.text(fx + fw * 64 / 224, sy + 96 * line_h, 'screen fetch', 11, TEXT, 'middle')
    s.text(fx + fw * 64 / 224, sy + 96 * line_h + 14, '128 T', 11, TEXT, 'middle', mono=True)
    s.text(fx + fw * 176 / 224, sy + 96 * line_h, 'border', 11, TEXT, 'middle')
    s.text(fx + fw * 176 / 224, sy + 96 * line_h + 14, '96 T', 11, TEXT, 'middle', mono=True)
    s.text(fx + fw / 2, fy + 32 * line_h + 4, 'top border and retrace', 11, DIM, 'middle')
    s.text(fx + fw / 2, fy + 284 * line_h + 4, 'bottom border', 11, DIM, 'middle')
    s.text(fx + fw + 8, sy + 4, 'line 64: T-state 14335', 10, DIM, mono=True)
    s.text(fx + fw + 8, fy + fh, '69888', 10, DIM, mono=True)
    s.text(fx + fw + 8, fy + 8, '0: interrupt', 10, DIM, mono=True)
    # The beam, line by line.
    s.add(f'<rect x="{fx}" y="{fy}" width="{fw}" height="2" fill="{AMBER}"><animate attributeName="y" values="{fy};{fy + fh - 2}" '
          f'dur="4s" repeatCount="indefinite"/></rect>')
    # The delay pattern.
    px, py = 480, 110
    s.text(px, py - 12, 'extra T-states for a contended access, by T-state in the line', 12, TEXT, weight='600')
    pattern = [6, 5, 4, 3, 2, 1, 0, 0]
    for k in range(16):
        d = pattern[k % 8]
        h = d * 12
        s.rect(px + k * 24, py + 80 - h, 20, max(h, 2), RED if d else TEAL, rx=2, extra=' fill-opacity="0.6"')
        s.text(px + k * 24 + 10, py + 96, str(d), 11, TEXT, 'middle', mono=True)
    s.text(px, py + 116, 'the pattern repeats 16 times across the 128 T-states', 11, DIM)
    rows = [('48K', '0x4000-0x7FFF', '14335', '224', '69888'), ('128K, +2', 'pages 1, 3, 5, 7', '14361', '228', '70908'),
            ('+2A, +3', 'pages 4 to 7', '14363', '', '')]
    ty = py + 160
    for k, head in enumerate(('model', 'contended', 'first', 'T/line', 'T/frame')):
        s.text(px + [0, 80, 210, 270, 330][k], ty, head, 11, DIM, weight='600')
    for r, row in enumerate(rows):
        for k, v in enumerate(row):
            s.text(px + [0, 80, 210, 270, 330][k], ty + 22 + r * 20, v, 11, TEXT, mono=k > 1)
    lines(s, px, ty + 100, ['The +2A and +3 use 7, 6, 5, 4, 3, 2, 1, 0, 0.', '"Late timing" ULAs run one T-state later.'], 11)
    return s.render()


@diagram('spectrum-tzx.svg')
def spectrum_tzx():
    s = Svg(900, 380, 'A TZX tape is a list of blocks with an ID, such as 0x10 standard speed, 0x11 turbo, 0x13 pulse '
                      'sequence, 0x15 direct recording and 0x19 generalised data. Each block stores pulse lengths in '
                      'T-states at 3.5 MHz.')
    s.heading('TZX stores the tape as timed pulses', 'Each block has an ID. Timings are in T-states at 3.5 MHz, so turbo and protected loaders still load.')
    blocks = [('0x10', 'standard', BLUE, 120), ('0x11', 'turbo', AMBER, 150), ('0x13', 'pulse seq.', RED, 90),
              ('0x15', 'direct rec.', PURPLE, 130), ('0x19', 'generalised', TEAL, 130), ('0x11', 'turbo', AMBER, 160)]
    x = 24
    for bid, name, colour, w in blocks:
        box(s, x, 84, w - 4, 48, colour, bid, name, op=0.25, mono=False)
        x += w
    s.add(f'<rect x="24" y="78" width="3" height="60" fill="{TEXT}"><animate attributeName="x" values="24;{x - 4}" '
          f'dur="8s" repeatCount="indefinite"/></rect>')
    # A standard block's pulses.
    y = 210
    s.text(24, y - 36, 'inside a standard-speed block (ROM loader timings)', 12, TEXT, weight='600')
    path = [f'M24 {y + 40}']
    xx, level = 24, 0
    def pulse(n, t):
        nonlocal xx, level
        for _ in range(n):
            level ^= 1
            path.append(f'V{y + (0 if level else 40)}H{xx + t:.1f}')
            xx += t
    pulse(10, 14)
    seg1 = xx
    pulse(1, 5)
    pulse(1, 6)
    seg2 = xx
    for bit in '1011':
        pulse(2, 22 if bit == '1' else 11)
    path_d = ''.join(path)
    s.add(f'<path d="{path_d}" fill="none" stroke="{TEAL}" stroke-width="1.8"/>')
    s.text(24, y + 62, 'pilot tone', 11, DIM)
    s.text(24, y + 78, '2168 T per pulse', 11, DIM, mono=True)
    s.line(seg1 + 5, y - 2, seg1 + 5, y - 14, DIM, 1)
    s.text(seg1 - 30, y - 18, 'sync: 667, 735 T', 11, DIM, mono=True)
    s.text(seg2 + 4, y + 62, 'data: two pulses per bit', 11, DIM)
    s.text(seg2 + 4, y + 78, '0 = 855 T, 1 = 1710 T', 11, DIM, mono=True)
    s.text(seg2 + 4, y + 94, 'bits shown: 1 0 1 1', 11, DIM)
    lines(s, 520, y, ['TAP keeps only the data of standard blocks,', 'with no timing. A turbo or protected loader',
                      'cannot load from it.', '', 'TZX keeps the timing of each block, so the', 'tape loads in an emulator as it did on',
                      'the real machine.'], 12, TEXT)
    s.text(24, 362, 'CPC tapes (CDT) use the same structure, with the timings scaled by 4/3.5.', 12, DIM)
    return s.render()


# ================================================================ Acorn 8-bit
@diagram('bbc-memory-map.svg')
def bbc_memory_map():
    s = Svg(900, 400, 'The BBC Micro memory map has RAM below &8000 and one of up to sixteen sideways ROMs at &8000 to '
                      '&BFFF, chosen by ROMSEL at &FE30. The MOS and the FRED, JIM and SHEILA I/O pages sit above.')
    s.heading('BBC Micro sideways ROMs', 'ROMSEL at &FE30 picks which 16 KB ROM appears at &8000. The four sockets can be expanded to sixteen.')
    x, y = 110, 84
    memmap(s, x, y, 230, [('&0000', 'RAM (32 KB on the Model B)', 128, TEAL), ('&8000', 'paged ROM (sideways)', 64, AMBER),
                          ('&C000', 'MOS ROM', 48, GREY), ('&FC00', 'FRED, JIM, SHEILA I/O', 18, RED), ('&FF00', 'MOS', 14, GREY)])
    s.text(x, y + 292, 'SHEILA at &FE00 holds the chips\' registers.', 11, DIM)
    # The sixteen sockets, one selected at a time.
    px, py = 470, 100
    s.text(px, py, 'ROMSEL value: one of 16 ROM banks', 12, TEXT, weight='600')
    labels = {15: 'BASIC', 14: 'DFS', 13: 'game', 12: 'utility'}
    dur = 6.0
    order = [15, 14, 13, 15]
    for k in range(16):
        bx, by = px + (k % 8) * 50, py + 14 + (k // 8) * 52
        s.rect(bx, by, 46, 44, AMBER, rx=3, extra=' fill-opacity="0.18"')
        s.text(bx + 23, by + 20, f'{k}', 12, TEXT, 'middle', '600', mono=True)
        if k in labels:
            s.text(bx + 23, by + 36, labels[k], 9, DIM, 'middle')
        keys = [i / len(order) for i, v in enumerate(order) if v == k]
        if keys:
            vals, kt = [], []
            for i, v in enumerate(order):
                kt.append(f'{i / len(order):.3f}')
                vals.append('1' if v == k else '0')
            kt.append('1')
            vals.append(vals[-1])
            s.add(f'<rect x="{bx}" y="{by}" width="46" height="44" rx="3" fill="none" stroke="{AMBER}" stroke-width="2.5" '
                  f'opacity="0"><animate attributeName="opacity" values="{";".join(vals)}" keyTimes="{";".join(kt)}" '
                  f'calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.text(px, py + 136, 'Which ROMs a machine has differs. These labels are examples.', 11, DIM)
    s.text(px, 270, 'Disk controller by model', 12, TEXT, weight='600')
    box(s, px, 282, 190, 44, BLUE, 'Intel 8271', 'Model A and B', op=0.22)
    box(s, px + 210, 282, 190, 44, PURPLE, 'WD1770', 'B+ and Master', op=0.22)
    lines(s, px, 350, ['Some disk protection works on one controller and fails', 'on the other. Record the model, the MOS version,',
                       'the fitted ROMs and ROMSEL for paged code.'], 11)
    return s.render()


@diagram('electron-ram.svg')
def electron_ram():
    s = Svg(900, 300, 'The Acorn Electron\'s RAM is 4 bits wide, so each byte takes two reads. The CPU runs at an '
                      'effective 1 MHz or less when it touches RAM.')
    s.heading('The Acorn Electron\'s 4-bit RAM', 'One ULA does video and sound. Its RAM is 4 bits wide, so a byte is read in two halves.')
    box(s, 40, 100, 140, 60, BLUE, '6502', 'CPU', op=0.25)
    box(s, 260, 100, 160, 60, PURPLE, 'ULA', 'video, sound, RAM access', op=0.25)
    box(s, 500, 100, 140, 60, TEAL, 'RAM', '32 KB, 4 bits wide', op=0.25)
    arrow(s, 184, 130, 256, 130)
    arrow(s, 424, 130, 496, 130)
    dur = 2.0
    s.text(500, 196, 'one byte from RAM', 12, TEXT, weight='600')
    for k, (lab, colour) in enumerate((('read 1: low 4 bits', TEAL), ('read 2: high 4 bits', AMBER))):
        bx = 500 + k * 140
        s.rect(bx, 206, 136, 32, colour, rx=3, extra=' fill-opacity="0.2"')
        s.text(bx + 68, 227, lab, 11, TEXT, 'middle')
        s.add(f'<rect x="{bx}" y="206" width="136" height="32" rx="3" fill="{colour}" fill-opacity="0.6" opacity="0">'
              f'<animate attributeName="opacity" values="{"1;0" if k == 0 else "0;1"}" keyTimes="0;0.5" calcMode="discrete" '
              f'dur="{dur}s" repeatCount="indefinite"/></rect>')
    lines(s, 680, 112, ['From ROM: full speed.', 'From RAM: an effective', '1 MHz or less.'], 12, TEXT)
    s.text(40, 270, 'The BBC Micro\'s timing does not fit the Electron. A port must model the slower RAM.', 12, DIM)
    return s.render()


# ================================================================ Commodore
@diagram('commodore-pal-ntsc.svg')
def commodore_pal_ntsc():
    s = Svg(900, 360, 'The VIC-20\'s 6560 and 6561 run at different clocks. The C64\'s VIC-II revisions have different '
                      'numbers of lines and cycles per line. PAL and NTSC builds of a game therefore differ.')
    s.heading('PAL and NTSC builds', 'The video chip sets the CPU clock and the frame. A PAL build and an NTSC build are often separate programs.')
    s.text(24, 96, 'VIC-20: the VIC chip', 13, TEXT, weight='600')
    box(s, 24, 108, 200, 50, BLUE, '6560 (NTSC)', '1.0227 MHz', op=0.22, mono=True)
    box(s, 236, 108, 200, 50, AMBER, '6561 (PAL)', '1.1087 MHz', op=0.22, mono=True)
    s.text(24, 196, 'C64: the VIC-II', 13, TEXT, weight='600')
    chips = [('6569 (PAL)', 312, 63, AMBER), ('6567 (NTSC)', 263, 65, BLUE), ('6567 R56A (early NTSC)', 262, 64, PURPLE)]
    for k, (name, nl, cyc, colour) in enumerate(chips):
        y = 210 + k * 40
        s.text(24, y + 20, name, 12, TEXT)
        w = nl * cyc / 19656 * 520
        s.rect(200, y + 4, w, 26, colour, rx=3, extra=' fill-opacity="0.35"')
        s.text(208, y + 22, f'{nl} lines x {cyc} cycles = {nl * cyc} cycles a frame', 11, TEXT, mono=True)
    s.text(24, 344, 'The PAL frame runs at 50.12 Hz. Record the VIC-II revision of the reference recording.', 12, DIM)
    s.text(200, 196, 'The 6510 runs at 1.023 MHz (NTSC) or 0.985 MHz (PAL).', 12, DIM)
    return s.render()


@diagram('c64-banking.svg')
def c64_banking():
    s = Svg(900, 420, 'The low three bits of the C64\'s 6510 port at $01 decide whether BASIC, the KERNAL, the I/O '
                      'chips or the character ROM appear over RAM at $A000, $D000 and $E000.')
    s.heading('The C64 $01 port picks the ROMs', 'The low three bits of $01 (LORAM, HIRAM, CHAREN) switch ROM and I/O over RAM. The map shows a C64 with no cartridge.')
    configs = [('$37', 'BASIC', 'I/O', 'KERNAL', 'the default at power on'),
               ('$36', 'RAM', 'I/O', 'KERNAL', 'BASIC switched out'),
               ('$35', 'RAM', 'I/O', 'RAM', 'only I/O left: common in games'),
               ('$34', 'RAM', 'RAM', 'RAM', 'all 64 KB RAM, no I/O'),
               ('$33', 'BASIC', 'char ROM', 'KERNAL', 'the character ROM at $D000')]
    colour = {'BASIC': BLUE, 'I/O': RED, 'KERNAL': PURPLE, 'RAM': TEAL, 'char ROM': AMBER}
    def draw(cfg):
        val, a, d, e, note = cfg
        def body(t):
            x, y = 180, 80
            regions = [('$0000', 'RAM, 40 KB', 200, TEAL), ('$A000', a, 40, colour[a]), ('$C000', 'RAM', 20, TEAL),
                       ('$D000', d, 20, colour[d]), ('$E000', e, 40, colour[e])]
            memmap(t, x, y, 220, regions)
            t.text(470, 120, f'$01 = {val}', 20, AMBER, weight='600', mono=True)
            bits = format(int(val[1:], 16) & 7, '03b')
            for k, (name, b) in enumerate(zip(('CHAREN', 'HIRAM', 'LORAM'), bits)):
                t.rect(470 + k * 96, 140, 90, 44, AMBER, rx=3, extra=' fill-opacity="0.2"')
                t.text(515 + k * 96, 158, name, 10, DIM, 'middle')
                t.text(515 + k * 96, 177, b, 14, TEXT, 'middle', '600', mono=True)
            t.text(470, 216, note, 13, TEXT)
        return part(body)
    s.add(show([draw(c) for c in configs], 10))
    lines(s, 470, 270, ['The I/O area holds the VIC-II at $D000, the SID at', '$D400 and the CIAs at $DC00 and $DD00.',
                        'CIA 2 picks the VIC-II\'s 16 KB bank.', '', 'Record $01 and the cartridge lines for every',
                        'code region you load into Ghidra.'], 12)
    return s.render()


@diagram('c64-badlines.svg')
def c64_badlines():
    s = Svg(900, 400, 'A PAL C64 frame is 312 lines of 63 cycles. On every eighth line of the text area the VIC-II '
                      'stops the CPU for about 40 cycles to fetch the character pointers.')
    s.heading('C64 badlines (PAL)', '312 lines of 63 cycles = 19656 cycles a frame. On a badline the VIC-II takes about 40 of them.')
    fx, fy, fw, fh = 40, 86, 300, 260
    lh = fh / 312
    s.rect(fx, fy, fw, fh, GREY, rx=3, extra=' fill-opacity="0.2"')
    top = 51
    s.rect(fx + 30, fy + top * lh, fw - 60, 200 * lh, BLUE, rx=2, extra=' fill-opacity="0.18"')
    for r in range(25):
        s.rect(fx + 30, fy + (top + r * 8) * lh, fw - 60, max(lh, 1.2), RED, rx=0, extra=' fill-opacity="0.9"')
    s.text(fx + fw / 2, fy + fh + 18, 'red: badlines, one per text row (25 rows)', 11, DIM, 'middle')
    s.add(f'<rect x="{fx}" y="{fy}" width="{fw}" height="2" fill="{AMBER}"><animate attributeName="y" values="{fy};{fy + fh - 2}" '
          f'dur="4s" repeatCount="indefinite"/></rect>')
    px = 400
    s.text(px, 110, 'one normal line: 63 cycles', 12, TEXT, weight='600')
    for k in range(63):
        s.rect(px + k * 7.5, 120, 6.5, 22, TEAL, rx=1, extra=' fill-opacity="0.55"')
    s.text(px, 176, 'one badline', 12, TEXT, weight='600')
    for k in range(63):
        stolen = 12 <= k < 52
        s.rect(px + k * 7.5, 186, 6.5, 22, RED if stolen else TEAL, rx=1, extra=f' fill-opacity="{0.8 if stolen else 0.55}"')
    s.text(px + 32 * 7.5, 226, 'about 40 cycles for the VIC-II', 11, RED, 'middle')
    lines(s, px, 262, ['Sprites take more cycles on the lines they appear on.', 'Raster interrupts fire on a chosen line. Games use them',
                       'to change colours, split the screen or play music.', '',
                       'A port that runs game logic per frame must give it the', 'same cycles the real frame left over.'], 12)
    return s.render()


@diagram('c64-1541.svg')
def c64_1541():
    s = Svg(900, 400, 'The 1541 drive has its own 6502. A fast loader uploads code into the drive\'s RAM and runs it. '
                      'A D64 image holds 683 sectors in four speed zones, and a G64 image keeps the GCR tracks.')
    s.heading('C64 and the 1541 drive', 'The 1541 drive has its own 6502 and ROM. Fast loaders and protections run their own code on it.')
    box(s, 24, 90, 200, 70, BLUE, 'C64', '6510 CPU', op=0.22)
    box(s, 400, 90, 220, 70, PURPLE, '1541 drive', '6502, 2 KB RAM, DOS ROM', op=0.22)
    s.line(228, 125, 396, 125, DIM, 2)
    s.text(312, 116, 'serial bus', 11, DIM, 'middle')
    dur = 4.0
    s.add(f'<rect x="230" y="134" width="40" height="16" rx="3" fill="{AMBER}"><animate attributeName="x" values="230;354;354" '
          f'keyTimes="0;0.5;1" dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.text(312, 172, 'loader code goes to the drive', 11, AMBER, 'middle')
    s.add(fade(f'<text x="510" y="180" font-family="{SANS}" font-size="12" fill="{AMBER}" text-anchor="middle" '
               f'font-weight="600">the drive runs it</text>', [(0.5, 1.0)], dur))
    lines(s, 650, 106, ['Fast loaders: Novaload,', 'Freeload, Cyberload, Vorpal.', 'Protections: bad sectors,',
                        'half tracks, density changes.'], 11, TEXT)
    # The D64 zones.
    s.text(24, 222, 'D64: 35 tracks, fewer sectors toward the centre', 12, TEXT, weight='600')
    zones = [(1, 17, 21, TEAL), (18, 24, 19, BLUE), (25, 30, 18, PURPLE), (31, 35, 17, AMBER)]
    x = 24
    for a, b, n, colour in zones:
        for t in range(a, b + 1):
            h = n * 4
            s.rect(x, 316 - h, 22, h, colour, rx=2, extra=' fill-opacity="0.45"')
            if t == 18:
                s.rect(x, 316 - h, 22, h, 'none', TEXT, rx=2, extra=' stroke-width="1.5"')
            x += 24
    x = 24
    for a, b, n, colour in zones:
        s.text(x + (b - a + 1) * 12, 334, f'{a}-{b}: {n}', 11, colour, 'middle', mono=True)
        x += (b - a + 1) * 24
    s.text(24, 362, '17 x 21 + 7 x 19 + 6 x 18 + 5 x 17 = 683 sectors x 256 bytes = 174,848 bytes. Track 18 (outlined) holds the directory.',
           11, DIM)
    s.text(24, 384, 'D64 keeps only the sector data. G64 keeps the GCR track bits, so protected disks still load.', 12, TEXT)
    return s.render()


# ================================================================ Amstrad CPC
@diagram('cpc-nop-timing.svg')
def cpc_nop_timing():
    s = Svg(900, 360, 'On the Amstrad CPC, wait states stretch every instruction to a multiple of 4 T-states, so time '
                      'is counted in NOPs of 1 microsecond. A Z80 cycle table from another machine gives the wrong '
                      'times.')
    s.heading('CPC time counted in NOPs', 'The gate array stretches every instruction to a multiple of 4 T-states. One NOP = 4 T-states = 1 microsecond.')
    rows = [('NOP', 4, 1), ('LD A,B', 4, 1), ('INC HL', 6, 2), ('LD A,(HL)', 7, 2), ('LD HL,nn', 10, 3), ('JP nn', 10, 3)]
    x0, y0, unit = 200, 104, 22
    s.text(24, y0 - 14, 'instruction', 11, DIM, weight='600')
    s.text(x0, y0 - 14, 'Z80 T-states (teal), wait states added on the CPC (amber)', 11, DIM, weight='600')
    s.text(650, y0 - 14, 'on a CPC', 11, DIM, weight='600')
    for r, (ins, t, nops) in enumerate(rows):
        y = y0 + r * 34
        s.text(24, y + 17, ins, 13, TEXT, mono=True)
        for k in range(nops * 4):
            colour = TEAL if k < t else AMBER
            s.rect(x0 + k * unit, y, unit - 3, 24, colour, rx=2, extra=f' fill-opacity="{0.55 if k < t else 0.7}"')
        for k in range(1, nops):
            s.line(x0 + k * 4 * unit - 1.5, y - 3, x0 + k * 4 * unit - 1.5, y + 27, TEXT, 1)
        s.text(650, y + 17, f'{nops} NOP{"s" if nops > 1 else ""} = {nops} microsecond{"s" if nops > 1 else ""}', 12, TEXT)
    lines(s, 24, y0 + 6 * 34 + 24, ['The Z80A runs at 4 MHz. With the wait states it gives about 3.3 MHz of work.',
                                    'Count a routine\'s time in NOPs. A Z80 cycle table from another machine gives the wrong times.'], 12)
    return s.render()


@diagram('cpc-interrupts.svg')
def cpc_interrupts():
    s = Svg(900, 380, 'The CPC gate array raises an interrupt every 52 lines, six times in a 312-line frame at 50 Hz. '
                      'Games time split screens from these interrupts. The CRTC type changes the edges of the picture.')
    s.heading('Six CPC interrupts a frame', 'The gate array interrupts every 52 lines. 6 x 52 = 312 lines, one 50 Hz frame.')
    fx, fy, fw, fh = 40, 86, 300, 250
    s.rect(fx, fy, fw, fh, GREY, rx=3, extra=' fill-opacity="0.2"')
    split = [BLUE, BLUE, PURPLE, PURPLE, TEAL, TEAL]
    for k in range(6):
        y = fy + k * fh / 6
        s.rect(fx + 2, y + 1, fw - 4, fh / 6 - 2, split[k], rx=2, extra=' fill-opacity="0.18"')
        s.line(fx - 10, y, fx + fw + 10, y, AMBER, 1.5)
        s.text(fx + fw + 16, y + 4, f'interrupt {k + 1}' + (f': {k * 52} lines later' if k else ''), 11, AMBER, mono=True)
    s.add(f'<rect x="{fx}" y="{fy}" width="{fw}" height="2" fill="{TEXT}"><animate attributeName="y" values="{fy};{fy + fh - 2}" '
          f'dur="3s" repeatCount="indefinite"/></rect>')
    s.text(fx, fy + fh + 26, 'A game can change the mode or palette at each interrupt.', 11, DIM)
    lines(s, 560, 104, ['Modes, chosen by the gate array:', '160 x 200 in 16 colours', '320 x 200 in 4 colours',
                        '640 x 200 in 2 colours'], 12, TEXT)
    lines(s, 560, 200, ['The 6845 CRTC comes in types 0 to 4.', 'They differ at the edges of the picture.',
                        'Split screens and overscan depend on', 'those differences, so record the CRTC', 'type of the reference machine.'], 12)
    return s.render()


# ================================================================ MSX and Japanese 8-bit
@diagram('msx-slots.svg')
def msx_slots():
    s = Svg(900, 430, 'On the MSX the Z80\'s 64 KB is four 16 KB pages. Port A8h picks which of four primary slots '
                      'each page comes from. A cartridge game is switched in this way.')
    s.heading('MSX pages and slots', 'Port A8h has 2 bits per page. Each 16 KB page can come from any of the four primary slots.')
    pages = ['0000-3FFF', '4000-7FFF', '8000-BFFF', 'C000-FFFF']
    slots = [('slot 0', 'BIOS and BASIC ROM', GREY), ('slot 1', 'cartridge', AMBER), ('slot 2', 'cartridge', PURPLE),
             ('slot 3', 'RAM', TEAL)]
    gx, gy, cw, ch = 150, 110, 150, 52
    for k, (name, what, colour) in enumerate(slots):
        s.text(gx + k * cw + cw / 2, gy - 22, name, 12, TEXT, 'middle', '600')
        s.text(gx + k * cw + cw / 2, gy - 8, what, 10, DIM, 'middle')
    for p in range(4):
        s.text(gx - 12, gy + p * ch + 31, f'page {p}', 11, TEXT, 'end')
        s.text(gx - 12, gy + p * ch + 45, pages[p], 9, DIM, 'end', mono=True)
        for k, (_, _, colour) in enumerate(slots):
            s.rect(gx + k * cw + 4, gy + p * ch + 4, cw - 8, ch - 8, colour, rx=3, extra=' fill-opacity="0.12"')
    states = [('F0h', [0, 0, 3, 3], 'BASIC: ROM in pages 0 and 1, RAM above'),
              ('F4h', [0, 1, 3, 3], 'a 16 KB cartridge in slot 1 at 4000h'),
              ('F8h', [0, 2, 3, 3], 'the same with the cartridge in slot 2')]
    def draw(state):
        val, sel, note = state
        def body(t):
            for p, k in enumerate(sel):
                t.rect(gx + k * cw + 4, gy + p * ch + 4, cw - 8, ch - 8, slots[k][2], rx=3, extra=' fill-opacity="0.75"')
            t.text(gx, gy + 4 * ch + 30, f'A8h = {val}', 16, AMBER, weight='600', mono=True)
            bits = format(int(val[:-1], 16), '08b')
            for i in range(4):
                t.text(gx + 150 + i * 70, gy + 4 * ch + 30, f'p{3 - i}: {bits[i * 2:i * 2 + 2]}', 12, TEXT, mono=True)
            t.text(gx, gy + 4 * ch + 54, note, 12, TEXT)
        return part(body)
    s.add(show([draw(st) for st in states], 7.5))
    s.text(gx, 414, 'This is a typical layout, and machines differ. Subslots are chosen through the register at FFFFh.', 11, DIM)
    return s.render()


@diagram('msx-megarom.svg')
def msx_megarom():
    s = Svg(900, 370, 'The Konami, Konami SCC, ASCII 8K and ASCII 16K mappers each split 4000h to BFFFh into banks. '
                      'Each switches its banks by writes to different addresses, so an emulator needs the right mapper '
                      'type.')
    s.heading('MSX MegaROM mappers', 'A cartridge bigger than 32 KB switches banks into 4000h-BFFFh. Each mapper uses different windows and registers.')
    x0, y0, span = 230, 104, 600
    for k, a in enumerate(('4000h', '6000h', '8000h', 'A000h', 'C000h')):
        s.text(x0 + k * span / 4, y0 - 10, a, 10, DIM, 'middle', mono=True)
    mappers = [('Konami', [('fixed: bank 0', 1, GREY), ('6000h', 1, TEAL), ('8000h', 1, TEAL), ('A000h', 1, TEAL)]),
               ('Konami SCC', [('5000h', 1, AMBER), ('7000h', 1, AMBER), ('9000h', 1, AMBER), ('B000h', 1, AMBER)]),
               ('ASCII 8K', [('6000h', 1, BLUE), ('6800h', 1, BLUE), ('7000h', 1, BLUE), ('7800h', 1, BLUE)]),
               ('ASCII 16K', [('6000h', 2, PURPLE), ('7000h', 2, PURPLE)])]
    for r, (name, wins) in enumerate(mappers):
        y = y0 + r * 50
        s.text(24, y + 26, name, 13, TEXT, weight='600')
        x = x0
        for label, n, colour in wins:
            w = n * span / 4
            s.rect(x, y, w - 4, 38, colour, rx=3, extra=' fill-opacity="0.28"')
            s.text(x + w / 2 - 2, y + 17, 'write to' if 'fixed' not in label else '', 9, DIM, 'middle')
            s.text(x + w / 2 - 2, y + 31, label, 11, TEXT, 'middle', mono='fixed' not in label)
            x += w
    lines(s, 24, y0 + 4 * 50 + 22, ['Each window shows one bank of the ROM. A write to the address in the box picks the bank.',
                                    'Konami SCC cartridges also hold the SCC sound chip. Name functions by bank and address.'], 12)
    return s.render()


@diagram('japan-8bit.svg')
def japan_8bit():
    s = Svg(900, 300, 'The NEC PC-8801 and the Sharp X1 are Z80 machines with their own video and sound chips. The '
                      'PC-8801\'s disk unit has a second Z80 that runs its own program.')
    s.heading('PC-8801 and Sharp X1', 'These were two of Japan\'s four major home computers. The other two were the FM-7 and the MSX.')
    s.rect(24, 80, 416, 196, BLUE, rx=8, extra=' fill-opacity="0.12"')
    s.text(40, 106, 'NEC PC-8801 (November 1981)', 14, TEXT, weight='600')
    box(s, 40, 122, 150, 54, BLUE, 'Z80', 'main CPU, N88-BASIC', op=0.25)
    box(s, 270, 122, 150, 54, PURPLE, 'disk unit', 'its own Z80', op=0.25)
    arrow(s, 194, 149, 266, 149)
    s.text(40, 204, 'Sound: YM2203 from the mkII SR, later YM2608.', 12, TEXT)
    lines(s, 40, 228, ['The second Z80 runs its own code (not confirmed).', 'Treat it as a separate program.'], 11)
    s.rect(460, 80, 416, 196, AMBER, rx=8, extra=' fill-opacity="0.12"')
    s.text(476, 106, 'Sharp X1 (1982; X1 turbo 1984)', 14, TEXT, weight='600')
    box(s, 476, 122, 120, 54, AMBER, 'Z80A', '4 MHz', op=0.25)
    box(s, 606, 122, 120, 54, BLUE, 'HD46505', 'CRTC', op=0.25)
    box(s, 736, 122, 120, 54, PURPLE, 'PCG', 'programmable chars', op=0.25)
    s.text(476, 204, 'Sound: AY-3-8910.', 12, TEXT)
    lines(s, 476, 228, ['These X1 facts are not confirmed. Check a second', 'source before you model the timing.'], 11)
    return s.render()


# ================================================================ Atari 8-bit
@diagram('atari8-display-list.svg')
def atari8_display_list():
    s = Svg(900, 420, 'ANTIC reads an Atari display list with one instruction per mode line, such as blank lines, a '
                      'text mode with a screen address, and a jump that waits for vertical blank. A display list '
                      'interrupt changes a colour part way down the screen.')
    s.heading('The Atari display list', 'ANTIC runs a small program each frame. Each instruction sets the mode of one or more lines.')
    ins = [('70 70 70', '24 blank lines', GREY), ('42 00 40', 'text mode 2, screen at $4000', BLUE), ('02 x 10', '10 more text rows', BLUE),
           ('82', 'text row with the DLI bit ($80)', AMBER), ('0F x 40', '40 lines of hi-res mode F', PURPLE),
           ('41 00 06', 'jump to the list, wait for VBLANK', GREY)]
    y = 92
    for code, what, colour in ins:
        s.rect(24, y, 340, 34, colour, rx=3, extra=' fill-opacity="0.2"')
        s.text(36, y + 22, code, 13, TEXT, weight='600', mono=True)
        s.text(130, y + 22, what, 11, TEXT)
        y += 40
    sx, sy, sw = 470, 92, 380
    bands = [(24, GREY, 0.15), (88, BLUE, 0.3), (8, AMBER, 0.5), (40, PURPLE, 0.3)]
    yy = sy
    for h, colour, op in bands:
        s.rect(sx, yy, sw, h * 1.3, colour, rx=0, extra=f' fill-opacity="{op}"')
        yy += h * 1.3
    total = yy - sy
    # After the DLI the background colour changes for the rest of the screen.
    dli_y = sy + (24 + 88 + 8) * 1.3
    dur = 3.0
    frac = (dli_y - sy) / total
    s.add(f'<rect x="{sx}" y="{dli_y}" width="{sw}" height="{sy + total - dli_y}" fill="{RED}" fill-opacity="0">'
          f'<animate attributeName="fill-opacity" values="0;0.35;0" keyTimes="0;{frac:.3f};0.999" calcMode="discrete" '
          f'dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.add(f'<rect x="{sx}" y="{sy}" width="{sw}" height="2" fill="{TEXT}"><animate attributeName="y" values="{sy};{sy + total - 2}" '
          f'dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.text(sx + sw + 6, dli_y + 4, 'DLI', 11, AMBER, weight='600')
    s.text(sx, sy + total + 20, 'The DLI handler writes WSYNC, then a new colour.', 11, DIM)
    s.text(sx, sy + total + 36, 'Everything below it shows the new colour.', 11, DIM)
    lines(s, 24, 372, ['Record the display list and every DLI. Together they set the mode and colours of each line.',
                       'Display list interrupts are enabled through NMIEN.'], 12)
    return s.render()


@diagram('atari8-memory.svg')
def atari8_memory():
    s = Svg(900, 360, 'On the 130XE, PORTB at $D301 pages one of four extra 16 KB banks into $4000 to $7FFF for the '
                      'CPU or for ANTIC. ANTIC also takes cycles from each line of 114.')
    s.heading('130XE banking and ANTIC DMA', 'PORTB at $D301 pages extra RAM into a 16 KB window at $4000. ANTIC takes CPU cycles on every line.')
    bx = 24
    s.text(24, 96, 'PORTB ($D301) on the 130XE', 12, TEXT, weight='600')
    for label, n, colour in (('self-test', 1, GREY), ('-', 1, GREY), ('ANTIC', 1, BLUE), ('CPU', 1, TEAL), ('bank', 2, AMBER),
                             ('BASIC', 1, GREY), ('OS ROM', 1, GREY)):
        w = n * 58
        s.rect(bx, 106, w - 2, 32, colour, rx=3, extra=' fill-opacity="0.3"')
        s.text(bx + w / 2 - 1, 126, label, 11, TEXT, 'middle', '600')
        bx += w
    lines(s, 24, 160, ['bits 2-3 pick one of four 16 KB banks.', 'bit 4 = 0: the CPU sees the bank at $4000.',
                       'bit 5 = 0: ANTIC sees the bank at $4000.', 'A game can show one bank while it draws in another.'], 12)
    memmap(s, 620, 92, 180, [('$0000', 'RAM', 34, TEAL), ('$4000', 'window: bank 0-3', 34, AMBER), ('$8000', 'RAM', 34, TEAL),
                             ('$C000', 'OS ROM, I/O', 34, GREY)])
    s.text(24, 262, 'one scan line: 114 machine cycles', 12, TEXT, weight='600')
    segs = [('display list', 3, GREY), ('refresh: 9', 9, PURPLE), ('screen data: 40 bytes in a 40-column line', 40, BLUE),
            ('left for the CPU', 62, TEAL)]
    x = 24
    for label, n, colour in segs:
        w = n * 7.4
        s.rect(x, 272, w - 2, 30, colour, rx=3, extra=' fill-opacity="0.45"')
        if n > 8:
            s.text(x + w / 2, 292, label, 11, TEXT, 'middle')
        x += w
    s.text(24, 326, 'An example line. ANTIC takes more cycles on some lines of a text mode. Model them if game logic depends on CPU time.', 11, DIM)
    return s.render()


# ================================================================ Apple II
@diagram('apple2-memory-map.svg')
def apple2_memory_map():
    s = Svg(900, 432, 'The Apple II memory map holds zero page, the stack, the text and hi-res screen pages, the soft '
                      'switches at $C000 to $C0FF and the language card RAM that can replace the ROM at $D000 to '
                      '$FFFF.')
    s.heading('Apple II memory and soft switches', 'Reading or writing an address in $C000-$C0FF flips a switch. The language card banks RAM over the ROM.')
    x, y = 110, 84
    memmap(s, x, y, 230, [('$0000', 'zero page', 16, PURPLE), ('$0100', 'stack', 16, PURPLE), ('$0200', 'RAM', 18, TEAL),
                          ('$0400', 'text / lo-res page 1', 24, BLUE), ('$0800', 'RAM', 40, TEAL),
                          ('$2000', 'hi-res page 1', 40, AMBER), ('$4000', 'hi-res page 2', 40, AMBER),
                          ('$6000', 'RAM', 56, TEAL), ('$C000', 'soft switches, I/O', 18, RED), ('$C100', 'slot ROMs', 16, GREY),
                          ('$D000', 'ROM, or language card RAM', 44, GREEN)])
    px = 440
    s.text(px, 100, 'some soft switches', 12, TEXT, weight='600')
    sw = [('$C030', 'click the speaker (1 bit)'), ('$C050 / $C051', 'graphics / text'), ('$C052 / $C053', 'full screen / mixed'),
          ('$C054 / $C055', 'page 1 / page 2'), ('$C056 / $C057', 'lo-res / hi-res'), ('$C080-$C08F', 'language card banks')]
    for k, (addr, what) in enumerate(sw):
        s.text(px, 126 + k * 22, addr, 12, AMBER, mono=True)
        s.text(px + 130, 126 + k * 22, what, 12, TEXT)
    lines(s, px, 280, ['Hi-res is 280 x 192 with six visible colours.', 'The IIgs adds the 65C816 and an Ensoniq 5503 (not confirmed).',
                       '', 'Record the soft switch and language card state', 'for each code region.'], 12)
    return s.render()


@diagram('apple2-disk.svg')
def apple2_disk():
    s = Svg(900, 360, 'On the Disk II, software controls the GCR encoding, so publishers wrote their own formats. A '
                      'DSK image keeps only 35 tracks of 16 sectors. A WOZ image keeps the bits and timing of each '
                      'track.')
    s.heading('Disk II and its images', 'Software writes the bits on a Disk II, so a publisher could change the format as protection.')
    cx, cy = 150, 210
    for t in range(35):
        r = 120 - t * 3
        colour = RED if t == 17 else TEAL
        s.add(f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="none" stroke="{colour}" stroke-opacity="{0.7 if t == 17 else 0.25}" '
              f'stroke-width="{2 if t == 17 else 1}"/>')
    s.add(f'<circle cx="{cx}" cy="{cy}" r="14" fill="{BG}" stroke="{DIM}"/>')
    s.add(f'<g><path d="M{cx} {cy - 120}L{cx} {cy - 18}" stroke="{AMBER}" stroke-width="2"/>'
          f'<animateTransform attributeName="transform" type="rotate" values="0 {cx} {cy};360 {cx} {cy}" dur="3s" '
          f'repeatCount="indefinite"/></g>')
    s.text(cx, cy + 140, '35 tracks; track 17 (red) holds the catalog', 11, DIM, 'middle')
    px = 330
    s.text(px, 100, 'what each image keeps', 13, TEXT, weight='600')
    box(s, px, 112, 250, 64, GREY, 'DSK, DO, PO', '35 x 16 sectors x 256 = 140 KB', op=0.25)
    box(s, px + 270, 112, 250, 64, GREEN, 'WOZ', 'the bits of each track and their timing', op=0.25)
    lines(s, px, 210, ['Protections a DSK image loses:', '- custom sector formats', '- nibble counts', '- spiral tracks',
                       '- changed address and data marks'], 12, TEXT)
    lines(s, px + 270, 210, ['GCR: each byte on disk has its', 'high bit set. DOS 3.3 packs 6 bits', 'into each disk byte (6-and-2).',
                             '', 'Keep the WOZ image as the record', 'of a protected disk.'], 12)
    return s.render()


# ================================================================ other 8-bit
@diagram('coco-dragon.svg')
def coco_dragon():
    s = Svg(900, 330, 'The Dragon and the TRS-80 Color Computer use a 6809E with the MC6883 SAM for memory and timing '
                      'and the MC6847 VDG for video. The CoCo 3 replaces them with the GIME. A speed-up poke doubles '
                      'the clock.')
    s.heading('Dragon 32/64 and CoCo', 'The SAM shares the RAM between the CPU and the video chip and sets the clock.')
    box(s, 40, 100, 150, 60, BLUE, '6809E', '0.89 MHz (Dragon 32)', op=0.25)
    box(s, 300, 100, 170, 60, AMBER, 'MC6883 SAM', 'memory and timing', op=0.25)
    box(s, 580, 100, 150, 60, PURPLE, 'MC6847 VDG', 'video', op=0.25)
    box(s, 300, 210, 170, 50, TEAL, 'RAM', '4 KB to 512 KB across the range', op=0.25)
    arrow(s, 194, 130, 296, 130)
    arrow(s, 474, 130, 576, 130)
    arrow(s, 385, 164, 385, 206)
    s.text(40, 200, 'sound: a 6-bit DAC', 12, TEXT)
    lines(s, 590, 200, ['CoCo 3 (1986-1991):', 'the GIME chip replaces', 'SAM and VDG, with a', '64-colour palette.'], 12, TEXT)
    s.text(40, 294, 'A "speed-up poke" doubles the clock on some machines. Record whether the game sets it, and the model.', 12, DIM)
    return s.render()


@diagram('oric-attributes.svg')
def oric_attributes():
    s = Svg(900, 300, 'On the Oric, an attribute byte in screen memory takes a character cell and shows as a blank. It '
                      'changes the ink or paper colour for the rest of the line.')
    s.heading('Oric serial attributes', 'An attribute is a byte in the screen itself. It takes one cell and changes the colours from there to the line\'s end.')
    cells = ['A', 'B', 'ink', 'C', 'D', 'E', 'paper', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N']
    x0, y0, c = 40, 110, 50
    ink, paper = '#ffffff', '#000000'
    for k, ch in enumerate(cells):
        if ch == 'ink':
            ink = '#ffff55'
        if ch == 'paper':
            paper = '#2b2bd7'
        x = x0 + k * c
        s.rect(x, y0, c - 2, 60, paper, rx=0)
        if ch in ('ink', 'paper'):
            s.rect(x + 2, y0 + 2, c - 6, 56, 'none', AMBER, rx=2, extra=' stroke-width="2"')
            s.text(x + c / 2 - 1, y0 - 8, ch, 11, AMBER, 'middle', '600')
        else:
            s.text(x + c / 2 - 1, y0 + 40, ch, 26, ink, 'middle', '700', mono=True)
    lines(s, 40, 210, ['Each attribute cell shows as blank paper. A colour change needs a free cell, so', 'two colours cannot touch inside a line.',
                       'The screen is 40 x 28 text or 240 x 200 hi-res, from the HCS 10017 ULA.'], 12)
    s.text(40, 280, 'Some games depend on BASIC ROM v1.0. Record the ROM version with the ground truth.', 12, TEXT)
    return s.render()


@diagram('ti99-workspace.svg')
def ti99_workspace():
    s = Svg(900, 360, 'The TMS9900 keeps only PC, WP and ST on the chip. Its sixteen registers are sixteen words in '
                      'RAM at the workspace pointer. BLWP switches to a new workspace and saves the old WP, PC and ST '
                      'in R13 to R15.')
    s.heading('TI-99/4A registers in RAM', 'The TMS9900 holds only PC, WP and ST. R0 to R15 are 16 words of RAM starting at WP.')
    for k, name in enumerate(('PC', 'WP', 'ST')):
        box(s, 40, 96 + k * 50, 110, 40, AMBER, name, op=0.25)
    s.text(40, 266, 'on the chip', 11, DIM)
    def ws(x, label, extra):
        s.text(x, 92, label, 12, TEXT, weight='600')
        for r in range(16):
            colour = PURPLE if r >= 13 and extra else TEAL
            s.rect(x, 100 + r * 15, 130, 13, colour, rx=2, extra=f' fill-opacity="{0.5 if r >= 13 and extra else 0.2}"')
            s.text(x + 6, 110 + r * 15, f'R{r}', 9, TEXT, mono=True)
        if extra:
            for r, v in ((13, 'old WP'), (14, 'old PC'), (15, 'old ST')):
                s.text(x + 124, 110 + r * 15, v, 9, TEXT, 'end', mono=True)
    ws(250, 'workspace A (old WP)', False)
    ws(520, 'workspace B (new WP)', True)
    s.add(f'<path d="M150 166C 200 166 210 110 246 110" fill="none" stroke="{AMBER}" stroke-width="1.5" marker-end="url(#arrow)"/>')
    dur = 3.0
    s.add(f'<path d="M384 180L512 180" stroke="{AMBER}" stroke-width="2" marker-end="url(#arrow)" opacity="0">'
          f'<animate attributeName="opacity" values="0;1;1" keyTimes="0;0.4;1" calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/></path>')
    s.add(fade(f'<text x="448" y="172" font-family="{SANS}" font-size="12" fill="{AMBER}" text-anchor="middle" font-weight="600">BLWP</text>',
               [(0.4, 1.0)], dur))
    lines(s, 690, 110, ['A register read is a', 'memory read. Trace the', 'workspace RAM with', 'the function.', '',
                        'Record WP at the entry', 'of each function you', 'emulate.'], 12)
    lines(s, 690, 270, ['The TMS9900 is a 16-bit', 'big-endian CPU.'], 12)
    return s.render()


# ================================================================ Amiga
@diagram('amiga-chipsets.svg')
def amiga_chipsets():
    s = Svg(900, 340, 'The Amiga had three chip sets. OCS came on the A1000, A500 and A2000 from 1985, ECS on the '
                      'A500+, A600 and A3000 from 1990, and AGA on the A1200, A4000 and CD32 from 1992.')
    s.heading('Amiga chip sets', 'Each chip set has its own models, chip RAM and Kickstart. Record the model and Kickstart with every capture.')
    x0, x1, y0 = 40, 860, 230
    def xo(year):
        return x0 + (year - 1985) / 12 * (x1 - x0)
    sets = [('OCS', 1985, 1992, TEAL, 'A1000, A500, A2000', '68000', '256 KB to 1 MB', 'Kickstart 1.0-1.3'),
            ('ECS', 1990, 1992, BLUE, 'A500+, A600, A3000', '68000; 68030 (A3000)', '1 MB', 'Kickstart 2.04-2.05'),
            ('AGA', 1992, 1996, AMBER, 'A1200, A4000, CD32', '68EC020; 68EC030 or 68040', '2 MB', 'Kickstart 3.0-3.1')]
    for k, (name, a, b, colour, models, cpu, ram, ks) in enumerate(sets):
        x = 24 + k * 286
        s.rect(x, 78, 272, 112, colour, rx=8, extra=' fill-opacity="0.15"')
        s.text(x + 14, 104, name, 16, colour, weight='700')
        s.text(x + 70, 104, models, 12, TEXT, weight='600')
        lines(s, x + 14, 128, [cpu, f'chip RAM: {ram}', ks], 12, TEXT, 18)
        s.rect(xo(a), y0 + k * 18, max(xo(b + 1) - xo(a), 8), 12, colour, rx=3, extra=' fill-opacity="0.7"')
    for yr in range(1985, 1998, 2):
        s.text(xo(yr), y0 + 70, str(yr), 10, DIM, 'middle', mono=True)
    s.text(24, 326, 'Floppies hold 880 KB on every model. A game built for 512 KB of chip RAM can behave differently on later models.', 11, DIM)
    return s.render()


@diagram('amiga-chips.svg')
def amiga_chips():
    s = Svg(900, 400, 'Agnus shares chip RAM between the 68000 and the custom chips and holds the Copper and the '
                      'Blitter. Denise draws bitplanes and sprites. Paula plays four audio channels and drives the '
                      'floppy and the interrupts.')
    s.heading('The Amiga OCS custom chips', 'Agnus shares chip RAM between the CPU and the chips. A takeover game writes their registers directly.')
    box(s, 40, 96, 150, 60, BLUE, '68000', 'CPU', op=0.25)
    box(s, 40, 206, 150, 50, GREY, 'fast RAM', 'CPU only, if fitted', op=0.25)
    box(s, 300, 96, 220, 130, TEAL, '', op=0.15)
    s.text(410, 118, 'Agnus', 14, TEXT, 'middle', '600')
    s.text(410, 134, 'chip RAM DMA, 512 KB (later 1-2 MB)', 10, DIM, 'middle')
    box(s, 314, 148, 92, 62, TEAL, 'Copper', 'beam program', op=0.35, sub_size=10)
    box(s, 414, 148, 92, 62, TEAL, 'Blitter', 'block copies', op=0.35, sub_size=10)
    box(s, 300, 290, 220, 56, GREEN, 'chip RAM', 'shared by CPU and chips', op=0.25)
    box(s, 640, 80, 220, 70, PURPLE, 'Denise', '1-5 bitplanes, 8 sprites, HAM, EHB', op=0.25, sub_size=10)
    box(s, 640, 176, 220, 70, AMBER, 'Paula', '4 audio channels, floppy, serial, IRQ', op=0.25, sub_size=10)
    arrow(s, 194, 126, 296, 126)
    arrow(s, 410, 230, 410, 286)
    arrow(s, 524, 120, 636, 115)
    arrow(s, 524, 190, 636, 211)
    s.line(115, 160, 115, 202, DIM, 1.5)
    # DMA slots flowing between chip RAM and the chips.
    for k, (path, colour) in enumerate((('M410 286L410 230', GREEN), ('M524 120L636 115', PURPLE), ('M524 190L636 211', AMBER))):
        s.add(f'<circle r="5" fill="{colour}"><animateMotion path="{path}" dur="1.2s" begin="{k * 0.4 - 1.2:.1f}s" '
              f'repeatCount="indefinite"/></circle>')
    lines(s, 640, 280, ['Time is counted in colour clocks', 'of 280 ns. Audio DMA peaks near', '28,867 samples/s (NTSC) and',
                        '28,837 (PAL) per channel.'], 11)
    s.text(40, 380, 'Trace an OS game at its library calls. Trace a takeover game at its chip register writes.', 12, DIM)
    return s.render()


@diagram('amiga-copper.svg')
def amiga_copper():
    s = Svg(900, 440, 'The Amiga Copper runs a list of WAIT and MOVE instructions in step with the beam. Each WAIT '
                      'holds until a screen line, and each MOVE writes a chip register such as the background colour, '
                      'so the colour changes part way down the screen.')
    s.heading('The Copper runs a program for the beam', 'MOVE writes a chip register. WAIT holds until the beam reaches a position. The list restarts every frame.')
    bars = [(0.0, '#0b1d3a', 'MOVE #$0013, COLOR00'), (0.22, '#7a2a2a', 'WAIT line $50'), (0.22, '#b5402a', 'MOVE #$0B42, COLOR00'),
            (0.44, '#d79a2a', 'WAIT line $78'), (0.44, '#e8c547', 'MOVE #$0EC4, COLOR00'), (0.66, '#2a7a5a', 'WAIT line $A0'),
            (0.66, '#2ab08a', 'MOVE #$02B8, COLOR00'), (0.99, None, 'WAIT $FFFE: end of list')]
    y = 92
    dur = 4.0
    for k, (frac, colour, ins) in enumerate(bars):
        is_wait = ins.startswith('WAIT')
        s.rect(24, y, 300, 30, AMBER if is_wait else TEAL, rx=3, extra=' fill-opacity="0.18"')
        s.text(36, y + 20, ins, 12, TEXT, mono=True)
        s.add(f'<rect x="24" y="{y}" width="300" height="30" rx="3" fill="none" stroke="{TEXT}" stroke-width="2" opacity="0">'
              f'<animate attributeName="opacity" values="0;1;0" keyTimes="0;{max(frac, 0.001):.3f};{min(frac + 0.06, 1):.3f}" '
              f'calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/></rect>')
        y += 36
    sx, sy, sw, sh = 420, 92, 440, 284
    cols = ['#001133', '#bb4422', '#eecc44', '#22bb88']
    for k, c in enumerate(cols):
        s.rect(sx, sy + k * sh / 4, sw, sh / 4, c, rx=0, extra=' fill-opacity="0.85"')
    s.add(f'<rect x="{sx}" y="{sy}" width="{sw}" height="2" fill="#ffffff"><animate attributeName="y" values="{sy};{sy + sh - 2}" '
          f'dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.text(sx, sy + sh + 18, 'COLOR00 is the background colour register at $DFF180.', 11, DIM)
    s.text(24, 424, 'Raster effects come from a Copper list per frame. Reproduce the list, the waits and the writes.', 12, DIM)
    return s.render()


@diagram('amiga-blitter.svg')
def amiga_blitter():
    s = Svg(900, 380, 'The Amiga Blitter reads up to three sources, A, B and C, combines them bit by bit with a logic '
                      'function and writes the destination D. A cookie-cut draws an object through its mask onto the '
                      'background.')
    s.heading('The Blitter combines three sources', 'Each output bit is a logic function (the minterm) of the A, B and C bits. Blocks up to 1,024 x 1,024 pixels.')
    srcs = [('A', 'mask', BLUE), ('B', 'object', AMBER), ('C', 'background', TEAL)]
    for k, (name, what, colour) in enumerate(srcs):
        box(s, 40, 90 + k * 76, 150, 60, colour, f'source {name}', what, op=0.25)
        arrow(s, 194, 120 + k * 76, 316, 186)
    box(s, 320, 150, 170, 72, PURPLE, 'minterm $CA', 'D = (A and B) or (not A and C)', op=0.3, sub_size=10)
    arrow(s, 494, 186, 576, 186)
    box(s, 580, 150, 150, 72, GREEN, 'destination D', 'the screen', op=0.25)
    # The cookie-cut drawn row by row.
    gx, gy, c = 760, 96, 12
    mask = ['..####..', '.######.', '########', '########', '.######.', '..####..', '...##...', '...##...']
    dur = 4.0
    for r, row in enumerate(mask):
        for k, ch in enumerate(row):
            s.rect(gx + k * c, gy + r * c, c - 1, c - 1, '#2a5a7a', rx=0)
    for r, row in enumerate(mask):
        for k, ch in enumerate(row):
            if ch == '#':
                s.add(f'<rect x="{gx + k * c}" y="{gy + r * c}" width="{c - 1}" height="{c - 1}" fill="{AMBER}" opacity="0">'
                      f'<animate attributeName="opacity" values="0;1;1" keyTimes="0;{(r + 1) / 10:.2f};1" calcMode="discrete" '
                      f'dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.text(gx + 48, gy + 116, 'row by row', 10, DIM, 'middle')
    lines(s, 40, 356, ['Where the mask bit is 1, the object bit is written. Elsewhere the background stays. The CPU runs while the Blitter works.'], 12)
    return s.render()


@diagram('amiga-disks.svg')
def amiga_disks():
    s = Svg(900, 330, 'An ADF image holds 80 cylinders of 2 sides of 11 sectors of 512 bytes, 880 KB in all, and '
                      'cannot hold most copy protection. An IPF image keeps the low-level track data, often from '
                      'KryoFlux captures.')
    s.heading('Amiga ADF and IPF disk images', 'The Amiga writes a whole track at once, so a track can hold any format a publisher chose.')
    s.text(24, 96, 'ADF: the sectors only', 13, TEXT, weight='600')
    for k in range(11):
        s.rect(24 + k * 36, 108, 34, 34, GREY, rx=3, extra=' fill-opacity="0.35"')
        s.text(41 + k * 36, 130, str(k), 11, TEXT, 'middle', mono=True)
    s.text(24, 162, '80 cylinders x 2 sides x 11 sectors x 512 bytes = 901,120 bytes (880 KB)', 12, TEXT, mono=False)
    s.text(470, 96, 'IPF: the whole track as written', 13, TEXT, weight='600')
    segs = [('gap', 30, GREY), ('sync', 22, RED), ('header', 36, PURPLE), ('data', 70, TEAL), ('sync', 22, RED), ('header', 36, PURPLE),
            ('data', 70, TEAL), ('long track', 90, AMBER)]
    x = 470
    for label, w, colour in segs:
        s.rect(x, 108, w - 2, 34, colour, rx=3, extra=' fill-opacity="0.4"')
        s.text(x + w / 2 - 1, 129, label, 9, TEXT, 'middle')
        x += w
    s.text(470, 162, 'IPF keeps sync marks, gaps, timing and odd track lengths.', 12, TEXT)
    lines(s, 24, 210, ['ADF cannot hold most copy protection. A protected game written to ADF has usually been cracked or will not load.',
                       'IPF comes from the Software Preservation Society. A file begins with "CAPS". An open-source library reads it.',
                       'Keep the IPF image as the record of what shipped.'], 12)
    s.text(24, 300, 'The "long track" here stands for any non-standard track. Each protection has its own layout.', 11, DIM)
    return s.render()


@diagram('amiga-copylock.svg')
def amiga_copylock():
    s = Svg(900, 360, 'In Copylock\'s trace vector decoder, the trace exception runs after every instruction. It '
                      'decrypts the next instruction and encrypts the last one again, so only one instruction is '
                      'readable at a time.')
    s.heading('Copylock\'s trace vector decoder', 'After each instruction the trace exception decrypts the next one and encrypts the last one again.')
    n = 8
    dur = 8.0
    s.text(40, 92, 'code in memory', 12, TEXT, weight='600')
    for k in range(n):
        y = 102 + k * 30
        s.rect(40, y, 260, 26, RED, rx=3, extra=' fill-opacity="0.3"')
        s.text(52, y + 18, '?? ?? ?? ??  encrypted', 11, DIM, mono=True)
        body = (f'<rect x="40" y="{y}" width="260" height="26" rx="3" fill="#1f8a78"/>'
                f'<text x="52" y="{y + 18}" font-family="ui-monospace, Menlo, monospace" font-size="11" fill="{TEXT}">'
                f'instruction {k + 1}  plain</text>')
        s.add(fade(body, [(k / n, (k + 1) / n)], dur))
    box(s, 400, 120, 210, 60, AMBER, 'trace exception', 'runs after every instruction', op=0.25)
    lines(s, 400, 214, ['1. encrypt the instruction that just ran', '2. decrypt the next instruction', '3. return to it'], 12, TEXT)
    lines(s, 640, 132, ['A static disassembly', 'sees only encrypted', 'bytes.'], 12)
    lines(s, 400, 300, ['Read a self-decrypting game from emulator memory after it has decrypted itself.',
                        'CAPS counted over 500 Amiga titles using Copylock.'], 12)
    return s.render()


# ================================================================ Atari ST
@diagram('atari-st-hardware.svg')
def atari_st_hardware():
    s = Svg(900, 420, 'The Atari ST has a 68000 at 8 MHz, the Shifter for video, the YM2149 for sound, the MC68901 MFP '
                      'for timers and interrupts and the WD-1772 floppy controller. The Shifter has three screen '
                      'modes.')
    s.heading('Atari ST chips and screen modes', 'The ST has a 68000 at 8 MHz and a few standard chips. The Shifter turns RAM into the picture.')
    chips = [('68000', '8 MHz', BLUE), ('Shifter', 'video', PURPLE), ('YM2149', 'sound', AMBER), ('MC68901 MFP', 'timers, interrupts', RED),
             ('WD-1772', 'floppy controller', TEAL)]
    for k, (name, sub, colour) in enumerate(chips):
        box(s, 24 + k * 172, 84, 160, 54, colour, name, sub, op=0.25)
    modes = [('low', 320, 200, '16 colours from 512', '50 or 60 Hz'), ('medium', 640, 200, '4 colours from 512', '50 or 60 Hz'),
             ('high', 640, 400, 'monochrome', '71.2 Hz')]
    x = 24
    for name, w, h, cols, hz in modes:
        bw, bh = w / 2.2, h / 2.2
        s.rect(x, 170, bw, bh, PURPLE, PURPLE, rx=2, extra=' fill-opacity="0.15" stroke-opacity="0.6"')
        s.text(x + 8, 188, f'{name}: {w} x {h}', 12, TEXT, weight='600')
        s.text(x + 8, 206, cols, 11, DIM)
        s.text(x + 8, 222, hz, 11, DIM)
        x += bw + 30
    s.text(24, 376, 'TOS with GEM. A game either calls GEMDOS, BIOS, XBIOS, AES and VDI, or takes over the hardware.', 12, TEXT)
    s.text(24, 400, 'Trace a TOS game at its system calls with Hatari. Trace a takeover game at its hardware writes.', 12, DIM)
    return s.render()


@diagram('atari-st-disks.svg')
def atari_st_disks():
    s = Svg(900, 300, 'An ST disk image holds the raw sector data. An MSA image adds a geometry header and RLE '
                      'compression. An STX (Pasti) image keeps non-standard sectors, fake track addresses and sector '
                      'timing.')
    s.heading('Atari ST disk images', 'Only STX keeps copy protection. A 720 KB disk is 80 tracks x 2 sides x 9 sectors x 512 bytes.')
    fmts = [('ST', 'raw sectors', GREY, ['no metadata', 'no protection']),
            ('MSA', 'header + RLE', BLUE, ['geometry header', 'run-length compression', 'no protection']),
            ('STX (Pasti)', 'track detail', GREEN, ['non-standard sectors', 'fake track addresses', 'sector timing'])]
    for k, (name, sub, colour, rows) in enumerate(fmts):
        x = 24 + k * 286
        s.rect(x, 80, 272, 150, colour, rx=8, extra=' fill-opacity="0.15"')
        s.text(x + 14, 106, name, 15, TEXT, weight='600')
        s.text(x + 14, 124, sub, 12, DIM)
        lines(s, x + 14, 154, rows, 12, TEXT, 20)
    s.text(24, 262, 'Its author never documented STX. Hatari 1.8.0 (2014) added the first open-source decoder.', 12, DIM)
    s.text(24, 282, 'Keep the STX image as the record of a protected disk.', 12, TEXT)
    return s.render()


@diagram('atari-ste-line.svg')
def atari_ste_line():
    s = Svg(900, 300, 'The STE from 1989 adds a Blitter, fine scrolling, DMA sound and 4096 colours. The Mega STE runs '
                      'at 16 MHz. The TT030 and Falcon030 use a 68030, and the Falcon adds a 56001 DSP.')
    s.heading('STE, Mega STE, TT030 and Falcon030', 'Each model adds hardware the ST lacks. Record the model with each capture.')
    models = [('STE', 'late 1989', '68000 at 8 MHz', ['Blitter, fine scrolling', 'split screen', '8-bit stereo DMA sound', '4096 colours'], TEAL),
              ('Mega STE', '', '68000 at 16 MHz', ['as the STE'], BLUE),
              ('TT030', '', '68030', ['32-bit CPU'], PURPLE),
              ('Falcon030', '1992', '68030', ['Motorola 56001 DSP', 'export the DSP code apart', 'from the 68030 code'], AMBER)]
    for k, (name, year, cpu, rows, colour) in enumerate(models):
        x = 24 + k * 216
        s.rect(x, 80, 202, 190, colour, rx=8, extra=' fill-opacity="0.15"')
        s.text(x + 14, 106, name, 15, TEXT, weight='600')
        if year:
            s.text(x + 188, 106, year, 11, DIM, 'end')
        s.text(x + 14, 128, cpu, 12, colour, weight='600')
        lines(s, x + 14, 156, rows, 12, TEXT, 20)
    s.text(24, 290, 'RAM to 4 MB on the STE. TOS 1.06 to 2.00.', 11, DIM)
    return s.render()


# ================================================================ Acorn 32-bit
@diagram('arm26-r15.svg')
def arm26_r15():
    s = Svg(900, 330, 'On the ARM2 and ARM3, R15 holds the flags, the interrupt masks, a 24-bit word address and the '
                      'processor mode. An instruction that reads or writes R15 therefore touches the flags as well as '
                      'the PC.')
    s.heading('26-bit ARM keeps the flags in R15', 'ARM2 and ARM3 pack the status into the program counter. Ghidra\'s ARM starts at v4, which has none of this.')
    fields = [('N', 1, RED), ('Z', 1, RED), ('C', 1, RED), ('V', 1, RED), ('I', 1, AMBER), ('F', 1, AMBER),
              ('PC: word address, 24 bits (64 MB)', 24, TEAL), ('M1', 1, PURPLE), ('M0', 1, PURPLE)]
    x, unit = 24, 26.5
    for label, n, colour in fields:
        w = n * unit
        s.rect(x, 100, w - 2, 36, colour, rx=3, extra=' fill-opacity="0.32"')
        s.text(x + w / 2 - 1, 123, label, 11, TEXT, 'middle', '600')
        x += w
    s.text(24, 152, 'bit 31', 10, DIM, mono=True)
    s.text(24 + 32 * unit, 152, 'bit 0', 10, DIM, 'end', mono=True)
    lines(s, 24, 186, ['NZCV: the condition flags. I and F: the IRQ and FIQ masks.', 'M1 M0: the mode (user, FIQ, IRQ, supervisor).',
                       'MOVS PC, R14 returns and restores the flags in one instruction.'], 12, TEXT)
    lines(s, 520, 186, ['The Risc PC\'s CPUs add a 32-bit mode,', 'with the flags in a separate CPSR.',
                        'Record which mode the code runs in.'], 12)
    s.text(24, 300, 'Take ground truth from a 26-bit-aware emulator. Unicorn\'s ARM models may give wrong results for this code.', 12, DIM)
    return s.render()


@diagram('archimedes-chips.svg')
def archimedes_chips():
    s = Svg(900, 320, 'The Archimedes uses the ARM CPU with MEMC for memory, VIDC for video and sound and IOC for '
                      'input and output. The ARM250 merges all four on one chip.')
    s.heading('The Archimedes chips', 'The Archimedes uses four main chips. The ARM250 in the A3010 range (1992) merges them into one.')
    box(s, 40, 100, 150, 60, BLUE, 'ARM2 / ARM3', '8 MHz / 25-33 MHz', op=0.25)
    box(s, 300, 100, 150, 60, TEAL, 'MEMC', 'memory, DMA', op=0.25)
    box(s, 560, 80, 150, 54, PURPLE, 'VIDC', 'video; 8 sound channels', op=0.25, sub_size=10)
    box(s, 560, 150, 150, 54, AMBER, 'IOC', 'I/O, timers, IRQs', op=0.25)
    box(s, 300, 210, 150, 46, GREEN, 'RAM', '', op=0.25)
    arrow(s, 194, 130, 296, 130)
    arrow(s, 454, 120, 556, 107)
    arrow(s, 454, 140, 556, 177)
    arrow(s, 375, 164, 375, 206)
    s.rect(24, 78, 708, 192, 'none', AMBER, rx=10, extra=' stroke-dasharray="6 5" stroke-opacity="0.5"')
    s.text(726, 264, 'ARM250: all on one chip', 11, AMBER, 'end')
    lines(s, 740, 104, ['VIDC1 sound:', '8-bit logarithmic', 'stereo.'], 11)
    s.text(40, 300, 'Games ran under Arthur and RISC OS, which use SWI calls. Disks are 800 KB ADFS.', 12, DIM)
    return s.render()


# ================================================================ Japanese 16 and 32-bit
@diagram('x68000.svg')
def x68000():
    s = Svg(900, 360, 'The Sharp X68000 has 1056 KB of VRAM split into graphics, text and sprite areas, and 128 '
                      'sprites with 32 per line that raster interrupts can raise. Its floppies hold 77 tracks of 8 '
                      'sectors of 1024 bytes.')
    s.heading('Sharp X68000', 'The X68000 is a 68000 machine with large VRAM, many sprites and FM sound. It was sold in Japan from 1987 to 1993.')
    s.text(24, 96, 'VRAM: 1056 KB', 12, TEXT, weight='600')
    x = 24
    for label, kb, colour in (('graphics 512 KB', 512, BLUE), ('text 512 KB', 512, TEAL), ('', 32, AMBER)):
        w = kb / 1056 * 840
        s.rect(x, 106, w - 2, 34, colour, rx=3, extra=' fill-opacity="0.35"')
        s.text(x + w / 2, 128, label, 11 if kb > 100 else 9, TEXT, 'middle', '600')
        x += w
    s.text(24, 160, 'The thin amber bar at the right is the 32 KB sprite area.', 11, DIM)
    s.text(24, 196, 'sprites: 128, with 32 on one line', 12, TEXT, weight='600')
    for k in range(32):
        s.rect(24 + k * 13, 206, 11, 16, AMBER, rx=2, extra=' fill-opacity="0.7"')
    s.text(450, 219, 'raster interrupts reuse sprites lower down, up to 512', 11, DIM)
    lines(s, 24, 262, ['CPU: HD68HC000 at 10 MHz; 16 MHz on the XVI; 68EC030 at 25 MHz on the X68030.',
                       'Sound: YM2151 (eight FM channels) and an MSM6258 ADPCM channel at 15.6 kHz.',
                       'Floppy: 77 tracks x 8 sectors x 1024 bytes x 2 sides = 1232 KiB, at 360 rpm. The OS is Human68k.'], 12, TEXT)
    s.text(24, 340, 'Record whether the game uses raster interrupts to show more than 128 sprites.', 12, DIM)
    return s.render()


@diagram('pc98.svg')
def pc98():
    s = Svg(900, 330, 'The NEC PC-9801 runs x86 code under MS-DOS. Its video uses two uPD7220 controllers for text and '
                      'graphics, with the GRCG and EGC blitters. Its FM sound comes from the 26 and 86 cards.')
    s.heading('The NEC PC-9801 video and sound', 'The PC-9801 runs MS-DOS on x86. Its video and sound chips differ from the IBM PC\'s, so VGA notes do not apply.')
    box(s, 24, 90, 170, 60, BLUE, '8086 to 80386', 'MS-DOS, N88-BASIC(86)', op=0.25)
    box(s, 260, 80, 190, 44, TEAL, 'uPD7220 (text)', op=0.25)
    box(s, 260, 132, 190, 44, PURPLE, 'uPD7220 (graphics)', op=0.25)
    box(s, 500, 80, 170, 44, AMBER, 'GRCG (1985)', op=0.25)
    box(s, 500, 132, 170, 44, AMBER, 'EGC (1986)', op=0.25)
    arrow(s, 198, 120, 256, 102)
    arrow(s, 198, 120, 256, 154)
    arrow(s, 454, 154, 496, 154)
    arrow(s, 454, 154, 496, 102)
    lines(s, 700, 100, ['640 x 400 in 8', 'colours, later 16', 'from 4,096.'], 12, TEXT)
    s.text(24, 214, 'FM sound cards', 12, TEXT, weight='600')
    box(s, 24, 224, 260, 50, RED, 'PC-9801-26', 'YM2203', op=0.22)
    box(s, 300, 224, 260, 50, RED, 'PC-9801-86', 'YM2608', op=0.22)
    s.text(24, 306, 'Record which FM card the reference recording used. 18.3 million PC-98s were sold, 1982 to 2003.', 12, DIM)
    return s.render()


@diagram('fm-towns.svg')
def fm_towns():
    s = Svg(900, 300, 'The Fujitsu FM Towns has an 80386DX at 16 MHz running Towns OS, which is MS-DOS with the Phar '
                      'Lap DOS extender. It has a CD-ROM drive as standard, 1024 sprites and RF5c68 PCM and YM2612 FM '
                      'sound.')
    s.heading('Fujitsu FM Towns', 'The FM Towns is a 386 machine with a CD-ROM drive as standard, sold from 1989 to 1997. The FM Towns Marty console (1993) runs its games.')
    layers = [('game', TEAL), ('Phar Lap DOS extender', AMBER), ('MS-DOS (Towns OS)', BLUE), ('80386DX at 16 MHz', GREY)]
    for k, (name, colour) in enumerate(layers):
        box(s, 24, 84 + k * 46, 300, 40, colour, name, op=0.28)
    for k, (name, sub, colour) in enumerate((('CD-ROM', 'standard', GREEN), ('sprites', '1024', PURPLE),
                                            ('RF5c68', 'PCM', RED), ('YM2612', 'FM', RED))):
        box(s, 380 + (k % 2) * 250, 84 + (k // 2) * 70, 230, 56, colour, name, sub, op=0.22)
    s.text(24, 286, 'Identify the extender and its executable format before loading the program into Ghidra.', 12, DIM)
    return s.render()


# ================================================================ CPU families
@diagram('cpu-byte-order.svg')
def cpu_byte_order():
    s = Svg(900, 340, 'A little-endian CPU stores the value 0x12345678 as 78 56 34 12, and a big-endian CPU stores it '
                      'as 12 34 56 78. The 8-bit families are little-endian except the 6809. The 16-bit console CPUs '
                      'are big-endian except x86.')
    s.heading('Byte order', 'Each kind of CPU stores the value 0x12345678 at address 0x100 in its own byte order.')
    for r, (name, order, colour) in enumerate((('little-endian', ['78', '56', '34', '12'], TEAL),
                                               ('big-endian', ['12', '34', '56', '78'], AMBER))):
        y = 96 + r * 76
        s.text(24, y + 26, name, 13, colour, weight='600')
        for k, b in enumerate(order):
            s.text(180 + k * 70 + 31, y - 6, f'0x{0x100 + k:X}', 10, DIM, 'middle', mono=True)
            s.rect(180 + k * 70, y, 62, 40, colour, rx=3, extra=' fill-opacity="0.3"')
            s.text(180 + k * 70 + 31, y + 26, b, 15, TEXT, 'middle', '600', mono=True)
    lines(s, 490, 112, ['Z80, 8080, SM83, 6502 line, x86, ARM', '(usually), the PlayStation\'s MIPS'], 12, TEXT)
    lines(s, 490, 188, ['6809, 68000, TMS9900, CP1600,', 'PowerPC, SH-2, the N64\'s MIPS'], 12, TEXT)
    lines(s, 24, 270, ['A data-file decoder must follow the byte order of the machine that wrote the file.',
                       'A file copied between a 68000 machine and an ARM or x86 machine may have been converted.'], 12)
    return s.render()


@diagram('cpu-families-map.svg')
def cpu_families_map():
    s = Svg(900, 470, 'Eleven CPU families ran almost every game platform. They are the Z80 line, the 6502 line, the '
                      '6809, the 68000, the TMS9900, the CP1600, x86, ARM, MIPS, SuperH and PowerPC.')
    s.heading('Eleven CPU families', 'The family tells you the byte order, the disassembler, the emulator cores and the arithmetic to copy.')
    fams = [('8080, Z80, SM83', '1976', 'Spectrum, CPC, MSX, PC-8801, X1, Game Boy, Master System', TEAL),
            ('6502 line', '1975', 'BBC, C64, VIC-20, Atari 8-bit, Apple II, Oric, NES, SNES (65C816)', TEAL),
            ('6809', '1978', 'Dragon, CoCo, Vectrex, Williams arcade boards', TEAL),
            ('68000', '1979', 'Amiga, ST, X68000, 68k Mac, Mega Drive, Neo Geo, CPS-2', BLUE),
            ('TMS9900', '1976', 'TI-99/4A registers in RAM', BLUE),
            ('CP1600', '1975', 'Intellivision: 10-bit opcodes', BLUE),
            ('x86', '', 'PC, PC-98, FM Towns, WonderSwan, Xbox, PS4 onward', AMBER),
            ('ARM', '', 'Archimedes, 3DO, GBA, DS, 3DS, Vita, Switch, Apple silicon', PURPLE),
            ('MIPS', '', 'PlayStation, N64, PS2, PSP', PURPLE),
            ('SuperH', '1992', 'Saturn, 32X, CPS-3, Dreamcast', PURPLE),
            ('PowerPC', '', 'PowerPC Mac, GameCube, Wii, Wii U, Xbox 360, PS3', PURPLE)]
    for k, (name, year, where, colour) in enumerate(fams):
        y = 80 + k * 31
        s.rect(24, y, 160, 26, colour, rx=3, extra=' fill-opacity="0.3"')
        s.text(34, y + 18, name, 12, TEXT, weight='600')
        if year:
            s.text(176, y + 18, year, 10, DIM, 'end', mono=True)
        s.text(198, y + 18, where, 12, TEXT)
    s.rect(24, 458 - 14, 12, 10, TEAL, rx=2, extra=' fill-opacity="0.5"')
    s.text(42, 453, '8-bit', 11, DIM)
    s.rect(100, 458 - 14, 12, 10, BLUE, rx=2, extra=' fill-opacity="0.5"')
    s.text(118, 453, '16 and 32-bit', 11, DIM)
    s.rect(220, 458 - 14, 12, 10, AMBER, rx=2, extra=' fill-opacity="0.5"')
    s.text(238, 453, 'x86', 11, DIM)
    s.rect(290, 458 - 14, 12, 10, PURPLE, rx=2, extra=' fill-opacity="0.5"')
    s.text(308, 453, 'RISC', 11, DIM)
    s.text(860, 453, 'Co-processors such as the Cell SPU and the Jaguar\'s RISC cores add to these.', 11, DIM, 'end')
    return s.render()


@diagram('cpu-tool-support.svg')
def cpu_tool_support():
    s = Svg(900, 520, 'The table shows whether Ghidra has each CPU family built in or by extension, and whether '
                      'Unicorn 2 and Capstone support it. The 8-bit families have no Unicorn support.')
    s.heading('Tool support per CPU family', 'The table covers Ghidra 12.1.4, Unicorn 2.1.4 and Capstone next (6.0.0-Alpha11).')
    Y, P, E, N = 'yes', 'part', 'ext', 'no'
    rows = [('6502, 65C02', Y, N, Y), ('65C816', E, N, Y), ('HuC6280', E, N, P), ('Z80, 8080', Y, N, N), ('SM83', E, N, N),
            ('6809, 6309', Y, N, Y), ('68000 family', Y, Y, Y), ('TMS9900', E, N, N), ('CP1600', Y, N, N), ('x86', Y, Y, Y),
            ('ARM', Y, P, Y), ('MIPS', P, P, Y), ('SuperH', Y, N, Y), ('PowerPC', P, Y, Y), ('Cell SPU', E, N, N),
            ('Jaguar RISC', N, N, N)]
    colour = {Y: GREEN, P: AMBER, E: BLUE, N: GREY}
    label = {Y: 'built in', P: 'partly', E: 'extension', N: 'none'}
    cx = [220, 400, 580]
    for k, head in enumerate(('Ghidra', 'Unicorn 2', 'Capstone')):
        s.text(cx[k] + 75, 90, head, 12, TEXT, 'middle', '600')
    for r, (name, *cells) in enumerate(rows):
        y = 100 + r * 24
        s.text(24, y + 16, name, 12, TEXT)
        for k, v in enumerate(cells):
            lab = label[v] if k == 0 else {Y: 'yes', P: 'partly', N: 'none', E: 'extension'}[v]
            s.rect(cx[k], y, 150, 20, colour[v], rx=3, extra=f' fill-opacity="{0.15 if v == N else 0.4}"')
            s.text(cx[k] + 75, y + 14, lab, 11, TEXT if v != N else DIM, 'middle')
    lines(s, 750, 106, ['Partly:', 'HuC6280 in 6502', 'modes only;', 'ARM: no ARM7TDMI', 'by name; MIPS:', 'no R3000, R5900;',
                        'PowerPC: no paired', 'singles in Ghidra.'], 11)
    s.text(24, 504, 'Where Unicorn has nothing, take ground truth from a small core such as py65, iz80, Musashi, Moira or MAME\'s cores.', 12, DIM)
    return s.render()


@diagram('cpu-6502-adc.svg')
def cpu_6502_adc():
    s = Svg(900, 380, 'Ghidra\'s 6502 ADC has no decimal mode, leaves the incoming carry out of the carry test and '
                      'copies V from C. Three worked cases show a real 6502 and Ghidra\'s p-code giving different '
                      'results.')
    s.heading('Ghidra\'s 6502 ADC is not exact', 'The p-code has three faults. Each case below gives a different answer from a real 6502.')
    cases = [('decimal mode', 'D = 1, A = $99, add $01, C = 0', 'A = $00, C = 1', 'A = $9A, C = 0', ['no decimal mode path']),
             ('carry in', 'D = 0, A = $FF, add $00, C = 1', 'A = $00, C = 1', 'A = $00, C = 0', ['carry in left out of', 'the carry test']),
             ('overflow', 'D = 0, A = $50, add $50, C = 0', 'A = $A0, V = 1', 'A = $A0, V = 0', ['V copied from C instead', 'of signed overflow'])]
    s.text(330, 92, 'real 6502', 12, GREEN, weight='600')
    s.text(520, 92, 'Ghidra p-code', 12, RED, weight='600')
    s.text(700, 92, 'fault', 12, DIM, weight='600')
    for k, (name, inp, real, ghidra, why) in enumerate(cases):
        y = 104 + k * 70
        s.rect(24, y, 840, 60, GREY, rx=6, extra=' fill-opacity="0.12"')
        s.text(36, y + 24, name, 13, TEXT, weight='600')
        s.text(36, y + 44, inp, 11, DIM, mono=True)
        s.rect(320, y + 12, 170, 36, GREEN, rx=4, extra=' fill-opacity="0.25"')
        s.text(405, y + 35, real, 12, TEXT, 'middle', mono=True)
        s.rect(510, y + 12, 170, 36, RED, rx=4, extra=' fill-opacity="0.25"')
        s.text(595, y + 35, ghidra, 12, TEXT, 'middle', mono=True)
        lines(s, 700, y + 27, why, 11, DIM, 15)
    lines(s, 24, 336, ['Do not use Ghidra\'s p-code emulation as ground truth for 6502 games. Use py65, mos6502, MAME or floooh/chips.',
                       'The NES 2A03 has no decimal mode, so the first case gives $9A on a NES.'], 12)
    return s.render()


@diagram('cpu-65816-widths.svg')
def cpu_65816_widths():
    s = Svg(900, 320, 'On the 65C816 the same bytes decode differently by the m flag. A9 34 12 EA is LDA #$34 then ORA '
                      '($EA) with an 8-bit accumulator, or LDA #$1234 then NOP with a 16-bit accumulator.')
    s.heading('65C816 operand widths change the decoding', 'The m and x flags set the operand widths. A disassembler must track REP and SEP.')
    bytes_ = ['A9', '34', '12', 'EA']
    for k, b in enumerate(bytes_):
        s.rect(280 + k * 70, 84, 62, 36, GREY, rx=3, extra=' fill-opacity="0.3"')
        s.text(311 + k * 70, 108, b, 15, TEXT, 'middle', '600', mono=True)
    rows = [('m = 1 (8-bit A, after SEP #$20)', [(0, 2, 'LDA #$34', TEAL), (2, 2, 'ORA ($EA)', AMBER)]),
            ('m = 0 (16-bit A, after REP #$20)', [(0, 3, 'LDA #$1234', BLUE), (3, 1, 'NOP', PURPLE)])]
    for r, (head, ins) in enumerate(rows):
        y = 150 + r * 64
        s.text(24, y + 24, head, 12, TEXT, weight='600')
        for start, n, text, colour in ins:
            s.rect(280 + start * 70, y, n * 70 - 8, 36, colour, rx=3, extra=' fill-opacity="0.35"')
            s.text(280 + start * 70 + (n * 70 - 8) / 2, y + 23, text, 13, TEXT, 'middle', mono=True)
    lines(s, 24, 290, ['If the disassembler guesses a width wrong, every instruction after the first immediate operand decodes wrongly.',
                       'The SNES 5A22 and the Apple IIgs use the 65C816. Record m and x at the entry of each function.'], 12)
    return s.render()


# ================================================================ the IBM PC
@diagram('ibmpc-exe-formats.svg')
def ibmpc_exe_formats():
    s = Svg(900, 400, 'A PC file with no MZ magic is a COM file. In an MZ file, the offset at e_lfanew (0x3C) leads to '
                      'an NE, LE, LX or PE header. The PE optional header magic tells PE32 from PE32+.')
    s.heading('How to identify a PC executable', 'Check the magic first. The tools you need depend on the format more than on the CPU.')
    box(s, 24, 90, 170, 54, GREY, 'first two bytes', op=0.25)
    arrow(s, 198, 117, 240, 117)
    box(s, 244, 90, 120, 54, BLUE, '"MZ"?', op=0.3)
    arrow(s, 304, 148, 304, 196)
    s.text(312, 178, 'no', 11, DIM)
    box(s, 244, 200, 120, 54, AMBER, 'COM', 'raw, loaded at 100h', op=0.25, sub_size=10)
    arrow(s, 368, 117, 410, 117)
    s.text(378, 108, 'yes', 11, DIM)
    box(s, 414, 90, 180, 54, BLUE, 'read e_lfanew', 'the dword at 0x3C', op=0.3, mono=True)
    arrow(s, 598, 117, 640, 117)
    targets = [('"NE"', 'Win16, Windows 3.x', TEAL), ('"LE" / "LX"', 'DOS/4GW, OS/2 style', PURPLE),
               ('"PE\\0\\0"', 'Win32 and x64', GREEN), ('nothing valid', 'plain DOS MZ', GREY)]
    for k, (magic, what, colour) in enumerate(targets):
        box(s, 644, 74 + k * 62, 220, 52, colour, magic, what, op=0.25)
    s.text(24, 300, 'PE optional header magic', 12, TEXT, weight='600')
    box(s, 24, 312, 200, 48, GREEN, '0x10B: PE32', '32-bit', op=0.22)
    box(s, 240, 312, 200, 48, GREEN, '0x20B: PE32+', 'x64', op=0.22)
    lines(s, 470, 326, ['DJGPP: COFF behind a go32 stub.', 'Ghidra loads MZ, NE and PE. LE and LX need', 'a community extension (not confirmed).'], 12)
    return s.render()


@diagram('ibmpc-eras.svg')
def ibmpc_eras():
    s = Svg(900, 340, 'Each PC era has its own executable format. DOS used COM and MZ from 1981, DOS extenders LE and '
                      'COFF from about 1990, Windows 3.x NE, Windows 95 to ME PE32, and x64 PE32+ from 2003.')
    s.heading('PC eras and executable formats', 'Each era has its own format and tool chain, and its own way to run it today.')
    x0, x1 = 40, 620
    def xo(y):
        return x0 + (y - 1981) / (2026 - 1981) * (x1 - x0)
    eras = [('IBM PC, XT, AT', 'COM, MZ', 1981, 2000, GREY, 'DOSBox, DOSBox-X'),
            ('PCjr, Tandy 1000', 'COM, MZ', 1984, 1993, GREY, 'DOSBox machine=tandy'),
            ('DOS extenders', 'LE, COFF', 1990, 1999, PURPLE, 'DOSBox'),
            ('Windows 3.x', 'NE', 1990, 1995, TEAL, 'Wine with otvdm, a VM'),
            ('Windows 95 to ME', 'PE32', 1995, 2001, BLUE, 'Windows, Wine'),
            ('XP to 11, x64', 'PE32+', 2001, 2026, GREEN, 'Windows, Wine, Proton')]
    for k, (name, fmt, a, b, colour, run) in enumerate(eras):
        y = 84 + k * 36
        s.rect(xo(a), y, xo(b) - xo(a), 28, colour, rx=4, extra=' fill-opacity="0.35"')
        s.text(xo(a) + 8, y + 18, f'{name}: {fmt}', 12, TEXT, weight='600')
        s.text(640, y + 18, run, 11, DIM)
    for yr in (1981, 1990, 2000, 2010, 2020):
        s.text(xo(yr), 314, str(yr), 10, DIM, 'middle', mono=True)
    s.text(640, 70, 'running it today', 11, TEXT, weight='600')
    s.text(860, 330, 'Bar ends are approximate where the page gives no end year.', 10, DIM, 'end')
    return s.render()


@diagram('ibmpc-tandy.svg')
def ibmpc_tandy():
    s = Svg(900, 280, 'The PCjr and Tandy 1000 add 16-colour modes at 160 by 200 and 320 by 200 to CGA, and an SN76496 '
                      'sound chip at port C0h. DOSBox runs them with machine set to tandy or pcjr.')
    s.heading('PCjr and Tandy 1000 paths', 'A DOS game may have a Tandy path beside its CGA path.')
    modes = [('CGA 320 x 200', ['#000000', '#55ffff', '#ff55ff', '#ffffff'], '4 colours'),
             ('Tandy 160 x 200', None, '16 colours'), ('Tandy 320 x 200', None, '16 colours')]
    rgbi = ['#000000', '#0000aa', '#00aa00', '#00aaaa', '#aa0000', '#aa00aa', '#aa5500', '#aaaaaa', '#555555', '#5555ff',
            '#55ff55', '#55ffff', '#ff5555', '#ff55ff', '#ffff55', '#ffffff']
    for k, (name, cols, n) in enumerate(modes):
        x = 24 + k * 220
        s.text(x, 96, name, 13, TEXT, weight='600')
        for i, c in enumerate(cols or rgbi):
            s.rect(x + i * 12, 108, 11, 22, c, rx=1)
        s.text(x, 148, n, 11, DIM)
    box(s, 690, 84, 180, 64, AMBER, 'SN76496', 'port C0h, 3 tones + noise', op=0.25, sub_size=10)
    lines(s, 24, 192, ['The game picks a path by the machine it detects. Record which path the reference recording took.',
                       'In DOSBox set machine=tandy or machine=pcjr to run that path.'], 12, TEXT)
    s.text(24, 256, 'These hardware facts are marked not confirmed on the page.', 11, DIM)
    return s.render()


@diagram('ibmpc-compilers.svg')
def ibmpc_compilers():
    s = Svg(900, 380, 'Each PC compiler leaves fingerprints, such as Turbo Pascal\'s runtime error 200 and 6-byte '
                      'Real, Borland C++\'s c0 startup and TDInfo, Watcom\'s DOS/4GW banner, DJGPP\'s go32 stub, '
                      'MSVC\'s Rich header and Delphi\'s VCL names.')
    s.heading('Compiler fingerprints', 'Once you know the compiler, you know which library code to skip and which arithmetic to copy.')
    rows = [('Turbo Pascal', 'Runtime error 200 on fast CPUs; 6-byte Real numbers', RED),
            ('Borland C++', 'copyright string, c0.asm startup code, sometimes TDInfo debug data', AMBER),
            ('Watcom', 'DOS/4GW start-up banner; arguments in EAX, EDX, EBX, ECX', PURPLE),
            ('DJGPP', 'COFF behind a go32 stub', BLUE),
            ('MSVC', 'the undocumented "Rich" header; MSVCRT.DLL or VCRUNTIME140.DLL', TEAL),
            ('Delphi', 'VCL class names, RTTI and DFM form resources', GREEN)]
    for k, (name, what, colour) in enumerate(rows):
        y = 84 + k * 42
        s.rect(24, y, 160, 34, colour, rx=4, extra=' fill-opacity="0.3"')
        s.text(104, y + 22, name, 13, TEXT, 'middle', '600')
        s.text(200, y + 22, what, 12, TEXT)
    s.text(24, 352, 'Doom was built with Watcom. Quake was built with DJGPP. These fingerprints are marked not confirmed on the page.', 11, DIM)
    return s.render()


@diagram('ibmpc-protection.svg')
def ibmpc_protection():
    s = Svg(900, 330, 'PC copy protection went from manual checks and code wheels to Copylock, SafeDisc from 1998 to '
                      '2009, StarForce, the Steam DRM wrapper and Denuvo from 2014. The checks moved toward the disc '
                      'and then online.')
    s.heading('PC copy protection over the years', 'The checks moved from the manual, to the disk, to the disc, and later online.')
    steps = [('manual checks', 'code wheels, such as', '"Dial-a-Pirate"', AMBER),
             ('Copylock', 'Rob Northen\'s disk', 'protection', RED),
             ('SafeDisc', '1998 to March 2009', 'secdrv.sys driver', BLUE),
             ('StarForce 3.0', 'a driver that stayed', 'after uninstall', PURPLE),
             ('Steam DRM wrapper', '"easily removed by a', 'motivated attacker"', TEAL),
             ('Denuvo', 'from FIFA 15 (2014),', 'online activation', GREEN)]
    for k, (name, a1, a2, colour) in enumerate(steps):
        x = 24 + k * 143
        s.rect(x, 96, 133, 110, colour, rx=6, extra=' fill-opacity="0.22"')
        s.text(x + 10, 120, name, 12, TEXT, weight='600')
        s.text(x + 10, 144, a1, 10, DIM)
        s.text(x + 10, 160, a2, 10, DIM)
        if k:
            arrow(s, x - 10, 151, x - 1, 151)
    s.line(24, 228, 876, 228, DIM, 1.5, ' marker-end="url(#arrow)"')
    s.text(24, 246, 'earlier', 10, DIM)
    s.text(876, 246, 'later', 10, DIM, 'end')
    lines(s, 24, 280, ['Protected or virtualised code is hard to decompile. Emulate the game\'s own logic.',
                       'A later DRM-free release is often the easiest owned copy to study.'], 12)
    return s.render()


# ================================================================ the Macintosh
@diagram('mac-eras.svg')
def mac_eras():
    s = Svg(900, 360, 'The Mac used 68k CODE resources from 1984, PowerPC PEF from 1994, Intel Mach-O from 2006 and '
                      'Apple silicon Mach-O from 2020. An emulator or translator bridged each change.')
    s.heading('Mac eras and code formats', 'The Mac has used four CPU families, each with its own code format. At each change, a translator ran the old code for some years.')
    x0, x1 = 40, 860
    def xo(y):
        return x0 + (y - 1984) / (2028 - 1984) * (x1 - x0)
    eras = [('68k', 'CODE resources, A5 world', 1984, 1998, GREY, 'Mini vMac, MAME, Basilisk II'),
            ('PowerPC', 'PEF, Code Fragment Manager', 1994, 2006, BLUE, 'SheepShaver, QEMU'),
            ('Intel', 'Mach-O, universal binaries', 2006, 2020, AMBER, 'native, VMs'),
            ('Apple silicon', 'Mach-O, ARM64', 2020, 2028, TEAL, 'native')]
    for k, (name, fmt, a, b, colour, emu) in enumerate(eras):
        y = 84 + k * 42
        s.rect(xo(a), y, xo(b) - xo(a), 32, colour, rx=4, extra=' fill-opacity="0.35"')
        s.text(xo(a) + 8, y + 14, name, 12, TEXT, weight='600')
        s.text(xo(a) + 8, y + 27, fmt, 10, TEXT)
        s.text(xo(b) + 8 if b < 2026 else xo(a) - 8, y + 21, emu, 11, DIM, 'start' if b < 2026 else 'end')
    bridges = [(1994, '68k emulator in ROM'), (2006, 'Rosetta (to 2011)'), (2020, 'Rosetta 2')]
    for yr, name in bridges:
        s.line(xo(yr), 80, xo(yr), 262, AMBER, 1.5, ' stroke-dasharray="4 3"')
        s.text(xo(yr), 280, name, 11, AMBER, 'middle')
    s.line(xo(2019.8), 80, xo(2019.8), 262, RED, 1.5)
    s.text(xo(2019.8) - 4, 300, 'Catalina (2019): 64-bit apps only', 11, RED, 'end')
    for yr in (1984, 1994, 2006, 2020):
        s.text(xo(yr), 330, str(yr), 10, DIM, 'middle', mono=True)
    return s.render()


@diagram('mac-a5-world.svg')
def mac_a5_world():
    s = Svg(900, 400, 'In a 68k Mac application, register A5 points between the application globals below it and the '
                      'application parameters and jump table above it. A call through the jump table makes the Segment '
                      'Loader load a CODE segment. Toolbox calls are A-line traps.')
    s.heading('The A5 world of a 68k Mac application', 'Code lives in CODE resources. Register A5 is the base for the globals and the jump table.')
    x, y = 200, 90
    memmap(s, x, y, 240, [('', 'jump table (from CODE 0)', 80, AMBER), ('', 'application parameters', 32, PURPLE),
                          ('', 'application globals', 90, TEAL), ('', 'QuickDraw globals', 40, BLUE)])
    s.line(x - 60, y + 112, x + 250, y + 112, RED, 2)
    s.text(x - 66, y + 116, 'A5', 14, RED, 'end', '700', mono=True)
    s.text(x + 120, y + 260, 'higher addresses at the top', 10, DIM, 'middle')
    s.add(f'<text x="{x - 66}" y="{y + 60}" font-family="{SANS}" font-size="11" fill="{DIM}" text-anchor="end">positive</text>')
    s.add(f'<text x="{x - 66}" y="{y + 74}" font-family="{SANS}" font-size="11" fill="{DIM}" text-anchor="end">offsets</text>')
    s.add(f'<text x="{x - 66}" y="{y + 170}" font-family="{SANS}" font-size="11" fill="{DIM}" text-anchor="end">negative</text>')
    s.add(f'<text x="{x - 66}" y="{y + 184}" font-family="{SANS}" font-size="11" fill="{DIM}" text-anchor="end">offsets</text>')
    for k, name in enumerate(('CODE 1', 'CODE 2', 'CODE 3')):
        box(s, 560 + k * 105, 96, 95, 44, GREY, name, op=0.25)
    dur = 4.0
    s.add(f'<path d="M444 130C 500 130 520 160 570 144" fill="none" stroke="{AMBER}" stroke-width="2" marker-end="url(#arrow)"/>')
    s.add(f'<rect x="665" y="96" width="95" height="44" rx="6" fill="{GREEN}" fill-opacity="0.5" opacity="0">'
          f'<animate attributeName="opacity" values="0;1;0" keyTimes="0;0.5;1" calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/></rect>')
    lines(s, 560, 170, ['A call through the jump table loads', 'the segment if it is not in memory', '(the Segment Loader).'], 12)
    box(s, 560, 244, 300, 50, RED, 'A-line trap: opcode $Axxx', 'the trap dispatcher calls the Toolbox', op=0.2)
    lines(s, 24, 368, ['Forks are lost on other file systems, so archives use MacBinary, BinHex, AppleSingle/AppleDouble or DiskCopy.'], 12)
    return s.render()


@diagram('mac-binaries.svg')
def mac_binaries():
    s = Svg(900, 330, 'A fat Classic app keeps 68k CODE resources in the resource fork and a PowerPC PEF fragment in '
                      'the data fork. A universal Mach-O file starts with 0xcafebabe and holds one Mach-O slice per '
                      'CPU.')
    s.heading('Mac files that hold two builds', 'Compare the two compiled versions of each function to check one against the other.')
    s.rect(24, 80, 410, 200, BLUE, rx=8, extra=' fill-opacity="0.1"')
    s.text(40, 106, 'fat Classic app', 14, TEXT, weight='600')
    box(s, 40, 120, 180, 120, GREY, 'resource fork', '68k CODE resources', op=0.25, sub_size=10)
    box(s, 236, 120, 180, 120, BLUE, 'data fork', 'PowerPC PEF fragment', op=0.25, sub_size=10)
    s.text(40, 264, 'The Code Fragment Manager loads the PEF.', 11, DIM)
    s.rect(466, 80, 410, 200, AMBER, rx=8, extra=' fill-opacity="0.1"')
    s.text(482, 106, 'universal Mach-O', 14, TEXT, weight='600')
    box(s, 482, 120, 378, 34, AMBER, '0xcafebabe: the fat header', op=0.3, size=12)
    box(s, 482, 162, 184, 78, PURPLE, 'slice 1', 'Mach-O 0xfeedfacf', op=0.25, mono=True)
    box(s, 676, 162, 184, 78, TEAL, 'slice 2', 'Mach-O 0xfeedface/cf', op=0.25, mono=True)
    s.text(482, 264, '0xfeedface is 32-bit Mach-O, 0xfeedfacf 64-bit.', 11, DIM)
    s.text(24, 310, 'A 32-bit build needs a Mac older than Catalina, or a VM. Rosetta 2 does not translate AVX instructions.', 12, DIM)
    return s.render()


# ================================================================ engine-era PC games
@diagram('engine-code.svg')
def engine_code():
    s = Svg(900, 380, 'In engine-era PC games, Mono games ship .NET assemblies and IL2CPP games ship GameAssembly.dll '
                      'with global-metadata.dat. Unreal packs assets in .pak files and keeps class names. Godot packs '
                      '.pck files.')
    s.heading('Where the logic lives in engine-era games', 'Much of the logic is outside the native executable. Each engine needs its own first tool.')
    rows = [('Unity with Mono, .NET', 'managed assemblies (IL)', 'a .NET decompiler, REA', 'near-source C#', TEAL),
            ('Unity with IL2CPP', 'GameAssembly.dll + global-metadata.dat', 'Il2CppDumper', 'names, then native code', BLUE),
            ('Unreal Engine 4 and 5', 'native code + .pak (may be AES)', 'reflection keeps class names', 'named functions', PURPLE),
            ('Godot', '.pck packs', 'gdsdecomp', 'the project and its scripts', AMBER)]
    s.text(24, 92, 'engine', 11, DIM, weight='600')
    s.text(240, 92, 'what ships', 11, DIM, weight='600')
    s.text(520, 92, 'first tool', 11, DIM, weight='600')
    s.text(720, 92, 'what you get', 11, DIM, weight='600')
    for k, (eng, ships, tool, gets, colour) in enumerate(rows):
        y = 104 + k * 56
        s.rect(24, y, 200, 44, colour, rx=4, extra=' fill-opacity="0.3"')
        s.text(36, y + 27, eng, 12, TEXT, weight='600')
        s.rect(236, y, 264, 44, GREY, rx=4, extra=' fill-opacity="0.2"')
        s.text(248, y + 27, ships, 11, TEXT)
        arrow(s, 502, y + 22, 514, y + 22)
        s.rect(516, y, 186, 44, colour, rx=4, extra=' fill-opacity="0.15"')
        s.text(528, y + 27, tool, 11, TEXT)
        arrow(s, 704, y + 22, 716, y + 22)
        s.text(720, y + 27, gets, 11, TEXT)
    s.text(24, 350, 'Check the decompiled code\'s float and integer rules against the runtime the game shipped with.', 12, DIM)
    return s.render()


@diagram('engine-second-binary.svg')
def engine_second_binary():
    s = Svg(900, 300, 'A Linux build sometimes keeps more symbols than the Windows build. Matching the functions '
                      'between the two names the Windows functions. Kernel anti-cheat can block debuggers.')
    s.heading('A Linux build as a second binary', 'Linux builds sometimes keep more symbols. Match the functions to name the Windows build.')
    names = ['Player::Update', 'Weapon::Fire', 'AI::Think', 'Save::Write']
    for k, n in enumerate(names):
        y = 94 + k * 40
        s.rect(40, y, 240, 32, GREEN, rx=4, extra=' fill-opacity="0.25"')
        s.text(52, y + 21, n, 12, TEXT, mono=True)
        s.rect(560, y, 240, 32, GREY, rx=4, extra=' fill-opacity="0.25"')
        s.add(f'<text x="572" y="{y + 21}" font-family="ui-monospace, Menlo, monospace" font-size="12" fill="{DIM}">FUN_14{k + 1:02d}2a0'
              f'<animate attributeName="opacity" values="1;0;0" keyTimes="0;{0.2 + k * 0.15:.2f};1" calcMode="discrete" dur="4s" '
              f'repeatCount="indefinite"/></text>')
        s.add(f'<text x="572" y="{y + 21}" font-family="ui-monospace, Menlo, monospace" font-size="12" fill="{TEXT}" opacity="0">{n}'
              f'<animate attributeName="opacity" values="0;1;1" keyTimes="0;{0.2 + k * 0.15:.2f};1" calcMode="discrete" dur="4s" '
              f'repeatCount="indefinite"/></text>')
        s.line(284, y + 16, 556, y + 16, DIM, 1, ' stroke-dasharray="4 4"')
    s.text(160, 84, 'Linux build (symbols)', 12, TEXT, 'middle', '600')
    s.text(680, 84, 'Windows build', 12, TEXT, 'middle', '600')
    s.text(420, 84, 'match by code', 11, DIM, 'middle')
    lines(s, 40, 268, ['Linux ports came from Loki Software (1998-2002). Kernel anti-cheat can block debuggers.'], 12)
    return s.render()
