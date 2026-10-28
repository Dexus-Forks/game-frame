"""Standard MIDI files (.mid) and RIFF MIDI (.rmi), for validate.py and catalog.py:
`decode(reader)` reads the MThd header and every track's events.

The result has the format (0, 1 or 2), the tracks, the ticks a quarter note (or the SMPTE
frames and ticks when the division's top bit is set), the first tempo, and for each track its
name, its note-ons (velocity above 0) and its length in ticks. Running status, meta events and
system exclusive events are followed; a track that ends inside an event, or a chunk count that
differs from the header's, is an error. Chunks other than MTrk are listed and skipped, as the
format allows.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _riff import find, riff  # noqa: E402

META, SYSEX, SYSEX_ESCAPE = 0xff, 0xf0, 0xf7
META_NAME, META_TEMPO, META_END = 0x03, 0x51, 0x2f
ONE_DATA_BYTE = (0xc0, 0xd0)          # program change, channel pressure


def vlq(data, at):
    """A variable-length number (seven bits a byte, high bit set on all but the last)."""
    value = 0
    for i in range(4):
        if at >= len(data):
            raise ValueError('a variable-length number runs past the track')
        b = data[at]
        at += 1
        value = value << 7 | b & 0x7f
        if not b & 0x80:
            return value, at
    raise ValueError('a variable-length number of more than four bytes')


def track_events(data):
    """Name, note-ons, length in ticks and first tempo of one MTrk chunk's data."""
    at = ticks = notes = 0
    status, name, tempo = None, None, None
    while at < len(data):
        delta, at = vlq(data, at)
        ticks += delta
        if at >= len(data):
            raise ValueError('a track ends after a delta time')
        if data[at] & 0x80:
            status = data[at]
            at += 1
        elif status is None or status >= SYSEX:
            raise ValueError(f'running status without a status byte at {at:#x}')
        if status == META:
            kind = data[at]
            length, at = vlq(data, at + 1)
            body = data[at:at + length]
            at += length
            if kind == META_NAME and name is None:
                name = body.decode('latin-1')
            elif kind == META_TEMPO and tempo is None and length == 3:
                tempo = int.from_bytes(body, 'big')
            elif kind == META_END:
                break
            status = None
        elif status in (SYSEX, SYSEX_ESCAPE):
            length, at = vlq(data, at)
            at += length
            status = None
        else:
            size = 1 if status & 0xf0 in ONE_DATA_BYTE else 2
            if at + size > len(data):
                raise ValueError('a track ends inside an event')
            if status & 0xf0 == 0x90 and data[at + 1]:
                notes += 1
            at += size
    if at > len(data):
        raise ValueError('a track ends inside an event')
    return {'name': name, 'notes': notes, 'ticks': ticks}, tempo


def smf(r):
    r.magic(b'MThd')
    size = r.u32be()
    if size < 6:
        raise ValueError(f'MThd of {size} bytes')
    fmt, ntracks, division = r.u16be(), r.u16be(), r.u16be()
    r.bytes(size - 6)
    tracks, others, tempo = [], [], None
    while r.remaining():
        cid, size = r.bytes(4), r.u32be()
        data = r.bytes(size)
        if cid != b'MTrk':
            others.append(cid.decode('latin-1'))
            continue
        track, t = track_events(data)
        tracks.append(track)
        tempo = tempo if tempo is not None else t
    if len(tracks) != ntracks:
        raise ValueError(f'{ntracks} tracks in the header, {len(tracks)} MTrk chunks')
    timing = ({'smpte_frames': 256 - (division >> 8), 'ticks_per_frame': division & 0xff}
              if division & 0x8000 else {'ticks_per_quarter': division})
    return {'format': fmt, 'tracks': tracks, **timing, 'tempo_us_per_quarter': tempo,
            'other_chunks': others}


def decode(r):
    if r.remaining() >= 4 and r.data[r.pos:r.pos + 4] == b'RIFF':
        items, problems = riff(r, b'RMID')
        data = find(items, 'data')
        if not data:
            raise ValueError('a RIFF MIDI file without a data chunk')
        return {'riff': True, 'problems': problems, **smf(data['body'])}
    return {'riff': False, **smf(r)}
