"""The version gate: no commit on top of a release until the next development version is open.

A repository that keeps its version in `.claude-plugin/plugin.json` and tags each release
`vX.Y.Z` (GameFrame does) moves to `X.(Y+1).0-dev` right after the release, so the changelog
has an Unreleased section for the next change. This check reads the version in the commit's
index. When it is a release version (no `-dev` or other suffix) and the tag `v` + version
already exists, the release is done and this commit should open the next version:

    python3 scripts/release.py next

The release commit itself passes, because its tag does not exist yet. A repository without
`.claude-plugin/plugin.json`, or with no such tag, is not checked. Standard library only.
"""
import json
import re
import subprocess

PLUGIN = '.claude-plugin/plugin.json'
RELEASE = re.compile(r'\d+\.\d+\.\d+')


def git(*args, cwd):
    return subprocess.run(['git', *args], cwd=cwd, capture_output=True, text=True)


def reason(cwd):
    """Why this commit is blocked, or None."""
    shown = git('show', f':{PLUGIN}', cwd=cwd)
    if shown.returncode != 0:
        return None
    try:
        version = json.loads(shown.stdout).get('version', '')
    except ValueError:
        return None
    if not RELEASE.fullmatch(version):
        return None
    if git('rev-parse', '-q', '--verify', f'refs/tags/v{version}', cwd=cwd).returncode != 0:
        return None
    return (f'{version} is released (tag v{version}), and {PLUGIN} still names it. Open the next '
            'development version first, so this change goes under Unreleased:\n'
            '  python3 scripts/release.py next')
