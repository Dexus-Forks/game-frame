#!/usr/bin/env python3
"""Build small Flash files for tests/run.sh: an uncompressed AS3 movie, its zlib and LZMA
versions, and a fake Windows projector with the movie appended and the projector trailer.

Usage: make_swf.py DIR
"""
import lzma
import os
import struct
import sys
import zlib


def tag(code, body):
    if len(body) < 0x3F:
        return struct.pack('<H', code << 6 | len(body)) + body
    return struct.pack('<HI', code << 6 | 0x3F, len(body)) + body


def rect(xmin, xmax, ymin, ymax, nbits=16):
    bits = nbits
    for v in (xmin, xmax, ymin, ymax):
        bits = (bits << nbits) | (v & ((1 << nbits) - 1))
    total = 5 + 4 * nbits
    nbytes = (total + 7) // 8
    return (bits << (nbytes * 8 - total)).to_bytes(nbytes, 'big')


def movie():
    body = rect(0, 550 * 20, 0, 400 * 20) + struct.pack('<HH', 24 << 8, 3)
    body += tag(69, struct.pack('<I', 0x08))                       # FileAttributes: AS3
    body += tag(9, b'\xff\xff\xff')                                # SetBackgroundColor
    body += tag(82, struct.pack('<I', 1) + b'frame1\0' + bytes(80)) # DoABC (stand-in bytes)
    body += tag(76, struct.pack('<HH', 1, 0) + b'GameMain\0')      # SymbolClass
    body += tag(14, bytes(70))                                     # DefineSound
    body += tag(1, b'') + tag(0, b'')
    return b'FWS' + bytes([10]) + struct.pack('<I', 8 + len(body)) + body


def main(out):
    os.makedirs(out, exist_ok=True)
    fws = movie()
    cws = b'CWS' + fws[3:8] + zlib.compress(fws[8:])
    raw = lzma.compress(fws[8:], format=lzma.FORMAT_ALONE)
    zws = b'ZWS' + fws[3:8] + struct.pack('<I', len(raw) - 13) + raw[:5] + raw[13:]
    for name, data in (('game.swf', fws), ('game-cws.swf', cws), ('game-zws.swf', zws)):
        with open(os.path.join(out, name), 'wb') as f:
            f.write(data)
    with open(os.path.join(out, 'projector.exe'), 'wb') as f:
        f.write(b'MZ' + bytes(4094) + cws + struct.pack('<II', 0xFA123456, len(cws)))


if __name__ == '__main__':
    main(sys.argv[1])
