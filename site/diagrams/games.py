"""Diagrams for the guides to kinds of games (docs/games/): which systems matter in which family,
the routes other projects took, games that run on their own clock and games that wait, the four
kinds of interaction and mixed games, and one diagram for each family's rule set. site/diagrams/make.py
writes them.
"""
import math

from engines import arrow, box, during, group, legend, lines, scratch
from runtime import show
from svg import AMBER, BG, BLUE, DIM, EDGE, GREEN, GREY, MONO, PURPLE, RED, TEAL, TEXT, Svg

FAMILIES = ['arcade', 'first-person', 'racing', 'vehicle sims', 'management', 'strategy', 'RPG', 'adventure', 'puzzle',
            'online']


# ---------------------------------------------------------------- README: connections
def connections():
    s = Svg(900, 514, 'A grid of shared systems and families of games. A large dot marks a family where a system matters most. A small dot marks a family where it also appears')
    s.heading('Systems that several kinds of game share',
              'A large dot marks the kinds of game where a system matters most. A small dot marks the others where it appears.')
    rows = [('Fixed-step frame loop', {'arcade', 'first-person', 'racing', 'vehicle sims', 'management', 'strategy'}, set()),
            ('Input recorded per frame', {'arcade', 'first-person', 'racing', 'strategy'}, set()),
            ('Lockstep determinism', {'strategy', 'management', 'arcade'}, set()),
            ('Random draws and their order', {'management', 'strategy', 'RPG', 'puzzle'}, set(FAMILIES)),
            ('Tile maps and layers', {'arcade', 'strategy', 'RPG', 'management'}, set()),
            ('Pathfinding', {'strategy', 'management', 'RPG'}, set()),
            ('Fixed-point maths', {'vehicle sims', 'first-person'}, set(FAMILIES) - {'online'}),
            ('Software 3D rendering', {'first-person', 'vehicle sims', 'racing'}, set()),
            ('Physics', {'racing', 'vehicle sims'}, set()),
            ('Scripts and byte code', {'adventure', 'RPG', 'strategy'}, set()),
            ('Data tables of rules', {'strategy', 'RPG', 'management', 'arcade'}, set()),
            ('Generation from a seed', {'vehicle sims', 'management', 'strategy'}, set()),
            ('A world simulated unseen', {'management', 'strategy', 'RPG', 'online'}, set()),
            ('Saves of the whole world', {'management', 'strategy', 'RPG'}, set()),
            ('Event scripts in levels', {'arcade', 'first-person', 'RPG'}, set())]
    x0, cw, y0, rh = 230, 64, 110, 24
    for k, f in enumerate(FAMILIES):
        x = x0 + k * cw + cw / 2
        s.add(f'<text x="{x:.1f}" y="{y0 - 8}" font-family="ui-sans-serif, Helvetica, Arial, sans-serif" font-size="10.5" '
              f'fill="{DIM}" text-anchor="start" transform="rotate(-28 {x:.1f} {y0 - 8})">{f}</text>')
    for r, (name, main, also) in enumerate(rows):
        y = y0 + 10 + r * rh
        if r % 2 == 0:
            s.rect(20, y - 4, 860, rh, GREY, rx=3, extra=' fill-opacity="0.08"')
        s.text(24, y + 12, name, 11.5, TEXT)
        for k, f in enumerate(FAMILIES):
            cx = x0 + k * cw + cw / 2
            if f in main:
                s.add(f'<circle cx="{cx:.1f}" cy="{y + 8}" r="7" fill="{TEAL}"/>')
            elif f in also:
                s.add(f'<circle cx="{cx:.1f}" cy="{y + 8}" r="3" fill="{DIM}"/>')
    s.add(f'<circle cx="30" cy="{y0 + 10 + len(rows) * rh + 16}" r="7" fill="{TEAL}"/>')
    s.text(44, y0 + 14 + len(rows) * rh + 16, 'matters most', 11, DIM)
    s.add(f'<circle cx="160" cy="{y0 + 10 + len(rows) * rh + 16}" r="3" fill="{DIM}"/>')
    s.text(172, y0 + 14 + len(rows) * rh + 16, 'also appears (random draws in every family, fixed point in DOS-era games of every family)',
           11, DIM)
    return s.render()


