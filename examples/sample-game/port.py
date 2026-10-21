"""A reimplementation of the sample game's `simulate`, written from the decompiled code, that must
match the original bit for bit.

Usage: python port.py [--naive] OUT_DIR    (writes seed-N.f32 and seed-N.trace like ground_truth.py)

`--naive` makes the most common mistake: it keeps the same formulas in the same order but
computes the simulation in 64-bit doubles (Python's default) instead of the original's 32-bit
floats. Run tools/verify/bitcompare.py on its output to see the drift that causes.

The original computes in 32-bit floats, so every operation here rounds to f32 (`f32()`), in the
original's order: `(rng_float + 1.0) * dt`, not `dt + dt * rng_float`, which can differ in the last
bit. A port in a language with a real f32 type (Rust, C, C++, C#) gets this for free if it keeps
the types and the order of operations.
"""
import os
import struct
import sys

GROWTH = (0.05, 0.125, 0.3333333, 1.0)  # read from the binary with tools/pe/readdata.py
STEPS = 600


def f32(x):
    return struct.unpack('<f', struct.pack('<f', x))[0]


class Random:
    """The engine's generator: MSVC rand() constants. Original: engine.dll rng_next, rng_float."""

    def __init__(self, seed, trace):
        self.state = seed & 0xFFFFFFFF
        self.trace = trace

    def next(self, site='port.py:Random.next'):
        self.trace.append(f'rng_next\t{site}')
        return self._next()

    def _next(self):
        self.state = (self.state * 214013 + 2531011) & 0xFFFFFFFF
        return (self.state >> 16) & 0x7FFF

    def float(self, site='port.py:Random.float'):
        self.trace.append(f'rng_float\t{site}')  # one draw, as the original counts it
        return f32(f32(self._next()) * f32(1.0 / 32768.0))


def simulate(seed, steps, naive=False):
    """Original: game.exe simulate (tree_update and tree_value inlined through the vtable)."""
    global f32
    if naive:
        f32 = float  # no rounding to 32 bits: doubles throughout
    trace = []
    rng = Random(seed, trace)
    growth = [f32(g) for g in GROWTH]
    age, stage = f32(0.0), 0
    out = []
    for _ in range(steps):
        dt = f32(1.0 / 60.0)
        if rng.next('port.py:simulate') % 4 == 0:
            dt = f32(f32(rng.float('port.py:simulate') + f32(1.0)) * dt)
        age = f32(age + f32(dt * growth[stage]))
        if age >= 1.0 and stage < 3:
            age = f32(age - f32(1.0))
            stage += 1
        out.append(f32(f32(stage) + age))
    return out, trace


def main(out, naive=False):
    os.makedirs(out, exist_ok=True)
    for seed in range(1, 21):
        values, trace = simulate(seed, STEPS, naive)
        with open(os.path.join(out, f'seed-{seed}.f32'), 'wb') as f:
            f.write(struct.pack(f'<{len(values)}f', *values))
        with open(os.path.join(out, f'seed-{seed}.trace'), 'w') as f:
            f.write('\n'.join(trace) + '\n')
    print(f'wrote 20 seeds to {out}')


if __name__ == '__main__':
    args = [a for a in sys.argv[1:] if a != '--naive']
    default = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'build', 'port')
    main(args[0] if args else default, naive='--naive' in sys.argv)
