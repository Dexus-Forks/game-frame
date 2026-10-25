# Creative SB16 Win16 MIDI short-message contract

This contract narrows the MIDI-port output findings to short-message state and dispatch. It records what the static export supports and leaves the packed argument layout and lower-service event format unresolved. No runtime MIDI trace was captured.

Evidence labels distinguish observed behavior from unknowns. The contract contains no code addresses, source excerpts, pseudocode, binary paths, or hashes.

## Per-port status state

[Observed binary behavior] Opening a port initializes its retained status-byte state to zero.

[Observed binary behavior] During short-message parsing, bytes in `0x80` through `0xEF` replace the retained status byte.

[Observed binary behavior] Bytes in `0xF0` through `0xF7` clear the retained status byte. Bytes in `0xF8` through `0xFF` do not replace the retained status byte in the traced branch.

[Observed binary behavior] A data byte below `0x80` consults the retained status state. If no status byte is retained, the parser records an expected data-byte count of zero. Otherwise, it derives an expected count from the retained status class.

[Unknown] The static export does not recover the full status-to-data-count table or establish how an incomplete or malformed message is reported to the caller.

## Reentrancy guard

[Observed binary behavior] The short-message path increments a per-port in-progress counter before parsing and sending. It accepts the first active invocation. If a nested invocation raises the counter above one, it decrements the counter and returns status `0x43` without sending through the lower service.

[Observed binary behavior] A send that passes the guard decrements the counter after the lower-service call and returns status `0`.

[Unknown] The trigger for nested calls and the externally visible effect of the rejected nested send were not tested at runtime.

## Packed input and lower-service event

[Observed binary behavior] The short-message branch reads message bytes from its dispatcher arguments, processes status and data bytes, and makes one lower-service event call on the accepted path.

[Observed binary behavior] The lower-service call receives an event value represented as two 16-bit parts. The first part is modified by the driver before submission. The second part is zero in the traced short-message path.

[Unknown] The current static export does not establish the exact Win16 packed input layout, byte-to-word packing order, meaning of the modified first part, or whether the returned event preserves message bytes in caller order. Ghidra's inferred parameter types are not reliable enough to make those claims.

[Unknown] The lower-service event's field meanings and its relationship to physical MIDI bytes are not established. This contract does not define a wire format.

## Observable status values

| Value | Evidence | Observed result |
|---|---|---|
| `0` | Observed binary behavior | Short-message path accepted and submitted to the lower service |
| `2` | Observed binary behavior | Invalid selector or missing port context |
| `3` | Observed binary behavior | Lower audio state unavailable |
| `0x43` | Observed binary behavior | Nested short-message invocation rejected by the reentrancy guard |

These values describe the traced path only. Their mapping to public multimedia error names was not verified.

## Unknowns and software-only verification

[Unknown] The exact public message name and packed Win16 argument layout are unresolved. Long-message parsing, timestamps, callback completion, port I/O, and hardware timing are outside this slice.

[Suggested] In a fresh disposable Windows 3.11 VM copy, send short-message arguments that exercise data-only bytes, each channel-status class in `0x80` through `0xEF`, system bytes in `0xF0` through `0xF7`, and real-time bytes in `0xF8` through `0xFF`. Capture the argument words, lower-service event words, return values, and retained status behavior.

[Suggested] Use a software MIDI capture or loopback endpoint where available. Compare status-bearing messages with follow-on data-only messages. Trigger a nested send during callback activity if the environment permits and record the result. Keep all probes in a disposable VM copy.
