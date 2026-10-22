"""Diagrams for the guides on observing the original (docs/observe/) and on working on a project
(docs/working/, OUTPUTS.md, LEGAL.md): memory scans, watchpoints, a frame's draw calls, levels of
emulation, inputs, the object model, captures, system call logs, the interface inventory, the
session loop, the core and its platforms, browser files, round trips, evidence, gates, labels
carried to a new build, practice, fixity, versions, outputs and what is shared.
site/diagrams/make.py writes them.
"""
from reference import arrow, box
from svg import AMBER, BG, BLUE, DIM, GREEN, GREY, PURPLE, RED, TEAL, TEXT, Svg

SANS = 'ui-sans-serif, Helvetica, Arial, sans-serif'
MONOF = 'ui-monospace, Menlo, monospace'


def build(n, i, dur):
    """An opacity animation that shows part i from its turn to the end of the loop."""
    t0 = i / n
    if i == 0:
        return '1', ''
    return '0', (f'<animate attributeName="opacity" values="0;0;1;1" keyTimes="0;{t0:.3f};{t0:.3f};1" dur="{dur}s" '
                 f'repeatCount="indefinite"/>')


def text_el(x, y, t, size, colour, anchor='start', weight='400', mono=False):
    return (f'<text x="{x}" y="{y}" font-family="{MONOF if mono else SANS}" font-size="{size}" font-weight="{weight}" '
            f'fill="{colour}" text-anchor="{anchor}">{t}</text>')


def flow(s, y, steps, w=150, gap=22, x=30, h=60):
    """Boxes left to right with arrows between them: steps are (title, sub, colour)."""
    for k, (a, b, c) in enumerate(steps):
        box(s, x, y, w, h, c, a, b, op='0.28', size=12)
        if k < len(steps) - 1:
            arrow(s, x + w + 2, y + h / 2, x + w + gap - 2, y + h / 2)
        x += w + gap


# ---------------------------------------------------------------- scan, change, scan again (animated)
def memory_scan():
    s = Svg(900, 320, 'A memory scan: search for the value, change it in the game, search again among the matches; '
                      'the candidates fall until one address is left')
    s.heading('Scan, change, scan again', 'Each scan keeps only the addresses whose value changed the way the game\'s did.')
    steps = [('gold = 250', 'every 4-byte value equal to 250', 1_204_311), ('buy a tool: gold = 230', 'of those, now 230', 2_117),
             ('sell wood: gold = 245', 'of those, now 245', 9), ('wait: gold = 245', 'unchanged', 1)]
    dur, n = 8, len(steps)
    maxw = 760
    import math
    for k, (what, how, count) in enumerate(steps):
        y = 92 + k * 52
        w = max(6, maxw * math.log10(count + 1) / math.log10(steps[0][2] + 1))
        op, anim = build(n, k, dur)
        colour = GREEN if count == 1 else AMBER
        s.add(f'<g opacity="{op}">'
              f'<rect x="40" y="{y}" width="{w:.1f}" height="30" rx="4" fill="{colour}" fill-opacity="0.35"/>'
              + text_el(50, y + 20, f'{what}  ({how})', 12, TEXT, weight='600')
              + text_el(860, y + 20, f'{count:,} left', 12, colour, 'end', '600', mono=True) + anim + '</g>')
    s.text(40, 304, 'The bar length is on a log scale. The counts are an example. memscan.py does this over saved memory dumps.',
           11, DIM)
    return s.render()


