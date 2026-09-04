---
title: USB lighting protocol
type: architecture
status: mixed
updated: 2026-09-05
sources:
  - scripts/gamedac-rgb
  - docs/raw/capture-effects-20260904-2323.usbmon
  - docs/raw/capture-zones-20260904.usbmon
  - docs/raw/capture-full-effects-mic-20260905.pcapng
  - docs/raw/capture-connected-modes-20260905.pcapng
---

# USB lighting protocol

## Scope and evidence standard

This page covers lighting configuration sent by SteelSeries GG 118.0.0 to original GameDAC control device `1038:1280`. “Observed” means present in a raw USB trace. “Verified” additionally means a Linux replay caused the expected physical change. “Inferred” is a working interpretation that still requires a controlled replay.

## HID interface

Lighting uses USB interface 0 of `1038:1280`. Its HID report descriptor is:

```text
06 c0 ff 09 01 a1 01 09 f0 15 00 26 ff 00 75 08
95 40 81 02 09 f1 95 40 91 02 09 f2 96 00 04 b1
02 c0
```

The descriptor declares 64-byte input and output reports and a 1,024-byte feature report. SteelSeries GG uses control endpoint zero:

| Operation | Setup fields | Size |
| --- | --- | ---: |
| Feature configuration | `bmRequestType=21`, `bRequest=09`, `wValue=0300`, `wIndex=0000` | 1,024 |
| Output/action report | `bmRequestType=21`, `bRequest=09`, `wValue=0200`, `wIndex=0000` | 64 |

HIDAPI requires an extra leading zero report-ID byte in the userspace buffer for these unnumbered reports. The kernel removes that API byte before the USB transfer.

## Steady color packet

The first 12 bytes of the 1,024-byte feature payload are verified:

```text
AA ZZ RR GG BB FF 32 C8 C8 00 ZZ 01
```

| Field | Meaning | Confidence |
| --- | --- | --- |
| `AA` | Lighting configuration prefix. | observed |
| `ZZ` at bytes 1 and 10 | Zone: `00` left earcup, `01` right earcup. | verified |
| `RR GG BB` | Eight-bit steady RGB value. | verified |
| Byte 5 `FF` | Unknown constant in all captured configurations. | observed |
| Bytes 6–8 `32 C8 C8` | Unknown constants in all captured configurations. | observed |
| Byte 9 `00` | Unknown constant. | observed |
| Byte 11 `01` | Steady-mode marker. | verified for steady; mode interpretation inferred |

The Linux utility zero-fills the remainder. That exact construction set both physical earcups to magenta, demonstrating that any unobserved bytes after byte 11 are not required for steady mode.

Static zone proof from `capture-zones-20260904.usbmon`:

```text
AA 00 FF 00 00 ... 00 01   left red
AA 01 00 00 FF ... 01 01   right blue
AA 00 00 00 FF ... 00 01   left blue
AA 01 FF 00 00 ... 01 01   right red
```

An RGB value of `000000` is used by Engine for an unlit steady zone. A separate “off” opcode has not been proven necessary.

## Apply and save reports

Observed 64-byte output reports contain one of these prefixes followed by zeros:

```text
A5 03 0A
A3
AC
09
```

Later saved configurations consistently use `A5 MM 0A`, then `AC`, then `09`. `MM` is an observed zone bitmask:

| Changed zones | Mask |
| --- | ---: |
| Left and right earcups, zones 0 and 1 | `03` |
| Live microphone, zone 2 | `04` |
| Muted microphone, zone 3 | `08` |

The bit relationship is exact across the full capture: `MM = OR(1 << zone)`. Some earlier preview traffic also used `A3`. The verified Linux steady-color utility conservatively sends `A5 03 0A`, `A3`, then commits with `AC` and `09`; captured-animation replay uses the computed mask followed by `AC` and `09`. The broader meanings of `A3`, `AC`, and `09` remain inferred rather than proven.

## Animated payloads

Animated configurations retain the header but use byte 11 value `00` and place coefficient records from byte 12 onward:

```text
AA ZZ RR GG BB FF 32 C8 C8 00 ZZ 00 <coefficients...>
```

The older text-mode sources record only the first 32 data bytes. The 2026-09-05 pcap sources retain every byte and prove that animated reports contain a second parameter block through byte 162; bytes 163–1,023 are zero in the captured configurations.

