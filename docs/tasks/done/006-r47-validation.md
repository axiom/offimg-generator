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

- [x] User confirms generated images display correctly on their R47 and accepts
  hardware validation as complete.
- [x] No format corrections were reported or required.
- [x] Basic usage and hardware confirmation are documented in README; firmware
  version and exact transfer directory were not supplied.

## Progress / handoff

Complete: the user tested generated images on the device, reported “Looks
perfect”, and explicitly requested closure. This records user-confirmed hardware
acceptance, not an inference from software tests. Firmware version, exact transfer
directory, individual test images/settings, lighting and viewing distance were
not supplied; no detailed per-pattern or per-algorithm observations are claimed.
The comparison guidance below is retained for reference, not as pending work.

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
