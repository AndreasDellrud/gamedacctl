---
title: Scoped GameDAC device access
type: operations
status: target
updated: 2026-09-05
sources:
  - packaging/udev/70-gamedacctl.rules
  - src/transport.rs
---

# Scoped GameDAC device access

## Boundary

`gamedacctl` needs write access to one hidraw node: SteelSeries vendor `1038`, original GameDAC control product `1280`, USB interface `00`. It must not receive access to the GameDAC audio product `1282`, control interfaces `01` or `02`, or unrelated SteelSeries devices.

The packaged rule applies systemd's `uaccess` tag instead of granting a permanent group or world-writable mode:

```udev
SUBSYSTEM=="hidraw", KERNEL=="hidraw*", ATTRS{idVendor}=="1038", ATTRS{idProduct}=="1280", ENV{ID_USB_INTERFACE_NUM}=="00", TAG+="uaccess"
```

On the development machine, read-only inspection resolved that interface to `/dev/hidraw16` at the time of inspection. The kernel number is not stable and must never be placed in a rule or script.

## Install

The rule is not installed automatically from a source checkout. After reviewing it, package installation should place it at `/usr/lib/udev/rules.d/70-gamedacctl.rules`. For a temporary development installation, copy it to `/etc/udev/rules.d/70-gamedacctl.rules`, reload rules, and physically reconnect the GameDAC. Installation and rule reload require administrator authorization.

Do not detach or rebind the GameDAC while audio is in use. Because this particular unit has a mechanically unreliable USB connector, prefer one deliberate reconnect with the DAC supported and stationary.

## Verify

After reconnecting, identify the current node from properties rather than guessing its number:

```bash
for node in /sys/class/hidraw/hidraw*; do
  udevadm info --query=property --path="$node" \
    | grep -E '^(DEVNAME|ID_VENDOR_ID|ID_MODEL_ID|ID_USB_INTERFACE_NUM|CURRENT_TAGS)='
done
```

The intended node must show `ID_VENDOR_ID=1038`, `ID_MODEL_ID=1280`, `ID_USB_INTERFACE_NUM=00`, and `uaccess` in `CURRENT_TAGS`. Interfaces `01`, `02`, and product `1282` must not gain access from this rule. Then run a dry-run command before one physically observed static-color test and confirm that Linux audio is still on product `1038:1282`.

## Rollback

Remove only the installed `70-gamedacctl.rules`, reload udev rules, and reconnect the GameDAC. The source-controlled copy remains available for review and later reinstall. Removing the rule does not alter saved headset lighting or PipeWire configuration.