### Complete Breathe layout

The complete Breathe reports use these fields; unlisted bytes from 28 through 139, 147 through 151, 153 through 155, and 163 onward were zero:

| Offsets | Value or interpretation | Confidence |
| --- | --- | --- |
| 12–14 | Signed negative RGB rate coefficients. | inferred mathematically |
| 15 | Zero record padding. | observed |
| 16–17 | Little-endian duration in 20 ms ticks. | verified against 5, 10, 15, and 25 seconds |
| 18 | First-record ordinal `01`. | observed |
| 19 | Zero record padding. | observed |
| 20–22 | Positive opposites of bytes 12–14. | observed |
| 23 | Zero record padding. | observed |
| 24–25 | Duplicate 20 ms duration. | verified |
| 26 | Second-record ordinal `00`. | observed |
| 27 | Zero record padding. | observed |
| 140–145 | Three little-endian 12-bit channel values, each `8-bit channel << 4`. | verified against entered colors |
| 146 | Constant `FF`. | observed |
| 152 | Connected phase flag: `01` Sweep, `00` Synchronized. | physically verified |
| 156–159 | Constant `01 00 02 00` in captured animations. | observed |
| 160–161 | Little-endian duration in centiseconds. | verified |
| 162 | Reverse flag; `01` appeared for reversed Sweep. | observed, not physically replayed |

The experiment used 5, 15, and 25 seconds rather than categorical slow, medium, and fast values. Their two duration encodings are exact:

| UI duration | Bytes 16–17 and 24–25 | Bytes 160–161 |
| ---: | --- | --- |
| 5 s | `FA 00` = 250 | `F4 01` = 500 |
| 10 s | `F4 01` = 500 | `E8 03` = 1,000 |
| 15 s | `EE 02` = 750 | `DC 05` = 1,500 |
| 25 s | `E2 04` = 1,250 | `C4 09` = 2,500 |

For color `#123456` at 5 seconds, the stored 12-bit channels are `0x120`, `0x340`, and `0x560`. Integer division by 250 gives coefficients 1, 3, and 5, exactly matching the captured negative and positive records. The same relationship holds for `#2468AC` at 10 seconds and the other controlled durations. This is sufficient to describe packet construction, but arbitrary generated packets remain unverified.

### Connected phase and direction behavior

The focused capture proves that Engine can send different animated payloads to each earcup. Animation headers can retain prior RGB values while the nibble-packed color at bytes 140–145 changes to the value entered in GG. For example, focused `#2468AC` reports retained header `FF3C00` but encoded `40 02 80 06 C0 0A` at bytes 140–145. Therefore offsets 2–4 must not yet be treated as the sole animated color.

With color `#2468AC` and duration 10 seconds held constant, Sweep normal and Synchronized packets were identical except at byte 152. Native replays physically showed Sweep alternating between earcups and Synchronized breathing together:

```text
Sweep:        byte 152 = 01
Synchronized: byte 152 = 00
```

Reversing Sweep changed only byte 162 from `00` to `01`. With identical color and duration, GG sent no replacement feature report for reversed Synchronized, Reflected, or reversed Reflected—only `AC` and `09` commits. Their distinct physical semantics, if any on this headset, remain unverified.

### Microphone zones

Two color reversals combined packet evidence with physical mute-button checks:

```text
zone 2 green, zone 3 red -> live green, muted red
zone 2 red, zone 3 green -> live red, muted green
```

Therefore zone 2 is definitively the microphone live/unmuted state and zone 3 the muted state. Zone 2 also accepts ColorShift and Multi Color Breathe reports with the same duration and coefficient structures used by the earcups.

## Known-safe implementation boundary

- Safe and verified: arbitrary steady color, both zones, with zero-filled remainder.
- Safe and verified: byte-for-byte replay of captured Breathe, connected Sweep, and Synchronized reports.
- Safe to implement from already verified static fields: black/off, independent left/right steady colors, and mic live/muted steady colors.
- Structurally understood but not yet directly verified as generated packets: arbitrary Breathe color/duration and connected reverse.
- Captured but not safe to synthesize yet: arbitrary ColorShift, Multi Color Breathe, and Reflected behavior.
- Out of scope without new evidence: firmware operations and unknown opcodes.
