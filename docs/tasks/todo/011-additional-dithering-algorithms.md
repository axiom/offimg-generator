# 011 — Additional monochrome dithering algorithms

Depends on: 004 and 005. Independent of hardware validation 006.

## Goal

Offer a few meaningfully different looks for 400×240 OFF-images, available in
both CLI and GUI, without changing existing conversions or defaults.

The user identifies the target as a **memory LCD, approximately 6×4 cm**.
At 400×240 this is roughly 150–170 ppi (dimensions are approximate), with pixels
about 0.15–0.17 mm across. Judge readability and apparent gray at normal handheld
viewing distance, not just enlarged desktop pixel art. The shortlist is provisional
for this physical display, not based on an assumption of e-paper behavior.

## Recommended algorithms

| CLI name | GUI label | Why include it | Tradeoff |
| --- | --- | --- | --- |
| `atkinson` | Atkinson | Classic Macintosh look, with stronger contrast and clear detail. | Discards a quarter of the error; subtle shadows/highlights can disappear. |
| `stucki` | Stucki | Wider diffusion kernel offers an alternative for photographs and gradients. | More work per pixel than Floyd–Steinberg; improvement is image-dependent. |
| `blue-noise` | Blue noise | Organic dispersed dots without Bayer's obvious grid or raster diffusion's directional patterns. | A tiled mask can still show repetition; appearance needs visual evaluation at target size. |

These are candidates for useful visual variety, not a claim that one is always
better. Compare them at actual 400×240 resolution before declaring acceptance.

## Scope

- Add implementations through the existing `Dither` trait and `Algorithm` enum
  in `src/dither.rs`. Keep Floyd–Steinberg as the default; preserve the existing
  `threshold`, `floyd-steinberg`, and `bayer` names and pixel output.
- Atkinson: scan left-to-right, top-to-bottom; distribute 1/8 of the error to
  `(1,0)`, `(2,0)`, `(-1,1)`, `(0,1)`, `(1,1)`, `(0,2)` relative to the pixel.
  Do not renormalize the six weights to sum to one.
- Stucki: same traversal; use the kernel below, divided by 42 (`X` is current):

  ```text
          X  8  4
    2  4  8  4  2
    1  2  4  2  1
  ```

- Match current diffusion conventions: floating-point error, white at
  `>= 127.5`, discard out-of-bounds contributions without row wrapping or
  renormalizing. Do not change the existing Floyd–Steinberg implementation's
  output while adding the new kernels. Share only small, actually useful code;
  no configurable diffusion framework.
- Blue noise: embed one fixed tileable 64×64 single-channel threshold mask from
  Christoph Peters' CC0 texture set linked below. Record the exact source file,
  selected channel, conversion (if any), checksum, and license alongside the
  asset. Keep its license notice in the repository. Use centered thresholds
  strictly inside (0,255), e.g. `(mask_byte + 0.5) * 255 / 256` for an 8-bit mask,
  so solid black/white remain unchanged. Tile at fixed output-pixel coordinates;
  no random offsets, seeds, downloads, or generation at build/runtime. Do not
  substitute white noise or a shuffled Bayer matrix and call it blue noise.
- Wire up enum parsing/display/error messages, CLI help in `src/bin/offimg.rs`,
  and the GUI algorithm choices in `src/gui.rs`. New algorithms use existing
  tone/inversion controls; the cutoff remains threshold-only.
- Preserve the pipeline: full-resolution framing/crop, grayscale, existing tone
  adjustments, dithering, then inversion. Preview and export share the same
  monochrome buffer; preview magnification never affects dithering.
- Use static spatial dithering only: an OFF-image is a single 1-bit BMP, not an
  animation. No temporal noise, flashing, refresh logic, or e-paper-specific
  compensation. Do not assume desktop gamma/contrast matches the memory LCD.
- Update tests and README usage with the choices and their visual tradeoffs.
  No new dependency, extra tuning controls, default changes, color-space changes,
  GPU path, or changes to BMP encoding.

## Acceptance

- [x] All three algorithms are selectable in both CLI and GUI, with documented
      names; old names/defaults/output remain compatible.
- [x] Small hand-calculated fixtures verify both diffusion kernels, including
      Atkinson's intentional error loss, edges, narrow images, and no row wrap.
- [x] Blue-noise tests verify asset dimensions, threshold endpoints, tiling, and
      deterministic output; checksum, asset provenance and CC0 notice recorded.
- [x] All new algorithms produce only black/white, preserve solid black/white,
      do not mutate input, and are deterministic on repeated conversion.
- [x] CLI integration tests match shared settings; GUI tests verify each new
      algorithm's preview/export agreement with tone, inversion, and crop.
- [ ] Visually compare a photograph, gradient/gray patches, and text/line art
      against Floyd–Steinberg/Bayer at 1× and 2× on KDE Wayland. Record observed
      tradeoffs, including shadow/highlight loss and blue-noise tile visibility.
