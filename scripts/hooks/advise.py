#!/usr/bin/env python3
"""GameFrame's know-how hook: after a shell command, name the known entries its output matches.

Usage (Claude Code's hook JSON on stdin):

    advise.py post     Claude Code PostToolUse or PostToolUseFailure, matcher Bash

It reads the command and its output (tool_response's stdout and stderr after PostToolUse; the
error text after PostToolUseFailure). Then it looks for two things in docs/knowhow/, GameFrame's
and the project's (tools/gameframe/advise.py):

- an entry whose error signature matches the output, such as "privileged instruction" or
  "nested exception on signal stack": a failure seen before, solved or not;
- a context entry whose command pattern matches the command, such as `wine` or `winedbg`: what
  to know when work turns to that runtime, era or kind of hardware.

It adds what it finds to the agent's context (hookSpecificOutput.additionalContext), with the
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
TAIL = 40000          # characters of output searched, from the end
SHOWN_MAX = 3         # entries named after one command


def output(event):
    """The text a command printed, from either event's fields."""
    r = event.get('tool_response')
    parts = []
    if isinstance(r, dict):
        parts += [str(r.get(k) or '') for k in ('stdout', 'stderr', 'output', 'error')]
    elif r:
        parts.append(str(r))
    parts.append(str(event.get('error') or ''))
    return '\n'.join(p for p in parts if p)[-TAIL:]


def memory(session):
    name = re.sub(r'[^\w.-]', '_', session or 'none')
    return os.path.join(tempfile.gettempdir(), f'gameframe-advise-{name}.json')


def note(event):
    """The context to add for this event, or None."""
    import advise
    command = (event.get('tool_input') or {}).get('command', '')
    cwd = event.get('cwd') or os.getcwd()
    hits = advise.signature_hits(output(event), cwd) + advise.command_hits(command, cwd)
    path = memory(event.get('session_id'))
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


def main():
    try:
        event = json.load(sys.stdin)
        if not isinstance(event, dict):
            return 0
        text = note(event)
    except Exception:            # a hint must never break the agent's work
        return 0
    if text:
        name = event.get('hook_event_name') or 'PostToolUse'
        print(json.dumps({'hookSpecificOutput': {'hookEventName': name, 'additionalContext': text}}))
    return 0


if __name__ == '__main__':
    if sys.argv[1:] != ['post']:
        print(__doc__)
        sys.exit(0 if len(sys.argv) == 1 or sys.argv[1] in ('-h', '--help') else 2)
    sys.exit(main())
