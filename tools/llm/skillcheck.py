#!/usr/bin/env python3
"""Check that each skill works on its own: give an agent one skill and a task on the sample game.

Usage:
  skillcheck.py [--tasks FILE] [--skill NAME ...] [--budget USD] [--model ID] [--keep]
                [--report FILE]

  --tasks   the tasks (default: tests/skill-tasks.json)
  --skill   run only the tasks for this skill (repeat for more)
  --budget  the most one task may spend, in US dollars (default 1.00; provider-reported cost)
  --model   the model (default: selected provider profile; GAMEFRAME_MODEL overrides one run)
  --keep    keep each task's folder and print where it is
  --report  also write the results as Markdown to FILE

AGENTS.md rule 4 says an agent that loads one skill must be able to do that step from it
alone. For each task this tool makes an empty folder holding only the sample game's built
binaries (examples/sample-game/build, never its C sources), then runs `claude -p --bare` there
with the skill's SKILL.md as the whole system prompt. Configure an Anthropic-compatible
provider, including a local server, for this agent run. The agent writes its answer to
answer.txt. The task passes when answer.txt holds every string in the task's "expect" list
(case is ignored). Build the sample first (tests/run.sh or examples/sample-game/build.sh).

A task is {"skill": "re-format", "task": "what to do", "expect": ["strings", "in the answer"]}.
Each expected string comes from the sample's ground truth, never from the agent.

It uses the configured provider (client.py). Hosted providers may spend credits; three tasks
at the default budget can spend up to 3 US dollars. Local servers may report no meaningful
cost. Exits 1 when a task fails.

What it does not do: stop an agent from reading GameFrame's own files. The prompt asks it not
to read examples/, where the sources are; a pass with a cheat would show in the transcript
(--keep keeps the folder; the transcript is not saved, as --bare runs without persistence).
"""
import json
import os
import shutil
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from client import MODEL, ROOT, run_claude  # noqa: E402

BUILD = os.path.join(ROOT, 'examples', 'sample-game', 'build')
FILES = ('game.exe', 'engine.dll', 'mixer32.dll', 'sample.dat')
PYTHON = os.path.join(ROOT, '.venv', 'bin', 'python')
CONTEXT = f"""

---
You are checking that the skill above is enough on its own. GameFrame, whose tools the skill \
names, is at {ROOT}; run its tools from there with {PYTHON}. The current folder is a game \
project that holds only build/. Do not read {ROOT}/examples/ or any other skill. When you are \
done, write the answer to answer.txt in the current folder, and nothing else there."""


def run_task(t, budget, model, keep):
    skill = os.path.join(ROOT, 'skills', t['skill'], 'SKILL.md')
    if not os.path.exists(skill):
        return {'skill': t['skill'], 'ok': False, 'cost': 0, 'missing': ['(no such skill)']}
    work = tempfile.mkdtemp(prefix=f'skillcheck-{t["skill"]}-')
    os.makedirs(os.path.join(work, 'build'))
    for f in FILES:
        if os.path.exists(os.path.join(BUILD, f)):
            shutil.copy(os.path.join(BUILD, f), os.path.join(work, 'build', f))
    system = open(skill, encoding='utf-8').read() + CONTEXT
    out = run_claude(t['task'], system=system, cwd=work, tools='Read,Glob,Grep,Bash,Write,Edit',
                     budget=budget, model=model)
    path = os.path.join(work, 'answer.txt')
    answer = open(path, encoding='utf-8', errors='replace').read().lower() if os.path.exists(path) else ''
    missing = [e for e in t['expect'] if e.lower() not in answer]
    if not answer:
        missing = ['(no answer.txt)'] + ([out.get('result', '')[:200]] if out.get('is_error') else [])
    if not keep:
        shutil.rmtree(work, ignore_errors=True)
    return {'skill': t['skill'], 'ok': not missing, 'cost': out.get('total_cost_usd', 0) or 0,
            'missing': missing, 'folder': work if keep else ''}


def main(argv):
    if argv[:1] in (['-h'], ['--help']):
        print(__doc__); return 0
    tasks_file, only, budget, model, keep, report = os.path.join(ROOT, 'tests', 'skill-tasks.json'), [], 1.0, MODEL, False, None
    while argv:
        a = argv.pop(0)
        if a == '--keep':
            keep = True
        elif a in ('--tasks', '--skill', '--budget', '--model', '--report') and argv:
            v = argv.pop(0)
            if a == '--tasks': tasks_file = v
            elif a == '--skill': only.append(v)
            elif a == '--budget': budget = float(v)
            elif a == '--model': model = v
            else: report = v
        else:
            sys.exit(f'unknown option {a}')
    if not os.path.exists(os.path.join(BUILD, 'game.exe')):
        sys.exit('build the sample game first: examples/sample-game/build.sh')
    tasks = [t for t in json.load(open(tasks_file)) if not only or t['skill'] in only]
    rows = []
    for t in tasks:
        r = run_task(t, budget, model, keep)
        rows.append(r)
        print(f'  {"ok  " if r["ok"] else "FAIL"}  {r["skill"]:<14} ${r["cost"]:.2f}'
              + (f'  missing: {", ".join(r["missing"])}' if r['missing'] else '')
              + (f'  ({r["folder"]})' if r['folder'] else ''))
    total = sum(r['cost'] for r in rows)
    passed = sum(r['ok'] for r in rows)
    print(f'{passed} of {len(rows)} skills passed on their own, ${total:.2f}')
    if report:
        with open(report, 'w', encoding='utf-8') as f:
            f.write(f'# Skill check\n\nModel: {model}. {passed} of {len(rows)} passed, ${total:.2f}.\n\n'
                    '| Skill | Result | Cost | Missing |\n|---|---|---|---|\n')
            for r in rows:
                f.write(f'| {r["skill"]} | {"pass" if r["ok"] else "fail"} | ${r["cost"]:.2f} | {", ".join(r["missing"])} |\n')
    return 0 if passed == len(rows) else 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
