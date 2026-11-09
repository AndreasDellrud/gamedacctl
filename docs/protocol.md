---
title: USB lighting protocol
type: architecture
status: mixed
updated: 2026-11-09
sources:
  - src/protocol.rs
  - scripts/gamedac-rgb
  - docs/raw/capture-effects-20260904-2323.usbmon
  - docs/raw/capture-zones-20260904.usbmon
  - docs/raw/capture-full-effects-mic-20260905.pcapng
  - docs/raw/capture-connected-modes-20260905.pcapng
  - docs/raw/capture-effect-presets-20260905.pcapng
  - docs/raw/capture-reflected-timing-controlled-20261021.pcapng
  - src/capture.rs
---

# USB lighting protocol

## Scope and evidence standard

This page covers lighting configuration sent by SteelSeries GG 118.0.0 and the explicitly identified GG 122.0.0 repeat to original GameDAC control device `1038:1280`. “Observed” means present in a raw USB trace. “Verified” additionally means a Linux replay caused the expected physical change. “Inferred” is a working interpretation that still requires a controlled replay. Exact replay of the sampled GG 122 marker/reflection reports now has [native physical acceptance](experiments.md#native-replay-acceptance-of-the-controlled-gg-122-reports); the subsequent [generated-plan acceptance](experiments.md#native-generated-acceptance-of-gg-122-marker-timing) is recorded separately.

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

The 64-byte interrupt input path is safe to observe passively, but it is not currently a firmware-readiness API. Existing GG captures include reports beginning with `0x10` and `0x20`; a settled native three-second observation received none, and a complete manual reconnect observation received none before removal or after the new control node became accessible. Their triggers and semantics remain unknown. The controller does not issue an unobserved `GET_REPORT` or other polling command.

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

The older text-mode sources record only the first 32 data bytes. The 2026-09-05 pcap sources retain every byte and prove that animated reports contain a coefficient-record area followed by a second parameter block through byte 162; bytes 163–1,023 are zero in the captured configurations.

### Coefficient records and color sequences

Animated reports store consecutive eight-byte coefficient records beginning at offset 12. The new preset capture contains examples with 3, 6, and 12 records and establishes this repeated shape:

| Record-relative offsets | Value or interpretation | Confidence |
| --- | --- | --- |
| 0–2 | Signed RGB rate coefficients. | inferred mathematically across captured palettes |
| 3 | Zero padding. | observed |
| 4–5 | Little-endian transition duration in 20 ms ticks. | observed; unit verified by controlled durations |
| 6 | One-based ordinal, except the final record uses `00`. | observed |
| 7 | Zero padding. | observed |

Byte 158 is the observed record count. Bytes 160–161 contain a little-endian aggregate value whose numeric value equals the sum of every record's duration count in all analyzed presets. In controlled one- and two-color effects it also matched the GG duration in centiseconds. The older multicolor presets are counterexamples to treating it as a direct copy of the displayed speed: the three-color Breathe case displayed 13.5 seconds but encoded `1,322`, and the six-marker ColorShift displayed 17.56 seconds but encoded `1,000`. The [recovered GG 122 serializer](gg122-device-serializer.md) now explains the Breathe result: integer half-spans 16/16/17 yield 216/216/216/216/229/229 and sum 1322. An independent test reproduces the entire old frame. The six-marker UI/capture mismatch remains unresolved.

The stored 12-bit RGB value at offsets 140–145 acts as the initial color in the controlled one- and two-color effects, the three-color Breathe preset, and the controlled GG 122 three-color ColorShift. It is not universally authoritative: the six-marker ColorShift packet retained red even though GG reportedly showed a first marker of `#FF9D00`. Its six coefficient records form a continuous RGB loop, while Multi Color Breathe records alternate toward black and toward the next selected color. This structural distinction is observed in complete payloads. The legacy CLI/profile path remains exactly two colors with implicit timing. The low-level structural constructor accepts up to 14 colors for fixture analysis; that is not the public plan boundary or a physical-support claim. The additive positioned-marker path supports two or three colors and reproduces the controlled GG 122 reports below. Generated three-color Synchronized, Reflected, and reversed Reflected plans now have owner physical acceptance; other bounded marker layouts remain generalizations of the observed formula.

### Controlled GG 122 marker timing

The [controlled October capture](experiments.md#controlled-gg-122-marker-reflection-and-fractional-timing-capture) holds red/green/blue markers at 0%/25%/70%. Its forward transition spans are 25%/45%/30%. For the observed inputs, the following model reproduces complete reports:

```text
transition_duration[i] = floor(stored_speed_centiseconds * span_percent[i] / 100)
channel_rate[i] = trunc_toward_zero(16 * (target_channel - start_channel) / transition_duration[i])
aggregate = sum(transition_duration)
```

The 12.00-second baseline produces record durations `[300, 540, 360]` and aggregate `1200`. The isolated 12.34-second change produces `[308, 555, 370]` and aggregate `1233`: rounding is applied independently to each segment, without redistributing the lost unit. The initial color is red, the retained header is `FF3C00`, and the next-record ordinals are `1, 2, 0`. A research-only fixture reconstructs every byte of all eight reports in the controlled capture. Native exact replay of both timing configurations was physically accepted, with synchronized colors and working audio. This establishes the sampled input-to-byte relationship and usable replay; the firmware timing unit, precise physical period, and general marker-layout behavior remain unmeasured.

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
| 156–159 | `01 00 NN 00`, where `NN` is the coefficient-record count; `02` for the verified single-color case. | observed |
| 160–161 | Little-endian duration in centiseconds. | verified |
| 162 | Engine reverse flag; `01` appeared for reversed Sweep. | exact-generation verified; visible direction remains ambiguous |

The experiment used 5, 15, and 25 seconds rather than categorical slow, medium, and fast values. Their two duration encodings are exact:

| UI duration | Bytes 16–17 and 24–25 | Bytes 160–161 |
| ---: | --- | --- |
| 5 s | `FA 00` = 250 | `F4 01` = 500 |
| 10 s | `F4 01` = 500 | `E8 03` = 1,000 |
| 15 s | `EE 02` = 750 | `DC 05` = 1,500 |
| 25 s | `E2 04` = 1,250 | `C4 09` = 2,500 |

For color `#123456` at 5 seconds, the stored 12-bit channels are `0x120`, `0x340`, and `0x560`. Integer division by 250 gives coefficients 1, 3, and 5, exactly matching the captured negative and positive records. The same relationship holds for `#2468AC` at 10 seconds and the other controlled durations. The generated reports reproduce eight Engine fixtures byte-for-byte across Synchronized, Sweep, reverse, 5-second, and 10-second cases. A new `#7A21E6` 10-second Synchronized combination then pulsed both earcups together on hardware. It did not fade completely to black, so no zero-minimum claim is made.

### Connected phase and direction behavior

The focused capture proves that Engine can send different animated payloads to each earcup. Animation headers can retain prior RGB values while the nibble-packed color at bytes 140–145 changes to the value entered in GG. For example, focused `#2468AC` reports retained header `FF3C00` but encoded `40 02 80 06 C0 0A` at bytes 140–145. Offsets 2–4 are therefore retained or initial state, not the authoritative effect color. The normal builder safely sets the header and effect color to the same requested value; that construction was physically verified with the new purple Synchronized case. Fixture tests can supply the historical header separately to reproduce captures exactly.

With color `#2468AC` and duration 10 seconds held constant, Sweep normal and Synchronized packets were identical except at byte 152. Native replays physically showed Sweep alternating between earcups and Synchronized breathing together:

```text
Sweep:        byte 152 = 01
Synchronized: byte 152 = 00
```

“Synchronized” is not a combined earcup feature command. GG sends one 1,024-byte feature report for each zone and then one joint `A5 03 0A` apply. Independent per-earcup animation reports also use phase byte `00`; when both sides changed, GG sent the two zone reports followed by the same joint apply. In frame 305 of the full-effects capture, GG updated only zone 1 but still followed it with `A5 03 0A`, apparently retaining zone 0's previously stored report. No captured field identifies a named profile or links the feature reports beyond the phase value and joint apply. The evidence therefore suggests that synchronized startup is coordinated when firmware applies both stored zone states, but separately committing masks `01` and `02` has not been tested for phase alignment.

Reversing Sweep changed only byte 162 from `00` to `01`. Generated normal and reverse packets both matched their Engine captures byte-for-byte and were sent successfully at five seconds. Normal Sweep was observed beginning left-to-right. The reverse run also appeared left-to-right, but every apply briefly illuminated both earcups and a repeating two-zone alternation has no persistent visual direction; the physical meaning of reversal could not be distinguished reliably. The CLI name therefore means “emit Engine's captured reverse flag,” not a promise about visible startup order. With identical color and duration, GG sent no replacement feature report for reversed Synchronized, Reflected, or reversed Reflected—only `AC` and `09` commits. Their distinct physical semantics, if any on this headset, remain unverified.

[Offline GG 122 inspection](gg122-lighting-research.md) found a GameDAC-specific settings transformation for Reflected: it mirrors marker positions and reverses palette order on one earcup, choosing the other earcup when reverse is selected. Unlike Sweep, it leaves the direction fields disabled. This static finding makes an asymmetric multicolor experiment more useful than the old single-color comparison. It does not establish a new HID field, prove GG 118 used the same transformation, or change the known-safe implementation boundary. The shared editor's centisecond speed representation also leaves the recorded multicolor packet/UI timing discrepancies unresolved.

The controlled GG 122 repeat corroborates that transformation in actual USB traffic and owner observations under GG. With red/green/blue at 0%/25%/70%, Reflected applies red/blue/green at 0%/30%/75% to the right side; reverse moves that mirrored palette to the left. Both cases leave bytes 152 and 162 zero. Returning to Synchronized restores matching reports, while unchanged earcup reports may be omitted. This is distinct from reverse Sweep's byte-162 flag. Native exact replay subsequently produced opposing color orders for Reflected and the mirrored left-earcup order after reversal. Both sampled replay configurations and subsequent generated Reflected/reversed Reflected plans are physically accepted. The generated plans use the first marker color in the retained header rather than copying historical header RGB; coefficient, initial-color, timing, and direction fields match the controlled capture.

### Recovered serializer boundary

The offline [device-definition trace](gg122-device-serializer.md) binds the shared forward serializer to GameDAC's four zone APIs. It confirms 14 input color slots, 16 output records, a per-record minimum duration of 33, nonzero-start hold sections, and Breathe integer half-span quantization before speed scaling. Header RGB is a separate lighting-element initial color; effect initial RGB comes from marker zero. The shared program also has an inverse converter for onboard-profile data, but no new readback command was executed or exposed.

These static findings explain the Breathe 1322 aggregate; static evidence alone does not establish physical behavior. The original positioned builder retains its accepted spacing. An opt-in GG timing builder now implements the minimum-duration clamp, up to 14 Flow markers, and nonzero-start padding; a positioned Pulse builder implements integer half-span timing for up to four colors. After the [native experiments](experiments.md#native-six-color-flow-experiment-2026-11-09), the owner accepted the sampled approximation as close enough for task completion, waiving further controlled GG capture and definitive visual Pulse-hold confirmation. The [support boundary](compatibility.md#lighting-support) preserves the remaining uncertainty. Legacy effects retain their accepted spacing. The six-color UI mismatch, physical cycle period, and general firmware brightness behavior remain open.

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
- Safe and verified: generated single-color Breathe, Synchronized, and Sweep reports for whole-second durations from 1 through 30 seconds; reverse emits the exact Engine-observed flag but its visible direction is ambiguous.
- Safe and verified: generated two-color ColorShift for whole-second durations from 1 through 30 seconds; red-to-blue physically traversed purple rather than black.
- Safe and physically verified: generated one-to-four-color Multi Color Breathe record chaining; a three-color red/green/blue plan displayed that order and faded to black between colors.
- Safe to implement from already verified static fields: black/off, independent left/right steady colors, and mic live/muted steady colors.
- Exact replay and generated plans physically accepted: three-color GG 122 ColorShift at 0%/25%/70%, normal and reversed Reflected at 12.00 seconds, and Synchronized at 12.34 seconds. The precise physical period was not measured. Production builders match all eight controlled reports byte-for-byte when supplied the historical header; ordinary plans initialize that header from the first color.
- Positioned generation is bounded to two or three markers, first at 0%, strictly increasing integer percentages ending below 100%, and exact hundredth-second speeds from 1.00 through 30.00. Signed-byte rate overflow is rejected before transport. These bounds generalize the captured formula; they do not imply physical testing of every palette, marker layout, or duration. Reverse requires Reflected.
- Not established generally: longer ColorShift palettes, nonzero first markers, duplicate/100% endpoints, fractional multicolor Breathe mapping, and physical semantics beyond the captured cases.
- Out of scope without new evidence: firmware operations and unknown opcodes.

The native `gamedacctl` protocol layer encodes this boundary as typed zones and Breathe modes, bounded duration values, exact-size reports, strict color parsing, computed zone masks, and rejection of unsupported reverse/mode combinations or captured reports with unexpected length, prefix, repeated zone, zone range, or mode marker. Its dry-run path performs no HID discovery or writes. Physical acceptance verified the Rust transport with independent steady earcup colors, both microphone states, every off target, exact Synchronized replay, generated Synchronized and Sweep, and restored static rollback while GameDAC audio remained functional.
