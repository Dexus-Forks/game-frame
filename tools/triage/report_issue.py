#!/usr/bin/env python3
"""List a port's reports, and file one as a GitHub issue, once per place.

A port built with GameFrame's `gameframe-report` crate writes a report the first time the game
reaches a path it has not ported, or crashes: `<id>.json` and `<id>.md` in its reports folder
(`unported-3f2a19c0.json`). The Markdown is the issue's body, laid out for an agent to debug from.
Any other tool may write the same JSON (the crate's README has its fields).

Usage:
  report_issue.py list [DIR]                    the reports, most frequent first (default DIR: reports)
  report_issue.py show REPORT                   the issue's title and body
  report_issue.py file REPORT [--repo OWNER/NAME] [--dry-run]

  REPORT     a report's .json file, or its id in the reports folder
  --repo     where the issue goes; default the report's own `meta.issues`, then gh's current repo
  --dry-run  print the gh commands instead of running them

`file` looks for an open issue whose body holds the report's id
(`<!-- gameframe-report: ID -->`). When there is one, it adds a comment with the new count.
Otherwise it opens an issue with the labels `gameframe-report` and the report's kind, making the
labels when the repository lacks them. It needs `gh`, signed in.

What it does not do: it does not read a running game, and it does not post anything in `list`
or `show`. Filing is up to you: a report in Dev mode stays on this machine until you run `file`.
Standard library only.
"""
import json
import os
import subprocess
import sys


def load(ref, folder='reports'):
    path = ref if ref.endswith('.json') else os.path.join(folder, ref + '.json')
    with open(path, encoding='utf-8') as f:
        report = json.load(f)
    md = path[:-5] + '.md'
    body = None
    if os.path.exists(md):
        with open(md, encoding='utf-8') as f:
            text = f.read()
        # The file starts with the title as a heading; the issue's body is the rest.
        body = text.split('\n', 2)[2] if text.startswith('# ') else text
    return report, body


def title(r):
    original = f" ({r['original']})" if r.get('original') else ''
    what = (r.get('what') or '').splitlines()[0][:90] if r.get('what') else ''
    return f"[{r.get('kind', 'unported')}] {what} at {r.get('file')}:{r.get('line')}{original}"


def plain_body(r):
    """A body for a report written without its Markdown."""
    lines = [f"<!-- gameframe-report: {r['id']} -->", '', f"> {r.get('what', '')}", '',
             f"Port: `{r.get('file')}:{r.get('line')}`"]
    if r.get('original'):
        lines.append(f"Original: `{r['original']}`")
    lines += ['', '```json', json.dumps(r, indent=2), '```']
    return '\n'.join(lines) + '\n'


def gh(args, dry):
    if dry:
        print('gh ' + ' '.join(a if ' ' not in a else repr(a) for a in args))
        return ''
    out = subprocess.run(['gh'] + args, capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"gh {args[0]} {args[1]}: {out.stderr.strip()}")
    return out.stdout


def cmd_list(folder):
    rows = []
    for name in sorted(os.listdir(folder)) if os.path.isdir(folder) else []:
        if name.endswith('.json'):
            with open(os.path.join(folder, name), encoding='utf-8') as f:
                r = json.load(f)
            rows.append(r)
    if not rows:
        print(f'no reports in {folder}')
        return
    rows.sort(key=lambda r: (-r.get('count', 0), r.get('id', '')))
    for r in rows:
        print(f"{r.get('count', 0):6}  {r.get('id')}  {title(r)}")


def cmd_file(ref, repo, dry):
    r, body = load(ref)
    body = body or plain_body(r)
    repo = repo or (r.get('meta') or {}).get('issues', '').replace('https://github.com/', '').strip('/')
    where = ['--repo', repo] if repo else []
    found = gh(['issue', 'list', *where, '--state', 'open', '--search', f"{r['id']} in:body",
                '--json', 'number', '--jq', '.[0].number'], dry).strip()
    if found:
        gh(['issue', 'comment', found, *where, '--body',
            f"Seen again: {r.get('count')} times in all, last at {r.get('last')} (`{r['id']}`)."], dry)
        print(f'issue #{found}: added the new count')
        return
    labels = ['gameframe-report', r.get('kind', 'unported')]
    for label in labels:
        gh(['label', 'create', label, *where, '--force', '--color', 'd93f0b'], dry)
    url = gh(['issue', 'create', *where, '--title', title(r), '--body', body,
              '--label', ','.join(labels)], dry).strip()
    print(url or 'issue: created')


def main(argv):
    if len(argv) < 2 or argv[1] in ('-h', '--help'):
        print(__doc__)
        return 0 if len(argv) > 1 else 2
    cmd, rest = argv[1], argv[2:]
    dry = '--dry-run' in rest
    rest = [a for a in rest if a != '--dry-run']
    repo = None
    if '--repo' in rest:
        i = rest.index('--repo')
        repo = rest[i + 1] if i + 1 < len(rest) else sys.exit('--repo needs OWNER/NAME')
        del rest[i:i + 2]
    if cmd == 'list':
        cmd_list(rest[0] if rest else 'reports')
    elif cmd == 'show' and rest:
        r, body = load(rest[0])
        print(title(r) + '\n\n' + (body or plain_body(r)))
    elif cmd == 'file' and rest:
        cmd_file(rest[0], repo, dry)
    else:
        print(__doc__)
        return 2
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
