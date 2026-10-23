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

Task 011 adds Atkinson, Stucki and fixed-mask blue noise. Its software checks
do not establish visual quality on the approximately 6×4 cm memory LCD. When
hardware is available, compare a photograph, gradient/gray patches and text/thin
line art with Floyd–Steinberg/Bayer at normal handheld viewing distance. Record
firmware, ambient lighting and viewing distance, text/thin-line readability,
fine-dot visibility, apparent midtones, Atkinson shadow/highlight loss, and any
blue-noise tile repetition. Desktop comparisons at a measured physical size are
only a proxy, not completion of this real-display evaluation.

Task 012 adds Sierra, Burkes, Jarvis–Judice–Ninke, Riemersma and clustered dot.
Include them in the same comparisons, especially clustered-dot fine-detail loss
and Riemersma's coarser texture versus the raster diffusion choices.

Task 013 adds Sauvola, Ostromoukhov, bounded DBS, optional serpentine scanning
and mild output-size sharpening. Check Sauvola on unevenly lit documents, the
photo/gradient textures, DBS thin-detail softness, and sharpening noise/halos.
DBS's simple blurred-error model is not calibrated to the physical LCD.