# ---------------------------------------------------------------- README: approaches
def approaches():
    s = Svg(900, 340, 'Four approaches seen in other projects, which are a reimplementation, an engine reimplementation, a matching decompilation and a source port, with what each starts from and what it makes')
    s.heading('Four routes that other projects took',
              'Each row shows what a route starts from, what it makes and what you can learn from it.')
    rows = [('Reimplementation', 'the shipped program', 'new code, checked against it', 'method, structure, verification', TEAL),
            ('Engine reimplementation', 'the game\'s data', 'a new engine that runs it', 'how the data and scripts work', BLUE),
            ('Matching decompilation', 'the shipped program', 'source that compiles to the same bytes', 'the exact structure, slowly',
             PURPLE),
            ('Source port', 'source the owner released', 'the same code on new systems', 'how the original was built', AMBER)]
    s.text(24, 88, 'route', 11, DIM)
    s.text(230, 88, 'starts from', 11, DIM)
    s.text(430, 88, 'makes', 11, DIM)
    s.text(680, 88, 'learn from it', 11, DIM)
    for k, (name, src, out, learn, colour) in enumerate(rows):
        y = 98 + k * 52
        s.rect(24, y, 852, 44, colour, rx=5, extra=' fill-opacity="0.14"')
        s.text(36, y + 27, name, 12.5, TEXT, weight='600')
        box(s, 226, y + 7, 170, 30, GREY, src, op=0.3, size=11)
        arrow(s, 398, y + 22, 424, y + 22)
        box(s, 428, y + 7, 236, 30, colour, out, op=0.3, size=11)
        s.text(680, y + 27, learn, 11, DIM)
    s.text(24, 322, 'A clean-room project reads other projects\' documents and method without copying their code.', 12, TEXT)
    return s.render()


# ---------------------------------------------------------------- interaction: the game's own clock (animated)
def own_clock():
    s = Svg(900, 380, 'A game that runs on its own clock. Each frame reads the input state once, updates in a fixed order and draws. Everything from outside comes in as one record per frame. The picture and sounds go out the same way')
    s.heading('A game that runs on its own clock',
              'The game runs one loop in a fixed order every frame. Everything from outside the game comes in through one record per frame.')
    dur = 4.0
    steps = [('read input state', TEAL), ('move the player', BLUE), ('move enemies, shots', BLUE),
             ('collisions', BLUE), ('scroll, spawn waves', BLUE), ('draw the frame and HUD', AMBER)]
    cx, cy, r = 450, 206, 96
    n = len(steps)
    for k, (name, colour) in enumerate(steps):
        a = -math.pi / 2 + k * 2 * math.pi / n
        x, y = cx + math.cos(a) * r, cy + math.sin(a) * r
        box(s, x - 72, y - 16, 144, 32, colour, name, op=0.25, size=11)
        s.add(f'<rect x="{x - 72:.1f}" y="{y - 16:.1f}" width="144" height="32" rx="4" fill="none" stroke="{AMBER}" '
              f'stroke-width="2.5" opacity="{1 if k == 0 else 0}">{during(k / n, (k + 1) / n, dur)}</rect>')
    s.text(cx, cy - 2, 'one frame', 12, DIM, 'middle')
    s.add(f'<text x="{cx}" y="{cy + 18}" font-family="{MONO}" font-size="13" fill="{TEXT}" text-anchor="middle">frame n'
          f'</text>')
    # In and out.
    box(s, 24, 100, 190, 220, GREY, None, op=0.15)
    s.text(36, 122, 'the record for frame n', 12, TEXT, weight='600')
    lines(s, 36, 146, ['keys and buttons (state)', 'the clock', 'the random seed', 'files and saves', 'settings',
                       'the hardware detected', 'window messages'], 11, DIM, gap=22)
    arrow(s, 216, 210, 300, 210)
    box(s, 690, 120, 186, 180, GREY, None, op=0.15)
    s.text(702, 142, 'out of frame n', 12, TEXT, weight='600')
    lines(s, 702, 166, ['the finished picture', 'the songs and sounds', 'asked for', '', 'compared exactly with',
                        'the original\'s'], 11, DIM, gap=20)
    arrow(s, 600, 210, 686, 210)
    s.text(24, 358, 'Two runs with the same records give the same frames. A demo, a test or a player can drive the core.', 12, TEXT)
    return s.render()


