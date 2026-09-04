# SteelSeries GameDAC Linux control

Reverse-engineering notes and a small Linux utility for the original wired SteelSeries Arctis Pro connected through the original GameDAC.

Current verified result: Linux can set arbitrary steady earcup colors and replay complete captured animations through GameDAC USB control device `1038:1280`, while audio remains on the separate `1038:1282` interface.

```bash
sudo scripts/gamedac-rgb FF00FF
```

The command above was physically verified: both earcups changed to magenta. Root is currently required because the relevant `hidraw` nodes are mode `0600`; a narrow udev rule remains future work.

An exact animation captured from SteelSeries GG can be replayed for research without synthesizing unknown bytes:

```bash
sudo scripts/gamedac-rgb \
  --replay-pcap docs/raw/capture-connected-modes-20260905.pcapng \
  --frames 7 11
```

Frames 7 and 11 are the verified 10-second connected Sweep for zones 1 and 0. Frames 31 and 33 are the matching Synchronized configuration. `wireshark-cli` is required for pcap replay.

Start with [the system overview](docs/overview.md), then use [the documentation index](docs/index.md) for protocol evidence, capture workflow, experiment status, the native application roadmap, and legal/publication considerations.

## Safety

Only packets observed from SteelSeries GG are replayed. Arbitrary animation generation remains under analysis; do not fuzz unknown commands. Do not accept GameDAC firmware updates inside the Windows VM during capture work.
