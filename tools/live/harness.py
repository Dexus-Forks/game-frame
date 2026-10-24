#!/usr/bin/env python3
"""Put together the modules a game needs to run: a runtime, and a layer for each need it lists.

Usage:
  harness.py plan PATH ... [--runtime ID] [--with ID ...] [--script FILE] [--json] [--no-cpu]
  harness.py modules [--json]
  harness.py show ID
  harness.py check

  plan      run tools/triage/needs.py on PATH (an executable, a folder or a disc image), choose
            a runtime module and the layer modules, and print the plan: each module, the needs
            it covers, and the needs no module covers
  --runtime the runtime to use (default: the one that covers the most needs)
  --with    add a module the analyser cannot ask for, such as silent-audio (repeat it)
  --script  write every chosen module's Set up steps, in order, as one bash script. Nothing is
            run: read the script, set its variables, then run it
  modules   list every module: GameFrame's docs/harness/modules/, then the project's own
            (a project module with the same id replaces GameFrame's)
  show      print one module
  check     check every module: id, kind, needs, runtimes, requires and sections

A module is a Markdown file with front matter (docs/harness/README.md): id, kind (runtime or
layer), runtime, provides, requires, vars, seen. Its "Set up" section holds bash blocks.

The plan picks the runtime first. A runtime module is scored by the needs it covers, and a
layer that works in it counts too. Then, for each need still open, it adds the layer that
covers the most open needs, with the modules it requires. Needs of kind packer and unknown
are shown and never matched. A need with no module is a gap: write a module for it from
docs/harness/modules/_template.md, or look for known failures with `gameframe advise`.

Exit code: 0 when every need is covered, 1 when some are not.

What it does not do: run anything, download anything, or prove that a module works for this
game. The script is a draft to read.
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TOOLS = os.path.dirname(HERE)
ROOT = os.path.dirname(TOOLS)
sys.path.insert(0, os.path.join(TOOLS, 'triage'))
sys.path.insert(0, os.path.join(TOOLS, 'gameframe'))
from knowhow import front_matter, sections  # noqa: E402

PROJECT = os.environ.get('GAMEFRAME_PROJECT') or ROOT
SKIP = ('packer', 'unknown')


def as_list(v):
    return v if isinstance(v, list) else [v] if v else []


def load():
    """{id: module} from GameFrame's folder, then the project's."""
    mods = {}
    for base in dict.fromkeys((ROOT, PROJECT, os.getcwd())):
        folder = os.path.join(base, 'docs', 'harness', 'modules')
        if not os.path.isdir(folder):
            continue
        for name in sorted(os.listdir(folder)):
            if not name.endswith('.md') or name.startswith('_'):
                continue
            path = os.path.join(folder, name)
            meta, body = front_matter(open(path, encoding='utf-8').read())
            m = {k: as_list(meta.get(k)) for k in ('runtime', 'provides', 'requires', 'vars', 'seen')}
            m.update(id=meta.get('id', name[:-3]), title=meta.get('title', ''), kind=meta.get('kind', 'layer'),
                     path=os.path.relpath(path, os.getcwd()), body=body)
            mods[m['id']] = m
    return mods


def with_requires(mods, ids, out):
    for i in ids:
        if i in out or i not in mods:
            continue
        with_requires(mods, mods[i]['requires'], out)
        out.append(i)
    return out


def plan(needs, mods, runtime=None, extra=()):
    open_needs = [n for n in needs if n.split(':')[0] not in SKIP]
    runtimes = [m for m in mods.values() if m['kind'] == 'runtime']
    if runtime:
        if runtime not in mods:
            sys.exit(f'{runtime}: no such module (harness.py modules)')
        rt = mods[runtime]
    else:
        def score(r):
            layers = [m for m in mods.values() if m['kind'] == 'layer' and r['id'] in m['runtime']]
            covered = set(r['provides']).union(*(set(m['provides']) for m in layers))
            return (len(set(r['provides']) & set(open_needs)), len(covered & set(open_needs)), len(r['seen']))
        rt = max(runtimes, key=score) if runtimes else None
    chosen = [rt['id']] if rt else []
    left = [n for n in open_needs if not rt or n not in rt['provides']]
    layers = [m for m in mods.values() if m['kind'] == 'layer' and (not rt or rt['id'] in m['runtime'])]
    while left:
        best = max(layers, key=lambda m: (len(set(m['provides']) & set(left)), len(m['seen'])), default=None)
        if not best or not set(best['provides']) & set(left):
            break
        chosen.append(best['id'])
        left = [n for n in left if n not in best['provides']]
    chosen = with_requires(mods, chosen + [e for e in extra], [])
    cover = {n: next((i for i in chosen if n in mods[i]['provides']), None) for n in needs}
    return chosen, cover


def script(mods, chosen, needs):
    lines = ['#!/usr/bin/env bash', '# Written by tools/live/harness.py plan. Read it before you run it.',
             f'# Needs: {" ".join(needs)}', 'set -euo pipefail', '']
    for v in dict.fromkeys(v for i in chosen for v in mods[i]['vars']):
        lines.append(f': "${{{v}:?set {v} first}}"')
    for i in chosen:
        m = mods[i]
        lines += ['', f'# ---- {i}: {m["title"]} ({m["path"]})']
        setup = sections(m['body']).get('Set up', '')
        blocks = re.findall(r'```bash\n(.*?)```', setup, re.S)
        lines += [b.rstrip() for b in blocks] or ['# (no steps written yet: see the module)']
    return '\n'.join(lines) + '\n'


KINDS = ('cpu', 'os', 'extender', 'api', 'lib', 'drm', 'store', 'media', 'policy')


def check(mods):
    """What is wrong with the modules, as text lines."""
    out = []
    for m in mods.values():
        where = m['path']
        if os.path.basename(where)[:-3] != m['id']:
            out.append(f'{where}: id {m["id"]} differs from the file name')
        if m['kind'] not in ('runtime', 'layer'):
            out.append(f'{where}: kind must be runtime or layer')
        if not m['provides']:
            out.append(f'{where}: provides nothing')
        for n in m['provides']:
            if ':' not in n or n.split(':')[0] not in KINDS:
                out.append(f'{where}: need {n} is not kind:value with a known kind')
        for r in m['runtime']:
            if r not in mods or mods[r]['kind'] != 'runtime':
                out.append(f'{where}: runtime {r} is not a runtime module')
        for r in m['requires']:
            if r not in mods:
                out.append(f'{where}: requires {r}, which does not exist')
        have = sections(m['body'])
        for s in ('What it does', 'Set up', 'Check'):
            if s not in have:
                out.append(f'{where}: no "## {s}" section')
    return out


def option(argv, name):
    vals = []
    while name in argv:
        i = argv.index(name)
        vals.append(argv[i + 1])
        del argv[i:i + 2]
    return vals


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__); return 0
    mods = load()
    as_json = '--json' in argv
    if argv[0] == 'modules':
        rows = sorted(mods.values(), key=lambda m: (m['kind'] != 'runtime', m['id']))
        if as_json:
            print(json.dumps([{k: m[k] for k in ('id', 'kind', 'title', 'runtime', 'provides', 'requires', 'path')} for m in rows], indent=1))
        for m in [] if as_json else rows:
            print(f'{m["id"]:<20} {m["kind"]:<8} {m["title"]}\n{"":<29} provides {", ".join(m["provides"])}')
        return 0
    if argv[0] == 'check':
        problems = check(mods)
        print('\n'.join(problems) or f'{len(mods)} modules, 0 problems')
        return 1 if problems else 0
    if argv[0] == 'show':
        m = mods.get(argv[1]) if len(argv) > 1 else None
        if not m:
            sys.exit('give a module id (harness.py modules)')
        print(open(m['path'], encoding='utf-8').read())
        return 0
    if argv[0] != 'plan':
        sys.exit(f'unknown command {argv[0]}')
    runtime = (option(argv, '--runtime') or [None])[0]
    extra, out = option(argv, '--with'), (option(argv, '--script') or [None])[0]
    cpu = '--no-cpu' not in argv
    paths = [a for a in argv[1:] if not a.startswith('--')]
    import needs as analyser
    found = analyser.analyse(paths, cpu)
    for e in extra:
        if e not in mods:
            sys.exit(f'{e}: no such module (harness.py modules)')
    chosen, cover = plan(list(found), mods, runtime, extra)
    gaps = [n for n, m in cover.items() if m is None and n.split(':')[0] not in SKIP]
    if as_json:
        print(json.dumps({'modules': chosen, 'cover': cover, 'gaps': gaps,
                          'evidence': found}, indent=1))
    else:
        print('Modules, in order:')
        for i in chosen:
            print(f'  {i:<20} {mods[i]["title"]}')
        print('Needs:')
        for n in found:
            where = cover[n] or ('(shown only)' if n.split(':')[0] in SKIP else 'NO MODULE')
            print(f'  {n:<28} {where:<20} {found[n][0]}')
        if gaps:
            print(f'{len(gaps)} needs have no module: write one from docs/harness/modules/_template.md, '
                  'or try `gameframe advise "<need>"`.')
    if out:
        with open(out, 'w') as f:
            f.write(script(mods, chosen, list(found)))
        os.chmod(out, 0o755)
        print(f'script: {out} (nothing was run)')
    return 1 if gaps else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
