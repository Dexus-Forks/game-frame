"""GameFrame's know-how: entries on failure modes and on context, read from Markdown files.

    from knowhow import entries, taxonomy, check
    for e in entries(): print(e.id, e.status, e.title)
    facets = taxonomy()            # {'runtime': {'wine': Value(...)}, ...}
    problems = check()             # what is wrong with the entries, as text lines

An entry is one Markdown file in docs/knowhow/ (GameFrame's own) or in a project's
docs/knowhow/ (that game's). Its front matter is YAML that this module reads with a small
parser: `key: value`, `key: [a, b]`, and a block of `- 'item'` lines. The design and the field
list are in docs/working/knowhow.md; the facets and their aliases are in docs/knowhow/README.md.

GAMEFRAME_KNOWHOW, a list of folders separated by `:`, replaces the folders searched (the tests
use it). Standard library only. It reads files and never writes them.
"""
import os
import re
from dataclasses import dataclass, field
from pathlib import Path

from paths import ROOT, rel

FOLDER = ROOT / 'docs' / 'knowhow'
INDEX = 'README.md'
KINDS = ('failure', 'context')
STATUSES = ('solved', 'workaround', 'dead-end', 'open', 'context')
FACETS = ('era', 'platform', 'runtime', 'hardware', 'api', 'compiler', 'genre', 'symptom')
LISTS = (*FACETS, 'tags', 'signatures', 'commands', 'tools', 'skills', 'docs', 'seen')
# The sections each status needs in the body.
NEEDS = {'solved': ('Symptoms', 'Cause', 'What to do'), 'workaround': ('Symptoms', 'Cause', 'What to do'),
         'dead-end': ('Symptoms', 'Tried'), 'open': ('Symptoms', 'Tried'), 'context': ('What to know',)}
VALUE = re.compile(r'^- `([a-z0-9][a-z0-9.+-]*)`: (.*?)(?: Also: (.*))?$')
HEADING = re.compile(r'^##\s+(.*?)\s*$')


@dataclass
class Value:
    name: str
    facet: str
    text: str
    aliases: list = field(default_factory=list)


@dataclass
class Entry:
    id: str
    title: str
    kind: str
    status: str
    path: Path
    body: str
    meta: dict
    project: bool = False

    def get(self, key):
        return self.meta.get(key, [] if key in LISTS else '')

    def facets(self):
        """Every facet value and tag of the entry, lower case."""
        return {v.lower() for k in (*FACETS, 'tags') for v in self.get(k)}

    def section(self, title):
        """The text under `## title`, or ''."""
        return sections(self.body).get(title, '')

    def as_dict(self):
        return {'id': self.id, 'title': self.title, 'kind': self.kind, 'status': self.status,
                'path': rel(self.path), 'project': self.project,
                **{k: self.get(k) for k in LISTS}}


def _scalar(text):
    text = text.strip()
    if len(text) >= 2 and text[0] == text[-1] == "'":
        return text[1:-1].replace("''", "'")
    if len(text) >= 2 and text[0] == text[-1] == '"':
        return text[1:-1].replace('\\"', '"').replace('\\\\', '\\')
    return text


def _flow(text):
    """`[a, 'b, c', d]` as a list."""
    items, cur, quote = [], '', None
    for ch in text.strip()[1:-1]:
        if quote:
            cur += ch
            quote = None if ch == quote else quote
        elif ch in '\'"':
            cur += ch
            quote = ch
        elif ch == ',':
            items.append(cur)
            cur = ''
        else:
            cur += ch
    items.append(cur)
    return [_scalar(i) for i in items if i.strip()]


def front_matter(text):
    """(meta, body) of a Markdown file with YAML front matter in GameFrame's subset."""
    if not text.startswith('---\n'):
        return {}, text
    end = text.find('\n---\n', 3)
    if end < 0:
        return {}, text
    meta, key = {}, None
    for line in text[4:end].splitlines():
        if not line.strip() or line.lstrip().startswith('#'):
            continue
        if line.startswith((' ', '\t')) and line.strip().startswith('- ') and key:
            meta.setdefault(key, []).append(_scalar(line.strip()[2:]))
            continue
        key, _, value = line.partition(':')
        key, value = key.strip(), value.strip()
        if value.startswith('['):
            meta[key] = _flow(value)
        elif value:
            meta[key] = [_scalar(value)] if key in LISTS else _scalar(value)
        else:
            meta[key] = []
    return meta, text[end + 5:].lstrip('\n')


