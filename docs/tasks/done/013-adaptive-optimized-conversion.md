# 013 — Adaptive/optimized conversion and sharpening

Depends on: 012's software implementation. Independent of hardware task 006.

## Goal / agreed scope

The user selected **all four + sharpening**: Sauvola adaptive thresholding,
Ostromoukhov variable-coefficient diffusion, direct binary search, serpentine
variants, and optional mild pre-dither sharpening in CLI and GUI. Keep existing
algorithm names/output, Floyd–Steinberg default, export safeguards and BMP format.
No new dependencies or extra algorithm-parameter controls.

## Decisions

- `sauvola`: clipped 15×15 integral-statistics windows, k=0.2, R=128. Zero stays
  black even at a zero local threshold. Document-oriented, not a photo dither.
- `ostromoukhov`: original-gray-indexed, mirrored 128-entry coefficient table;
  always serpentine, floating error, white at 127.5, no intermediate clamp.
  Pinned table source/checksum and MIT notice in `assets/ostromoukhov/`.
- `dbs`: fixed 3×3 binomial perceptual blur, full-convolution squared-error
  objective including halo, Floyd–Steinberg seed, best improving toggle or
  eight-neighbor opposite-color swap, up to four deterministic sweeps. Bounded
  approximation, not global optimization or an LCD-calibrated quality guarantee.
- `--serpentine` / **Serpentine scan**: reverse odd rows and horizontal tap
  directions for Floyd–Steinberg, Atkinson, Stucki, Sierra, Burkes and JJN.
  Unsupported choices ignore it; GUI disables it but retains its value.
- `--sharpen` / **Mild sharpening**: output-size 3×3 binomial unsharp mask,
  amount 0.5, clamped borders, round/clamp to bytes, **before** tone adjustment.
  Both options default/reset off. No cumulative edits to grayscale caches.
- GUI DBS work is detached/coalesced: one worker, no request queue, revision
  checks across tone/framing/crop/source/algorithm changes, 50 ms result polling.
  While pending, show a label and retain/export displayed pixels. New DBS image
  loads initially display the Floyd–Steinberg seed. Closing never joins workers.

## Acceptance

- [x] All three named algorithms and the two opt-in controls are wired through
  shared settings, CLI help/parsing, GUI controls, defaults/reset and docs.
- [x] Existing raster/default output is unchanged with options off; numeric
  validation and source/overwrite safeguards remain intact.
- [x] Tests cover local Sauvola statistics against a direct reference, narrow and
  clipped windows, Ostromoukhov fixtures/table provenance, and all six serpentine
  kernels against independent mirrored-kernel calculations.
- [x] DBS tests independently measure full blurred-error energy, require
  non-increasing energy over each sweep, and exercise a swap where neither
  individual toggle improves. All choices are binary, deterministic and preserve
  solid endpoints without input mutation.
- [x] Sharpening tests cover the hand-calculated filter, borders, constants,
  rounding, opt-in defaults, fresh input, and sharpening-before-tone order.
- [x] CLI integration and GUI tone/crop/inversion/zoom/preview/export tests cover
  all fourteen algorithms with sharpening and serpentine enabled.
- [x] GUI tests hold a render open while coalescing 100 edits, reject stale
  results after crops/source/algorithm changes, check displayed-preview export,
  current-worker disconnection and dropping the receiver on close.
- [x] Pass `just check` and CLI-only tests; record timings/results below.
- [x] User checks KDE Wayland controls/responsiveness and visual tradeoffs on
  documents, photographs, gradients and thin-line art at 1×/2× and measured
  approximately 6×3.6 cm. Real memory-LCD evaluation remains task 006.

## Progress / handoff

Complete: implementation and automated checks pass, and the user explicitly
confirmed acceptance and requested closure. Desktop/visual acceptance is recorded
on that confirmation; no detailed per-image observations or ruler measurements
were supplied. Hardware results are not inferred; task 006 remains open.
Full algorithm definitions and
worker behavior are in `docs/design.md`; usage/tradeoffs are in README.
The original blue-noise mask, BMP fixtures and dependencies are unchanged.

Verified commands/results:

- `nix develop --command just check`: formatting, all-target/all-feature Clippy
  with warnings denied, and **48 tests passed** (3 manual timing tests ignored).
- `nix develop --command cargo test --no-default-features`: **37 tests passed**.
- All 128 Ostromoukhov triples were compared with the pinned source's Python
  literal; SHA-256/provenance and the source MIT license are retained alongside
  the table documentation.
- `nix develop --command cargo test --all-features dithering_switch_timings -- --ignored --nocapture --test-threads=1`:
  debug build, generated 400×240 gray pattern, cached tone/dither/CPU texture
  update, 20 measured runs after warmup (3 for DBS):

  | Choice | Mean render/texture time |
  | --- | --- |
  | Floyd–Steinberg, options off | 12.19 ms |
  | Sauvola | 17.15 ms |
  | Ostromoukhov | 15.89 ms |
  | DBS | 171.57 ms |
  | Floyd–Steinberg + sharpening + serpentine | 42.39 ms |

- `nix develop --command cargo test --all-features dbs_coalesces -- --nocapture --test-threads=1`:
  passed; 100 control updates with a held DBS request took **14.8 µs**, with no
  additional worker or snapshot queued. This measures the coalescing path, not
  end-to-end GUI frame latency.

Timings vary with host load and are not desktop frame measurements. The user's
confirmation completes desktop acceptance, not real calculator validation.
Task 006 retains the checks for Sauvola noise, diffusion texture, DBS fine-detail
loss and sharpening halos on the memory LCD. Neither the numerical DBS objective
nor desktop magnification establishes real LCD quality.
