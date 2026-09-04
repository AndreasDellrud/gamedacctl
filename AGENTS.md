# Repository instructions

This repository documents and implements Linux control of an original SteelSeries GameDAC and wired Arctis Pro headset.

- Read `docs/index.md` before protocol work and follow `docs/AGENTS.md` for documentation changes.
- Treat `docs/raw/` USB captures as immutable evidence. Add new captures; never rewrite old ones.
- Distinguish observed packet behavior from inference. Physical lighting behavior requires user verification.
- Do not fuzz opcodes, accept firmware updates, or send unobserved firmware/control packets.
- Never commit Windows credentials, device serial numbers, or downloaded proprietary installers.
- Run `scripts/validate` before committing.
