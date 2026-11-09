---
title: GG 122 device serializer findings
type: synthesis
status: mixed
updated: 2026-11-09
sources:
  - docs/gg122-lighting-research.md
  - docs/protocol.md
  - docs/experiments.md
  - docs/raw/capture-effect-presets-20260905.pcapng
  - docs/raw/capture-reflected-timing-controlled-20261021.pcapng
  - src/capture.rs
---

# GG 122 device serializer findings

## Scope and result

The bounded follow-up recovered the original GameDAC device declaration and the shared lighting serializer it includes, from the owner's installed GG 122.0.0 files. The route was offline: native disassembly/decompilation, local resource decryption with integrity verification, and reading the resulting text. Neither Engine nor the stored Lisp program was executed; no hardware requests were sent. This page contains independently written interface findings, not vendor source or decoding secrets.

The recovered program establishes the general forward serializer and an inverse converter for onboard-profile data. It explains the older three-color Breathe timing discrepancy exactly, while leaving the six-color UI/capture mismatch and firmware rendering behavior unresolved. Static program behavior does not extend the physically accepted product boundary.

## Artifact identities

Paths are relative to the installed `apps/engine/` directory. Plaintext identities refer to private derived artifacts, not repository files.

| Artifact | Source bytes | Source SHA-256 | Decoded bytes | Decoded SHA-256 |
| --- | ---: | --- | ---: | --- |
| `deviceSpecifications/arctis-pro-gameDac.edevice` | 70982 | `e65ea0b205f9c42cf8315203e5d0d1125a7011f58f29fbb753b20ff39ceb1a45` | 50947 | `9cb8e0aa73dc5835167ed1d49d4a523973d452cf09d5bc295228d2dd2b2623ab` |
| `deviceSpecifications/fancy_lighting_engine.edevice` | 60793 | `9327f0c27ab9e9a4f1e2d7dbd6d6ef025e41e3d1e63d7def8d50bb1bb3d8809a` | 44051 | `f5d04799d1a47ce32554edf833393ef0039dec66cf17629aaf2a4283c7d03e69` |
| `deviceSpecifications/common_lighting.edevice` | 4369 | `bd0cb996d049547da04d2d20f253066bbb0aff07fa60123893a0972c5e6d4d96` | 2978 | `7a84aa355269af40247a167097c1f78773e92d068e3f50d58c433c753c66e621` |

Engine remains SHA-256 `c1b8e182e62c8bb6667a3c678394c4d6363a532409f3de5a8fd7a31fdf0c0ec1`. The GameDAC armor contains a symmetric-session packet and integrity-protected encrypted data. A private raw carrier has SHA-256 `2e52ca8c214e4e707530d32a0d5133ccf39316248ed48d348d530afa9584297d`. GnuPG 2.4.9 / libgcrypt 1.12.4 reported successful decryption and valid modification-detection checks for all three resources, using an isolated temporary keyring. Integrity verification establishes successful decoding, not publisher authentication.

## Loader-to-program chain

Addresses below apply only to the identified Engine build. REA 6.7.0 used Ghidra 12.1.4; GNU objdump 2.47 and an independent read of the embedded Go function table corroborated relevant code addresses and direct calls. Decompiler parameter recovery has limitations; assembly and successful resource decoding corroborate the important data flow.

| Address | Recovered function | Relevant observation |
| --- | --- | --- |
| `0x140479260` | `support.loadDeviceDeclarations` | Walks the device-definition directory with the encrypted declaration loader, followed by the ordinary loader. |
| `0x140478ea0` | `support.loadEncryptedDeviceDeclaration` | Selects `.edevice` files, calls the decoder, and passes decoded text to the device-support Lisp environment. |
| `0x14046aa00` | `fileencryption.DecryptDeviceFile` | Reads the file, decodes OpenPGP armor, reads the encrypted message using a password callback, then consumes its plaintext body. |
| `0x14046abc0` | `fileencryption.DecryptDeviceFile.func1` | Supplies bytes assembled from embedded constants; those bytes remain private. |
| `0x1404502e0` | `openpgp/armor.Decode` | Decodes the armored carrier. |
| `0x140468dc0` | `openpgp.ReadMessage` | Performs the message-decryption step. |
| `0x140378b60` | `golisp.ParseAndEvalAllInEnvironment` | Receives the decoder's result in the loader. This call was inspected, not invoked. |

The decoded GameDAC declaration binds product identity `1038:1280`, includes `fancy_lighting_engine`, and defines left, right, live-microphone, and muted-microphone lighting APIs with zone constants 0 through 3. All four use the same effect conversion. The shared lighting resource includes `common_lighting`, which defines its input structures and helpers. This explicit include and API chain binds the shared formulas to the GameDAC; the previously readable generic `gradients.lsp` did not provide that link.

The GameDAC feature chunk is declared as 1,025 bytes including the leading HID report ID. Removing that report ID yields the captured 1,024-byte payload. The serializer preserves the 12-byte lighting header, then appends the shared effect conversion. No new HID prefix should be added to the existing native transport.

## Forward serializer findings

