"""Make the x87 test vectors: real x87 instructions run under GameFrame's emulator (Unicorn).

    .venv/bin/python crates/x87/tests/make_vectors.py crates/x87/tests/vectors.txt
    .venv/bin/python crates/x87/tests/make_vectors.py /tmp/big.txt --cases 2000 --seed 7

A small routine, assembled here by hand, loads a control word and two 80-bit operands, runs one
instruction (`faddp`, `fsubp`, `fmulp`, `fdivp`), and stores the 80-bit result. A second routine
stores one 80-bit value as a `Double` and a `Single`, as 16-, 32- and 64-bit integers (`fist`),
rounds it with `frndint` and takes `fsqrt` of its absolute value. The operands are drawn from a
seeded generator, so the file is the same on every run; no game's files are used.

Each line is an operation, the control word in hex, and the values in hex: an 80-bit value as
20 digits (sign and exponent, then the 64-bit significand), a Double as 16, a Single as 8, an
integer as 4, 8 or 16. The crate's test `vectors.rs` reads them.
"""
import argparse
import random
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / 'tools' / 'emu'))
from pe32_emu import Emu32  # noqa: E402

BASE = 0x401000
DATA = BASE + 0x800
CW, A, B, R, R64, R32, I16, I32, I64, RND, SQ = (DATA + o for o in
                                                  (0x00, 0x10, 0x20, 0x30, 0x40, 0x48, 0x4c, 0x50, 0x58, 0x60, 0x70))

# Control words: precision 64, 53 or 24 bits; rounding nearest, down, up or toward zero.
CONTROL_WORDS = (0x037f, 0x027f, 0x007f, 0x077f, 0x0b7f, 0x0f7f, 0x067f, 0x0c7f)


def m(op, reg, addr):
    """An x87 instruction with a [disp32] memory operand: opcode, ModRM (mod 00, rm 101)."""
    return bytes([op, reg << 3 | 5]) + struct.pack('<I', addr)


def start():
    return b'\xdb\xe3' + m(0xd9, 5, CW)          # fninit; fldcw [CW]


def binop(op):
    # fld tbyte [A]; fld tbyte [B]; op st(1), st(0) and pop: st(1) = A op B; fstp tbyte [R]
    return start() + m(0xdb, 5, A) + m(0xdb, 5, B) + op + m(0xdb, 7, R) + b'\xc3'


OPS = {'add': b'\xde\xc1', 'sub': b'\xde\xe9', 'mul': b'\xde\xc9', 'div': b'\xde\xf9'}

CONV = (start()
        + m(0xdb, 5, A)                       # fld tbyte [A]
        + m(0xdd, 2, R64)                     # fst qword [R64]
        + m(0xd9, 2, R32)                     # fst dword [R32]
        + m(0xdf, 2, I16)                     # fist word [I16]
        + m(0xdb, 2, I32)                     # fist dword [I32]
        + m(0xdf, 7, I64)                     # fistp qword [I64]
        + m(0xdb, 5, A) + b'\xd9\xfc' + m(0xdb, 7, RND)                   # frndint
        + m(0xdb, 5, A) + b'\xd9\xe1\xd9\xfa' + m(0xdb, 7, SQ)            # fabs; fsqrt
        + b'\xc3')


def build():
    code = bytearray(0x1000)
    entry = {}
    for i, (name, op) in enumerate(list(OPS.items()) + [('conv', None)]):
        body = CONV if op is None else binop(op)
        code[0x80 * i:0x80 * i + len(body)] = body
        entry[name] = BASE + 0x80 * i
    return Emu32.raw(bytes(code), base=BASE), entry


def ext(neg, e, mant):
    """An 80-bit value: sign, biased exponent, 64-bit significand."""
    return struct.pack('<QH', mant, (neg << 15) | e)


def hx(b):
    return b[::-1].hex()


