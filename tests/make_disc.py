#!/usr/bin/env python3
"""Build small CD images for tests/run.sh: an ISO 9660 volume with Joliet names, a date and a
hidden file, wrapped as a mixed-mode CUE/BIN (a Mode 1 data track and an audio track) and as a
plain .iso.

Usage: make_disc.py DIR
"""
import os
import struct
import sys


def both16(n):
    return struct.pack('<H', n) + struct.pack('>H', n)


def both32(n):
    return struct.pack('<I', n) + struct.pack('>I', n)


def date7():
    return bytes([95, 11, 3, 14, 30, 0, 4])          # 1995-11-03 14:30:00 UTC+01:00


def record(name, lba, size, directory=False, hidden=False):
    flags = (2 if directory else 0) | (1 if hidden else 0)
    body = (bytes([0]) + both32(lba) + both32(size) + date7() + bytes([flags, 0, 0])
            + both16(1) + bytes([len(name)]) + name)
    if len(body) % 2 == 0:
        body += b'\x00'
    return bytes([len(body) + 1]) + body


def descriptor(kind, volume, root, joliet=False):
    d = bytearray(2048)
    d[0] = kind
    d[1:7] = b'CD001\x01'
    pad = '                '
    enc = (lambda s: (s + pad * 2).encode('utf-16-be')[:32]) if joliet else \
        (lambda s: (s + pad * 2).encode('latin-1')[:32])
    d[8:40] = enc('GAMEFRAME')
    d[40:72] = enc(volume)
    d[80:88] = both32(40)
    d[120:124] = both16(1)
    d[124:128] = both16(1)
    d[128:132] = both16(2048)
    if joliet:
        d[88:91] = b'%/E'
    d[156:156 + len(root)] = root
    d[318:318 + 32] = enc('TEST PUBLISHER')
    d[574:574 + 32] = enc('GAMEFRAME TESTS')
    d[813:830] = b'1995110314300000' + bytes([4])
    return bytes(d)


def iso():
    files = [(b'README.TXT;1', 'ReadMe File.txt', b'hello from the disc\n', False),
             (b'SECRET.DAT;1', 'Secret Data.dat', b'\x01\x02\x03\x04' * 300, True)]
    sectors = {}
    lba = 24
    plain, wide = [], []
    for short, long, data, hidden in files:
        plain.append(record(short, lba, len(data), hidden=hidden))
        wide.append(record(long.encode('utf-16-be'), lba, len(data), hidden=hidden))
        sectors[lba] = data
        lba += (len(data) + 2047) // 2048
    root_a, root_b = 20, 21
    for root, entries in ((root_a, plain), (root_b, wide)):
        body = record(b'\x00', root, 2048, True) + record(b'\x01', root, 2048, True) + b''.join(entries)
        sectors[root] = body
    image = bytearray(2048 * 40)
    image[16 * 2048:17 * 2048] = descriptor(1, 'TESTDISC', record(b'\x00', root_a, 2048, True))
    image[17 * 2048:18 * 2048] = descriptor(2, 'Test Disc', record(b'\x00', root_b, 2048, True), joliet=True)
    image[18 * 2048:18 * 2048 + 7] = b'\xffCD001\x01'
    for at, data in sectors.items():
        image[at * 2048:at * 2048 + len(data)] = data
    return bytes(image)


def raw_mode1(cooked):
    out = bytearray()
    for i in range(len(cooked) // 2048):
        m, s, f = (i + 150) // 4500, (i + 150) // 75 % 60, (i + 150) % 75
        bcd = lambda n: (n // 10) << 4 | n % 10
        out += b'\x00' + b'\xff' * 10 + b'\x00' + bytes([bcd(m), bcd(s), bcd(f), 1])
        out += cooked[i * 2048:(i + 1) * 2048] + bytes(288)
    return bytes(out)


def main(out):
    os.makedirs(out, exist_ok=True)
    cooked = iso()
    with open(os.path.join(out, 'test.iso'), 'wb') as f:
        f.write(cooked)
    audio = struct.pack('<hh', 1000, -1000) * (588 * 150)      # 2 seconds of a square-ish tone
    with open(os.path.join(out, 'test.bin'), 'wb') as f:
        f.write(raw_mode1(cooked) + bytes(2352 * 150) + audio)
    with open(os.path.join(out, 'test.cue'), 'w') as f:
        f.write('FILE "TEST.BIN" BINARY\n  TRACK 01 MODE1/2352\n    INDEX 01 00:00:00\n'
                '  TRACK 02 AUDIO\n    INDEX 00 00:00:40\n    INDEX 01 00:02:40\n')


if __name__ == '__main__':
    main(sys.argv[1])
