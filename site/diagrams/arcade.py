"""Diagrams for the arcade guides (docs/arcade/): clock trees and refresh rates, board CPUs and DSPs,
memory maps, ciphers and key chips, and the practices for testing against MAME. site/diagrams/make.py
writes them. The drawing helpers come from computers.py.
"""
from computers import SANS, arrow, box, diagram as _diagram, fade, lines, memmap
from runtime import part, show
from svg import AMBER, BG, BLUE, DIM, GREEN, GREY, PURPLE, RED, TEAL, TEXT, Svg

DIAGRAMS = {}


def diagram(name):
    def wrap(fn):
        DIAGRAMS[name] = fn
        return fn
    return wrap


def clock_tree(s, x, y, master, branches, w=170, gap=56):
    """A crystal on the left and its divided clocks on the right: branches are (label, value, colour)."""
    box(s, x, y + (len(branches) - 1) * gap / 2, w, 46, AMBER, master[0], master[1], op=0.3)
    for k, (label, value, colour) in enumerate(branches):
        by = y + k * gap
        s.add(f'<path d="M{x + w + 4} {y + (len(branches) - 1) * gap / 2 + 23}C {x + w + 60} {y + (len(branches) - 1) * gap / 2 + 23} '
              f'{x + w + 60} {by + 23} {x + w + 116} {by + 23}" fill="none" stroke="{DIM}" stroke-width="1.5" '
              f'marker-end="url(#arrow)"/>')
        box(s, x + w + 120, by, 210, 46, colour, label, value, op=0.25)


def cpu_row(s, x, y, chips, h=56, gap=10):
    """A row of chip boxes: (name, detail, colour, width). Returns the end x."""
    for name, detail, colour, w in chips:
        box(s, x, y, w, h, colour, name, detail, op=0.25, sub_size=10)
        x += w + gap
    return x


# ================================================================ the arcade README
@diagram('arcade-three-problems.svg')
def arcade_three_problems():
    s = Svg(900, 330, 'Later arcade boards add three problems. From the 1980s custom CPUs decrypt the program. From '
                      '1988 3D boards split the game across a CPU and DSPs. From 2004 the boards are x86 PCs with a '
                      'keychip.')
    s.heading('The problems that later boards add', 'Each era of boards adds one problem before you can read the game\'s code.')
    x0, x1 = 40, 860
    def xo(y):
        return x0 + (y - 1972) / (2024 - 1972) * (x1 - x0)
    eras = [(1980, 'encrypted CPUs', 'Battery-backed custom CPUs decrypt', 'the program. Decrypt it first.', RED),
            (1988, 'CPU + DSPs', 'Each DSP runs its own program.', 'Ghidra lacks most DSP modules.', PURPLE),
            (2004, 'x86 PCs', 'The code is decrypted only on a', 'running machine with its keychip.', BLUE)]
    for yr in range(1972, 2025, 8):
        s.text(xo(yr), 280, str(yr), 10, DIM, 'middle', mono=True)
    for k, (yr, name, a, b, colour) in enumerate(eras):
        x = xo(yr)
        by = 206 + k * 18
        s.rect(x, by, x1 - x, 12, colour, rx=3, extra=' fill-opacity="0.6"')
        bx = 40 + k * 280
        s.rect(bx, 84, 260, 100, colour, rx=8, extra=' fill-opacity="0.15"')
        s.text(bx + 14, 108, f'{name}, from {yr if yr != 1980 else "the 1980s"}', 13, TEXT, weight='600')
        lines(s, bx + 14, 134, [a, b], 12, TEXT)
        s.line(bx + 40, 184, x + 4, by, colour, 1.2)
    s.rect(xo(1972), 206, xo(1980) - xo(1972) - 4, 12, GREY, rx=3, extra=' fill-opacity="0.4"')
    s.text(xo(1972), 200, 'plain ROMs', 10, DIM)
    s.text(40, 300, 'Arcade refresh rates also differ, from about 54.71 Hz (Midway TMS34010 boards) to about 61.5 Hz (Asteroids).', 12, DIM)
    return s.render()


