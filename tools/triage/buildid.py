#!/usr/bin/env python3
"""Pin down which build of a game this is: everything a copy says about its own version.

Usage: buildid.py SOURCE [--name LABEL] [--write docs/versions.md] [--json]

  SOURCE   a folder (an install, an extracted disc), a disc image (.cue, .bin, .iso) read in
           place, or one executable
  --name   what to call this copy in the table (default: the folder or file name)
  --write  put the result into that file, between `<!-- buildid LABEL -->` and
           `<!-- /buildid LABEL -->`, replacing an earlier result for the same label, or under a
           "## Build identity" heading at the end when there is none yet
  --json   everything as JSON

What it reads:

- the disc's volume descriptor, for a disc image: label, the date it was mastered (created) and
  last modified, publisher, data preparer and mastering application;
- each executable and DLL: the PE link date, and the VERSIONINFO resource (product name,
  product and file version, company, description);
- version strings inside each executable ("Version 2.8", "v1.0", "Ver 1.02"), from its ASCII and
  UTF-16 text;
- version lines in the copy's own text files, such as readme files, `setup.ini`, `autorun.inf`,
  `*.nfo` and `*.txt` near the top: lines that name a version, a patch or an update, or give a
  date.

It prints Markdown for the "Build identity" section of the project's `docs/versions.md`. Then
compare it with the official releases you find (re-inventory, "Every edition, and the
reference"): is this the latest official build, and if not, which is?

It reads only the copy, and prints short identifying lines from it, not its content. Version
lines from a readme are quoted, at most a line each, so the table stays publishable. Needs
pefile (.venv) for VERSIONINFO; the rest uses the standard library.
"""
import json
import os
import re
import sys
from datetime import date

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(os.path.dirname(HERE), 'pe'))
sys.path.insert(0, os.path.join(os.path.dirname(HERE), 'disc'))
import versiondiff  # noqa: E402

VERSION = re.compile(r'(?i)(?:\bversion|\bver\.?|\brev(?:ision)?\.?|(?<![a-z0-9])v)\s?(\d{1,2}(?:\.\d{1,4}){1,3}[a-z]?)\b')
MONTHS = r'(?:jan|feb|mar|apr|may|jun|jul|aug|sep|sept|oct|nov|dec)[a-z]*'
DATE = re.compile(rf'(?i)\b(?:{MONTHS}\.?\s+\d{{1,2}}(?:st|nd|rd|th)?,?\s+(?:19|20)\d\d|\d{{1,2}}(?:st|nd|rd|th)?\s+{MONTHS}\s+(?:19|20)\d\d'
                  rf'|(?:19|20)\d\d-\d\d-\d\d|\d{{1,2}}[/.]\d{{1,2}}[/.](?:19|20)\d\d)\b')
WORDS = re.compile(r'(?i)\b(?:patch|update|release|build)\b')
TEXT_NAMES = re.compile(r'(?i)^(?:read[ _-]?me.*|.*\.txt|.*\.nfo|setup\.ini|.*\.ini|autorun\.inf|whatsnew.*|version.*|history.*|changes.*)$')
PER_FILE = 8


def strings(data, minimum=5):
    """ASCII and UTF-16LE runs of printable text."""
    out = [m.group().decode('ascii') for m in re.finditer(rb'[\x20-\x7e]{%d,}' % minimum, data)]
    out += [m.group().decode('utf-16-le') for m in re.finditer(rb'(?:[\x20-\x7e]\x00){%d,}' % minimum, data)]
    return out


def version_strings(data):
    seen, out = set(), []
    for s in strings(data):
        if VERSION.search(s) and len(s) < 120:
            s = s.strip()
            if s.lower() not in seen:
                seen.add(s.lower())
                out.append(s)
    return out[:PER_FILE]


def version_info(data):
    try:
        import pefile
        from peres import version_info as read
        pe = pefile.PE(data=data, fast_load=True)
        pe.parse_data_directories(directories=[pefile.DIRECTORY_ENTRY['IMAGE_DIRECTORY_ENTRY_RESOURCE']])
        info = read(pe)
    except Exception:          # no pefile, or a resource table pefile cannot read
        return {}
    keep = ('ProductName', 'ProductVersion', 'FileVersion', 'FileVersion (fixed)', 'CompanyName',
            'FileDescription', 'OriginalFilename', 'LegalCopyright')
    return {k: info[k].strip('\x00 ') for k in keep if info.get(k, '').strip('\x00 ')}


