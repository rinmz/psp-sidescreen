# Wire Protocol

Normative description of the format between the host and the PSP application. The reference
implementation is the [`sidescreen-proto`](../crates/sidescreen-proto) crate. The two must
never disagree: a change updates this document and the crate in the same commit.

The format is the one of PSPdisp 0.6 (BSD licence, `source/psp/source/shared.h` and
`com.c` of that project), so the host works with the unmodified PSPdisp application.
It was read from that source, not specified here.

## Conventions

- All integers are **little-endian**.
- The PSP is a USB device (vendor `0x054C`, product `0x01C9` or `0x01CA`); the host writes to
  bulk endpoint `0x02` and reads from `0x81`.

## Opening the link

1. The host writes the 4 bytes `12 08 2F 78` (`HOSTFS_MAGIC`).
2. The host waits about 500 ms. Without the wait the next write "succeeds" without reaching
   the PSP.
3. The host writes 12 more bytes. If that write succeeds, the PSP was already initialised
   and nothing else is needed; the PSP discards those bytes because they carry no valid magic.
4. Otherwise the host reads 12 bytes (the PSP's hello) and writes the same 12 bytes back.

## Frame (host to PSP)

A 16-byte header, then `image_size` bytes of image, then `settings_size` bytes of settings.

| Offset | Size | Field | Notes |
|-------:|-----:|-------|-------|
| 0 | 4 | magic | `0xAFFE0600` |
| 4 | 4 | flags | see below |
| 8 | 4 | image_size | bytes of image after the header |
| 12 | 4 | settings_size | bytes of settings after the image; `0` for none |

Flags used: `0x00000001` the frame carries an image; `0x00000010` the image is a JPEG;
`0x00100000` clear the screen first. A JPEG frame is `0x11`. The image is 480x272.

## Response (PSP to host)

After every frame the PSP sends a response. Over USB it is always 90 bytes: a 14-byte header,
then the PSP's copy of the PC settings (19 `u32` fields), which the host ignores.

| Offset | Size | Field | Notes |
|-------:|-----:|-------|-------|
| 0 | 4 | magic | `0xAFFE0600` |
| 4 | 4 | flags | |
| 8 | 4 | buttons | PSP `SceCtrlButtons` bits |
| 12 | 1 | analog_x | 0 left, about 127 centre, 255 right |
| 13 | 1 | analog_y | 0 up, about 127 centre, 255 down |

## Verified on the console

On 2026-11-16, with a PSP-2000 on CFW 6.61 PRO-C running the unmodified PSPdisp 0.6
application and its WinUSB driver (type C, product `0x01CA`):

- The hello (above) opens the link, and a second hello on an already initialised PSP is
  accepted.
- A frame without a settings block (`settings_size = 0`) is accepted from the very first
  frame, and a 480x272 JPEG is displayed correctly.
- The response is 90 bytes and its header decodes as described.
- The host sends the header and the image as two separate writes, as PSPdisp does.

Not yet checked: behaviour over a long run, and whether one write for header and image together
would also work.
