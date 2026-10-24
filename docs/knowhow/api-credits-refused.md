---
id: api-credits-refused
title: A tools/llm run is refused, though the key works and the credits are there
kind: failure
status: solved
era: []
platform: []
runtime: []
hardware: []
api: []
compiler: []
genre: []
symptom: [tool-error]
tags: [api-credits, claude-api, headless, workspace]
signatures:
  - 'Credit balance is too low'
  - 'does not support this model; version [0-9.]+ or newer is required'
  - 'must include the anthropic-workspace-id header'
  - 'ANTHROPIC_API_KEY is not set'
tools: [client, headless, skillcheck, name_functions, ask]
skills: []
docs: [docs/working/api-credits.md]
seen: ['GameFrame itself, Claude Code 2.1.190 and 2.1.310, 2026-10-24']
---

# A tools/llm run is refused, though the key works and the credits are there

## Symptoms

A tool in `tools/llm/` stops with one of these messages, and nothing is spent:

- `Credit balance is too low`, from `headless.py` or `skillcheck.py`, while `ask.py` works
  with the same key.
- `Claude Code 2.1.190 does not support this model; version 2.1.280 or newer is required`.
- `This API key is not scoped to a workspace, so this request must include the
  anthropic-workspace-id header`.
- `ANTHROPIC_API_KEY is not set`.

## Cause

Each message has its own cause (*observed* on 2026-10-24).

- The credits pay for API-key use. They do not pay for Claude Code started from the desktop
  app. A run started from inside a Claude Code session inherited that session's `CLAUDE*`
  variables, such as `CLAUDE_CODE_ENTRYPOINT`, and counted as part of the session.
- The installed `claude` was older than the model.
- The key belonged to the organization and to no workspace.
- The key was not in the environment or in the `.env` of the current folder, the project or
  GameFrame.

## Evidence

A request through the SDK with the same key and `max_tokens` up to 64000 succeeded, while
`claude -p --bare` failed. With the `CLAUDE*` variables left out, the same `claude -p` call
succeeded for $0.004.

## What to do

Pull GameFrame: since 2026-10-24, `client.py` leaves the `CLAUDE*` variables out of every run
it starts. Then fix whichever message remains.

- An old `claude`: run `claude update`, or set `GAMEFRAME_CLAUDE` to a newer `claude`.
- No workspace: create the key in a workspace, or put `ANTHROPIC_WORKSPACE_ID=...` in `.env`.
- No key: put `ANTHROPIC_API_KEY=...` in the project's `.env`.
- Check the setup with `./gameframe run tools/llm/client.py`.

## Tried

- A larger or smaller `max_tokens`: no change; the balance was never the cause.
