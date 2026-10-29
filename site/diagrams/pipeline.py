"""The rendering pipeline's diagrams, for docs/hardware/rendering-pipeline.md: the five stages of
a frame, where each stage ran on the machines of each era, what each seam lets a port check,
the port drawn as a card with devices behind one interface, and a frame that a card and the
CPU draw together. make.py loads DIAGRAMS from here.
"""
from svg import AMBER, BLUE, DIM, EDGE, GREEN, GREY, PURPLE, RED, TEAL, TEXT, Svg

STAGES = [('1', 'game state', 'objects, camera,', 'what is visible'),
          ('2', 'geometry', 'transform, light,', 'project, clip, sort'),
          ('3', 'the seam', 'the calls that hand', 'the frame over'),
          ('4', 'rasterising', 'triangles to pixels:', 'texture, blend, fog'),
          ('5', 'display', 'palette or 16-bit,', 'gamma, page flip')]


def _arrow(s, x1, y1, x2, y2, colour=DIM, width=1.5):
    s.line(x1, y1, x2, y2, colour, width, ' marker-end="url(#arrow)"')



# ---------------------------------------------------------------- the five stages (animated)
def pipeline_stages():
    s = Svg(900, 330, 'The five stages of a frame: game state, geometry, the seam, rasterising and display. A '
                      'highlight steps through them while the label below says what passes between them')
    s.heading('The five stages of a frame', 'Every game of the period draws a frame through these stages, '
                                            'whatever card it runs on.')
    where = [('CPU', TEAL), ('CPU', TEAL), ('function calls', AMBER), ('CPU or card', PURPLE), ('card', BLUE)]
    for k, (num, name, a, b) in enumerate(STAGES):
        x = 30 + k * 174
        colour = where[k][1]
        s.rect(x, 86, 156, 96, colour, rx=6, extra=' fill-opacity="0.2"')
        s.text(x + 14, 108, num, 12, colour, weight='700', mono=True)
        s.text(x + 78, 128, name, 14, TEXT, 'middle', '600')
        s.text(x + 78, 150, a, 10, DIM, 'middle')
        s.text(x + 78, 164, b, 10, DIM, 'middle')
        s.rect(x, 192, 156, 22, colour, rx=4, extra=' fill-opacity="0.45"')
        s.text(x + 78, 207, where[k][0], 11, TEXT, 'middle', '600')
        if k:
            _arrow(s, x - 17, 134, x - 2, 134)
    flows = ['objects and a camera', 'screen-space vertices with 1/w', 'draw calls and texture uploads',
             'pixels in a frame buffer', 'a picture on the monitor']
    dur, n = 7.5, len(flows)
    s.add(f'<rect x="27" y="83" width="162" height="134" rx="8" fill="none" stroke="{AMBER}" stroke-width="2.5">'
          f'<animateTransform attributeName="transform" type="translate" '
          f'values="{";".join(f"{k * 174} 0" for k in range(n))}" calcMode="discrete" dur="{dur}s" '
          f'repeatCount="indefinite"/></rect>')
    s.text(450, 250, 'passes on:', 11, DIM, 'middle')
    for k, flow in enumerate(flows):
        a, b = k / n, (k + 1) / n
        keys = f'0;{a:.3f};{a:.3f};{b:.3f};{b:.3f};1'
        s.add(f'<text x="450" y="274" font-family="ui-sans-serif, -apple-system, \'Segoe UI\', Helvetica, Arial, '
              f'sans-serif" font-size="15" font-weight="600" fill="{AMBER}" text-anchor="middle" opacity="0">'
              f'{flow}<animate attributeName="opacity" values="0;0;1;1;0;0" keyTimes="{keys}" dur="{dur}s" '
              f'repeatCount="indefinite"/></text>')
    s.text(450, 306, 'The cards of 1996 to 1998 did stage 4 only. Stages 1 and 2 stayed on the CPU, in the '
                     'game\'s own code.', 11, DIM, 'middle')
    return s.render()


