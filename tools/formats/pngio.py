#!/usr/bin/env python3
"""Read and write PNG pictures with the standard library alone.

    from pngio import read_png, write_png, grey
    w, h, rgb = read_png('shot.png')        # rgb: bytes, 3 per pixel, rows top to bottom
    write_png('out.png', w, h, rgb)
    g = grey(w, h, rgb)                     # one byte per pixel

Usage: pngio.py FILE                        print the size, colour type and bit depth

It reads 8-bit greyscale, grey with alpha, RGB, RGBA and palette pictures (bit depth 8, or 1, 2
and 4 for palette and grey), with every filter. Alpha is dropped. It writes 8-bit RGB.

What it does not read: interlaced (Adam7) or 16-bit pictures. Convert those first, for example
with `sips -s format png` on macOS or ImageMagick's `convert -depth 8 -interlace none`.
"""
import struct
import sys
import zlib

SIGNATURE = b'\x89PNG\r\n\x1a\n'
CHANNELS = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}


def _chunks(data):
    i = 8
    while i < len(data):
        n, kind = struct.unpack_from('>I4s', data, i)
        yield kind, data[i + 8:i + 8 + n]
        i += 12 + n


def _unfilter(raw, h, stride, bpp):
    out, prev, i = bytearray(), bytearray(stride), 0
    for _ in range(h):
        f, line = raw[i], bytearray(raw[i + 1:i + 1 + stride])
        i += 1 + stride
        if f == 1:
            for x in range(bpp, stride):
                line[x] = (line[x] + line[x - bpp]) & 255
        elif f == 2:
            for x in range(stride):
                line[x] = (line[x] + prev[x]) & 255
        elif f == 3:
            for x in range(stride):
                left = line[x - bpp] if x >= bpp else 0
                line[x] = (line[x] + ((left + prev[x]) >> 1)) & 255
        elif f == 4:
            for x in range(stride):
                a = line[x - bpp] if x >= bpp else 0
                b, c = prev[x], prev[x - bpp] if x >= bpp else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                line[x] = (line[x] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        out += line
        prev = line
    return out


def info(path):
    data = open(path, 'rb').read()
    if data[:8] != SIGNATURE:
        raise ValueError(f'{path}: not a PNG file')
    w, h, depth, ctype, _, _, interlace = struct.unpack_from('>IIBBBBB', data, 16)
    return w, h, depth, ctype, interlace


def read_png(path):
    """(width, height, rgb bytes)."""
    data = open(path, 'rb').read()
    if data[:8] != SIGNATURE:
        raise ValueError(f'{path}: not a PNG file')
    w, h, depth, ctype, _, _, interlace = struct.unpack_from('>IIBBBBB', data, 16)
    if interlace or depth == 16 or ctype not in CHANNELS:
        raise ValueError(f'{path}: interlaced, 16-bit or unknown PNG; convert it to 8-bit first')
    palette, idat = b'', bytearray()
    for kind, body in _chunks(data):
        if kind == b'PLTE':
            palette = body
        elif kind == b'IDAT':
            idat += body
    ch = CHANNELS[ctype]
    bits = depth * ch
    stride = (w * bits + 7) // 8
    px = _unfilter(zlib.decompress(bytes(idat)), h, stride, max(1, bits // 8))
    if depth < 8:                       # unpack 1, 2 or 4 bits per sample, row by row
        mask, out = (1 << depth) - 1, bytearray()
        for y in range(h):
            row = px[y * stride:(y + 1) * stride]
            for x in range(w):
                bit = x * depth
                out.append((row[bit >> 3] >> (8 - depth - (bit & 7))) & mask)
        px = out if ctype == 3 else bytearray(v * 255 // mask for v in out)
    rgb = bytearray(w * h * 3)
    if ctype == 2:
        return w, h, bytes(px)
    for i in range(w * h):
        if ctype == 3:
            rgb[i * 3:i * 3 + 3] = palette[px[i] * 3:px[i] * 3 + 3]
        elif ctype == 6:
            rgb[i * 3:i * 3 + 3] = px[i * 4:i * 4 + 3]
        else:                           # grey, or grey with alpha
            v = px[i * ch]
            rgb[i * 3:i * 3 + 3] = bytes((v, v, v))
    return w, h, bytes(rgb)


def write_png(path, w, h, rgb):
    raw = b''.join(b'\0' + rgb[y * w * 3:(y + 1) * w * 3] for y in range(h))

    def chunk(kind, body):
        return struct.pack('>I', len(body)) + kind + body + struct.pack('>I', zlib.crc32(kind + body) & 0xffffffff)
    with open(path, 'wb') as f:
        f.write(SIGNATURE + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0))
                + chunk(b'IDAT', zlib.compress(bytes(raw), 6)) + chunk(b'IEND', b''))


def grey(w, h, rgb):
    """One byte per pixel: (2R + 5G + B) / 8."""
    return bytes((2 * rgb[i] + 5 * rgb[i + 1] + rgb[i + 2]) >> 3 for i in range(0, w * h * 3, 3))


if __name__ == '__main__':
    if len(sys.argv) != 2 or sys.argv[1] in ('-h', '--help'):
        print(__doc__); sys.exit(0)
    w, h, depth, ctype, interlace = info(sys.argv[1])
    print(f'{w} x {h}, colour type {ctype}, {depth}-bit' + (', interlaced' if interlace else ''))
