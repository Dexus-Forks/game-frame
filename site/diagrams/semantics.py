"""Diagrams for the pages on how code behaves (docs/semantics/) and on script runtimes in
docs/engines/: division edge cases, what IEEE 754 makes the same, rand() in three runtimes, a
Shift-JIS byte that looks like a backslash, a Lua table, three ways to dispatch opcodes, a C#
iterator as a state machine, and a destroyed Unity object. site/diagrams/make.py writes them.
"""
from engines import arrow, box, during, group, legend, lines, scratch
from svg import AMBER, BG, BLUE, DIM, EDGE, GREEN, GREY, MONO, PURPLE, RED, TEAL, TEXT, Svg


# ---------------------------------------------------------------- integer division edges
def integer_division():
    s = Svg(900, 450, 'What happens when a signed integer is divided by zero, and when the most negative value is '
                      'divided by -1, on six CPUs and in four languages a port may use')
    s.heading('Two divisions that differ everywhere',
              'Each row gives the result of x / 0 and of the most negative value / -1 for 32-bit signed integers.')
    rows = [('x86 (idiv)', ('divide error, crash', RED), ('divide error, crash', RED)),
            ('68000 (divs)', ('trap, vector 5', RED), ('overflow flag, no change', AMBER)),
            ('MIPS (div)', ('no trap, unpredictable', AMBER), ('no trap, unpredictable', AMBER)),
            ('PowerPC (divw)', ('undefined result', AMBER), ('undefined result', AMBER)),
            ('ARMv7 and later (sdiv)', ('0', TEAL), ('the most negative value', TEAL)),
            ('6502, Z80, early ARM', ('the game\'s own routine', GREY), ('the game\'s own routine', GREY)),
            ('Rust', ('panic', RED), ('panic', RED)),
            ('JavaScript, then | 0', ('Infinity, then 0', TEAL), ('2147483648, then the most negative', TEAL)),
            ('WebAssembly (i32.div_s)', ('trap', RED), ('trap', RED)),
            ('C#', ('DivideByZeroException', RED), ('OverflowException', RED))]
    s.text(260, 92, 'x / 0', 12, TEXT, weight='600')
    s.text(560, 92, 'most negative value / -1', 12, TEXT, weight='600')
    for k, (name, a, b) in enumerate(rows):
        y = 102 + k * 30
        if k == 6:
            s.line(24, y - 3, 876, y - 3, EDGE, 1.5)
        s.text(24, y + 17, name, 11.5, TEXT)
        for x, (label, colour) in ((260, a), (560, b)):
            s.rect(x, y, 290, 24, colour, rx=3, extra=' fill-opacity="0.28"')
            s.text(x + 10, y + 16, label, 11, TEXT)
    s.text(24, 410, 'The first six rows are CPUs. The last four are languages a port may be written in.', 11.5, DIM)
    legend(s, 24, 436, [(RED, 'stops the program'), (AMBER, 'no defined result'), (TEAL, 'a defined value'),
                        (GREY, 'depends on the routine')])
    return s.render()


# ---------------------------------------------------------------- IEEE 754: same and different
def float_sameness():
    s = Svg(900, 330, 'The operations that IEEE 754 makes give the same bits on every machine, beside the things that '
                      'still differ between CPUs, runtimes and languages')
    s.heading('Floats that are the same everywhere',
              'With the same inputs, precision, rounding mode and order of operations, the left column gives the same bits.')
    s.rect(24, 86, 410, 196, TEAL, rx=6, extra=' fill-opacity="0.12"')
    s.text(40, 112, 'The same bits on every IEEE 754 machine', 12.5, TEAL, weight='600')
    lines(s, 40, 138, ['addition and subtraction', 'multiplication and division', 'square root',
                       'conversion between integers and floats', 'comparisons'], 12, TEXT, gap=24)
    s.rect(466, 86, 410, 196, AMBER, rx=6, extra=' fill-opacity="0.12"')
    s.text(482, 112, 'Can differ between platforms', 12.5, AMBER, weight='600')
    lines(s, 482, 138, ['sin, cos, pow, exp and other maths functions', 'rcpss and rsqrtss estimates (Intel and AMD)',
                        'fused multiply-add where a compiler adds it', 'the bits of a NaN result',
                        'denormals flushed to zero', 'x87 extended precision between steps'], 12, TEXT, gap=24)
    s.text(24, 312, 'A port gets the same results when it keeps the left column exact and reproduces each item on the right.',
           11.5, DIM)
    return s.render()


