# Architecture

PSP Sidescreen turns a PSP into a side display for a Windows PC. The PC captures a monitor,
reduces it to the PSP's screen, compresses it and sends it over USB (or Wi-Fi). The PSP decodes
and shows it.

## Overview

```
Windows host                                     PSP (CFW 6.61 PRO-C)
┌─────────────────────────────┐                 ┌──────────────────────────┐
│ Source monitor              │                 │ EBOOT.PBP (user mode)    │
│   │  Desktop Duplication    │                 │   video decode (HW)      │
│ ┌─▼───────────────────┐     │   USB bulk      │   display 480x272        │
│ │ Scale to 480x272    │     │ ═════════════►  │        ▲                 │
│ │ (GPU, good filter)  │     │                 │   transport              │
│ └─┬───────────────────┘     │                 │   (usbhostfs-style PRX)  │
│ ┌─▼───────────────────┐     │                 └──────────────────────────┘
│ │ Compress (see below)│     │
│ └─┬───────────────────┘     │
│   ▼  Transport (WinUSB)     │
└─────────────────────────────┘
```

## Resolution policy

The PSP screen is **480x272** and the decoder on the console works at that size, so what goes
over the cable is always the PSP's size. The *source* size is a separate choice: it is the size
of the monitor the PC captures.

| Preset | Source monitor | Scale to PSP | Use |
|---|---|---|---|
| Native | 480x272 | 1:1 | Sharpest text, smallest workspace |
| Sharp | 960x544 | 2:1 exact | Crisp text with an integer scale, room for small windows |
| Workspace | 1366x768 | about 2.85:1 | Room for ordinary windows; text is small |
| Full HD | 1920x1080 | about 4:1 | Most room; text is only readable when Windows scaling is raised |

The trade-off is physical: a 480x272 screen cannot show more than 480x272 pixels, so a larger
source gives more room, not sharper text. Sharp text at a larger source size comes from Windows
display scaling (125% to 200%), which makes the fonts bigger before they are reduced. The host
downscales on the GPU with a filter that keeps thin strokes (not nearest-neighbour), which
makes a larger source look better than a naive reduction would. The preset is chosen once and
stored.

## Open questions, to be answered by measurement (milestone M0)

1. **Codec.** Hardware H.264 decode through the Media Engine, or hardware JPEG decode. The
   choice decides the compression stage on the host and the protocol payload. Measure on the
   console: decode time per frame, sustained frame rate, CPU use.
2. **USB throughput.** Bulk transfer rate achievable with the PSP's USB stack.
3. **Wi-Fi.** The PSP radio is 802.11b; measure real throughput before deciding it is useful.

## Key decisions already taken

- **The host owns scaling.** The PSP never scales; it decodes at its own size.
- **The pointer is drawn on the PC** into the picture, with the monitor's origin and Windows
  DPI scaling taken into account (the cause of the drift seen with PSPdisp is suspected to be
  missing DPI awareness; verified when the host exists).
- **Frames may be dropped before the encoder, never after it.** Keeps latency bounded.
- **Independent of Pane.** The host can capture any monitor, including one that another
  program owns, without changing that program or its driver.