def sections(body):
    """{title: text} for each `## title` of a body."""
    out, title, lines = {}, None, []
    for line in body.splitlines():
        m = HEADING.match(line)
        if m:
            if title:
                out[title] = '\n'.join(lines).strip()
            title, lines = m.group(1), []
        elif title:
            lines.append(line)
    if title:
        out[title] = '\n'.join(lines).strip()
    return out


def folders(cwd=None):
    """The folders searched: GAMEFRAME_KNOWHOW, else GameFrame's own and the project's."""
    if os.environ.get('GAMEFRAME_KNOWHOW'):
        return [(Path(p), False) for p in os.environ['GAMEFRAME_KNOWHOW'].split(':') if p]
    found = [(FOLDER, False)]
    for root in (os.environ.get('GAMEFRAME_PROJECT'), cwd or os.getcwd()):
        if root and (Path(root) / 'docs' / 'knowhow').is_dir():
            p = (Path(root) / 'docs' / 'knowhow').resolve()
            if p not in [f.resolve() for f, _ in found]:
                found.append((p, True))
    return found


def load(path, project=False):
    meta, body = front_matter(path.read_text(encoding='utf-8'))
    return Entry(meta.get('id', path.stem), meta.get('title', path.stem), meta.get('kind', ''),
                 meta.get('status', ''), path, body, meta, project)


def entries(cwd=None):
    """Every entry of every folder searched, GameFrame's first."""
    out = []
    for folder, project in folders(cwd):
        for path in sorted(folder.glob('*.md')):
            if path.name != INDEX and not path.name.startswith('_'):
                out.append(load(path, project))
    return out


def taxonomy(cwd=None):
    """{facet: {value: Value}} from each searched folder's README.md (`### facet` lists)."""
    out = {f: {} for f in FACETS}
    for folder, _ in folders(cwd):
        index = folder / INDEX
        if not index.exists():
            continue
        facet = None
        for line in index.read_text(encoding='utf-8').splitlines():
            if line.startswith('### '):
                name = line[4:].strip().strip('`')
                facet = name if name in out else None
            m = VALUE.match(line.strip()) if facet else None
            if m:
                aliases = [a.strip().rstrip('.').lower() for a in (m.group(3) or '').split(',') if a.strip()]
                out[facet][m.group(1)] = Value(m.group(1), facet, m.group(2).rstrip('.'), aliases)
    return out


def check(cwd=None):
    """Problems with the entries, one line each: fields, facets, regexes, links, sections."""
    import catalog
    import skills
    problems, seen = [], {}
    known = taxonomy(cwd)
    for e in entries(cwd):
        where = rel(e.path)
        say = lambda msg: problems.append(f'{where}: {msg}')
        if e.id in seen:
            say(f'id {e.id!r} is also used by {seen[e.id]}')
        seen[e.id] = where
        if e.id != e.path.stem:
            say(f'id {e.id!r} differs from the file name')
        if e.kind not in KINDS:
            say(f'kind {e.kind!r} is not one of {", ".join(KINDS)}')
        if e.status not in STATUSES:
            say(f'status {e.status!r} is not one of {", ".join(STATUSES)}')
        if (e.kind == 'context') != (e.status == 'context'):
            say('a context entry has status context, and only a context entry does')
        if not e.title or e.title == e.path.stem:
            say('no title')
        for facet in FACETS:
            for v in e.get(facet):
                if v not in known[facet]:
                    say(f'{facet} {v!r} is not in the taxonomy (docs/knowhow/README.md)')
        for key in ('signatures', 'commands'):
            for pattern in e.get(key):
                try:
                    re.compile(pattern)
                except re.error as err:
                    say(f'{key} {pattern!r}: {err}')
        for t in e.get('tools'):
            if not catalog.find(t)[0]:
                say(f'tool {t!r} not found (gameframe tools)')
        for s in e.get('skills'):
            if not skills.find(s)[0]:
                say(f'skill {s!r} not found (gameframe skills)')
        for d in e.get('docs'):
            if not (ROOT / d.split('#')[0]).exists() and not (e.path.parents[2] / d.split('#')[0]).exists():
                say(f'doc {d!r} not found')
        have = sections(e.body)
        for need in NEEDS.get(e.status, ()):
            if need not in have:
                say(f'status {e.status} needs a "## {need}" section')
        if e.kind == 'failure' and not e.get('symptom'):
            say('a failure entry names at least one symptom')
    return problems
