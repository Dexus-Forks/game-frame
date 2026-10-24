#!/usr/bin/env python3
"""Compare two editions of a game, folder against folder or disc image against disc image, and
write the data delta for docs/versions.md: files added, removed and changed, by folder, with
the executables' link dates and the languages each edition has.

Usage: versiondiff.py OLD NEW [--depth N] [--list N] [--json]

  OLD, NEW   a folder (an install, an extracted disc) or a disc image (.cue, .bin, .iso),
             read in place with tools/disc: nothing is extracted
  --depth N  group the table by the first N folders of each path (default 2)
  --list N   list at most N files of each kind (added, removed, changed) (default 40)
  --json     everything as JSON

It prints Markdown, ready to paste into the project's docs/versions.md:

- a summary: files and bytes in each edition, and how many were added, removed, changed and
  kept;
- a table by folder: added, removed, changed and the same;
- the executables and DLLs of each edition with their PE link dates (the date the linker
  wrote, which dates a build better than file dates), and which changed;
- the languages each edition has, found from folder and file names (ENGLISH, FRANCAIS, DE, ...);
- the files added, removed and changed, with their sizes.

Files are matched by path, ignoring case, and compared by MD5 (shown in the JSON). A file
moved to another folder shows as removed and added: tools/triage/compare_trees.py --similar
pairs renamed files. Code changes need the Ghidra exports of both: match their functions
with tools/triage/match_functions.py (docs/working/versions.md).

It reads the editions only, and prints no file content: the table is safe to commit. Standard
library only.
"""
import hashlib
import json
import os
import re
import struct
import sys
from datetime import datetime, timezone

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'disc'))

IMAGE = ('.cue', '.bin', '.iso', '.img')
LANGS = {
    'english': 'English', 'anglais': 'English', 'en': 'English', 'eng': 'English', 'uk': 'English', 'us': 'English',
    'french': 'French', 'francais': 'French', 'fr': 'French', 'fra': 'French', 'fre': 'French',
    'german': 'German', 'deutsch': 'German', 'allemand': 'German', 'de': 'German', 'ger': 'German', 'deu': 'German',
    'italian': 'Italian', 'italiano': 'Italian', 'italien': 'Italian', 'it': 'Italian', 'ita': 'Italian',
    'spanish': 'Spanish', 'espanol': 'Spanish', 'espagnol': 'Spanish', 'es': 'Spanish', 'spa': 'Spanish',
    'dutch': 'Dutch', 'nederlands': 'Dutch', 'nl': 'Dutch', 'japanese': 'Japanese', 'jp': 'Japanese',
    'jpn': 'Japanese', 'portuguese': 'Portuguese', 'pt': 'Portuguese', 'swedish': 'Swedish', 'polish': 'Polish',
    'russian': 'Russian', 'ru': 'Russian', 'korean': 'Korean', 'chinese': 'Chinese',
}
# Short codes count only as a whole folder name, or a file name's last part (TEXT_FR.DAT).
SHORT = {k for k in LANGS if len(k) <= 3}


class File:
    def __init__(self, path, size, read):
        self.path, self.size, self._read, self._md5, self._pe = path, size, read, None, False

    def data(self):
        return self._read()

    @property
    def md5(self):
        if self._md5 is None:
            self._md5 = hashlib.md5(self.data()).hexdigest()
        return self._md5

    @property
    def link_date(self):
        if self._pe is False:
            self._pe = pe_date(self.data()) if os.path.splitext(self.path)[1].lower() in ('.exe', '.dll') else None
        return self._pe


def pe_date(data):
    """The PE header's TimeDateStamp as an ISO date, or None for a file that is not a PE."""
    if len(data) < 0x40 or data[:2] != b'MZ':
        return None
    off = struct.unpack_from('<I', data, 0x3c)[0]
    if off + 12 > len(data) or data[off:off + 4] != b'PE\0\0':
        return None
    stamp = struct.unpack_from('<I', data, off + 8)[0]
    return datetime.fromtimestamp(stamp, timezone.utc).strftime('%Y-%m-%d %H:%M') if stamp else None


def from_folder(root):
    out = {}
    for d, _, files in os.walk(root):
        for f in files:
            p = os.path.join(d, f)
            rel = os.path.relpath(p, root).replace(os.sep, '/')
            out[rel.lower()] = File(rel, os.path.getsize(p), lambda p=p: open(p, 'rb').read())
    return out


def from_image(path):
    from image import open_image
    from volumes import scan
    disc = open_image(path)
    volumes = [v for t in disc.tracks for v in scan(t) if v.root is not None]
    if not volumes:
        raise SystemExit(f'versiondiff: {path}: no readable file system')
    chosen = next((v for v in volumes if v.joliet), volumes[0])
    out = {}
    for e in chosen.entries():
        if not e.directory:
            out[e.path.lower()] = File(e.path, e.size, lambda e=e, v=chosen: v.file(e))
    return out


def load(path):
    if os.path.isdir(path):
        return from_folder(path)
    if os.path.splitext(path)[1].lower() in IMAGE:
        return from_image(path)
    raise SystemExit(f'versiondiff: {path} is neither a folder nor a disc image (.cue, .bin, .iso)')


