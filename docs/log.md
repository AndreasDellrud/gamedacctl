---
title: Documentation log
type: journal
status: current
updated: 2026-09-05
sources: []
---

# Documentation log

## [2026-09-05] release | Prepare v0.1.4

Raised the package version to `0.1.4` after the launch-readiness and terminology commit passed hosted validation and clean-room Arch bundle inspection. This release presents the GTK controls as Lighting effect, Left, Right, Together, and Across; the CLI presents `effect` as the default off target while retaining the previous `earcups` spelling as a tested hidden alias. It also adds install-first public onboarding, verified compatibility and scope summaries, application and Omarchy panel screenshots, cross-repository responsibility links, focused GitHub discovery metadata, and a privacy-conscious hardware-report form. The immutable tag will be created only after this release commit passes the same hosted validation and packaging gate.

## [2026-09-05] release | Improve repository launch readiness

Reorganized the public README around the downloadable Arch package, verified compatibility boundary, application screenshot, optional Omarchy integration, and structured hardware reporting before development instructions. Added a privacy-conscious GitHub issue form that asks for the exact USB identity, firmware or device release, system, package version, tested features, physical lighting result, and audio result without requesting unfiltered captures or serial numbers. Cross-linked the controller and adapter repositories with their distinct responsibilities and retained the independent-project disclaimer. Repository descriptions and topics now use the specific original-GameDAC, Linux-lighting, GTK, HID, Arch, and Omarchy discovery terms without suggesting support for GG, proprietary audio processing, firmware updates, or newer SteelSeries hardware. Simplified current user-facing terminology to Lighting effect, Left, Right, Together, and Across; the CLI presents `effect` as the off target while retaining `earcups` as a hidden compatibility alias, and protocol or historical documentation keeps the physical term where it remains precise evidence.

## [2026-09-05] release | Publish and verify v0.1.3

Published the immutable annotated `v0.1.3` tag at commit `a54981e66481f551bf57b3aa21f435966fe29da2`. GitHub Actions run `33989660405` passed the full validation suite, clean-room Arch package build and inspection, transferred-artifact checksum gate, and prerelease publication. Independent destination verification downloaded the public package, source archive, `PKGBUILD`, `SRCINFO`, and `SHA256SUMS`; every listed checksum passed, package metadata reported `gamedacctl 0.1.3-1`, the extracted CLI reported `gamedacctl 0.1.3`, and the verified all-zone off dry run remained intact. The release notes document the accepted desktop UX, lighting effects, microphone controls, profile-preserving master switch, compatibility boundary, and changelog. The initial-product epic is complete; multi-device selection, distinct-color direction research, and speculative reconnect-readiness discovery remain explicitly deferred.

## [2026-09-05] release | Prepare v0.1.3

Raised the package version to `0.1.3` after the final desktop UX and corrected-icon commit passed the hosted release dry run. This release adds the adaptive GTK/libadwaita interface, named Solid, Color Flow, and Color Pulse effects, accessible palette and profile management, independent microphone lighting, profile icons, the profile-preserving master lighting state used by both the desktop application and Omarchy panel, reconnect restoration, original packaged icons, and the accepted copy-density and switch-sizing refinements. The immutable tag will be created only after this release commit passes the same hosted validation and Arch bundle build.

## [2026-09-05] fix | Remove application-icon base band

Removed the semi-transparent dark path that covered roughly the lower fifth of the full-color SVG icon. The overlay was intended to add base depth but rendered as a distinct rectangular gray band and muted part of the RGB arc. The existing body gradient and inset border retain depth without it. The corrected 512-pixel render passed SVG and project validation and was visually accepted by the user; the symbolic icon was unaffected.

## [2026-09-05] refinement | Reduce UI copy and normalize switch sizing

Removed repeated group descriptions from the GTK window and retained only short, control-specific constraints. Consolidated the workflow, effect meanings, earcup timing, and master-lighting behavior into a header-level How to use dialog available by click, tooltip, keyboard focus, and F1. Renamed the window to GameDAC Lighting while retaining the original-hardware subtitle. Visual inspection found that the Omarchy GTK theme stretched suffix switches to the full preference-row height; centering the master-lighting, reverse, and reconnect switches at their natural height restored normal horizontal-pill proportions. The user accepted both the quieter text hierarchy and corrected switches.

## [2026-09-05] implementation | Add profile-preserving master lighting state

Added a backward-compatible store-level lighting flag that remains independent from the selected profile. Turning lighting off applies the already verified four-zone steady-black plan; turning it on restores the selected saved profile, and explicitly applying a profile enables lighting. Exposed the state through the stable status response and a narrow profile-lighting CLI command so both the GTK controller and thin Omarchy panel use the same locked persistence and HID boundary. The desktop effect labels now describe behavior—Solid, Color Flow, Color Pulse, Together, and Across earcups—while stored values, protocol documentation, and diagnostics retain the observed protocol terminology. Automated compatibility and dry-run coverage passed. After replacing a zero-sized generic Qt control with an ordinary-QML toggle and using Omarchy's supported shell restart, the user physically accepted the panel's off/restore round trip; status confirmed lighting enabled again with `Everyday` still selected.

