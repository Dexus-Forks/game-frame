"""Decoder for the sample game's data file, as `load_records` in game.c reads it.

Run over every file of the type: tools/formats/validate.py examples/sample-game/decode_sample.py \
'examples/sample-game/build/*.dat' --show 1 (add --write to check encode as well)
"""
import struct


def decode(r):
    r.magic(b'SMPL')
    count = r.u32()
    return [{'id': r.u32(), 'weight': r.f32(), 'name': r.pstr8()} for _ in range(count)]


def encode(records):
    out = b'SMPL' + struct.pack('<I', len(records))
    for rec in records:
        name = rec['name'].encode('latin-1') if isinstance(rec['name'], str) else rec['name']
        out += struct.pack('<IfB', rec['id'], rec['weight'], len(name)) + name
    return out
