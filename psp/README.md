# The program on the PSP

PSP Sidescreen does not yet have a PSP program of its own. It ships PSPdisp's (BSD licence, see
`../NOTICE.md`), so nobody has to download PSPdisp.

| Folder | What it is |
|---|---|
| `vendor/pspdisp-0.6/` | PSPdisp 0.6's PSP files exactly as released, with their licences and the SHA-256 of each file (`SHA256SUMS`). |
| `package/PSP/GAME/Sidescreen/` | What goes on the Memory Stick. Made from `vendor/` by `scripts/package-psp.sh`. |
| `icon0.png` | The icon the PSP's menu shows, 144 x 80, made from the logo. |

## What the package changes

- The **title** in the menu is "PSP Sidescreen" and the **icon** is the logo's. The code inside
  `EBOOT.PBP` is PSPdisp's, unchanged: the script's check is that `DATA.PSP`, the part of the
  file that is the program, is the same before and after.
- `pspdisp.cfg` makes the program count down three seconds when it starts and then start USB mode
  by itself, so nothing has to be chosen on the PSP. Cancel in the countdown leads to its menu.

## Making the package again

`scripts/package-psp.sh` needs the PSPSDK tools (`unpack-pbp`, `pack-pbp`, `mksfoex`) in `PATH`,
for example from [pspdev](https://github.com/pspdev/pspdev). It checks `vendor/` against
`SHA256SUMS` first. The result is committed because the installer is built on Windows, where the
PSPSDK is not available.