# ---------------------------------------------------------------- rand() in three runtimes
def _msvc(n):
    s, o = 1, []
    for _ in range(n):
        s = (s * 214013 + 2531011) & 0xffffffff
        o.append((s >> 16) & 0x7fff)
    return o


def _borland(n):
    s, o = 1, []
    for _ in range(n):
        s = (s * 22695477 + 1) & 0xffffffff
        o.append((s >> 16) & 0x7fff)
    return o


def _glibc(n):
    """glibc's TYPE_3 random() after srand(1): an additive generator with 31 words of state."""
    r = [1]
    for i in range(1, 31):
        hi, lo = divmod(r[i - 1], 127773)
        word = 16807 * lo - 2836 * hi
        r.append(word + 2147483647 if word < 0 else word)
    r += r[:3]
    for i in range(34, 344):
        r.append((r[i - 31] + r[i - 3]) & 0xffffffff)
    out = []
    while len(out) < n:
        i = len(r)
        r.append((r[i - 31] + r[i - 3]) & 0xffffffff)
        out.append(r[i] >> 1)
    return out


def rand_runtimes():
    s = Svg(900, 300, 'The first three numbers that rand() returns after srand(1) in the Microsoft C runtime, in '
                      'Borland C and in the GNU C library')
    s.heading('rand() after srand(1) in three C runtimes',
              'The same call gives other numbers in each runtime. A port must use the original\'s generator.')
    rows = [('Microsoft C', _msvc(3), TEAL, 'RAND_MAX 32767'), ('Borland C, Turbo C', _borland(3), BLUE, 'RAND_MAX 32767'),
            ('GNU C library', _glibc(3), PURPLE, 'RAND_MAX 2147483647')]
    dur = 6.0
    for r, (name, vals, colour, note) in enumerate(rows):
        y = 92 + r * 52
        s.text(24, y + 26, name, 12.5, TEXT, weight='600')
        for k, v in enumerate(vals):
            body = scratch(lambda t, x=210 + k * 170, y=y, v=v, colour=colour:
                           box(t, x, y + 4, 160, 36, colour, f'{v:,}', None, op=0.3, size=12, mono=True))
            s.add(group(body, 0.08 + k * 0.2, 0.97, dur))
        s.text(730, y + 26, note, 10.5, DIM, mono=True)
    s.text(24, 270, 'The numbers are computed from each runtime\'s documented algorithm. rand() % 6 then gives other dice in each one.',
           11.5, DIM)
    return s.render()