# ---------------------------------------------------------------- interaction: waiting for the player (animated)
def waits():
    s = Svg(900, 320, 'A game that waits for the player. Nothing changes until a command arrives. The rules then resolve it in a fixed order into a new state. An animation loop may run without changing the state')
    s.heading('A game that waits for the player',
              'Nothing changes until the player gives a command. The rules then resolve the command, in a fixed order, into a new state.')
    dur = 6.0
    box(s, 24, 110, 170, 60, GREY, 'state', 'board, flags, inventory', op=0.3)
    s.add(group(scratch(lambda t: box(t, 24, 200, 170, 40, AMBER, 'the command "open door"', op=0.35, size=11)), 0.2, 0.98, dur))
    s.add(f'<path d="M110 198L110 174" stroke="{AMBER}" stroke-width="2" marker-end="url(#arrow)" opacity="0">'
          f'{during(0.25, 0.98, dur)}</path>')
    res = ['check it is allowed', 'apply effects in order', 'draw random numbers', 'run scripts it triggers']
    for k, name in enumerate(res):
        x = 236 + k * 162
        body = scratch(lambda t, x=x, name=name: box(t, x, 118, 150, 44, BLUE, name, op=0.3, size=11))
        s.add(group(body, 0.3 + k * 0.1, 0.98, dur))
        s.rect(x, 118, 150, 44, BLUE, rx=4, extra=' fill-opacity="0.08"')
    s.add(group(scratch(lambda t: box(t, 236, 190, 636, 34, TEAL, 'the new state, then wait again', op=0.3, size=11.5)), 0.72, 0.98, dur))
    s.text(236, 100, 'resolving one move', 11, DIM)
    lines(s, 24, 264, ['An animation loop may run while the game waits. Find which parts only animate and which change the state.',
                       'A recording is the list of commands.'], 11.5, DIM, gap=18)
    s.text(24, 304, 'Check the rules move by move. The same command on the same state must give the same new state.', 12, TEXT)
    return s.render()


# ---------------------------------------------------------------- interaction: the four kinds (animated)
def four_kinds():
    s = Svg(900, 400, 'What happens over time in each of the four kinds of game. A stream has events every frame. A simulation steps steadily with occasional commands. An environment has triggers on entering areas. Turns resolve only after a move')
    s.heading('The four kinds of game over time',
              'Each row is a strip of time. The marks show what changes the game and who caused the change.')
    dur = 6.0
    x0, w = 250, 620
    rows = [('A stream with controls', 'Tyrian, Doom', TEAL), ('Software over a simulation', 'SimCity 2000, Banished', BLUE),
            ('An environment to move through', 'Morrowind, Ultima VII', PURPLE), ('Turns and choices', 'Civilization, Zork', AMBER)]
    for r, (name, ex, colour) in enumerate(rows):
        y = 92 + r * 70
        s.text(24, y + 20, name, 12.5, TEXT, weight='600')
        s.text(24, y + 38, ex, 11, DIM)
        s.line(x0, y + 26, x0 + w, y + 26, EDGE, 1.5)
        if r == 0:
            for k in range(60):
                s.rect(x0 + k * w / 60, y + 14, 6, 24, colour, rx=1, extra=' fill-opacity="0.55"')
            s.text(x0, y + 54, 'input, movement, shots and collisions in every frame', 10.5, DIM)
        elif r == 1:
            for k in range(30):
                s.rect(x0 + k * w / 30, y + 20, 4, 12, colour, rx=1, extra=' fill-opacity="0.45"')
            for k in (6, 17, 23):
                s.add(f'<path d="M{x0 + k * w / 30 + 2:.1f} {y + 2}L{x0 + k * w / 30 + 2:.1f} {y + 18}" stroke="{AMBER}" '
                      f'stroke-width="2" marker-end="url(#arrow)"/>')
            s.text(x0, y + 54, 'steady steps, with each click applied as a command at one step', 10.5, DIM)
        elif r == 2:
            for k in range(40):
                s.rect(x0 + k * w / 40, y + 22, 3, 8, colour, rx=1, extra=' fill-opacity="0.35"')
            for k, label in ((10, 'enter area'), (25, 'seen'), (33, 'talk')):
                s.rect(x0 + k * w / 40 - 4, y + 12, 12, 28, RED if label != 'talk' else AMBER, rx=2, extra=' fill-opacity="0.7"')
                s.text(x0 + k * w / 40 + 2, y + 8, label, 9.5, DIM, 'middle')
            s.text(x0, y + 54, 'free movement, with triggers on areas and sight, and choices on use', 10.5, DIM)
        else:
            for k, (a, b) in enumerate(((60, 120), (250, 300), (470, 540))):
                s.rect(x0 + a, y + 14, b - a, 24, colour, rx=3, extra=' fill-opacity="0.5"')
                s.text(x0 + (a + b) / 2, y + 30, 'resolve', 10, TEXT, 'middle')
            s.text(x0, y + 54, 'nothing changes until the player moves', 10.5, DIM)
    s.add(f'<rect y="84" width="2" height="280" fill="{AMBER}"><animate attributeName="x" values="{x0};{x0 + w}" dur="{dur}s" '
          f'repeatCount="indefinite"/></rect>')
    legend(s, 250, 384, [(AMBER, 'the player\'s action'), (RED, 'the world acts on the player')])
    return s.render()


