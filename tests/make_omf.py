#!/usr/bin/env python3
"""Build a tiny OMF library and a DOS program that contains its code, for tests/run.sh: the
library holds two public functions in a CODE segment, one calling the other through a fixup.
The program holds the same code at another place, with the call already resolved to a
different distance, as a linker would leave it.

Usage: make_omf.py DIR      writes DIR/RT.LIB and DIR/GAME.EXE
"""
import os
import struct
import sys


def record(kind, body):
    rec = bytes([kind]) + struct.pack('<H', len(body) + 1) + body
    return rec + bytes([(-sum(rec)) & 0xFF])


def pstr(s):
    return bytes([len(s)]) + s.encode()


# _strlen: push bp / mov bp,sp / mov bx,[bp+4] / xor ax,ax / ... / pop bp / ret
STRLEN = bytes.fromhex('55 8B EC 8B 5E 04 33 C0 80 3F 00 74 04 40 43 EB F7 5D C3'.replace(' ', ''))
# _puts: push bp / mov bp,sp / push word [bp+4] / call _strlen (fixup) / add sp,2 / mov cx,ax / pop bp / ret
PUTS = bytes.fromhex('55 8B EC FF 76 04 E8 00 00 83 C4 02 8B C8 5D C3'.replace(' ', ''))


def module():
    code = STRLEN + PUTS
    out = record(0x80, pstr('rtlib.c'))
    out += record(0x96, pstr('') + pstr('_TEXT') + pstr('CODE'))
    out += record(0x98, bytes([0x28]) + struct.pack('<H', len(code)) + bytes([2, 3, 1]))   # byte-aligned, public
    out += record(0x90, bytes([0, 1]) + pstr('_strlen') + struct.pack('<H', 0) + bytes([0])
                  + pstr('_puts') + struct.pack('<H', len(STRLEN)) + bytes([0]))
    out += record(0xA0, bytes([1]) + struct.pack('<H', 0) + code)
    # One fixup: the call's 2-byte displacement at code offset len(STRLEN) + 7, self-relative
    # (M = 0), location 1 (16-bit offset), frame F5 (the target's), target T0 segment 1 with a
    # displacement of 0.
    at = len(STRLEN) + 7
    locat = 0x8000 | (1 << 10) | at
    out += record(0x9C, struct.pack('>H', locat) + bytes([0x50, 1]) + struct.pack('<H', 0))
    out += record(0x8A, bytes([0]))
    return out


def library(mod):
    page = 16
    header = record(0xF0, bytes(page - 3 - 1))
    body = header + mod
    body += bytes((-len(body)) % page)
    return body + record(0xF1, b'')


def program():
    """A small MZ program: some game code, then the library code with the call resolved."""
    game = bytes.fromhex('B8 00 4C CD 21'.replace(' ', '')) + bytes(57)
    puts = bytearray(PUTS)
    rel = -(len(game) + len(STRLEN) + 9 - len(game))      # back to _strlen, as the linker would
    struct.pack_into('<h', puts, 7, rel)
    image = game + STRLEN + bytes(puts) + bytes(32)
    header = bytearray(32)
    header[0:2] = b'MZ'
    size = 32 + len(image)
    struct.pack_into('<HHHHHHHHHHHHH', header, 2, size % 512, (size + 511) // 512, 0, 2, 0x10, 0xFFFF,
                     0, 0x100, 0, 0, 0, 0x1C, 0)
    return bytes(header) + image


def main(out):
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, 'RT.LIB'), 'wb') as f:
        f.write(library(module()))
    with open(os.path.join(out, 'GAME.EXE'), 'wb') as f:
        f.write(program())


if __name__ == '__main__':
    main(sys.argv[1])
