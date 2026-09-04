---
title: USB lighting protocol
type: architecture
status: mixed
updated: 2026-09-04
sources:
  - scripts/gamedac-rgb
  - docs/raw/capture-effects-20260904-2323.usbmon
  - docs/raw/capture-zones-20260904.usbmon
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

Later saved configurations consistently use `A5 03 0A`, then `AC`, then `09`. Some earlier preview traffic also used `A3`. The verified Linux steady-color utility conservatively sends `A5 03 0A`, `A3`, then commits with `AC` and `09`; this sequence worked physically. The individual semantics of `A5`, `A3`, `AC`, and `09` remain inferred rather than proven.

## Animated payloads

Animated configurations retain the header but use byte 11 value `00` and place coefficient records from byte 12 onward:

```text
AA ZZ RR GG BB FF 32 C8 C8 00 ZZ 00 <coefficients...>
```

The text-mode `usbmon` source records only the first 32 data bytes even though the transfer length is 1,024. Therefore animation replay must not assume all later bytes are zero. A full-payload pcap capture or a safe controlled zero-fill replay is required before animation support is called verified.

### Breathe speed candidates

The planned experiment changed only breathe speed in slow, medium, fast order. These first-20-byte coefficient blocks appeared in the same order, so the labels are sequence-derived and need replay verification:

| Candidate | Captured coefficient prefix |
| --- | --- |
| Slow | `FF FD FB 00 FA 00 01 00 01 03 05 00 FA 00 00 00` |
| Medium | `00 FF FF 00 EE 02 01 00 00 01 01 00 EE 02 00 00` |
| Fast | `00 00 FF 00 E2 04 01 00 00 00 01 00 E2 04 00 00` |

The repeated little-endian values `00FA`, `02EE`, and `04E2` are plausible timing fields, but that interpretation is not yet verified.

### Per-zone and direction behavior

The focused capture proves that Engine can send different animated payloads to each earcup. A requested left `#0084FF` appeared verbatim in zone 0. A requested right `#FF3700` appeared as header `FF3C00` in zone 1; the reason for this discrepancy is unknown.

The final multicolor direction tests included this common prefix for both zones:

```text
19 00 00 00 A3 00 01 00 EA 00 00 00 B8 00 02 00 19 19 00 00
```

One direction variant changed only zone 1 to:

```text
19 00 19 00 A3 00 01 00 EA 00 EA 00 B8 00 02 00 00 00 19 00
```

This proves independent per-zone direction coefficients, but not yet which coefficient arrangement corresponds to each Engine direction label.

## Known-safe implementation boundary

- Safe and verified: arbitrary steady color, both zones, with zero-filled remainder.
- Safe to implement after direct replay verification: black/off and independent left/right steady colors.
- Captured but not safe to synthesize yet: arbitrary breathe, ColorShift, multicolor breathe, and direction.
- Out of scope without new evidence: firmware operations and unknown opcodes.
