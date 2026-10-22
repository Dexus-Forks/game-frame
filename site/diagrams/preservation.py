"""Diagrams for the preservation guides (docs/preservation/, not the law pages): how many games are
in print, changes that stopped games running, a game on sale that nobody updates, where the
source and media go, GameFrame's method, who keeps which part of a game, what a project ships,
the checks on an agent's work, and replacing a program one function at a time.
site/diagrams/make.py writes them.
"""
from engines import arrow, box, during, group, legend, lines, scratch
from svg import AMBER, BG, BLUE, DIM, EDGE, GREEN, GREY, MONO, PURPLE, RED, TEAL, TEXT, Svg


# ---------------------------------------------------------------- out of print
def in_print():
    s = Svg(900, 330, 'Games still in print, from the 2023 study of 1,500 US games released before 2010. 13% of the sample is in print, with 5.87% of the Game Boy library, 4.5% of Commodore 64 games and fewer than 3% of games from before 1985')
    s.heading('How many old games can still be bought',
              'From the study by the Video Game History Foundation and the Software Preservation Network, July 2023.')
    rows = [('All 1,500 games sampled', 13.0, '13%', TEAL), ('Game Boy library', 5.87, '5.87%', BLUE),
            ('Commodore 64 games', 4.5, '4.5%', BLUE), ('Games from before 1985', 3.0, 'fewer than 3%', AMBER)]
    x0, scale = 250, 6.0
    for k, (name, pct, label, colour) in enumerate(rows):
        y = 92 + k * 46
        s.text(24, y + 22, name, 12, TEXT)
        s.rect(x0, y + 6, 100 * scale, 24, GREY, rx=3, extra=' fill-opacity="0.15"')
        s.add(f'<rect x="{x0}" y="{y + 6}" height="24" rx="3" fill="{colour}" fill-opacity="0.7" width="{pct * scale:.1f}">'
              f'<animate attributeName="width" values="0;{pct * scale:.1f}" dur="1.2s"/></rect>')
        s.text(x0 + pct * scale + 8, y + 23, label, 12, TEXT, weight='600')
    s.text(x0, 288, 'Each bar is out of 100%. The grey part is the games that cannot be bought today.', 11, DIM)
    s.text(24, 314, 'About 1,000 more games left sale when Nintendo closed the 3DS and Wii U eShops in March 2023.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- games that stop running
def breaks():
    s = Svg(900, 330, 'Changes that stopped bought games from running. 64-bit Windows runs no 16-bit programs. Windows 10 has no SafeDisc drivers. macOS Catalina runs no 32-bit apps. Windows 11 has no 32-bit edition. The Crew\'s servers closed. Rosetta 2 is being retired')
    s.heading('System changes that stopped old games running',
              'Each of these changes to Windows, macOS or an online service stopped some bought games from running.')
    events = [(2005, '64-bit Windows', 'no 16-bit programs', BLUE, 0), (2015, 'Windows 10', 'no SafeDisc drivers', BLUE, 1),
              (2019, 'macOS Catalina', 'no 32-bit apps', PURPLE, 0), (2021, 'Windows 11', 'no 32-bit edition', BLUE, 1),
              (2024, 'The Crew', 'servers closed 31 March', RED, 0), (2027, 'macOS 28', 'Rosetta 2 only in part', PURPLE, 1)]
    lo, hi, x0, x1 = 2003, 2029, 60, 860
    X = lambda yr: x0 + (yr - lo) / (hi - lo) * (x1 - x0)
    s.line(x0, 200, x1, 200, DIM, 1.5)
    for yr in range(2004, 2029, 4):
        s.line(X(yr), 196, X(yr), 204, DIM, 1)
        s.text(X(yr), 222, str(yr), 10.5, DIM, 'middle')
    for yr, name, what, colour, up in events:
        x = X(yr)
        y = 96 if up == 0 else 236
        s.line(x, 200, x, y + (44 if up == 0 else 0), colour, 1.5)
        s.add(f'<circle cx="{x:.1f}" cy="200" r="5" fill="{colour}"/>')
        box(s, x - 72, y, 144, 44, colour, name, what, op=0.25, size=11.5)
    s.text(24, 314, 'Most DOS and early Windows games are 16-bit. Games written for 3dfx Glide need a wrapper to run at all.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- on sale, not updated
def not_updated():
    s = Svg(900, 266, 'Banished was released on 18 February 2014 by a studio of one person. Its last build, 1.0.7, is dated 14 September 2017. It is still on sale on Steam and GOG with no updates since')
    s.heading('A game on sale that nobody updates',
              'Banished is one example. If a new system breaks it, there may be no update to fix it.')
    lo, hi, x0, x1 = 2013.5, 2027, 60, 860
    X = lambda yr: x0 + (yr - lo) / (hi - lo) * (x1 - x0)
    s.line(x0, 196, x1, 196, DIM, 1.5)
    for yr in range(2014, 2027, 2):
        s.line(X(yr), 192, X(yr), 200, DIM, 1)
        s.text(X(yr), 214, str(yr), 10.5, DIM, 'middle')
    a, b = X(2014 + 49 / 365), X(2017 + 256 / 365)
    s.rect(a, 156, b - a, 28, TEAL, rx=3, extra=' fill-opacity="0.45"')
    s.text((a + b) / 2, 175, 'updates', 11, TEXT, 'middle', '600')
    s.rect(b, 156, X(2026.8) - b, 28, GREY, rx=3, extra=' fill-opacity="0.25"')
    s.text((b + X(2026.8)) / 2, 175, 'on sale on Steam and GOG with no updates', 11, DIM, 'middle')
    for x, label, sub in ((a, '18 February 2014', 'released'), (b, '14 September 2017', 'the last build, 1.0.7')):
        s.line(x, 110, x, 156, AMBER, 1.5)
        s.text(x, 98, label, 11.5, AMBER, 'middle', '600')
        s.text(x, 84 + 0, sub, 10.5, DIM, 'middle')
    s.text(24, 244, 'Nobody has counted how many games are in this position. Many of them come from small studios.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- lost source and media
def lost():
    s = Svg(900, 264, 'Where a game can be lost. The studio\'s source code and data, the discs that decay and the rights that nobody can establish are shown beside the copies players bought, which are often all that is left')
    s.heading('Lost source code and media',
              'When a studio loses its source code, the copies that players bought are often all that is left.')
    items = [('The studio\'s source', 'The final code of Silent Hill 2', 'and 3 is gone.', RED),
             ('The discs', 'Recordable discs last from a few', 'years to a century.', AMBER),
             ('The rights', 'No One Lives Forever has no clear', 'owner and was not sold in 2025.', AMBER),
             ('Players\' copies', 'They are often all that is left.', 'GameFrame works from them.', TEAL)]
    for k, (title, a, b, colour) in enumerate(items):
        x = 24 + k * 216
        s.rect(x, 90, 200, 110, colour, rx=6, extra=' fill-opacity="0.16"')
        s.text(x + 14, 116, title, 13, TEXT, weight='600')
        s.text(x + 14, 144, a, 11, DIM)
        s.text(x + 14, 162, b, 11, DIM)
    s.text(24, 240, 'Disc lifetimes depend on the dye and the metal layer (Canadian Conservation Institute, CCI Notes 19/1).',
           11, DIM)
    return s.render()


# ---------------------------------------------------------------- what GameFrame does: the method (animated)
def method():
    s = Svg(900, 330, 'GameFrame\'s method as a loop. Read the owner\'s copy and cite each address, run the original\'s functions under emulation for exact answers, write the port, test that it gives the same answers to the last bit, and write documents along the way')
    s.heading('Documents of how the game works',
              'Each finding cites the place in the original that it comes from. The port must give the same results to the last bit.')
    dur = 8.0
    steps = [('the owner\'s copy', 'the only source', GREY, 40, 96), ('read and cite', 'each claim names its address', BLUE, 330, 96),
             ('emulate', 'exact answers for chosen inputs', PURPLE, 620, 96), ('port', 'new code', AMBER, 620, 214),
             ('test', 'same answers, to the last bit', TEAL, 330, 214), ('documents', 'usable without the port', GREEN, 40, 214)]
    for k, (title, sub, colour, x, y) in enumerate(steps):
        box(s, x, y, 240, 56, colour, title, sub, op=0.22)
        s.add(f'<rect x="{x}" y="{y}" width="240" height="56" rx="4" fill="none" stroke="{AMBER}" stroke-width="2.5" '
              f'opacity="{1 if k == 0 else 0}">{during(k / 6, (k + 1) / 6, dur)}</rect>')
    arrow(s, 282, 124, 326, 124)
    arrow(s, 572, 124, 616, 124)
    arrow(s, 740, 154, 740, 210)
    arrow(s, 616, 242, 572, 242)
    arrow(s, 326, 242, 282, 242)
    s.add(f'<path d="M450 212C 450 190 450 176 450 156" stroke="{RED}" stroke-width="1.5" fill="none" stroke-dasharray="4 3" '
          f'marker-end="url(#arrow)"/>')
    s.text(458, 188, 'if the results differ, go back to the evidence', 10.5, RED)
    s.text(24, 312, 'MAME calls its code documentation of the machines. GameFrame applies the idea to one game.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- who keeps what
def who_keeps():
    s = Svg(900, 300, 'Which part of a game each group keeps. Museums and archives keep hardware, media and papers. Emulators keep the machine. Matching decompilations keep source code. Reimplementations keep the game running from its data')
    s.heading('Other preservation work',
              'Different groups keep different parts of a game. GameFrame works in the same way as the reimplementations in the last row.')
    cols = ['hardware, boxes, papers', 'the machine', 'the original program runs', 'source code', 'runs on current systems']
    rows = [('Museums and archives', [1, 0, 0, 2, 0]), ('Emulators', [0, 1, 1, 0, 1]),
            ('Matching decompilations', [0, 0, 0, 1, 0]), ('Reimplementations', [0, 0, 0, 0, 1])]
    for k, c in enumerate(cols):
        s.text(270 + k * 122 + 55, 96, c, 10, DIM, 'middle')
    for r, (name, vals) in enumerate(rows):
        y = 108 + r * 40
        s.rect(24, y, 852, 34, TEAL if r == 3 else GREY, rx=4, extra=f' fill-opacity="{0.14 if r == 3 else 0.08}"')
        s.text(40, y + 22, name, 12, TEXT, weight='600')
        for k, v in enumerate(vals):
            cx = 270 + k * 122 + 55
            if v == 1:
                s.add(f'<circle cx="{cx}" cy="{y + 17}" r="7" fill="{TEAL}"/>')
            elif v == 2:
                s.add(f'<circle cx="{cx}" cy="{y + 17}" r="6" fill="none" stroke="{TEAL}" stroke-width="2"/>')
    s.add(f'<circle cx="30" cy="284" r="6" fill="none" stroke="{TEAL}" stroke-width="2"/>')
    s.text(42, 288, 'sometimes', 11, DIM)
    s.text(140, 288, 'Examples are The Strong and the VGHF, MAME, DOSBox and Dolphin, Super Mario 64, and ScummVM, OpenRCT2 and DevilutionX.',
           11, DIM)
    return s.render()


# ---------------------------------------------------------------- what a project ships
def ships():
    s = Svg(900, 290, 'What a GameFrame project publishes and what stays on the owner\'s computer. New code and documents are published. The game\'s files, extracted assets and decompiled code are never published. Players bring their own copy')
    s.heading('What a project ships',
              'A project publishes only its own code and documents.')
    s.rect(24, 86, 410, 150, TEAL, rx=6, extra=' fill-opacity="0.14"')
    s.text(40, 112, 'published', 13, TEAL, weight='600')
    lines(s, 40, 138, ['the project\'s own new code', 'its documents of how the game works', 'its tests and tools'], 12, TEXT, gap=22)
    s.rect(466, 86, 410, 150, RED, rx=6, extra=' fill-opacity="0.12"')
    s.text(482, 112, 'stays on the owner\'s computer', 13, RED, weight='600')
    lines(s, 482, 138, ['the game\'s files', 'extracted assets', 'decompiled code and disassembly'], 12, TEXT, gap=22)
    s.text(24, 268, 'Players bring their own copy of the game. The new code reads it when it runs.', 12, TEXT)
    return s.render()


# ---------------------------------------------------------------- AI: the checks (animated)
def ai_gates():
    s = Svg(900, 250, 'The checks GameFrame puts on an agent\'s work. The original is required, results are measured, the port must match, every claim is cited, a person reviews each commit, and nothing from the game is shared')
    s.heading('What GameFrame asks of an agent',
              'Each piece of work must pass every check before it is committed.')
    dur = 7.0
    gates = [('the original', 'is required'), ('results', 'are measured'), ('the port', 'must match'),
             ('every claim', 'is cited'), ('a person', 'reviews it'), ('nothing', 'is shared')]
    for k, (a, b) in enumerate(gates):
        x = 24 + k * 145
        box(s, x, 100, 132, 56, BLUE, None, op=0.22)
        s.add(f'<rect x="{x}" y="100" width="132" height="56" rx="4" fill="{TEAL}" opacity="0">{during((k + 0.5) / 7, 0.97, dur, on="0.35")}</rect>')
        s.text(x + 66, 125, a, 12, TEXT, 'middle', '600')
        s.text(x + 66, 141, b, 10.5, DIM, 'middle')
    s.add(f'<circle cy="128" r="9" fill="{AMBER}"><animate attributeName="cx" values="10;{24 + 6 * 145}" dur="{dur}s" '
          f'repeatCount="indefinite"/></circle>')
    lines(s, 24, 196, ['This takes longer than making an imitation. Anyone with the same copy of the game can run the',
                       'same tests and get the same answers.'], 12, TEXT, gap=20)
    return s.render()


# ---------------------------------------------------------------- one function at a time (animated)
def function_swap():
    s = Svg(900, 300, 'Replacing a program one function at a time. The original program keeps running while its functions are replaced by new ones. Each new function is checked against the original before the next')
    s.heading('Replacing a program one function at a time',
              'OpenRCT2 and dethrace replaced the original program piece by piece while the game kept running.')
    dur = 8.0
    n = 24
    for k in range(n):
        r, c = divmod(k, 12)
        x, y = 24 + c * 71, 96 + r * 56
        s.rect(x, y, 66, 46, GREY, rx=4, extra=' fill-opacity="0.25"')
        s.text(x + 33, y + 28, 'original', 10, DIM, 'middle')
        t0 = 0.9 * ((k * 7) % n) / n
        body = scratch(lambda t, x=x, y=y: box(t, x, y, 66, 46, TEAL, 'new', 'checked', op=0.45, size=11))
        s.add(group(body, t0 + 0.02, 0.97, dur))
    lines(s, 24, 238, ['Each new function must give the original\'s answers before the next is replaced. At every step the game runs,',
                       'so a mistake shows at once. When every function is replaced, no original code is left.'], 12, TEXT, gap=20)
    return s.render()


DIAGRAMS = {
    'preservation-in-print.svg': in_print, 'preservation-breaks.svg': breaks,
    'preservation-not-updated.svg': not_updated, 'preservation-lost.svg': lost, 'preservation-method.svg': method,
    'preservation-who-keeps.svg': who_keeps, 'preservation-ships.svg': ships, 'preservation-ai-gates.svg': ai_gates,
    'preservation-function-swap.svg': function_swap,
}
