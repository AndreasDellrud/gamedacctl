---
title: Native application plan
type: plan
status: target
updated: 2026-09-05
sources:
  - docs/protocol.md
  - docs/experiments.md
  - scripts/gamedac-rgb
---

# Native application plan

## Outcome

Build a safe, open-source Linux controller for the original SteelSeries GameDAC and wired Arctis Pro lighting. It should work as a normal desktop application on Linux, preserve GameDAC audio, require no Windows VM after setup, and optionally integrate with the Omarchy shell.

The application must remain useful outside Omarchy. Omarchy integration is an adapter around the native controller, not the owner of USB protocol logic.

## Product boundary

The first supported hardware target is exactly:

| Function | USB identity | Scope |
| --- | --- | --- |
| GameDAC control | `1038:1280`, HID interface 0 | Lighting configuration only. |
| GameDAC audio | `1038:1282` | Must remain under Linux audio ownership and must not be claimed by the controller. |

Firmware updates, audio DSP emulation, DTS/Headphone:X, cloud services, account integration, and unknown GameDAC opcodes are not application goals. Additional SteelSeries devices require their own captures, tests, and support declarations.

## Architecture

```text
GameDAC 1038:1280 interface 0
              |
        protocol transport
              |
      native controller core
       /          |          \
     CLI      desktop UI    D-Bus API
                              |
                    optional Omarchy panel
```

### Protocol core

The core owns packet construction, validation, zone masks, timing fields, and effect models. Packet builders are pure functions with byte-for-byte fixtures derived from documented captures. Transport is a separate boundary so tests never need hardware.

The preferred durable implementation is Rust for typed packet layouts, predictable deployment, and a single native executable. The existing Python utility remains the executable reference during migration and can continue serving as the research tool.

### Device transport

The transport discovers vendor `1038`, product `1280`, interface 0 through HIDAPI. It sends only supported 1,024-byte feature reports and known 64-byte apply/save reports. It must refuse unsupported devices, malformed reports, and unknown modes.

A narrow udev rule should grant the active desktop user access through `TAG+="uaccess"` or an equivalently scoped mechanism. The GUI and Omarchy shell must never launch `sudo`, embed credentials, or run a privileged daemon.

### Native interfaces

The CLI is the stable automation surface. The desktop UI may use GTK4/libadwaita and should call the same library rather than shelling out. A small user service and D-Bus API become justified only when reconnect persistence or multiple front ends require serialization.

The application state consists of named profiles, the last selected profile, and an explicit reconnect policy. The device remains the authority for its current persisted state; application state must not imply that an unverified write succeeded.

### Omarchy integration

Omarchy 4 [shell plugins](https://github.com/omacom/omarchy/blob/quattro/manual/32-shell-plugins.md) are unsandboxed QML loaded into the long-running `omarchy-shell` process. The optional integration should therefore be a thin bar widget plus panel that displays device status, selects profiles, and calls the native controller over D-Bus or a narrow CLI.

The plugin must contain no raw HID access, packet construction, privilege escalation, firmware handling, or long-running polling loop. It should be distributable through `omarchy plugin add` as its own manifest-root Git repository once the native controller has a stable release interface.

## User experience

The complete target control surface is:

- Independent left and right earcup static colors.
- Earcup illumination off through verified static black.
- Synchronized Breathe and connected Sweep with duration expressed in seconds.
- Microphone live/unmuted and muted static colors.
- Supported microphone live effects after their packet builders are verified.
- Named profiles and an optional apply-on-reconnect policy.
- Clear unsupported-device, permission, and disconnected states.
- An advanced diagnostics view that reports USB identities and errors without exposing raw firmware operations.

Effect names must distinguish physically verified behavior from labels merely observed in GG. A setting is not promoted into the normal UI until its generated packet and physical result pass the acceptance process in [Reverse-engineering and verification process](research-process.md).

## Delivery milestones

### Research foundation

This milestone is complete. It includes filtered immutable captures, protocol synthesis, physically verified static color, exact animation replay, zone mapping, duration encoding, and connected Sweep versus Synchronized behavior.

### Safe controller core

Deliver a tested packet library and CLI for static left/right colors, off, microphone live/muted colors, and exact captured animation presets. Replace root execution with a reviewed device-access rule. Acceptance requires fixture equality, rejection tests, device reconnect tests, preserved Linux audio, and physical confirmation for every exposed command.

Implementation status: the Rust packet library, HID transport boundary, CLI, dry-run output, steady/off zone controls, and guarded exact-capture replay are implemented and covered by deterministic tests. Physical tests verified non-root left/right colors, microphone live/muted colors, every earcup/microphone/all-zone off target, and exact Synchronized replay while GameDAC 5.1 audio remained functional. A deliberate physical reconnect test of the installed access rule remains before the complete milestone is closed.

### Generated Breathe and connected modes

Resolve the animated header RGB field, implement generated Breathe duration/color packets, and verify normal and reverse connected behavior. Acceptance requires generated packets to reproduce known captures byte-for-byte before one new color/duration combination is tried on hardware. The application must provide an immediate static-black rollback.

### Desktop application

Add the profile model and native GUI over the stable core. Acceptance requires no root prompt, correct disconnected/error states, persistence across application restarts, and no change to the system audio profile.

### Omarchy adapter

Publish a separate thin shell plugin with a bar indicator and profile panel. Validate it with `omarchy plugin validate`, confirm shell reload behavior, and prove that controller failure cannot destabilize `omarchy-shell`.

### Public release

Add a chosen open-source license, contribution and support boundaries, a compatibility matrix, reproducible packaging, and the legal/publication safeguards in [Legal and publication considerations](legal.md). Commercial distribution or paid support requires a dedicated Swedish/EU legal review first.

## Engineering acceptance rules

Every supported effect must have:

1. A named immutable source capture and action label.
2. A documented packet layout separating observation, inference, and verification.
3. A deterministic fixture test for all 1,024 feature bytes and relevant output reports.
4. A dry-run representation suitable for review.
5. A device test preceded by ownership and audio checks.
6. A human-observed physical result and a known static rollback.
7. Documentation updated in the same change that exposes the feature.

No feature may depend on bundled SteelSeries executables, firmware, graphics, fonts, or captured proprietary assets.
