# Creative SB16 Win16 wave-output completion contract

This contract follows an accepted output buffer through the driver's static queue and completion paths. It describes behavior visible through the wave header and driver callback. No runtime completion trace was captured.

Evidence labels distinguish observed behavior from interpretation and unknowns. The contract contains no code addresses, source excerpts, pseudocode, binary paths, or hashes.

## Inputs and outputs

[Observed binary behavior] Buffer submission takes a wave header that was prepared for the open output stream. The header carries the buffer data and length. The driver uses header flags and per-stream queue state to track submission and completion.

[Observed binary behavior] An accepted submission returns status `0`. A header already marked in queue returns `0x21`. A header without the prepared flag returns `0x22`.

[Observed binary behavior] Completion updates the header flags before invoking the configured driver callback. It sets the done flag (`0x01`), clears the in-queue flag (`0x10`), clears the driver's link fields, and sends callback message value `0x3BD` with the completed header information. No callback is sent when the callback target is absent.

[Inferred from Win16 multimedia conventions] Callback value `0x3BD` identifies the wave-output buffer-done notification. A captured callback or matching Win16 SDK constant would confirm the name.

## Queue and dispatch order

[Observed binary behavior] Accepted headers are appended after headers already queued. The output processor consumes queued data in order and maintains a separate list for headers ready to complete.

[Observed binary behavior] When an output segment is submitted and the stream is not paused, the driver may dispatch the queue to a lower audio service. The processing path advances through buffer data and updates stream position as data is consumed.

[Observed binary behavior] The completion drain removes finished headers from its pending list. For each drained header, the driver marks it done and no longer queued, then invokes the callback if configured.

[Unknown] The static export does not establish the exact lower-service protocol, interrupt boundary, or elapsed time between accepting a header and dispatching its data.

[Unknown] Callback timing relative to hardware completion, application scheduling, and other completed headers was not measured. The code shows the flag update before the callback call, but it does not establish when that completion routine runs.

## Pause, restart, and reset effects

[Observed binary behavior] The pause path marks the stream paused and sends a control request to the lower audio service. The restart path clears the paused state and sends a different control request. These paths affect whether queued data is dispatched.

[Observed binary behavior] The reset path asks the lower service to stop and clears playback-position and active-stream fields.

[Unknown] It is not established whether reset returns every queued header through the normal done callback, clears their flags, or discards them without callbacks.

## Remaining unknowns

[Unknown] The complete mapping from public multimedia messages to the observed driver branches is not verified.

[Unknown] The exact lower-service calls that start, feed, pause, resume, and stop hardware are not mapped to a public protocol.

[Unknown] The number of buffers accepted at once, partial-buffer progress reports, looped buffers, callback ordering under pause/restart, and behavior after a lower-service failure remain unverified.

[Unknown] No runtime result establishes whether the CSP limitation observed in the 86Box profile changes this queue or callback behavior.

## Software-only verification suggestions

[Suggested] In a fresh disposable copy of the DOS 6.22 and Windows 3.11 environment, submit several prepared buffers with distinct contents and lengths. Record each return status, header flags, and callback value and arguments. Verify that each accepted header becomes done and leaves the in-queue state before its callback is delivered.

[Suggested] Pause after submitting multiple buffers, then restart. Record whether data dispatch pauses, how completed headers are ordered, and whether flags change while paused.

[Suggested] Reset with no buffers queued, one buffer queued, and several buffers queued. Record each header's final flags and whether each receives the done callback.

[Suggested] Repeat only in a disposable VM copy. The observed 86Box profile reports CSP unavailable, so CSP-dependent behavior remains outside that profile's coverage.