- [ ] Also compare at approximately 6×4 cm physical size, preserving the image's
      400:240 aspect ratio rather than stretching to match approximate dimensions.
      Record text/thin-line readability, fine-dot visibility, apparent midtones,
      and whether Atkinson loses important shadow/highlight detail. Desktop
      physical size must be measured, not inferred from the preview zoom label.
      If hardware is available, repeat on the memory LCD at normal viewing
      distance under typical ambient light and record conditions; otherwise
      explicitly hand off real-display evaluation to task 006.
- [ ] Check GUI algorithm switching remains responsive at 400×240, record a
      simple render timing comparison, and pass `just check`.

## Research references

Consulted online during planning:

- [Tanner Helland — Image Dithering: Eleven Algorithms and Source Code](https://tannerhelland.com/2012/12/28/dithering-eleven-algorithms-source-code.html):
  Atkinson/Stucki kernels and comparisons with Sierra, Burkes, and JJN.
- [Surma — Ditherpunk](https://surma.dev/things/ditherpunk/): monochrome visual
  comparisons, blue-noise threshold maps, Atkinson, and Riemersma.
- [Christoph Peters — Free blue noise textures](https://momentsingraphics.de/BlueNoise.html):
  tileable void-and-cluster threshold textures; the downloadable texture database
  is explicitly CC0. Verify the selected archive's notice when vendoring it.

Defer Sierra/Burkes/JJN variants because of overlap with the selected diffusion
algorithms; defer Riemersma's Hilbert traversal and clustered-dot halftoning
until these three have been evaluated. No copied reference photos are needed:
use generated patterns and user-provided or suitably licensed photographs.

## Progress / handoff

Software implementation and automated verification complete; keep this task in
`todo/` until desktop visual/physical-size comparisons and live switching pass.

Implemented:

- `src/dither.rs`: Atkinson/Stucki through one private diffusion loop; original
  Floyd–Steinberg, Bayer and threshold implementations unchanged. Added enum
  names/parsing/display, CLI help, and GUI radio choices; defaults unchanged.
- `assets/blue-noise/mask.bin`: 4096 raw red-channel bytes from
  `Data/64_64/LDR_LLL1_0.png` in Peters' texture archive. Verified the archive's
  CC0 notice; retained its notice/legal text and recorded archive/source/mask
  SHA-256 checksums beside the asset. No dependency or runtime download added.
- Core tests cover small diffusion fixtures (single pixel, row, column, edges,
  two future rows, intentional error loss, no wrapping), all-byte centered
  thresholds, fixed mask distribution/64-pixel tiling, names/defaults, immutable
  input, binary/deterministic output, and solid endpoints. Non-threshold choices
  ignore the cutoff. Existing regression tests still pass.
- CLI integration now compares all six choices with shared crop/tone/inversion
  settings. Headless GUI tests compare each new choice with a non-centered 2×
  crop, brightness/contrast/gamma/inversion, decoded BMP, and preview pixels at
  all three preview magnifications, retaining the grayscale cache.
- README and design document describe the choices, provenance and tradeoffs.
  The supplied `numbers.bmp` and `numbers2.bmp` were not changed.

Verified commands:

- `nix develop --command just check`: formatting, all-feature/all-target Clippy
  with warnings denied, and 37 tests passed (3 manual timing tests ignored).
- `nix develop --command cargo test --no-default-features`: 27 tests passed.
- `sha256sum assets/blue-noise/mask.bin`: matches
  `fe602dd3f05c3d79ef1ff01533923c45a9eede45531251901818250445fbe4b1`.
- `nix develop --command cargo test --all-features dithering_switch_timings -- --ignored --nocapture --test-threads=1`:
  passed. Debug build, generated 400×240 gray pattern, cached GUI
  tone/dither/CPU texture-update path, one warmup and 20 measured renders each:

  | Algorithm | Mean render/texture update |
  | --- | --- |
  | Threshold | 7.51 ms |
  | Floyd–Steinberg | 10.07 ms |
  | Bayer | 7.72 ms |
  | Atkinson | 12.80 ms |
  | Stucki | 17.97 ms |
  | Blue noise | 7.67 ms |

These are headless CPU timings, not desktop frame/presentation measurements;
they do not establish KDE Wayland interaction responsiveness or display quality.

Next step (requires the user/desktop):

1. Run `just gui` on KDE Wayland. Compare a user-provided/suitably licensed photo,
   generated gradient/gray patches, and text/thin-line art across Floyd–Steinberg,
   Bayer, Atkinson, Stucki and blue noise at 1× and 2×. Record shadow/highlight
   loss, dot texture, grid/directional patterns, tile repetition and switching
   responsiveness. No actual visual observations have been recorded yet.
2. Measure the displayed image with a ruler and compare at approximately
   **6×3.6 cm** (400:240 aspect ratio), not simply the GUI's 1× zoom. Record text
   and thin-line readability, fine dots, apparent midtones, and important detail
   lost with Atkinson. Do not stretch to 6×4 cm or infer centimetres from zoom.
3. Real memory-LCD evaluation remains explicitly handed off to task 006; its
   handoff now includes these algorithms and lighting/distance criteria.
   Hardware compatibility and real-display visual quality remain unverified.
4. After those desktop acceptance items pass, record the observations and move
   this file to `done/` without changing its name.