## [2026-09-05] implementation | Apply the GNOME HIG desktop UX pass

Reworked the functional GTK/libadwaita prototype around GNOME HIG conventions. Added synchronized native color dialogs and exact accessible hex entry, ordered one-to-four-color palette controls, a fixed independent microphone section that persists with animated earcup effects, explicit new and confirmed-delete profile flows, a built-in symbol picker, narrow-layout breakpoints, standard keyboard shortcuts, transient toasts, and a persistent profile-corruption banner while reserving the device row for connection state. Apply is now the only emphasized view action. Added independently drawn full-color and symbolic dial-and-RGB-arc icons and required them in the Arch package. Human checks accepted both emoji input routes, profile management, section ordering, three- and four-color Breathe application, and the combined animated-earcup plus live/muted microphone behavior. Normal and 439-pixel narrow layouts were visually reviewed, keyboard focus reached and revealed palette controls, Ctrl+N reset and focused the profile name, Ctrl+W closed the application, and Ctrl+Enter applied the physically accepted combined profile.

## [2026-09-05] release | Prepare corrective v0.1.2

Independent verification of the published `v0.1.1` download found that the filtered Cargo archive still contained the nested `docs/AGENTS.md` instruction file. The release remained checksum-valid and the binary package contained only its intended runtime files, but this violated the documented source-exclusion boundary. Excluded the nested file, generalized the release builder to reject `AGENTS.md` or `CLAUDE.md` at any archive depth, and raised the successor version to `0.1.2` without moving or replacing the immutable `v0.1.1` tag.

## [2026-09-05] release | Prepare v0.1.1

Raised the package version to `0.1.1` after the hosted main-branch pipeline independently rebuilt and verified the complete `0.1.0` dry-run bundle. This release carries the hardened XDG profile transactions and the repository-owned, least-privilege GitHub Actions publication pipeline. The immutable `v0.1.1` tag is published only after this release commit passes the same hosted dry run.

## [2026-09-05] implementation | Automate verified release publication

Added a repository-owned release builder and a GitHub Actions pipeline that exercises the same filtered Cargo source, generated checksum-pinned PKGBUILD, frozen Arch build, distributable tests, package inspection, and checksum verification on pull requests and main-branch pushes. Matching version tags pass the verified bundle to a separate least-privilege publish job, which creates a draft, downloads and verifies every asset from GitHub, and only then exposes it as a prerelease. Pinned the official Arch build container by digest and GitHub-maintained actions by full commit SHA; documented routine publication, manual dry runs, failure rollback, unsigned-package verification, and the decision to keep the source-only Omarchy plugin outside this compiled-artifact workflow.

## [2026-09-05] implementation | Harden XDG profile storage

Preserved the version-1 store at `$XDG_CONFIG_HOME/gamedacctl/profiles.json` while replacing predictable temporary writes and stale whole-store saves with serialized read-modify-write transactions. Writers now use a private persistent advisory lock, reload and validate current disk state, write through a unique same-directory `0600` temporary file, synchronize it, atomically rename it, and synchronize the `0700` application directory. Malformed stores remain unchanged and produce an actionable error. Added concurrent-writer, interrupted-update, corruption, XDG-path, permission, and legacy-profile coverage plus backup, recovery, and uninstall documentation.

## [2026-09-05] release | Add a direct Arch release package

Added a checksum-pinned Arch package recipe for the `v0.1.0` source archive so installation does not depend on AUR publication. The package builds both Rust binaries from the locked dependency graph with Cargo frozen, runs the distributable protocol/profile/CLI test subset, and installs the desktop entry, narrowly scoped udev rule, README, and both license texts. Release assets are designed to include the filtered Cargo source archive and the resulting pacman-tracked `x86_64` package.

## [2026-09-05] release | Select dual MIT or Apache-2.0 licensing

Added the complete MIT and Apache-2.0 license texts and Cargo SPDX metadata after the owner selected the recommended Rust dual-license form. Clarified that the license covers the independently written project and does not convey rights in SteelSeries marks or third-party material; Apache contributor patent terms do not represent a patent license from SteelSeries.

## [2026-09-05] release | Define the public compatibility boundary

Added a release-facing compatibility matrix that limits verified support to the original GameDAC `1038:1280` with the original wired Arctis Pro, records the tested device release and Linux environment, distinguishes each accepted lighting subset from unsupported GG parity, and states that successful writes are not firmware readback. Explicitly excluded GameDAC Gen 2, Arctis Nova and wireless products, other USB identities, and multiple-device selection rather than inferring compatibility from product-family names.

## [2026-09-05] verification | Accept named ColorShift and Multi Color Breathe generation

