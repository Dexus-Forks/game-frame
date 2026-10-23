"""One function per command of the CLI. Each takes its parsed arguments and returns an exit code.

Output is plain text for people, or JSON (--json) for agents and scripts. The text comes from
the skills and the tools themselves (catalog.py, skills.py); nothing here copies it.
"""
import difflib
import json
import shutil
import subprocess
import sys
import textwrap
from pathlib import Path

import advise as know
import agents
import catalog
import knowhow
import clients
import skills
from paths import NAME, ROOT, SCRIPTS, python, rel
from search import search

GROUPS = {'disc': 'Discs and disk images', 'pe': 'Windows PE', 'dos': '16-bit DOS', 'emu': 'Emulators', 'triage': 'Triage',
          'formats': 'Formats', 'verify': 'Verify', 'ghidra': 'Ghidra scripts',
          'site': 'Sites', 'scripts': 'Scripts'}


def _width():
    return min(shutil.get_terminal_size((100, 24)).columns, 100)


def _row(name, text, pad):
    """`name` in a column of `pad`, then `text` wrapped under itself."""
    lines = textwrap.wrap(text, max(_width() - pad - 2, 30)) or ['']
    out = [f'  {name:<{pad}}{lines[0]}'] + [' ' * (pad + 2) + line for line in lines[1:]]
    return '\n'.join(out)


def _dump(data):
    print(json.dumps(data, indent=2))
    return 0


def _not_found(what, name, close):
    hint = f' Did you mean: {", ".join(close)}?' if close else ''
    print(f'{NAME}: no {what} named {name!r}.{hint} See `{NAME} {what}s`.', file=sys.stderr)
    return 2


def steps(args):
    diagram, rows = skills.steps()
    if args.json:
        return _dump([{'step': s, 'question': q, 'skill': k} for s, q, k in rows])
    print(diagram + '\n')
    pad = max(len(s) for s, _, _ in rows) + 2
    for step, question, skill in rows:
        print(_row(step, f'{skill}: {question}', pad))
    print(f'\nRead a step with `{NAME} skill NAME`. Start with `{NAME} skill re-start`.')
    return 0


def list_skills(args):
    every = skills.skills()
    if args.json:
        return _dump([s.as_dict() for s in every])
    pad = max(len(s.name) for s in every) + 2
    for s in every:
        print(_row(s.name, s.description.split(' Use ')[0], pad))
    print(f'\nPrint one with `{NAME} skill NAME`, or one part with `--section TITLE`.')
    return 0


def skill(args):
    s, close = skills.find(args.name)
    if not s:
        return _not_found('skill', args.name, close)
    if args.sections:
        for level, title, _ in skills.headings(s.body):
            print('  ' * (level - 1) + title)
        return 0
    if args.section:
        text = skills.section(s.body, args.section)
        if text is None:
            print(f'{NAME}: {s.name} has no section matching {args.section!r}. '
                  f'See `{NAME} skill {s.name} --sections`.', file=sys.stderr)
            return 2
        print(text)
        return 0
    print(s.body.rstrip())
    return 0


def list_agents(args):
    if args.table:
        print(agents.table())
        return 0
    if args.emit:
        return emit_agents(args.emit, args.out)
    found = agents.agents()
    if args.json:
        return _dump([{k: v for k, v in a.items() if k != 'body'} for a in found])
    pad = max(len(a['name']) for a in found) + 2
    for a in found:
        print(_row(a['name'], a['summary'], pad))
    return 0


def emit_agents(client, out):
    """Write each client's agent files, and remove generated files whose agent is gone."""
    out = Path(out) if out else ROOT
    for c in clients.CLIENTS if client == 'all' else (client,):
        files = clients.render(c)
        for old in {(out / f).parent for f in files}:
            for p in old.glob('re-*') if old.is_dir() else ():
                if str(p.relative_to(out)) not in files:
                    p.unlink()
        clients.write(files, out)
        print(f'{c}: {len(files)} agents in {rel(out / next(iter(files))).rsplit("/", 1)[0]}/')
    return 0


def list_tools(args):
    every = catalog.tools()
    if args.json:
        return _dump([t.as_dict() for t in every])
    pad = max(len(t.name) for t in every) + 2
    group = None
    for t in every:
        if t.group != group:
            group = t.group
            print(f'\n{GROUPS.get(group, group)} ({"scripts" if group == "scripts" else "tools/" + group})')
        mark = '' if t.runs else f'[{t.kind}] '
        print(_row(t.name, mark + (t.summary or '(no help yet)'), pad))
    print(f'\nShow one with `{NAME} tool NAME`; run it with `{NAME} run NAME ARGS`.')
    return 0


