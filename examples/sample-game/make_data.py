"""Write the sample game's data file (see `load_records` in game.c) for the format tools to read.

Usage: python3 make_data.py OUT.dat
"""
import struct
import sys

RECORDS = [(1, 0.5, b"oak"), (2, 1.25, b"birch"), (3, 2.0, b"pine")]

with open(sys.argv[1], "wb") as f:
    f.write(b"SMPL" + struct.pack("<I", len(RECORDS)))
    for rid, weight, name in RECORDS:
        f.write(struct.pack("<IfB", rid, weight, len(name)) + name)
