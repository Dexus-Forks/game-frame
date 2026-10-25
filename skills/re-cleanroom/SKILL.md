---
name: re-cleanroom
description: Run a source-visible specification and source-blind implementation workflow when a project wants independent reimplementation from a neutral behavior contract.
---

# Clean-room-style implementation

Use this skill when the project asks for a clean-room process. It gives different roles different
inputs. It does not itself create filesystem or legal separation.

Read [the full process guide](../../docs/working/clean-room-implementations.md). The source-facing
role uses [source-side instructions](references/source-side.md). The implementer uses
[source-blind instructions](references/source-blind.md). Use the
[behavior contract template](references/contract-template.md) and
[packet checklist](references/packet-checklist.md).

## Choose the role

- `re-specifier` may inspect authorized original artifacts and runtime output. It writes a
  neutral behavior contract and keeps source evidence in a separate ledger.
- `re-cleanroomporter` receives only the allow-listed packet. It implements the contract and
  does not inspect original artifacts or the evidence ledger.
- `re-verifier` compares observable results with the contract and, when available, the original.
  It reports failing inputs and values without source-derived explanations.
- `re-lander` records provenance, verification limits and the result.

Never reuse a source-reading task as the source-blind implementer. If both roles can read the same
source files, describe the boundary as procedural. A stronger boundary needs a separate workspace
that contains only the approved packet.

## Handoff

1. Bound one interface or behavior slice. Record the artifact version and desired result.
2. Have `re-specifier` write the contract and evidence ledger separately.
3. Review the contract for source code, pseudocode, unexplained assumptions and missing unknowns.
4. Build an allow-listed packet. Record paths and hashes. Do not include the evidence ledger or
   original artifacts.
5. Start a fresh `re-cleanroomporter` task with only the packet and implementation workspace.
6. Send unanswered behavior questions to `re-specifier`. Give the implementer a revised contract
   in a fresh task.
7. Verify software behavior. Physical hardware is optional. Record unresolved cases and suggest
   software probes.
8. Hand the slice to `re-lander`.

The user chooses the license and project scope. Do not infer a legal conclusion from using this
workflow. Keep game files and decompiled code out of GameFrame itself.