# ---------------------------------------------------------------- where each stage ran, by era
ERAS = [
    ('Software renderer', '1993-1998', ['CPU', 'CPU', 'CPU', 'VGA'],
     'the game\'s own spans or polygons, inside one program'),
    ('S3 ViRGE, ATI Rage', '1995-1997', ['CPU', 'CPU', 'split', 'card'],
     'S3D Toolkit or ATI3DCIF faces; the CPU draws the rest'),
    ('3dfx Voodoo (Glide)', '1996-1998', ['CPU', 'CPU', 'card', 'card'],
     'grDrawTriangle: screen x, y and 1/w per vertex'),
    ('PowerVR (SGL)', '1997', ['CPU', 'driver', 'card', 'card'],
     'a scene graph the library transforms, or projected faces'),
    ('Direct3D 3 to 6', '1996-1998', ['CPU', 'CPU', 'card', 'card'],
     'execute buffers, then DrawPrimitive; transformed or not'),
    ('Direct3D 7, GeForce 256', '1999', ['CPU', 'card', 'card', 'card'],
     'untransformed vertices, matrices and lights'),
    ('Shaders (Direct3D 8 on)', '2001-', ['CPU', 'program', 'program', 'card'],
     'vertex buffers and compiled shader programs'),
]
WHO = {'CPU': TEAL, 'VGA': GREY, 'card': BLUE, 'split': RED, 'driver': PURPLE, 'program': GREEN}


