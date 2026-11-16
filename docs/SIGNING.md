# Signing

What is signed, what is not, and what that means for a person installing PSP Sidescreen.

## Where things stand

| Piece | Signed? | Effect when it is not |
|---|---|---|
| Virtual display driver | Yes, by SignPath Foundation (the driver's own project) | None |
| `sidescreen-setup.exe` and `psp-sidescreen.exe` | No | Windows SmartScreen warns on the installer ("unknown publisher"); the person chooses Run anyway |
| USB driver for the PSP (`drivers/psp/sidescreen-psp.inf`) | **No** | Windows refuses to install it, so the PSP cannot be reached on a machine that does not already have a driver for it |

The last row is the one that matters. Windows only accepts a driver package whose catalog is
signed by a publisher it trusts. The INF itself carries no driver code, it only tells Windows to
use its own WinUSB driver for the PSP, but Windows checks the signature all the same. The
installer reports this in its own words (exit code 4 of `sidescreen-setup`) instead of failing
quietly, and installs everything else.

A machine that already has a working driver for the PSP, for example from an earlier PSPdisp
install, is detected and left alone.

## Ways to fix it

1. **Sign with SignPath Foundation (planned).** The Foundation signs open-source projects for
   free. It needs the repository to be public, an OSI licence (this is MIT), builds on a public CI
   system, and a short application. The Virtual Display Driver is signed that way. What the
   application needs from this repository is a signing policy page and a CI workflow that submits
   `sidescreen-psp.cat` (made with `inf2cat` from the Windows Driver Kit) and the executables.
2. **Give the PSP program its own USB descriptors.** A device can tell Windows to use WinUSB by
   itself, with Microsoft OS descriptors, and then no driver package is needed at all. That means
   a PSP program of our own with its own USB driver module; the program shipped now is PSPdisp's.
   It is the best result for the person installing, and the most work.
3. **Sign it with a certificate of our own that the installer trusts** (the way Zadig does). It
   works, but it adds a trusted root certificate to the person's machine, which is not something
   to do quietly. It is not planned.

Until one of these is done, a clean Windows machine needs the PSP driver to be installed by hand.