# ---------------------------------------------------------------- interaction: mixed games
def mixed_modes():
    s = Svg(900, 300, 'X-COM alternates a real-time world map with turn-based battles. Frontier mixes real-time flight with docked screens that wait. In each game, a state variable switches between the modes')
    s.heading('Games that switch between modes',
              'A state variable switches the game from one mode to another. Treat each mode as the kind of game it is.')
    for k, (game, a, ka, b, kb) in enumerate((('X-COM', 'world map', 'real time', 'battle', 'turns'),
                                              ('Frontier: Elite II', 'flight', 'real time, a stream', 'docked screens',
                                               'software that waits'),
                                              ('A shooter', 'levels', 'a stream', 'menus and shop', 'software that waits'))):
        y = 90 + k * 66
        s.text(24, y + 30, game, 12.5, TEXT, weight='600')
        box(s, 200, y + 6, 220, 44, TEAL, a, ka, op=0.22)
        box(s, 600, y + 6, 240, 44, AMBER, b, kb, op=0.22)
        s.add(f'<path d="M424 {y + 20}L594 {y + 20}" stroke="{DIM}" stroke-width="1.5" marker-end="url(#arrow)"/>')
        s.add(f'<path d="M596 {y + 38}L426 {y + 38}" stroke="{DIM}" stroke-width="1.5" marker-end="url(#arrow)"/>')
        s.text(510, y + 32, 'mode variable', 10, DIM, 'middle', mono=True)
    return s.render()


# ---------------------------------------------------------------- arcade: an event script (animated)
def arcade_events():
    s = Svg(900, 360, 'The event script of a shoot \'em up level. Each record fires at a scroll position, starting waves and changing speed. Actors live in slots that are updated in slot order')
    s.heading('The event script of an arcade level',
              'Each record fires when the screen scrolls to its position. Each new actor takes the first free slot.')
    dur = 8.0
    events = [(120, 'wave 1, 5 fighters'), (300, 'speed up'), (420, 'wave 2, turrets'), (600, 'boss music'),
              (700, 'boss')]
    x0, w = 40, 760
    s.rect(x0, 92, w, 40, GREY, rx=3, extra=' fill-opacity="0.15"')
    for pos, label in events:
        x = x0 + pos
        s.line(x, 88, x, 136, AMBER, 1.5)
        s.text(x + 4, 104, f'{pos}', 10, AMBER, mono=True)
        s.text(x + 4, 124, label, 10.5, TEXT)
    s.text(x0, 150, 'scroll position', 10.5, DIM)
    s.add(f'<rect y="84" width="3" height="56" fill="{TEAL}"><animate attributeName="x" values="{x0};{x0 + w}" dur="{dur}s" '
          f'repeatCount="indefinite"/></rect>')
    # Slots fill as waves fire.
    s.text(40, 188, 'actor slots, updated and drawn in slot order', 12, TEXT, weight='600')
    fills = [('player', 0, TEAL)] + [(f'fighter', 120 / w, RED)] * 5 + [('turret', 420 / w, PURPLE)] * 3 + \
            [('boss', 700 / w, AMBER)]
    for k in range(16):
        x = 40 + k * 52
        s.rect(x, 200, 48, 40, GREY, rx=3, extra=' fill-opacity="0.15"')
        s.text(x + 24, 256, str(k), 10, DIM, 'middle', mono=True)
        if k < len(fills):
            name, t0, colour = fills[k]
            body = scratch(lambda t, x=x, name=name, colour=colour: box(t, x, 200, 48, 40, colour, name, op=0.4, size=9.5))
            s.add(group(body, t0, 0.98, dur) if t0 > 0 else body)
    lines(s, 40, 286, ['A new actor takes the first free slot. The slot order decides who is drawn on top and who is hit first.',
                       'Read the event format and the weapon and enemy tables before porting the code that uses them.'],
          11.5, DIM, gap=18)
    s.text(40, 340, 'The frame order, positions in the game\'s units and every random draw must be exact.', 12, TEXT)
    return s.render()


