"""Decoder for the sample game's data file, as `load_records` in game.c reads it.

Run over every file of the type: tools/formats/validate.py examples/sample-game/decode_sample.py \
'examples/sample-game/build/*.dat' --show 1
"""


def decode(r):
    r.magic(b'SMPL')
    count = r.u32()
    return [{'id': r.u32(), 'weight': r.f32(), 'name': r.pstr8()} for _ in range(count)]
