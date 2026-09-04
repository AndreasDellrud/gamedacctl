---
title: Windows and capture workflow
type: pattern
status: current
updated: 2026-09-05
sources:
  - docs/raw/capture-effects-20260904-2323.usbmon
  - docs/raw/capture-zones-20260904.usbmon
  - docs/raw/capture-full-effects-mic-20260905.pcapng
---

# Windows and capture workflow

## Windows VM

Omarchy 4.0.1 uses the `dockurr/windows` container, QEMU/KVM, a local noVNC console on port 8006, and RDP on localhost port 3389. Windows 11 and SteelSeries GG 118.0.0 were installed successfully.

The active root-owned Compose file is `/var/lib/omarchy/windows/docker-compose.yml`. A pre-passthrough rollback copy is `/var/lib/omarchy/windows/docker-compose.yml.pre-gamedac-20260904-1808`.

The relevant supported Dockur configuration is:

```yaml
environment:
  ARGUMENTS: "-rtc base=localtime,clock=host,driftfix=slew -device usb-host,vendorid=0x1038,productid=0x1280"
devices:
  - /dev/kvm
  - /dev/net/tun
  - /dev/bus/usb
```

Only the control PID is passed through. Do not pass `1038:1282` unless Windows must own GameDAC audio too.

Start and retain the VM with:

```bash
omarchy windows vm launch --keep-alive
```

Stop it cleanly with:

```bash
omarchy windows vm stop
```

The official upstream USB-passthrough pattern is documented at <https://github.com/dockur/windows#how-do-i-pass-through-a-usb-device>. SteelSeries distributes GG from <https://steelseries.com/gg>. The proprietary installer is deliberately not committed.

## Verify ownership before capture

After Windows starts:

```bash
lsusb -d 1038:1280
lsusb -t
wpctl status
```

Expected state:

- All three `1038:1280` interfaces show driver `usbfs`, meaning QEMU claimed them.
- `1038:1282` remains on `snd-usb-audio`.
- GameDAC Game remains the Linux default sink.

USB bus and device numbers change after reconnects and host reboots. Resolve them immediately before every capture.

## Text usbmon capture

Load the read-only monitor after each host reboot:

```bash
sudo modprobe usbmon
```

For bus 1, device 22, the capture used:

```bash
sudo timeout 600 stdbuf -oL cat /sys/kernel/debug/usb/usbmon/1u \
  | rg --line-buffered ':0*22:' \
  | tee capture.usbmon
```

Replace both bus and device numbers with current values. Start capture before changing Engine settings, enter hex colors directly, avoid dragging the picker, apply once, and wait at least five seconds between labeled actions.

Text usbmon is sufficient to identify transport and early fields but truncates the displayed data portion of long transfers. Use a binary usbmon/pcap capture for complete animation reports.

## Full binary capture

Install Arch package `wireshark-cli`, then capture the whole USB bus with a full snap length:

```bash
sudo dumpcap -q -i usbmon1 -s 0 -w /tmp/gamedac-capture.pcapng
```

Stop with Ctrl-C. Linux's USB capture backend does not accept a device-address capture filter, so the temporary file contains unrelated traffic from that bus. Resolve the current GameDAC address, immediately extract only that device, and discard the temporary whole-bus file:

```bash
tshark -r /tmp/gamedac-capture.pcapng \
  -Y 'usb.device_address == 22' \
  -w docs/raw/capture-filtered-YYYYMMDD.pcapng
```

Replace device 22 with the current address. Verify the filtered file contains only that address, record its SHA-256, and never commit the temporary whole-bus capture.

## Safety and cleanup

- Do not accept a GameDAC firmware update during protocol capture.
- Do not pass USB storage devices into a first-time Windows installation.
- Stop the VM before replaying packets natively; otherwise QEMU owns the HID interface.
- Confirm GameDAC audio still exists after every USB handoff.
- Preserve raw traces unchanged and record user-visible behavior separately.

To roll back passthrough, stop the VM, restore the dated Compose backup with root privileges, validate it with `docker compose config --quiet`, and start the VM again. Do not edit packaged files under `/usr/share/omarchy`.