class Draw:
    def __init__(self, seed):
        self.r = random.Random(seed)

    def mant(self):
        r = self.r
        kind = r.randrange(6)
        top = 1 << 63
        if kind == 0:                                   # 64 random bits
            return top | r.getrandbits(63)
        if kind == 1:                                   # a Double's 53 bits
            return (top | r.getrandbits(63)) & ~0x7ff
        if kind == 2:                                   # a short value, such as a small integer
            return (top | r.getrandbits(63)) & ~((1 << r.randrange(40, 64)) - 1)
        if kind == 3:                                   # near a tie at 53 or 24 bits
            keep = r.choice((53, 24))
            low = 1 << (63 - keep)
            m = (top | r.getrandbits(63)) & ~((low << 1) - 1)
            return m | low | r.choice((0, 1, low - 1)) if r.random() < 0.7 else m | (low - 1)
        if kind == 4:                                   # all ones below the top bits
            return top | ((1 << 63) - 1) & ~((1 << r.randrange(0, 20)) - 1)
        return top                                      # a power of two

    def value(self, lo=-40, hi=40):
        r = self.r
        if r.random() < 0.03:
            return ext(r.getrandbits(1), 0, 0)          # +0 or -0
        return ext(r.getrandbits(1), 16383 + r.randint(lo, hi), self.mant())

    def pair(self, op):
        r = self.r
        a = self.value()
        k = r.random()
        if op in ('add', 'sub') and k < 0.35:           # near cancellation
            mant, se = struct.unpack('<QH', a)
            if se & 0x7fff == 0:
                return a, self.value()
            d = r.choice((0, 1, -1))
            bm = (mant + r.randint(-(1 << 12), 1 << 12)) | 1 << 63 if r.random() < 0.5 else self.mant()
            sign = (se >> 15) ^ (1 if (op == 'add') else 0)
            return a, ext(sign, (se & 0x7fff) + d, bm & ((1 << 64) - 1))
        if op in ('add', 'sub') and k < 0.55:           # far apart: the smaller is all sticky
            mant, se = struct.unpack('<QH', a)
            e = se & 0x7fff or 16383
            return a, ext(r.getrandbits(1), e - r.randint(50, 140), self.mant())
        if k < 0.6:
            return a, a
        return a, self.value()

    def conv_value(self):
        r = self.r
        k = r.random()
        if k < 0.15:                                    # an integer, a half or near one
            v = r.choice((r.randint(-40000, 40000), r.randint(-(1 << 33), 1 << 33),
                          r.randint(-(1 << 64), 1 << 64)))
            frac = r.choice((0.0, 0.5, 0.25, 0.75))
            n = abs(v) * 4 + int(frac * 4)
            if n == 0:
                return ext(0, 0, 0)
            sh = 64 - n.bit_length()
            return ext(int(v < 0), 16383 + n.bit_length() - 1 - 2, n << sh) if sh >= 0 else self.value()
        if k < 0.25:                                    # past a Double's range, into denormals
            return self.value(-1090, -1015) if r.random() < 0.5 else self.value(1015, 1030)
        if k < 0.35:                                    # past a Single's range
            return self.value(-160, -118) if r.random() < 0.5 else self.value(120, 135)
        return self.value(-70, 70)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('out')
    ap.add_argument('--cases', type=int, default=40, help='cases per operation and control word')
    ap.add_argument('--seed', type=int, default=1987)
    args = ap.parse_args()
    emu, entry = build()
    draw = Draw(args.seed)
    lines = [f'# made by make_vectors.py --cases {args.cases} --seed {args.seed}: x87 under Unicorn '
             '(GameFrame tools/emu/pe32_emu.py)']
    for cw in CONTROL_WORDS:
        emu.write(CW, struct.pack('<H', cw))
        for op in OPS:
            for _ in range(args.cases):
                a, b = draw.pair(op)
                emu.write(A, a)
                emu.write(B, b)
                emu.call(entry[op])
                lines.append(f'{op} {cw:04x} {hx(a)} {hx(b)} {hx(emu.read(R, 10))}')
        for _ in range(args.cases * 2):
            a = draw.conv_value()
            emu.write(A, a)
            emu.call(entry['conv'])
            outs = [emu.read(R64, 8), emu.read(R32, 4), emu.read(I16, 2), emu.read(I32, 4),
                    emu.read(I64, 8), emu.read(RND, 10), emu.read(SQ, 10)]
            lines.append(f'conv {cw:04x} {hx(a)} ' + ' '.join(hx(o) for o in outs))
    Path(args.out).write_text('\n'.join(lines) + '\n')
    print(f'{len(lines) - 1} vectors in {args.out}')


if __name__ == '__main__':
    main()
