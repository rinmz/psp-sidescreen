# PSP Sidescreen

Use a PlayStation Portable as a low-latency side display for a Windows PC.

## Status

Working prototype. The host shows a monitor, or a virtual monitor of its own, on a PSP that is
running the PSPdisp 0.6 application. See [the roadmap](docs/ROADMAP.md).

## Use

Start PSPdisp on the PSP and choose **Start using USB**; close PSPdisp on the PC. Then:

```
sidescreen-host run --virtual                    # the PSP gets a monitor of its own (extended)
sidescreen-host run --virtual --quality lossless # sharpest picture
sidescreen-host run --monitor 0                  # mirror an existing monitor instead
sidescreen-host detach                           # remove the virtual monitor if a run left it
```

Ctrl+C ends a run and removes the virtual monitor.

| Option | Meaning |
|---|---|
| `--virtual` | Give the PSP a virtual monitor of its own. Needs the Virtual Display Driver. |
| `--size WxH` | Size of the virtual monitor; the driver offers 800x600, 1366x768, 1920x1080, 2560x1440 and 3840x2160. Default 1366x768. |
| `--quality NAME` | `lossless`, `high`, `medium` (default) or `low`. |
| `--monitor N` | Show monitor `N` (see `sidescreen-host monitors`) when `--virtual` is not given. |

### Quality

The PSP screen has 480x272 pixels, so every picture is reduced to that size on the PC. The
quality sets how it is then sent:

| Name | How | Expected |
|---|---|---|
| `lossless` | raw pixels, 391 680 bytes a frame | exact picture; the most data per frame, so the lowest frame rate |
| `high` | JPEG 92 | very close to exact |
| `medium` | JPEG 80 | good, balanced |
| `low` | JPEG 50 | smallest frames and the highest frame rate, with visible compression |

The last column is what the settings are expected to do; frame rates per setting have not been
measured on the console yet.

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
