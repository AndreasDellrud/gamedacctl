---
title: GG 122 offline lighting investigation
type: synthesis
status: mixed
updated: 2026-10-21
sources:
  - docs/protocol.md
  - docs/experiments.md
  - docs/research-process.md
  - docs/raw/capture-connected-modes-20260905.pcapng
  - docs/raw/capture-effect-presets-20260905.pcapng
  - docs/raw/capture-reflected-timing-controlled-20261021.pcapng
  - src/capture.rs
---

# GG 122 offline lighting investigation

## Scope and provenance

This investigation studies the owner's installed SteelSeries GG 122.0.0 to explain gaps in the original GameDAC lighting protocol. It does not expand the controller's supported packet generation. The USB baseline remains GG 118.0.0; static GG 122 findings are not evidence that either version emitted a particular packet or that hardware accepted it.

The subsequent [controlled GG 122 capture](experiments.md#controlled-gg-122-marker-reflection-and-fractional-timing-capture) corroborated the palette-reflection path and established an exact three-marker timing reconstruction, including independent segment truncation at 12.34 seconds. All eight complete captured reports now have a byte-for-byte research regression fixture. [Native exact replay](experiments.md#native-replay-acceptance-of-the-controlled-gg-122-reports) then received physical acceptance for synchronized lighting, both Reflected orientations, and the captured fractional-speed configuration, with working audio. Production generation and precise physical timing measurements remain separate.

The installation is under `~/.wine/drive_c/Program Files/SteelSeries/GG`. The installer, extracted JavaScript, decompiler output, and REA Evidence remain outside tracked source. Only independently written findings and artifact identities belong here. No GG process, installer, firmware operation, or HID command was run during the offline phase. The later capture phase used the owner's Windows GG installation and normal lighting controls, as recorded in the experiment journal.

| Local artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `SteelSeriesGG122.0.0Setup.exe` | 433403168 | `f7708ea437b431ff1941782d9840985e6c2efdc6b9e61a65fc28c019011b6345` |
| `resources/app.asar` | 117827704 | `9b9d2a5f8675dbf3e8bc674838ccbb804f94d0282310a7e73a280e743190da06` |
| `apps/engine/SteelSeriesEngine.exe` | 16673336 | `c1b8e182e62c8bb6667a3c678394c4d6363a532409f3de5a8fd7a31fdf0c0ec1` |
| `apps/engine/SSEdevice.dll` | 823352 | `50284715f971c6daea99bbe714caad9de329a681eb8f54915081777c4ce1264a` |
| `apps/engine/deviceSpecifications/arctis-pro-gameDac.edevice` | 70982 | `e65ea0b205f9c42cf8315203e5d0d1125a7011f58f29fbb753b20ff39ceb1a45` |
| `apps/engine/lisp/gradients.lsp` | 10314 | `c096cfa753236246b5f27e2c466418ecd0b7fd89a4af48324e0696ab3f081323` |

The version manifest reports `122.0.0`. In the initial pass the descriptor was an opaque armored resource and was not decrypted; the later [device serializer investigation](gg122-device-serializer.md) recovered it and its lighting includes. The readable Lisp file implements generic color sequences, but its filename and contents alone do not establish a connection to the GameDAC HID serializer.

## Static entry points

REA 6.7.0 inspected the ASAR and parsed selected JavaScript without evaluating its bootstrap or module factories. A separate Babel AST read located exact webpack factories. Module numbers and offsets below apply only to this artifact version. Source ranges are zero-based UTF-16 offsets, with an exclusive end, in the original minified text.

| Source | Locator | Role |
| --- | --- | --- |
| `render/166.js` | module `99244` | Configuration lookup maps `arctis_pro_plus_gamedac/configuration` to module `11662`. |
| `render/166.js` | module `11662`, `[137805, 150901)` | GameDAC configuration decorator and per-earcup connected-mode transformation. |
| `render/166.js` | module `25763`, `[370105, 379086)` | Shared lighting constants, limits, and legacy gradient conversion. |
| `render/363.js` | module `48386`, `[265732, 292894)` | Custom lighting editor, marker positions, speed, and named effect selection. |
| `render/index.js` | module `55489`, `[2178868, 2184531)` | Effect identities: ColorShift `1`, Breathe `2`; these are settings identities, not established HID opcodes. |
| `render/deviceConfig.html` | multi-zone button bindings | Sweep selects state `1`, Synchronized `2`, Reflected `3`. |
| `render/templates/arctis_pro_plus_gamedac/settings.html` | lighting model bindings | Connects the GameDAC page to left, right, combined, live-mic, and muted-mic models. |

`render/166.js` has SHA-256 `332006380a03f9ef332999aaecd23d5c6ab6c9feb4ae76b47f5a940763395d10`; `render/363.js` has SHA-256 `3ee4e184361d4e4e33b5f898c4ba8c5cc400de89fa4ffde80a50af6a2f65e13b`; `render/index.js` has SHA-256 `cc8f8249bfbfa0454d20861ccf028d951921c4480f1f15496a28373bbb2c0f80`.

## Connected-mode findings

These are direct observations of static GG 122 code paths, not hardware verification:

- The GameDAC decorator clones combined lighting into separate left and right settings. Both initially have direction disabled and inversion cleared.
- Sweep enables direction on both sides. Its reverse option toggles inversion on both sides.
- Synchronized leaves both cloned palettes and their direction fields unchanged.
- Reflected changes one palette: normally the right side, or the left side when reverse is selected. It reverses marker order and maps each position `p` to `100 - p`. It does not set the Sweep direction flag.
- To handle the wraparound boundary, a palette beginning at 0% but not ending at 100% moves its first color to 100% before reflection. The converse boundary case moves the last color to 0%. It preserves the number of selected markers.
- The resulting colors, positions, speed, color count, and direction settings are written to each earcup's settings model. This path does not itself construct a 1,024-byte HID report.

For example, a combined palette of red at 0%, green at 25%, and blue at 70% gives the normally reflected right side red at 0%, blue at 30%, and green at 75%. This is a derived example of the static transformation, not a tested device setting.

The single-color GG 118 Reflected experiment was therefore weakly discriminating: reversing a one-color cycle can leave its visible sequence unchanged. This provides a plausible explanation for the absence of replacement feature reports in that capture, but does not prove the same implementation existed in GG 118 or establish firmware behavior. That initial static finding alone did not establish product support; subsequent capture and generated-plan acceptance are recorded below.

## Timing, marker, and limit findings

The shared editor uses percent marker positions and stores speed as an integer representing hundredths of a second: the displayed value divides stored speed by 100; editing seconds multiplies by 100 and rounds. The available speed range is 33 through 3,000 stored units, with the minimum additionally constrained by approximately three color stops per second. Breathe counts two transitions per selected color. These are editor constraints, not evidence of firmware capacity or safe coefficient limits.

The manual editor constants allow 14 ColorShift colors and four Breathe colors, agreeing with the prior owner observations. They do not prove that 14 arbitrary markers fit the GameDAC's coefficient representation.

The GameDAC model forwards marker positions and stored speed without converting them to the eight-byte transition records described in [the protocol](protocol.md). The shared legacy preset converter translates a different gradient representation into this editor format; that conversion is not the GameDAC wire serializer. Later controlled USB evidence independently established the conversion for the sampled three-color layout: each duration is the stored centisecond speed times its percent span, truncated separately, and the aggregate is the sum of those durations. Thus 12.34 seconds with 25%/45%/30% spans encodes 308/555/370 and aggregate 1,233.

## Comparison with immutable GG 118 captures

Re-reading all three binary captures yielded 63 complete `AA` lighting feature reports, including 45 animated reports. Their original checksums remain those in [the raw catalog](raw/README.md).

| Capture and frame | Existing packet observation | What static GG 122 analysis establishes |
| --- | --- | --- |
| Connected modes, controlled cases | Sweep and Synchronized differ at phase byte 152; reverse Sweep changes byte 162. | The device decorator has matching direction/inversion distinctions, but the final settings-to-byte mapping still needs tracing. |
| Presets, frame 21 | Six records, aggregate `1322`, with the recorded UI speed 13.5 seconds. | UI speed has centisecond precision. This does not explain the `1350` versus `1322` discrepancy. |
| Presets, frame 37 | Six records, aggregate `1000`, with the recorded UI speed 17.56 seconds and incomplete palette notes. | The editor can represent `1756`; this does not prove that value was deployed or explain why this packet contains `1000`. |

Static editor findings must not be used to rewrite the historical capture labels or replace the counterexamples with an assumed timing formula. The controlled GG 122 repeat now correlates one marker layout and fractional speed with complete packets, but the older preset discrepancies, general marker handling, and ambiguous initial-color behavior remain unresolved.

## Boundary of the initial native trace

REA's Ghidra 12.1.4 provider completed default analysis of the installed Engine executable using an isolated Temurin JDK 21.0.12.1+1. It recovered the generic serialization path below. These are decompiler interpretations with recovered Go names/types, corroborated by the chunk serializer's assembly and resolved call inventory; they are not original Go source or observed execution.

| Engine address | Function in `pkg/device/support` | Recovered role |
| --- | --- | --- |
| `0x14047d3e0` | `(*DeviceDeclaration).SerializeToBytes` | Selects a named API, binds settings, evaluates API-local bindings, and serializes its write commands. |
| `0x14048c020` | `(*ApiCommand).SerializePayload` | Iterates chunk groups and any interpreted loop counts, then calls the chunk serializer. |
| `0x14048b780` | `(*ApiChunk).Serialize` | Evaluates the chunk's stored Lisp `Code` in its environment, checks that the result is a byte array of bounded size, then packages/pads it. |
| `0x1404a75a0` | `(*service).defSerializeApiToByteArray` | Exposes a generic device-PID/JSON serialization primitive that delegates to the device declaration. This primitive was inspected, not invoked. |

At `0x14048b9e0`, the chunk serializer directly calls `0x140370680`, identified as `github.com/steelseries/golisp.evalHelper`. That direct edge is present in both the call inventory and disassembly. The recovered `self->Code` interpretation comes from pseudocode and recovered type information. The serializer's internal type/size envelope is not an additional GameDAC HID header and must not be added to the `AA` packet layout.

This explains why searching the native function inventory alone did not yield a named GameDAC gradient compiler: the generic serializer delegates byte construction to interpreted code. A bounded name search for gradient, fancy, lighting, color-stop, and breathe terms found framework/database/event functions, but did not establish the GameDAC-specific coefficient builder. That search does not prove no such logic exists elsewhere.

The trace therefore reaches the generic byte-generation mechanism but does not bind a particular stored Lisp program to the GameDAC lighting API. The armored GameDAC descriptor remains opaque in this pass. Neither the readable generic `gradients.lsp` nor the native serializer supplies a proven replacement for the missing GameDAC-specific formula. The subsequent controlled capture supplied a bounded input-to-byte reconstruction independently; continuing into unrelated native code or firmware is not justified by these findings.

Retained native Evidence includes `ev_69d74864e26d6f4b65d1c68de8bb4bad4e4e34f9af8c20eb74d6ca6e891181c6` for device serialization, `ev_ff114d2b5ce84728f7111bfe9df3294b631d5a44df27eaad2f2516dae66a448a` for command serialization, `ev_0caf88af87cb678d88329e522193cb9afa5069ce5e7eb9e3905257e13ec4b928` for chunk pseudocode, and `ev_37bfa433ff542e8f962c3a2a4ee4002292e3d2e49365c70f85d3a556863ec1b0` for chunk assembly. Ghidra reported some missing parameter/return signatures and unreachable-block warnings; these limit detailed pseudocode interpretation. No native runtime was executed. The ephemeral analysis database was closed after retaining the relevant private results.

## Reproducing the static inspection

The local investigation uses `~/.cache/gamedacctl-research/gg122` for private output. REA can be run through a version-pinned package runner without installing a global skill or changing an MCP registration:

```bash
npx --yes rea-agents@6.7.0 inspect-artifact \
  "$GG_ROOT/resources/app.asar" --format json
npx --yes rea-agents@6.7.0 analyze-javascript-application \
  "$GG_ROOT/resources/app.asar" --source-path render/166.js \
  --max-semantic-nodes 10000 --max-semantic-unknowns 2000 \
  --analysis-timeout-ms 60000 --format json
```

Here `GG_ROOT` is the installed GG directory, not the installer. Retain complete output locally and inspect its coverage limits. The ASAR inventory Evidence ID is `ev_54f250dde614fe2845fd3e4e63861084e728d3dd0ee1246094ee640474b2ca78`; the selected GameDAC-containing source analysis is `ev_02b18502ef257e0afd5d8e3e4401116948ff9139ef9908fa1d654537be014478`. These identities refer to the retained investigation records; fresh runs may have different envelopes.

Full REA extraction failed because the archive declares an unpacked `native_modules/win32/ia32/audioMeter.node` that is absent from the installed companion directory. No replacement file was invented and integrity enforcement was not relaxed. Selected embedded JavaScript analysis succeeded; relationships through unselected sources remain unknown. Locally retained module text was read only from embedded ASAR entries and was never executed.

The native tools are isolated under `~/.local/share/gamedacctl-research`; no shell profile, global package, Codex skill, or MCP registration was changed. The Ghidra release archive was verified against publisher SHA-256 `ddac49f903da9d5bac833e5cc79395098b9c33cfd3279be5f31bd00387d2d4db`, and the Temurin JDK archive against `ce79869e1307ed8ee1e2baa86a412b1eb5b75d10a01006d788a6f968bcfaee94`. Native queries used a temporary local MCP session to reuse one Ghidra import. For a standalone CLI query, the equivalent is:

```bash
GHIDRA_INSTALL_DIR="$HOME/.local/share/gamedacctl-research/ghidra_12.1.4_PUBLIC" \
JAVA_HOME="$HOME/.local/share/gamedacctl-research/jdk-21.0.12.1+1" \
REA_GHIDRA_STARTUP_TIMEOUT_MS=600000 \
npx --yes rea-agents@6.7.0 function \
  "$GG_ROOT/apps/engine/SteelSeriesEngine.exe" 0x14048b780 \
  --provider ghidra --format json
```

Keep native output private. A new CLI process imports/analyzes the image again; preserve the retained results rather than repeating queries just to view them.

## Discriminating hardware evidence

The next useful controlled sequence uses three distinct colors and unequal positions, with live preview disabled and one Apply per case:

| Case | Combined earcup settings | Dimension isolated |
| --- | --- | --- |
| A | ColorShift red/green/blue at 0%/25%/70%, 12 seconds, Synchronized | Asymmetric baseline. |
| B | Same palette, positions, and speed, Reflected | One-side palette reflection. |
| C | Same as B, reverse enabled | Which earcup receives the reflected palette. |
| D | Same as A, speed 12.34 seconds | Fractional timing alone. |

Use distinctive static start/end markers and record every selected color, position, speed, and mode. Follow [the capture workflow](capture-workflow.md), preserve the new filtered capture, and compare full feature reports before proposing any native generated packet. Existing fixtures must still match byte-for-byte, and physical acceptance remains required before exposing additional product settings.

An [initial attempt](experiments.md#gg-122-three-color-exploration-with-confounded-ui-settings) produced useful palette and timing differences, but the owner reported unintended 0%/50%/75% markers and uncertainty between the earcup-link control and Synchronized mode. It does not establish that the planned sequence was performed. The [controlled repeat](experiments.md#controlled-gg-122-marker-reflection-and-fractional-timing-capture) then followed the intended values with linked earcups throughout, exact displayed-value confirmation, and one Apply at a time. It also restored baseline A before changing only the duration for D.

The native production integration subsequently reproduced all eight controlled reports and passed [generated-plan acceptance](experiments.md#native-generated-acceptance-of-gg-122-marker-timing) for normal/reversed Reflected and fractional Synchronized. The additive marker profile/CLI path and opt-in GUI controls preserve legacy two-color implicit timing; unsupported marker layouts and excessive transition rates are rejected. Static findings above remain separately labeled from this later packet and physical evidence.

The later [device serializer follow-up](gg122-device-serializer.md) crossed this initial boundary: it recovered the encrypted GameDAC declaration and its shared lighting includes, established the general forward serializer, and reproduced the older 1322-duration Breathe packet from the recorded 13.5-second inputs. The six-color UI mismatch and firmware rendering remain unresolved. No proprietary resource or decryption input was added to the repository.
