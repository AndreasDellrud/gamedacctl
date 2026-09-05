---
title: Profile storage and recovery
type: operations
status: current
updated: 2026-09-05
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
