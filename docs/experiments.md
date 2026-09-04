---
title: Capture experiments
type: journal
status: mixed
updated: 2026-09-04
sources:
  - docs/raw/capture-effects-20260904-2323.usbmon
  - docs/raw/capture-zones-20260904.usbmon
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

## Next experiment

Capture full 1,024-byte feature data in pcap form. Then replay one isolated breathe preset per speed, first with identical zones and then with left/right colors swapped. Record the physical result after each replay before exposing stable CLI names.
