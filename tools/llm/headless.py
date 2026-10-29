#!/usr/bin/env python3
"""Run one of GameFrame's agents through Claude Code and save its report.

Usage:
  headless.py AGENT TASK [--dir DIR] [--write] [--budget USD] [--model ID] [--report FILE]
  headless.py docs-check [PATH ...] [--budget USD] [--model ID] [--report FILE]

  AGENT     an agent in agents/ (re-decoder, re-measurer, re-checker...)
  TASK      what it should do, in one message: it cannot ask you anything
  --dir     the folder it works in (default: the project, GAMEFRAME_PROJECT)
  --write   let it edit and write files. Without it, it can read and run commands only.
            Give it a branch or a worktree of its own, and read its diff before you merge
  --budget  the most the run may spend, in US dollars (default 5.00)
  --report  the report's file (default: analysis/llm/runs/<date>-<agent>.md in the folder)

  docs-check  runs re-checker, then re-editor, over the PATHs (default: docs skills GAPS.md),
              without --write: they report and change nothing. The report lists claims whose
              evidence is missing, addresses or files that do not exist, gaps that are already
              covered, and sentences that break the writing rules. Its budget is split between
              the two (default 5.00 in all)

The agent's system prompt is its agents/<name>.md file plus the SKILL.md of every skill it
names, as Claude Code with --bare loads no skills or settings of its own. Configure an
Anthropic-compatible provider, which may be a local model server. The run ends when
the agent ends its turn or the budget runs out. The report is the agent's final message, under a
line with the agent, the model and the cost.

Schedule it with cron or launchd only when requested. It uses the configured provider through
client.py, so the Claude Code GitHub Action is not needed.

What it does not do: commit, push or merge. It never runs without a budget.
"""
import datetime
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from client import MODEL, PROJECT, ROOT, run_claude  # noqa: E402

READ_TOOLS = 'Read,Glob,Grep,Bash'
WRITE_TOOLS = 'Read,Glob,Grep,Bash,Edit,Write'
UNATTENDED = """

---
You run unattended: nobody can answer a question. Make each decision yourself and record it. \
GameFrame is at {root}; its tools run with {root}/.venv/bin/python. End with a report in \
Markdown: what you did, what you found with its evidence, what is left, and anything a person \
must decide."""
READ_ONLY = ' Do not edit or write any file: put every correction in the report instead.'


def agent_prompt(name):
    path = os.path.join(ROOT, 'agents', f'{name}.md')
    if not os.path.exists(path):
        sys.exit(f'{name}: no such agent in agents/')
    text = open(path, encoding='utf-8').read()
    front = re.match(r'^---\n(.*?)\n---\n', text, re.S)
    skills = re.findall(r'^\s+- (re-[\w-]+)\s*$', front.group(1), re.M) if front else []
    parts = [text[front.end():] if front else text]
    for s in skills:
        p = os.path.join(ROOT, 'skills', s, 'SKILL.md')
        if os.path.exists(p):
            parts.append(f'\n\n---\n# Skill {s}\n\n' + open(p, encoding='utf-8').read())
    return ''.join(parts)


def run(agent, task, cwd, write, budget, model):
    system = agent_prompt(agent) + UNATTENDED.format(root=ROOT) + ('' if write else READ_ONLY)
    out = run_claude(task, system=system, cwd=cwd, tools=WRITE_TOOLS if write else READ_TOOLS,
                     budget=budget, model=model)
    cost = out.get('total_cost_usd', 0) or 0
    state = 'failed' if out.get('is_error') else 'done'
    return f'## {agent} ({state}, {model}, ${cost:.2f})\n\n{out.get("result", "").strip()}\n', cost


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__); return 0
    opts = {'--dir': PROJECT, '--budget': '5', '--model': MODEL, '--report': None}
    write, rest = False, []
    while argv:
        a = argv.pop(0)
        if a == '--write':
            write = True
        elif a in opts and argv:
            opts[a] = argv.pop(0)
        else:
            rest.append(a)
    budget, model, cwd = float(opts['--budget']), opts['--model'], opts['--dir']
    stamp = datetime.date.today().isoformat()
    if rest[0] == 'docs-check':
        paths = ' '.join(rest[1:]) or 'docs skills GAPS.md'
        task = (f'Check {paths} in {cwd}. Report each problem with its file and line, '
                'most serious first. Report only; change nothing.')
        parts, cost = [], 0
        for agent in ('re-checker', 're-editor'):
            text, c = run(agent, task, cwd, False, budget / 2, model)
            parts.append(text)
            cost += c
        name = 'docs-check'
    else:
        if len(rest) < 2:
            sys.exit('give an agent and a task')
        text, cost = run(rest[0], ' '.join(rest[1:]), cwd, write, budget, model)
        parts, name = [text], rest[0]
    report = opts['--report'] or os.path.join(cwd, 'analysis', 'llm', 'runs', f'{stamp}-{name}.md')
    os.makedirs(os.path.dirname(os.path.abspath(report)), exist_ok=True)
    with open(report, 'w', encoding='utf-8') as f:
        f.write(f'# {name}, {stamp}\n\nTotal ${cost:.2f}.\n\n' + '\n'.join(parts))
    print(f'${cost:.2f}; report in {report}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
