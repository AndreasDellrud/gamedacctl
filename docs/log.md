---
title: Documentation log
type: journal
status: current
updated: 2026-09-05
sources: []
---

# Documentation log

## [2026-09-05] decision | Accept gamedacctl public identity

Confirmed `gamedacctl` as the repository, Cargo package, binary, and future application name. The live public repository is `AndreasDellrud/gamedacctl`; exact GitHub and crates.io searches found no other matching project or package on this date. Added an explicit unofficial, non-affiliation, non-endorsement, and compatibility-only disclaimer. No artwork is shipped, and future artwork remains subject to the original-design rule.

## [2026-09-05] implementation | Generate and verify single-color animations

Added typed single-color Breathe, Synchronized, Sweep, and Engine-reverse packet generation with whole-second duration bounds and invalid-mode rejection. Generated reports match eight complete GG captures byte-for-byte. Hardware accepted a new purple 10-second Synchronized case and five-second Sweep; Breathe retained a nonzero brightness floor. Both normal and reverse Sweep appeared left-to-right after a simultaneous apply flash, so the reverse bit is exposed as the exact Engine-observed flag without claiming a visibly distinct direction. Restored orange-left/blue-right earcups and green-live/red-muted microphone states afterward; Reflected, ColorShift, and multicolor synthesis remain disabled.

## [2026-09-05] verification | Verify device-access reconnect and rollback

Removed the installed udev rule, reloaded rules, and logically deauthorized and reauthorized only USB control function `1038:1280`. Its hidraw node was renumbered, returned without `uaccess`, and rejected a non-root `gamedacctl` open with `Permission denied`. Reinstalled the byte-identical rule and repeated the kernel-level reconnect; interface `00` automatically regained `uaccess` and the active-user ACL while interfaces `01`, `02`, and audio product `1282` remained excluded. Non-root static control and spoken GameDAC audio then succeeded. The control authorization cycle briefly reset the audio sibling despite targeting `1280`; it recovered automatically, so this development test mechanism must be treated as audio-disruptive. A subsequent manual upstream-cable disconnect produced complete remove/add events for both USB products; interface `00` again gained access automatically, `gamedacctl` restored the four accepted static colors, and the user heard spoken audio through the recovered default sink.

## [2026-09-05] verification | Accept native controller on hardware

Installed the source-controlled udev rule and applied it to the existing interface without moving the mechanically unreliable connector. Verified that only `1038:1280` interface `00` gained the active user's read/write ACL; interfaces `01`, `02`, and audio product `1282` remained root-only. Physically confirmed non-root `gamedacctl` control of orange-left/blue-right steady colors, green-live/red-muted microphone states, earcup-only, microphone-only, and all-zone off, plus exact 10-second Synchronized replay. Restored orange-left/blue-right and green-live/red-muted colors and heard a spoken test through the unchanged six-channel GameDAC default sink at 60 percent. A deliberate reconnect and rule-removal rollback test remain outstanding.

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
