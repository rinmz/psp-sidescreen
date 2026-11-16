# Security policy

## Supported versions

Only the latest release is supported. Fixes are made on `main` and released as a new version.

## Reporting a vulnerability

Please do **not** open a public issue for a security problem.

Use GitHub's private reporting instead: open the
[Security tab](https://github.com/rinmz/psp-sidescreen/security/advisories/new) of this
repository and choose **Report a vulnerability**. Say what you found, how to reproduce it, and
what you think it allows.

You can expect an answer within a week. Once a fix is released, the report is published with
credit to you unless you prefer otherwise.

## What matters most here

The installer runs with administrator rights and installs two drivers; the program talks to a USB
device and can send keyboard and mouse input. Reports about those parts are the most valuable:
the installer and `sidescreen-setup`, the driver files, the USB link, and the handling of what the
PSP sends.
