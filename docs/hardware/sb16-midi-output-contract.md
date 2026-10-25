# Creative SB16 Win16 MIDI-port output contract

This contract records the MIDI-port open and short-message paths from the installed Windows 3.1 audio driver. It does not cover long-message completion or synth output. The findings come from static analysis; no runtime MIDI trace was captured.

Evidence labels distinguish observed behavior from inference and unknowns. The contract contains no code addresses, source excerpts, pseudocode, binary paths, or hashes.

## Open and initialized state

[Observed binary behavior] The open path accepts a port selector, an open descriptor, and callback information. The traced short-message path accepts port selectors `0` and `1`.

[Observed binary behavior] When the lower audio state is available, open allocates and retains per-port state. It stores the port selector and callback data, clears the current status-byte state, and calls the lower service to initialize the port.

[Observed binary behavior] Successful open returns status `0`. It sends callback message value `0x3C7` when a callback target is configured.

[Inferred from Win16 multimedia conventions] Callback value `0x3C7` identifies the MIDI-output-open notification. A captured callback or matching Win16 SDK constant would confirm the name.

[Observed binary behavior] The open path can return `2` when the port selector or device context is invalid, `3` when the lower audio state is unavailable, `4` when initialization fails, and `7` when local state allocation fails.

[Unknown] The exact number of usable MIDI ports, their physical routing, and whether the two accepted selector values refer to different hardware endpoints are not established.

## Short-message submission

[Observed binary behavior] The short-message path checks that a port selector is in the range `0` through `1` and that the selected port has an active context. An invalid selector or missing context returns status `2`. If the lower audio state is unavailable, the path returns status `3`.

[Observed binary behavior] The path uses per-port state to track a current status byte. Status bytes in the channel-message range update that state. System status bytes clear or replace it according to the message class. Data bytes are interpreted using the retained status state.

[Inferred from MIDI conventions] This retained status behavior provides running-status handling for short messages. Capturing a channel message followed by data-only bytes would confirm the behavior at the lower-service boundary. Exact handling of incomplete or malformed messages was not established.

[Observed binary behavior] A reentrancy counter guards the short-message path. A nested send that exceeds the accepted depth returns status `0x43`.

[Observed binary behavior] A send that passes the checks is converted to a lower-service event and submitted through one lower-service call. The driver returns status `0` after this path. No per-message completion callback is visible in the short-message path.

## Call order and state

[Observed binary behavior] Open creates per-port state before later sends can be handled. Short-message submission validates the selector and active context, enters the reentrancy guard, parses status and data bytes using the per-port status state, calls the lower service, and leaves the guard.

[Unknown] The exact packed message layout, timestamp interpretation, lower-service event format, and physical byte ordering at the MIDI port are not established.

## Status summary

| Value | Evidence | Observed result |
|---|---|---|
| `0` | Observed binary behavior | Open or short-message send succeeded |
| `2` | Observed binary behavior | Invalid selector or missing port context |
| `3` | Observed binary behavior | Lower audio state unavailable |
| `4` | Observed binary behavior | Port initialization failure |
| `7` | Observed binary behavior | Open-state allocation failure |
| `0x43` | Observed binary behavior | Short-message reentrancy guard rejected the send |

These values describe the traced paths only. Their mapping to public multimedia error names was not verified.

## Unknowns

[Unknown] Long-message byte parsing, system-exclusive buffering, long-message completion callbacks, and reset or close behavior are outside this contract.

[Unknown] The lower service's command protocol, port I/O, MIDI timing, message queueing, and hardware errors are not mapped.

[Unknown] The order and timing of physical output relative to the successful return are not measured.

## Software-only verification suggestions

[Suggested] In a fresh disposable copy of the Windows 3.11 environment, open each accepted selector and record return values, open notifications, and behavior when the lower service is unavailable.

[Suggested] Send channel messages with explicit status bytes followed by data-only short messages. Record the bytes observed at a software MIDI capture or loopback endpoint, return values, and any callback activity. Include system status and incomplete-message cases.

[Suggested] Repeat sends during any callback activity to probe the reentrancy guard. Keep long-message tests separate because their completion path is not described here.
