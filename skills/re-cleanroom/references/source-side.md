# Source-side role

The source-side role is `re-specifier`. It may inspect the permitted original binary, public
manuals, disassembly and controlled runtime observations. It does not write implementation code.

## Work

1. Read the project's `AGENTS.md`, the task scope and the existing topic documents.
2. Confirm the exact artifact and version. Record file hashes in the private evidence ledger.
3. Trace one bounded API or behavior slice. Use static evidence for structure and runtime probes
   for behavior that static analysis cannot settle.
4. Write the contract in neutral terms. Use input/output tables and state transitions. Record
   callbacks, ordering, errors, lifetime and side effects.
5. Mark claims as observed, measured, inferred or unknown. Add a software-only probe for each
   important unknown when one is available.
6. Keep addresses, tool output, source-specific symbols and detailed observations in the private
   ledger. Link contract statements to ledger entries using opaque IDs such as `E01`.
7. Review the contract with the packet checklist before handing it off.

## Do not

- Write source code, pseudocode or line-by-line translations in the contract.
- Put decompiler output, source strings that are not part of the public API, or source snippets in
  the implementer's packet.
- Fill unknown behavior with a guess to make the contract look complete.
- Explain implementation choices in terms of source structures when a behavior-level statement
  is enough.

## Handoff

Name the contract and evidence ledger separately. State the exact behavior covered, the claims
that remain unknown, and the allowed files for the implementer. The next role is
`re-cleanroomporter` in a fresh task.