def tool(args):
    t, close = catalog.find(args.name)
    if not t:
        return _not_found('tool', args.name, close)
    if args.json:
        return _dump(dict(t.as_dict(), doc=t.doc))
    print(f'{rel(t.path)}  ({t.kind})\n')
    print(t.doc.rstrip() or '(this tool has no help yet)')
    if t.runs:
        print(f'\nRun it: {NAME} run {t.name} ARGS')
    elif t.kind == 'ghidra':
        print(f'\nRun it through Ghidra: {NAME} run ghidra-run BINARY ... or {NAME} run ghidra-export')
    return 0


def run(name, rest):
    t, close = catalog.find(name)
    if not t:
        return _not_found('tool', name, close)
    if not t.runs:
        how = ('a Ghidra script: run it with scripts/ghidra-run.sh or ghidra-export.sh'
               if t.kind == 'ghidra' else 'a library to import from Python code')
        print(f'{NAME}: {t.name} is {how}. See `{NAME} tool {t.name}`.', file=sys.stderr)
        return 2
    py, note = python()
    if note and t.kind == 'program':
        print(note, file=sys.stderr)
    return subprocess.call(catalog.command(t, py, rest))


def find(args):
    hits = search(args.word)
    if args.json:
        return _dump([dict(zip(('kind', 'name', 'where', 'line'), h)) for h in hits])
    if not hits:
        print(f'Nothing mentions {args.word!r}.')
        return 1
    shown = {}
    for kind, name, where, line in hits:
        key = (kind, name)
        shown[key] = shown.get(key, 0) + 1
        if shown[key] == 1:
            print(f'\n{kind} {name}')
        if shown[key] <= args.max:
            print(f'  [{where}] {line}' if where else f'  {line}')
    more = {k: n - args.max for k, n in shown.items() if n > args.max}
    if more:
        print('\nMore lines: ' + ', '.join(f'{name} +{n}' for (_, name), n in more.items())
              + ' (use --max N)')
    return 0


def doctor(rest):
    return subprocess.call(['bash', str(SCRIPTS / 'setup.sh'), *rest])


def advise(args):
    """Known failure modes and context for some text, an entry in full, the list, or a check."""
    if args.check:
        problems = knowhow.check()
        for p in problems:
            print(p)
        print(f'{len(knowhow.entries())} entries, {len(problems)} problems')
        return 1 if problems else 0
    if args.entry:
        e = next((e for e in knowhow.entries() if e.id == args.entry), None)
        if not e:
            close = difflib.get_close_matches(args.entry, [e.id for e in knowhow.entries()], n=3, cutoff=0.4)
            return _no_entry(args.entry, close)
        if args.json:
            return _dump({**e.as_dict(), 'body': e.body})
        print(e.path.read_text(encoding='utf-8').rstrip())
        return 0
    if args.list:
        every = knowhow.entries()
        if args.json:
            return _dump([e.as_dict() for e in every])
        pad = max(len(e.id) for e in every) + 2
        for e in every:
            print(_row(e.id, f'[{know.LABEL[e.status]}] {e.title}', pad))
        return 0
    text = ' '.join(args.text)
    if text in ('', '-'):
        text = sys.stdin.read()
    if not text.strip():
        print(f'{NAME} advise: give an error message, a log on stdin (-) or a topic. '
              f'See `{NAME} help advise`.', file=sys.stderr)
        return 2
    hits = know.signature_hits(text) if args.signatures else know.search(text)
    hits = hits[:args.max]
    if args.json:
        return _dump([{**h.entry.as_dict(), 'score': h.score, 'why': h.why} for h in hits])
    if not hits:
        print(f'No know-how matches. If you solve it, or give it up, record it: '
              f'`{NAME} skill re-land --section know-how`.')
        return 1
    for h in hits:
        print(know.brief(h.entry, NAME))
        if args.why:
            print('  Why: ' + '; '.join(h.why))
        print()
    return 0


def _no_entry(name, close):
    hint = f' Did you mean: {", ".join(close)}?' if close else ''
    print(f'{NAME}: no know-how entry {name!r}.{hint} See `{NAME} advise --list`.', file=sys.stderr)
    return 2
