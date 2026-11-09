---
title: Profile storage and recovery
type: operations
status: current
updated: 2026-11-09
sources:
  - src/profile.rs
  - src/main.rs
  - src/bin/gamedacctl-gui.rs
---

# Profile storage and recovery

## Location and ownership

`gamedacctl` stores named lighting profiles, the last selected profile, the master lighting state, and the opt-in reconnect policy in:

```text
$XDG_CONFIG_HOME/gamedacctl/profiles.json
```

When `XDG_CONFIG_HOME` is unset or not an absolute path, the Linux default is `~/.config/gamedacctl/profiles.json`. Profiles are application configuration, not cache: they survive logout, reboot, application upgrades, and package removal. The package manager never creates, edits, or removes this user-owned file.

The current schema remains version 1. The additive `lighting_enabled` store field defaults to `true` when absent, so existing files require no migration. It is independent of the selected profile: turning lighting off sends verified steady black to all four zones while retaining the profile that will be restored when lighting is enabled again. Explicitly applying a profile enables lighting. A successful write makes the application directory user-only (`0700`) and creates the profile and lock files user-readable and user-writable only (`0600`). An existing broader mode is tightened on the next successful application write; read-only commands do not silently mutate permissions.

## Positioned Color Flow compatibility

Existing `color-shift` profiles retain their `colors` and whole-second `seconds` fields and implicit transition spacing. Loading or applying them does not migrate them to marker timing. The GUI's opt-in Marker positions setting saves a separate additive `color-shift-markers` effect under schema version 1:

```json
{
  "effect": "color-shift-markers",
  "markers": [
    {"color": "#FF0000", "percent": 0},
    {"color": "#00FF00", "percent": 25},
    {"color": "#0000FF", "percent": 70}
  ],
  "centiseconds": 1234,
  "mode": "reflected",
  "reverse": false
}
```

Two or three markers are accepted, beginning at 0% and increasing strictly below 100%. Duration is stored as integer hundredths, from 100 through 3000. Modes are `synchronized` and `reflected`; reverse requires Reflected. Invalid layouts or rates fail before device writes. Older application versions cannot decode this new effect and will report unsupported data; retain a backup before downgrading after saving marker profiles. Merely installing this version does not rewrite existing profiles.

### Explicit GG timing extensions

The local GG timing implementation received owner acceptance as a bounded approximation after native testing; the [support boundary](compatibility.md#lighting-support) retains the evidence limits. It adds an optional `gg122` boolean to `color-shift-markers`. An absent or false value retains the previously accepted two/three-marker zero-start arithmetic and is omitted when saving, so old profiles do not silently change. Setting it to true opts into two through fourteen markers, a possible nonzero start, and the recovered minimum duration of 33 per transition. Percentages must still increase strictly and stay below 100. Existing Synchronized/Reflected and reverse rules are retained.

Positioned Color Pulse uses a separate additive `breathe-markers` effect with `markers` and integer `centiseconds` fields. It accepts one through four increasing percentages below 100 and uses the recovered integer half-span arithmetic, leading holds, and minimum duration. It is synchronized only. Existing `breathe` and `multi-color-breathe` profiles retain their old spacing and mode handling.

The GUI exposes Marker positions for both effects and a separate GG timing switch for positioned Flow. Pulse marker timing always uses the recovered rules. The duration range remains 1.00–30.00 seconds; the sum encoded in the packet can differ because of integer rounding and the minimum-duration clamp. Older application versions fail closed on the new Pulse effect; do not downgrade without a profile backup.

## Consistency and durability

All application mutations cooperate through `profiles.lock` in the same directory. A writer takes an exclusive advisory lock, reloads and validates the latest on-disk store, applies only its requested change, and then validates the complete result. This prevents the CLI and GUI from independently saving stale whole-file snapshots and silently dropping profiles.

The replacement file is created with a unique name in the same directory and mode `0600`. The application writes the complete JSON document and newline, synchronizes that file, atomically renames it over `profiles.json`, and synchronizes the parent directory. Readers therefore observe either the previous complete file or the new complete file, not a partially written document. Temporary files are removed automatically when a write fails before the rename.

The lock is advisory: external editors and scripts must either avoid writing while `gamedacctl` is active or implement the same locking and atomic-replacement contract. The lock file is intentionally persistent and contains no profile data.

## Invalid or unsupported data

Missing storage is treated as a new empty profile collection. Invalid JSON, an unsupported schema version, invalid effect values, duplicate names, and a missing selected-profile reference are errors. The CLI reports the error. The GUI can still show its safe default editor and device state, but any attempted mutation reloads the on-disk file and fails; it does not replace malformed or unsupported data with an empty store.

There is no automatic repair because guessing could discard the only copy of a user profile. Preserve the file, inspect the reported error, and restore a known-good copy or correct a copy before replacing the original.

## Backup and restore

Close the GUI before a planned backup or restore. The Omarchy panel only reads the store except when it invokes the controller to select a profile or change the master lighting state.

Back up the file while preserving its mode:

```bash
install -Dm600 "${XDG_CONFIG_HOME:-$HOME/.config}/gamedacctl/profiles.json" \
  "$HOME/gamedacctl-profiles.$(date +%Y%m%d-%H%M%S).json"
```

To restore, first move the current file to a separate recovery name, then install the known-good copy with mode `0600`. Launch `gamedacctl status --json` before opening the GUI to confirm that the restored schema and profiles validate. Do not delete the recovery copy until the expected profiles appear.

## Removal

Removing the Arch package leaves the profile store in place for later reinstall. If permanent removal is desired, uninstall the package first, verify that no `gamedacctl` or Omarchy integration process is using the profiles, and then remove only the resolved `gamedacctl` application directory under `XDG_CONFIG_HOME`. Profile deletion is irreversible unless a backup exists.
