# Usage

## The window

Start **PSP Sidescreen**. The window has three pages next to the **Setup** page that is shown
the first time:

- **Display**: the **Start** button, what the PSP shows (an extended screen of its own, or a
  monitor you already have), the size of that screen and the quality.
- **Controls**: what the PSP's buttons do on the PC (see below).
- **About**: the version, and the projects it is built on.

The icon stays in the notification area when the window is closed. **Quit** in its menu ends the
program.

## Controls

Off by default. Turn on **Use the PSP's buttons**, then choose for each button:

- **Nothing**, one of the ready-made ones (keys such as Enter, Escape, Space, Tab and
  Delete; copy, paste and undo; refresh; mute and the other media keys), or **Custom**,
  where you press the keys you want and they are recorded.
- A shortcut stays pressed while the button is held, like holding the keys. A short tap is not
  missed: the PSP is asked for its buttons about thirty times a second.
- **Stick moves the mouse** turns the analog stick into a pointer.

The volume, note and screen buttons of the PSP are not reported by the program that runs on the
PSP, so they cannot be used yet; see the [roadmap](ROADMAP.md).

## Quality

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

## Command line

The same session can be run without the window. The program on the PSP has to be running in USB
mode. Then:

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

Other commands: `monitors` lists the monitors, `probe` checks the link to the PSP, `buttons`
prints what the PSP reports as you press buttons, and `send <file.jpg>` shows one picture.

## Repository naming

| Thing | Name |
|---|---|
| Product | PSP Sidescreen |
| Repository | `psp-sidescreen` |
| Rust crates | `sidescreen-<role>`, for example `sidescreen-host`, `sidescreen-proto` |
| PSP application folder | `PSP/GAME/Sidescreen/EBOOT.PBP` |
| PSP module name | `sidescreen` |
| PSP title ID | `SIDE00001` |
