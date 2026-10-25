---
name: re-rea
description: When and how to use REA (Reverse Engineer Anything, the rea-agents MCP server) alongside GameFrame - quick evidence-backed inspection of binaries, archives, Android/iOS/.NET/Electron/browser targets, focused decompilation and call graphs through Ghidra, and runtime observation. Use for targets the text export does not cover, for a fast first answer about one function, or when a claim needs a citable Evidence record.
---

# Investigate with REA

REA (https://github.com/morluto/rea) is an MCP server that gives the agent reverse-engineering
tools and records every result as Evidence with an ID, coverage and limits. Its own skill,
`reverse-engineer-anything`, documents each tool in detail: install it with REA's setup
(`re-setup`) and read it before a large REA investigation. This skill says how REA fits into
the playbook's loop. REA is opt-in for each project. Enable it with `./gameframe mcp --enable rea`;
the project launcher uses the pinned `rea-agents` 6.7.0 and reads local overrides from `.env`.

## REA or the text export?

| Use REA's tools (`mcp__rea__*` in Claude Code) when | Use GameFrame's Ghidra export (`re-ghidra`) when |
|---|---|
| The target is not a plain native binary: an APK/IPA, a .NET assembly, an Electron app or ASAR, a browser page, firmware, an archive | You need to search the whole program as text, many times (grep is instant) |
| You want a quick, cited answer about one function: `open_binary` then `analyze_function`, `procedure_callers`, `xrefs` | You trace a system across hundreds of functions over several sessions |
| You need a call path between two functions (`build_call_path`, `trace_call_path`) | You need `decomp_query.py` with your labels applied |
| You need runtime evidence from a running browser or Electron app (`observe_web_session`, `capture_browser_scenario`) | You emulate functions (`re-emulate`): REA does not replace the Unicorn harness |
| You compare two versions of an application (`compare_application_versions`, `find_changed_behavior`) | |
| A finding must carry a citable Evidence ID in a report | |

They agree because both read the same binary through Ghidra; addresses match. Use both: REA for
targeted questions and evidence, the export for broad search.

## The workflow

1. **Route by target** (from REA's skill): a native EXE/DLL → use the Ghidra workflow below;
   an archive or package → `open_binary` then `inspect_artifact`; .NET → `inspect_managed_artifact`;
   Android → `inspect_android_package`; extracted JavaScript or ASAR →
   `analyze_javascript_application`; Windows resources and icons → `inspect_pe_resources`.
2. **Start with a summary** (`binary_overview` when available), then ask focused questions:
   `search_strings`, `search_procedures`, `analyze_function`, `procedure_callers`,
   `procedure_callees`, `xrefs`, `procedure_pseudo_code`.
3. **Name what you learn in the session**: `set_address_name` and `set_comment` keep labels in
   REA's database; also copy each label into `labels/names.tsv` so GameFrame's tools see it.
4. **Keep the Evidence IDs** in your notes; cite them in the finding's document next to the
   address ("observed: FUN_1400be770 writes +0x44 (REA evidence ev_...)").
5. **Close** native sessions with `close_binary` when done.

## Native functions through Ghidra

Use this path when a question needs one native function or its exact instructions. It assumes the
REA tools are connected. If they are missing, follow [When REA is not connected](#when-rea-is-not-connected).

1. Check the active session with `binary_session`. Then call `analysis_provider_candidates` and
   confirm that `ghidra` is available. Note the reported Ghidra and Java versions. Availability
   describes the local provider; it does not prove that a particular binary can be analyzed.
2. If `ghidra` is unavailable or `open_binary` returns `provider_unavailable`, inspect
   `details.rejections` and the provider's `diagnostics`. Run `rea doctor` in the same environment
   that starts REA. See [re-setup](re-setup/SKILL.md) for the versioned, client-specific command
   and setup options. Set `GHIDRA_INSTALL_DIR` to the extracted Ghidra folder and `JAVA_HOME` to
   JDK 21, unless the diagnostics require different versions. The GameFrame project launcher
   reads these from its ignored `.env`; the Ghidra folder must contain `support/analyzeHeadless`.
   Restart the agent after changing the server configuration or its environment, then check
   provider availability again.
3. Open the exact executable with `open_binary(path=..., provider_id="ghidra")`. Record its
   reported format, architecture and SHA-256. Ghidra completes its default analysis before REA
   serves function results. Start with `binary_overview` when available before asking about a
   specific function.
4. Use `analyze_function(procedure="0x...")` for the function identity and body, pseudocode,
   assembly, calls, references, strings and control-flow blocks. Keep its Evidence ID and note
   its coverage and limitations with the finding.
5. For arithmetic, calling-convention or register questions, read every relevant instruction
   with `read_function_instructions(procedure="0x...")` or `procedure_assembly`. Check those
   instructions against the decompiler's claims. Use GameFrame's Ghidra export for repeated
   whole-program searches and the emulator for measured behavior.
6. Before closing with `close_binary`, use `export_evidence_bundle(path=...)` if the evidence
   must persist. The imported program and project are temporary; closing the session removes
   them. Store target-derived evidence under an ignored analysis directory, never in tracked
   files.

## Rules that carry over

- REA's evidence labels match ours: observations, inferences and unknowns stay separate; static
  analysis never proves run-time behaviour.
- Do not repeat an identical REA call; page large results with `inspect_analysis_view` instead.
- REA's runtime tools run the target: run them on the target the user chose, with their
  agreement. A game's online service belongs to someone else; if the user points REA at one,
  mention its terms once, then follow the user's decision.
- Decompiled output from REA is still decompiled code: it stays out of tracked files.

## When REA is not connected

No REA tools in the session (no `mcp__rea__*` in the client) means the MCP server is not enabled
or did not start. If REA would help, enable it with `./gameframe mcp --enable rea`, then restart
the agent session. Follow `re-setup` section 3 to install or register it. If REA tools are
connected but Ghidra is unavailable, use the provider diagnostics in the previous section.
Meanwhile, GameFrame's Ghidra export, `decomp_query.py`, `ghidra-run.sh` and the emulator cover
native binaries.
