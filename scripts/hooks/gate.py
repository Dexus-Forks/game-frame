#!/usr/bin/env python3
"""GameFrame's gates, for git and for each coding agent's hooks.

Usage (an agent's hook JSON is on stdin):

    gate.py commit     Claude Code PreToolUse, Bash `git commit ...`
    gate.py push       Claude Code PreToolUse, Bash `git push ...`
    gate.py write      Claude Code PreToolUse, Write and Edit
    gate.py session    Claude Code SessionStart
    gate.py codex      Codex PreToolUse (Bash, apply_patch)
    gate.py gemini     Gemini CLI BeforeTool (run_shell_command, write_file, replace)
    gate.py cursor     Cursor beforeShellExecution, or preToolUse for Shell
    gate.py copilot    GitHub Copilot preToolUse (bash)
    gate.py staged     git pre-commit (scripts/hooks/git/pre-commit): no stdin; checks the
                       index of the repository in the current folder

What the checks do (scripts/hooks/gates/checks.py):

- commit: blocks a commit that stages game material (a file in target/, analysis/, derived/,
  game-data/ or save-game/, or a game binary or data file such as .exe, .dll, .pkg, .sav) or
  added lines that look like decompiled code. It also blocks a commit made by a GameFrame stage
  or task agent other than re-lander, which is the only agent that commits.
- push: blocks a push made by a GameFrame agent other than re-lander. The main session and
  people can push; the project's permissions ask the user first.
- write: blocks writing decompiled code (Ghidra pseudocode) into a file outside analysis/,
  derived/ and target/. Findings are written in prose, with addresses and constants.
- session: adds a few facts to the start of a session: the tools found, and the next open
  items in docs/CHECKLIST.md.

The agent comes from Claude Code's agent_type, else from GAMEFRAME_AGENT in the environment,
else from a leading GAMEFRAME_AGENT=re-name on the shell command. The agent modes find commit
and push in the command themselves, because not every agent can filter hooks by command.

A block exits with status 2 and says why on stderr. Claude Code, Codex, Gemini CLI, Cursor,
Copilot and git all treat that as a refusal. Every other case exits 0 with nothing on stdout
(session prints its JSON) and leaves the decision to the normal permissions. Hooks are a
safety net for honest mistakes. They are not a security boundary: an agent or a person that
wants to get around them can (git commit --no-verify skips the git hook). It does not check
file writes in Cursor or Copilot, whose hook input for edits is not documented. Standard
library only.
"""
import json
import os
import sys

sys.dont_write_bytecode = True  # no __pycache__ in the repository or a copied project
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), 'gates'))
from adapters import AGENTS, checks_for  # noqa: E402
from checks import Check, env_agent, reason  # noqa: E402
from session import context  # noqa: E402

CLAUDE = ('commit', 'push', 'write')
MODES = (*CLAUDE, 'session', *AGENTS[1:], 'staged')


def block(why):
    print(why, file=sys.stderr)
    sys.exit(2)


def main():
    if len(sys.argv) != 2 or sys.argv[1] not in MODES:
        print(__doc__)
        return 0 if len(sys.argv) == 1 else 2
    mode = sys.argv[1]
    if mode == 'staged':
        why = reason(Check('commit', os.getcwd(), env_agent()))
        if why:
            block(why)
        return 0
    try:
        event = json.load(sys.stdin)
    except ValueError:
        return 0  # no usable input: leave the decision to the permissions
    if not isinstance(event, dict):
        return 0
    if mode == 'session':
        root = event.get('cwd') or os.getcwd()
        print(context(root, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))))
        return 0
    found = checks_for('claude', event, mode) if mode in CLAUDE else checks_for(mode, event)
    for check in found:
        why = reason(check)
        if why:
            block(why)
    return 0


if __name__ == '__main__':
    sys.exit(main())