def pipeline_eras():
    s = Svg(900, 420, 'Where each stage of the frame ran: software renderers did everything on the CPU; the cards '
                      'of 1995 to 1998 took rasterising; the GeForce 256 took geometry; shaders made both '
                      'programmable')
    s.heading('Where each stage ran', 'The work moved from the CPU to the card one stage at a time. The seam '
                                      'moved with it.')
    cols = [(206, 62, '1 state'), (274, 72, '2 geometry'), (704, 80, '4 raster'), (792, 80, '5 display')]
    s.text(24, 92, 'machine', 11, DIM, weight='600')
    s.text(358, 92, '3  what crosses the seam', 11, AMBER, weight='600')
    for x, w, name in cols:
        s.text(x + w / 2, 92, name, 11, DIM, 'middle', '600')
    for r, (name, years, cells, seam) in enumerate(ERAS):
        y = 104 + r * 40
        s.rect(20, y, 860, 34, EDGE, rx=4, extra=' fill-opacity="0.35"')
        s.text(30, y + 15, name, 12, TEXT, weight='600')
        s.text(30, y + 28, years, 10, DIM, mono=True)
        for (x, w, _), who in zip(cols, cells):
            s.rect(x, y + 5, w, 24, WHO[who], rx=4, extra=' fill-opacity="0.5"')
            s.text(x + w / 2, y + 21, who, 11, TEXT, 'middle', '600')
        s.text(358, y + 21, seam, 11, TEXT)
    for k, (who, note) in enumerate([('CPU', 'the game\'s code'), ('driver', 'a library on the CPU'),
                                     ('split', 'some faces each'), ('card', 'fixed hardware'),
                                     ('program', 'a shader')]):
        x = 30 + k * 170
        s.rect(x, 392, 14, 14, WHO[who], rx=3, extra=' fill-opacity="0.6"')
        s.text(x + 20, 403, f'{who}: {note}', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- what each seam checks
def pipeline_checks():
    s = Svg(900, 360, 'Three seams and what each lets a port check: the numbers after geometry, the call stream at '
                      'the seam, and the pixels after rasterising, which are exact against a software renderer or '
                      'a chip model and close against a modern GPU')
    s.heading('What each seam lets you check', 'Each seam is a place to compare the port with the original. '
                                               'Each means something different by exact.')
    for k, (num, name, _, _) in enumerate(STAGES):
        x = 30 + k * 174
        s.rect(x, 78, 156, 40, [TEAL, TEAL, AMBER, PURPLE, BLUE][k], rx=6, extra=' fill-opacity="0.22"')
        s.text(x + 78, 103, f'{num}  {name}', 12, TEXT, 'middle', '600')
        if k:
            _arrow(s, x - 17, 98, x - 2, 98)
    checks = [
        (195, 'A  numbers', TEAL, 'bit-exact', ['vertices, matrices, the sort order', 'against the emulator (re-emulate,',
                                                'bitcompare.py)']),
        (456, 'B  the call stream', AMBER, 'call for call', ['every call and argument, in order', 'against a recording '
                                                             'made by the', 'replacement DLLs']),
        (717, 'C  pixels', PURPLE, 'depends on the device', ['software reference: exact', 'chip model: exact against '
                                                             'the model', 'modern GPU: within a tolerance']),
    ]
    for x, name, colour, verdict, lines in checks:
        s.line(x, 120, x, 152, colour, 2, ' stroke-dasharray="4 3"')
        s.add(f'<circle cx="{x}" cy="120" r="5" fill="{colour}"/>')
        s.rect(x - 125, 156, 250, 150, colour, rx=6, extra=' fill-opacity="0.14"')
        s.text(x, 180, name, 13, colour, 'middle', '700')
        s.text(x, 202, verdict, 12, TEXT, 'middle', '600')
        for j, line in enumerate(lines):
            s.text(x, 230 + j * 18, line, 11, DIM, 'middle')
    s.text(450, 334, 'Check the earliest seam first: a wrong number at A shows as wrong calls at B and wrong '
                     'pixels at C.', 11, DIM, 'middle')
    return s.render()


# ---------------------------------------------------------------- the port as a virtual card
def pipeline_devices():
    s = Svg(900, 400, 'The port drawn as a graphics card: an exact core runs stages 1 to 3 and calls one device '
                      'interface; behind it a software device, card profiles and a modern device; hooks at the '
                      'interface record, replace and add; the vanilla build ships the reference devices and the '
                      'extra build adds the rest')
    s.heading('The port as a virtual graphics card', 'Stages 1 to 3 are ported exactly. Stage 4 is a device you '
                                                     'can swap.')
    s.rect(30, 110, 230, 190, TEAL, rx=8, extra=' fill-opacity="0.18"')
    s.text(145, 138, 'the exact core', 14, TEXT, 'middle', '600')
    for j, line in enumerate(['1 game state', '2 geometry', '3 the calls of a frame']):
        s.rect(50, 156 + j * 42, 190, 32, TEAL, rx=4, extra=' fill-opacity="0.3"')
        s.text(145, 177 + j * 42, line, 12, TEXT, 'middle')
    s.rect(330, 96, 36, 218, AMBER, rx=6, extra=' fill-opacity="0.45"')
    s.text(348, 205, 'device interface', 12, TEXT, 'middle', '600', extra=' transform="rotate(-90 348 205)"')
    _arrow(s, 262, 205, 326, 205, AMBER, 2)
    devices = [('software device', 'the game\'s own rasteriser, ported exactly', PURPLE, 'vanilla'),
               ('card profiles', 'gameframe-voodoo, -rage, -virge, -powervr', BLUE, 'vanilla'),
               ('modern device', 'wgpu or OpenGL, any size, filtering, widescreen', GREEN, 'extra')]
    for k, (name, note, colour, build) in enumerate(devices):
        y = 100 + k * 72
        s.rect(430, y, 320, 60, colour, rx=6, extra=' fill-opacity="0.22"')
        s.text(446, y + 24, name, 13, TEXT, weight='600')
        s.text(446, y + 44, note, 11, DIM)
        s.text(764, y + 35, build, 12, colour, weight='600')
        _arrow(s, 368, 205, 426, y + 30, DIM, 1.2)
    hooks = ['record the calls', 'replace textures', 'add draws (mods)']
    for k, hook in enumerate(hooks):
        x = 286 + k * 130
        s.rect(x - 56, 330, 112, 28, GREEN, rx=4, extra=' fill-opacity="0.2"')
        s.text(x, 349, hook, 11, TEXT, 'middle')
    s.line(348, 316, 348, 328, GREEN, 1.5)
    s.text(30, 349, 'extra build hooks:', 11, GREEN, weight='600')
    s.text(30, 382, 'Every device and hook takes the same calls, so a recording, a test or a mod works whichever '
                    'device draws.', 11, DIM)
    return s.render()


# ---------------------------------------------------------------- a frame drawn by card and CPU (animated)
def pipeline_mixed_frame():
    s = Svg(900, 340, 'One frame drawn by the card and the CPU together: the card clears and draws the 3D scene, '
                      'the CPU locks the frame buffer and writes the interface into it, then the buffers swap')
    s.heading('A frame the card and the CPU draw together', 'A Glide game often draws its 3D on the card and its '
                                                            'interface with the CPU, into the same frame buffer.')
    steps = [('card', BLUE, 'grBufferClear', 'clear colour and depth'),
             ('card', BLUE, 'grDrawTriangle  x N', 'the 3D scene, textured and fogged'),
             ('CPU', TEAL, 'grLfbLock ... grLfbUnlock', 'the interface, written as pixels'),
             ('card', BLUE, 'grBufferSwap', 'show the finished frame')]
    dur, n = 8.0, len(steps)
    for k, (who, colour, call, note) in enumerate(steps):
        y = 84 + k * 56
        s.rect(30, y, 420, 46, colour, rx=6, extra=' fill-opacity="0.18"')
        s.text(46, y + 20, call, 12, TEXT, weight='600', mono=True)
        s.text(46, y + 37, note, 11, DIM)
        s.text(436, y + 28, who, 11, colour, 'end', '700')
    s.add(f'<rect x="27" y="81" width="426" height="52" rx="8" fill="none" stroke="{AMBER}" stroke-width="2.5">'
          f'<animateTransform attributeName="transform" type="translate" '
          f'values="{";".join(f"0 {k * 56}" for k in range(n))}" calcMode="discrete" dur="{dur}s" '
          f'repeatCount="indefinite"/></rect>')
    sx, sy, sw, sh = 520, 84, 340, 214
    s.rect(sx, sy, sw, sh, '#05090a', EDGE, rx=4)

    def layer(k, body):
        a = k / n
        keys = f'0;{a:.3f};{a:.3f};1'
        s.add(f'<g opacity="{1 if k == 0 else 0}"><animate attributeName="opacity" values="0;0;1;1" '
              f'keyTimes="{keys}" dur="{dur}s" repeatCount="indefinite"/>{body}</g>')
    layer(0, f'<rect x="{sx + 2}" y="{sy + 2}" width="{sw - 4}" height="{sh - 4}" fill="#16313a"/>')
    ground = f'{sx + 2},{sy + sh - 2} {sx + sw - 2},{sy + sh - 2} {sx + sw / 2 + 30},{sy + 104} {sx + sw / 2 - 30},{sy + 104}'
    layer(1, f'<polygon points="{ground}" fill="{BLUE}" fill-opacity="0.55"/>'
             f'<polygon points="{sx + 80},{sy + 150} {sx + 140},{sy + 70} {sx + 190},{sy + 150}" fill="{PURPLE}" '
             f'fill-opacity="0.7"/><polygon points="{sx + 200},{sy + 140} {sx + 250},{sy + 90} {sx + 290},{sy + 140}" '
             f'fill="{GREEN}" fill-opacity="0.6"/>')
    layer(2, f'<rect x="{sx + 2}" y="{sy + sh - 34}" width="{sw - 4}" height="32" fill="{TEAL}" fill-opacity="0.85"/>'
             f'<rect x="{sx + 12}" y="{sy + sh - 26}" width="90" height="16" fill="#05090a"/>'
             f'<rect x="{sx + 14}" y="{sy + sh - 24}" width="62" height="12" fill="{AMBER}"/>'
             f'<rect x="{sx + sw - 70}" y="{sy + 10}" width="58" height="14" fill="{TEAL}" fill-opacity="0.85"/>')
    layer(3, f'<rect x="{sx}" y="{sy}" width="{sw}" height="{sh}" rx="4" fill="none" stroke="{AMBER}" '
             f'stroke-width="3"/>')
    s.text(sx + sw / 2, 318, 'Port each layer from its own code: the card calls and the CPU\'s pixel writes.',
           11, DIM, 'middle')
    return s.render()


DIAGRAMS = {
    'pipeline-stages.svg': pipeline_stages, 'pipeline-eras.svg': pipeline_eras,
    'pipeline-checks.svg': pipeline_checks, 'pipeline-devices.svg': pipeline_devices,
    'pipeline-mixed-frame.svg': pipeline_mixed_frame,
}