@diagram('arcade-order-of-work.svg')
def arcade_order_of_work():
    s = Svg(900, 300, 'Read the MAME driver, decrypt the program, capture the DSP programs and record the DIP '
                      'settings, BIOS, region and NVRAM. Then check MAME\'s flags before you use it as ground truth.')
    s.heading('Order of work on an arcade board', 'Do these five steps before the port starts.')
    steps = [('1. MAME driver', ['clocks, screen', 'totals, flags'], TEAL), ('2. decrypt', ['wherever the board', 'has a custom CPU'], RED),
             ('3. DSP programs', ['on a 3D board', 'they hold game logic'], PURPLE),
             ('4. test inputs', ['DIPs, BIOS, region,', 'NVRAM image'], AMBER), ('5. check flags', ['MAME as ground', 'truth, or not'], BLUE)]
    dur = 7.5
    for k, (name, rows, colour) in enumerate(steps):
        x = 24 + k * 174
        s.rect(x, 90, 154, 110, colour, rx=8, extra=' fill-opacity="0.2"')
        s.text(x + 12, 116, name, 13, TEXT, weight='600')
        lines(s, x + 12, 142, rows, 11, TEXT, 17)
        if k:
            arrow(s, x - 18, 145, x - 3, 145)
        s.add(f'<rect x="{x}" y="90" width="154" height="110" rx="8" fill="none" stroke="{AMBER}" stroke-width="2.5" opacity="0">'
              f'<animate attributeName="opacity" values="0;1;0;0" keyTimes="0;{k / 5:.2f};{(k + 1) / 5:.2f};1" calcMode="discrete" '
              f'dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.text(24, 240, 'If MAME does not run a game, take ground truth from another emulator or from owned hardware.', 12, DIM)
    s.text(24, 262, 'A port must run at the board\'s own refresh rate to stay in step with a recording.', 12, DIM)
    return s.render()


# ================================================================ discrete logic and the 8080
@diagram('arcade-pong-netlist.svg')
def arcade_pong_netlist():
    s = Svg(900, 300, 'Pong has no CPU. Counters and gates driven by a clock make the picture. MAME emulates it as a '
                      'netlist, and a port simulates the circuit.')
    s.heading('Pong as a circuit of counters and gates', 'Counters and logic gates make the picture. There is no code to disassemble.')
    box(s, 24, 120, 130, 56, AMBER, 'clock', op=0.3)
    box(s, 200, 90, 160, 50, TEAL, 'horizontal counter', op=0.25, size=12)
    box(s, 200, 156, 160, 50, TEAL, 'vertical counter', op=0.25, size=12)
    box(s, 410, 90, 160, 50, BLUE, 'ball and paddle', 'position counters', op=0.25)
    box(s, 410, 156, 160, 50, PURPLE, 'gates', 'compare and combine', op=0.25)
    box(s, 620, 120, 130, 56, GREEN, 'video signal', op=0.3)
    arrow(s, 158, 140, 196, 115)
    arrow(s, 158, 156, 196, 181)
    arrow(s, 364, 115, 406, 115)
    arrow(s, 364, 181, 406, 181)
    arrow(s, 574, 115, 616, 140)
    arrow(s, 574, 181, 616, 156)
    s.rect(770, 96, 110, 104, '#000000', GREY, rx=4)
    s.rect(780, 130, 4, 24, TEXT, rx=0)
    s.rect(866, 120, 4, 24, TEXT, rx=0)
    arrow(s, 754, 148, 766, 148)
    s.add(f'<rect width="5" height="5" fill="{TEXT}"><animate attributeName="x" values="788;860;788" dur="2s" repeatCount="indefinite"/>'
          f'<animate attributeName="y" values="104;190;104" dur="1.3s" repeatCount="indefinite" data-drift="free"/></rect>')
    lines(s, 24, 246, ['The game\'s behaviour comes from the gates, the counters and their timing. Use MAME\'s netlist as the',
                       'description of the circuit. Breakout uses the same approach.'], 12)
    return s.render()


@diagram('arcade-invaders-board.svg')
def arcade_invaders_board():
    s = Svg(900, 360, 'On the Midway 8080 board a 19.968 MHz master clock gives the 8080 at 1.9968 MHz and a 4.992 MHz '
                      'pixel clock. With totals of 320 by 262 the screen runs at about 59.54 Hz. The MB14241 shifter '
                      'and analog sound sit beside it.')
    s.heading('The Midway 8080 board of Space Invaders', 'One crystal sets the CPU, the pixel clock and the refresh rate.')
    clock_tree(s, 24, 90, ('19.968 MHz', 'master crystal'),
               [('Intel 8080', 'master / 10 = 1.9968 MHz', BLUE), ('pixel clock', 'master / 4 = 4.992 MHz', TEAL)], w=160, gap=70)
    arrow(s, 520, 183, 560, 183)
    box(s, 564, 156, 300, 54, GREEN, '4.992 MHz / (320 x 262)', 'about 59.54 Hz (computed)', op=0.25, mono=True)
    box(s, 24, 250, 260, 54, PURPLE, 'Fujitsu MB14241', 'shifter that helps draw sprites', op=0.22)
    box(s, 300, 250, 260, 54, RED, 'analog sound + SN76477', 'part circuit, part chip', op=0.22)
    s.text(24, 336, 'An exact port needs the 8080 code, the shifter as the game uses it and a model of the analog sound.', 12, DIM)
    return s.render()


@diagram('arcade-mb14241.svg')
def arcade_mb14241():
    s = Svg(900, 330, 'The game writes two bytes and a shift amount to the MB14241 shifter. It reads back 8 bits taken '
                      'at that offset, so a sprite byte is shifted into place without CPU shifts.')
    s.heading('The MB14241 shifter', 'The 8080 has no fast multi-bit shift. The chip shifts a 16-bit value for it.')
    s.text(24, 96, 'write data: each byte pushes the older one to the low half', 12, TEXT, weight='600')
    hi = '11000011'
    lo = '01111110'
    for k, b in enumerate(hi + lo):
        x = 24 + k * 30
        s.rect(x, 108, 28, 30, AMBER if k < 8 else TEAL, rx=2, extra=' fill-opacity="0.3"')
        s.text(x + 14, 128, b, 13, TEXT, 'middle', '600', mono=True)
    s.text(24 + 4 * 30, 156, 'newest byte', 11, DIM, 'middle')
    s.text(24 + 12 * 30, 156, 'previous byte', 11, DIM, 'middle')
    dur = 6.0
    def window(shift):
        def body(t):
            x = 24 + shift * 30
            t.rect(x - 2, 104, 8 * 30 + 2, 38, 'none', TEXT, rx=4, extra=' stroke-width="2.5"')
            t.text(560, 128, f'shift amount = {shift}', 14, AMBER, weight='600', mono=True)
            bits = (hi + lo)[shift:shift + 8]
            for k, b in enumerate(bits):
                t.rect(24 + k * 30, 210, 28, 30, PURPLE, rx=2, extra=' fill-opacity="0.35"')
                t.text(38 + k * 30, 230, b, 13, TEXT, 'middle', '600', mono=True)
            t.text(24, 200, 'read result: 8 bits from the window', 12, TEXT, weight='600')
        return part(body)
    s.add(show([window(n) for n in (0, 2, 5, 7)], dur))
    lines(s, 24, 280, ['The shift amount and the data go to separate output ports. The result comes from an input port.',
                       'Reproduce the shifter as the game uses it.'], 12)
    return s.render()


# ================================================================ Z80 and 6502 boards
@diagram('arcade-namco-clocks.svg')
def arcade_namco_clocks():
    s = Svg(900, 410, 'On Namco\'s early boards an 18.432 MHz crystal gives Z80s at 3.072 MHz and a 6.144 MHz pixel '
                      'clock. The 384 by 264 totals give about 60.61 Hz. Galaga runs three Z80s with custom chips on a '
                      '06xx bus.')
    s.heading('Namco Galaxian, Pac-Man and Galaga', 'One 18.432 MHz crystal drives each board. Galaga adds two more Z80s and custom chips.')
    clock_tree(s, 24, 84, ('18.432 MHz', 'master crystal'),
               [('Z80', '/ 6 = 3.072 MHz', BLUE), ('pixel clock', '/ 3 = 6.144 MHz', TEAL), ('51xx, 54xx', '1.536 MHz (Galaga)', PURPLE)],
               w=150, gap=58)
    box(s, 540, 142, 330, 54, GREEN, '6.144 MHz / (384 x 264)', 'about 60.61 Hz (computed)', op=0.25, mono=True)
    arrow(s, 504, 165, 536, 168)
    s.text(24, 290, 'Galaga: three Z80s', 12, TEXT, weight='600')
    for k, name in enumerate(('main Z80', 'sub Z80', 'sound Z80')):
        box(s, 24 + k * 130, 300, 120, 44, BLUE, name, op=0.25, size=12)
    s.line(24, 352, 850, 352, DIM, 2)
    for x in (84, 214, 344, 615, 775):
        s.line(x, 344, x, 352, DIM, 2)
    s.text(437, 366, '06xx bus interface', 11, DIM, 'middle')
    box(s, 540, 300, 150, 44, PURPLE, '51xx', 'I/O', op=0.25)
    box(s, 700, 300, 150, 44, PURPLE, '54xx', 'noise sound', op=0.25)
    s.text(24, 392, 'Run the three CPUs interleaved in the right order. Pac-Man shows 288 x 224 of the 384 x 264 total.', 12, DIM)
    return s.render()


@diagram('arcade-early-boards.svg')
def arcade_early_boards():
    s = Svg(900, 412, 'Donkey Kong has a Z80 and an 8035 sound CPU. Sega System 1 has Z80s and two SN76489A. '
                      'Nintendo\'s VS. System and PlayChoice-10 are built from NES chips.')
    s.heading('Donkey Kong, Sega System 1, VS. System', 'Several of these boards carry a second CPU for sound. Each CPU is a program to trace.')
    rows = [('Donkey Kong (1981)', [('Z80', '4 MHz, game', BLUE, 150), ('Intel 8035', '6 MHz, sound', PURPLE, 150)],
             '61.44 MHz crystal, 6.144 MHz pixel clock, about 60.61 Hz'),
            ('Sega System 1', [('Z80', '4 MHz, from 20 MHz', BLUE, 150), ('sound CPU', '20/8 or 20/16 MHz', PURPLE, 150),
                               ('2 x SN76489A', 'by board revision', RED, 150)], 'about 60.10 Hz; encrypted Z80 parts and the MC8123'),
            ('VS. System', [('RP2A03', 'NES CPU + sound', BLUE, 150), ('2C0x PPU', '341 dots a line', TEAL, 150)],
             'NTSC line count; dual-screen games have two CPUs'),
            ('PlayChoice-10', [('Z80', '4 MHz, the menu', BLUE, 150), ('NES hardware', 'the game', TEAL, 150)],
             'nominal 60 Hz in MAME. Treat the rate as unknown.')]
    for k, (name, chips, note) in enumerate(rows):
        y = 84 + k * 80
        s.text(24, y + 26, name, 13, TEXT, weight='600')
        x = 190
        for cname, detail, colour, w in chips:
            box(s, x, y, w, 50, colour, cname, detail, op=0.25, sub_size=10)
            x += w + 8
        s.text(190, y + 66, note, 11, DIM)
    return s.render()


@diagram('arcade-asteroids-vector.svg')
def arcade_asteroids_vector():
    s = Svg(900, 360, 'Asteroids draws on a vector display, where each frame the beam strokes a list of lines. The '
                      '12.096 MHz master clock gives a 6502 at 1.512 MHz and a vector refresh of about 61.5 Hz.')
    s.heading('Asteroids draws with the beam', 'A vector game draws a list of strokes each frame. Compare these lists before you compare pixels.')
    pts = [(120, 160), (170, 120), (230, 140), (250, 200), (200, 250), (130, 230), (120, 160)]
    ship = [(420, 220), (440, 170), (460, 220), (440, 208), (420, 220)]
    def path(p):
        return 'M' + 'L'.join(f'{x} {y}' for x, y in p)
    total = 0
    for poly in (pts, ship):
        d = path(poly)
        length = sum(((poly[i + 1][0] - poly[i][0]) ** 2 + (poly[i + 1][1] - poly[i][1]) ** 2) ** 0.5 for i in range(len(poly) - 1))
        s.add(f'<path d="{d}" fill="none" stroke="#dff" stroke-width="2" stroke-dasharray="{length:.0f}" stroke-dashoffset="{length:.0f}">'
              f'<animate attributeName="stroke-dashoffset" values="{length:.0f};0;0" keyTimes="0;0.6;1" dur="2s" repeatCount="indefinite"/></path>')
    s.rect(60, 90, 460, 200, 'none', GREY, rx=6)
    lines(s, 560, 104, ['stroke list, one frame:', 'MOVE 120,160', 'DRAW 170,120', 'DRAW 230,140', '...', 'MOVE 420,220', 'DRAW 440,170', '...'],
          12, TEXT, 18, mono=True)
    lines(s, 60, 316, ['12.096 MHz master: 6502 at 1.512 MHz. Vector refresh (12.096 MHz / 4096) / 12 / 4 = about 61.5 Hz.',
                       'Lunar Lander divides by 6 instead of 4. Asteroids Deluxe adds a POKEY.'], 12)
    s.text(560, 268, 'The coordinates are an illustration.', 10, DIM)
    return s.render()


# ================================================================ Williams and Midway
@diagram('arcade-williams-map.svg')
def arcade_williams_map():
    s = Svg(900, 330, 'The Williams 6809 board has a blitter at CA00 to CA07 on games that have one, a watchdog at '
                      'CBFF and 1K by 4 bits of battery-backed CMOS RAM at CC00 to CFFF for settings and high scores.')
    s.heading('Williams 6809 boards', 'The MC6809E runs at 1 MHz. An 8 MHz pixel clock and 512 x 260 totals give about 60.10 Hz.')
    memmap(s, 120, 90, 260, [('CA00', 'blitter (not Defender, Stargate)', 40, BLUE), ('CA08', '', 30, GREY),
                             ('CBFF', 'watchdog', 24, RED), ('CC00', 'CMOS RAM, 1K x 4 bits', 64, AMBER)])
    s.text(120, 290, 'Only these entries come from the page. The rest of the map is left out.', 10, DIM)
    lines(s, 460, 104, ['Battery-backed CMOS holds the settings and', 'high scores. Load it as an input to each test.', '',
                        'Early games use an M6802 sound CPU.', 'Later ones use an M6808.', '',
                        'Original assembly source exists. The shipped', 'ROM decides exact behaviour.'], 12)
    return s.render()


@diagram('arcade-midway-units.svg')
def arcade_midway_units():
    s = Svg(900, 360, 'The Midway Y-unit runs its TMS34010 at 40, 48 or 50 MHz and the T-unit at 50 MHz with DCS '
                      'sound. The W-unit adds an ADSP-2105 for DCS and a PIC16C57 security chip. The screen runs at '
                      'about 54.71 Hz.')
    s.heading('Midway Y, T and W-units', 'The TMS34010 graphics CPU runs the game. The screen runs at about 54.71 Hz (506 x 289 totals).')
    units = [('Y-unit, 1988 on', 'Smash TV, Mortal Kombat', ['TMS34010: 40 MHz "slow"', 'or 48 MHz "fast", 50 MHz T2', 'CVSD or ADPCM sound',
                                                             'Z80 + MSM6295 on ADPCM sets'], BLUE),
             ('T-unit, 1992 on', 'Mortal Kombat, NBA Jam', ['TMS34010 at 50 MHz', '4 MHz pixel clock', 'DCS sound (2K variant)'], PURPLE),
             ('W-unit, 1994 on', 'MK3, WrestleMania', ['TMS34010', 'ADSP-2105 at 10 MHz', 'for DCS (8K variant)', 'PIC16C57 security chip'], AMBER)]
    for k, (name, games, rows, colour) in enumerate(units):
        x = 24 + k * 286
        s.rect(x, 80, 272, 184, colour, rx=8, extra=' fill-opacity="0.15"')
        s.text(x + 14, 104, name, 14, TEXT, weight='600')
        s.text(x + 14, 122, games, 11, DIM)
        lines(s, x + 14, 152, rows, 12, TEXT, 20)
    lines(s, 24, 300, ['A port at 60 Hz shows about 317 more frames per minute than a 54.71 Hz board.',
                       'Attract mode, audits, adjustments and diagnostics are separate modules in the source. Rebuild them all.'], 12)
    return s.render()


@diagram('arcade-midway-zeus.svg')
def arcade_midway_zeus():
    s = Svg(900, 300, 'On Midway Zeus a TI TMS32C032 DSP at 60 MHz runs the game and drives the Zeus custom 3D '
                      'controller. The Zeus chip has 512 registers in the CPU\'s address space and 16 MB of Wave RAM '
                      'in two banks.')
    s.heading('Midway Zeus (1997)', 'The game logic runs on a DSP. Plan for a TMS320 disassembler or a small core.')
    box(s, 40, 100, 220, 70, BLUE, 'TMS32C032 DSP', '60 MHz, runs the game', op=0.25)
    box(s, 340, 100, 220, 70, PURPLE, 'Zeus', 'Midway custom 3D controller', op=0.25)
    arrow(s, 264, 135, 336, 135)
    s.text(300, 126, '512 registers', 10, DIM, 'middle')
    box(s, 640, 84, 220, 46, GREEN, 'Wave RAM bank 0', '8 MB', op=0.25)
    box(s, 640, 140, 220, 46, GREEN, 'Wave RAM bank 1', '8 MB', op=0.25)
    arrow(s, 564, 125, 636, 107)
    arrow(s, 564, 145, 636, 163)
    lines(s, 40, 226, ['Games: Mortal Kombat 4, Invasion. MAME\'s emulation is imperfect.',
                       'Check MAME\'s Zeus output against another record before trusting it as ground truth.'], 12)
    return s.render()


# ================================================================ Atari coin-op
@diagram('arcade-atari-boards.svg')
def arcade_atari_boards():
    s = Svg(900, 380, 'Atari System 1 is a BIOS board with game cartridges. A 68010 runs the game and a 6502 runs the '
                      'sound with a YM2151, a POKEY and a TMS5220C. System 2 uses a DEC T-11. The GT board uses a '
                      '68EC020 with the CAGE sound board.')
    s.heading('Atari System 1, System 2 and GT', 'Every board has a separate sound CPU. Each CPU is a separate program to trace.')
    rows = [('System 1 (1984)', [('MC68010', '7.16 MHz, game', BLUE, 160), ('6502', '1.79 MHz, sound', PURPLE, 150),
                                 ('YM2151, POKEY', 'TMS5220C speech', RED, 170)], 'BIOS board + game cartridges. 336 x 240, about 59.92 Hz.'),
            ('System 2 (1984)', [('DEC T-11', '10 MHz, game', AMBER, 160), ('6502', 'as System 1', PURPLE, 150)],
             'Paperboy. 512 x 384, about 60.10 Hz. No tool here supports the T-11.'),
            ('GT (1994)', [('MC68EC020', '25 MHz, game', BLUE, 160), ('CAGE', 'sound board', PURPLE, 150)],
             'T-MEK, Primal Rage. About 59.92 Hz.')]
    for k, (name, chips, note) in enumerate(rows):
        y = 86 + k * 92
        s.text(24, y + 30, name, 13, TEXT, weight='600')
        x = 170
        for cname, detail, colour, w in chips:
            box(s, x, y, w, 52, colour, cname, detail, op=0.25, sub_size=10)
            x += w + 8
        s.text(170, y + 72, note, 11, DIM)
    return s.render()


@diagram('arcade-hard-drivin.svg')
def arcade_hard_drivin():
    s = Svg(900, 330, 'Hard Drivin\' runs a 68010 for the game, a TMS34010 GSP for graphics and sometimes a second '
                      'TMS34010 MSP. Its polygon DSPs differ by board, from an ADSP-2100 to a DSP32C.')
    s.heading('Atari Hard Drivin\' (1988)', 'Each game uses three or more instruction sets. The DSP programs are part of the game.')
    box(s, 24, 100, 170, 60, BLUE, 'M68010', '8 MHz, game logic', op=0.25)
    box(s, 230, 84, 190, 46, TEAL, 'TMS34010 GSP', 'graphics, 48 MHz crystal', op=0.25, sub_size=10)
    box(s, 230, 140, 190, 46, TEAL, 'TMS34010 MSP', 'some boards, 50 MHz', op=0.15, sub_size=10)
    s.text(470, 92, 'polygon DSP, by board', 12, TEXT, weight='600')
    dsps = [('ADSP-2100', '32 MHz / 4'), ('ADSP-2101', '12 MHz'), ('2 x ADSP-2105', '10 MHz, DS III'), ('DSP32C', '40 MHz')]
    for k, (n, c) in enumerate(dsps):
        box(s, 470 + (k % 2) * 200, 102 + (k // 2) * 56, 190, 46, PURPLE, n, c, op=0.25, sub_size=10)
    arrow(s, 198, 125, 226, 108)
    arrow(s, 198, 135, 226, 162)
    lines(s, 24, 250, ['Record which board variant and which DSP the set uses.', 'Ghidra has the 68000 built in and the TMS34010 by extension.',
                       'It has nothing for these DSPs.'], 12)
    return s.render()


# ================================================================ Sega 2D
@diagram('arcade-sega-2d.svg')
def arcade_sega_2d():
    s = Svg(900, 384, 'System 16B has one 68000 and an option for an encrypted CPU. System 18 runs at a measured 57.23 '
                      'Hz. The X Board has two 68000s and the Y Board has three, which run interleaved.')
    s.heading('Sega System 16B, 18, X and Y Boards', 'Later boards have more 68000s. A port must run them interleaved.')
    boards = [('System 16B', 1, '10 MHz', 'Z80 5 MHz, YM2151', 'about 60.05 Hz (computed)', 'FD1089, FD1094, MC8123, 8751'),
              ('System 18', 1, '10 MHz', 'Z80 8 MHz, YM3438, RF5C68', '57.23 Hz (verified on pcb)', ''),
              ('X Board', 2, '12.5 MHz', '315-5248 multiplier, 315-5250', 'about 59.64 Hz (computed)', 'After Burner'),
              ('Y Board', 3, '12.5 MHz', 'Galaxy Force 2, Power Drift', 'nominal 60 Hz: unknown', '')]
    for k, (name, n, clk, other, hz, note) in enumerate(boards):
        y = 84 + k * 64
        s.text(24, y + 30, name, 13, TEXT, weight='600')
        for c in range(n):
            box(s, 150 + c * 70, y + 4, 62, 46, BLUE, '68000', clk, op=0.3, size=11, sub_size=9)
        s.text(370, y + 22, other, 11, TEXT)
        s.text(370, y + 40, note, 11, DIM)
        s.text(860, y + 30, hz, 11, GREEN if 'verified' in hz else (RED if 'unknown' in hz else DIM), 'end')
    s.text(24, 368, 'Model the 315-5195 mapper (16B) and the 315-5248 and 315-5250 chips (X Board) as the CPUs see them.', 12, DIM)
    return s.render()


# ================================================================ Capcom CPS
@diagram('arcade-cps1.svg')
def arcade_cps1():
    s = Svg(900, 330, 'Capcom CPS-1 has a 68000 at 10 or 12 MHz and a Z80 at 3.58 MHz with a YM2151 and an MSM6295. '
                      'QSound games use a Kabuki Z80 at 8 MHz. An 8 MHz pixel clock with 512 by 262 totals gives about '
                      '59.64 Hz.')
    s.heading('Capcom CPS-1 (1988)', 'CPS-1 ran Forgotten Worlds, Final Fight and Street Fighter II. It has one 68000 and a Z80 for sound.')
    cpu_row(s, 24, 90, [('68000', '10 MHz (verified); later 12', BLUE, 200), ('Z80', '3.58 MHz', PURPLE, 130),
                        ('YM2151', 'FM', RED, 120), ('MSM6295', '1 MHz, samples', RED, 140), ('B-board PALs', 'protected', GREY, 150)])
    s.text(24, 182, 'QSound games (Warriors of Fate, 1992)', 12, TEXT, weight='600')
    cpu_row(s, 24, 194, [('Kabuki Z80', '8 MHz, encrypted', AMBER, 200), ('QSound', '250 Hz audio interrupt', PURPLE, 200)])
    box(s, 500, 194, 370, 56, GREEN, '8 MHz / (512 x 262)', 'about 59.64 Hz; 384 x 224 visible', op=0.25)
    s.text(24, 300, 'Record the 68000 clock of the board revision. At 60 Hz a port shows about 22 more frames a minute.', 12, DIM)
    return s.render()


@diagram('arcade-cps2.svg')
def arcade_cps2():
    s = Svg(900, 360, 'The CPS-2 A-board holds the 68000, RAM, graphics, sound and a 93C46 EEPROM. The B-board holds '
                      'the game ROMs and the custom chips, including the battery-backed key. QSound runs on a DSP-16A.')
    s.heading('Capcom CPS-2 (1993 to 2004)', 'CPS-2 has two boards in a box coloured by region. About 50 games ran on it, in over 170 variants.')
    s.rect(24, 84, 410, 200, BLUE, rx=8, extra=' fill-opacity="0.1"')
    s.text(40, 108, 'A-board', 14, TEXT, weight='600')
    cpu_row(s, 40, 120, [('68000', '16 MHz', BLUE, 120), ('Z80', '8 MHz', PURPLE, 110), ('93C46', 'EEPROM', AMBER, 130)])
    cpu_row(s, 40, 190, [('QSound', 'DSP-16A at 60 MHz', PURPLE, 190), ('graphics, I/O', '', GREY, 180)])
    s.rect(466, 84, 410, 200, RED, rx=8, extra=' fill-opacity="0.1"')
    s.text(482, 108, 'B-board', 14, TEXT, weight='600')
    cpu_row(s, 482, 120, [('game ROMs', 'program encrypted', RED, 180), ('custom chips', '', GREY, 180)])
    box(s, 482, 190, 370, 56, AMBER, 'battery-backed key', 'a dead battery gives a blue or green screen', op=0.3)
    s.add(f'<rect x="482" y="190" width="370" height="56" rx="6" fill="none" stroke="{AMBER}" stroke-width="2">'
          f'<animate attributeName="stroke-opacity" values="1;0.1;1" dur="2s" repeatCount="indefinite"/></rect>')
    lines(s, 24, 312, ['Measured on a board: VSync 59.6388 Hz, HSync 15.4445 kHz. Its raw timing gives 59.637 Hz.',
                       'Load the 93C46 image as an input to each test. No CPS-2 game encrypted its sound program.'], 12)
    return s.render()


@diagram('arcade-cps3.svg')
def arcade_cps3():
    s = Svg(900, 330, 'The CPS-3 security cartridge holds the encrypted SH-2 derivative, a boot flash ROM and NVRAM. A '
                      'SCSI CD-ROM loads the game into custom flash SIMMs. The screen runs at 59.59 Hz.')
    s.heading('Capcom CPS-3 (1996 to 1999)', 'Six games ran on CPS-3, among them Red Earth and Street Fighter III. The game loads from CD.')
    box(s, 24, 96, 170, 60, GREY, 'SCSI CD-ROM', 'the game', op=0.25)
    box(s, 260, 84, 220, 84, GREEN, 'flash SIMMs', 'program 64 Mbit; graphics and sound 128 Mbit', op=0.25, sub_size=10)
    s.rect(560, 76, 310, 160, AMBER, rx=8, extra=' fill-opacity="0.12"')
    s.text(576, 100, 'security cartridge', 13, TEXT, weight='600')
    box(s, 576, 112, 280, 50, RED, 'SH-2 derivative (SH7014 core)', 'key in battery SRAM on the die', op=0.25, sub_size=10)
    box(s, 576, 172, 135, 50, AMBER, 'boot flash', op=0.25, size=12)
    box(s, 721, 172, 135, 50, AMBER, 'NVRAM', op=0.25, size=12)
    arrow(s, 198, 126, 256, 126)
    arrow(s, 484, 126, 572, 136)
    lines(s, 24, 268, ['Only the boot flash and the program areas are encrypted. MAME models the CPU as an SH7604 at 25 MHz.',
                       'The CPU erases its key on tampering. Screen: 42.954545 MHz / 5, 59.59 Hz.'], 12)
    return s.render()


# ================================================================ Neo Geo
@diagram('arcade-neogeo-clocks.svg')
def arcade_neogeo_clocks():
    s = Svg(900, 360, 'On the Neo Geo MVS a 24 MHz master clock gives a 68000 at 12 MHz, a Z80 at 4 MHz, a YM2610 at 8 '
                      'MHz and a 6 MHz pixel clock. The 384 by 264 totals give about 59.19 Hz. Most games use 304 of '
                      'the 320 visible pixels.')
    s.heading('Neo Geo MVS clocks', 'One 24 MHz master clock drives the board. Motherboards had 1, 2, 4 or 6 slots and were made from 1990 to 2004.')
    clock_tree(s, 24, 84, ('24 MHz', 'master clock'),
               [('68000', '/ 2 = 12 MHz', BLUE), ('Z80', '/ 6 = 4 MHz', PURPLE), ('YM2610', '/ 3 = 8 MHz', RED), ('pixel clock', '/ 4 = 6 MHz', TEAL)],
               w=150, gap=56)
    box(s, 540, 190, 330, 54, GREEN, '6 MHz / (384 x 264)', 'about 59.19 Hz (computed)', op=0.25, mono=True)
    arrow(s, 504, 275, 560, 248)
    s.rect(540, 92, 320, 70, GREY, rx=4, extra=' fill-opacity="0.25"')
    s.rect(548, 98, 304, 58, TEAL, rx=2, extra=' fill-opacity="0.3"')
    s.text(700, 132, '304 of 320 pixels, used by most games', 11, TEXT, 'middle')
    s.text(540, 84, '320 x 224 visible', 11, DIM)
    s.text(24, 340, 'MVS motherboards differ by generation (MV-1 to MV-1C). The AES home system\'s timing differs from the MVS.', 12, DIM)
    return s.render()


@diagram('arcade-neogeo-roms.svg')
def arcade_neogeo_roms():
    s = Svg(900, 390, 'On late Neo Geo games, NEO-SMA decrypts the P-ROM program and has a random number generator. '
                      'NEO-CMC decrypts the C-ROM graphics and banks the S-ROM, and on some boards decrypts the M-ROM. '
                      'NEO-PCM2 decrypts the V-ROM samples.')
    s.heading('Neo Geo ROMs and protection chips', 'Late games, such as The King of Fighters \'99, 2000 and 2003, have chips that decrypt their ROMs.')
    roms = [('P-ROM', '68000 program', BLUE), ('S-ROM', 'fix layer', TEAL), ('M-ROM', 'Z80 sound', PURPLE),
            ('V-ROM', 'ADPCM samples', RED), ('C-ROM', 'sprites', AMBER)]
    for k, (name, what, colour) in enumerate(roms):
        box(s, 180 + k * 140, 84, 130, 50, colour, name, what, op=0.25)
        s.line(245 + k * 140, 138, 245 + k * 140, 300, DIM, 1, ' stroke-dasharray="3 3"')
    chips = [('NEO-SMA', [0]), ('NEO-CMC 7042', [4, 1]), ('NEO-CMC 7050', [4, 1, 2]), ('NEO-PCM2', [3, 0])]
    for k, (name, targets) in enumerate(chips):
        y = 150 + k * 38
        s.rect(24, y, 140, 28, GREY, rx=3, extra=' fill-opacity="0.35"')
        s.text(94, y + 19, name, 11, TEXT, 'middle', '600')
        s.line(168, y + 14, 870, y + 14, GREY, 1, ' stroke-opacity="0.4"')
        for tg in targets:
            s.add(f'<circle cx="{245 + tg * 140}" cy="{y + 14}" r="8" fill="{roms[tg][2]}"/>')
    lines(s, 24, 318, ['NEO-SMA decrypts and banks the P-ROM, gives random numbers and stores 256 kb of game data.',
                       'NEO-CMC decrypts the C-ROM and banks the S-ROM; the 7050 also decrypts the M-ROM. NEO-PCM2 decrypts the V-ROM and decodes the P-ROM.'], 11)
    s.text(24, 372, 'On a NEO-SMA game, reproduce the chip\'s random number generator from MAME\'s model.', 12, DIM)
    return s.render()


# ================================================================ other 2D makers
@diagram('arcade-other-2d.svg')
def arcade_other_2d():
    s = Svg(900, 452, 'The table lists the main CPUs, sound chips, refresh rates and MCUs of the 2D boards of Namco, '
                      'Konami, Taito, Irem, Data East, Toaplan and Cave.')
    s.heading('The boards of other 2D makers', 'Most run a 68000 with a Z80 for sound. Each maker has its own crystal, so each has its own refresh rate.')
    rows = [('Namco System 1', '3 x MC6809E, 1.536 MHz', 'YM2151, CUS30', '60.61', 'HD63701, 6.144 MHz'),
            ('Namco System 2', '2 x 68000, 12.288 MHz', 'YM2151, C140', '60.61', ''),
            ('Konami (TMNT)', '68000, 8 MHz', 'Z80, YM2151', '59.19', ''),
            ('Taito F2', '68000, 12 MHz', 'Z80, YM2610', '60 (nominal)', ''),
            ('Taito F3', '68EC020', '68000, ES5505, ES5510', '58.94', ''),
            ('Irem M72', 'NEC V30, 8 MHz', 'Z80, YM2151, DAC', '55.02', '8751 or 80C31 (protection)'),
            ('Irem M92', 'NEC V33, 9 MHz', 'NEC V35, 14.318 MHz', '60.01', ''),
            ('Data East DEC0', '68000, 10 MHz', '65C02, YM2203, YM3812, MSM6295', '57.44', '8751, M68705R3 (protection)'),
            ('Toaplan 1', '68000, 10 MHz', 'Z80, YM3812', '', ''),
            ('Cave 68000', '68000, 16 MHz', 'YMZ280B, or others', '57.55', '')]
    heads = [('board', 24), ('main CPU', 170), ('sound', 360), ('Hz', 610), ('MCU', 700)]
    for name, x in heads:
        s.text(x, 90, name, 11, DIM, weight='600')
    for k, (name, cpu, snd, hz, mcu) in enumerate(rows):
        y = 100 + k * 32
        s.rect(20, y, 860, 28, GREY if k % 2 else BG, rx=3, extra=' fill-opacity="0.15"')
        s.text(24, y + 19, name, 12, TEXT, weight='600')
        s.text(170, y + 19, cpu, 11, TEXT)
        s.text(360, y + 19, snd, 11, TEXT)
        if hz:
            v = float(hz.split()[0])
            s.rect(610, y + 6, (v - 54) * 12, 16, AMBER if 'nominal' in hz else TEAL, rx=2, extra=' fill-opacity="0.45"')
            s.text(614, y + 19, hz, 10, TEXT, mono=True)
        s.text(700, y + 19, mcu, 11, RED if 'protection' in mcu else TEXT)
    s.text(24, 440, 'Most rates are computed from MAME\'s screen timing. Taito F2\'s rate is nominal. Treat it as unknown.', 11, DIM)
    return s.render()


@diagram('arcade-taito-f3.svg')
def arcade_taito_f3():
    s = Svg(900, 300, 'Taito F3 has a 68EC020 main CPU that MAME sets to 16 MHz, although it should be 30.47618 MHz / '
                      '2. The Taito EN sound board has a 68000, an Ensoniq ES5505, an ES5510 effects DSP and an '
                      'MC68681 DUART.')
    s.heading('Taito F3 (1992)', 'MAME marks its own F3 CPU clock as wrong. The sound board has its own 68000 and DSP.')
    box(s, 24, 96, 240, 64, BLUE, '68EC020', 'MAME: 16 MHz; "should be 30.47618 / 2"', op=0.25, sub_size=10)
    s.rect(300, 80, 576, 100, PURPLE, rx=8, extra=' fill-opacity="0.12"')
    s.text(316, 102, 'Taito EN sound board', 12, TEXT, weight='600')
    cpu_row(s, 316, 112, [('68000', '15.238 MHz', BLUE, 120), ('ES5505', 'Ensoniq', RED, 120), ('ES5510', 'effects DSP, 10 MHz', RED, 150),
                          ('MC68681', 'DUART', GREY, 120)], h=54, gap=6)
    lines(s, 24, 218, ['Screen: 26.686 MHz / 4, 432 x 262 totals, 58.94 Hz, from the crystal and community measurements.',
                       'Another F3 configuration uses 58.97 Hz.'], 12)
    return s.render()


# ================================================================ early 3D boards
@diagram('arcade-namco-s21.svg')
def arcade_namco_s21():
    s = Svg(900, 360, 'On Namco System 21 in Cyber Sled and Star Blade, two 68000s send a display list to a master C67 '
                      'DSP. The master applies high-level geometry and hands primitives to four slave C67s that '
                      'transform, project and clip them.')
    s.heading('Namco System 21 "Polygonizer"', 'On Cyber Sled and Star Blade, five DSPs share the geometry. Each runs its own program.')
    box(s, 24, 140, 150, 64, BLUE, '2 x M68000', '12.288 MHz, game', op=0.25)
    box(s, 250, 140, 160, 64, PURPLE, 'master C67', 'parses the display list', op=0.3, sub_size=10)
    for k in range(4):
        box(s, 500, 84 + k * 56, 170, 44, PURPLE, f'slave C67 {k + 1}', 'transform, project, clip', op=0.2, sub_size=9)
        arrow(s, 414, 172, 496, 106 + k * 56)
        arrow(s, 674, 106 + k * 56, 736, 172)
    box(s, 740, 140, 130, 64, GREEN, 'polygons', 'to the screen', op=0.25)
    arrow(s, 178, 172, 246, 172)
    s.text(212, 164, 'display list', 10, DIM, 'middle')
    for k in range(4):
        s.add(f'<circle r="4" fill="{AMBER}"><animateMotion path="M414 172L496 {106 + k * 56}L674 {106 + k * 56}L736 172" dur="2s" '
              f'begin="{k * 0.5 - 2:.1f}s" repeatCount="indefinite"/></circle>')
    lines(s, 24, 330 - 12, ['A C67 is a TMS320C25 with custom ROM. MAME clocks them from a 40 MHz crystal. Winning Run and Driver\'s Eyes',
                            'use one TMS320C25 each. Ghidra has no TMS320C25 module.'], 12)
    return s.render()


@diagram('arcade-namco-s22.svg')
def arcade_namco_s22():
    s = Svg(900, 300, 'On Namco System 22 the main CPU uploads code to the master DSP, which uploads code to the slave '
                      'DSP. Sound and I/O run on a Mitsubishi MCU with a C352 PCM chip.')
    s.heading('Namco System 22 and Super System 22', 'The DSP programs are uploaded at start. The Prop Cycle test menu shows the uploads.')
    box(s, 24, 96, 200, 64, BLUE, 'M68020', '49.152 MHz / 2', op=0.25)
    box(s, 330, 96, 200, 64, PURPLE, 'master DSP', 'TI320C25 (C71)', op=0.25)
    box(s, 636, 96, 200, 64, PURPLE, 'slave DSP', 'TI320C25 (C71)', op=0.25)
    dur = 3.0
    for x1, x2, d in ((228, 326, 0), (534, 632, 1.5)):
        s.add(f'<path d="M{x1} 128L{x2} 128" stroke="{AMBER}" stroke-width="2" marker-end="url(#arrow)"/>')
        s.add(f'<rect y="134" width="30" height="14" rx="2" fill="{AMBER}"><animate attributeName="x" values="{x1};{x2 - 30}" '
              f'dur="1.5s" begin="{d}s" repeatCount="indefinite"/></rect>')
        s.text((x1 + x2) / 2, 118, 'uploads code', 10, AMBER, 'middle')
    cpu_row(s, 24, 196, [('C74 / M37710', 'sound and I/O MCU', TEAL, 260), ('Namco C352', '32-channel PCM, 24.576 MHz', RED, 260)])
    s.text(24, 284, 'Capture the uploaded DSP code from a running machine. It is part of the game.', 12, DIM)
    return s.render()


@diagram('arcade-sega-models.svg')
def arcade_sega_models():
    s = Svg(900, 380, 'Sega Model 1 uses a NEC V60 with a TGP coprocessor, Model 2 an i960 with TGPs or SHARCs, and '
                      'Model 3 a PowerPC 603ev with Real3D graphics. Each has a 68000 for sound and its own '
                      'protection.')
    s.heading('Sega Model 1, 2 and 3', 'Each Model changes the main CPU and the geometry hardware. Each needs its own tools.')
    models = [('Model 1 (1992)', 'NEC V60, 16 MHz', 'Fujitsu MB86233 "TGP"', '68000 10 MHz, YM3438', 'Virtua Racing, Virtua Fighter',
               'none found for the V60', TEAL),
              ('Model 2 (1993/1994)', 'i960KB, 25 MHz', 'TGPs or SHARCs (by variant)', '68000, SCSP', 'Daytona USA',
               'i960 by extension', BLUE),
              ('Model 3 (1996)', 'PowerPC 603ev, 66-166 MHz', 'Real3D (Lockheed Martin)', '68000, two SCSP', 'Virtua Fighter 3, Scud Race',
               'PowerPC built in', PURPLE)]
    for k, (name, cpu, geo, snd, games, ghidra, colour) in enumerate(models):
        x = 24 + k * 286
        s.rect(x, 80, 272, 230, colour, rx=8, extra=' fill-opacity="0.13"')
        s.text(x + 14, 104, name, 14, TEXT, weight='600')
        s.text(x + 14, 122, games, 11, DIM)
        for r, (lab, val) in enumerate((('CPU', cpu), ('geometry', geo), ('sound', snd), ('Ghidra', ghidra))):
            s.text(x + 14, 152 + r * 38, lab, 10, DIM)
            s.text(x + 14, 168 + r * 38, val, 12, TEXT)
    lines(s, 24, 340, ['Model 1\'s TGP ROMs are marked BAD_DUMP on four games. Model 3 runs at 57.524 frames a second (Supermodel).',
                       'Protection: 315-5881 crypt device on Model 2A, 2B and Model 3.'], 12)
    return s.render()


@diagram('arcade-model2-variants.svg')
def arcade_model2_variants():
    s = Svg(900, 330, 'Every Sega Model 2 variant uses an i960KB. The geometry is done by MB86234 TGPs on Model 2 and '
                      '2A-CRX, two ADSP-21062 SHARCs on 2B-CRX and an MB86235 on 2C-CRX.')
    s.heading('Sega Model 2 variants', 'Every variant uses the i960KB main CPU with its own geometry processor.')
    rows = [('Model 2', 'MB86234 TGPs, 50 MHz', 'Model 1 sound board', TEAL),
            ('Model 2A-CRX', 'MB86234', '68000 at 45.1584 MHz / 4, SCSP at / 2', BLUE),
            ('Model 2B-CRX', 'ADSP-21062 SHARCs, 32 MHz', 'as 2A', PURPLE),
            ('Model 2C-CRX', 'MB86235, 20 MHz', 'as 2A', AMBER)]
    for k, (name, geo, snd, colour) in enumerate(rows):
        y = 86 + k * 50
        s.text(24, y + 26, name, 13, TEXT, weight='600')
        box(s, 170, y, 150, 40, BLUE, 'i960KB', op=0.25, size=12)
        box(s, 330, y, 250, 40, colour, geo, op=0.3, size=12)
        s.text(596, y + 25, snd, 11, DIM)
    s.text(24, 302, 'Video timing in MAME: 16 MHz pixel clock, 656 x 496 totals, 424 x 384 visible. The driver notes it may not be accurate.', 11, DIM)
    return s.render()


@diagram('arcade-3d-others.svg')
def arcade_3d_others():
    s = Svg(900, 320, 'Taito JC has a 68040 and a TMS320C51 DSP. Konami GTI Club and Hornet have a PowerPC 403GA and '
                      'up to two SHARCs. Gaelco 3D has a 68000 and an ADSP-2115 sound DSP.')
    s.heading('Taito JC, Konami Hornet, Gaelco 3D', 'A general CPU and a DSP on every board. Expect two or more instruction sets.')
    rows = [('Taito JC (1995)', [('M68040', '20 MHz', BLUE, 150), ('TMS320C51', '40 MHz DSP', PURPLE, 150)], 'Dangerous Curves, Landing Gear'),
            ('GTI Club, Hornet', [('PowerPC 403GA', '32 MHz', BLUE, 150), ('68000', '16 MHz, sound', TEAL, 150),
                                  ('up to 2 SHARCs', 'ADSP-21062, 36 MHz', PURPLE, 170)], 'Gradius IV; X76 secure EEPROM on some'),
            ('Gaelco 3D (1996)', [('M68000', '15 MHz', BLUE, 150), ('ADSP-2115', '16 MHz, sound', PURPLE, 150)], 'Speed Up; settings in a 93C66 EEPROM')]
    for k, (name, chips, note) in enumerate(rows):
        y = 84 + k * 76
        s.text(24, y + 28, name, 13, TEXT, weight='600')
        x = cpu_row(s, 180, y, chips, h=50, gap=8)
        s.text(180, y + 66, note, 11, DIM)
    return s.render()


# ================================================================ console-derived boards
@diagram('arcade-console-derived.svg')
def arcade_console_derived():
    s = Svg(900, 414, 'The PlayStation led to ZN-1, ZN-2, Namco System 11 and 12 and Konami System 573. The Saturn led '
                      'to ST-V, the Dreamcast to NAOMI, the GameCube to Triforce and the Xbox to Chihiro. The PS2 led '
                      'to Namco System 246 and Konami Python 2.')
    s.heading('Boards built from consoles', 'These boards use a console\'s CPU, GPU and sound chip. They add arcade media, a JVS I/O board and a security chip per game.')
    rows = [('PlayStation', 'MIPS R3000', ['ZN-1, ZN-2', 'Namco System 11, 12', 'Konami System 573'], 'ZN: 3 of 116 not working', GREEN),
            ('Saturn', '2 x SH-2', ['Sega ST-V'], '67 of 120 not working', AMBER),
            ('Dreamcast', 'SH-4', ['NAOMI, NAOMI 2', 'Atomiswave'], 'in MAME', GREEN),
            ('GameCube', 'PowerPC 750', ['Triforce'], 'all 73 not working', RED),
            ('Xbox', 'x86', ['Chihiro'], 'all 64 not working', RED),
            ('PS2', 'R5900', ['Namco System 246, 256', 'Konami Python 2'], 'System 246: all 60 not working', RED)]
    for k, (console, cpu, boards, mame, colour) in enumerate(rows):
        y = 82 + k * 50
        box(s, 24, y, 160, 42, GREY, console, cpu, op=0.3, size=12, sub_size=10)
        arrow(s, 188, y + 21, 214, y + 21)
        s.rect(218, y, 400, 42, BLUE, rx=6, extra=' fill-opacity="0.15"')
        s.text(230, y + 26, ', '.join(boards), 12, TEXT)
        s.rect(630, y + 8, 12, 26, colour, rx=2, extra=' fill-opacity="0.7"')
        s.text(650, y + 26, mame, 11, DIM)
    s.text(630, 74, 'MAME status', 11, DIM, weight='600')
    s.text(24, 402, 'A dump without the matching key or security-chip data is incomplete.', 12, DIM)
    return s.render()


@diagram('arcade-naomi-security.svg')
def arcade_naomi_security():
    s = Svg(900, 400, 'Each NAOMI cart has a 315-5881 with a key, an X76F100 secure flash and a CPLD. M4 carts decrypt '
                      'with an FPGA and a PIC that holds a 32-bit key. GD-ROM games load through the DES of the DIMM '
                      'board. Atomiswave uses an 8-bit key from the cart CPLD.')
    s.heading('NAOMI, Triforce and Chihiro security', 'Each kind of media has its own decryption. Record the key with the dump.')
    cols = [('ROM cart', ['315-5881, usually a key per game', '(e.g. Crazy Taxi 317-0248-COM)', 'X76F100 secure flash', '315-6213 XC9536 CPLD'], BLUE),
            ('M4 cart', ['Spartan XC3S50 FPGA', 'PIC16C621A: 32-bit key', 'stream cipher: 16-bit,', '2-round SP-network'], PURPLE),
            ('M1 cart', ['identified by an Actel ID', 'takes a key as a parameter'], TEAL),
            ('GD-ROM', ['DIMM board 315-6322:', 'SDRAM, DES decryption,', 'host communication', 'NAOMI, Triforce, Chihiro'], AMBER)]
    for k, (name, rows, colour) in enumerate(cols):
        x = 24 + k * 216
        s.rect(x, 80, 204, 160, colour, rx=8, extra=' fill-opacity="0.15"')
        s.text(x + 12, 104, name, 14, TEXT, weight='600')
        lines(s, x + 12, 130, rows, 11, TEXT, 19)
    s.text(24, 272, 'the key suffix', 12, TEXT, weight='600')
    box(s, 24, 282, 200, 44, RED, '-JPN', 'needs the Japanese BIOS', op=0.22)
    box(s, 240, 282, 200, 44, GREEN, '-COM', 'runs with any BIOS', op=0.22)
    lines(s, 470, 290, ['M4 feed value resets every 16 blocks of 32 bytes.', 'Atomiswave: 8-bit key from the cart CPLD.',
                        'MAME computes it from the decrypted code.'], 11)
    s.text(24, 382, 'Six known NAOMI cart PCB types, 171-7885A to 171-8346C. Each cart has the three security parts in the first column.', 11, DIM)
    return s.render()


# ================================================================ PC-based boards
@diagram('arcade-pc-boards.svg')
def arcade_pc_boards():
    s = Svg(900, 380, 'From 2004 arcade boards are PCs. Taito Type X to X4 run Windows, Sega Lindbergh runs Linux, and '
                      'Sega RingEdge, RingWide, Nu and ALLS run Windows Embedded. Namco ES1 and ES3 are also PCs. Each '
                      'board is locked with a key.')
    s.heading('PC-based boards', 'These boards are x86 PCs with custom I/O, a locked OS image and a hardware key. Ghidra handles the code once it is decrypted.')
    x0, x1 = 200, 860
    def xo(y):
        return x0 + (y - 2004) / (2024 - 2004) * (x1 - x0)
    rows = [('Taito Type X', [(2004, 'X'), (2008, 'X2 (SF IV)')], 'later: Zero, X3, X4', 'dongles', BLUE),
            ('Sega Lindbergh', [(2005, 'red / yellow box')], 'MontaVista Linux', 'PIC keychip, AES-CBC', AMBER),
            ('Sega Ring', [(2009, 'RingEdge'), (2011, 'RingWide')], 'Windows Emb. Standard 2009', 'Ring keychip, AES-128', PURPLE),
            ('Sega Nu, ALLS', [(2013, 'Nu'), (2018, 'ALLS')], 'Win Emb. 8; Win 10 IoT', 'keychips', TEAL),
            ('Namco ES1, ES3', [], 'no years on the page', 'HASP dongle, BitLocker', RED)]
    for k, (name, marks, os, key, colour) in enumerate(rows):
        y = 90 + k * 50
        s.text(24, y + 16, name, 12, TEXT, weight='600')
        s.text(24, y + 32, key, 10, DIM)
        s.line(x0, y + 12, x1, y + 12, GREY, 1, ' stroke-opacity="0.4"')
        if not marks or name == 'Taito Type X':
            s.text(x1, y + 30, os, 10, DIM, 'end')
        for yr, lab in marks:
            s.add(f'<circle cx="{xo(yr):.1f}" cy="{y + 12}" r="6" fill="{colour}"/>')
            s.text(xo(yr), y + 34, lab, 10, TEXT, 'middle')
    for yr in range(2004, 2025, 4):
        s.text(xo(yr), 352, str(yr), 10, DIM, 'middle', mono=True)
    s.text(860, 372, 'Only the years the page gives are marked.', 10, DIM, 'end')
    return s.render()


@diagram('arcade-lindbergh.svg')
def arcade_lindbergh():
    s = Svg(900, 360, 'On Sega Lindbergh the Compact Flash derives a 176-byte key by having the PIC keychip decrypt '
                      'sixteen blocks of 00 to 0b. It then decrypts the DVD and hard disk files with a custom cipher '
                      'and unlocks the disk.')
    s.heading('The Sega Lindbergh start-up chain', 'The CF and the hard disk are locked. The PIC keychip\'s AES-CBC engine starts the chain.')
    steps = [('PIC keychip', ['AES-CBC engine.', 'Its key cannot', 'be dumped.'], RED),
             ('CF card', ['has the PIC decrypt', '16 x 0x00 to 16 x 0x0b,', 'giving a 176-byte key'], AMBER),
             ('segaboot', ['decrypts DVD and', 'HDD files with a', 'custom cipher'], PURPLE),
             ('lxunlock.hdb', ['unlocks the disk'], BLUE),
             ('game', ['runs on MontaVista', 'Linux'], GREEN)]
    dur = 7.5
    for k, (name, rows, colour) in enumerate(steps):
        x = 24 + k * 174
        s.rect(x, 96, 158, 110, colour, rx=8, extra=' fill-opacity="0.18"')
        s.text(x + 12, 120, name, 13, TEXT, weight='600')
        lines(s, x + 12, 146, rows, 11, TEXT, 17)
        if k:
            arrow(s, x - 14, 151, x - 2, 151)
        s.add(f'<rect x="{x}" y="96" width="158" height="110" rx="8" fill="none" stroke="{AMBER}" stroke-width="2.5" opacity="0">'
              f'<animate attributeName="opacity" values="0;1;0;0" keyTimes="0;{k / 5:.2f};{(k + 1) / 5:.2f};1" calcMode="discrete" '
              f'dur="{dur}s" repeatCount="indefinite"/></rect>')
    lines(s, 24, 248, ['Red box: Celeron D 335 at 2.8 GHz, 512 MB. Yellow box: Pentium 4 at 3.00 GHz, 1 GB.',
                       'Both: Intel 875 northbridge, 6300ESB southbridge, a custom GeForce, VIA VT1616 audio.',
                       'Everything on the keychip except its key can be dumped. 18 of 19 Lindbergh entries do not work in MAME.'], 12)
    return s.render()


@diagram('arcade-ring-keychip.svg')
def arcade_ring_keychip():
    s = Svg(900, 320, 'The Sega Ring keychip holds a PIC, a cryptography chip, a SHA engine and flash. All bus traffic '
                      'is AES-128 ECB with a different key in each direction. The keychip returns a four-character '
                      'game ID, decrypts with the game key and keeps the play counts.')
    s.heading('Sega Ring keychip', 'Every message on the bus is encrypted, with a different key in each direction.')
    box(s, 24, 110, 220, 80, BLUE, 'RingEdge / RingWide', 'Windows Embedded 2009', op=0.25)
    s.rect(560, 90, 316, 140, AMBER, rx=8, extra=' fill-opacity="0.12"')
    s.text(576, 114, 'keychip', 13, TEXT, weight='600')
    lines(s, 576, 140, ['PIC microcontroller', 'cryptography chip', 'hardware SHA engine', 'flash storage'], 12, TEXT, 20)
    s.line(250, 136, 554, 136, BLUE, 2)
    s.line(250, 168, 554, 168, AMBER, 2)
    s.text(400, 128, 'AES-128 ECB, key A', 11, BLUE, 'middle')
    s.text(400, 186, 'AES-128 ECB, key B', 11, AMBER, 'middle')
    for k in range(3):
        s.add(f'<rect y="131" width="22" height="10" rx="2" fill="{BLUE}"><animate attributeName="x" values="250;532" dur="2s" '
              f'begin="{k * 0.66:.2f}s" repeatCount="indefinite"/></rect>')
        s.add(f'<rect y="163" width="22" height="10" rx="2" fill="{AMBER}"><animate attributeName="x" values="532;250" dur="2s" '
              f'begin="{k * 0.66:.2f}s" repeatCount="indefinite"/></rect>')
    lines(s, 24, 262, ['The keychip returns a four-character game ID, decrypts with the game key and keeps billing play counts.',
                       'Get decrypted, unlocked files from hardware you own. Then use the Windows PE path.'], 12)
    return s.render()


# ================================================================ practices
@diagram('arcade-machine-flags.svg')
def arcade_machine_flags():
    s = Svg(900, 400, 'The NOT_WORKING, UNEMULATED_PROTECTION and IMPERFECT flags say whether MAME can be ground truth '
                      'for a game. SUPPORTS_SAVE, MECHANICAL and IS_BIOS_ROOT describe the set.')
    s.heading('MAME machine flags', 'Check the flags before you use MAME as ground truth for a game.')
    flags = [('MACHINE_NOT_WORKING', 'imperfect emulation prevents using the system as intended', RED),
             ('MACHINE_UNEMULATED_PROTECTION', 'protection is not emulated', RED),
             ('MACHINE_IMPERFECT_GRAPHICS', 'graphics are imperfect', AMBER),
             ('MACHINE_IMPERFECT_SOUND', 'sound is imperfect', AMBER),
             ('MACHINE_IMPERFECT_TIMING', 'timing is imperfect', AMBER),
             ('MACHINE_NODEVICE_LAN', 'unemulated local area networking', AMBER),
             ('MACHINE_SUPPORTS_SAVE', 'save states work', GREEN),
             ('MACHINE_MECHANICAL', 'a mechanical game', GREY),
             ('MACHINE_IS_BIOS_ROOT', 'a BIOS set', GREY)]
    for k, (flag, what, colour) in enumerate(flags):
        y = 80 + k * 32
        s.rect(24, y, 320, 26, colour, rx=3, extra=' fill-opacity="0.3"')
        s.text(34, y + 18, flag, 11, TEXT, mono=True)
        s.text(360, y + 18, what, 12, TEXT)
    s.rect(24, 376, 12, 12, RED, rx=2, extra=' fill-opacity="0.6"')
    s.text(42, 386, 'not ground truth', 11, DIM)
    s.rect(170, 376, 12, 12, AMBER, rx=2, extra=' fill-opacity="0.6"')
    s.text(188, 386, 'check the part you need', 11, DIM)
    s.text(860, 386, 'A ROM marked BAD_DUMP is known to be faulty.', 11, DIM, 'end')
    return s.render()


@diagram('arcade-chd.svg')
def arcade_chd():
    s = Svg(900, 320, 'CHD stores disks and optical media as compressed hunks of 16 bytes to 1 MiB. The chdman tool '
                      'creates, extracts, checks and verifies them with the zlib, zstd, lzma, huff and flac codecs.')
    s.heading('ROM sets and CHD', 'A ROM set lists every chip dump. Disks and discs go in a CHD (Compressed Hunks of Data).')
    s.text(24, 96, 'a CHD file', 12, TEXT, weight='600')
    box(s, 24, 106, 120, 44, AMBER, 'header', 'metadata', op=0.3)
    for k in range(8):
        codec = ['zlib', 'zstd', 'lzma', 'flac', 'zlib', 'huff', 'zstd', 'lzma'][k]
        box(s, 152 + k * 88, 106, 82, 44, TEAL, f'hunk {k}', codec, op=0.22, size=11, sub_size=10)
    s.text(24, 172, 'Each hunk is compressed on its own with the best codec. Hunks are 16 bytes to 1 MiB.', 11, DIM)
    cmds = [('create', 'createhd, createcd, createdvd, createld, createraw', BLUE),
            ('extract', 'the matching extract commands', PURPLE), ('check', 'info, verify', GREEN)]
    for k, (name, what, colour) in enumerate(cmds):
        y = 200 + k * 30
        s.rect(24, y, 90, 24, colour, rx=3, extra=' fill-opacity="0.35"')
        s.text(69, y + 16, name, 11, TEXT, 'middle', '600')
        s.text(126, y + 16, what, 12, TEXT, mono=True)
    lines(s, 560, 214, ['CD codecs: cdzl, cdzs, cdlz, cdfl.', 'LaserDisc: avhu.', 'To read a CPS-3 CD or a GD-ROM,',
                        'extract it from the CHD first.'], 12)
    return s.render()


@diagram('arcade-test-inputs.svg')
def arcade_test_inputs():
    s = Svg(900, 356, 'An arcade test case records every input the game reads. These are the ROM set, the BIOS and '
                      'region, the DIP switches, the NVRAM or EEPROM image and the recorded inputs. The same inputs on '
                      'the original and the port should give the same frames.')
    s.heading('The inputs of an arcade test case', 'The game reads the board\'s settings too. Record them all with each test.')
    parts = [('ROM set', 'with its BIOS set', BLUE), ('BIOS, region,', 'language', PURPLE), ('DIP switches', '', AMBER),
             ('NVRAM / EEPROM', 'CMOS, 93C46', RED), ('input recording', 'from power-on', TEAL)]
    for k, (a, b, colour) in enumerate(parts):
        box(s, 24, 80 + k * 46, 200, 38, colour, a, b or None, op=0.25, size=12, sub_size=10)
        s.add(f'<path d="M228 {99 + k * 46}C 290 {99 + k * 46} 290 190 340 190" fill="none" stroke="{DIM}" stroke-width="1.2"/>')
    box(s, 344, 160, 160, 60, GREY, 'test case', op=0.3)
    arrow(s, 508, 176, 586, 130)
    arrow(s, 508, 204, 586, 250)
    box(s, 590, 100, 140, 54, GREEN, 'original', 'in MAME', op=0.25)
    box(s, 590, 226, 140, 54, BLUE, 'the port', op=0.25)
    s.text(800, 196, 'same frames?', 12, AMBER, 'middle', '600')
    arrow(s, 734, 127, 790, 180)
    arrow(s, 734, 253, 790, 208)
    s.text(24, 340, 'Fatal Fury 3 crashes in its ending in MAME unless the language is Japanese. MAME\'s default NVRAM can differ from a factory board.', 11, DIM)
    return s.render()


@diagram('arcade-attract-mode.svg')
def arcade_attract_mode():
    s = Svg(900, 266, 'Attract mode depends on the random state and on NVRAM, so start the original and the port from '
                      'the same NVRAM image and power-on state. Released source keeps attract mode, audits, '
                      'adjustments and diagnostics as separate modules.')
    s.heading('Attract mode and the other modules', 'Released source keeps attract mode, audits, adjustments and diagnostics apart. Rebuild all of them.')
    mods = [('attract', 'ATTRACT.ASM', TEAL), ('audits', 'AUDIT.ASM', BLUE), ('adjustments', 'ADJUST.ASM', PURPLE),
            ('diagnostics', 'DIAG.ASM', AMBER)]
    for k, (name, f, colour) in enumerate(mods):
        box(s, 24 + k * 214, 90, 200, 56, colour, name, f, op=0.25, mono=True)
    s.text(24, 176, 'These file names are from NBA Jam\'s source. Smash TV\'s has DEMO.ASM and HSTD.ASM for the high-score table.', 11, DIM)
    lines(s, 24, 216, ['A demo plays from recorded inputs or scripted AI. What it shows depends on the random state and on NVRAM.',
                       'To compare a port\'s attract mode with the original, start both from the same NVRAM image and power-on state.'], 12, TEXT)
    return s.render()


@diagram('arcade-input-recording.svg')
def arcade_input_recording():
    s = Svg(900, 320, 'MAME\'s -record and -playback can go out of step if the .cfg, NVRAM and memory card files '
                      'differ from the original run, so record and play back with them deleted. Other options capture '
                      'video, audio and debugger output.')
    s.heading('Recording inputs in MAME', '-record and -playback "do not work reliably for all systems". Start both runs from the same state.')
    box(s, 24, 96, 220, 60, TEAL, '-record game.inp', 'delete .cfg, .nv, memory cards', op=0.25, mono=False)
    box(s, 340, 96, 220, 60, BLUE, '-playback game.inp', 'the same files deleted', op=0.25)
    arrow(s, 248, 126, 336, 126)
    box(s, 656, 96, 220, 60, GREEN, 'the same frames', op=0.25)
    arrow(s, 564, 126, 652, 126)
    s.add(fade(f'<rect x="656" y="96" width="220" height="60" rx="6" fill="#6b3434"/>'
               f'<text x="766" y="131" font-family="{SANS}" font-size="13" font-weight="600" fill="{TEXT}" text-anchor="middle">'
               f'out of step if a file differed</text>', [(0.5, 1.0)], 6))
    opts = [('-exit_after_playback', 'quit at the end'), ('-mngwrite, -aviwrite, -wavwrite', 'capture video and audio'),
            ('-debugger, -debugscript', 'run debugger commands from a file'), ('-autoboot_script', 'run a Lua script at start')]
    for k, (o, w) in enumerate(opts):
        y = 196 + k * 26
        s.text(24, y, o, 12, AMBER, mono=True)
        s.text(330, y, w, 12, TEXT)
    return s.render()


@diagram('arcade-refresh-rates.svg')
def arcade_refresh_rates():
    s = Svg(900, 440, 'Arcade refresh rates run from about 54.71 Hz on Midway TMS34010 boards to about 61.5 Hz on '
                      'Asteroids. Boards with the same crystal and totals share a rate.')
    s.heading('Arcade refresh rates', 'Boards with the same crystal and the same totals have the same rate. Few boards run at exactly 60 Hz.')
    rows = [('Midway TMS34010', 54.71, 'Y, T, W-units'), ('Irem M72', 55.02, '32 MHz, 512 x 284'),
            ('Sega System 18', 57.23, 'measured'), ('Data East DEC0', 57.44, '384 x 272'), ('Cave 68000', 57.55, 'set'),
            ('Taito F3', 58.94, '26.686 MHz / 4'), ('6 MHz pixel clock', 59.19, 'Neo Geo, TMNT, Bubble Bobble'),
            ('8 MHz pixel clock', 59.64, 'CPS-1, CPS-2, X Board'), ('Atari 456 x 262', 59.92, 'System 1, GT'),
            ('18.432 MHz / 3', 60.61, 'Namco, Donkey Kong'), ('Vector', 61.5, 'Asteroids')]
    x0, scale = 200, 52.5
    def xv(v):
        return x0 + (v - 54) * scale
    s.line(xv(60), 74, xv(60), 410, AMBER, 1.5, ' stroke-dasharray="5 4"')
    for k, (name, hz, note) in enumerate(rows):
        y = 80 + k * 30
        s.text(x0 - 10, y + 16, name, 11, TEXT, 'end')
        colour = GREEN if 'measured' in note else TEAL
        s.rect(x0, y + 3, xv(hz) - x0, 20, colour, rx=3, extra=' fill-opacity="0.45"')
        if hz < 55.5:
            s.text(xv(hz) + 6, y + 17, f'{hz:g}', 11, TEXT, mono=True)
        else:
            s.text(xv(hz) - 6, y + 17, f'{hz:g}', 11, TEXT, 'end', mono=True)
        s.text(650, y + 17, note, 11, DIM)
    s.text(xv(60), 426, '60 Hz', 11, AMBER, 'middle')
    for v in (54, 56, 58, 62):
        s.text(xv(v), 426, f'{v}', 10, DIM, 'middle', mono=True)
    return s.render()


@diagram('arcade-frame-drift.svg')
def arcade_frame_drift():
    s = Svg(900, 300, 'A port at a fixed 60 Hz runs ahead of a 54.71 Hz Midway board by about 317 frames a minute and '
                      'of a 59.64 Hz CPS board by about 22 frames a minute. A frame-by-frame comparison then goes out '
                      'of step.')
    s.heading('Frame drift at a fixed 60 Hz', 'The same second of play has a different number of frames. Run the port at the board\'s rate.')
    dur = 6.0
    rows = [('port at 60 Hz', 60, AMBER), ('CPS board, 59.64 Hz', 59.64, BLUE), ('Midway board, 54.71 Hz', 54.71, PURPLE)]
    for k, (name, hz, colour) in enumerate(rows):
        y = 90 + k * 46
        s.text(24, y + 20, name, 12, TEXT, weight='600')
        n = 20
        step = 600 / n * 60 / hz
        for i in range(n):
            x = 220 + i * step
            if x > 830:
                break
            s.rect(x, y + 4, step - 3, 24, colour, rx=2, extra=' fill-opacity="0.4"')
    s.add(f'<rect x="220" y="84" width="2" height="140" fill="{TEXT}"><animate attributeName="x" values="220;820" dur="{dur}s" '
          f'repeatCount="indefinite"/></rect>')
    lines(s, 24, 246, ['At 60 Hz a port shows about 317 more frames a minute than a Midway board and about 22 more than a CPS board.',
                       'On a Midway board a frame-by-frame comparison goes out of step within seconds.'], 12)
    return s.render()


@diagram('arcade-cpu-support.svg')
def arcade_cpu_support():
    s = Svg(900, 500, 'Ghidra has most of the classic arcade CPUs built in and the TMS34010, SHARC and i960 by '
                      'extension. It has nothing for the V60 or the DSPs. Unicorn has no 8-bit CPUs, no SH-2 and no '
                      'TMS34010.')
    s.heading('Tool support per arcade CPU', 'Where Ghidra or Unicorn has nothing, MAME\'s debugger or a small core gives ground truth.')
    Y, E, N, P = 'built in', 'extension', 'none', 'decrypt first'
    rows = [('8080', Y, N), ('Z80', Y, N), ('6502, 65C02', Y, N), ('6809, HD6309', Y, N), ('Konami-1', P, N), ('68000 to 68040', Y, Y),
            ('NEC V30, V33, V35', 'x86 16-bit', 'x86 16-bit'), ('TMS34010', E, N), ('TMS320, ADSP-21xx, DSP32C', N, N),
            ('ADSP-21062 SHARC', E, N), ('NEC V60', N, N), ('i960', E, N), ('SH-2, SH-4', Y, N), ('PowerPC', Y, Y),
            ('MIPS R3000, R5900', 'built in; R5900 ext.', 'MIPS, no R5900'), ('x86', Y, Y), ('8051, PIC', Y, N)]
    colour = {Y: GREEN, E: BLUE, N: GREY, P: AMBER}
    s.text(370, 86, 'Ghidra', 12, TEXT, 'middle', '600')
    s.text(600, 86, 'Unicorn', 12, TEXT, 'middle', '600')
    for k, (cpu, g, u) in enumerate(rows):
        y = 94 + k * 22
        s.text(24, y + 15, cpu, 11, TEXT)
        for x, v in ((290, g), (520, u)):
            c = colour.get(v, AMBER if v != Y else GREEN)
            if v == Y and x == 520:
                v = 'yes'
            s.rect(x, y, 160, 19, c, rx=3, extra=f' fill-opacity="{0.15 if c == GREY else 0.4}"')
            s.text(x + 80, y + 14, v, 10, TEXT if c != GREY else DIM, 'middle')
    lines(s, 700, 110, ['The V30 entries', 'are not confirmed.', '', 'Other tools:', 'MAME for all;', 'Supermodel, Dolphin,',
                        'Flycast, Play!,', 'lindbergh-loader,', 'TeknoParrot.'], 11)
    return s.render()


@diagram('arcade-source-vs-rom.svg')
def arcade_source_vs_rom():
    s = Svg(900, 260, 'Released source shows what the programmers meant, but it may not match the final revision. The '
                      'shipped ROM decides the exact behaviour. A project never copies the source into its repository.')
    s.heading('Released source and the shipped ROM', 'Exact behaviour comes from the shipped ROM, even where the source was released.')
    box(s, 24, 96, 250, 70, BLUE, 'released source', 'assembly, what was meant', op=0.25)
    box(s, 324, 96, 250, 70, AMBER, 'later fixes', 'may not be in the source', op=0.2)
    box(s, 624, 96, 250, 70, GREEN, 'shipped ROM', 'the exact behaviour', op=0.3)
    arrow(s, 278, 131, 320, 131)
    arrow(s, 578, 131, 620, 131)
    lines(s, 24, 206, ['The source is third-party evidence. GameFrame\'s rule against game material means a project does not copy it',
                       'into its own repository.'], 12)
    return s.render()


# ================================================================ protection
@diagram('arcade-suicide.svg')
def arcade_suicide():
    s = Svg(900, 400, 'Many arcade CPUs keep their decryption key in battery-backed RAM. When the battery dies the key '
                      'is lost, and each scheme fails in its own way. Preservation groups wrote keys back to revive '
                      'boards.')
    s.heading('When the battery dies', 'The key lives in battery-backed RAM. Each scheme fails in its own way when the battery runs out.')
    dur = 6.0
    s.rect(24, 92, 70, 34, 'none', TEXT, rx=4, extra=' stroke-width="2"')
    s.rect(94, 102, 6, 14, TEXT, rx=1)
    s.add(f'<rect x="28" y="96" width="62" height="26" rx="2" fill="{GREEN}"><animate attributeName="width" values="62;0;0;62" '
          f'keyTimes="0;0.5;0.95;1" dur="{dur}s" repeatCount="indefinite"/><animate attributeName="fill" values="{GREEN};{RED}" '
          f'keyTimes="0;0.5" dur="{dur}s" calcMode="discrete" repeatCount="indefinite"/></rect>')
    s.add(fade(f'<text x="120" y="114" font-family="{SANS}" font-size="13" fill="{RED}" font-weight="600">key lost</text>', [(0.5, 0.95)], dur))
    rows = [('FD1094', 'Sega, custom 68000', 'runs with a wrong key'), ('FD1089A, B', 'Sega, custom 68000', 'stops'),
            ('Kabuki', 'Capcom, custom Z80', 'pin 28 low gives a plain Z80'), ('CPS-2', 'Capcom, 68000 board', 'blue or green screen'),
            ('CPS-3', 'Capcom, SH-2 derivative', 'thought to fall back to SFIII 2nd Impact keys'), ('Konami-1', 'Konami, custom 6809', 'no battery; the key is fixed by address')]
    for k, (name, what, result) in enumerate(rows):
        y = 148 + k * 34
        s.text(24, y + 18, name, 12, TEXT, weight='600')
        s.text(130, y + 18, what, 11, DIM)
        s.rect(330, y, 540, 28, GREY, rx=3, extra=' fill-opacity="0.15"')
        s.add(fade(f'<rect x="330" y="{y}" width="540" height="28" rx="3" fill="{RED}" fill-opacity="0.25"/>'
                   f'<text x="342" y="{y + 18}" font-family="{SANS}" font-size="12" fill="{TEXT}">{result}</text>',
                   [(0.5, 0.95)], dur))
    s.text(24, 380, 'Preservation groups extracted keys from working boards, then wrote keys back into dead boards so they run again.', 12, DIM)
    return s.render()


@diagram('arcade-fd1094.svg')
def arcade_fd1094():
    s = Svg(900, 330, 'The Sega FD1094 decrypts only words fetched from program space. A chain of conditional XORs and '
                      'bit swaps is controlled by 32 variables. Eight of them vary with the address and repeat every '
                      '0x2000 words. The program can switch between 256 states.')
    s.heading('Sega FD1094 and FD1089', 'The FD1094 decrypts opcode fetches only. Its result depends on the address and on the current state.')
    box(s, 24, 96, 170, 56, GREY, 'encrypted word', 'fetched as an opcode', op=0.3)
    box(s, 260, 86, 240, 76, RED, 'conditional XORs, bit swaps', '32 variables', op=0.25)
    box(s, 566, 96, 170, 56, GREEN, 'plain opcode', op=0.3)
    arrow(s, 198, 124, 256, 124)
    arrow(s, 504, 124, 562, 124)
    box(s, 160, 200, 200, 52, BLUE, '8 vary by address', 'repeat every 0x2000 words', op=0.25, sub_size=10)
    box(s, 400, 200, 200, 52, PURPLE, '256 states', 'switched by the program', op=0.25)
    arrow(s, 260, 196, 320, 166)
    arrow(s, 500, 196, 440, 166)
    lines(s, 24, 300, ['Data reads are not decrypted. A final step turns about 300 or 5000 values into FFFF. The FD1089 encrypts opcodes',
                       'and data. It changes 8 of 16 bits, with 12 address bits selecting the key. Take FD1094 ground truth from a running emulator.'], 11)
    return s.render()


@diagram('arcade-cps2-cipher.svg')
def arcade_cps2_cipher():
    s = Svg(900, 360, 'The CPS-2 cipher uses two 4-round Feistel networks with a 64-bit key. The first runs on the low '
                      '16 address bits. Its output, expanded and mixed with the key, keys the second network, which '
                      'decrypts the 16-bit opcode.')
    s.heading('The CPS-2 opcode cipher', 'D = FN2(E, K xor EX(FN1(A, K))). The cipher encrypts opcodes only.')
    box(s, 24, 96, 170, 56, BLUE, 'A', 'low 16 address bits', op=0.25)
    box(s, 250, 96, 170, 56, PURPLE, 'FN1', '4-round Feistel', op=0.3)
    box(s, 476, 96, 150, 56, PURPLE, 'EX', 'expand', op=0.25)
    box(s, 682, 96, 190, 56, AMBER, 'K xor ...', '64-bit key', op=0.3)
    box(s, 24, 220, 170, 56, GREY, 'E', 'encrypted 16-bit opcode', op=0.3)
    box(s, 476, 220, 170, 56, PURPLE, 'FN2', '4-round Feistel', op=0.3)
    box(s, 702, 220, 170, 56, GREEN, 'D', 'plain opcode', op=0.3)
    arrow(s, 198, 124, 246, 124)
    arrow(s, 424, 124, 472, 124)
    arrow(s, 630, 124, 678, 124)
    s.add(f'<path d="M776 156C 776 190 600 190 570 216" fill="none" stroke="{DIM}" stroke-width="1.5" marker-end="url(#arrow)"/>')
    s.text(700, 192, 'round keys', 10, DIM)
    arrow(s, 198, 248, 472, 248)
    arrow(s, 650, 248, 698, 248)
    s.text(335, 82, 'K', 11, AMBER, 'middle', '600')
    lines(s, 24, 312, ['The key, the watchdog opcodes and the address range are in battery-backed RAM.',
                       'Andreas Naive broke the algorithm. Nicola Salmoria wrote the code. Charles MacDonald and Razoola extracted the data.'], 12)
    return s.render()


@diagram('arcade-konami1.svg')
def arcade_konami1():
    s = Svg(900, 300, 'Konami-1 is a 6809 that XORs each opcode byte with 0x22, 0x82, 0x28 or 0x88, chosen by address '
                      'bits 1 and 3. Data bytes are left alone.')
    s.heading('Konami-1 opcode scrambling by address', 'The XOR value depends on address bits 1 and 3 (pc & 0xA). Data bytes are left alone.')
    vals = [(0, 0, '0x22'), (0, 1, '0x82'), (1, 0, '0x28'), (1, 1, '0x88')]
    s.text(24, 92, 'bit 3', 11, DIM, weight='600')
    s.text(90, 92, 'bit 1', 11, DIM, weight='600')
    s.text(170, 92, 'opcode XOR', 11, DIM, weight='600')
    for k, (b3, b1, v) in enumerate(vals):
        y = 102 + k * 34
        s.text(36, y + 18, str(b3), 13, TEXT, 'middle', mono=True)
        s.text(102, y + 18, str(b1), 13, TEXT, 'middle', mono=True)
        s.rect(160, y, 120, 26, AMBER, rx=3, extra=' fill-opacity="0.3"')
        s.text(220, y + 18, v, 13, TEXT, 'middle', '600', mono=True)
    s.text(24, 266, 'From MAME\'s konami1.cpp.', 11, DIM)
    s.text(380, 100, 'bytes in ROM, by address (an illustration)', 12, TEXT, weight='600')
    for k in range(16):
        x = 380 + (k % 8) * 60
        y = 112 + (k // 8) * 44
        op = k in (0, 3, 7, 8, 10, 13)
        s.rect(x, y, 54, 36, AMBER if op else TEAL, rx=3, extra=f' fill-opacity="{0.35 if op else 0.2}"')
        s.text(x + 27, y + 15, f'{0x4000 + k:04X}', 9, DIM, 'middle', mono=True)
        s.text(x + 27, y + 29, 'opcode' if op else 'data', 10, TEXT, 'middle')
    lines(s, 380, 232, ['Only opcode fetches are XORed. A disassembler must', 'know which bytes are opcodes. MAME has a decrypting', 'disassembler for it.'], 11)
    return s.render()


@diagram('arcade-protection-mcus.svg')
def arcade_protection_mcus():
    s = Svg(900, 330, 'Taito, Data East, Irem, Midway and Sega used 68705, 8751, 80C31 and PIC chips for protection. '
                      'These MCUs often hold game logic such as enemy behaviour or tables. If the MCU ROM was never '
                      'dumped, MAME may use a simulation.')
    s.heading('Protection MCUs', 'A protection MCU often holds part of the game, such as enemy behaviour or lookup tables.')
    rows = [('Taito', 'Bubble Bobble', 'M68705P3 or M6801U4'), ('Data East', 'DEC0', 'Intel 8751, M68705R3'),
            ('Irem', 'M72', 'Intel 8751 or 80C31'), ('Midway', 'W-unit', 'PIC16C57'), ('Sega', 'System 16B', 'Intel 8751')]
    for k, (maker, board, mcu) in enumerate(rows):
        y = 84 + k * 34
        s.text(24, y + 18, maker, 12, TEXT, weight='600')
        s.text(130, y + 18, board, 12, DIM)
        s.rect(270, y, 230, 26, RED, rx=3, extra=' fill-opacity="0.25"')
        s.text(282, y + 18, mcu, 12, TEXT, mono=True)
    box(s, 560, 90, 300, 56, GREEN, 'MCU ROM dumped', 'MAME runs the real program', op=0.25)
    box(s, 560, 166, 300, 56, AMBER, 'MCU ROM never dumped', 'MAME may use a simulation', op=0.25)
    lines(s, 24, 280, ['Check which one MAME uses before you trust its output. Ghidra has 8051 and PIC modules built in.'], 12)
    return s.render()
