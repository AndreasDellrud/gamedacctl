# Raw capture catalog

These files are immutable GameDAC USB evidence for device `1038:1280`. The SteelSeries GG captures below use GG 118.0.0 in September and GG 122.0.0 in October. Separately labeled native captures record this project's Linux-generated traffic and do not corroborate what GG would send. They contain USB transfer metadata and payload bytes, not Windows credentials or device serial numbers.

## SteelSeries GG traffic

| File | Lines | SHA-256 | Purpose |
| --- | ---: | --- | --- |
| `capture-effects-20260904-2323.usbmon` | 314 | `4f8f5e563c407fb62040d2a71b649038241f8d65bcc1243c48e682ee9f25f9cd` | Broad steady, off, breathe-speed, ColorShift, and multicolor exploration. |
| `capture-zones-20260904.usbmon` | 88 | `87aeb7186b665d05ad5939a0642906594695d360980c9f4e190c754d3d009340` | Focused left/right static mapping, breathe colors, and multicolor direction variants. |
| `capture-full-effects-mic-20260905.pcapng` | 420 packets | `b1dbd7e5afba503d3bacc4645f4cd88aa4476643907a0ad0ad881a0cdba03a06` | Complete 1,024-byte earcup animation, connected-mode, microphone-zone, and microphone-animation reports. |
| `capture-connected-modes-20260905.pcapng` | 66 packets | `61da5dd308203e19e71a7a71a043ff465c60a9eacb5c79c526d3c32de466fa76` | Controlled 10-second `#2468AC` Sweep, Synchronized, reverse, and disabled comparisons. |
| `capture-effect-presets-20260905.pcapng` | 52 packets | `d515f652185dc3969c99baac1a9415ea96b5f2e6ca0fdc900e2fe822a33c789d` | Full-payload steady markers and GG preset examples that distinguish continuous ColorShift records from color-to-black Multi Color Breathe records. |
| `capture-reflected-timing-20261021.pcapng` | 90 packets | `3d327e8210546aae02df5411e36d039499d4b6defb65e5946bfc507fba4ca464` | GG 122.0.0 three-color exploration with uncertain UI mode attribution and owner-reported unintended marker positions. Preserved as confounded evidence, not the planned controlled sequence. |
| `capture-reflected-timing-controlled-20261021.pcapng` | 46 packets | `1feb516c59e9e27789e89bb36a0231422536c7d4ff56c4dc1a35f878d273398f` | Controlled GG 122.0.0 RGB markers at 0%/25%/70%: Synchronized, Reflected, reversed Reflected, baseline restore, and isolated 12.00-to-12.34-second timing change. |

## Native Linux traffic

| File | Packets | SHA-256 | Purpose |
| --- | ---: | --- | --- |
| `capture-native-six-color-flow-20261109.pcapng` | 10 | `f887e57cb2784bf6bafdc99cd151109543085f2f8b564cc65b98c6b016da4acc` | Linux-generated synchronized Flow, six rainbow markers at 0/17/33/50/66/83%, 12.34 seconds. Two complete feature reports and three Apply/finish reports, with successful completions. Demonstrates native transmission, not GG equivalence or physical acceptance. |
| `capture-native-leading-hold-reflected-20261109.pcapng` | 20 | `3fa03ca2b057645a61daf874db2d3800a5e32cb899a0915e27167598a2b589d5` | Linux-generated six-color Flow with red moved to 10%, first Synchronized then Reflected at 12.34 seconds. Four complete feature reports and two Apply/finish sequences, with successful completions. Source-derived native traffic, not GG corroboration. |
| `capture-native-minimum-duration-flow-20261109.pcapng` | 10 | `187a8c58784d51f5a500afea0c4a3de9624a3e07e4535bb06585c7005f632a1c` | Same nonzero-start Reflected Flow with requested duration reduced to 1.00 second. Both reports have seven 33-tick records and aggregate 231. Native traffic; the requested duration is not a measured physical period. |
| `capture-native-positioned-pulse-20261109.pcapng` | 10 | `d6a28aa0db56ef1f5fb93a6d925926302e23ee901f34eef1e40055c8fb47f7ce` | Linux-generated positioned Pulse, red/orange/gold at 0/33/66%, 13.50 seconds. Reports match GG 118 preset frame 21 after substituting only known earcup zone/header fields; aggregate 1322. |
| `capture-native-positioned-pulse-hold-20261109.pcapng` | 10 | `13976636ed699234862143f8c4363b299de8ade17f7e5d21a2d86e5aa96416e9` | Same native Pulse with first red marker moved to 10%; seven records, aggregate 1253. Source-model reconstruction matches both reports. Owner could not confidently distinguish the visual change. |

Do not normalize, reformat, or replace these files. Add a new dated capture when evidence changes.
