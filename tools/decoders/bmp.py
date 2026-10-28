"""Windows and OS/2 BMP pictures, for validate.py and catalog.py: `decode(reader)` and
`images(result)`.

Reads every header version (OS/2's 12-byte core header, Windows' 40-byte info header and the
larger V2 to V5 headers), 1, 2, 4 and 8 bits a pixel with a palette, 16, 24 and 32 bits a pixel
(plain or with bit-field masks), and the run-length codes RLE8 and RLE4. Rows are stored bottom
up unless the height is negative. JPEG and PNG inside a BMP are refused.

The result holds the header fields, the palette as (r, g, b) entries, and `pixels`, top row
first: one palette index a pixel when `rgb` is false, else three bytes R, G, B a pixel. Nothing
is hidden: a file shorter than its header says keeps the rows that are there and counts the
rest in `missing_rows` (shown as index 0, or black); bytes after the pixels are counted in
`bytes_after_pixels`, and a gap between the palette and the pixels in `gap_before_pixels`.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import _bmp_pixels as px  # noqa: E402

RGB, RLE8, RLE4, BITFIELDS, JPEG, PNG, ALPHABITFIELDS = range(7)
CORE, INFO = 12, 40
DEFAULT_MASKS = {16: (0x7c00, 0x03e0, 0x001f), 32: (0xff0000, 0xff00, 0xff)}
MAX_PIXELS = 1 << 26              # a bigger picture is a misread header, not a game's picture
MISSING = (255, 0, 255)           # the colour images() gives indices past the palette


def header(r):
    """The info header's fields; the bit-field masks, from the header or from the dwords
    after a 40-byte one."""
    size = r.u32()
    if size == CORE:
        width, height, planes, bits = r.u16(), r.u16(), r.u16(), r.u16()
        return {'header': size, 'width': width, 'height': height, 'planes': planes, 'bits': bits,
                'compression': RGB, 'image_size_field': 0, 'colours_used': 0, 'important': 0}, None
    if size < INFO:
        raise ValueError(f'info header of {size} bytes')
    h = {'header': size, 'width': r.i32(), 'height': r.i32(), 'planes': r.u16(), 'bits': r.u16(),
         'compression': r.u32(), 'image_size_field': r.u32()}
    r.i32(), r.i32()                                  # pixels a metre
    h['colours_used'], h['important'] = r.u32(), r.u32()
    masks = None
    if size > INFO:
        rest = r.block(size - INFO, 'header')
        if size >= 52:
            masks = (rest.u32(), rest.u32(), rest.u32())
        rest.bytes(rest.remaining())
    elif h['compression'] in (BITFIELDS, ALPHABITFIELDS):
        masks = (r.u32(), r.u32(), r.u32())
        if h['compression'] == ALPHABITFIELDS:
            r.u32()
    return h, masks


def check(h):
    if h['compression'] in (JPEG, PNG):
        raise ValueError('a JPEG or PNG inside the BMP')
    allowed = {RGB: (1, 2, 4, 8, 16, 24, 32), RLE8: (8,), RLE4: (4,), BITFIELDS: (16, 32), ALPHABITFIELDS: (16, 32)}
    if h['bits'] not in allowed.get(h['compression'], ()):
        raise ValueError(f"{h['bits']} bits a pixel with compression {h['compression']}")
    w, hh = abs(h['width']), abs(h['height'])
    if w == 0 or hh == 0 or w * hh > MAX_PIXELS:
        raise ValueError(f'a picture of {w} x {hh} pixels')


def decode(r):
    r.magic(b'BM')
    file_size, _, bits_offset = r.u32(), r.u32(), r.u32()
    h, masks = header(r)
    check(h)
    bits, width, height = h['bits'], abs(h['width']), abs(h['height'])
    count = h['colours_used'] or (1 << bits if bits <= 8 else 0)
    entry = 3 if h['header'] == CORE else 4
    if h['header'] == CORE and bits_offset > r.pos:   # no count field: as many as fit before the pixels
        count = min(count, (bits_offset - r.pos) // entry)
    palette = [tuple(r.bytes(entry)[2::-1]) for _ in range(count)]     # stored B, G, R (, 0)
    gap = bits_offset - r.pos
    if gap < 0:
        raise ValueError(f'pixels start at {bits_offset:#x}, inside the header or palette')
    r.bytes(min(gap, r.remaining()))
    data = r.bytes(r.remaining())
    if h['compression'] in (RLE8, RLE4):
        rows, used = px.rle(data, width, height, bits)
        missing = 0
    else:
        line = px.stride(width, bits)
        stored = [data[i * line:(i + 1) * line] for i in range(min(height, len(data) // line))]
        used, missing = len(stored) * line, height - len(stored)
        if bits <= 8:
            rows = [px.indices(s, width, bits) for s in stored] + [bytes(width)] * missing
        else:
            use = masks if h['compression'] in (BITFIELDS, ALPHABITFIELDS) else DEFAULT_MASKS.get(bits)
            rows = [px.rgb(s, width, bits, use) for s in stored]
            rows += [bytes(3 * width)] * missing
    if h['height'] > 0:
        rows.reverse()
    return {**h, 'height': height, 'top_down': h['height'] < 0, 'rgb': bits > 8, 'palette': palette,
            'masks': masks, 'file_size_field': file_size, 'gap_before_pixels': gap,
            'missing_rows': missing, 'bytes_after_pixels': len(data) - used, 'pixels': b''.join(rows)}


def images(result):
    w, h = abs(result['width']), result['height']
    if result['rgb']:
        return [('picture', w, h, result['pixels'], 'rgb')]
    pal = result['palette'] + [MISSING] * (256 - len(result['palette']))
    return [('picture', w, h, b''.join(bytes(pal[i]) for i in result['pixels']), 'rgb')]
