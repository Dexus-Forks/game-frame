#!/usr/bin/env python3
"""GameFrame's know-how hook: after a shell command, name the known entries its output matches.

Usage (the agent's hook JSON on stdin):

    advise.py post      Claude Code PostToolUse or PostToolUseFailure, matcher Bash
    advise.py codex     Codex PostToolUse, matcher Bash
    advise.py gemini    Gemini CLI AfterTool, matcher run_shell_command
    advise.py cursor    Cursor postToolUse (Shell)
    advise.py copilot   GitHub Copilot postToolUse and postToolUseFailure (bash)

A project made for the plugin runs it as ./gameframe advise-hook AGENT. It reads the command and
its output from the event (scripts/hooks/gates/post_events.py has each agent's fields). Then it
looks for two things in docs/knowhow/, GameFrame's and the project's (tools/gameframe/advise.py):

- an entry whose error signature matches the output, such as "privileged instruction" or
  "nested exception on signal stack": a failure seen before, solved or not;
- a context entry whose command pattern matches the command, such as `wine` or `winedbg`: what
  to know when work turns to that runtime, era or kind of hardware.

It adds what it finds to the agent's context, in the reply each agent reads, with the
command that reads each entry in full. It names each entry once per session: it keeps the ids it
has named in a small file in the system's temporary folder, one per session_id.

It never blocks and never fails a tool call: on any problem it exits 0 with nothing on stdout.
It only reads the hook input and the entries. Matching uses the signatures and command patterns
alone, not the looser topic search of `gameframe advise`, so it stays quiet on ordinary output.
Standard library only. The design: docs/working/knowhow.md.
"""
import json
import os
import re
import sys
import tempfile

sys.dont_write_bytecode = True
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(ROOT, 'tools', 'gameframe'))
sys.path.insert(0, os.path.join(ROOT, 'scripts', 'hooks', 'gates'))
import post_events  # noqa: E402
TAIL = 40000          # characters of output searched, from the end
SHOWN_MAX = 3         # entries named after one command


def memory(session):
    name = re.sub(r'[^\w.-]', '_', session or 'none')
    return os.path.join(tempfile.gettempdir(), f'gameframe-advise-{name}.json')


def note(run):
    """The context to add after this shell run, or None."""
    import advise
    cwd = run.cwd
    hits = advise.signature_hits(run.output[-TAIL:], cwd) + advise.command_hits(run.command, cwd)
    path = memory(run.session)
    try:
        with open(path, encoding='utf-8') as f:
            shown = set(json.load(f))
    except (OSError, ValueError):
        shown = set()
    fresh, ids = [], set()
    for h in hits:
        if h.entry.id not in shown and h.entry.id not in ids:
            fresh.append(h)
            ids.add(h.entry.id)
    fresh = fresh[:SHOWN_MAX]
    if not fresh:
        return None
    try:
        with open(path, 'w', encoding='utf-8') as f:
            json.dump(sorted(shown | ids), f)
    except OSError:
        pass
    launcher = './gameframe' if os.path.exists(os.path.join(cwd, 'gameframe')) else 'gameframe'
    head = ('GameFrame know-how: this has been seen before. Read the entry before trying something new;'
            ' when you learn more, add it to the entry (re-land, "Know-how").')
    return head + '\n\n' + '\n\n'.join(advise.brief(h.entry, launcher) for h in fresh)


def main(agent):
    try:
        event = json.load(sys.stdin)
        if not isinstance(event, dict):
            return 0
        run = post_events.read(agent, event)
        text = note(run) if run else None
        if text:
            print(json.dumps(post_events.reply(agent, event, text)))
    except Exception:            # a hint must never break the agent's work
        return 0
    return 0


if __name__ == '__main__':
    mode = sys.argv[1] if len(sys.argv) == 2 else None
    agent = 'claude' if mode == 'post' else mode
    if agent not in post_events.AGENTS:
        print(__doc__)
        sys.exit(0 if len(sys.argv) == 1 or sys.argv[1] in ('-h', '--help') else 2)
    sys.exit(main(agent))
