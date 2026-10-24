#!/usr/bin/env python3
"""Run a decoder over every file of a type and report which files fail or leave bytes unread.

Usage: validate.py DECODER.py GLOB [GLOB ...] [--show N] [--write]

DECODER.py defines `decode(reader)`, which reads one whole file from a `Reader` (reader.py) and
returns what it found; validate.py then calls `reader.done()`, so a file with unread bytes
counts as a failure. A decoder is right only when every file of its type passes: one sample is
not enough, because optional fields and version differences show up in a few files only.
`--show N` prints the result of the first N files. Exit status 1 if any file fails.

`--write` also checks the writer: DECODER.py then defines `encode(result)`, which returns the
bytes of a file from what `decode` returned. Each file is decoded, encoded again and compared
with the original, byte for byte; a file that differs fails, with the first differing offset
and both bytes. A reader is a writer only when every file of the player's copy comes back
identical (docs/knowhow/writer-round-trip-every-file.md). Change one value only after that.

    validate.py examples/sample-game/decode_sample.py 'examples/sample-game/build/*.dat' --show 1
"""
import glob
import importlib.util
import os
import pprint
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from reader import FormatError, Reader  # noqa: E402


def main(argv):
    show = 0
    write = '--write' in argv
    argv = [a for a in argv if a != '--write']
    if '--show' in argv:
        i = argv.index('--show')
        show = int(argv[i + 1])
        argv = argv[:i] + argv[i + 2:]
    decoder_path, *patterns = argv
    spec = importlib.util.spec_from_file_location('decoder', decoder_path)
    decoder = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(decoder)
    if write and not hasattr(decoder, 'encode'):
        sys.exit(f'{decoder_path} has no encode(result): --write needs one')
    files = sorted({f for p in patterns for f in glob.glob(p, recursive=True)})
    if not files:
        sys.exit('no files match')
    failed = 0
    for i, path in enumerate(files):
        data = open(path, 'rb').read()
        r = Reader(data, name=os.path.basename(path))
        try:
            result = decoder.decode(r)
            r.done()
        except (FormatError, ValueError, IndexError, UnicodeDecodeError) as e:
            failed += 1
            print(f'FAIL {path}: {e}')
            continue
        if write:
            try:
                again = bytes(decoder.encode(result))
            except Exception as e:      # a writer's bug is a failed file, not a crash
                failed += 1
                print(f'FAIL {path}: encode: {type(e).__name__}: {e}')
                continue
            if again != data:
                failed += 1
                at = next((k for k, (a, b) in enumerate(zip(data, again)) if a != b), min(len(data), len(again)))
                was = f'{data[at]:#04x}' if at < len(data) else 'end'
                now = f'{again[at]:#04x}' if at < len(again) else 'end'
                print(f'FAIL {path}: written back, it differs at offset {at:#x} (file {was}, written {now}); '
                      f'{len(data)} bytes in, {len(again)} out')
                continue
        if i < show:
            print(f'{path}:')
            pprint.pprint(result, width=100)
    what = 'decode completely and write back identical' if write else 'decode completely'
    print(f'{len(files) - failed} of {len(files)} files {what}')
    sys.exit(1 if failed else 0)


if __name__ == '__main__':
    if len(sys.argv) < 3:
        print(__doc__)
        sys.exit(2)
    main(sys.argv[1:])
