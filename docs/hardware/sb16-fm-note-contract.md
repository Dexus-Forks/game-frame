# Creative SB16 Win16 FM synth note contract

This contract records the FM synth driver's open and short-message paths. It describes the packed message transformation visible in the wrapper and the boundary where it delegates to a lower synth service. It does not establish OPL register writes or note conversion.

Evidence labels distinguish observed behavior from unknowns. The contract contains no code addresses, source excerpts, pseudocode, binary paths, or hashes.

## Open and device state

[Observed binary behavior] The open path accepts device selectors `0` and `1`. It requires the lower synth service to be available, allocates per-open state, stores caller callback and instance data, initializes the lower service, and marks the instance open.

[Observed binary behavior] Successful open returns status `0` and sends callback value `0x3C7` when a callback target is configured.

[Observed binary behavior] Open returns status `2` when the selector or device context is invalid, `3` when the lower service is unavailable, `4` when initialization fails or the instance is already open, and `7` when state allocation fails.

[Unknown] The two accepted selectors are not mapped to a named OPL chip or physical output endpoint.

## Short-message input and transformation

[Observed binary behavior] The short-message path accepts a packed 32-bit message value and a device selector. Selectors outside `0` and `1`, or a missing device context, return status `2`. An unavailable lower service returns status `3`.

[Observed binary behavior] The low byte's high bit selects whether the packed word contains a new status byte. If that bit is set, the driver retains the low byte as status and forwards the packed word without this status insertion step.

[Observed binary behavior] If the low byte's high bit is clear, the driver shifts the packed word left by one byte and inserts the retained status byte into the low byte before forwarding it.

[Observed binary behavior] The retained status byte is stored in driver-global state. The open path does not explicitly clear it. The short-message code does not associate it with the selected device instance.

[Unknown] The exact interpretation of each remaining packed byte, the handling of a data-only message before any status byte has been retained, and the status-specific payload lengths are not established.

## Lower-service handoff and statuses

[Observed binary behavior] A short message that passes validation and the reentrancy guard is passed as a two-word event to an indirect lower synth-service call. The short-message path returns status `0` after the call. The wrapper does not convert note number or velocity into an OPL register sequence on this path.

[Observed binary behavior] A dispatch-context counter permits one active short-message call. A nested call that raises the counter above one is rejected with status `0x43`; the counter is restored before return.

| Value | Evidence | Observed result |
|---|---|---|
| `0` | Observed binary behavior | Open or short-message dispatch succeeded |
| `2` | Observed binary behavior | Invalid selector or missing device context |
| `3` | Observed binary behavior | Lower synth service unavailable |
| `4` | Observed binary behavior | Open initialization failure or already-open state |
| `7` | Observed binary behavior | Open-state allocation failure |
| `0x43` | Observed binary behavior | Nested short-message dispatch rejected |

These values describe the traced paths. Their mapping to public multimedia error names was not verified.

## Unknown OPL behavior

[Unknown] The lower service's event fields, physical byte ordering, and relationship to OPL writes are not established.

[Unknown] Note-on and note-off conversion, pitch-to-frequency formulas, operator selection, voice allocation, instrument tables, velocity scaling, key-on ordering, and register values are not visible in the traced wrapper path.

[Unknown] Callback behavior after a note event and the timing of lower-service processing are not captured.

## Software-only verification suggestions

[Suggested] In a fresh disposable Windows 3.11 VM copy with an instrumented software OPL service, open each accepted selector and send short messages with an explicit status followed by data-only messages. Record the packed input, lower-service event words, and any resulting OPL register writes.

[Suggested] Test note-on, note-off, system status, and data-only messages before and after a status byte. Record state changes and return values. Trigger a nested call if the harness can do so and verify the `0x43` result.

[Suggested] Keep the test in a disposable VM copy. No physical audio hardware is required for collecting lower-service or emulated OPL writes.