def languages(files):
    found = {}
    for f in files.values():
        parts = f.path.lower().split('/')
        names = parts[:-1] + [re.split(r'[._-]', os.path.splitext(parts[-1])[0])[-1]]
        for n in names:
            n = n.replace('ç', 'c').replace('ñ', 'n')
            if n in LANGS and (n not in SHORT or n == names[-1] or n in parts[:-1]):
                found.setdefault(LANGS[n], set()).add('/'.join(parts[:-1]) or '.')
    return {k: sorted(v) for k, v in sorted(found.items())}


def group(path, depth):
    parts = path.split('/')[:-1]
    return '/'.join(parts[:depth]) or '.'


def compare(old, new, depth=2):
    a, b = load(old), load(new)
    added = sorted(set(b) - set(a))
    removed = sorted(set(a) - set(b))
    both = sorted(set(a) & set(b))
    changed = [k for k in both if a[k].size != b[k].size or a[k].md5 != b[k].md5]
    folders, spelling = {}, {}
    for kind, keys, side in (('added', added, b), ('removed', removed, a), ('changed', changed, b),
                             ('same', [k for k in both if k not in set(changed)], b)):
        for k in keys:
            name = group(side[k].path, depth)
            spelling.setdefault(name.lower(), name)       # folders match in any case, as files do
            row = folders.setdefault(name.lower(), {'added': 0, 'removed': 0, 'changed': 0, 'same': 0})
            row[kind] += 1
    exes = {}
    for side, files in (('old', a), ('new', b)):
        for k, f in files.items():
            if os.path.splitext(k)[1] in ('.exe', '.dll'):
                exes.setdefault(k, {'path': f.path})[side] = f.link_date
    return {
        'old': {'path': old, 'files': len(a), 'bytes': sum(f.size for f in a.values()), 'languages': languages(a)},
        'new': {'path': new, 'files': len(b), 'bytes': sum(f.size for f in b.values()), 'languages': languages(b)},
        'added': [{'path': b[k].path, 'size': b[k].size, 'md5': b[k].md5} for k in added],
        'removed': [{'path': a[k].path, 'size': a[k].size, 'md5': a[k].md5} for k in removed],
        'changed': [{'path': b[k].path, 'old_size': a[k].size, 'new_size': b[k].size,
                     'old_md5': a[k].md5, 'new_md5': b[k].md5} for k in changed],
        'same': len(both) - len(changed),
        'folders': {spelling[k]: v for k, v in sorted(folders.items())},
        'executables': [dict(v, changed=k in set(changed)) for k, v in sorted(exes.items())],
    }


def markdown(r, limit):
    o, n = r['old'], r['new']
    lines = [f'Compared with `tools/triage/versiondiff.py`: {o["files"]} files ({o["bytes"]:,} bytes) in the old '
             f'edition, {n["files"]} ({n["bytes"]:,} bytes) in the new. {len(r["added"])} added, '
             f'{len(r["removed"])} removed, {len(r["changed"])} changed, {r["same"]} the same.', '',
             '| Folder | Added | Removed | Changed | Same |', '|---|---|---|---|---|']
    for name, row in r['folders'].items():
        lines.append(f'| `{name}` | {row["added"]} | {row["removed"]} | {row["changed"]} | {row["same"]} |')
    if r['executables']:
        lines += ['', '| Executable | Old link date | New link date | Changed |', '|---|---|---|---|']
        for e in r['executables']:
            lines.append(f'| `{e["path"]}` | {e.get("old") or "-" if "old" in e else "(none)"} | '
                         f'{e.get("new") or "-" if "new" in e else "(none)"} | {"yes" if e["changed"] else ""} |')
    lines += ['', '| Language | Old | New |', '|---|---|---|']
    for lang in sorted(set(o['languages']) | set(n['languages'])):
        lines.append(f'| {lang} | {"yes" if lang in o["languages"] else ""} | {"yes" if lang in n["languages"] else ""} |')
    for kind, rows in (('Added', r['added']), ('Removed', r['removed']), ('Changed', r['changed'])):
        if rows:
            lines += ['', f'{kind} ({len(rows)}):', '']
            for x in rows[:limit]:
                size = (f'{x["old_size"]:,} -> {x["new_size"]:,} bytes' if kind == 'Changed'
                        else f'{x["size"]:,} byte' + ('' if x['size'] == 1 else 's'))
                lines.append(f'- `{x["path"]}` ({size})')
            if len(rows) > limit:
                lines.append(f'- and {len(rows) - limit} more (--list N, or --json)')
    return '\n'.join(lines)


def main(argv):
    if len(argv) < 2 or argv[0] in ('-h', '--help'):
        print(__doc__.rstrip())
        return 0 if argv and argv[0] in ('-h', '--help') else 2
    opt = lambda k, d: int(argv[argv.index(k) + 1]) if k in argv else d
    args = [x for i, x in enumerate(argv) if not x.startswith('--') and argv[i - 1] not in ('--depth', '--list')]
    r = compare(args[0], args[1], opt('--depth', 2))
    if '--json' in argv:
        print(json.dumps(r, indent=2))
    else:
        print(markdown(r, opt('--list', 40)))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
