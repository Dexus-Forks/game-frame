"""RIFF chunks (WAV, AVI, RIFF MIDI): a four-byte id, a 32-bit little-endian size, the data, a
pad byte when the size is odd. RIFF and LIST chunks hold a four-byte form type and more chunks.
Shared by wav.py, avi.py and midi.py."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'formats'))
from reader import Reader  # noqa: E402

CONTAINERS = (b'RIFF', b'LIST')


def chunks(r, depth=0, limit=4):
    """Every chunk to the end of `r`, as dicts. A leaf keeps `body`, a fresh reader over its data;
    containers are walked up to `limit` levels deep."""
    out = []
    while r.remaining():
        cid = r.bytes(4)
        size = r.u32()
        start = r.pos
        present = min(size, r.remaining())
        r.bytes(present)
        if size % 2 and r.remaining():
            r.u8()
        body = Reader(r.data, start, start + present, cid.decode('latin-1'))
        item = {'id': cid.decode('latin-1'), 'size': size, 'offset': start - 8}
        if present < size:
            item['short_by'] = size - present      # the chunk runs past its parent or the file
        if cid in CONTAINERS and depth < limit and present >= 4:
            item['form'] = body.bytes(4).decode('latin-1')
            item['chunks'] = chunks(body, depth + 1, limit)
        else:
            item['body'] = body
        out.append(item)
    return out


def find(items, cid):
    """The first chunk with this id or LIST form, searching containers depth first."""
    for item in items:
        if cid in (item['id'], item.get('form')):
            return item
        found = find(item.get('chunks', []), cid)
        if found:
            return found
    return None


def short_chunks(items):
    """Ids of every chunk, at any depth, whose declared size runs past what is there."""
    out = []
    for item in items:
        if 'short_by' in item:
            out.append((item['id'], item['short_by']))
        out += short_chunks(item.get('chunks', []))
    return out


def riff(r, form):
    """The top RIFF header of a file of this form, its chunks, and what does not add up:
    `riff_size_field` against the bytes that follow it, and chunks cut short. Nothing is
    hidden: a decoder puts `problems` in its result. Bytes after the RIFF stay unread."""
    r.magic(b'RIFF')
    size = r.u32()
    if size < 4:
        raise ValueError(f'RIFF size {size} is smaller than its form type')
    r.magic(form)
    available = r.remaining()
    items = chunks(r.block(min(size - 4, available), form.decode('latin-1')))
    problems = {}
    if size - 4 != available:
        problems['riff_size_field'] = size
        problems['bytes_after_header'] = available + 4
    if short_chunks(items):
        problems['short_chunks'] = short_chunks(items)
    return items, problems
