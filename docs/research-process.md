---
title: Reverse-engineering and verification process
type: pattern
status: current
updated: 2026-09-05
sources:
  - docs/capture-workflow.md
  - docs/protocol.md
  - docs/experiments.md
  - scripts/gamedac-rgb
---

# Reverse-engineering and verification process

## Purpose

This process turns normal SteelSeries GG device traffic into reproducible Linux support without guessing unknown commands. It governs capture design, evidence storage, protocol claims, native replay, implementation, and publication.

## Evidence vocabulary

Every protocol claim uses one of these levels:

| Level | Required evidence |
| --- | --- |
| Observed | The bytes or transfer sequence occur in an immutable raw capture. |
| Inferred | Multiple controlled observations support a semantic interpretation, but no isolated native replay has proved it. |
| Verified | A controlled native replay caused the expected physical behavior reported by the user. |
| Unknown | Evidence does not yet distinguish plausible interpretations. |

GG labels describe actions requested in the UI. They do not prove firmware semantics. Physical behavior is verified only through direct observation after a native replay.

## Safety boundary

Allowed research is limited to ordinary lighting reports observed from GG while operating the owned GameDAC. Do not fuzz opcodes, probe firmware paths, accept firmware updates, copy proprietary executables, or send a packet whose command family was not observed.

Before any native replay:

1. Stop the Windows VM rather than merely closing its viewer.
2. Confirm no Windows container or QEMU process remains.
3. Confirm `1038:1280` interface 0 uses `usbhid`, not `usbfs`.
4. Confirm `1038:1282` remains available to Linux audio.
5. Prepare a verified static color or black command as rollback.

The mechanically unreliable GameDAC USB connector is a separate risk. Keep the device stationary, resolve its current bus/device address immediately before capture, and do not interpret disconnect/reset errors as protocol results.

## Experiment design

Use one-factor-at-a-time experiments. Assign exact values that are easy to locate in packets, record the intended order before capture, disable GG live preview, press Apply once per case, and wait at least two seconds between actions.

A useful sequence has:

- A distinctive static start marker.
- One changed variable per Apply action.
- Exact hexadecimal colors rather than picker dragging.
- Exact duration values rather than informal speed labels.
- Normal and reversed cases where direction exists.
- A distinctive static end marker.
- Immediate notes for deviations, skipped controls, double Apply actions, and physical observations.

Avoid changing color while testing mode, or duration while testing direction. If GG retains separate values per mode, re-enter the same controlled values after switching modes.

## Capture procedure

[Windows and capture workflow](capture-workflow.md) is the operational command reference. Prefer a binary `usbmon` pcap with full snap length because text `usbmon` truncates long feature reports.

The acquisition file temporarily contains all traffic on the selected USB bus because Linux's USB capture backend does not accept the attempted device-address capture filter. Treat it as private and short-lived:

1. Capture into a root-owned temporary file under `/tmp`.
2. Stop capture cleanly and confirm zero reported drops.
3. Filter immediately to the current GameDAC device address with `tshark`.
4. Verify the filtered file contains only that address.
5. Record packet count, capture duration, full snap length, and SHA-256.
6. Delete the unfiltered whole-bus temporary file.
7. Add the filtered capture once under `docs/raw/`; never rewrite it.

Filtered captures must contain no Windows credentials, device serial number, unrelated USB traffic, downloaded installer, or other access-enabling material. Device address is ephemeral metadata, not a stable identifier.

## Analysis procedure

Start from transfer structure before assigning field semantics:

- Group feature reports by zone and Apply boundary.
- Separate 1,024-byte feature reports from 64-byte output reports.
- Compare complete payloads and record every changed offset.
- Look for little-endian counters, bit masks, repeated values, signed opposites, alignment, padding, and retained state.
- Compare identical UI actions across zones and durations.
- Treat a stale or unchanged field as unresolved rather than forcing it to match the UI label.

For a proposed formula, verify it against every relevant capture, not just one example. Record counterexamples and confounded variables. The maintained synthesis belongs in [USB lighting protocol](protocol.md); chronological actions and user observations belong in [Capture experiments](experiments.md).

## Replay gates

Replay progresses through increasingly interpretive stages:

### Exact capture replay

Extract complete feature frames from a filtered pcap, validate length, prefix, repeated zone, supported zone range, and known mode marker, then send the observed zone-mask Apply and commit sequence. This stage changes no packet bytes and is the first hardware test for an animation.

### Exact generated reproduction

Implement a packet builder, but initially give it inputs that must reproduce an existing captured packet byte-for-byte. Any difference blocks hardware transmission until explained.

### New generated combination

Only after fixture equality may one new combination of already understood fields be tried. Change a single dimension, dry-run it, retain static rollback, and ask the user to describe the physical result without prompting them toward an expected answer.

### Product promotion

Expose a setting in the normal CLI or GUI only after its generated packet and physical behavior are verified. Research-only pcap replay remains clearly labeled and must not become a hidden dependency of the distributed application.

## Documentation and change loop

For every meaningful experiment:

1. Add the immutable filtered capture and checksum to `docs/raw/README.md`.
2. Add the labeled sequence and physical observations to `docs/experiments.md`.
3. Reconcile protocol fields and confidence in `docs/protocol.md`.
4. Update implementation boundaries in `docs/overview.md` and the application plan when they change.
5. Add a newest-first entry to `docs/log.md`.
6. Add deterministic fixtures or validation for implemented behavior.
7. Run `scripts/validate` and `git diff --check`.
8. Review the staged diff for private data, proprietary files, unsupported claims, and accidental raw-capture changes before committing.

Publishing should favor a derived protocol specification and independently written implementation. Raw captures remain research provenance and should not be required by release binaries.
