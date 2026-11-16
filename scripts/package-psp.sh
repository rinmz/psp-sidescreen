#!/usr/bin/env bash
# Builds the folder that goes on the PSP's Memory Stick, psp/package/PSP/GAME/Sidescreen, from the
# PSPdisp files kept in psp/vendor/pspdisp-0.6.
#
# Only the title, the icon and the settings change. The code in the EBOOT is PSPdisp's, as
# NOTICE.md says. Needs the PSPSDK tools (unpack-pbp, pack-pbp, mksfoex); see docs/BUILDING.md.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PSPDEV="${PSPDEV:-/usr/local/pspdev}"
export PATH="$PATH:$PSPDEV/bin"

vendor="$root/psp/vendor/pspdisp-0.6"
out="$root/psp/package/PSP/GAME/Sidescreen"
title="PSP Sidescreen"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

echo "checking the PSPdisp files against their recorded hashes"
(cd "$vendor" && sha256sum --check --quiet SHA256SUMS)

echo "unpacking the EBOOT"
cp "$vendor/EBOOT.PBP" "$work/EBOOT.PBP"
(cd "$work" && unpack-pbp EBOOT.PBP >/dev/null)

echo "making the new title"
mksfoex "$title" "$work/NEW.SFO"

echo "packing the EBOOT with the new title and icon"
rm -rf "$root/psp/package"
mkdir -p "$out"
pack-pbp "$out/EBOOT.PBP" "$work/NEW.SFO" "$root/psp/icon0.png" NULL NULL NULL NULL "$work/DATA.PSP" NULL

cp "$vendor/kernel.prx" "$vendor/usbhostfs.prx" "$out/"
cp -r "$vendor/graphics" "$out/graphics"
cp "$vendor/LICENSE-PSPdisp.txt" "$vendor/LICENSE-PSPLINK.txt" "$out/"

# On start the program counts down three seconds and then starts USB mode by itself; Cancel in
# that countdown leads to its menu. The file's format is one "name value" per line.
printf 'usbMode 1\nautoMode 2\nautoTimeout 3\n' > "$out/pspdisp.cfg"

echo "done: $out"
(cd "$out" && find . -type f | sort | xargs ls -l | awk '{print $5, $9}')
