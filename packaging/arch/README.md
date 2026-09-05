# Arch release package

The GitHub release carries both the checksum-pinned source archive referenced by `PKGBUILD` and the resulting `x86_64` package. The source archive is produced by `cargo package`, so it contains the independently written build inputs and license texts while excluding raw USB captures and repository-internal agent and task metadata.

Build and inspect the package from this directory:

```bash
makepkg --cleanbuild --verifysource
makepkg --cleanbuild
pacman -Qip ./gamedacctl-0.1.0-1-x86_64.pkg.tar.zst
pacman -Qlp ./gamedacctl-0.1.0-1-x86_64.pkg.tar.zst
```

Install the downloaded release package with pacman so its files remain tracked:

```bash
sudo pacman -U ./gamedacctl-0.1.0-1-x86_64.pkg.tar.zst
```

Reconnect the GameDAC after installation so the packaged udev rule is applied. Package installation does not alter PipeWire or WirePlumber configuration.