# ---------------------------------------------------------------- Shift-JIS and the backslash (animated)
def shift_jis():
    s = Svg(900, 320, 'The Shift-JIS bytes of a Japanese word: the second byte of its first character is 0x5C, the code '
                      'of the backslash, so a search one byte at a time finds a backslash that is not there')
    s.heading('A backslash inside a Japanese character',
              'In Shift-JIS, the second byte of a character can be 0x5C. A byte-by-byte search finds a backslash there.')
    data = [(0x95, 0), (0x5C, 0), (0x8E, 1), (0xA6, 1), (0x2E, 2), (0x74, 3), (0x78, 4), (0x74, 5)]
    chars = ['表 (two bytes)', '示 (two bytes)', '.', 't', 'x', 't']
    x0, cw = 40, 92
    for k, (b, c) in enumerate(data):
        x = x0 + k * cw
        colour = RED if b == 0x5C else (AMBER if c < 2 else BLUE)
        s.rect(x, 110, cw - 6, 40, colour, rx=4, extra=' fill-opacity="0.3"')
        s.text(x + (cw - 6) / 2, 136, f'{b:02X}', 15, TEXT, 'middle', '600', mono=True)
    for c, label in enumerate(chars):
        ks = [k for k, (_, cc) in enumerate(data) if cc == c]
        a, b = x0 + ks[0] * cw, x0 + ks[-1] * cw + cw - 6
        s.line(a, 162, b, 162, DIM, 1.5)
        s.text((a + b) / 2, 180, label, 11.5, DIM, 'middle')
    s.text(x0, 98, 'the file name 表示.txt in Shift-JIS', 11.5, DIM)
    dur = 6.0
    xs = ';'.join(str(x0 - 3 + k * cw) for k in range(len(data)))
    kt = ';'.join(f'{k / len(data):.4f}' for k in range(len(data)))
    s.add(f'<rect y="106" width="{cw}" height="48" rx="6" fill="none" stroke="{TEAL}" stroke-width="2.5">'
          f'<animate attributeName="x" values="{xs}" keyTimes="{kt}" calcMode="discrete" dur="{dur}s" '
          f'repeatCount="indefinite"/></rect>')
    s.add(group(scratch(lambda t: t.text(x0 + cw + 43, 214, 'a byte search for 0x5C stops here and splits the character',
                                         11.5, RED, 'middle')), 1 / 8, 0.97, dur))
    lines(s, 24, 256, ['Japanese fonts show 0x5C as a yen sign. Code that splits paths at a backslash must step through text',
                       'one character at a time. A lead byte of 0x81 to 0x9F or 0xE0 to 0xEF starts a two-byte character.'],
          11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- a Lua table
def lua_table():
    s = Svg(900, 330, 'A Lua table with an array part holding keys 1 to 4 and a hash part holding named keys in its '
                      'internal node order; next() visits the array part first, then the nodes in their order')
    s.heading('A Lua table and the order of next()',
              'The array part holds keys 1 to n. The hash part holds the rest, in nodes placed by each key\'s hash.')
    s.text(24, 96, 'array part', 12, TEXT, weight='600')
    for k, v in enumerate(['"sword"', '"shield"', '"potion"', '"key"']):
        box(s, 24 + k * 100, 106, 94, 44, TEAL, f'[{k + 1}]', v, op=0.25, size=11.5, mono=True)
    s.text(470, 96, 'hash part (an example of node order)', 12, TEXT, weight='600')
    nodes = [('"hp"', '30'), ('empty', ''), ('"name"', '"Bob"'), ('"x"', '12'), ('empty', ''), ('"gold"', '50')]
    for k, (key, val) in enumerate(nodes):
        x = 470 + (k % 3) * 136
        y = 106 + (k // 3) * 54
        if key == 'empty':
            box(s, x, y, 128, 44, GREY, 'empty node', None, op=0.12, size=10.5)
        else:
            box(s, x, y, 128, 44, PURPLE, key, val, op=0.25, size=11.5, mono=True)
    order = ['[1]', '[2]', '[3]', '[4]', '"hp"', '"name"', '"x"', '"gold"']
    s.text(24, 252, 'pairs() and next() visit', 12, TEXT, weight='600')
    for k, name in enumerate(order):
        x = 24 + k * 106
        box(s, x, 262, 98, 30, TEAL if k < 4 else PURPLE, f'{k + 1}. {name}', None, op=0.3, size=11, mono=True)
    s.text(24, 316, 'The node order depends on the hashes, the table size and the order of insertions. The manual does not fix it.',
           11.5, DIM)
    return s.render()


# ---------------------------------------------------------------- dispatch styles
def dispatch_styles():
    s = Svg(900, 330, 'Three ways an interpreter dispatches opcodes: a switch inside one loop, a loop that calls '
                      'handlers through a table of function pointers, and threaded code where each handler jumps to the next')
    s.heading('Three ways to dispatch opcodes',
              'Each panel shows how control moves from one opcode to the next. The table of handlers is the list of opcodes.')
    panels = [('A switch in a loop', TEAL, ['read opcode', 'jump through switch table', 'case: do the work',
                                            'back to the top']),
              ('A table of handlers', BLUE, ['read opcode', 'call handlers[opcode]', 'handler does the work',
                                             'return to the loop']),
              ('Threaded code', PURPLE, ['handler does the work', 'read the next opcode', 'jump through the table',
                                         'into the next handler'])]
    for p, (title, colour, steps) in enumerate(panels):
        x = 24 + p * 290
        s.rect(x, 82, 272, 210, colour, rx=6, extra=' fill-opacity="0.1"')
        s.text(x + 14, 106, title, 12.5, TEXT, weight='600')
        for k, step in enumerate(steps):
            y = 120 + k * 40
            box(s, x + 24, y, 224, 30, colour, step, None, op=0.3, size=11)
            if k < 3:
                arrow(s, x + 136, y + 30, x + 136, y + 38)
        if p < 2:
            s.add(f'<path d="M{x + 250} 285C {x + 268} 285 {x + 268} 135 {x + 250} 135" fill="none" stroke="{DIM}" '
                  f'stroke-width="1.5" marker-end="url(#arrow)"/>')
    s.text(24, 316, 'SCUMM, SCI and Lua use a switch. UnrealScript calls native functions through a table. GCC\'s computed goto gives threaded code.',
           11.5, DIM)
    return s.render()


# ---------------------------------------------------------------- a C# iterator as a state machine (animated)
def iterator_state_machine():
    s = Svg(900, 360, 'A C# iterator method with two yield returns, beside the class the compiler makes from it, whose '
                      'MoveNext switches on a state field and stores the next state before each return')
    s.heading('A Unity coroutine after compiling',
              'The compiler turns the method into a class. MoveNext jumps to the code after the last yield.')
    src = ['IEnumerator FadeOut() {', '    alpha = 1;', '    yield return null;', '    alpha = 0.5f;',
           '    yield return new WaitForSeconds(2);', '    alpha = 0;', '}']
    s.rect(24, 84, 380, 190, GREY, rx=6, extra=' fill-opacity="0.12"')
    s.text(36, 104, 'the source', 11, DIM)
    for k, line in enumerate(src):
        indent = (len(line) - len(line.lstrip(' '))) // 4
        s.text(36 + indent * 24, 128 + k * 20, line.strip(), 11.5, TEXT, mono=True)
    s.rect(440, 84, 436, 190, PURPLE, rx=6, extra=' fill-opacity="0.12"')
    s.text(452, 104, 'class <FadeOut>d__3, method MoveNext', 11, DIM, mono=True)
    cases = [('case 0:', 'alpha = 1; state = 1; return true'), ('case 1:', 'alpha = 0.5f; state = 2; return true'),
             ('case 2:', 'alpha = 0; state = -1; return false')]
    dur = 6.0
    for k, (c, body) in enumerate(cases):
        y = 122 + k * 46
        s.text(456, y + 20, c, 11.5, AMBER, mono=True)
        s.rect(520, y, 344, 30, BLUE, rx=3, extra=' fill-opacity="0.22"')
        s.text(530, y + 20, body, 11, TEXT, mono=True)
        s.add(f'<rect x="452" y="{y - 4}" width="416" height="38" rx="5" fill="none" stroke="{TEAL}" stroke-width="2.5" '
              f'opacity="{1 if k == 0 else 0}">{during(k / 3, (k + 1) / 3, dur)}</rect>')
    lines(s, 24, 300, ['Each call of MoveNext runs one piece. Unity calls it again in the next frame after yield return null,',
                       'and two seconds of scaled game time after WaitForSeconds(2). The state field <>1__state holds the piece to run.'],
          11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- Unity's Destroy over a frame
def unity_destroy():
    s = Svg(900, 300, 'A Unity object destroyed during Update: it stays until the end of the Update loop, is destroyed '
                      'before drawing, and afterwards compares equal to null while the C# object still exists')
    s.heading('A destroyed Unity object, frame by frame',
              'Destroy waits until the Update loop ends. After that, == null is true, but the C# object remains.')
    cols = [('Update calls Destroy(enemy)', AMBER), ('the rest of the Update loop', BLUE),
            ('destroyed, before drawing', RED), ('later frames', GREY)]
    x0, cw = 200, 168
    for k, (label, colour) in enumerate(cols):
        box(s, x0 + k * cw, 90, cw - 6, 44, colour, label, None, op=0.3, size=10.5)
    rows = [('enemy == null', ['false', 'false', '', 'true']), ('enemy?.name', ['works', 'works', '', 'throws']),
            ('FindObjectOfType', ['finds it', 'finds it', '', 'does not']), ('its Update runs', ['yes', 'yes', '', 'no'])]
    for r, (check, vals) in enumerate(rows):
        y = 152 + r * 28
        s.rect(24, y, 852, 24, GREY, rx=3, extra=' fill-opacity="0.08"')
        s.text(36, y + 16, check, 11, AMBER, mono=True)
        for k, v in enumerate(vals):
            if v:
                s.text(x0 + k * cw + (cw - 6) / 2, y + 16, v, 11, TEXT, 'middle', mono=True)
    s.text(24, 284, 'A port must give each kind of check the same answer at the same point of the frame.', 11.5, DIM)
    return s.render()


DIAGRAMS = {
    'semantics-integer-division.svg': integer_division, 'semantics-float-sameness.svg': float_sameness,
    'semantics-rand-runtimes.svg': rand_runtimes, 'semantics-shift-jis.svg': shift_jis,
    'engine-lua-table.svg': lua_table, 'engine-dispatch-styles.svg': dispatch_styles,
    'engine-iterator-state-machine.svg': iterator_state_machine, 'engine-unity-destroy.svg': unity_destroy,
}
