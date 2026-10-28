"""The pixel data of a BMP: uncompressed rows of 1 to 32 bits a pixel, and the RLE8 and RLE4
run-length codes. Used by bmp.py. Rows come back top to bottom."""

RLE_END_LINE, RLE_END, RLE_DELTA = 0, 1, 2


def stride(width, bits):
    """Bytes in one stored row: whole 32-bit words."""
    return (width * bits + 31) // 32 * 4


def indices(row, width, bits):
    """One palette index a pixel from a row packed `bits` (1, 2, 4, 8) to a pixel, high bits first."""
    if bits == 8:
        return bytes(row[:width])
    per, mask = 8 // bits, (1 << bits) - 1
    return bytes(row[x // per] >> (8 - bits * (x % per + 1)) & mask for x in range(width))


def channel(value, mask):
    """A colour channel picked by a bit mask, scaled to 0..255."""
    if not mask:
        return 0
    shift = (mask & -mask).bit_length() - 1
    top = mask >> shift
    return ((value & mask) >> shift) * 255 // top


def rgb(row, width, bits, masks):
    """Three bytes R, G, B a pixel from a 16, 24 or 32-bit row. `masks` (red, green, blue) pick
    the channels of 16 and 32-bit pixels; 24-bit pixels are stored B, G, R."""
    out = bytearray()
    if bits == 24:
        for x in range(width):
            b, g, r = row[3 * x:3 * x + 3]
            out += bytes((r, g, b))
        return bytes(out)
    size = bits // 8
    for x in range(width):
        v = int.from_bytes(row[size * x:size * x + size], 'little')
        out += bytes(channel(v, m) for m in masks)
    return bytes(out)


def rle(data, width, height, bits):
    """Indices of an RLE8 (bits 8) or RLE4 (bits 4) picture, bottom row first, and the bytes the
    code used. Pixels the code skips (a delta, an early end) stay index 0. A run past the right
    edge is cut there; the code may end without its end-of-bitmap mark."""
    grid = [bytearray(width) for _ in range(height)]
    x = y = at = 0

    def put(v):
        nonlocal x
        if x < width and y < height:
            grid[y][x] = v
        x += 1

    while at + 1 < len(data) and y < height:
        count, value = data[at], data[at + 1]
        at += 2
        if count:                                     # a run: count pixels of one byte
            for i in range(count):
                put(value if bits == 8 else (value >> 4 if i % 2 == 0 else value & 15))
        elif value == RLE_END_LINE:
            x, y = 0, y + 1
        elif value == RLE_END:
            break
        elif value == RLE_DELTA:
            if at + 1 >= len(data):
                break
            x, y = x + data[at], y + data[at + 1]
            at += 2
        else:                                         # absolute: `value` pixels, word aligned
            used = value if bits == 8 else (value + 1) // 2
            raw = data[at:at + used]
            for i in range(value):
                if bits == 8:
                    put(raw[i] if i < len(raw) else 0)
                else:
                    b = raw[i // 2] if i // 2 < len(raw) else 0
                    put(b >> 4 if i % 2 == 0 else b & 15)
            at += used + used % 2
    return [bytes(row) for row in grid], min(at, len(data))
