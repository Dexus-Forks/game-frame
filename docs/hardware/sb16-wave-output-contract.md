# Creative SB16 Win16 wave-output contract

This contract records one bounded slice of the installed Windows 3.11 audio driver. It covers wave-output open and buffer submission. The findings come from static analysis. No runtime call to these paths was captured.

Evidence labels separate observed behavior from inference and unknowns. The contract describes public behavior only. It omits code addresses, source excerpts, and binary locations.

## Open and format checks

[Observed binary behavior] The open path receives a device context, an open descriptor, a wave format description, and open flags. The driver reads the format tag and format parameters, then asks a lower audio service to check or open the requested format.

[Inferred from binary behavior] Open flags with bit 0 set select a query-only path. The path checks the format and returns without allocating the per-stream state used by a successful open. A Win16 probe with this flag set and clear would confirm the distinction at runtime.

[Observed binary behavior] Format tag 1, which denotes PCM in the Win16 wave format, is the only format family confirmed by this trace. A non-PCM tag is rejected with status `0x20` on a path where the format helper is available. Other format tags are not confirmed as usable.

[Observed binary behavior] The query path returns success (`0`) when the lower service accepts the request. It returns `0x20` when the format check fails or the helper reports an unsupported result.

[Observed binary behavior] A successful non-query open allocates and retains per-stream state, then calls the lower audio service to open the format. It returns `0` after the open succeeds.

[Observed binary behavior] The open path returns status `4` when the device context is already marked open or when stream state cannot be allocated. Other failure branches return `3`, `7`, or `0x20`; the static evidence does not associate every status with one public failure cause.

[Unknown] The accepted PCM combinations of channel count, sample rate, bit depth, and extension size are not established. The driver delegates part of the format decision to a lower service, and no runtime format matrix was captured.

## Buffer submission

[Observed binary behavior] Buffer submission receives a standard wave header and checks its flags before adding it to the output stream.

[Observed binary behavior] The prepared flag is bit `0x02`. If it is clear, the driver returns status `0x22` and leaves the buffer out of the queue.

[Observed binary behavior] The in-queue flag is bit `0x10`. If it is already set, the driver returns status `0x21` and does not add a second queue entry for that header.

[Observed binary behavior] On acceptance, the driver sets the in-queue flag, clears the done flag (`0x01`), resets per-buffer link fields, and appends the header after existing queued headers. The observed queue order follows submission order.

[Observed binary behavior] Accepted submission returns status `0`. The driver may pass the queued buffer to a lower audio service when output is not paused. Completion timing and callback ordering were not captured.

## Call order and state

[Observed binary behavior] A caller first opens a format on a device. The driver retains per-stream state after a successful open. A buffer submission then validates the prepared and in-queue flags before changing the header and appending it to the stream queue.

[Unknown] The complete preparation and unpreparation contract is not established. This slice only establishes the prepared flag requirement at submission time.

[Unknown] The exact public message names for the traced branches have not been verified. This contract names operations by their observable role.

## Status values

| Value | Evidence | Observed result |
|---|---|---|
| `0` | Observed binary behavior | Open/query success or accepted buffer submission |
| `0x20` | Observed binary behavior | Format rejection on the traced open paths |
| `0x21` | Observed binary behavior | Header already marked in queue |
| `0x22` | Observed binary behavior | Header missing the prepared flag |
| `3`, `4`, `7` | Observed binary behavior | Returned by open failure branches; exact public causes are not fully mapped |

## Unknowns

[Unknown] No runtime tests establish the accepted PCM format matrix, treatment of unusual format structures, or behavior with non-PCM data.

[Unknown] Buffer completion, callback sequence, pause and restart interactions, reset behavior, and whether a queued buffer is always started immediately remain unverified.

[Unknown] Hardware register activity, IRQ and DMA use, timing, and output sample conversion are outside this slice.

## Software-only verification suggestions

[Suggested] In a fresh copy of the DOS 6.22 and Windows 3.11 test environment, call query-only open and actual open with PCM formats that vary one field at a time. Include non-PCM tags and malformed format sizes. Record return values and whether a second open succeeds.

[Suggested] Submit headers with the prepared flag clear, set, and already in queue. Record the returned status, changed flags, queue order, completion flags, and callbacks. Repeat with two and then several buffers.

[Suggested] Run these probes only in a disposable VM copy. The installed 86Box profile reports that CSP is unavailable, so tests that depend on CSP remain outside the supported profile.
