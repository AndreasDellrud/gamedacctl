---
title: Compatibility and support boundary
type: operations
status: current
updated: 2026-11-09
sources:
  - src/protocol.rs
  - src/transport.rs
  - docs/overview.md
  - docs/protocol.md
  - docs/experiments.md
  - docs/device-access.md
---

# Compatibility and support boundary

## Verified configuration

The following combination has been exercised on physical hardware rather than inferred from product-family names:

| Component | Verified value |
| --- | --- |
| Headset | Original wired SteelSeries Arctis Pro |
| Controller | Original GameDAC, USB control identity `1038:1280`, device release `1.40` |
| Lighting interface | HID interface `00` |
| Audio function | Separate USB identity `1038:1282` |
| Reference software | SteelSeries GG 118.0.0 for the native-accepted baseline; GG 122.0.0 for the separately identified marker/reflection research captures |
| Native test system | Arch Linux with Omarchy, PipeWire/WirePlumber, GTK 4, and libadwaita |

The controller discovers the GameDAC by vendor ID, product ID, and interface number; it does not depend on a changing `/dev/hidrawN` path. One matching device is currently supported. Selection among multiple matching GameDAC units is backlog work.

## Device support

| Device or environment | Status | Notes |
| --- | --- | --- |
| Original GameDAC `1038:1280` with wired Arctis Pro | Verified | All released lighting operations have physical acceptance on this combination. |
| Original GameDAC audio `1038:1282` | Preserved, not controlled | Audio remains a separate Linux device; `gamedacctl` does not implement proprietary DTS or Headphone:X processing. |
| Other Linux distributions | Expected but unverified | Requires compatible GTK 4/libadwaita libraries, hidraw, udev, and an active local session that honors `uaccess`. |
| GameDAC Gen 2, Arctis Nova products, wireless base stations, and other SteelSeries devices | Unsupported | Product names do not imply protocol compatibility. Unknown USB identities are rejected. |
| Corsair ST100 | Unsupported by this project | Its audio and lighting devices are unrelated to GameDAC control. |
| Multiple original GameDAC units connected at once | Unsupported | The current transport opens the first exact interface match. |

## Lighting support

| Operation | Status | Boundary |
| --- | --- | --- |
| Steady | Verified | Independent left, right, microphone-live, and microphone-muted RGB colors. |
| Off | Verified | Implemented as observed steady black for earcups, microphone states, or all zones. |
| ColorShift | Verified subset | Exactly two colors, synchronized across the earcups, with a whole-second duration from 1 through 30. |
| Multi Color Breathe | Verified subset | One through four colors, synchronized across the earcups, fading to black between colors, with a whole-second duration from 1 through 30. |
| Connected Sweep | Verified subset | One color, with the observed connected apply behavior and optional captured reverse flag. |
| Exact captured animation replay | Research feature, sampled reports physically accepted | Only complete recognized reports from explicit capture frames; requires `wireshark-cli`. Accepted GG 122 samples include three-color markers, both Reflected orientations, and the captured 12.34-second configuration. |
| Positioned Color Flow / Reflected | Implemented, sampled configurations verified | Two or three colors at increasing integer percentages, first 0% and last below 100%; exact 1.00–30.00-second duration. Three-color 0%/25%/70% normal/reversed Reflected at 12.00 seconds and Synchronized at 12.34 seconds passed owner lighting/audio acceptance. Other bounded combinations generalize the captured formula; excessive signed-byte rates are rejected. |
| Extended GG timing layouts | Owner accepted bounded approximation | Opt-in Flow extends to 14 markers and nonzero first positions; positioned Pulse accepts fractional duration. Sampled native acceptance and remaining evidence limits are described below. Duplicate/100% endpoints remain rejected. Legacy implicit Flow remains exactly two colors. |

The local opt-in GG timing extension (`color-shift-markers --gg122`, or the GUI's GG timing switch) implements two through fourteen Flow markers, nonzero-start holds, and minimum transition duration 33. The local `breathe-markers` command implements one through four positioned Pulse colors and fractional duration. The [2026-11-09 native experiments](experiments.md#native-six-color-flow-experiment-2026-11-09) received qualitative owner acceptance for sampled six-color Flow, nonzero-start Reflected motion, minimum-duration Flow, and zero-start fractional Pulse with visually complete dimming and working audio. Pulse's nonzero-start change was too subtle for the owner to assess confidently. **Expanded controlled GG corroboration remains missing**, exact physical periods were not measured, and the full 14-color boundary was not physically exercised. The owner accepted these limitations as close enough for task completion, waiving further GG capture and definitive visual Pulse-hold confirmation. This bounded approximation retains its separate status from the verified subsets above. Existing marker profiles without `gg122: true` retain the previously accepted behavior.

“Verified” means a generated command passed deterministic packet checks and its visible behavior was accepted on the configuration above. Exact captured replay has a separate, explicitly labeled acceptance scope. Neither means the GameDAC firmware reports its current lighting state back to the application. The GUI reports requested writes and saved profiles, not authoritative firmware readback.

## Safety and issue reports

The release transport opens only `1038:1280` interface `00` and sends only packet families already observed from GG. It does not update firmware, fuzz commands, or change the `1038:1282` audio interface. The packaged udev rule grants the active local desktop user access only to that known lighting interface.

When reporting a compatibility problem, include the USB vendor/product IDs, interface number, GameDAC device release shown by `lsusb -v`, Linux distribution, package version, and the exact `gamedacctl` error. Do not publish USB captures until they have been filtered to the target device and checked for unrelated traffic, serial numbers, and credentials.
