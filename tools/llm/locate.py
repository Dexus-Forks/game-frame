#!/usr/bin/env python3
"""Find a screen element, or a reference screen, in a capture with a vision model.

Usage:
  locate.py NOW --find "the Exit button" [--units F]
  locate.py NOW --ref REF [--save FILE.json]

  NOW     a capture of what is shown now (PNG, JPEG, GIF or WebP)
  --find  the element to find, in words. Prints its centre in NOW's pixels, or "not found"
  --ref   a reference picture of the screen. Prints where it sits in NOW as an offset and a
          scale, like tools/live/align.py, and --save writes the same JSON, so
          `align.py map FILE.json X Y` gives click points from it
  --units divide the point given out by F (2 for a Retina capture's points)

The model gives positions as fractions of the picture's width and height, so they hold
whatever size the API scales the picture to. A position from a model is good to a few pixels.
For an exact offset, run tools/live/align.py first: it measures, and this tool guesses. Use
this one when align.py reports a weak match: another menu is open, the theme changed, or a
dialog covers part of the screen.

It uses the configured model API (client.py): one call with one or two pictures, about 2,000 to
4,000 input tokens. A picture must be under 5 MB. The selected model must accept images.

What it does not do: click. Give the point to winput or the VM's input tool yourself.
"""
import base64
import json
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from client import MODEL, api, text_of  # noqa: E402

TYPES = {b'\x89PNG': 'image/png', b'\xff\xd8\xff': 'image/jpeg', b'GIF8': 'image/gif', b'RIFF': 'image/webp'}
FIND = {'type': 'object', 'additionalProperties': False, 'required': ['found', 'x', 'y', 'note'],
        'properties': {'found': {'type': 'boolean'}, 'x': {'type': 'number'}, 'y': {'type': 'number'},
                       'note': {'type': 'string'}}}
PLACE = {'type': 'object', 'additionalProperties': False,
         'required': ['found', 'left', 'top', 'right', 'bottom', 'note'],
         'properties': {'found': {'type': 'boolean'}, 'left': {'type': 'number'}, 'top': {'type': 'number'},
                        'right': {'type': 'number'}, 'bottom': {'type': 'number'}, 'note': {'type': 'string'}}}


def picture(path):
    data = open(path, 'rb').read()
    kind = next((t for magic, t in TYPES.items() if data.startswith(magic)), None)
    if not kind:
        sys.exit(f'{path}: not a PNG, JPEG, GIF or WebP picture')
    if len(data) > 5 * 1024 * 1024:
        sys.exit(f'{path}: over 5 MB; save it as JPEG or crop it')
    return {'type': 'image', 'source': {'type': 'base64', 'media_type': kind,
                                        'data': base64.standard_b64encode(data).decode()}}


def size(path):
    data = open(path, 'rb').read(64 * 1024)
    if data.startswith(b'\x89PNG'):
        return struct.unpack_from('>II', data, 16)
    if data.startswith(b'GIF8'):
        return struct.unpack_from('<HH', data, 6)
    i = 2
    while data.startswith(b'\xff\xd8') and i < len(data) - 9:    # JPEG: the first SOF marker
        marker, n = data[i + 1], struct.unpack_from('>H', data, i + 2)[0]
        if 0xc0 <= marker <= 0xcf and marker not in (0xc4, 0xc8, 0xcc):
            h, w = struct.unpack_from('>HH', data, i + 5)
            return w, h
        i += 2 + n
    sys.exit(f'{path}: cannot read its size; give a PNG, GIF or JPEG')


def ask(content, schema, model):
    msg = api().messages.create(model=model, max_tokens=4000,
                                output_config={'format': {'type': 'json_schema', 'schema': schema}},
                                messages=[{'role': 'user', 'content': content}])
    print(f'({msg.usage.input_tokens} tokens in, {msg.usage.output_tokens} out, {model})', file=sys.stderr)
    return json.loads(text_of(msg))


def option(argv, name, default=None):
    if name in argv:
        i = argv.index(name)
        v = argv[i + 1]
        del argv[i:i + 2]
        return v
    return default


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__); return 0
    target, ref, save = option(argv, '--find'), option(argv, '--ref'), option(argv, '--save')
    units, model = float(option(argv, '--units', '1')), option(argv, '--model', MODEL)
    now = argv[0]
    nw, nh = size(now)
    if target:
        d = ask([picture(now), {'type': 'text', 'text':
                 f'This is a capture of a screen. Find {target}. Give the centre of it as fractions '
                 'of the picture: x from 0 (left edge) to 1 (right edge), y from 0 (top) to 1 '
                 '(bottom). If it is not there, set found to false. In note, say what you matched.'}],
                FIND, model)
        if not d['found']:
            print(f'not found: {d["note"]}')
            return 1
        print(f'{d["x"] * nw / units:.0f} {d["y"] * nh / units:.0f}   ({d["note"]})')
        return 0
    if ref:
        rw, rh = size(ref)
        d = ask([{'type': 'text', 'text': 'The first picture is a reference screen:'}, picture(ref),
                 {'type': 'text', 'text': 'The second is a capture that may show the same screen inside '
                  'a window, a frame or a larger desktop, at another size:'}, picture(now),
                 {'type': 'text', 'text': 'Where does the reference screen\'s content sit in the '
                  'second picture? Give the edges of its area as fractions of the second picture: '
                  'left and right from 0 to 1 across, top and bottom from 0 to 1 down. Leave out '
                  'window frames and title bars. If it is not there, set found to false.'}],
                PLACE, model)
        if not d['found']:
            print(f'not found: {d["note"]}')
            return 1
        sx = (d['right'] - d['left']) * nw / rw
        sy = (d['bottom'] - d['top']) * nh / rh
        a = {'ref': ref, 'now': now, 'ref_size': [rw, rh], 'now_size': [nw, nh],
             'scale_x': round(sx, 5), 'scale_y': round(sy, 5), 'offset_x': round(d['left'] * nw, 2),
             'offset_y': round(d['top'] * nh, 2), 'score': None, 'source': 'locate.py (a model\'s estimate)'}
        print(f'scale {sx:.4f} x {sy:.4f}, offset {a["offset_x"]:.1f}, {a["offset_y"]:.1f}   ({d["note"]})')
        if save:
            with open(save, 'w') as f:
                json.dump(a, f, indent=1)
        return 0
    sys.exit('give --find TEXT or --ref REF')


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