def text_lines(data):
    try:
        text = data.decode('utf-8')
    except UnicodeDecodeError:
        text = data.decode('latin-1')
    out = []
    for line in text.splitlines():
        line = ' '.join(line.split())
        if 4 <= len(line) <= 160 and (VERSION.search(line) or DATE.search(line) or
                                      (WORDS.search(line) and re.search(r'\d', line))):
            out.append(line)
            if len(out) == PER_FILE:
                break
    return out


def disc_info(path):
    if os.path.isdir(path) or os.path.splitext(path)[1].lower() not in versiondiff.IMAGE:
        return None
    from image import open_image
    from volumes import scan
    disc = open_image(path)
    for t in disc.tracks:
        for v in scan(t):
            if v.root is not None:
                i = v.info
                return {k: i.get(k) for k in ('volume', 'created', 'modified', 'publisher', 'preparer', 'application')
                        if i.get(k)}
    return None


def identify(source):
    if os.path.isfile(source) and os.path.splitext(source)[1].lower() in ('.exe', '.dll'):
        files = {os.path.basename(source).lower(): versiondiff.File(
            os.path.basename(source), os.path.getsize(source), lambda: open(source, 'rb').read())}
    else:
        files = versiondiff.load(source)
    exes, texts = [], []
    for key in sorted(files):
        f = files[key]
        ext = os.path.splitext(key)[1]
        if ext in ('.exe', '.dll'):
            data = f.data()
            exes.append({'path': f.path, 'size': f.size, 'link_date': versiondiff.pe_date(data),
                         'version_info': version_info(data), 'version_strings': version_strings(data)})
        elif key.count('/') <= 2 and TEXT_NAMES.match(os.path.basename(key)) and f.size < 1 << 20:
            lines = text_lines(f.data())
            if lines:
                texts.append({'path': f.path, 'lines': lines})
    dates = sorted(e['link_date'] for e in exes if e['link_date'])
    return {'source': source, 'disc': disc_info(source), 'executables': exes, 'text': texts,
            'link_dates': [dates[0], dates[-1]] if dates else None, 'read': date.today().isoformat()}


def markdown(r, name):
    lines = [f'<!-- buildid {name} -->', f'### {name}', '',
             f'Read with `tools/triage/buildid.py` on {r["read"]}.', '']
    if r['disc']:
        d = r['disc']
        lines.append('Disc: ' + '; '.join(f'{k} {v}' for k, v in d.items()) + '.')
        lines.append('')
    if r['link_dates']:
        a, b = r['link_dates']
        lines += [f'Executables linked {a}' + (f' to {b}' if b != a else '') + '.', '']
    if r['executables']:
        lines += ['| Executable | Link date | Version information | Version strings |', '|---|---|---|---|']
        for e in r['executables']:
            vi = '; '.join(f'{k} {v}' for k, v in e['version_info'].items()) or '(none)'
            vs = '; '.join(f'"{s}"' for s in e['version_strings'][:3]) or ''
            lines.append(f'| `{e["path"]}` | {e["link_date"] or "-"} | {vi.replace("|", "/")} | {vs.replace("|", "/")} |')
        lines.append('')
    for t in r['text']:
        lines.append(f'- `{t["path"]}`: ' + ' / '.join(f'"{l}"' for l in t['lines'][:4]))
    if r['text']:
        lines.append('')
    lines.append(f'<!-- /buildid {name} -->')
    return '\n'.join(lines)


def write(path, block, name):
    text = open(path, encoding='utf-8').read() if os.path.exists(path) else ''
    start, end = f'<!-- buildid {name} -->', f'<!-- /buildid {name} -->'
    if start in text and end in text:
        a, b = text.index(start), text.index(end) + len(end)
        text = text[:a] + block + text[b:]
    else:
        if '## Build identity' not in text:
            text = text.rstrip('\n') + '\n\n## Build identity\n'
        i = text.index('## Build identity')
        j = text.find('\n## ', i + 1)
        j = len(text) if j < 0 else j
        text = text[:j].rstrip('\n') + '\n\n' + block + '\n' + text[j:]
    with open(path, 'w', encoding='utf-8') as f:
        f.write(text if text.endswith('\n') else text + '\n')


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__.rstrip())
        return 0 if argv else 2
    opt = lambda k: argv[argv.index(k) + 1] if k in argv and argv.index(k) + 1 < len(argv) else None
    source = argv[0]
    name = opt('--name') or os.path.basename(os.path.normpath(source))
    r = identify(source)
    if '--json' in argv:
        print(json.dumps(r, indent=2))
        return 0
    block = markdown(r, name)
    if opt('--write'):
        write(opt('--write'), block, name)
        print(f'wrote the build identity of {name} into {opt("--write")}')
    else:
        print(block)
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
