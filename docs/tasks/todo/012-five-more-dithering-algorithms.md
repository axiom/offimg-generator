# 012 — Five more dithering algorithms

Depends on: 011's software implementation. Independent of hardware task 006.

## Goal / scope

After approving the previous set, the user requested more algorithms and selected
**all five** offered: Riemersma, clustered dot, Sierra, Burkes, and
Jarvis–Judice–Ninke. Add them to the shared library, CLI, and GUI without changing
existing names/output, the Floyd–Steinberg default, tone/crop/inversion, or BMP.
No new dependencies, random behavior, tuning controls, or temporal dithering.

## Decisions

- Full three-row **Sierra**, not two-row Sierra or Sierra Lite. Weights divide
  `X 5 3 / 2 4 5 4 2 / 0 2 3 2 0` by 32.
- **Burkes**: `X 8 4 / 2 4 8 4 2`, divided by 32.
- **Jarvis–Judice–Ninke**: `X 7 5 / 3 5 7 5 3 / 1 3 5 3 1`, divided by 48.
  All three reuse the existing private raster diffusion loop: left-to-right,
  top-to-bottom, floating error, white at 127.5, discard out-of-image taps.
- **Riemersma**: Hilbert curve over the containing power-of-two square, pruning
  out-of-image quadrants. Keep 16 raw source-minus-output errors. Newest weight
  1, oldest 1/16, exponential weights `16^(-age/15)`, unnormalized sum. Skipped
  coordinates do not change history. No runtime recursion grows beyond the
  power-of-two depth; narrow rectangles do not traverse the padded square.
- **Clustered dot**: fixed 4×4 ranks
  `12 5 6 13 / 4 0 1 7 / 11 3 2 8 / 15 10 9 14`, centered thresholds, tiled
  at output coordinates. This intentionally favors clusters over fine stipple.

Algorithm references (formulas/concepts, not copied implementation code):

- https://tannerhelland.com/2012/12/28/dithering-eleven-algorithms-source-code.html
- https://www.compuphase.com/riemer.htm — finite error history, exponential decay,
  and raw source-minus-output error, rather than corrected-minus-output.

## Acceptance

- [x] All five have documented CLI names, parsing/display/errors/help, and GUI
  choices: `sierra`, `burkes`, `jarvis-judice-ninke`, `riemersma`, `clustered-dot`.
- [x] Kernel fixtures cover rows, columns, edges, two future rows, and no wrapping;
  clustered-dot tests verify center growth, coverage, endpoints and tiling.
- [x] Hilbert tests verify known order, uniqueness, adjacency for full squares,
  clipping, empty/tiny inputs, 400×240 and extreme narrow rectangles. Independent
  iterative Hilbert/f64 history checks cover Riemersma queue expiration and raw
  errors, not merely a second call to the implementation.
- [x] All eleven choices are deterministic, binary, preserve solid endpoints,
  leave input unchanged, ignore cutoff except threshold, and invert correctly.
- [x] CLI integration and GUI crop/tone/preview/export tests cover every algorithm,
  including all three preview magnifications without changing cached grayscale.
- [x] `just check`, CLI-only tests, and a headless cached-render timing comparison.
- [ ] User compares the new choices in the GUI on photos, gradients/gray patches,
  and text/thin-line art; confirms switching responsiveness and records tradeoffs.
  Check at 1×/2× and measured approximately 6×3.6 cm (400:240 aspect ratio).
  Real memory-LCD results remain task 006, not inferred from desktop approval.

## Progress / handoff

Implemented the five algorithms, shared `Algorithm::ALL` enumeration, front-end
selection/help, README usage/tradeoffs, design decisions, regression/integration
tests, and task 006 display-evaluation handoff. Original algorithm implementations,
mask, BMP fixtures, and defaults are unchanged. The controls sidebar already
scrolls vertically, keeping the longer algorithm list and tone/reset accessible.

Verified:

- `nix develop --command just check`: formatting, all-target/all-feature Clippy
  with warnings denied, and **41 tests passed** (3 manual timing tests ignored).
- `nix develop --command cargo test --no-default-features`: **31 tests passed**.
- `nix develop --command cargo test --all-features dithering_switch_timings -- --ignored --nocapture --test-threads=1`:
  passed. Debug build, generated 400×240 gray pattern, one warmup and 20 measured
  cached GUI tone/dither/CPU texture updates per algorithm:

  | Algorithm | Mean render/texture update |
  | --- | --- |
  | Threshold | 9.84 ms |
  | Floyd–Steinberg | 13.54 ms |
  | Bayer | 10.35 ms |
  | Atkinson | 17.45 ms |
  | Stucki | 23.31 ms |
  | Blue noise | 7.01 ms |
  | Sierra | 15.22 ms |
  | Burkes | 12.91 ms |
  | Jarvis–Judice–Ninke | 16.64 ms |
  | Riemersma | 37.69 ms |
  | Clustered dot | 6.77 ms |

These single-run timing averages vary with host load and do not measure actual
desktop presentation or responsiveness. No new desktop/physical-size/hardware
observations have been claimed.

Next: `just gui`, choose each new algorithm, and compare the same source/crop/tone
settings. Riemersma should give a less raster-oriented but coarser texture;
clustered dot intentionally exposes growing halftone dots and may obscure small
text/lines. Sierra/Burkes/JJN differences are image-dependent, not guaranteed
improvements. Record actual observations before moving this task to `done/`.
