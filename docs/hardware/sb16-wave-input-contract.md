# Creative SB16 Win16 wave-input contract

This contract records a bounded Windows 3.1 wave-input path. It covers input open and format checks, buffer submission, and the statically visible completion behavior. No runtime recording trace was captured.

Evidence labels separate observed behavior from inference and unknowns. The contract contains no code addresses, source excerpts, pseudocode, binary paths, or hashes.

## Input open and format checks

[Observed binary behavior] The open path receives an input device context, an open descriptor, a wave format description, and open flags. It reads the format tag and format parameters, then asks a lower audio service to check or open the requested format.

[Inferred from binary behavior] Open flags with bit 0 set select a query-only path. This path performs the format check and returns without creating the per-stream state used by a successful open. Comparing Win16 calls with this flag set and clear would confirm the distinction at runtime.

[Observed binary behavior] PCM format tag 1 is the only format family confirmed by this trace. A non-PCM tag is rejected with status `0x20` on paths where the format helper is available. Other format tags are not confirmed as usable.

[Observed binary behavior] The query path returns status `0` when the lower service accepts the format. It returns `0x20` when the format check fails or the helper reports an unsupported result.

[Observed binary behavior] A successful non-query open retains per-stream state and returns status `0`. It sends callback message value `0x3BE` if a callback target is configured.

[Inferred from Win16 multimedia conventions] Callback value `0x3BE` identifies the input-open notification. A captured callback or matching Win16 SDK constant would confirm the name.

[Observed binary behavior] Other open failure paths return `4`, `7`, or `0x20`. The static evidence does not associate every status with a single public failure cause.

[Unknown] The accepted PCM combinations of channel count, sample rate, bit depth, and extension size are not established. A lower service makes part of the format decision, and no runtime format matrix was captured.

## Recording buffer submission

[Observed binary behavior] Buffer submission takes a wave header and an input stream handle. The handle must match an active input stream. A mismatch returns status `5`.

[Observed binary behavior] The prepared flag is bit `0x02`. If it is clear, submission returns status `0x22` and does not add the header to the capture queue.

[Observed binary behavior] The in-queue flag is bit `0x10`. If it is already set, submission returns status `0x21` and does not add the header again.

[Observed binary behavior] On acceptance, submission sets the in-queue flag, clears the done flag (`0x01`), resets the header's queue-link fields, and appends it after existing input headers. Accepted submission returns status `0`.

[Observed binary behavior] The capture path copies recorded data into the current queued header and increments its recorded-byte count. It advances to the next header after the current header is filled.

## Completion and callback

[Observed binary behavior] When the input path finishes a header, it sets the done flag (`0x01`), clears the in-queue flag (`0x10`), removes its queue links, and then sends callback message value `0x3C0` if a callback target is configured.

[Inferred from Win16 multimedia conventions] Callback value `0x3C0` identifies the input-buffer data notification. A captured callback or matching Win16 SDK constant would confirm the name.

[Observed binary behavior] The header flags are updated before the callback call. The static path advances through queued headers in submission order.

[Unknown] The export does not establish the elapsed time from device capture to completion callback, callback scheduling relative to interrupts, or behavior when capture data overruns the available headers.

## State and status summary

| Value | Evidence | Observed result |
|---|---|---|
| `0` | Observed binary behavior | Format query accepted, open succeeded, or buffer accepted |
| `5` | Observed binary behavior | Buffer submission stream handle does not match an active input stream |
| `0x20` | Observed binary behavior | Format rejection on the traced open paths |
| `0x21` | Observed binary behavior | Header already marked in queue |
| `0x22` | Observed binary behavior | Header missing the prepared flag |
| `4`, `7` | Observed binary behavior | Returned by open failure paths; exact public causes are not fully mapped |

[Observed binary behavior] The driver keeps per-stream state for the active format and queued input headers.

[Unknown] The exact number of supported input streams and the interaction between simultaneous streams remain open.

## Unknowns

[Unknown] The complete mapping from public multimedia messages to the observed input branches is not verified.

[Unknown] The lower-service format rules, capture start conditions, hardware register activity, IRQ and DMA behavior, and audio timing are not established.

[Unknown] The handling of pause, restart, stop, and reset while input headers are queued is not included in this slice. The final flags and callbacks for those cases are unknown.

[Unknown] No runtime result establishes the meaning of the CSP-missing report for wave-input capture. The installed 86Box profile did report CSP unavailable.

## Software-only verification suggestions

[Suggested] In a fresh disposable copy of the DOS 6.22 and Windows 3.11 environment, issue query-only and actual input opens with PCM formats that vary one field at a time. Include non-PCM tags and malformed format sizes. Record return values and whether a second stream can open.

[Suggested] Submit several prepared headers with distinct lengths. Record each return value, recorded-byte count, done and in-queue flags, callback value, callback arguments, and order of completion. Repeat with unprepared and already queued headers.

[Suggested] Exercise pause, restart, stop, and reset with no queued headers, one queued header, and multiple queued headers. Record final header flags and callbacks. Use a disposable VM copy because the current emulated profile reports CSP unavailable.
