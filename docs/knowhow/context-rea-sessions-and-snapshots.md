---
id: context-rea-sessions-and-snapshots
title: 'REA through Ghidra: what a session costs, what a snapshot keeps and how labels interact'
kind: context
status: context
platform: [windows-nt, macos]
tags: [rea, mcp, ghidra, snapshot, labels, evidence]
tools: [rea_session, mcp_client, mcp-provider]
skills: [re-rea, re-ghidra, re-trace]
docs: [docs/working/mcp-toolchain.md]
seen: ['Banished (2014), 2026-10-27']
---

# REA through Ghidra: what a session costs, what a snapshot keeps and how labels interact

## What to know

REA 6.7.0 with its Ghidra provider (Ghidra 12.1.4) on Banished's `Application-steam-x64.exe`, a
2 MB x64 program, on an Apple M3 (*measured*):

| Step | Time |
|---|---|
| `open_binary` | 1 to 2 s; Ghidra has not run yet |
| First `analyze_function` of a session | 45 to 50 s: Ghidra imports and analyses the whole binary |
| The same question again in the session | 0.1 s |
| `close_binary` with `snapshot_path` | 1 s |
| Reopen with `snapshot_path`, a question in the snapshot | 0.1 s; the whole run took 3 s |
| Reopen with `snapshot_path`, a new question | 45 s: Ghidra runs again |
| `set_addresses_names` with 1,167 labels in chunks of 400 | runs Ghidra first if it has not run; 1,165 applied |

A snapshot holds REA's answers keyed by question (operation and parameters). It is not a
Ghidra database. It saves time only for questions asked before. One `analyze_function`
dossier added 2 to 3 MB to it. The dossier itself was 0.5 to 1 MB, mostly `native_value_flow`
(the P-Code operations, def-use chains and memory effects). That is too large to read whole,
so read it with `jq`.

Labels and snapshots work against each other (*observed* in REA's limitations and *measured*).
A session that renamed anything cannot save a snapshot: "metadata edits invalidate immutable
analysis snapshots". After a rename, `analyze_function` answers from Ghidra again with the new
names in its pseudocode, even for a question in the snapshot. Names are discarded on close.

`set_addresses_names` takes an object of address to name and answers, in `normalized_result`,
an object of address to whether the name stuck. It refuses an address inside a function that
already has a primary symbol. Ghidra reads `::` in a name as a namespace path. Every name must
be unique.

The `mcpServers.rea` entry that `./gameframe mcp --enable rea` writes starts
`npx -y rea-agents@VERSION mcp`. In an agent's environment `npx` prints one NDJSON notice on
stdout before the server's first answer. A client of its own must skip lines that are not the
JSON-RPC answer. Claude Code copes with it.

## What to do

Use `tools/rea/rea_session.py` (`./gameframe run rea_session ...`). `snapshot BINARY ADDR...`
saves the answers you will come back to. Run it without renames, adding to the old snapshot.
`fn BINARY ADDR...` applies the project's labels, then asks; each run pays the Ghidra cost once.
`fn --snapshot-only` skips the renames and answers warmed functions in 0.1 s, with the labels
only in its summary. `labels BINARY` writes `set_addresses_names` input for an agent's own REA
session. Keep the snapshot and the dossiers in an ignored folder (`analysis/rea/`), because they
hold decompiled code. Keep the text export for whole-program searches.
