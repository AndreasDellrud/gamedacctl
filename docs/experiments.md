---
title: Capture experiments
type: journal
status: mixed
updated: 2026-09-05
sources:
  - docs/raw/capture-effects-20260904-2323.usbmon
  - docs/raw/capture-zones-20260904.usbmon
  - docs/raw/capture-full-effects-mic-20260905.pcapng
  - docs/raw/capture-connected-modes-20260905.pcapng
  - docs/raw/capture-effect-presets-20260905.pcapng
  - scripts/gamedac-rgb
---

# Capture experiments

## Initial steady-color discovery

SteelSeries GG was observed while setting steady red, green, and blue. The changing bytes were identified as RGB at feature-payload offsets 2–4. Engine sent configurations for zones 1 and 0, followed by apply/save reports.

The initial temporary trace did not survive a later host reboot. Its result is independently supported by the later zone capture and by the native replay: `sudo ~/.local/bin/gamedac-rgb FF00FF` visibly changed both earcups to magenta.

## Broad effect capture

Source: `raw/capture-effects-20260904-2323.usbmon`.

The requested one-factor-at-a-time order was:

1. Static `#123456`.
2. Off.
3. Static `#123456`.
4–6. Breathe `#123456` at slow, medium, and fast.
7. Breathe `#A1B2C3` at medium.
8–10. ColorShift `#123456` to `#A1B2C3` at slow, medium, and fast.
11. Reverse ColorShift order.
12. Change the secondary ColorShift color.
13. Try three-color ColorShift if supported.
14–15. Separate and swap zone colors.

The user completed most of the sequence, used different colors for a version of step 12, and also made a multicolor-breathe variant. Engine exposed no brightness setting. Because the UI emitted preview traffic and not every planned action was completed literally, the early/late effect labels are provisional. The slow/medium/fast breathe groups are strongly aligned with chronological order but still require native replay verification.

## Focused zone capture

Source: `raw/capture-zones-20260904.usbmon`.

Requested sequence:

1. Static left red, right blue.
2. Static left blue, right red.
3. Medium breathe, both `#123456`.
4. Medium breathe, left `#0084FF`, right `#FF3700`.
5. Swap those breathe colors.

The user additionally performed three multicolor-breathe tests while changing zone directions.

### Definitive observations

| Observation | Evidence |
| --- | --- |
| Zone 0 is left. | First static pair is zone 0 `FF0000`; physical request was left red. |
| Zone 1 is right. | First static pair is zone 1 `0000FF`; physical request was right blue. |
| Mapping reverses correctly. | Second pair is zone 0 blue and zone 1 red. |
| Animation is per-zone. | Later feature pairs have different headers or coefficient blocks for zones 0 and 1. |
| Direction affects coefficients. | Final variant retains zone 0 coefficients while changing zone 1 channel coefficients. |
| No brightness dimension exists in this Engine UI. | User explicitly reported no brightness setting. |

### Remaining ambiguity

- The exact association between the three additional direction actions and their Engine labels was not externally marked in the trace.
- Text-mode usbmon truncates feature data after 32 bytes, so later coefficient records may be missing.
- Engine encoded requested right `#FF3700` as header `FF3C00`; this needs a small direct-color retest if exact animated colors matter.

## Full-payload and microphone capture

Source: `raw/capture-full-effects-mic-20260905.pcapng`.

Binary `usbmon` capture retained every byte of each 1,024-byte feature report with zero packet loss. The earcup batch covered static markers, Breathe at 5, 15, and 25 seconds, different colors, swapped per-side colors, six connected direction/reverse actions, and disabled illumination. The microphone batch covered two static color reversals, physical mute toggles, ColorShift and Multi Color Breathe at 5, 15, and 25 seconds, reversed ColorShift order, and a static end marker.

Physical checks proved zone 2 is live/unmuted and zone 3 is muted. The duration fields and later color encoding are documented in [the protocol page](protocol.md).

## Controlled connected-mode capture and replay

Source: `raw/capture-connected-modes-20260905.pcapng`.

Color `#2468AC` and duration 10 seconds were held constant while testing Sweep, Synchronized, Reflected, and reverse. Sweep normal and reverse differed only at byte 162. Synchronized differed from Sweep at byte 152. GG emitted no new feature payload for reversed Synchronized, Reflected, or reversed Reflected under these controlled inputs; it emitted only commit reports.

Linux then replayed the exact captured reports:

- Frames 7 and 11: user observed alternation between earcups, verifying connected Sweep.
- Frames 31 and 33: user observed both earcups breathing together, verifying Synchronized.
- Frames 175 and 177 from the larger capture: user observed synchronized 5-second Breathe, verifying complete animated replay at a second duration.

## Generated animation acceptance

The Rust builder reproduced eight complete Engine reports byte-for-byte: normal Sweep, reversed Sweep, and Synchronized for both earcups at `#2468AC`/10 seconds, plus both earcups of the `#123456`/5-second Synchronized case with their distinct retained headers.

The new combination `#7A21E6`/10-second Synchronized was then generated rather than replayed. Both earcups pulsed together at the expected pace, but did not fade completely to black. Five-second normal Sweep visibly began left-to-right and alternated. The matching reversed packet was applied repeatedly; it still appeared to begin left-to-right, but a simultaneous apply flash obscured startup and a repeating two-zone alternation makes direction intrinsically difficult to distinguish. The static orange-left/blue-right and green-live/red-muted configuration was restored afterward.

This accepts generated single-color Breathe and Sweep while retaining two explicit unknowns: the waveform's nonzero brightness floor and the visible meaning of Engine's reverse flag. Reflected, ColorShift, and multicolor synthesis remain disabled.

## Effect-preset sequence capture

Source: `raw/capture-effect-presets-20260905.pcapng`.

A 155.6-second, 52-packet full-snap-length capture was filtered immediately to GameDAC address 69 and USB identity `1038:1280`; the temporary whole-bus source was then deleted. The intended action order was not followed literally, so distinctive microphone Steady markers and packet structure, rather than chronology alone, delimit the results:

- Frame 7: microphone-live Steady `#010203` start marker. Applying it also re-emitted retained earcup animations: frame 9 contains 12 paired rainbow records on zone 0, and frame 11 contains three continuous transition records on zone 1.
- Frame 21: six microphone-live records alternate toward black and toward the next color, structurally identifying a three-color Multi Color Breathe preset. GG showed markers at 0% `#FF0000`, 33% `#FF7300`, and 66% `#FF9D00`, with a displayed speed of 13.5 seconds; the packet's aggregate field is `1,322`.
- Frame 29: microphone-live Steady `#040506` separator.
- Frame 37: six continuous microphone-live transitions form a color loop, structurally identifying a ColorShift preset. GG showed a speed of 17.56 seconds and six markers at 0%, 17%, 34%, 51%, 68%, and 85%, beginning at `#FF9D00` and ending at `#FF00FF`; the four intermediate colors were not recorded. The packet retained red at offsets 140–145 and encoded aggregate value `1,000`, so arbitrary multi-marker color and speed mapping is not established.
- Frame 45: microphone-live Steady `#070809` end marker.

The six-color retained rainbow in frame 9 fades each color to black and reverses palette order relative to the matching retained zone-1 report in the earlier full-effects capture. This strongly identifies those 12-record reports as a rainbow Multi Color Breathe configuration. The user also clarified that GG permits up to four manual color selections for Multi Color Breathe and up to 14 for ColorShift; presets may emit a longer built-in rainbow than the manual Multi Color Breathe selector. Exact replay is still required before the structural effect labels become physically verified. The initial product scope therefore limits ColorShift to its already correlated two-color form rather than claiming 14-color parity.

## Generated named-effect acceptance

After the Windows VM stopped and Linux `usbhid` regained interface 0, a generated five-second two-color ColorShift used bright red and blue. The user observed a continuous transition through purple, distinguishing it from a fade through black. A generated nine-second Multi Color Breathe then used red, green, and blue; the user observed that exact order and confirmed each color breathed down to black before the next appeared. The accepted Steady orange-left/blue-right and green-live/red-muted configuration was restored. PipeWire still exposed GameDAC Game 5.1 as the default sink at 60 percent, plus GameDAC Chat and microphone.

The rebuilt GTK application then saved `Shift Test` as a two-color ColorShift profile and `Breathe Test` as a synchronized three-color Multi Color Breathe profile. The user applied both and switched between them through the saved-profile selector; the fields reloaded correctly. The persisted version-1 JSON and `status --json` output retained the ordered color arrays and reported `color-shift` and `multi-color-breathe` respectively.

After installing the final binaries and synchronizing the enabled Omarchy adapter, the user applied both profiles again from the panel and confirmed that each worked. The transport intentionally retains GG's observed 60 ms spacing between the two zone reports and the joint apply, which can produce a momentary startup mismatch; the continuing effects remained synchronized.