# ---------------------------------------------------------------- a write watchpoint (animated)
def watchpoint():
    s = Svg(900, 300, 'A write watchpoint on a value\'s address: when the game writes it, the debugger stops at the '
                      'instruction that wrote, which leads to the rule')
    s.heading('From the address to the code', 'Watch the address for writes. The instruction that writes it is in the rule you want.')
    box(s, 40, 100, 220, 60, AMBER, 'gold at 0x04A1F3C', 'watch: break on write', op='0.25')
    box(s, 340, 100, 220, 60, TEAL, 'the game buys a tool', 'gold 250 -> 230', op='0.25')
    arrow(s, 262, 130, 336, 130)
    op, anim = build(3, 1, 6)
    s.add(f'<g opacity="{op}"><rect x="640" y="100" width="220" height="60" rx="6" fill="{RED}" fill-opacity="0.3"/>'
          + text_el(750, 126, 'stopped at 0x4127A0', 13, TEXT, 'middle', '600', mono=True)
          + text_el(750, 146, 'sub [esi+0x3c], eax', 12, DIM, 'middle', mono=True)
          + f'<path d="M562 130H636" stroke="{RED}" stroke-width="2" marker-end="url(#arrow)"/>' + anim + '</g>')
    op, anim = build(3, 2, 6)
    s.add(f'<g opacity="{op}">' + text_el(40, 210, '0x4127A0 is in FUN_004126f0: the purchase rule. esi points to the '
                                          'player, and +0x3c is the gold field.', 12, TEXT)
          + text_el(40, 234, 'Name the function in labels/names.tsv and the field in labels/fields.tsv.', 12, DIM) + anim
          + '</g>')
    return s.render()


