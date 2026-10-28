#!/usr/bin/env python3
"""Build tiny BMP, WAV, MIDI and AVI files for the tests of tools/decoders (our own data, made
here byte by byte from the formats' published layouts, not by the decoders under test).

Usage: make_media.py OUT_DIR

Every picture is the same 4 x 3 image (PICTURE, indices into PALETTE) in another BMP variant:
8 bits bottom-up, 24 bits top-down, RLE8, RLE4, 1 bit (two colours: its own picture, MONO),
4 bits with an OS/2 core header, 16 bits with 5-6-5 bitfields, 32 bits with a V5 header, and a
file cut short after its first stored row. The sounds are 16-bit stereo PCM and a WAV whose
data size field claims more than is there. The songs are a format 1 MIDI file with two tracks
and the same song wrapped in a RIFF RMID. The film is an AVI with one video and one audio
stream and two frames. tests/run.sh checks what the decoders read against these values.
"""
import os
import struct
import sys

PALETTE = [(0, 0, 0), (255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 255)]
PICTURE = [[1, 2, 3, 4],
           [4, 3, 2, 1],
           [0, 1, 0, 2]]          # rows top to bottom
MONO = [[1, 0, 1, 0],
        [0, 1, 0, 1],
        [1, 1, 0, 0]]
W, H = 4, 3
RATE, SAMPLES = 11025, 8          # the WAV: frames of 16-bit stereo
NOTES = [(60, 96), (64, 96), (67, 192)]   # the MIDI melody: key, length in ticks
DIVISION = 96


def bmp(info, palette, pixels, *, core=False, extra=b''):
    """A BMP file: the file header, `info` (the info header without its size), any `extra`
    (bitfield masks after a 40-byte header), the palette and the pixel bytes."""
    head = struct.pack('<I', len(info) + 4) + info
    pal = b''.join(bytes((b, g, r)) + (b'' if core else b'\0') for r, g, b in palette)
    offset = 14 + len(head) + len(extra) + len(pal)
    return b'BM' + struct.pack('<IHHI', offset + len(pixels), 0, 0, offset) + head + extra + pal + pixels


def info40(width, height, bits, compression, image_size, colours):
    return struct.pack('<iiHHIIiiII', width, height, 1, bits, compression, image_size, 2835, 2835, colours, 0)


def rows_packed(rows, bits):
    """Rows of indices packed `bits` to a pixel, most significant first, padded to four bytes."""
    out = b''
    for row in rows:
        acc, n, line = 0, 0, bytearray()
        for v in row:
            acc, n = acc << bits | v, n + bits
            if n == 8:
                line.append(acc)
                acc, n = 0, 0
        if n:
            line.append(acc << (8 - n))
        out += bytes(line) + bytes(-len(line) % 4)
    return out


def rgb_rows(rows, pixel):
    out = b''
    for row in rows:
        line = b''.join(pixel(PALETTE[i]) for i in row)
        out += line + bytes(-len(line) % 4)
    return out


def pictures():
    up = PICTURE[::-1]                                        # bottom-up storage
    files = {'pal8.bmp': bmp(info40(W, H, 8, 0, 12, len(PALETTE)), PALETTE, rows_packed(up, 8))}
    files['top24.bmp'] = bmp(info40(W, -H, 24, 0, 0, 0), [], rgb_rows(PICTURE, lambda c: bytes(c[::-1])))
    # RLE8, bottom row first: deltas over the zeros, an absolute run, runs and the end.
    rle8 = (bytes([0, 2, 1, 0, 1, 1, 0, 2, 1, 0, 1, 2, 0, 0])  # skip, 1, skip, 2, end of line
            + bytes([0, 4, 4, 3, 2, 1, 0, 0])                 # absolute 4 3 2 1, end of line
            + bytes([1, 1, 1, 2, 1, 3, 1, 4, 0, 1]))          # 1 2 3 4, end of bitmap
    files['rle8.bmp'] = bmp(info40(W, H, 8, 1, len(rle8), len(PALETTE)), PALETTE, rle8)
    rle4 = (bytes([2, 0x01, 2, 0x02, 0, 0])                    # runs 0 1 and 0 2, end of line
            + bytes([0, 4, 0x43, 0x21, 0, 0])                 # absolute 4 3 2 1, end of line
            + bytes([2, 0x12, 2, 0x34, 0, 1]))                # runs 1 2 and 3 4, end of bitmap
    files['rle4.bmp'] = bmp(info40(W, H, 4, 2, len(rle4), len(PALETTE)), PALETTE, rle4)
    files['mono.bmp'] = bmp(info40(W, H, 1, 0, 0, 2), [(0, 0, 0), (255, 255, 255)], rows_packed(MONO[::-1], 1))
    core = struct.pack('<HHHH', W, H, 1, 4)
    files['core4.bmp'] = bmp(core, PALETTE, rows_packed(up, 4), core=True)
    masks = struct.pack('<III', 0xf800, 0x07e0, 0x001f)
    p565 = lambda c: struct.pack('<H', (c[0] >> 3) << 11 | (c[1] >> 2) << 5 | c[2] >> 3)
    files['rgb565.bmp'] = bmp(info40(W, H, 16, 3, 0, 0), [], rgb_rows(up, p565), extra=masks)
    v5 = info40(W, H, 32, 3, 0, 0) + struct.pack('<IIII', 0xff0000, 0xff00, 0xff, 0xff000000) + bytes(124 - 4 - 56)
    files['v5_32.bmp'] = bmp(v5, [], rgb_rows(up, lambda c: bytes((c[2], c[1], c[0], 255))))
    files['short.bmp'] = files['pal8.bmp'][:-8]               # only the bottom row is there
    return files


