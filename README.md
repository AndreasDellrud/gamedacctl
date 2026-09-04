# SteelSeries GameDAC Linux control

Reverse-engineering notes and a small Linux utility for the original wired SteelSeries Arctis Pro connected through the original GameDAC.

Current verified result: Linux can set both headset RGB zones to an arbitrary steady color through GameDAC USB control device `1038:1280`, while audio remains on the separate `1038:1282` interface.

```bash
sudo scripts/gamedac-rgb FF00FF
```

The command above was physically verified: both earcups changed to magenta. Root is currently required because the relevant `hidraw` nodes are mode `0600`; a narrow udev rule remains future work.

Start with [the system overview](docs/overview.md), then use [the documentation index](docs/index.md) for protocol evidence, capture workflow, and experiment status.

## Safety

Only packets observed from SteelSeries GG are replayed. Animation support remains under analysis; do not infer or fuzz unknown commands. Do not accept GameDAC firmware updates inside the Windows VM during capture work.