# ---------------------------------------------------------------- a frame's draw calls (animated)
def frame_capture():
    s = Svg(900, 360, 'A captured frame as its list of draw calls; each call adds to the picture in order, and the '
                      'capture records the state and inputs of each')
    s.heading('A frame is a list of draw calls', 'A capture tool records each call in order, with its shaders, textures and constants.')
    calls = [('clear', GREY, None), ('terrain', GREEN, (0, 120, 400, 110)), ('buildings', AMBER, (60, 60, 200, 100)),
             ('people', TEAL, (240, 150, 40, 40)), ('trees', '#2f855a', (300, 40, 70, 120)),
             ('interface', BLUE, (0, 0, 400, 24))]
    n, dur = len(calls), 7.2
    for k, (name, colour, rect) in enumerate(calls):
        y = 86 + k * 40
        op, anim = build(n, k, dur)
        s.add(f'<g opacity="{op}"><rect x="40" y="{y}" width="300" height="32" rx="4" fill="{colour}" fill-opacity="0.3"/>'
              + text_el(52, y + 21, f'draw {k + 1}: {name}', 12, TEXT, weight='600') + anim + '</g>')
        if rect:
            x, yy, w, h = rect
            s.add(f'<g opacity="{op}"><rect x="{440 + x}" y="{86 + yy}" width="{w}" height="{h}" rx="3" fill="{colour}" '
                  f'fill-opacity="0.75"/>{anim}</g>')
    s.rect(440, 86, 400, 230, GREY, GREY, rx=3, extra=' fill-opacity="0.1" stroke-opacity="0.6"')
    s.text(440, 334, 'The picture builds up in the order of the calls. The port must keep the order where it shows.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- levels of emulation
def emulation_levels():
    s = Svg(900, 340, 'Levels at which the original can be replaced: low-level emulation copies the chips, high-level '
                      'emulation copies the system software, a reimplementation copies the game')
    s.heading('What each approach replaces', 'The same stack, replaced at different heights.')
    layers = [('the game\'s rules and data', TEAL), ('the game\'s engine code', TEAL),
              ('system software: OS, sound driver, graphics microcode', PURPLE), ('the chips: CPU, video, sound', GREY)]
    for k, (name, colour) in enumerate(layers):
        box(s, 40, 86 + k * 58, 380, 50, colour, name, op='0.25', size=12)
    cols = [('low-level emulation', 3, 3, BLUE), ('high-level emulation', 2, 3, AMBER), ('reimplementation', 0, 3, GREEN)]
    for k, (name, a, b, colour) in enumerate(cols):
        x = 470 + k * 140
        s.text(x + 60, 80, name, 11, colour, 'middle', '600')
        s.rect(x, 86 + a * 58, 120, (b - a + 1) * 58 - 8, colour, rx=6, extra=' fill-opacity="0.3"')
        s.text(x + 60, 86 + a * 58 + ((b - a + 1) * 58 - 8) / 2 + 4, 'new code', 12, TEXT, 'middle', '600')
    s.text(40, 330, 'Above each column the original code still runs. A reimplementation runs none of it.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- the kinds of input
def input_sources():
    s = Svg(900, 360, 'Every input the game reads from outside, gathered through one entry point into the port\'s core')
    s.heading('Everything the game reads from outside', 'Controls are only one input. Each source goes through one entry point in the port.')
    box(s, 350, 160, 200, 70, TEAL, 'the core', 'one input record per step', op='0.3')
    srcs = [('controls', 'keys, mouse, pads', 60, 90), ('time', 'clock, timers, frame time', 60, 180),
            ('randomness', 'seeds from the clock', 60, 270), ('files', 'data, saves, settings', 690, 90),
            ('the system', 'window, focus, display', 690, 180), ('other machines', 'network messages', 690, 270)]
    for name, sub, x, y in srcs:
        box(s, x, y, 160, 52, AMBER, name, sub, op='0.22', size=12)
        if x < 400:
            arrow(s, x + 162, y + 26, 346, 195)
        else:
            arrow(s, x - 2, y + 26, 554, 195)
    return s.render()


# ---------------------------------------------------------------- the input record (animated)
def input_record():
    s = Svg(900, 300, 'A recording of inputs: for each step, the commands and time the core received; replaying it '
                      'gives the same game')
    s.heading('An input record', 'Each step lists what the core received. Replayed, it gives the same game, step by step.')
    rows = [(0, 'seed 1234567'), (1, ''), (2, 'place house at 41, 17'), (3, ''), (4, 'speed 3'),
            (5, ''), (6, ''), (7, 'assign 2 workers to farm 9')]
    n, dur = len(rows), 6.4
    for k, (step, cmd) in enumerate(rows):
        x = 40 + k * 104
        s.rect(x, 100, 96, 90, BLUE if cmd else GREY, rx=4, extra=f' fill-opacity="{0.25 if cmd else 0.15}"')
        s.text(x + 48, 120, f'step {step}', 11, DIM, 'middle', mono=True)
        words = cmd.split()
        for j in range(0, len(words), 2):
            s.text(x + 48, 142 + j * 8, ' '.join(words[j:j + 2]), 10, TEXT, 'middle')
    s.add(f'<rect x="40" y="96" width="96" height="98" rx="5" fill="none" stroke="{AMBER}" stroke-width="2.5">'
          f'<animateTransform attributeName="transform" type="translate" '
          f'values="{";".join(f"{k * 104} 0" for k in range(n))}" calcMode="discrete" dur="{dur}s" '
          f'repeatCount="indefinite"/></rect>')
    s.text(40, 230, 'The same record drives tests, replays, bug reports and lockstep multiplayer.', 12, DIM)
    s.text(40, 252, 'The commands enter the core where the original reacts to input, so random draws stay in order.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- the object model
def object_model():
    s = Svg(900, 320, 'An object model of the running game with entry points: tests and agents query it, press things '
                      'by name and advance time')
    s.heading('A model to query', 'Tests and agents ask the running game what it shows, and act on it by name.')
    box(s, 330, 110, 240, 120, TEAL, 'the running game', 'world, interface, clock', op='0.25')
    users = [('tests', 60, 110), ('agents', 60, 180), ('the probe', 660, 110), ('replays', 660, 180)]
    for name, x, y in users:
        box(s, x, y, 180, 50, BLUE, name, op='0.22')
    for k, (verb, colour) in enumerate((('query: "how many houses?"', AMBER), ('press: "Build > House"', GREEN),
                                        ('advance: 10 seconds', PURPLE))):
        s.text(450, 260 + k * 18, verb, 12, colour, 'middle', mono=True)
    for name, x, y in users:
        if x < 300:
            arrow(s, x + 182, y + 25, 326, 170)
        else:
            arrow(s, x - 2, y + 25, 574, 170)
    return s.render()


# ---------------------------------------------------------------- captures
def capture_pipeline():
    s = Svg(900, 320, 'A run of the original produces captures of frames, state, inputs, calls, files and sound, kept '
                      'in derived/ and compared with the same captures from the port')
    s.heading('What a run of the original gives', 'Each capture is kept out of git, and compared with the same capture from the port.')
    box(s, 30, 130, 170, 70, TEAL, 'the original', 'scripted run', op='0.28')
    caps = ['frames', 'state', 'inputs', 'calls', 'files', 'sound']
    for k, c in enumerate(caps):
        y = 84 + k * 36
        s.rect(260, y, 150, 30, AMBER, rx=4, extra=' fill-opacity="0.25"')
        s.text(335, y + 20, c, 12, TEXT, 'middle', '600')
        arrow(s, 202, 165, 256, y + 15)
    box(s, 470, 130, 160, 70, GREY, 'derived/', 'not in git', op='0.3')
    arrow(s, 412, 165, 466, 165)
    box(s, 700, 80, 170, 60, BLUE, 'the port', 'same inputs', op='0.25')
    box(s, 700, 190, 170, 60, GREEN, 'compare', 'bit by bit', op='0.25')
    arrow(s, 632, 165, 696, 215)
    arrow(s, 785, 142, 785, 186)
    return s.render()


# ---------------------------------------------------------------- system call logs
def syscall_trace():
    s = Svg(900, 330, 'A layer between the game and the system logs each call: Wine\'s relay channel, a proxy DLL, '
                      'Process Monitor or strace; the log shows files, clocks and devices in order')
    s.heading('Logging the calls to the system', 'A layer between the game and the system writes down each call, in order.')
    box(s, 40, 96, 170, 56, TEAL, 'the game', op='0.28')
    box(s, 270, 96, 220, 56, AMBER, 'the logging layer', 'Wine relay, proxy DLL, strace', op='0.28')
    box(s, 550, 96, 170, 56, GREY, 'the system', op='0.3')
    arrow(s, 212, 124, 266, 124)
    arrow(s, 492, 124, 546, 124)
    log = ['CreateFileA("data\\units.dat", GENERIC_READ) = 0x58', 'ReadFile(0x58, 1024 bytes) = 1024',
           'timeGetTime() = 1904223', 'QueryPerformanceCounter() = 81022931', 'DirectSoundCreate(NULL) = 0',
           'RegQueryValueExA("InstallPath") = 0']
    s.rect(40, 176, 820, 130, '#0f2427', rx=4)
    for k, line in enumerate(log):
        s.text(54, 198 + k * 19, line, 12, TEXT if k % 2 == 0 else DIM, mono=True)
    return s.render()


# ---------------------------------------------------------------- the interface inventory
def ui_inventory():
    s = Svg(900, 340, 'An inventory of the interface as a graph of screens, each with its states and the input that '
                      'reaches it')
    s.heading('The interface as a map of screens', 'Each screen, how to reach it, and its states. Capture every one.')
    nodes = {'title': (80, 120), 'main menu': (260, 120), 'options': (260, 230), 'new game': (450, 120),
             'the map': (640, 120), 'build menu': (640, 230), 'pause': (820, 120), 'load': (450, 230)}
    edges = [('title', 'main menu', 'any key'), ('main menu', 'options', 'Options'), ('main menu', 'new game', 'New'),
             ('main menu', 'load', 'Load'), ('new game', 'the map', 'Start'), ('load', 'the map', 'a save'),
             ('the map', 'build menu', 'B'), ('the map', 'pause', 'Esc')]
    for a, b, label in edges:
        (x1, y1), (x2, y2) = nodes[a], nodes[b]
        s.line(x1, y1, x2, y2, DIM, 1.3)
        s.text((x1 + x2) / 2 + 4, (y1 + y2) / 2 - 6, label, 10, AMBER)
    for name, (x, y) in nodes.items():
        s.rect(x - 60, y - 18, 120, 36, BG, rx=18)
        s.rect(x - 60, y - 18, 120, 36, TEAL if name != 'the map' else GREEN, rx=18, extra=' fill-opacity="0.35"')
        s.text(x, y + 5, name, 12, TEXT, 'middle', '600')
    s.text(40, 300, 'States of one screen: empty, full, disabled buttons, error messages, each language. Each is a capture.',
           12, DIM)
    return s.render()


# ---------------------------------------------------------------- the session loop
def session_loop():
    s = Svg(900, 300, 'The loop of a working session: read what is known, trace, measure, port, verify, write it down, '
                      'commit, and start the next slice')
    s.heading('A working session', 'Every session starts by reading what earlier sessions wrote, and ends by writing it down.')
    steps = [('read', 'docs, labels', GREY), ('trace', 'the code', BLUE), ('measure', 'the original', AMBER),
             ('port', 'new code', TEAL), ('verify', 'against truth', GREEN), ('land', 'docs, commit', PURPLE)]
    flow(s, 110, steps, w=120, gap=26)
    s.add(f'<path d="M800 172C 800 230 90 230 90 174" fill="none" stroke="{DIM}" stroke-width="1.5" stroke-dasharray="5 4" '
          f'marker-end="url(#arrow)"/>')
    s.text(445, 246, 'the next slice, or back to trace when a check fails', 11, DIM, 'middle')
    return s.render()


# ---------------------------------------------------------------- the core and its platforms
def core_platforms():
    s = Svg(900, 330, 'The exact core in the middle, with no drawing, sound or windows, wrapped by replaceable platform '
                      'layers: desktop, browser, headless tests and a server')
    s.heading('One core, several platforms', 'The core decides what happens. Each platform gives it time, input and files.')
    box(s, 330, 130, 240, 90, TEAL, 'the core', 'rules, data, clock, random', op='0.32')
    plats = [('desktop', 'window, GPU, sound', 40, 90), ('browser', 'WebAssembly, canvas', 40, 200),
             ('headless tests', 'scripted, no window', 660, 90), ('server', 'rooms, replays', 660, 200)]
    for name, sub, x, y in plats:
        box(s, x, y, 200, 60, BLUE, name, sub, op='0.22')
        if x < 300:
            arrow(s, x + 202, y + 30, 326, 175)
        else:
            arrow(s, x - 2, y + 30, 574, 175)
    s.text(450, 250, 'in: inputs, time, file bytes', 11, AMBER, 'middle')
    s.text(450, 268, 'out: state, draw lists, sound events', 11, AMBER, 'middle')
    s.text(40, 310, 'A test fails if the core starts to depend on a drawing, window, sound or network library.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- the player's files in a browser
def browser_files():
    s = Svg(900, 280, 'In a browser build, the player picks the game folder, the page keeps the files in the browser\'s '
                      'own storage, and the WebAssembly core reads them there; nothing is uploaded')
    s.heading('The player\'s files in a browser', 'The files go from the player\'s disk to the browser\'s storage. Nothing leaves the machine.')
    flow(s, 110, [('the game folder', 'on the player\'s disk', GREY), ('a file picker', 'or a dropped folder', AMBER),
                  ('browser storage', 'IndexedDB or OPFS', PURPLE), ('the core', 'WebAssembly', TEAL)], w=180, gap=40)
    s.rect(30, 200, 840, 34, RED, rx=4, extra=' fill-opacity="0.12"')
    s.text(450, 222, 'no server receives the game files', 12, RED, 'middle', '600')
    return s.render()


# ---------------------------------------------------------------- a round trip
def round_trip():
    s = Svg(900, 280, 'A writer proved by a round trip: read every file, write it back, compare the bytes; each '
                      'difference is a field the writer loses')
    s.heading('A round trip proves a writer', 'Read every file, write it back, compare. Any difference is a field the writer loses.')
    flow(s, 100, [('original file', 'from the game', GREY), ('read', 'the port\'s reader', BLUE),
                  ('write', 'the port\'s writer', AMBER), ('compare', 'byte by byte', GREEN)], w=180, gap=40)
    s.text(40, 210, 'byte 0x1c: 00 vs 7f  -> an unknown flag the reader dropped', 12, RED, mono=True)
    s.text(40, 232, 'Then load the written file in the original, where that is possible.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- evidence labels
def evidence_labels():
    s = Svg(900, 300, 'Evidence labels from strongest to weakest: measured by running the original, observed in its '
                      'code, inferred from other facts, assumed with no evidence')
    s.heading('How strong is a claim?', 'Every finding carries one label. A port is built on measured and observed claims.')
    labels = [('measured', 'produced by running the original', GREEN, 1.0), ('observed', 'read in the code or the data', TEAL, 0.8),
              ('inferred', 'follows from other facts, not checked', AMBER, 0.5), ('assumed', 'no evidence yet', RED, 0.25)]
    for k, (name, sub, colour, strength) in enumerate(labels):
        y = 90 + k * 46
        s.text(40, y + 22, name, 14, colour, weight='600')
        s.rect(150, y + 4, 420 * strength, 26, colour, rx=4, extra=' fill-opacity="0.35"')
        s.text(590, y + 22, sub, 12, TEXT)
    s.text(40, 284, 'Mark inferred and assumed claims in the text, and list them under Open questions.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- evidence chain
def evidence_chain():
    s = Svg(900, 280, 'A chain of evidence: the copy identified by its hashes, a finding tied to an address in that '
                      'copy, written into the documents with its label, and checked by a test')
    s.heading('A finding tied to the copy', 'Each link names the one before it, so anyone can check the chain.')
    flow(s, 100, [('the copy', 'SHA-256 of each file', GREY), ('the finding', 'FUN_004126f0 +0x3c', BLUE),
                  ('the document', 'with its evidence label', AMBER), ('the test', 'fails if it changes', GREEN)], w=180, gap=40)
    s.text(40, 210, 'A finding without the copy\'s hashes cannot be repeated on another copy.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- gates
def gate_flow():
    s = Svg(900, 330, 'How an agent\'s command is checked: permissions allow, ask or deny it; then gates stop game '
                      'material, decompiled code and commits of other agents\' files')
    s.heading('Permissions, then gates', 'A command is allowed, asked about or denied. Gates then check what it would change.')
    box(s, 30, 120, 150, 60, TEAL, 'a command', op='0.28', size=12)
    box(s, 230, 120, 160, 60, BLUE, 'permissions', 'allow, ask, deny', op='0.25', size=12)
    arrow(s, 182, 150, 226, 150)
    gates = [('game material?', 'target/, derived/, saves'), ('decompiled code?', 'pseudocode in tracked files'),
             ('someone else\'s files?', 'staging only your own')]
    for k, (a, b) in enumerate(gates):
        box(s, 450, 80 + k * 70, 200, 56, AMBER, a, b, op='0.22', size=12)
        arrow(s, 392, 150, 446, 108 + k * 70)
    box(s, 720, 120, 150, 60, GREEN, 'runs', op='0.28', size=12)
    for k in range(3):
        arrow(s, 652, 108 + k * 70, 716, 150)
    s.text(450, 312, 'A gate that finds a problem stops the command and says why.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- labels carried to a new build
def label_carry():
    s = Svg(900, 320, 'Labels carried from one build to the next: functions matched by their bytes, calls and strings, '
                      'and each name moved to its new address')
    s.heading('Carrying labels to a new build', 'Match each function in the new build, then move its name to the new address.')
    old = [('0x4126f0', 'buy_item'), ('0x412900', 'sell_item'), ('0x413a10', 'rng_next'), ('0x414000', 'save_game')]
    new = [('0x4127b0', 'buy_item'), ('0x4129c0', 'sell_item'), ('0x413b00', 'rng_next'), ('0x414200', '?')]
    s.text(60, 92, 'build 1.0 (labelled)', 12, TEXT, weight='600')
    s.text(560, 92, 'build 1.1', 12, TEXT, weight='600')
    for k, ((a1, n1), (a2, n2)) in enumerate(zip(old, new)):
        y = 104 + k * 46
        s.rect(60, y, 280, 34, TEAL, rx=4, extra=' fill-opacity="0.25"')
        s.text(72, y + 22, f'{a1}  {n1}', 12, TEXT, mono=True)
        ok = n2 != '?'
        s.rect(560, y, 280, 34, GREEN if ok else GREY, rx=4, extra=f' fill-opacity="{0.25 if ok else 0.15}"')
        s.text(572, y + 22, f'{a2}  {n2 if ok else "no match: changed"}', 12, TEXT if ok else DIM, mono=True)
        if ok:
            arrow(s, 342, y + 17, 556, y + 17, GREEN)
    s.text(60, 300, 'match_functions.py matches two Ghidra exports. A function with no match changed: read it first.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- the practice ladder
def practice_ladder():
    s = Svg(900, 330, 'A ladder of practice exercises, from reading one function to porting a whole system, each '
                      'checked against the game\'s released source')
    s.heading('A ladder of exercises', 'Each rung is checked against a game whose source was released.')
    rungs = ['name one function from its strings', 'recover one structure', 'decode one file format',
             'emulate one function, compare with the source', 'port one system and match it bit for bit']
    for k, r in enumerate(rungs):
        y = 270 - k * 44
        x = 60 + k * 70
        s.rect(x, y, 420, 34, [GREY, BLUE, PURPLE, AMBER, GREEN][k], rx=4, extra=' fill-opacity="0.3"')
        s.text(x + 12, y + 22, f'{k + 1}. {r}', 12, TEXT, weight='600')
    return s.render()


# ---------------------------------------------------------------- fixity (animated)
def fixity_check():
    s = Svg(900, 300, 'A fixity check: each file\'s checksum is recorded once, then recomputed later; a mismatch shows '
                      'that a file changed')
    s.heading('Fixity: has anything changed?', 'Record each file\'s checksum once. Recompute it later and compare.')
    files = [('GAME.EXE', 'a3f0...91c2', True), ('DATA.PAK', '7be1...04aa', True), ('MUSIC.PAK', '19c4...ee70', False),
             ('README.TXT', '00d9...5b13', True)]
    for k, (name, h, ok) in enumerate(files):
        y = 92 + k * 44
        s.text(40, y + 22, name, 13, TEXT, mono=True)
        s.text(200, y + 22, h, 13, DIM, mono=True)
        op, anim = build(2, 1, 5)
        mark = 'same' if ok else 'CHANGED'
        s.add(f'<g opacity="{op}"><rect x="420" y="{y}" width="160" height="32" rx="4" fill="{GREEN if ok else RED}" '
              f'fill-opacity="0.3"/>' + text_el(500, y + 21, mark, 12, TEXT, 'middle', '600') + anim + '</g>')
    s.text(40, 280, 'fixity.py writes a BagIt bag with a manifest of SHA-256 sums, and checks it again later.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- versions
def version_tree():
    s = Svg(900, 300, 'The versions of one game: a demo, the release, patches, a regional release and a re-release; '
                      'the port chooses one and records it')
    s.heading('The versions of one game', 'Choose the version the port copies. Use the others as evidence.')
    pts = [('demo', 'symbols left in', 60, 160, GREY), ('1.0', 'first release', 230, 110, BLUE), ('1.01', 'patch', 400, 110, BLUE),
           ('1.02', 'last patch', 570, 110, GREEN), ('1.0 (Japan)', 'regional', 400, 210, PURPLE),
           ('GOG / Steam', 're-release', 740, 110, AMBER)]
    links = [(0, 1), (1, 2), (2, 3), (1, 4), (3, 5)]
    for a, b in links:
        (_, _, x1, y1, _), (_, _, x2, y2, _) = pts[a], pts[b]
        if y2 > y1 + 40:
            s.line(x1 + 60, y1 + 52, x2 - 2, y2 + 25, DIM, 1.5, ' marker-end="url(#arrow)"')
        else:
            s.line(x1 + 122, y1 + 25, x2 - 2, y2 + 25, DIM, 1.5, ' marker-end="url(#arrow)"')
    for name, sub, x, y, c in pts:
        box(s, x, y, 120, 50, c, name, sub, op='0.28', size=12)
    s.text(570, 186, 'the port follows this one', 11, GREEN, 'middle')
    s.text(40, 284, 'A patch note names a rule. The function that changed between two builds is that rule.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- outputs
def outputs_map():
    s = Svg(900, 320, 'What a project can ship: findings as documents and a website, the reimplementation as programs '
                      'for each system, a browser version and a server, and tools for players')
    s.heading('What a project ships', 'Everything here is new work. None of it contains the game\'s files.')
    box(s, 350, 84, 200, 56, TEAL, 'the project', op='0.3')
    outs = [('documents', 'findings, guides', 40, 200, BLUE), ('a website', 'built from the docs', 210, 200, BLUE),
            ('programs', 'Windows, macOS, Linux', 380, 200, GREEN), ('a browser version', 'on the player\'s files', 550, 200, GREEN),
            ('a server', 'multiplayer, replays', 720, 200, PURPLE)]
    for name, sub, x, y, c in outs:
        box(s, x, y, 150, 56, c, name, sub, op='0.25', size=12)
        arrow(s, 450, 142, x + 75, y - 2)
    s.text(40, 300, 'Releases are built by CI from a tag. Players bring their own copy of the game.', 12, DIM)
    return s.render()


# ---------------------------------------------------------------- what is shared and what is kept
def shared_kept():
    s = Svg(900, 320, 'What goes into the public repository (new code, documents, tools, labels) and what stays on the '
                      'player\'s machine (the game\'s files, captures, decompiled output)')
    s.heading('Shared and kept', 'The repository holds new work. The game and everything made from it stays on your machine.')
    s.rect(40, 84, 390, 200, GREEN, rx=8, extra=' fill-opacity="0.12"')
    s.text(60, 110, 'the public repository', 14, GREEN, weight='600')
    for k, t in enumerate(['new code, written from documented behaviour', 'documents and findings, in prose and tables',
                           'tools and tests', 'labels: names, addresses, offsets']):
        s.text(60, 140 + k * 26, '+ ' + t, 12, TEXT)
    s.rect(470, 84, 390, 200, RED, rx=8, extra=' fill-opacity="0.12"')
    s.text(490, 110, 'your machine only', 14, RED, weight='600')
    for k, t in enumerate(['the game\'s files (target/)', 'captures and dumps (derived/)',
                           'Ghidra projects and decompiled code (analysis/)', 'saves']):
        s.text(490, 140 + k * 26, '- ' + t, 12, TEXT)
    s.text(40, 306, 'The gates and the ignore file keep the right column out of git.', 12, DIM)
    return s.render()


DIAGRAMS = {
    'memory-scan.svg': memory_scan, 'watchpoint.svg': watchpoint, 'frame-capture.svg': frame_capture,
    'emulation-levels.svg': emulation_levels, 'input-sources.svg': input_sources, 'input-record.svg': input_record,
    'object-model.svg': object_model, 'capture-pipeline.svg': capture_pipeline, 'syscall-trace.svg': syscall_trace,
    'ui-inventory.svg': ui_inventory, 'session-loop.svg': session_loop, 'core-platforms.svg': core_platforms,
    'browser-files.svg': browser_files, 'round-trip.svg': round_trip, 'evidence-labels.svg': evidence_labels,
    'evidence-chain.svg': evidence_chain, 'gate-flow.svg': gate_flow, 'label-carry.svg': label_carry,
    'practice-ladder.svg': practice_ladder, 'fixity-check.svg': fixity_check, 'version-tree.svg': version_tree,
    'outputs-map.svg': outputs_map, 'shared-kept.svg': shared_kept,
}
