---
title: Documentation log
type: journal
status: current
updated: 2026-09-05
sources: []
---

# Documentation log

## [2026-09-05] discovery | Capture and verify complete animations

Added full-payload GameDAC-only pcap captures for earcup effects, connected modes, and microphone states. Proved duration encodings, nibble-packed color storage, zone apply masks, and the live/muted microphone mapping. Native replay verified 10-second connected Sweep alternation, matching Synchronized behavior, and 5-second synchronized Breathe. Added guarded exact-pcap replay to the Linux utility; arbitrary animation generation remains intentionally disabled pending header-color resolution.

## [2026-09-04] ingest | Preserve GameDAC RGB investigation

Created the repository from the live Arch/Omarchy investigation. Preserved two filtered USB captures, the physically verified static-color utility, Windows passthrough procedure, audio configuration, packet analysis, experiment labels, and unresolved animation questions.

## [2026-09-04] discovery | Verify native static RGB control

Captured SteelSeries GG 118.0.0 setting red, green, and blue. Replayed the observed feature and output reports from Linux through HIDAPI; both headset zones changed to magenta.

## [2026-09-04] discovery | Map headset zones and animation variants

Verified that packet zone 0 controls the left earcup and zone 1 controls the right. Captured steady, off, breathe-speed, multicolor, per-zone, and direction variants. Animation coefficient semantics remain partly inferred.
