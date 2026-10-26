#!/usr/bin/env python3
"""Cut a GameFrame release, then open the next development version.

Usage:

    python3 scripts/release.py X.Y.Z [--push] [--publish] [--no-next]
    python3 scripts/release.py next [X.Y.Z]

`X.Y.Z` releases the Unreleased work:
- CHANGELOG.md: the `## [...-dev] - Unreleased` heading becomes `## [X.Y.Z] - TODAY`, and its
  compare link ends at `vX.Y.Z`;
- .claude-plugin/plugin.json: the version becomes `X.Y.Z`;
- it commits those two files as "Release X.Y.Z" and makes the tag `vX.Y.Z`;
- then it opens the next version, as `next` does, unless `--no-next` is given.
`--push` pushes the branch and the tag. `--publish` also makes the GitHub release with `gh`,
with the changelog's section as its notes.

`next` opens a development version on top of a release: an empty `## [N-dev] - Unreleased`
section above the newest release, its compare link, and `N-dev` in plugin.json, committed as
"Open N-dev". N is the next minor version (1.4.0 opens 1.5.0-dev) unless it is given. The
pre-commit gate (scripts/hooks/gates/version.py) blocks other commits until this is done.

It works in the repository of the current folder and needs a clean CHANGELOG.md and
plugin.json. It does not run the tests: run `tests/run.sh --ghidra` before a release. It does
not write the changelog's entries. Standard library only, plus git, and gh for --publish.
"""
import datetime
import json
import re
import subprocess
import sys
from pathlib import Path

CHANGELOG = Path('CHANGELOG.md')
PLUGIN = Path('.claude-plugin/plugin.json')
VERSION = re.compile(r'\d+\.\d+\.\d+')
HEADING = re.compile(r'^## \[([^\]]+)\]', re.M)
LINK = re.compile(r'^\[([^\]]+)\]: (\S+)$', re.M)


def git(*args):
    subprocess.run(['git', *args], check=True)


def fail(why):
    print(why, file=sys.stderr)
    sys.exit(2)


def set_version(version):
    text = PLUGIN.read_text()
    PLUGIN.write_text(re.sub(r'"version": "[^"]*"', f'"version": "{version}"', text, count=1))


def compare_base(text):
    """The repository's compare URL prefix, from the changelog's links."""
    for _, url in LINK.findall(text):
        if '/compare/' in url:
            return url.split('/compare/')[0] + '/compare/'
    fail('CHANGELOG.md has no compare link to copy the repository URL from.')


def release(version, publish, push, open_next):
    text = CHANGELOG.read_text()
    found = HEADING.search(text)
    if not found or not found.group(1).endswith('-dev'):
        fail('CHANGELOG.md does not start with a -dev section to release.')
    dev = found.group(1)
    today = datetime.date.today().isoformat()
    text = text.replace(f'## [{dev}] - Unreleased', f'## [{version}] - {today}', 1)
    text = re.sub(rf'^\[{re.escape(dev)}\]: (\S+)\.\.\.HEAD$', rf'[{version}]: \1...v{version}', text, count=1, flags=re.M)
    CHANGELOG.write_text(text)
    set_version(version)
    git('add', str(CHANGELOG), str(PLUGIN))
    git('commit', '-q', '-m', f'Release {version}')
    git('tag', '-a', f'v{version}', '-m', f'GameFrame {version}')
    print(f'Released {version} and tagged v{version}.')
    if push or publish:
        git('push', '-q', 'origin', 'HEAD')
        git('push', '-q', 'origin', f'v{version}')
    if publish:
        notes = text[text.index(f'## [{version}]'):]
        notes = notes[notes.index('\n') + 1:]
        notes = notes[:notes.find('\n## [')] if '\n## [' in notes else notes
        subprocess.run(['gh', 'release', 'create', f'v{version}', '--title', f'GameFrame {version}',
                        '--notes', notes.strip(), '--verify-tag'], check=True)
    if open_next:
        open_dev(None, push or publish)


def open_dev(target, push):
    text = CHANGELOG.read_text()
    found = HEADING.search(text)
    if not found or not VERSION.fullmatch(found.group(1)):
        fail('CHANGELOG.md does not start with a released version to open the next one after.')
    last = found.group(1)
    major, minor, _ = map(int, last.split('.'))
    nxt = target or f'{major}.{minor + 1}.0'
    if not VERSION.fullmatch(nxt):
        fail(f'{nxt} is not a version like 1.5.0.')
    dev = f'{nxt}-dev'
    text = text[:found.start()] + f'## [{dev}] - Unreleased\n\n' + text[found.start():]
    first_link = LINK.search(text)
    line = f'[{dev}]: {compare_base(text)}v{last}...HEAD\n'
    text = text[:first_link.start()] + line + text[first_link.start():] if first_link else text + '\n' + line
    CHANGELOG.write_text(text)
    set_version(dev)
    git('add', str(CHANGELOG), str(PLUGIN))
    git('commit', '-q', '-m', f'Open {dev}')
    print(f'Opened {dev}.')
    if push:
        git('push', '-q', 'origin', 'HEAD')


def main(args):
    if not args or args[0] in ('-h', '--help'):
        print(__doc__)
        return 0 if args else 2
    if not CHANGELOG.exists() or not PLUGIN.exists():
        fail('Run this in a repository with CHANGELOG.md and .claude-plugin/plugin.json.')
    if subprocess.run(['git', 'diff', '--quiet', 'HEAD', '--', str(CHANGELOG), str(PLUGIN)]).returncode:
        fail('CHANGELOG.md or plugin.json has uncommitted changes. Commit them first.')
    flags = {a for a in args[1:] if a.startswith('--')}
    rest = [a for a in args[1:] if not a.startswith('--')]
    if args[0] == 'next':
        open_dev(rest[0] if rest else None, '--push' in flags)
    elif VERSION.fullmatch(args[0]):
        release(args[0], '--publish' in flags, '--push' in flags, '--no-next' not in flags)
    else:
        fail(f'{args[0]} is not a version like 1.4.0, or next.')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
