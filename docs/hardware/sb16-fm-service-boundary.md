# SB16FM synth-service boundary

This note follows the FM synth driver's short-message event only to the installed provider boundary. The available static evidence does not establish the note-to-OPL mapping.

## Confirmed behavior

[Observed binary behavior] The Win16 synth wrapper initializes a lower service during open. For a short message, it prepares a packed event and passes it through an indirect lower-service call. The wrapper itself does not expose note-number or velocity conversion into OPL register writes.

[Observed artifact] The wrapper has no static import of the Creative CSP manager. Its lower-service call is supplied indirectly, and the available export does not identify the provider bound to that call.

[Observed artifact] The installed CSP manager exposes wave/CSP operations and EFM handler-management entry points. The available static export does not link those entry points to the synth wrapper's short-message service.

[Observed artifact] The installed virtual sound driver is not covered by a usable function-level export for this question. Its visible handler-map strings do not establish an OPL event converter.

## Boundary and unknowns

[Unknown] It is not established whether the installed driver set contains a software provider that converts the wrapper event into OPL operations, or whether the provider is supplied by a Windows component outside the package or profile.

[Unknown] The lower-service provider identity, event fields, note and velocity formulas, voice allocation, register order, register values, and hardware timing remain unresolved.

## Software-only verification suggestion

[Suggested] In a disposable Windows 3.11 VM copy, run the synth wrapper with a logging lower-service substitute or an instrumented virtual sound provider. Open the synth, send known note-on and note-off events, and record the provider calls and all emulated OPL-port writes. This will establish whether the converter is in the installed profile and reveal its event-to-register sequence without physical audio hardware.
