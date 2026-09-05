# Arch release package

The GitHub release carries both the checksum-pinned source archive referenced by its generated `PKGBUILD` and the resulting `x86_64` package. The source archive is produced by `cargo package`, so it contains the independently written build inputs and license texts while excluding raw USB captures and repository-internal agent and task metadata. `PKGBUILD.in` is the maintained template; `scripts/build-release` substitutes the Cargo version and actual source checksum.

Build and inspect the complete release bundle from the repository root:

```bash
scripts/validate
mise exec -- scripts/build-release --output target/release-assets/vVERSION
```

The script runs `makepkg`, its distributable test subset, package inspection, extracted-binary checks, and checksum verification. See the [release process](../../docs/release-process.md) for GitHub Actions publication and the local fallback.

Install the downloaded release package with pacman so its files remain tracked:

```bash
sudo pacman -U ./gamedacctl-VERSION-1-x86_64.pkg.tar.zst
```

Reconnect the GameDAC after installation so the packaged udev rule is applied. Package installation does not alter PipeWire or WirePlumber configuration.
