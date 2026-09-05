# gamedacctl

An independent Linux lighting controller for the original wired SteelSeries Arctis Pro connected through the original GameDAC, plus the reverse-engineering evidence behind it.

Current verified hardware result: `gamedacctl` can set independent steady earcup and microphone-state colors, turn selected zones off, and replay complete captured animations through GameDAC USB control device `1038:1280`, while audio remains on the separate `1038:1282` interface.

## Build

The project pins its Rust toolchain through [mise](https://mise.jdx.dev/):

```bash
mise install
mise exec -- cargo build
```

Dry runs construct and validate complete reports without opening the device:

```bash
mise exec -- cargo run -- --dry-run static \
  --left FF3700 --right 0084FF \
  --microphone-live 00FF00 --microphone-muted FF0000

mise exec -- cargo run -- --dry-run off
```

After device permissions are configured, omit `--dry-run` to apply the selected configuration. `off` defaults to the two earcups; use `off --target microphone` or `off --target all` explicitly for the microphone zones.

An exact complete animation captured from SteelSeries GG can be replayed for research without synthesizing unknown bytes:

```bash
mise exec -- cargo run -- --dry-run replay \
  --pcap docs/raw/capture-connected-modes-20260905.pcapng \
  --frames 7 11
```

Frames 7 and 11 are the verified 10-second connected Sweep for zones 1 and 0. Frames 31 and 33 are the matching Synchronized configuration. `wireshark-cli` is required for pcap replay.

The original `scripts/gamedac-rgb` Python utility remains the research reference for comparison with early experiments. The packaged narrow udev rule gives the active desktop user access to only `1038:1280` interface `00`; the other GameDAC control interfaces and the `1038:1282` audio device remain outside its scope.

Start with [the system overview](docs/overview.md), then use [the documentation index](docs/index.md) for protocol evidence, capture workflow, experiment status, the native application roadmap, and legal/publication considerations.

## Safety

Only packets observed from SteelSeries GG are replayed. Generated commands are limited to verified steady fields, known zones, and observed apply/save reports. Arbitrary animation generation remains under analysis; do not fuzz unknown commands. Do not accept GameDAC firmware updates inside the Windows VM during capture work.

## Validation

```bash
scripts/validate
```

This checks documentation, formatting, linting, packet fixtures, CLI validation, and exact hashes for the physically verified captured presets.
