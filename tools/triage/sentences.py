#!/usr/bin/env python3
"""List the sentences in a program: the long strings a person wrote for players, such as
messages, mission text, item descriptions and notes left by the developers.

Usage: sentences.py FILE [--min-words N] [--grep WORD] [--sort address|words] [--json]

  FILE           a binary (any kind: PE, DOS, a data file, a memory dump), or the strings.txt
                 of a Ghidra export (lines of ADDRESS<TAB>TEXT), read as is
  --min-words N  the fewest words a sentence has (default 5)
  --grep WORD    only sentences that contain WORD (case-blind; repeat for any of several)
  --sort         address (default: the order the program keeps them, which is often the order
                 of the story) or words (longest first)
  --json         one record per sentence: offset, virtual address, encoding, words, text

A game's story and its rules are often written in its own text before they are visible in its
code (docs/working/lore.md). This tool finds that text. From a binary it reads runs of
printable bytes (ASCII and Latin-1) and UTF-16LE text, splits them at line breaks, and keeps
the pieces that read like sentences: at least N words, mostly letters, with spaces between
the words. Format codes such as %s and %d count as words and are kept, so templates like
"%s is low on power" appear too. For a Windows PE file each hit also gets its virtual address,
which finds the code that uses it (refs, decomp_query).

It does not decode compressed or encrypted text (xorscan.py, filetypes.py), text built on the
stack (stackstrings.py), or text in a custom encoding. The output is the game's own text: write
it into analysis/ or another ignored folder, never into a tracked file. Standard library only.
"""
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from constants import pe_mapper  # noqa: E402

WORD = re.compile(r"%[-+ #0-9.]*[a-zA-Z]|[A-Za-zÀ-ÿ][A-Za-zÀ-ÿ'\-]*")
PRINTABLE = set(range(32, 127)) | set(range(0xa0, 0x100)) | {9, 10, 13}


def runs8(data):
    """(offset, text) for every run of printable single-byte text of 12 bytes or more."""
    start = None
    for i, b in enumerate(data + b'\0'):
        if b in PRINTABLE:
            if start is None:
                start = i
        elif start is not None:
            if i - start >= 12:
                yield start, data[start:i].decode('latin-1')
            start = None


def runs16(data):
    """(offset, text) for every run of UTF-16LE text of 12 characters or more, ASCII range."""
    for parity in (0, 1):
        start = None
        for i in range(parity, len(data) + 1, 2):
            ok = i + 1 < len(data) and data[i + 1] == 0 and data[i] in PRINTABLE and data[i] < 0x80
            if ok:
                if start is None:
                    start = i
            elif start is not None:
                if (i - start) // 2 >= 12:
                    yield start, data[start:i].decode('utf-16-le')
                start = None


def is_sentence(text, min_words):
    words = WORD.findall(text)
    if len(words) < min_words:
        return 0
    letters = sum(ch.isalpha() for ch in text)
    if letters < 0.6 * len(text.replace(' ', '')) or text.count(' ') < min_words - 1:
        return 0
    if sum(len(w) for w in words) / len(words) > 14:    # long runs with no spaces: tables, keys
        return 0
    return len(words)


# A few stray bytes before the text, which happened to be printable: "/The ..", "L\xcd!This ..".
LEAD = re.compile(r'^.{0,3}?[^A-Za-z0-9 ,.\'"%(-]+(?=[A-Z%"])')


def pieces(offset, text):
    """Split at line breaks (real ones, or a Ghidra export's escaped \\n), keeping offsets."""
    at = 0
    for part in re.split(r'(\r?\n|\\n|\\r)', text):
        if part not in ('\n', '\r\n', '\\n', '\\r') and part.strip():
            lead = len(part) - len(part.lstrip())
            body = part.strip()
            cut = LEAD.match(body)
            if cut:
                lead, body = lead + cut.end(), body[cut.end():]
            yield offset + at + lead, body
        at += len(part)


def from_binary(path, min_words):
    data = open(path, 'rb').read()
    to_va = pe_mapper(data)
    out = []
    for enc, finder, width in (('ascii', runs8, 1), ('utf-16le', runs16, 2)):
        for off, text in finder(data):
            for o, piece in pieces(0, text):
                n = is_sentence(piece, min_words)
                if n:
                    at = off + o * width
                    va = to_va(at)[0] if to_va else None
                    out.append({'offset': at, 'va': va, 'encoding': enc, 'words': n, 'text': piece})
    return out


def from_export(path, min_words):
    out = []
    for line in open(path, encoding='utf-8', errors='replace'):
        addr, _, text = line.rstrip('\n').partition('\t')
        try:
            va = int(addr.split(':')[-1], 16)
        except ValueError:
            continue
        for _, piece in pieces(0, text):
            n = is_sentence(piece, min_words)
            if n:
                out.append({'offset': None, 'va': va, 'encoding': 'export', 'words': n, 'text': piece})
    return out


def looks_like_export(path):
    try:
        head = open(path, 'rb').read(4096)
    except OSError:
        return False
    lines = [l for l in head.split(b'\n')[:20] if l]
    return bool(lines) and b'\0' not in head and all(re.match(rb'^[0-9a-fA-F:]+\t', l) for l in lines[:-1] or lines)


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__)
        return 0 if argv else 2
    opt = lambda name, default=None: argv[argv.index(name) + 1] if name in argv else default
    path, min_words = argv[0], int(opt('--min-words', '5'))
    rows = (from_export if looks_like_export(path) else from_binary)(path, min_words)
    greps = [argv[i + 1].lower() for i, a in enumerate(argv) if a == '--grep' and i + 1 < len(argv)]
    if greps:
        rows = [r for r in rows if any(g in r['text'].lower() for g in greps)]
    key = (lambda r: -r['words']) if opt('--sort') == 'words' else (lambda r: (r['va'] or 0, r['offset'] or 0))
    rows.sort(key=key)
    if '--json' in argv:
        print(json.dumps(rows, indent=1))
        return 0
    for r in rows:
        where = f'0x{r["va"]:08x}' if r['va'] is not None else f'@0x{r["offset"]:08x}'
        tag = ' [utf-16]' if r['encoding'] == 'utf-16le' else ''
        print(f'{where}  {r["words"]:3d}  {r["text"]}{tag}')
    print(f'# {len(rows)} sentences, {sum(r["words"] for r in rows)} words, at least {min_words} words each')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
