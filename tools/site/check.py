#!/usr/bin/env python3
"""Check a project's website before it is published: safe to share, and useful to read.

Usage:
    tools/site/check.py SITE_DIR [SITE_DIR ...]       for example sites/pod, or site
    tools/site/check.py SITE_DIR --json

Inputs: one or more VitePress site folders made by scripts/new-site.sh (or written by hand in
the same shape: Markdown pages, a diary/ folder of YYYY-MM-DD-slug.md entries).

Errors (exit status 1):
- a game file in the site (an .exe, .dll, .pak, ROM image and the other extensions the commit
  gate blocks), or any file over 5 MB;
- lines that look like Ghidra's decompiled code, in a page or in a document a page includes;
- an <!--@include: PATH--> whose file does not exist;
- a diary entry without a title or a summary in its front matter, or with a file name that is
  not YYYY-MM-DD-slug.md;
- a TODO(site) line left from the template.
Warnings (printed, exit status 0):
- a diary entry that reads like a commit log: three or more commit hashes, or more than half
  of its lines are bullets that start with a commit verb (Add, Fix, Update, Merge ...);
- a diary entry under 120 words (a note, not an entry: fold it into the next one) or over
  2,500 words (split it, or move the reference part into docs/);
- a picture over 1 MB.
Output: one line per problem, `error` or `warn`, with the file. With --json, a list of
{level, file, message}.

It does not build the site (run `npm run build`, which also finds dead links), judge the prose,
or check that a picture was made by the project's own code: that is the re-site skill's checklist.
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '..', '..', 'scripts', 'hooks'))
from gates.checks import GAME_EXTS, PSEUDOCODE  # noqa: E402  (the commit gate's own lists)

SKIP_DIRS = {'node_modules', 'dist', 'cache', '.git'}
PICTURES = {'.png', '.jpg', '.jpeg', '.gif', '.webp', '.svg', '.avif', '.mp4', '.webm'}
INCLUDE = re.compile(r'<!--\s*@include:\s*([^\s>]+?)\s*-->')
ENTRY_NAME = re.compile(r'^\d{4}-\d{2}-\d{2}-[a-z0-9][a-z0-9-]*\.md$')
HASH = re.compile(r'\b[0-9a-f]{7,40}\b')
COMMIT_BULLET = re.compile(r'^\s*[-*]\s+(Add|Adds|Added|Fix|Fixes|Fixed|Update|Updates|Updated|Remove|Removed|'
                           r'Merge|Merged|Bump|Refactor|Rename|Move|Moved|Revert|WIP|Chore|feat|fix|chore)\b')


def front_matter(text):
    """The YAML front matter's top-level keys, as plain strings (enough for title and summary)."""
    m = re.match(r'---\n(.*?)\n---\n', text, re.S)
    if not m:
        return {}, text
    keys = {}
    for line in m.group(1).splitlines():
        k = re.match(r'^([A-Za-z_][\w-]*):\s*(.*)$', line)
        if k:
            keys[k.group(1)] = k.group(2).strip().strip('"\'')
    return keys, text[m.end():]


def site_files(site):
    for base, dirs, files in os.walk(site):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for f in files:
            yield os.path.join(base, f)


def check(site):
    problems = []
    say = lambda level, path, msg: problems.append({'level': level, 'file': os.path.relpath(path), 'message': msg})
    for path in site_files(site):
        ext = os.path.splitext(path)[1].lower()
        size = os.path.getsize(path)
        if ext in GAME_EXTS:
            say('error', path, f'a game file ({ext}) does not belong on the site')
        if size > 5_000_000:
            say('error', path, f'{size // 1_000_000} MB: too large for a site file')
        elif ext in PICTURES and size > 1_000_000:
            say('warn', path, f'a {size // 1000} KB picture: compress it (WebP) or make it smaller')
        if ext != '.md':
            continue
        text = open(path, encoding='utf-8', errors='replace').read()
        sources = [(path, text)]
        for inc in INCLUDE.findall(text):
            target = os.path.normpath(os.path.join(os.path.dirname(path), inc.split('{')[0].split('#')[0]))
            if not os.path.isfile(target):
                say('error', path, f'includes {inc}, which does not exist')
            else:
                sources.append((target, open(target, encoding='utf-8', errors='replace').read()))
        for src, body in sources:
            hits = [l for l in body.splitlines() if PSEUDOCODE.search(l)]
            if hits:
                where = '' if src == path else f' (in {os.path.relpath(src)})'
                say('error', path, f'{len(hits)} lines look like decompiled code{where}: describe it instead')
        if 'TODO(site)' in text:
            say('error', path, 'a TODO(site) line from the template is still there')
        if os.path.basename(os.path.dirname(path)) == 'diary' and os.path.basename(path) != 'index.md':
            problems += diary_entry(path, text, say)
    return problems


def diary_entry(path, text, say):
    name = os.path.basename(path)
    if not ENTRY_NAME.match(name):
        say('error', path, 'a diary entry is named YYYY-MM-DD-slug.md')
    keys, body = front_matter(text)
    for k in ('title', 'summary'):
        if not keys.get(k):
            say('error', path, f'the front matter has no {k}: the diary index shows it')
    lines = [l for l in body.splitlines() if l.strip()]
    words = len(re.findall(r'\w+', re.sub(r'```.*?```', '', body, flags=re.S)))
    hashes = len(HASH.findall(re.sub(r'`[^`]*`|0x[0-9a-fA-F]+', '', body)))
    bullets = sum(1 for l in lines if COMMIT_BULLET.match(l))
    if hashes >= 3 or (lines and bullets * 2 > len(lines)):
        say('warn', path, 'reads like a commit log: say what changed for a reader and how it is known, '
                          'and link the commits instead of listing them')
    if words < 120:
        say('warn', path, f'{words} words: a note, not an entry; fold it into the next entry')
    elif words > 2500:
        say('warn', path, f'{words} words: split it, or move the reference part into docs/')
    return []


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__.strip())
        return 0 if argv else 2
    as_json = '--json' in argv
    sites = [a for a in argv if a != '--json']
    problems = []
    for site in sites:
        if not os.path.isdir(site):
            problems.append({'level': 'error', 'file': site, 'message': 'not a folder'})
            continue
        problems += check(site)
    if as_json:
        print(json.dumps(problems, indent=2))
    else:
        for p in problems:
            print(f'{p["level"]:5}  {p["file"]}: {p["message"]}')
        errors = sum(p['level'] == 'error' for p in problems)
        print(f'{len(sites)} site(s): {errors} errors, {len(problems) - errors} warnings')
    return 1 if any(p['level'] == 'error' for p in problems) else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
