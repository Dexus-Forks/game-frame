# SBBASIC driver workflow record

This page records the clean-room-style process used to model bounded Win16 Sound Blaster driver
paths. The implementation target was the Creative SBBASIC Revision 1 package installed in a
disposable DOS 6.22 and Windows 3.11 environment. This work complements the chip and card models;
it does not produce a loadable Windows driver.

The process follows the general [clean-room-style implementation guide](../working/clean-room-implementations.md).
The package and installation evidence is in the [driver interface record](sbbasic-driver-interface.md).

## Environment

The work used a clean MS-DOS 6.22 snapshot and a clean MS-DOS 6.22 plus Windows 3.11 snapshot.
The driver installer ran in a disposable 86Box clone, leaving the clean snapshots unchanged.
This made it possible to restore the original state after configuration changes.

The installed profile reports that CSP is missing. The remaining driver setup loads. CSP was
recorded as an unavailable provider path, not as a reason to stop modeling the wrapper behavior.
Physical hardware was not required.

## Role separation

The source-facing tasks inspected the authorized driver binaries and produced behavior contracts.
Each contract described caller-visible inputs, results, state and known limits. Source addresses
and analysis detail stayed outside the implementer handoff.

Fresh implementer tasks received one contract and the crate conventions. They added logical Rust
models for that contract. They did not receive source-facing transcripts or analysis notes. The
roles used the same host and repository, so the separation was procedural rather than technical.
The result is recorded as clean-room-style work.

The handoff process did not yet create a manifest with hashes for every allow-listed packet file.
The new checklist makes that record part of future runs. This page does not claim that the
earlier prompts provided file-level isolation.

Each slice was kept small enough to identify the layer it covered:

| Slice | Behavior recorded | Implementation module | Implementation boundary |
|---|---|---|---|
| Auxiliary volume | Query and update message results | [win16_aux.rs](../../crates/sound-blaster/src/win16_aux.rs) | Logical query/update model; no Windows ABI or hardware volume change |
| Wave output | Open, format query, buffer submission, queue state and controls | [win16_wave.rs](../../crates/sound-blaster/src/win16_wave.rs) | Logical headers and queues; caller supplies completion and callback events |
| Wave input | Open, query, capture queue and caller-supplied byte progress | [win16_wave_input.rs](../../crates/sound-blaster/src/win16_wave_input.rs) | Logical input queue; no captured audio or hardware overrun model |
| MIDI output | Port open, short messages, status-byte state and reentrancy guard | [win16_midi_output.rs](../../crates/sound-blaster/src/win16_midi_output.rs) | Logical event preparation; no lower-service event encoding or timed output |
| Mixer details | Descriptor-driven read and update operations | [win16_mixer_details.rs](../../crates/sound-blaster/src/win16_mixer_details.rs) | Generic control details; no master-volume selector, gain curve or hardware mixer |
| FM synth wrapper | Open and short-message wrapper behavior | [win16_fm_output.rs](../../crates/sound-blaster/src/win16_fm_output.rs) | Message transformation only; no voice allocator or OPL register writes |
| FM provider boundary | Where the wrapper delegates its event | No implementation yet | Provider and note-to-OPL conversion remain unknown |

The linked contracts contain the observed values and software-only probe proposals:

- [Auxiliary volume](sbbasic-driver-behavior.md)
- [Wave output](sb16-wave-output-contract.md) and [completion](sb16-wave-completion-contract.md)
- [Wave input](sb16-wave-input-contract.md)
- [MIDI output](sb16-midi-output-contract.md) and [short messages](sb16-midi-short-message-contract.md)
- [Mixer volume details](sb16-mixer-volume-contract.md)
- [FM synth wrapper](sb16-fm-note-contract.md) and [provider boundary](sb16-fm-service-boundary.md)

## Tool findings

Ghidra and REA supplied static binary views. 86Box supplied the period PC and guest OS. The
disposable clone and its saved clean bases avoided repeating the Windows installation during
driver experiments.

GameFrame has DOSBox-X debugger scripts and an MCP server configuration. The debugger tools
reported their static command catalog during this task, but the live DOSBox-X endpoint was not
connected. This did not affect the 86Box driver analysis. The current `.mcp.json` configures
REA, DOSBox-X and GameFrame; it does not configure 86Box control.

The Windows 3.1 SDK documents CodeView for Windows and WDEB386. A maintainer has also described
an optional GDB-stub build for 86Box. These are candidates for follow-up software probes. Their
availability and connection to this VM profile have not been established. The
[debugger guide](../observe/debuggers.md) records their roles and limits.

## Result and limits

The Rust modules model the logical contracts. They are not replacement `.DRV` binaries and do not
yet reproduce every Sound Blaster card, Windows service, DSP transfer or OPL event path. The
models have been build- and format-checked, but this work does not claim behavioral parity for
the whole driver package.

The next useful checks are software-only. Use a Win16 harness or logging lower-service substitute
to capture callback and status behavior. Add virtual OPL-port logging for the FM provider path.
Use a named software card reference for DSP, DMA and IRQ integration. Record each result as a
separate contract revision. Physical hardware remains optional.

## Reusable lessons

- Preserve clean operating-system snapshots before installing drivers.
- Split a large driver into observable messages and service boundaries.
- Keep the chip model, card model, Windows wrapper and lower provider distinct.
- Give the implementer behavior tables and state transitions, not source expressions.
- Send unknowns back through a contract revision instead of asking the implementer to infer them.
- Record the tool connection state. A configured MCP server may not have a live emulator attached.
- Build a packet allow-list by hand for now. A packet creation and hash tool remains a GameFrame
  gap until it can be tested against the framework's own samples.