# ---------------------------------------------------------------- first-person: per-tick commands (animated)
def fps_tics():
    s = Svg(900, 330, 'In first-person games, mouse movement adds up between ticks. At each tick the game reads the total and builds one command of forward move, side move, turn and buttons, which a demo stores')
    s.heading('One command per tick in a first-person game',
              'Mouse movement adds up between ticks. At each tick, the game turns the total into a command.')
    dur = 4.0
    x0, w, ticks = 130, 740, 6
    tw = w / ticks
    s.text(24, 116, 'mouse moves', 11, DIM)
    for k in range(ticks):
        for j in range(5):
            x = x0 + k * tw + 12 + j * tw / 6
            s.line(x, 120, x + 8, 108 - (j % 3) * 4, BLUE, 2)
        s.line(x0 + k * tw, 100, x0 + k * tw, 220, AMBER, 1.5)
        s.text(x0 + k * tw + 4, 232, f'tick {k}', 10, AMBER, mono=True)
    s.text(24, 172, 'frames drawn', 11, DIM)
    for k in range(int(w / 37) - 1):
        s.rect(x0 + k * 37 + 4, 160, 30, 14, GREY, rx=2, extra=' fill-opacity="0.35"')
    s.text(24, 204, 'commands', 11, DIM)
    for k in range(1, ticks):
        x = x0 + k * tw
        body = scratch(lambda t, x=x: box(t, x + 4, 186, 116, 26, TEAL, 'fwd, side, turn, btn', op=0.35, size=9.5))
        s.add(group(body, (k - 0.02) / ticks, 0.99, dur))
    s.add(f'<rect y="100" width="2" height="120" fill="{TEXT}"><animate attributeName="x" values="{x0};{x0 + w}" dur="{dur}s" '
          f'repeatCount="indefinite"/></rect>')
    lines(s, 40, 262, ['Doom ticks 35 times a second. The renderer draws as often as it can. The ticks must match the original exactly.',
                       'A demo stores these commands. Playing them back runs the same game, if the thinkers update in the same order.'],
          11.5, DIM, gap=18)
    s.text(40, 314, 'Movement (acceleration, friction, step-up, sliding) is in the game\'s units, often 16.16 fixed point.', 12, TEXT)
    return s.render()


