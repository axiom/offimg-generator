# 006 — R47 hardware validation

Depends on: 005.

## Goal

Confirm generated BMP files actually display correctly on the user's R47.

## Scope

Prepare a small asymmetric test pattern, a text/line-art image, and a dithered
photograph. Have the user copy them into the firmware's OFF-image directory
using its documented procedure. Record firmware version and observed results.
Do not automatically mount, modify, or eject the calculator's filesystem.

## Acceptance

- [ ] User confirms the correct OFF-image directory for their R47 firmware.
- [ ] Test pattern displays without inversion, mirroring, flipping, or shifts.
- [ ] Text/line art and dithered photograph both display successfully.
- [ ] Any required format corrections have regression tests.
- [ ] Basic usage, transfer instructions, and tested firmware are documented.

## Progress / handoff

Not started. Requires the user and physical hardware; software-only testing is
not a substitute. Preserve existing calculator files and safely eject after
manual copying.

The requested GUI follow-ups (007–010) take priority in the normal work order.
This hardware check can still run independently when the user and calculator
are available; it does not require interactive cropping or background loading.
