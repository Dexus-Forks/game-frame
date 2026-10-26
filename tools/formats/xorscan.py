#!/usr/bin/env python3
"""Find the key of a simply obfuscated data file: XOR with one byte or a repeating key, adding
a byte, a key that steps by a fixed amount each byte, or each byte XORed with the one before.

Usage: xorscan.py FILE [--offset N] [--length N] [--known HEX@OFFSET] [--top N]
       xorscan.py FILE --apply SCHEME -o OUT
       xorscan.py FILE --records START:STRIDE:FIELD:LENGTH[:COUNT] [--key-in BINARY]

  --offset, --length   scan only part of the file (a header's encrypted body, one entry)
  --known HEX@OFFSET   bytes you know are there once decoded (a magic number, a name, zeros),
                       for example 52494646@0 for "RIFF" at the start
  --top N              how many candidates to show (default 8)
  --apply SCHEME       decode the whole file with one result line's scheme, such as
                       xor:5a  xor-key:1f2e3d  add:80  step:5a+3  chain:00  rol:3
  --records ...        a table of fixed-size records (an archive's directory) where one field
                       of each record, such as the entry name, is XORed with the same key.
                       START is the table's offset, STRIDE the record size, FIELD the field's
                       offset in the record, LENGTH its size, COUNT the records (default: to the
                       end of the file). Names are padded with zero bytes, and a zero XORed with
                       the key is the key itself: the stored bytes of the padding are the key.
                       Prints the key bytes the padding shows, ?? for the columns it never
                       reaches
  --key-in BINARY      with --records: look in BINARY (the game's executable keeps the key) for
                       the bytes the padding showed, and take the whole key from there when it
                       turns every field into a name. Prints where it is stored, with the
                       virtual address for a PE file, and the first decoded fields

Many games hide their text, scripts or tables with a simple XOR so that people cannot read
them in a file viewer. The schemes are weak, and the decoded bytes give the key away: game
data has many zero bytes and much ASCII. Each candidate is scored by how much of the result
is zeros or printable text, with a bonus when a known format's signature appears. Read the
preview, then confirm the scheme in the code that reads the file (re-format, re-trace) before
porting it: the code may also compress, or change the key per file or per entry.

If nothing scores well, the file may be compressed (filetypes.py shows its entropy) or use
a real cipher. Then find the decoder in the code. Decoded files are game material: write
them into game/ or another folder that git ignores. Standard library only.
"""
import os
import sys

MAGICS = [b'RIFF', b'MZ', b'PK\x03\x04', b'FORM', b'MThd', b'BM', b'\x89PNG', b'GIF8', b'OggS',
          b'<?xml', b'IWAD', b'PWAD', b'PACK', b'Creative', b'SQLite']


def score(d):
    if not d:
        return 0.0
    good = sum(1 for b in d if b == 0 or 32 <= b < 127 or b in (9, 10, 13))
    s = good / len(d)
    if any(d.startswith(m) for m in MAGICS):
        s += 0.5
    return s


def decode(data, scheme, start=0):
    """Apply a scheme. `start` is the file offset of data[0], for schemes whose key depends on
    the position (a repeating key, a stepping key)."""
    kind, _, arg = scheme.partition(':')
    out = bytearray(len(data))
    if kind == 'xor':
        k = int(arg, 16)
        for i, b in enumerate(data):
            out[i] = b ^ k
    elif kind == 'xor-key':
        key = bytes.fromhex(arg)
        for i, b in enumerate(data):
            out[i] = b ^ key[(start + i) % len(key)]
    elif kind == 'add':
        k = int(arg, 16)
        for i, b in enumerate(data):
            out[i] = (b + k) & 0xFF
    elif kind == 'step':
        k0, step = (int(x, 16) for x in arg.split('+'))
        for i, b in enumerate(data):
            out[i] = b ^ ((k0 + step * (start + i)) & 0xFF)
    elif kind == 'chain':
        prev = int(arg, 16)
        for i, b in enumerate(data):
            out[i] = b ^ prev
            prev = b
    elif kind == 'rol':
        n = int(arg) % 8
        for i, b in enumerate(data):
            out[i] = ((b << n) | (b >> (8 - n))) & 0xFF
    else:
        raise SystemExit(f'unknown scheme {scheme}')
    return bytes(out)