- The input structure reserves 14 RGB colors and 14 integer percentage positions. The shared output reserves 16 eight-byte records, followed by the initial RGB and parameter block. These are representation limits; the manual Breathe UI still permits four colors, and hardware acceptance remains separately bounded.
- Each ColorShift marker span produces one record. For a span beginning at `p` and ending at `q`, its raw duration is `max(33, floor(S * (q - p) / 100))`, where `S` is the stored integer speed. The final span ends at 100 and targets the first color.
- A nonzero first marker adds a leading section from 0 to that marker, holding the first color. ColorShift therefore uses one additional record in this case. Breathe has its own leading-section handling. Duplicate positions and a 100% endpoint are not justified product inputs by this finding alone.
- Breathe divides each percentage span by two using integer division **before** multiplying by speed. Each of its fade-out/fade-in records uses `max(33, floor(S * floor((q - p) / 2) / 100))`. Ordinary spans alternate from a selected color toward black and from black toward the next selected color.
- Each signed RGB rate is the integer quotient of `16 * (target - start)` divided by the clamped duration, truncated toward zero, and encoded as a signed byte. The 33 minimum bounds a full-scale rate within the representable range.
- The stored aggregate is the sum of emitted durations, including any minimum-duration or padding effects. It is not necessarily a copy of the requested speed.
- The initial effect RGB is always taken from the first effect color and multiplied by 16. Header RGB belongs to a separate lighting-element initial-color field; it is not the active animation palette's source.
- The fields previously described as phase/direction are part of horizontal/vertical propagation scaling and an inversion value. The GameDAC decorator supplies the horizontal direction used by observed Sweep; support for other shared directions is not established for this headset. Reflected remains a per-earcup palette transformation in the front end, rather than a new wire flag.

The source also contains inverse conversion from effect bytes back into color/position/speed settings for onboard-profile data. It reconstructs colors from quantized rates and durations and positions from integer duration proportions, so round trips can be lossy. This is a static finding, not an implemented or authorized device-readback feature.

## Capture corroboration and remaining counterexamples

For the controlled GG 122 ColorShift layout, all durations exceed 33, so the recovered formula reduces to the already accepted per-span truncation model. RGB 0%/25%/70% gives 300/540/360 at speed 1200 and 308/555/370 at speed 1234, with aggregates 1200 and 1233. Existing full-report production fixtures cover all eight captured reports, including reflected palettes on either earcup.

The older GG 118 Breathe preset at frame 21 has known colors red, `#FF7300`, and `#FF9D00`, positions 0%/33%/66%, and displayed duration 13.5 seconds. Its spans are 33/33/34. Integer half-spans are 16/16/17, giving record durations 216/216/216/216/229/229 and aggregate **1322**. An independently written research test reproduces the entire 1,024-byte frame, including signed rates and the historical header. This corroborates the numerical model against older traffic without proving that GG 118 used the identical program revision.

The six-color frame 37 remains unresolved: its incomplete UI notes report 17.56 seconds and an orange first marker, while the packet contains aggregate 1000 and initial red. The recovered serializer would use the input's first color and computed durations; it does not justify claiming those noted UI inputs produced that packet. Do not rewrite its historical labels or invent the four missing colors.

The recovered Breathe serializer explicitly targets black. Its rate quantization can leave a mathematical integration residual, but no firmware trace establishes whether that explains the observed brightness floor. Exact physical cycle timing and firmware interpolation/clamping remain unknown.

## Product implications and reproduction boundary

The original positioned builder remains available for old profiles and CLI calls. A local opt-in GG timing implementation now encodes the 33-duration clamp, two through fourteen Flow colors, and nonzero-start holds, including reflected palette handling. A separate positioned Pulse builder accepts one through four colors and implements integer half-span arithmetic. It matches all bytes of the old Breathe frame 21; the GG Flow builder also matches all eight controlled GG 122 reports. On 2026-11-09 the owner accepted the sampled native behavior as close enough for task completion, waiving additional controlled GG captures and definitive visual confirmation of Pulse's leading hold. Those source/physical evidence limits remain documented in [the support boundary](compatibility.md#lighting-support). Legacy Breathe and Flow spacing remain unchanged. See [profile compatibility](profile-storage.md#explicit-gg-timing-extensions).

To reproduce the investigation, verify the artifact hashes, open the exact Engine build through the existing REA/Ghidra route, and inspect the loader, decoder, password callback, and their direct calls. Decode only the three named resources into private storage with integrity verification; do not execute their Lisp. Follow the explicit GameDAC include and lighting API chain, then independently evaluate the documented arithmetic against the immutable captures. Keep the decoding inputs, proprietary plaintext, and native decompiler output out of the Git checkout. No firmware, unrelated device definitions, or live device reads are required.

Retained private Evidence identities:

- Declaration walker: `ev_21fc3d55c8e02810545a04b1ee01c47ba9fd5043c1d2806f62f01aea076035bb`.
- Encrypted loader: `ev_7f07d3a51418170481cab72e9eee1618e99fd72b815cbf8e879fb81e252bf9e2`; direct callees: `ev_1af7ca7d88706d024eb04f6bad866e6330fdd5212dc607398f7082b816da7e8a`.
- Decoder: `ev_10241a4211840844466c6067b8a251714534e0cd6a53f7523090892f61813b6a`; direct callees: `ev_14863aa977f864fc808d03be5b09969a69f8ff5daa4f1b38b8cd4d6a9b48a04c`.
- Callback interpretation: `ev_df7676c3be7e6e6f01fc154926eff804a2c55375b3c907abe5fcea208c98794e`; callback assembly: `ev_d3bef1f2d4fd6eeb4154270b4367623efee2782b6065261ecb65f5f99f81aa79`.
