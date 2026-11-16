# Roadmap

Where the project is, and where help is most welcome. Items are in rough order of priority.

## Done

- The host: capture, scale to 480x272, JPEG or raw, over USB, with a keep-alive and reconnect.
- A virtual monitor of its own for the PSP (extended display), or mirroring an existing monitor.
- Four quality levels; screen sizes from 800x600 to 3840x2160.
- The PSP's buttons as keyboard and mouse shortcuts, including recorded chords; the stick as a
  mouse.
- Resilience: resolution changes, <kbd>Win</kbd>+<kbd>P</kbd>, a pulled cable, a PSP that sleeps.
- The window, the tray icon, and a Windows installer that carries the drivers and the program for
  the PSP, with a Setup page that puts it on the Memory Stick.

## Next

| Item | Why | Notes |
|---|---|---|
| Update check | People should not have to look for new versions | Check the latest GitHub release; offer to install it |
| Signed USB driver | Removes the manual step on a new PC | See [SIGNING.md](SIGNING.md) |
| Our own PSP program | The volume, note and screen buttons are not reported by the current one; it would also let Windows use the PSP without any driver (Microsoft OS descriptors) | Needs a PSP toolchain and the console to test |
| Frame rate measurements | The quality levels are not measured on the console yet | Recorded in `docs/MEASUREMENTS.md` |
| Start with Windows | The tray program should be there when the PSP is plugged in | |
| More PSP models and firmware | Only a PSP-2000 on 6.61 PRO-C has been tried | Reports welcome |

## Ideas

- Wi-Fi as a fallback to the cable.
- Audio to the PSP.
- Several virtual monitors, so the program and other programs using the same driver can share it.