def riff(form, *chunks):
    body = form + b''.join(chunks)
    return b'RIFF' + struct.pack('<I', len(body)) + body


def chunk(cid, data):
    return cid + struct.pack('<I', len(data)) + data + bytes(len(data) % 2)


def lst(form, *chunks):
    return chunk(b'LIST', form + b''.join(chunks))


def sounds():
    fmt = struct.pack('<HHIIHH', 1, 2, RATE, RATE * 4, 4, 16)
    data = b''.join(struct.pack('<hh', 1000 * i, -1000 * i) for i in range(SAMPLES))
    wav = riff(b'WAVE', chunk(b'fmt ', fmt), chunk(b'data', data))
    cut = riff(b'WAVE', chunk(b'fmt ', fmt), b'data' + struct.pack('<I', 64) + data)
    return {'pcm16.wav': wav, 'short.wav': cut}


def vlq(n):
    out = [n & 0x7f]
    while n > 127:
        n >>= 7
        out.insert(0, n & 0x7f | 0x80)
    return bytes(out)


def songs():
    tempo = b'\x00\xff\x51\x03' + (500000).to_bytes(3, 'big') + b'\x00\xff\x03\x05Tempo' + b'\x00\xff\x2f\x00'
    melody = b'\x00\xff\x03\x06Melody\x00\xc0\x00'
    for i, (key, length) in enumerate(NOTES):
        on = bytes([0x90, key, 100]) if i == 0 else bytes([key, 100])     # running status after the first
        melody += b'\x00' + on + vlq(length) + bytes([key, 0])            # a note-on with velocity 0 ends it
    melody += b'\x00\xff\x2f\x00'
    track = lambda d: b'MTrk' + struct.pack('>I', len(d)) + d
    smf = b'MThd' + struct.pack('>IHHH', 6, 1, 2, DIVISION) + track(tempo) + track(melody)
    return {'song.mid': smf, 'song.rmi': riff(b'RMID', chunk(b'data', smf))}


def film():
    avih = struct.pack('<10I', 66666, 0, 0, 0x10, 2, 0, 2, 0, W, H) + bytes(16)
    vids = struct.pack('<4s4sIHHIIIIIIIIhhhh', b'vids', b'DIB ', 0, 0, 0, 0, 1, 15, 0, 2, 0, 0, 0, 0, 0, W, H)
    vfmt = struct.pack('<I', 40) + info40(W, H, 8, 0, 16, 0)
    auds = struct.pack('<4s4sIHHIIIIIIIIhhhh', b'auds', b'\0\0\0\0', 0, 0, 0, 0, 1, RATE, 0, 0, 0, 0, 2, 0, 0, 0, 0)
    afmt = struct.pack('<HHIIHH', 1, 1, RATE, RATE * 2, 2, 16)
    hdrl = lst(b'hdrl', chunk(b'avih', avih), lst(b'strl', chunk(b'strh', vids), chunk(b'strf', vfmt)),
               lst(b'strl', chunk(b'strh', auds), chunk(b'strf', afmt)))
    movi = lst(b'movi', chunk(b'00db', bytes(16)), chunk(b'01wb', bytes(4)), chunk(b'00db', bytes(range(16))))
    return {'film.avi': riff(b'AVI ', hdrl, movi)}


def main(argv):
    if len(argv) != 1 or argv[0] in ('-h', '--help'):
        print(__doc__.rstrip())
        return 0 if argv and argv[0] in ('-h', '--help') else 2
    os.makedirs(argv[0], exist_ok=True)
    files = {**pictures(), **sounds(), **songs(), **film()}
    for name, data in files.items():
        with open(os.path.join(argv[0], name), 'wb') as f:
            f.write(data)
    print(f'built {len(files)} files in {argv[0]}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
