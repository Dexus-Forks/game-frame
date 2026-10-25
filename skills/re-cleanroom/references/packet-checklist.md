# Source-blind packet checklist

## Include

- The neutral behavior contract and its revision.
- A manifest with each allowed file path and SHA-256 hash.
- Public hardware, operating-system or API documents the implementer may use.
- The new code's relevant interfaces, crate conventions and build instructions.
- Synthetic input/output examples that are already in the contract.
- A note naming who prepared the packet and which role will receive it.

## Exclude

- Original binaries, installers, extracted files, firmware and assets.
- Disassembly, decompiler output, Ghidra databases, REA evidence bundles and private source
  notes.
- Source-reader transcripts, source-specific addresses and explanatory comments.
- Earlier implementations written after source inspection.
- Test fixtures copied from the original, unless the project owner explicitly authorizes them
  and the contract identifies them as permitted data.

## Record

- Packet creation date and manifest revision.
- File paths, sizes and hashes.
- Any exception the project owner approved.
- Whether the implementation workspace can access excluded files by another path.
- Whether the split is prompt-based, workspace-isolated or independently staffed.

If excluded files remain reachable from the implementer's environment, record that limitation.
Do not describe a prompt-only boundary as technical isolation.
