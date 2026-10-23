"""Find the know-how entries that match some text: an error message, a log, a command or a topic.

    from advise import search, signature_hits, command_hits, brief
    for hit in search('privileged instruction at 004a88cd'): print(hit.score, hit.entry.id)
    hits = signature_hits(log_text)       # only entries whose error signature matches
    hits = command_hits('wine game.exe')  # context entries whose command pattern matches

Scoring is plain and explained in docs/working/knowhow.md: an error signature (a regular
expression) that matches counts most; then a facet value or tag the text names, directly or by
an alias from the taxonomy; then the text's words in an entry's title, and in its body. There
are no embeddings. Standard library only.
"""
import re
from dataclasses import dataclass, field

import knowhow

SIGNATURE, FACET, TITLE, BODY, BODY_CAP = 100, 12, 4, 1, 6
STOP = set('the and for with not but this that from into when what was were are has have had '
           'its out all any can does did how why who which there their then than also only one '
           'two use used using get got run runs ran game games error failed failure fails'.split())
WORD = re.compile(r'[a-z0-9][a-z0-9_+.-]*[a-z0-9]|[a-z0-9]')
LABEL = {'solved': 'Solved', 'workaround': 'Workaround', 'dead-end': 'Dead end', 'open': 'Open',
         'context': 'Context'}


@dataclass
class Hit:
    entry: object
    score: int = 0
    why: list = field(default_factory=list)


def words(text):
    return {w for w in WORD.findall(text.lower()) if len(w) > 2 and w not in STOP}


def _phrase(alias, text):
    return re.search(r'(?<![a-z0-9])' + re.escape(alias) + r'(?![a-z0-9])', text) is not None


def named_values(text, facets):
    """{value: facet} for every facet value the text names, itself or by an alias."""
    low, found = text.lower(), {}
    for facet, values in facets.items():
        for name, v in values.items():
            if any(_phrase(a, low) for a in (name, name.replace('-', ' '), *v.aliases)):
                found[name] = facet
    return found


def _match(pattern, text, flags=0):
    """The match, or None; a pattern that does not compile matches nothing (`--check` names it)."""
    try:
        return re.search(pattern, text, flags)
    except re.error:
        return None


def _signature(entry, text):
    for pattern in entry.get('signatures'):
        m = _match(pattern, text, re.IGNORECASE | re.MULTILINE)
        if m:
            return pattern, m.group(0)
    return None


def search(text, cwd=None, every=None):
    """Hits for `text`, best first; every entry scores, only those above 0 are kept."""
    every = knowhow.entries(cwd) if every is None else every
    facets, ws = knowhow.taxonomy(cwd), words(text)
    named = named_values(text, facets)
    tags = {t.lower() for e in every for t in e.get('tags')}
    named.update({t: 'tags' for t in tags if _phrase(t, text.lower()) or _phrase(t.replace('-', ' '), text.lower())})
    hits = []
    for e in every:
        h = Hit(e)
        sig = _signature(e, text)
        if sig:
            h.score += SIGNATURE
            h.why.append(f'signature {sig[0]!r} matches {sig[1][:60]!r}')
        shared = sorted(set(named) & e.facets())
        if shared:
            h.score += FACET * len(shared)
            h.why.append('names ' + ', '.join(shared))
        title = ws & words(e.title)
        if title:
            h.score += TITLE * len(title)
            h.why.append('title has ' + ', '.join(sorted(title)))
        body = ws & words(e.body)
        if body:
            h.score += BODY * min(len(body), BODY_CAP)
            h.why.append('body has ' + ', '.join(sorted(body)[:BODY_CAP]))
        if h.score > BODY * 1 or sig:
            hits.append(h)
    return sorted(hits, key=lambda h: (-h.score, h.entry.id))


def signature_hits(text, cwd=None):
    """Only the entries whose error signature matches `text`."""
    out = []
    for e in knowhow.entries(cwd):
        sig = _signature(e, text)
        if sig:
            out.append(Hit(e, SIGNATURE, [f'signature {sig[0]!r} matches {sig[1][:60]!r}']))
    return out


def command_hits(command, cwd=None):
    """Entries whose `commands` pattern matches a shell command: the context for that work.

    Only the command's first line counts, so the text of a heredoc or a script does not."""
    first = (command or '').strip().split('\n', 1)[0]
    out = []
    for e in knowhow.entries(cwd):
        for pattern in e.get('commands'):
            if _match(pattern, first):
                out.append(Hit(e, FACET, [f'the command matches {pattern!r}']))
                break
    return out


def first_paragraph(text, limit=400):
    para = text.strip().split('\n\n')[0].strip() if text.strip() else ''
    para = ' '.join(line.strip() for line in para.splitlines())
    return para if len(para) <= limit else para[:limit].rsplit(' ', 1)[0] + ' ...'


def tried(entry):
    return [l.strip()[2:].rstrip('.') for l in entry.section('Tried').splitlines() if l.strip().startswith(('- ', '* '))]


def brief(entry, command='gameframe'):
    """A few lines on one entry: its status, what to do or what was tried, and its links."""
    lines = [f'[{LABEL.get(entry.status, entry.status)}] {entry.id}: {entry.title}'
             + (' (this project)' if entry.project else '')]
    if entry.status in ('solved', 'workaround'):
        lines.append('  Do: ' + first_paragraph(entry.section('What to do')))
    elif entry.status == 'context':
        lines.append('  ' + first_paragraph(entry.section('What to know')))
    else:
        t = tried(entry)
        lines.append(f'  No known fix. Tried {len(t)} things: ' + '; '.join(t[:4]) + (' ...' if len(t) > 4 else '')
                     + '. Try something new, and add it to the entry.' if t else
                     '  No known fix yet. Add what you try to the entry.')
    links = [*(f'tool {t}' for t in entry.get('tools')), *(f'skill {s}' for s in entry.get('skills')),
             *entry.get('docs')]
    if links:
        lines.append('  See: ' + ', '.join(links))
    lines.append(f'  Read it: {command} advise --entry {entry.id}')
    return '\n'.join(lines)