# ---------------------------------------------------------------- management: steps, speeds and slices (animated)
def management_steps():
    s = Svg(900, 360, 'In a management game, the game speed sets how many fixed steps run per frame. Some systems run every n steps. Work that is spread over steps updates one slice of the map each step')
    s.heading('Steps, speeds and slices in a management game',
              'The game speed sets how many steps run in a frame. Some work is spread over many steps.')
    dur = 5.0
    for k, (speed, n) in enumerate((('speed 1', 1), ('speed 2', 2), ('speed 3', 4))):
        y = 92 + k * 34
        s.text(24, y + 17, speed, 11.5, TEXT)
        for f in range(6):
            s.rect(110 + f * 70, y, 66, 24, GREY, rx=3, extra=' fill-opacity="0.15"')
            for j in range(n):
                s.rect(112 + f * 70 + j * 64 / n, y + 3, 64 / n - 3, 18, TEAL, rx=2, extra=' fill-opacity="0.55"')
    s.text(110, 206, 'frames, each with 1, 2 or 4 simulation steps', 10.5, DIM)
    # The map in slices.
    mx, my = 580, 88
    s.text(mx, my, 'the map, a tenth each step', 11.5, TEXT, weight='600')
    for k in range(10):
        s.rect(mx, my + 10 + k * 14, 290, 12, GREEN, rx=2, extra=' fill-opacity="0.15"')
    s.add(f'<rect x="{mx}" width="290" height="12" rx="2" fill="{AMBER}" fill-opacity="0.7">'
          f'<animate attributeName="y" values="{";".join(str(my + 10 + k * 14) for k in range(10))}" '
          f'keyTimes="{";".join(f"{k / 10:.2f}" for k in range(10))}" calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.text(mx, my + 166, 'keep the slice order', 10.5, DIM)
    # Systems every n steps.
    s.text(24, 240, 'systems', 11.5, TEXT, weight='600')
    systems = [('agents', 1), ('growth', 4), ('births and deaths', 10)]
    for k, (name, every) in enumerate(systems):
        y = 252 + k * 22
        s.text(24, y + 12, f'{name}, every {every}', 10.5, DIM)
        for st in range(30):
            on = st % every == 0
            s.rect(170 + st * 12, y, 10, 14, BLUE if on else GREY, rx=1, extra=f' fill-opacity="{0.6 if on else 0.12}"')
    s.text(170, 334, 'steps (illustrative periods)', 10.5, DIM)
    lines(s, 580, 270, ['The same seed and the same commands', 'must give the same world. Commands', 'apply at a known step.'],
          11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- strategy: lockstep (animated)
def lockstep():
    s = Svg(900, 330, 'In lockstep multiplayer, each command is tagged with the tick at which it takes effect and is sent to every player. Every machine runs the same commands at the same tick')
    s.heading('Lockstep commands in a strategy game',
              'Only commands travel between the machines. Each command carries the tick at which it takes effect.')
    dur = 6.0
    x0, w, n = 160, 700, 10
    for row, (name, y) in enumerate((('player 1', 110), ('player 2', 230))):
        s.text(24, y + 18, name, 12, TEXT, weight='600')
        for k in range(n):
            s.rect(x0 + k * w / n + 2, y, w / n - 4, 28, BLUE, rx=3, extra=' fill-opacity="0.18"')
            s.text(x0 + k * w / n + w / n / 2, y + 18, f'tick {k}', 10, DIM, 'middle', mono=True)
    # A command issued by player 1 at tick 2 for tick 5.
    for target_y in (110, 230):
        s.add(group(scratch(lambda t, y=target_y: t.rect(x0 + 5 * w / n + 2, y, w / n - 4, 28, AMBER, rx=3,
                                                        extra=' fill-opacity="0.6"')), 0.5, 0.98, dur))
    s.add(f'<circle r="8" fill="{AMBER}"><animateMotion path="M{x0 + 2.5 * w / n} 124L{x0 + 4 * w / n} 190L{x0 + 5.5 * w / n} 244" '
          f'keyPoints="0;0;1;1" keyTimes="0;0.2;0.5;1" calcMode="linear" dur="{dur}s" repeatCount="indefinite"/></circle>')
    s.text(x0 + 4 * w / n, 180, 'the network carries only commands', 10.5, DIM, 'middle')
    s.text(x0 + 2.5 * w / n, 100, 'issued at tick 2 for tick 5, "move unit 14 to (40, 22)"', 11, AMBER, 'middle')
    s.text(x0 + 5.5 * w / n, 280, 'both run it at tick 5', 11, AMBER, 'middle')
    s.add(f'<rect y="104" width="2" height="160" fill="{TEXT}"><animate attributeName="x" values="{x0};{x0 + w}" dur="{dur}s" '
          f'repeatCount="indefinite"/></rect>')
    s.text(24, 314, 'Nothing per-player (the local clock, settings, the camera) may reach the simulation. A replay stores the same commands.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- RPG: a cosmetic draw
def rpg_draws():
    s = Svg(900, 300, 'In a role-playing game, one generator serves every draw. A draw spent on a cosmetic idle animation moves every later draw, so the to-hit and damage rolls change')
    s.heading('How a cosmetic random draw changes a fight',
              'The game uses one generator for every draw. A port that leaves out the draw for an idle animation changes every later draw.')
    rows = [('the original', [('idle animation', PURPLE, '#1'), ('to-hit roll', TEAL, '#2: hits'), ('damage roll', TEAL, '#3: 7'),
                              ('loot', TEAL, '#4: sword')]),
            ('a port without the idle draw', [('to-hit roll', RED, '#1: misses'), ('damage roll', RED, 'skipped'),
                                              ('loot', RED, '#2: gold')])]
    for r, (name, cells) in enumerate(rows):
        y = 92 + r * 80
        s.text(24, y + 26, name, 12, TEXT, weight='600')
        for k, (label, colour, val) in enumerate(cells):
            box(s, 250 + k * 156, y + 4, 148, 44, colour, label, val, op=0.25, size=11.5)
    lines(s, 24, 262, ['The values here are illustrative. Record which draws the original makes, in order, including the ones that only',
                       'change the look.'], 11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- adventure: the interpreter (animated)
def adventure_interpreter():
    s = Svg(900, 340, 'An adventure game interpreter reads the next opcode from a script, looks it up in a dispatch table and runs one handler that changes variables, flags and objects')
    s.heading('The opcode loop of an adventure game',
              'The executable reads the scripts one opcode at a time and runs a handler for each opcode.')
    dur = 6.0
    ops = [('op A', '2 operands'), ('op B', '1 operand'), ('op C', 'a jump'), ('op D', 'wait for input')]
    x0 = 40
    s.text(x0, 92, 'a script, as bytes (illustrative)', 11, DIM)
    xs = []
    x = x0
    for k, (op, desc) in enumerate(ops):
        n = 3 if k == 0 else (2 if k == 1 else (3 if k == 2 else 1))
        for j in range(n):
            s.rect(x + j * 34, 102, 32, 28, AMBER if j == 0 else GREY, rx=3, extra=f' fill-opacity="{0.4 if j == 0 else 0.2}"')
        s.text(x + 16, 121, op[-1], 11, TEXT, 'middle', '600', mono=True)
        xs.append(x)
        x += n * 34 + 8
    s.add(f'<rect y="98" width="36" height="36" rx="4" fill="none" stroke="{TEAL}" stroke-width="2.5">'
          f'<animate attributeName="x" values="{";".join(str(v - 2) for v in xs)}" keyTimes="0;0.25;0.5;0.75" '
          f'calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/></rect>')
    box(s, 40, 170, 180, 60, TEAL, 'fetch the opcode', 'and its operands', op=0.22)
    arrow(s, 222, 200, 262, 200)
    s.rect(266, 160, 220, 120, PURPLE, rx=4, extra=' fill-opacity="0.15"')
    s.text(278, 180, 'dispatch table', 12, TEXT, weight='600')
    for k, (op, desc) in enumerate(ops):
        y = 192 + k * 20
        s.text(278, y + 10, f'{op[-1]}  ->  handler_{op[-1].lower()}', 10.5, TEXT, mono=True)
        s.add(f'<rect x="272" y="{y - 3}" width="208" height="18" rx="3" fill="{AMBER}" opacity="{1 if k == 0 else 0}" '
              f'fill-opacity="0.25">{during(k / 4, (k + 1) / 4, dur)}</rect>')
    arrow(s, 488, 220, 528, 220)
    box(s, 532, 170, 344, 100, BLUE, None, op=0.15)
    s.text(546, 192, 'state the handlers change', 12, TEXT, weight='600')
    lines(s, 546, 214, ['variables and flags', 'objects, where they are and who holds them', 'rooms, actors, the cursor'],
          11, DIM, gap=18)
    s.text(40, 316, 'Find the dispatch table first. Check that your decoder consumes every byte of every script file.', 12, TEXT)
    return s.render()


# ---------------------------------------------------------------- racing: a small difference grows
def racing_divergence():
    s = Svg(900, 330, 'In racing physics, a tiny difference at the first tick grows quickly. The distance between the original\'s car and the port\'s car rises steeply over the ticks')
    s.heading('Why racing games need checks from the first tick',
              'A difference in the last bit of a value grows every tick. Compare from the first tick and stop at the first difference.')
    x0, y0, w, h = 80, 90, 520, 200
    s.line(x0, y0 + h, x0 + w, y0 + h, DIM, 1)
    s.line(x0, y0, x0, y0 + h, DIM, 1)
    s.text(x0 + w, y0 + h + 18, 'ticks', 10.5, DIM, 'end')
    s.text(x0 + 8, y0 + 8, 'distance between the two cars', 10.5, DIM)
    pts = []
    for k in range(101):
        t = k / 100
        v = (math.exp(t * 6) - 1) / (math.exp(6) - 1)
        pts.append(f'{x0 + t * w:.1f} {y0 + h - v * h:.1f}')
    s.add(f'<path d="M{"L".join(pts)}" fill="none" stroke="{RED}" stroke-width="2.5"/>')
    s.add(f'<circle cx="{x0 + 4}" cy="{y0 + h - 1}" r="5" fill="{AMBER}"/>')
    s.text(x0 + 14, y0 + h - 10, 'tick 1, a difference in the last bit of one float', 10.5, AMBER)
    s.text(x0 + w - 70, y0 + 14, 'a different line', 10.5, RED, 'end')
    s.text(x0 + w - 70, y0 + 30, 'through the corner', 10.5, RED, 'end')
    s.text(x0, y0 + h + 34, 'The shape is illustrative.', 10.5, DIM)
    lines(s, 640, 100, ['Causes to check first:', '- x87 versus SSE rounding', '- the order of integration', '- grip curves that were not read',
                        '  from the tables', '- collision against track', '  segments in a different order', '',
                        'Replays are recorded inputs', 'plus the seed, so the physics', 'must repeat exactly.'],
          11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- vehicle sims: a world from a seed
def vehicle_seed():
    s = Svg(900, 320, 'A generated world is stored only as a seed. The generator draws numbers in order to make each star system, so one wrong draw changes that system and every one after it')
    s.heading('A world generated from a seed',
              'The game stores only the seed. If one draw differs, every system generated after it also differs.')
    box(s, 24, 110, 110, 44, AMBER, 'seed', op=0.35)
    arrow(s, 136, 132, 166, 132)
    for row, (name, bad) in enumerate((('the original', None), ('one wrong draw', 2))):
        y = 92 + row * 86
        s.text(170, y - 2, name, 11, DIM)
        for k in range(6):
            colour = RED if (bad is not None and k >= bad) else TEAL
            box(s, 170 + k * 116, y + 6, 108, 50, colour, f'system {k + 1}', 'name, star, planets', op=0.25, size=11.5)
    s.text(170 + 2 * 116 + 54, 270, 'from here on, every name differs', 10.5, RED, 'middle')
    lines(s, 24, 296, ['Emulate the generator on many seeds and compare every field, including the names.'], 12, TEXT)
    return s.render()


# ---------------------------------------------------------------- puzzle: a bag and a gravity table
def puzzle_bag():
    s = Svg(900, 330, 'A seven-piece bag deals each piece once before it is refilled. Gravity in NES Tetris is a table of frames per row for each level')
    s.heading('Generators and frame tables in puzzle games',
              'Players notice small changes in these rules, so the port must use the same values.')
    pieces = ['I', 'O', 'T', 'S', 'Z', 'J', 'L']
    colours = {'I': TEAL, 'O': AMBER, 'T': PURPLE, 'S': GREEN, 'Z': RED, 'J': BLUE, 'L': '#fb923c'}
    import random
    rnd = random.Random(3)
    s.text(24, 92, 'A seven-piece bag deals each piece once in a shuffled order, then starts a new bag', 12, TEXT, weight='600')
    x = 24
    for b in range(3):
        bag = pieces[:]
        rnd.shuffle(bag)
        s.rect(x - 4, 102, 7 * 34 + 4, 42, GREY, rx=4, extra=' fill-opacity="0.12"')
        for k, p in enumerate(bag):
            box(s, x + k * 34, 106, 30, 34, colours[p], p, op=0.45, size=13, mono=True)
        x += 7 * 34 + 14
    s.text(24, 164, 'Later Tetris games use a bag. Other games use plain random draws or an anti-repeat rule.', 10.5, DIM)
    s.text(24, 196, 'Gravity in NES Tetris, in frames for a piece to fall one row', 12, TEXT, weight='600')
    table = [48, 43, 38, 33, 28, 23, 18, 13, 8, 6]
    for k, v in enumerate(table):
        x = 24 + k * 64
        hh = v * 1.3
        s.rect(x, 296 - hh, 54, hh, BLUE, rx=3, extra=' fill-opacity="0.45"')
        s.text(x + 27, 292 - hh, str(v), 11, TEXT, 'middle', '600', mono=True)
        s.text(x + 27, 312, f'level {k}', 10, DIM, 'middle')
    lines(s, 680, 228, ['At 60 frames a second,', 'level 0 drops a row every', '0.8 seconds.'], 11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- online: client and server (animated)
def online_split():
    s = Svg(900, 330, 'In an online game, the client sends input, predicts and shows results. The server holds the authority and sends back the true state, which the client follows')
    s.heading('How an online game splits its rules',
              'The client shows a predicted result at once and then follows the result that the server sends.')
    dur = 4.0
    box(s, 24, 100, 300, 170, BLUE, None, op=0.14)
    s.text(40, 124, 'the client', 13, TEXT, weight='600')
    lines(s, 40, 148, ['the interface and rendering', 'prediction, which moves at once', 'some formulas, to show numbers',
                       'data tables the server also uses'], 11.5, DIM, gap=20)
    box(s, 576, 100, 300, 170, AMBER, None, op=0.14)
    s.text(592, 124, 'the server', 13, TEXT, weight='600')
    lines(s, 592, 148, ['the authority on what', 'really happened', 'server ticks', 'known only from captured',
                        'traffic, marked inferred'], 11.5, DIM, gap=20)
    s.line(330, 150, 570, 150, DIM, 1.5, ' marker-end="url(#arrow)"')
    s.line(570, 220, 330, 220, DIM, 1.5, ' marker-end="url(#arrow)"')
    s.text(450, 142, 'input, "attack target"', 10.5, TEXT, 'middle')
    s.text(450, 240, 'result, "hit for 12"', 10.5, TEXT, 'middle')
    s.add(f'<circle r="6" fill="{TEAL}"><animateMotion path="M330 150L570 150" keyPoints="0;1;1" keyTimes="0;0.4;1" '
          f'calcMode="linear" dur="{dur}s" repeatCount="indefinite"/></circle>')
    s.add(f'<circle r="6" fill="{AMBER}"><animateMotion path="M570 220L330 220" keyPoints="0;0;1;1" keyTimes="0;0.5;0.9;1" '
          f'calcMode="linear" dur="{dur}s" repeatCount="indefinite"/></circle>')
    s.text(24, 300, 'Record traffic only from your own sessions, on servers you may use. Keep other players\' data private.', 12, TEXT)
    return s.render()


DIAGRAMS = {
    'games-connections.svg': connections, 'games-approaches.svg': approaches, 'games-own-clock.svg': own_clock,
    'games-waits.svg': waits, 'games-four-kinds.svg': four_kinds, 'games-mixed-modes.svg': mixed_modes,
    'games-arcade-events.svg': arcade_events, 'games-fps-tics.svg': fps_tics,
    'games-management-steps.svg': management_steps, 'games-lockstep.svg': lockstep, 'games-rpg-draws.svg': rpg_draws,
    'games-adventure-interpreter.svg': adventure_interpreter, 'games-racing-divergence.svg': racing_divergence,
    'games-vehicle-seed.svg': vehicle_seed, 'games-puzzle-bag.svg': puzzle_bag, 'games-online-split.svg': online_split,
}
