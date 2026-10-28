"""WAV sound, for validate.py and catalog.py: `decode(reader)` reads a RIFF WAVE file's fmt
and data chunks and every other chunk's id.

The result names the format (PCM, IEEE float, A-law, mu-law, Microsoft or IMA ADPCM, or the
tag's number), the channels, rate and bits, and the length in sample frames and seconds from
the sample bytes actually in the file, not from the data chunk's size field. What does not add
up is kept in `problems`: a RIFF size field that differs from the file, chunks cut short.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _riff import find, riff  # noqa: E402

FORMATS = {1: 'PCM', 2: 'Microsoft ADPCM', 3: 'IEEE float', 6: 'A-law', 7: 'mu-law',
           0x11: 'IMA ADPCM', 0x55: 'MPEG layer 3', 0xfffe: 'extensible'}


def decode(r):
    items, problems = riff(r, b'WAVE')
    fmt, data = find(items, 'fmt '), find(items, 'data')
    if not fmt or not data:
        raise ValueError('no fmt or data chunk')
    f = fmt['body']
    tag, channels, rate, byte_rate, align, bits = f.u16(), f.u16(), f.u32(), f.u32(), f.u16(), f.u16()
    f.bytes(f.remaining())                            # cbSize and the extension, if any
    present = data['body'].end - data['body'].pos
    data['body'].bytes(present)
    return {'format_tag': tag, 'format': FORMATS.get(tag, f'tag {tag:#x}'), 'channels': channels,
            'rate': rate, 'bits': bits, 'block_align': align, 'byte_rate': byte_rate,
            'frames': present // max(1, align), 'seconds': round(present / max(1, byte_rate), 3),
            'data_size_field': data['size'], 'data_bytes': present,
            'chunks': [c['id'] for c in items], 'problems': problems}