def key_lengths(data, top=4, longest=32):
    """Likely repeating-key lengths: the length whose columns repeat values most often."""
    out = []
    for n in range(2, min(longest, len(data) // 8) + 1):
        coinc = 0.0
        for c in range(n):
            col = data[c::n]
            counts = {}
            for b in col:
                counts[b] = counts.get(b, 0) + 1
            coinc += sum(v * (v - 1) for v in counts.values()) / max(1, len(col) * (len(col) - 1))
        out.append((coinc / n, n))
    out.sort(reverse=True)
    return [n for _, n in out[:top]]


def repeating_key(data, n, start):
    """Each key byte: the value that turns its column's most common byte into zero, or into a
    space when that gives more printable text."""
    key = bytearray(n)
    for c in range(n):
        col = data[c::n]
        counts = {}
        for b in col:
            counts[b] = counts.get(b, 0) + 1
        common = max(counts, key=counts.get)
        best = max((common, common ^ 0x20), key=lambda k: score(bytes(b ^ k for b in col)))
        key[(start + c) % n] = best
    for p in range(1, n):                     # a multiple of the real length: keep one period
        if n % p == 0 and all(key[i] == key[i % p] for i in range(n)):
            return bytes(key[:p])
    return bytes(key)


def candidates(data, start):
    out = []
    for k in range(256):
        out.append(f'xor:{k:02x}')
        out.append(f'add:{k:02x}')
        out.append(f'chain:{k:02x}')
    for n in range(1, 8):
        out.append(f'rol:{n}')
    # A key that steps: try each step with the first byte's best start value.
    for step in range(1, 256):
        best = max(range(256), key=lambda k0: score(decode(data[:256], f'step:{k0:02x}+{step:02x}', start)))
        out.append(f'step:{best:02x}+{step:02x}')
    for n in key_lengths(data):
        out.append(f'xor-key:{repeating_key(data, n, start).hex()}')
    return out


def from_known(data, start, known):
    hexpart, _, at = known.partition('@')
    plain, at = bytes.fromhex(hexpart), int(at, 0) - start
    cipher = data[at:at + len(plain)]
    if len(cipher) < len(plain):
        return []
    key = bytes(c ^ p for c, p in zip(cipher, plain))
    out = [f'xor:{key[0]:02x}'] if len(set(key)) == 1 else []
    for n in range(2, len(key) // 2 + 1):     # the shortest period that explains the bytes
        if all(key[i] == key[i % n] for i in range(len(key))):
            rot = (start + at) % n
            out.append(f'xor-key:{(key[-rot:] + key[:-rot]).hex() if rot else key[:n].hex()}')
            break
    out.append(f'add:{(plain[0] - cipher[0]) & 0xFF:02x}')
    return out


NAME_CHARS = set(b'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 ._-~!$#')


def record_key(data, spec):
    """The key bytes that zero padding shows in one XORed field of every record, right to left.
    A column's key byte is known when at least two records still pad there: their stored byte
    is the key itself. To the left of the shortest padding nothing is known (None)."""
    parts = [int(x, 0) for x in spec.split(':')]
    start, stride, field, length = parts[:4]
    count = parts[4] if len(parts) > 4 else (len(data) - start) // stride
    fields = [data[start + i * stride + field:start + i * stride + field + length] for i in range(count)]
    fields = [f for f in fields if len(f) == length]
    key = [None] * length
    padding = list(range(len(fields)))              # records that are still padding at this column
    for c in reversed(range(length)):
        counts = {}
        for i in padding:
            counts[fields[i][c]] = counts.get(fields[i][c], 0) + 1
        if not counts:
            break
        k = max(counts, key=counts.get)
        if counts[k] < 2 or counts[k] * 2 < len(padding):
            break
        key[c] = k
        padding = [i for i in padding if fields[i][c] == k]
    return key, fields


def names_fit(fields, key):
    """True when the key turns every field into name characters followed by zero padding."""
    for f in fields:
        plain = bytes(a ^ b for a, b in zip(f, key)).rstrip(b'\0')
        if not plain or any(b not in NAME_CHARS for b in plain):
            return False
    return True


def show_records(whole, spec, key_in):
    key, fields = record_key(whole, spec)
    known = sum(k is not None for k in key)
    shown = ' '.join('??' if k is None else f'{k:02x}' for k in key)
    text = ''.join('?' if k is None else chr(k) if 32 <= k < 127 else '.' for k in key)
    print(f'{len(fields)} records; zero padding shows {known} of {len(key)} key bytes: {shown}  "{text}"')
    if known and key_in:
        exe = open(key_in, 'rb').read()
        sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'triage'))
        from constants import pe_mapper
        to_va = pe_mapper(exe)
        first = next(c for c, k in enumerate(key) if k is not None)
        found = set()
        # The left end of what the padding showed is the least sure: try shorter tails too.
        for skip in range(first, max(first + 1, len(key) - 3)):
            tail = bytes(key[skip:])
            at = exe.find(tail)
            while at >= 0:
                full = exe[at - skip:at - skip + len(key)] if at >= skip else b''
                if len(full) == len(key) and at - skip not in found and names_fit(fields, full):
                    found.add(at - skip)
                    va = to_va(at - skip)[0] if to_va else None
                    text = ''.join(chr(b) if 32 <= b < 127 else '.' for b in full)
                    print(f'key {full.hex()}  "{text}" stored in {key_in} at file offset 0x{at - skip:x}'
                          + (f', address 0x{va:x}' if va else '') + '; every field decodes to a name')
                    for f in fields[:8]:
                        print('  ' + preview(bytes(a ^ b for a, b in zip(f, full)).rstrip(b'\0')))
                at = exe.find(tail, at + 1)
            if found:
                break
        if not found:
            print(f'no stored copy of the known bytes in {key_in} gives readable names')
    elif known:
        print('find the rest: --key-in the executable, or --known with a name you expect')
    print('Confirm the key in the code that reads the table (re-trace) before porting it.')


def preview(d):
    return ''.join(chr(b) if 32 <= b < 127 else '.' for b in d[:64])


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__)
        return 0 if argv else 2
    opt = lambda name, default=None: argv[argv.index(name) + 1] if name in argv else default
    with open(argv[0], 'rb') as f:
        whole = f.read()
    if '--apply' in argv:
        out = opt('-o')
        if not out:
            raise SystemExit('--apply needs -o OUT')
        with open(out, 'wb') as f:
            f.write(decode(whole, opt('--apply')))
        print(f'wrote {out}')
        return 0
    if '--records' in argv:
        show_records(whole, opt('--records'), opt('--key-in'))
        return 0
    start = int(opt('--offset', '0'), 0)
    length = int(opt('--length', str(min(len(whole) - start, 1 << 16))), 0)
    data = whole[start:start + length]
    schemes = candidates(data, start)
    if '--known' in argv:
        schemes = from_known(whole, 0, opt('--known')) + schemes
    seen, rows = set(), []
    for s in schemes:
        if s in seen:
            continue
        seen.add(s)
        d = decode(data, s, start)
        rows.append((score(d), s, d))
    rows.sort(key=lambda r: (-r[0], len(r[1])))   # on a tie, the simpler scheme
    print(f'as stored  score {score(data):.2f}  {preview(data)}')
    for sc, s, d in rows[:int(opt('--top', '8'))]:
        print(f'{s:28s} score {sc:.2f}  {preview(d)}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
