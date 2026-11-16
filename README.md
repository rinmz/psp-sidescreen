# PSP Sidescreen

Use a PlayStation Portable as a low-latency side display for a Windows PC.

## Status

Planning. Nothing is built yet. The first milestone is a measurement on real hardware (see
[the roadmap](docs/ROADMAP.md)), because the choice of video codec depends on it.

## Target

- PSP-2000 running custom firmware 6.61 (PRO-C Infinity).
- Windows 11 host with a GPU that has a hardware video encoder.
- USB cable first, Wi-Fi as a fallback.

## Repository layout

```
crates/   Rust: the Windows host and the protocol shared with the PSP side
psp/      C (PSPSDK): the EBOOT.PBP that runs on the console
docs/     architecture, roadmap, engineering standards
scripts/  tool setup and packaging
```

## Naming

| Thing | Name |
|---|---|
| Product | PSP Sidescreen |
| Repository | `psp-sidescreen` |
| Rust crates | `sidescreen-<role>`, for example `sidescreen-host`, `sidescreen-proto` |
| PSP application folder | `PSP/GAME/Sidescreen/EBOOT.PBP` |
| PSP module name | `sidescreen` |
| PSP title ID | `SIDE00001` |

This project is independent of the Pane repository and does not modify it.

## Contributing

Read the [engineering standards](docs/CONTRIBUTING.md) first.
