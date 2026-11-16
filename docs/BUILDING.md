# Building

## What is needed

- Windows 10 or 11, 64-bit.
- Rust (the version is pinned by `rust-toolchain.toml`) and the Visual Studio C++ Build Tools.
- Node.js 24 or newer.
- The WebView2 runtime, which Windows 11 has. The installer fetches it on Windows 10.

## Run the checks

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd app
npm ci
npm run typecheck && npm run lint && npm run format:check && npm test && npm run build
```

## Build the installer

```
powershell -ExecutionPolicy Bypass -File scripts\build-installer.ps1
```

It fetches the virtual display driver (checked against a pinned hash), builds the helper, gathers
what the installer carries, builds the window and the program, and packs it all. The result is
`target\release\bundle\nsis\PSP Sidescreen_<version>_x64-setup.exe`.

## What the installer does

A classic Windows wizard: welcome, licence, folder, install, finish. While installing it
puts the virtual display driver and the USB driver for the PSP on the machine (see
`docs/SIGNING.md` for the second), by running `sidescreen-setup.exe`. The first time the program
opens, its Setup page walks the person through putting the program on the PSP.

On uninstall it removes the virtual display driver only if the installer put it there and the
person agrees, because other programs may use it.

## The program on the PSP

`psp/package` is committed. To make it again from the PSPdisp files in `psp/vendor`, run
`scripts/package-psp.sh` where the PSPSDK tools are available (see `psp/README.md`).
