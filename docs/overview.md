---
title: System overview
type: overview
status: current
updated: 2026-09-05
sources:
  - src/protocol.rs
  - src/transport.rs
  - scripts/gamedac-rgb
  - docs/raw/capture-effects-20260904-2323.usbmon
  - docs/raw/capture-zones-20260904.usbmon
  - docs/raw/capture-full-effects-mic-20260905.pcapng
  - docs/raw/capture-connected-modes-20260905.pcapng
  - docs/raw/capture-effect-presets-20260905.pcapng
---

# System overview

## Goal

Control the wired Arctis Pro headset lighting natively on Arch/Omarchy without keeping SteelSeries GG or a Windows VM running. Preserve normal Linux audio through the GameDAC.

The intended product architecture and delivery gates are maintained in [Native application plan](app-plan.md). All new protocol work follows [Reverse-engineering and verification process](research-process.md), and public distribution is bounded by [Legal and publication considerations](legal.md).

## Hardware boundaries

| Function | USB identity | Linux role |
| --- | --- | --- |
| GameDAC control | `1038:1280` | Three HID interfaces; interface 0 carries lighting configuration. |
| GameDAC audio | `1038:1282` | USB audio plus HID interface 5; remains owned by Linux during control capture. |
| Corsair ST100 audio | `1b1c:0a32` | Separate headset-output USB audio device. |
| Corsair ST100 LEDs | `1b1c:0a34` | Separate ST100 LED driver; not involved in Arctis Pro lighting. |

The GameDAC is physically connected through a USB hub and has a mechanically unreliable connector. Kernel error `-32`, failed resets, and a cable warning were previously observed. Three cables were tested; the current cable is stable while the DAC remains stationary.

## Linux audio state

The GameDAC audio function is independent of its control function. A user-owned PipeWire/WirePlumber profile exposes:

- GameDAC Game as a six-channel `FL FR FC LFE RL RR` sink.
- GameDAC Chat as a separate sink.
- GameDAC microphone as a source.
- Yeti X as the default microphone.

GameDAC Game was verified as the default output at 60–65 percent volume and survived a WirePlumber restart. Test tones were heard on the headset. This verifies the channel map and output path, not SteelSeries' proprietary DTS/Headphone:X processing.

Relevant live files are under `~/.config/alsa-card-profile/` and `~/.config/wireplumber/wireplumber.conf.d/51-steelseries-gamedac.conf`; they are workstation state and are not duplicated here.

## RGB state

SteelSeries GG runs in Omarchy's Windows VM with only `1038:1280` passed to QEMU. Linux retains `1038:1282`, so audio continues during captures.

Verified native behavior:

- Both earcups accept arbitrary steady RGB values.
- Zone 0 is the left earcup.
- Zone 1 is the right earcup.
- Complete captured Breathe packets replay from Linux.
- Connected Sweep alternates between earcups; Synchronized breathes together.
- Zone 2 is the live/unmuted microphone LED state.
- Zone 3 is the muted microphone LED state.
- The commit sequence persists the chosen state after the command exits.
- The Rust `gamedacctl` transport applies independent earcup colors, microphone live/muted colors, every explicit off target, and captured Synchronized animation without root.
- The six-channel GameDAC default audio sink remained available and produced an audible spoken test after the controller acceptance sequence.
- Removing the scoped udev rule and reconnecting makes non-root access fail closed; reinstalling it grants access automatically to newly enumerated interface `00` nodes after both logical and manual cable reconnects.

Implemented and physically accepted through the Rust controller:

- Independent steady left, right, live-microphone, and muted-microphone colors.
- Earcup, microphone, or all-zone off through verified steady black.
- Guarded exact replay of complete captured reports.
- Generated single-color Breathe, synchronized earcups, and connected Sweep for whole-second durations from 1 through 30 seconds.
- Generated two-color ColorShift and one-to-four-color Multi Color Breathe across synchronized earcups.
- Deterministic dry-run output and fail-closed packet validation.

Still outside the generated controller boundary:

- ColorShift lists longer than two colors, arbitrary GG marker positions, and fractional multicolor speed mapping.
- Reflected behavior.

## Current implementations

`gamedacctl` is the native Rust controller. Its pure protocol layer constructs zero-filled Steady and bounded animated reports, computes exact zone masks, validates captured reports, and is isolated from the HID transport. It selects only `1038:1280` interface 0. Tests compare generated Sweep, reversed Sweep, Synchronized, Breathe, and two-color ColorShift reports byte-for-byte with Engine capture fixtures. Physical acceptance additionally verified red-to-blue ColorShift through purple and red/green/blue Multi Color Breathe with black between colors, followed by Steady rollback without disrupting the configured audio device.

`gamedacctl-gui` is the native GTK4/libadwaita front end over the same library. It exposes Steady, two-color ColorShift, and one-to-four-color Multi Color Breathe, while retaining the captured connected Sweep option and legacy single-color profile decoding. It reports disconnected, transient access, persistent permission, and write-result states and atomically stores versioned JSON profiles in the user's XDG configuration directory. Reconnect restore is opt-in. It detects both absence and changed hidraw paths, waits four seconds for firmware initialization, reopens the device, and then sends the last selected saved profile. Hardware testing verified persistence across a full process restart and successful two-earcup restore after a fast manual USB reconnect.

`scripts/gamedac-rgb` uses the installed HIDAPI library through Python `ctypes`. It finds vendor `1038`, product `1280`, interface 0, sends the observed 1,024-byte feature reports, and follows them with observed 64-byte apply/save reports. Its research replay mode extracts only complete, recognized lighting reports from a pcap through `tshark`; it does not synthesize unknown animation fields.

The older deployed workstation research copy is `~/.local/bin/gamedac-rgb`. The Rust CLI and GUI use the scoped `uaccess` rule and run without root.

## Explicit unknowns

- The exact firmware interpretation of the animation coefficients, despite a consistent mathematical relationship with color and duration.
- Whether arbitrary multicolor gradients can be generated safely or should initially be exposed as captured presets.
- Whether animation header RGB is an initial state or merely retained UI state. The normal builder sets it to the requested effect color, which worked physically, while exact fixture tests preserve each captured header independently.
- Why the observed single-color Breathe waveform does not fade completely to black despite there being no brightness control in GG.
- Whether unsolicited 64-byte control-interface input reports can serve as a reconnect-readiness signal. Existing captures contain repeated `0x10` and later `0x20` reports, but do not correlate their first appearance after enumeration with successful two-zone lighting writes.
- Which older preview path requires output report `A3`; later saved configurations used `A5 03 0A`, `AC`, and `09`.

A preliminary passive observer found no heartbeat in a settled three-second sample. In a complete 60-second upstream reconnect trial, the active read failed at 19.392 seconds, the device was absent at 19.647 seconds, and the control node became accessible again at 35.176 seconds; no unsolicited input report arrived before removal or after access returned. This does not prove the reports can never correlate with readiness, but it rejects using them as a dependable predicate from current evidence. The physically accepted four-second post-access fallback remains in production pending a repeatable raw reconnect capture.
