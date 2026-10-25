# Creative SB16 Win16 mixer volume-details contract

This contract records the generic mixer control-details read and write paths. Static evidence does not identify a stable selector for the master or another specific output-volume control. No runtime mixer query was captured.

Evidence labels distinguish observed behavior from unknowns. The contract contains no code addresses, source excerpts, pseudocode, binary paths, or hashes.

## Observable request and response

[Observed binary behavior] The mixer dispatcher accepts device selectors `0` and `1` for the traced control-details paths. A request supplies a control identifier, channel and item counts, detail size, and a pointer to the caller's detail buffer.

[Observed binary behavior] The read path validates the control identifier against the driver's descriptor table and checks the request's channel, item, and detail shape. On success, it copies the current stored value or values for the selected control into the caller's buffer and returns status `0`.

[Observed binary behavior] The write path applies the same descriptor and request-shape checks. It validates supplied values against descriptor limits, stores accepted values in per-control state, and calls an apply routine selected by the control descriptor. On success, it returns status `0` and notifies open mixer clients with callback value `0x3D1`.

[Observed binary behavior] A separate switch-like control path changes a stored on/off state and notifies open mixer clients with callback value `0x3D0`. The static evidence does not establish that this path is a volume control.

[Observed artifact] Mixer resource text contains a label “Master Volume Level”. The exported control-detail path does not connect that label to a stable public control identifier.

## State and value handling

[Observed binary behavior] The driver keeps descriptor records and current details in mixer state. The read path returns values from that state. The write path changes the selected record before invoking its descriptor-selected apply routine.

[Observed binary behavior] Some control-detail paths handle a pair of numeric values. The export also contains distinct handling for switch-like and other control types.

[Unknown] The public control identifier for “Master Volume Level”, its channel count, and whether it is the numeric-pair control are not recoverable from the static export.

[Unknown] The accepted numeric range, scale, channel ordering, clamping behavior, and mapping from detail values to audible gain are not established.

## Status values

| Value | Evidence | Observed result |
|---|---|---|
| `0` | Observed binary behavior | Control details read or write succeeded |
| `2` | Observed binary behavior | Dispatcher could not resolve the requested device context |
| `3` | Observed binary behavior | Lower audio state is unavailable |
| `5` | Observed binary behavior | Details helper received an invalid context |
| `8` | Observed binary behavior | Unsupported request mode or operation |
| `0x400` | Observed binary behavior | Invalid details shape or unsupported control category on a traced path |
| `0x401` | Observed binary behavior | Control identifier failed descriptor validation or lookup |
| `0x402` | Observed binary behavior | Request fields or values did not match the selected descriptor |
| `0x0B` | Observed binary behavior | Returned on a details-shape failure path; exact public meaning is unresolved |

These values describe the traced generic paths. Their mapping to public mixer error names was not verified.

## Why master-volume selection is unresolved

[Observed binary behavior] The caller supplies a control identifier. The driver validates that identifier against a runtime descriptor table before reading or writing details.

[Unknown] The available export does not connect the “Master Volume Level” resource label to a control identifier, descriptor type, channel count, or value range. A specific master-volume request cannot be described reliably from the current evidence.

## Software-only verification suggestion

[Suggested] In a fresh disposable Windows 3.11 VM copy, query mixer device capabilities. Enumerate line information and controls for output lines. Find the control labeled “Master Volume Level”. Record its identifier, type, channel count, detail size, and numeric limits.

[Suggested] Read the current details. Write left and right values at zero, midpoint, and maximum. Read them back. Record return statuses, callbacks, and any change reported by the software mixer. Keep the probe in a disposable VM copy.
