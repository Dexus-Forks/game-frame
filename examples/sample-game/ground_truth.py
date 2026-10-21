"""Capture ground truth from the sample game: run the original `simulate` under the emulator.

Usage: python ground_truth.py OUT_DIR

For seeds 1 to 20 it writes OUT_DIR/seed-N.f32 (600 little-endian floats, the tree's value
after each step) and OUT_DIR/seed-N.trace (every random draw, in order, with its call site).
This is step "emulate" of the method (skills/re-emulate): the reimplementation in port.py must
reproduce both files exactly. In a real project the outputs go to a git-ignored folder, since
they are derived from the original game.
"""
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '..', '..', 'tools', 'emu'))
from call_trace import CallTrace  # noqa: E402
from pe_emu import Emu  # noqa: E402

STEPS = 600


def main(out):
    os.makedirs(out, exist_ok=True)
    build = os.path.join(HERE, 'build')
    for seed in range(1, 21):
        # A fresh emulator per seed, so no state leaks between runs.
        emu = Emu(os.path.join(build, 'game.exe'), [os.path.join(build, 'engine.dll')])
        trace = CallTrace(emu, {emu.export('engine.dll', 'rng_next'): 'rng_next',
                                emu.export('engine.dll', 'rng_float'): 'rng_float'})
        values = emu.malloc(4 * STEPS)
        emu.call(emu.export('game.exe', 'simulate'), seed, STEPS, values)
        with open(os.path.join(out, f'seed-{seed}.f32'), 'wb') as f:
            f.write(emu.read(values, 4 * STEPS))
        trace.write(os.path.join(out, f'seed-{seed}.trace'))
    last = struct.unpack('<f', open(os.path.join(out, 'seed-20.f32'), 'rb').read()[-4:])[0]
    print(f'wrote 20 seeds to {out} (seed 20 ends at {last!r})')


if __name__ == '__main__':
    main(sys.argv[1] if len(sys.argv) > 1 else os.path.join(HERE, 'build', 'truth'))