Generated a five-second red/blue two-color ColorShift after exact two-color fixture equality; the user observed a continuous path through purple rather than black. Generated a nine-second red/green/blue Multi Color Breathe; the user observed the requested order and a fade to black between each color. Restored the accepted four-zone Steady configuration. GameDAC Game 5.1 remained the default sink at 60 percent, with Chat and microphone present. The rebuilt GTK application then saved, applied, switched, and correctly reloaded one profile of each new type; persisted JSON and the status API retained their ordered color arrays and effect identities. The enabled Omarchy panel subsequently applied both profiles successfully. Product limits remain two ColorShift colors, one to four synchronized Breathe colors, single-color connected Sweep, and whole-second durations; broader GG marker and fractional-speed parity remains out of scope.

## [2026-09-05] discovery | Distinguish palette effect record structures

Added a full-payload, GameDAC-only preset capture bracketed by three microphone Steady markers. Complete reports show that ColorShift chains continuous color-to-color transitions, while Multi Color Breathe alternates each selected color with black. A retained six-color rainbow contains 12 paired breathe records, and a new six-record shift preset contains continuous transitions. Correlating the reported GG markers and speeds exposed that multicolor stored-color and aggregate-duration fields are not direct copies of the general UI values. Recorded GG's manual limits of four Breathe colors and 14 ColorShift colors, while restricting initial ColorShift generation to the already correlated two-color case.

## [2026-09-05] verification | Accept profile icons and event-driven panel refresh

Human testing saved a `🎮` profile icon, then pasted a rainbow icon and observed the still-open Omarchy panel update automatically after the profile-store change. This verifies profile persistence, status serialization, event-driven file watching across atomic replacement, and live rendering without a polling loop. The desktop emoji selector did not insert into the GTK field, while ordinary paste worked; that input/focus defect is retained in the dedicated UX backlog rather than the accepted profile-data feature.

## [2026-09-05] implementation | Add optional profile icons

Extended the backwards-compatible profile schema with an optional icon containing at most eight Unicode characters. The GTK editor can save and reload an emoji, symbol, or font glyph; the stable status JSON carries the value; and the Omarchy selector renders it while retaining effect-specific fallbacks for existing profiles. Validation rejects surrounding whitespace and oversized values without changing lighting behavior.

## [2026-09-05] verification | Accept Omarchy adapter interactions

Human inspection confirmed that the locally enabled GameDAC bar icon and panel interactions work as intended. Together with the prior manifest, lifecycle, contained-failure, shell-responsiveness, and real profile-apply checks, this completes the thin Omarchy adapter implementation acceptance. Publishing its separate repository remains release work.

## [2026-09-05] implementation | Add thin Omarchy controller adapter

Added versioned `status --json` and saved-profile apply commands as the stable native integration boundary, with CLI tests for machine-readable status, missing profiles, and dry-run state preservation. Created the separate manifest-root `omarchy-gamedacctl` project with an on-demand headset bar widget and profile panel containing no USB, packet, polling, shell interpolation, or privilege logic. Omarchy 4.0.1 accepted the manifest; local add, enable, open, hot reload, disable, removal, and re-add paths completed. A missing executable and a missing profile stayed contained while the shell continued answering IPC. A real `Everyday` profile request through plugin IPC completed and atomically rewrote the selected profile store. The plugin remains locally enabled pending human visual acceptance and publication.

## [2026-09-05] discovery | Test passive HID reconnect readiness

Added a bounded research-only `observe-input` command that reads unsolicited 64-byte reports without sending HID data. A settled three-second sample received no input. A complete 60-second manual upstream reconnect trial recorded input failure at 19.392 seconds, device absence at 19.647 seconds, and renewed `/dev/hidraw15` access at 35.176 seconds, but no input report before removal or after access returned. Existing captured `0x10` and `0x20` reports therefore cannot yet replace the physically accepted four-second post-access fallback. An unobserved feature-read or firmware query remains prohibited; the Beads task stays open for a repeatable raw reconnect capture.

## [2026-09-05] implementation | Add native desktop profiles and reconnect restore

Added a current-stack Rust GTK4/libadwaita application over the shared protocol and transport library, plus an atomically persisted versioned JSON profile model and desktop launcher. The UI exposes only verified static and single-color Breathe/Sweep settings, separates effect style from connected behavior, reports device/write states without claiming physical readback, and keeps reconnect restore off by default. Live acceptance saved and reloaded a profile across a process restart. The first boolean reconnect poll missed a fast cycle; path-aware detection then wrote too early and only one earcup changed. The accepted implementation tracks hidraw path changes, distinguishes transient udev access from persistent denial, waits four seconds, reopens the HID device, and restored synchronized purple Breathe on both earcups. Audio returned and the GameDAC Game 5.1 sink remained default at 60 percent. A separate Beads task retains the requested full UX/UI pass.

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
