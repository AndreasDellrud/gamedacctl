# Raw capture catalog

These files are immutable evidence captured from SteelSeries GG 118.0.0 controlling GameDAC USB device `1038:1280`. They contain USB transfer metadata and payload bytes, not Windows credentials or device serial numbers.

| File | Lines | SHA-256 | Purpose |
| --- | ---: | --- | --- |
| `capture-effects-20260904-2323.usbmon` | 314 | `4f8f5e563c407fb62040d2a71b649038241f8d65bcc1243c48e682ee9f25f9cd` | Broad steady, off, breathe-speed, ColorShift, and multicolor exploration. |
| `capture-zones-20260904.usbmon` | 88 | `87aeb7186b665d05ad5939a0642906594695d360980c9f4e190c754d3d009340` | Focused left/right static mapping, breathe colors, and multicolor direction variants. |
| `capture-full-effects-mic-20260905.pcapng` | 420 packets | `b1dbd7e5afba503d3bacc4645f4cd88aa4476643907a0ad0ad881a0cdba03a06` | Complete 1,024-byte earcup animation, connected-mode, microphone-zone, and microphone-animation reports. |
| `capture-connected-modes-20260905.pcapng` | 66 packets | `61da5dd308203e19e71a7a71a043ff465c60a9eacb5c79c526d3c32de466fa76` | Controlled 10-second `#2468AC` Sweep, Synchronized, reverse, and disabled comparisons. |
| `capture-effect-presets-20260905.pcapng` | 52 packets | `d515f652185dc3969c99baac1a9415ea96b5f2e6ca0fdc900e2fe822a33c789d` | Full-payload steady markers and GG preset examples that distinguish continuous ColorShift records from color-to-black Multi Color Breathe records. |

Do not normalize, reformat, or replace these files. Add a new dated capture when evidence changes.
