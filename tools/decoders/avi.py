"""Video for Windows films (AVI), for validate.py and catalog.py: `decode(reader)` reads the
RIFF AVI headers: the main header (frames, frame rate, size) and, for each stream, its type,
handler and format (a video stream's compression and size, an audio stream's format tag, rate
and channels). The frames in `movi` are counted by stream, not decoded. What does not add up
is kept in `problems` (wav.py's rules).
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _riff import find, riff  # noqa: E402

FOURCC_RAW = {0: 'RGB', 1: 'RLE8', 2: 'RLE4', 3: 'BITFIELDS'}


def text(b):
    return b.decode('latin-1')


def stream_format(kind, body):
    """What a strf chunk says: BITMAPINFOHEADER for video, WAVEFORMATEX for audio."""
    if body is None:
        return {}
    if kind == 'vids' and body.remaining() >= 40:
        body.u32()
        width, height, _, bits, compression = body.i32(), body.i32(), body.u16(), body.u16(), body.u32()
        codec = FOURCC_RAW.get(compression) or text(compression.to_bytes(4, 'little'))
        out = {'width': width, 'height': abs(height), 'bits': bits, 'compression': codec}
    elif kind == 'auds' and body.remaining() >= 16:
        tag, channels, rate = body.u16(), body.u16(), body.u32()
        body.u32(), body.u16()
        out = {'format_tag': tag, 'channels': channels, 'rate': rate, 'bits': body.u16()}
    else:
        out = {}
    body.bytes(body.remaining())
    return out


def decode(r):
    items, problems = riff(r, b'AVI ')
    avih, hdrl = find(items, 'avih'), find(items, 'hdrl')
    if not avih or not hdrl or 'chunks' not in hdrl:
        raise ValueError('no avih chunk or hdrl list')
    h = avih['body']
    usec, _, _, _, frames, _, nstreams, _, width, height = [h.u32() for _ in range(10)]
    h.bytes(h.remaining())
    streams = []
    for strl in [c for c in hdrl['chunks'] if c.get('form') == 'strl']:
        strh = find(strl['chunks'], 'strh')
        if not strh or strh['body'].remaining() < 8:
            raise ValueError('a stream list without a stream header')
        kind, handler = text(strh['body'].bytes(4)), text(strh['body'].bytes(4))
        strh['body'].bytes(strh['body'].remaining())
        strf = find(strl['chunks'], 'strf')
        streams.append({'type': kind, 'handler': handler, **stream_format(kind, strf and strf['body'])})
    movi = find(items, 'movi')
    chunks_by_stream = {}
    for c in (movi or {}).get('chunks', []):
        if c['id'][:2].isdigit():
            chunks_by_stream[c['id']] = chunks_by_stream.get(c['id'], 0) + 1
    return {'frames': frames, 'fps': round(1e6 / usec, 3) if usec else None, 'width': width,
            'height': height, 'streams_field': nstreams, 'streams': streams,
            'movi_chunks': chunks_by_stream, 'problems': problems,
            'chunks': [c['id'] + ('/' + c['form'] if 'form' in c else '') for c in items]}
