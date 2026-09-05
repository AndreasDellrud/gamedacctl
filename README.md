# gamedacctl

An independent Linux lighting controller for the original wired SteelSeries Arctis Pro connected through the original GameDAC, plus the reverse-engineering evidence behind it.

`gamedacctl` is an unofficial community project. It is not affiliated with, endorsed by, or supported by SteelSeries. Product names are used only to describe compatibility.

Current verified hardware result: `gamedacctl` can set independent Steady earcup and microphone-state colors, turn selected zones off, generate two-color ColorShift, one-to-four-color Multi Color Breathe, and connected Sweep effects, and replay complete captured animations through GameDAC USB control device `1038:1280`, while audio remains on the separate `1038:1282` interface.

Physical support is currently limited to the original GameDAC with the original wired Arctis Pro. GameDAC Gen 2, Arctis Nova products, wireless base stations, other SteelSeries USB identities, and multiple simultaneously connected GameDAC units are not supported. See the [compatibility matrix](docs/compatibility.md) for the tested firmware and precise feature boundaries.

## Build

The project pins its Rust toolchain through [mise](https://mise.jdx.dev/):

```bash
mise install
mise exec -- cargo build
```

Launch the native GTK/libadwaita application during development with:

```bash
mise exec -- cargo run --bin gamedacctl-gui
```

The GUI edits the three GG-style illumination types—Steady, ColorShift, and Multi Color Breathe—plus the captured connected Sweep behavior. It saves versioned profiles with optional emoji or glyph icons under the user's XDG configuration directory and can optionally restore the last selected saved profile after reconnect. Existing single-color `breathe` profiles remain compatible. Reconnect restore is disabled by default. Distribution packages should install `packaging/io.github.andreasdellrud.gamedacctl.desktop` with the `gamedacctl-gui` binary.

The CLI also exposes a versioned, machine-readable surface for thin desktop
integrations:

```bash
gamedacctl status --json
gamedacctl profile apply Everyday --json
```

`status` opens the known HID interface without writing and reports `ready`,
`disconnected`, `permission-denied`, or `error` together with saved-profile
summaries and their optional icons. Applying a profile records it as selected only after every HID write
succeeds. The optional Omarchy adapter lives in the separate manifest-root
`omarchy-gamedacctl` project and calls only these commands.

Dry runs construct and validate complete reports without opening the device:

```bash
mise exec -- cargo run -- --dry-run static \
  --left FF3700 --right 0084FF \
  --microphone-live 00FF00 --microphone-muted FF0000

mise exec -- cargo run -- --dry-run off

mise exec -- cargo run -- --dry-run breathe \
  --color 7A21E6 --seconds 10 --mode synchronized

mise exec -- cargo run -- --dry-run breathe \
  --color 2468AC --seconds 5 --mode sweep --reverse

mise exec -- cargo run -- --dry-run color-shift \
  --color FF0000 --color 0000FF --seconds 5

mise exec -- cargo run -- --dry-run multi-color-breathe \
  --color FF0000 --color 00FF00 --color 0000FF --seconds 9
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

For passive reconnect research, `observe-input` prints device accessibility and unsolicited 64-byte HID input reports for a bounded period. It sends no report to the DAC:

```bash
mise exec -- cargo run --bin gamedacctl -- observe-input --seconds 60
```

Start with [the system overview](docs/overview.md), then use [the documentation index](docs/index.md) for protocol evidence, capture workflow, experiment status, the native application roadmap, and legal/publication considerations.

## Safety

Only packets observed from SteelSeries GG are replayed. Generated commands are limited to verified Steady fields and the observed coefficient-record family on known zones. Durations intentionally accept only whole seconds from 1 through 30. ColorShift is limited to the verified two-color form; Multi Color Breathe accepts the manually observed one-to-four-color range. Longer ColorShift palettes, arbitrary GG marker positions, fractional GG speed mapping, and Reflected generation remain disabled. Do not fuzz unknown commands or accept GameDAC firmware updates inside the Windows VM during capture work.

The application reports successful writes, not firmware readback. Use it at your own risk; unsupported hardware is rejected rather than probed.

## Validation

```bash
scripts/validate
```

This checks documentation, udev and desktop packaging, formatting, linting, packet fixtures, CLI validation, profile persistence, and exact hashes for the physically verified captured presets.

## License

The independently written source code, documentation, and packaging are available under your choice of the [MIT License](LICENSE-MIT) or the [Apache License 2.0](LICENSE-APACHE). SteelSeries product names remain the property of their respective owner and are used only to describe compatibility.
