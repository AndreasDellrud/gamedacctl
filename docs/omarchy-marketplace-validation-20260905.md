---
title: Omarchy marketplace validation on 2026-09-05
type: evidence
status: current
updated: 2026-09-06
sources: []
---

# Omarchy marketplace validation on 2026-09-05

## Scope

This record binds the marketplace-readiness result to the exact plugin and
validator snapshots that produced it. It is evidence of the marketplace's
documented structural and deterministic static checks, not a general security
audit or a marketplace approval.

| Input | Exact revision |
| --- | --- |
| Plugin repository | `AndreasDellrud/omarchy-gamedacctl` at `dac989636fa485830e6bce3f4e96f324cdd60f93` |
| Marketplace validator | `omacom/omarchy-plugin-marketplace` at `2408c2197dd412bab05ed4108f6106c3af37f894` |
| Quattro shell reference | `omacom/omarchy` branch `quattro` observed at `36e56f4fb463547dd877849bd3bd951410c442e9` |

The reviewed upstream documentation was the stable
[publishing guide](https://plugins.omarchy.org/publish.html), stable
[development guide](https://plugins.omarchy.org/develop.html), and
[Quattro first-party plugin reference](https://github.com/omacom/omarchy/blob/36e56f4fb463547dd877849bd3bd951410c442e9/shell/plugins/README.md).

## Commands

The marketplace repository was cloned at the revision above and installed with
`npm ci`. Authentication was supplied only as a process environment value and
was not written into the repository or captured output.

```bash
GITHUB_TOKEN=<authenticated-token> \
  VALIDATION_METADATA_PATH=/tmp/gamedacctl-plugin-validation.json \
  node scripts/validate-submission.mjs \
  --repo=https://github.com/AndreasDellrud/omarchy-gamedacctl

GITHUB_TOKEN=<authenticated-token> \
  node scripts/security-baseline.mjs \
  --metadata=/tmp/gamedacctl-plugin-validation.json \
  --json=/tmp/gamedacctl-plugin-security.json
```

The development-guide checks were also run from the plugin checkout:

```bash
omarchy plugin validate .
/usr/lib/qt6/bin/qmllint -I /usr/share/omarchy/shell Panel.qml
```

Both commands exited successfully. `qmllint` emitted unresolved static-import
warnings for Omarchy's runtime `qs.Commons` and `qs.Ui` modules; it did not
produce an error exit. Runtime lifecycle testing and the marketplace's Quattro
compatibility check remain the stronger evidence for those shell-owned types.

## Results

Before the plugin fix, the structural validator returned `license-missing`:
the marketplace recognizes a root `LICENSE`, `LICENCE`, or `COPYING` filename,
while the repository contained only `LICENSE-MIT` and `LICENSE-APACHE`.

After protected plugin PR 1 merged, the validator resolved exactly one root
manifest and reported:

```json
{
  "repository": "AndreasDellrud/omarchy-gamedacctl",
  "defaultBranch": "main",
  "commitSha": "dac989636fa485830e6bce3f4e96f324cdd60f93",
  "pluginIds": ["io.github.andreasdellrud.gamedacctl"],
  "entryPoints": ["Panel.qml"],
  "readme": "detected",
  "license": "detected",
  "preview": "detected",
  "quattroCompatibility": "passed"
}
```

The marketplace security baseline recorded:

```json
{
  "schemaVersion": 1,
  "baselineVersion": "3",
  "commitSha": "dac989636fa485830e6bce3f4e96f324cdd60f93",
  "checkedAt": "2026-09-05T21:53:44.746Z",
  "outcome": "passed",
  "enforcementMode": "selective",
  "findings": [],
  "capabilities": [],
  "verifiedPublicationDisposition": "clear"
}
```

The current official Quattro reference lists multiple first-party
`bar-widget` plugins with a direct `Panel.qml` entry point. That evidence,
together with accepted runtime lifecycle tests, supports retaining the
plugin's combined panel and bar-button entry point instead of refactoring it
merely to match the development tutorial's split clock example.

## Submission boundary

The repository is ready for listing review, but no submission was created.
The official workflow requires the owner to confirm code and preview ownership,
external dependencies, safe configuration behavior, and every other checklist
statement before an issue is opened. The appropriate listing metadata is
category `Hardware` with tags `bar` and `quickshell`.
