---
title: Documentation log
type: journal
status: current
updated: 2026-09-05
sources: []
---

# Documentation log

## [2026-09-05] implementation | Add safe native controller core

Added the initial `gamedacctl` Rust library and CLI with typed lighting zones, exact-size steady reports, computed apply masks, strict validation, HID interface selection, dry-run output, all static microphone and earcup controls, explicit off targets, and guarded exact-capture replay. Regression tests cover every generated packet byte and exact hashes for the physically verified Sweep, Synchronized, and five-second Breathe captures. Added a validated udev rule scoped to `1038:1280` interface `00`; installation and physical acceptance remain pending.

## [2026-09-05] plan | Define application, research, and publication boundaries

Added the canonical native-application roadmap, repeatable reverse-engineering and verification process, and Sweden/EU legal-risk synthesis. Chose a standalone native controller with an optional thin Omarchy shell adapter; recorded milestone acceptance gates, USB capture privacy handling, exact-to-generated replay promotion, compatibility branding, proprietary-material exclusions, and the requirement for legal review before commercialization.

## [2026-09-05] discovery | Capture and verify complete animations

Added full-payload GameDAC-only pcap captures for earcup effects, connected modes, and microphone states. Proved duration encodings, nibble-packed color storage, zone apply masks, and the live/muted microphone mapping. Native replay verified 10-second connected Sweep alternation, matching Synchronized behavior, and 5-second synchronized Breathe. Added guarded exact-pcap replay to the Linux utility; arbitrary animation generation remains intentionally disabled pending header-color resolution.

## [2026-09-04] ingest | Preserve GameDAC RGB investigation

Created the repository from the live Arch/Omarchy investigation. Preserved two filtered USB captures, the physically verified static-color utility, Windows passthrough procedure, audio configuration, packet analysis, experiment labels, and unresolved animation questions.

## [2026-09-04] discovery | Verify native static RGB control

Captured SteelSeries GG 118.0.0 setting red, green, and blue. Replayed the observed feature and output reports from Linux through HIDAPI; both headset zones changed to magenta.

## [2026-09-04] discovery | Map headset zones and animation variants

Verified that packet zone 0 controls the left earcup and zone 1 controls the right. Captured steady, off, breathe-speed, multicolor, per-zone, and direction variants. Animation coefficient semantics remain partly inferred.
