"""GameFrame's gates as a package: the checks, and adapters for each agent's hook input.

- checks.py: the core. It knows nothing about agents. It checks the staged files of a
  repository (game material, decompiled code), one write (decompiled code outside the private
  folders), and which GameFrame agent may commit or push.
- adapters.py: turns one agent's hook event (Claude Code, Codex, Gemini CLI, Cursor, GitHub
  Copilot) into a list of Check records for the core.
- session.py: the short note Claude Code adds at the start of a session.

scripts/hooks/gate.py is the command line. Standard library only.
"""
