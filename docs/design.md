# R47 OFF-image generator

## Scope

A small, Linux-only Rust utility for personal use. Convert images to 400×240,
1-bit BMP OFF-images for the SwissMicros R47, which uses DM42n hardware with
community firmware. Provide both a command-line converter and a simple desktop
GUI with a live preview. Confirm compatibility on an actual R47.
The user describes the target display as a memory LCD approximately 6×4 cm;
evaluate static 1-bit images for readability at that physical size and normal
viewing distance, not only in enlarged desktop previews.

## Architecture

Use one Cargo package: a shared conversion library and two binaries, `offimg`
and `offimg-gui`. Keep GUI dependencies optional behind a `gui` feature so a
CLI-only build does not need the desktop stack. No multi-crate workspace,
plugin system, or async runtime. The GUI uses detached background loading and
crop-preparation threads, with at most one request of each kind per editor/source.
DBS alone also uses a detached, coalescing render worker (one per editor).

Suggested modules: `pipeline`, `dither`, `mono`, and `bmp`. Use `image` for
decoding and resizing, `clap` for CLI parsing, `eframe`/`egui` for the GUI,
and `rfd` with its desktop-portal backend for file dialogs. Check compatible
crate versions and feature names when adding dependencies. Start with PNG,
JPEG, BMP, and WebP input rather than every decoder.

Both front ends share settings, defaults, validation, and conversion code:

1. Decode, apply orientation, and composite transparency onto white.
2. Fit with white margins or crop to 400×240, preserving aspect ratio (centered
   in the CLI, interactive in the GUI).
3. Convert to grayscale.
4. Optionally apply fixed mild output-size sharpening, then brightness, contrast,
   and gamma adjustments.
5. Dither to black/white.
6. Optionally invert.
7. Preview or export the same monochrome buffer.

Keep the oriented, white-composited original input and cache both exact
400×240 Fit/Fill grayscale buffers in the GUI (192,000 bytes total, plus a small
pending crop result). Never apply
adjustments cumulatively to a previous result. Measurements on a 3840×2160 JPEG
in the debug build found full-source resizing cost about 2.4 s per framing,
versus about 9 ms for cached tone/dither/texture updates. Caching avoids those
repeated resizes without a working-image downsize or any quality/CLI change.

One detached standard thread decodes, prepares both modes, and creates the
bounded source thumbnail. A capacity-one channel transfers owned CPU buffers or
an error message; the worker requests a repaint after sending. Only the UI
thread renders the small final buffer (except DBS), creates/updates textures, and replaces
editor state. Open/export are disabled while a request is pending, so requests
cannot accumulate or overtake each other. Controls still update the old preview;
new buffers render with the latest settings, not a stale snapshot. Errors retain
the old image and clear pending state. Closing drops the receiver without joining
the decoder. Decoder limits are unchanged; no full source is cloned to cross the
thread boundary. At most the old source and one replacement coexist, plus the
existing bounded decoder/resampling intermediates. Original dimensions/detail
remain available for crop editing, with coordinates tied to the oriented original.
Preparing both modes lengthens initial loading but removes reframing stalls;
native dialogs and export remain synchronous. The user confirmed that the GUI
no longer freezes on KDE Wayland, completing task 008's desktop check.

## Dithering and controls

Define a small Rust `Dither` trait operating on a grayscale image and returning
a monochrome image whose API enforces black/white pixels. An enum selects the
built-in algorithm. Initial algorithms: threshold, Floyd–Steinberg (default),
and ordered Bayer. No dynamic plugin registry.

Task 011 adds three additional shared CLI/GUI choices: Atkinson for a more
contrasty classic look, Stucki for wider error diffusion, and fixed-mask blue
noise for an organic texture. A single embedded tileable 64×64 CC0 mask from
Christoph Peters' `Data/64_64/LDR_LLL1_0.png` red channel is stored as raw bytes;
provenance, SHA-256 checksums, and license are in `assets/blue-noise/`.
Thresholds are centered at `(byte + 0.5) * 255 / 256` and repeat at fixed output
coordinates, with no runtime generation. Existing algorithm names,
Floyd–Steinberg default, tone pipeline, and output remain unchanged. These
additions received the user's confirmation closing tasks 011–013. Desktop/visual
acceptance is recorded on that confirmation, without inventing detailed per-image
observations or ruler measurements. The user subsequently confirmed generated
images look perfect on the device, completing task 006; firmware version and
detailed per-algorithm hardware observations were not supplied.

Atkinson scans left-to-right/top-to-bottom, sending 1/8 of the error to
`(1,0), (2,0), (-1,1), (0,1), (1,1), (0,2)`, intentionally losing 1/4.
Stucki uses rows `X 8 4 / 2 4 8 4 2 / 1 2 4 2 1`, divided by 42.
Both use floating-point error, white at `>= 127.5`, and discard out-of-bounds
contributions without wrapping or renormalizing. The original Floyd–Steinberg
implementation is unchanged; the additional raster kernels share a private loop.

Following the user's request for all five further choices, task 012 adds full
three-row Sierra (divisor 32), Burkes (32), Jarvis–Judice–Ninke (48), Riemersma,
and 4×4 clustered-dot halftoning. CLI names are `sierra`, `burkes`,
`jarvis-judice-ninke`, `riemersma`, and `clustered-dot`. The raster kernels retain
the same traversal, floating-point error, cutoff and edge conventions as Stucki.
Their rows are respectively `X 5 3 / 2 4 5 4 2 / 0 2 3 2 0`,
`X 8 4 / 2 4 8 4 2`, and `X 7 5 / 3 5 7 5 3 / 1 3 5 3 1`.

Riemersma follows a Hilbert curve over the smallest containing power-of-two
square. The 2×2 order is (0,0), (0,1), (1,1), (1,0); clipped coordinates do not
advance error history. Entire off-image quadrants are pruned so narrow images
do not require scanning a large padded square. The last 16 **source minus output**
errors are weighted by `16^(-age/15)` (newest age 0) and summed without
normalization before thresholding at 127.5. History starts at zero per conversion;
no corrected/accumulated error is fed back. This is the finite-history Riemersma
variant, not simple one-error diffusion along a curve.

Clustered dot uses rows `12 5 6 13 / 4 0 1 7 / 11 3 2 8 / 15 10 9 14`,
centered thresholds `(rank + 0.5) * 255 / 16`, and fixed output-pixel tiling.
The center cluster grows as gray increases, unlike Bayer's dispersed grid.
These eleven algorithms share framing/tone/inversion and preview/export buffers;
no new dependency, tuning control, output format or default change is introduced.

Task 013 follows the user's selection of all four further capabilities plus
optional sharpening. New algorithm names are `sauvola`, `ostromoukhov`, and
`dbs`; `--serpentine` is an opt-in variant of the six fixed-kernel raster
algorithms, not six more radio choices. Odd rows reverse traversal and every
horizontal tap; edge error is still discarded without renormalization. The
original Floyd–Steinberg default path remains unchanged. Unsupported algorithms
ignore the flag; Ostromoukhov always scans serpentine.

Sauvola uses clipped 15×15 local mean/variance from u64 integral sums, fixed
`k=0.2, R=128`, and threshold `mean * (1 + k * (stddev/R - 1))`. White is at or
above threshold, except zero stays black even in a zero-mean window. This targets
documents, not photographic gray. Ostromoukhov uses 128 mirrored integer
coefficient triples indexed by original tone-adjusted gray, no propagated-error
clamping, and the shared 127.5 cutoff. Table provenance, pinned source checksum
and MIT notice are in `assets/ostromoukhov/`; implementation conventions are
explicit rather than promising identical pixels to every other implementation.

DBS starts from the unchanged raster Floyd–Steinberg seed, then greedily chooses
the best improving toggle or opposite-color swap among eight neighbors per pixel
in left-to-right/top-to-bottom order. At most four sweeps (stop early if unchanged)
bound work. The objective is full-convolution squared error after a separable
`[1,2,1]/4` blur, including its halo. Its 5×5 autocorrelation updates the error
gradient incrementally; f64 calculations accept changes below -1e-12. The method
is deterministic and approximate, not a globally optimal or LCD-calibrated model.

Optional `--sharpen` / **Mild sharpening** applies
`round(clamp(gray + 0.5 * (gray - blur(gray)), 0, 255))` with the same 3×3
binomial blur and clamped borders, on fresh output-size grayscale **before** tone.
The amount/radius are fixed; sharpening and serpentine default off and reset off.
Both front ends share the exact settings/pipeline; neither changes source caches.

Only DBS rendering moves off-thread: one capacity-one channel, monotonically
increasing render revision, one in-flight request and a boolean latest-request
marker, never a queue. Settings edits, accepted crop buffers and source installs
advance the revision. Obsolete results cannot replace current pixels, including
after switching algorithms or replacing a source. A worker owns one 400×240
grayscale snapshot and settings, not the full source. The UI retains the last
displayed monochrome buffer; new DBS sources first display their fast
Floyd–Steinberg seed. **Optimizing DBS…** and **Updating crop…** may both apply;
export always writes the displayed result. Polling schedules repaints every
50 ms while a worker is pending; close drops the receiver without joining.
Worker errors retain displayed pixels and are surfaced without endless retries.

Controls: framing, algorithm, brightness, contrast, gamma (neutral 1.0),
threshold for the threshold algorithm, inversion, optional serpentine/sharpening,
and reset. Specify numeric
ranges and adjustment formulas in the shared core, not independently in each
front end. Other controls default to neutral settings; default framing is fit.

Shared numeric ranges are brightness −1..1 (default 0), contrast 0..4
(default 1), gamma 0.1..4 (default 1), and threshold 0..255 (default 128).
All floating-point controls must be finite. For grayscale `s` normalized to
0..1, compute `t = clamp((s - 0.5) * contrast + 0.5 + brightness, 0, 1)`;
the adjusted byte is `round(255 * t^(1/gamma))`. A 256-entry lookup table
applies this without mutating the cached grayscale. Threshold uses `>= cutoff`
as white. Floyd–Steinberg scans left-to-right, top-to-bottom, with weights
7/16 right, 3/16 down-left, 5/16 down, 1/16 down-right, discarding error
outside image edges. Ordered Bayer uses a repeating 4×4 matrix with centered
thresholds `(entry + 0.5)/16`. Inversion swaps output pixels after dithering.

The GUI shows source and result with nearest-neighbor preview at integer zoom
levels (1×, 2×, 3×). Export must match the current preview pixel-for-pixel.
The CLI refuses to overwrite output without `--force`. The supported KDE
Wayland portal save dialog confirms replacement; the GUI exports an accepted
selection directly, without a second in-app prompt. Cancellation writes
nothing. Existing selections allow atomic replacement; new destinations retain
no-clobber writes. Source paths and their symlink/hardlink aliases remain
rejected even after native approval. Other Linux portals are not verified.
The user confirmed task 007's single native confirmation flow on KDE Wayland.

Fill now supports GUI-only crop editing: a fixed 400:240 frame, drag-to-pan,
1×–16× mouse-wheel crop zoom, without zoom/pan sliders (the user's preferred
interaction after desktop testing). The focusable crop frame retains keyboard
access: arrow keys pan the image, +/− zoom.
Shared `Crop` math validates finite controls, clamps the normalized center inside
the oriented original, and maps frame-relative drags to source coordinates.
Whole-pixel crop dimensions/origins preserve the legacy centered CLI output;
each axis stays at least one pixel for tiny/extreme-aspect inputs. Selection
precedes Lanczos3 resizing. The thumbnail supplies only a UV-mapped framing guide,
not conversion pixels; the monochrome preview remains the authoritative result.
Fit retains white margins. Crop zoom is independent of preview magnification;
Fit/Fill retains selection and tone, while new images/reset restore centered 1×.
CLI flags, settings and defaults are unchanged.

Each loaded source shares its original with a detached crop worker through `Arc`,
without copying its pixels. Only one crop request runs at a time; further edits
replace desired UI state, never append to a queue. After completion, mismatched
selections are discarded and the latest desired selection is started. Replacing
the source drops its old receiver, preventing old-source installation. Returning
to Fit does not start crop work. Current tone/dither settings render an accepted
grayscale result on the UI thread. While pending, keep the previous result and
label it **Updating crop…**; export still writes exactly that displayed buffer.
On a generated 3840×2160 RGB PNG in the debug build, crop preparation measured
2.42 s / 692 ms / 62 ms at 1× / 2× / 16×; 200 request-coalescing calls while the
worker ran took 6.3 µs (not an end-to-end UI timing). Headless egui tests exercise
drag/wheel handling without page scrolling and focused-frame keyboard adjustment.
The user confirmed cropping/zooming, the updated slider-free UI, and responsiveness
with a large image, completing task 009's desktop checks. Exact image dimensions
were not recorded; calculator hardware compatibility was later accepted by the
user in task 006.

The source/crop editor is stacked above the monochrome export preview to better
fit the landscape images within the window. The controls sidebar stays separate;
both preview axes scroll on overflow without shrinking images to fit.
Display magnification defaults to 2× on startup and reset, retaining 1×/2×/3×
choices independently of crop zoom (which resets to centered 1×).
The user confirmed task 010's KDE Wayland visual check, completing desktop acceptance.

## BMP findings

The personal `magic-numbers.bmp` image in `tests/fixtures/` serves as the BMP
format fixture. Header inspection found:

| Field | Value in fixture |
| --- | --- |
| Dimensions | 400×240 |
| Bits per pixel | 1 |
| Compression | BI_RGB (uncompressed) |
| DIB header | 108-byte BITMAPV4HEADER |
| Pixel offset | 130 bytes |
| Row layout | Bottom-up; 50 pixel bytes + 2 padding bytes |
| Pixel data size | 12,480 bytes |
| Total size | 12,610 bytes |

Interpret pixel values through the palette, not a hardcoded bit polarity. Pixels
within each byte use the standard BMP most-significant-bit first order.
Resolution metadata differs and is not the image dimensions.

Initially export the same V4 structure with a consistent black/white palette,
standard bit order, bottom-up rows, and zeroed row padding. Inspect the rest of
the sample V4 fields before implementing the writer. A visually monochrome
24-bit BMP is not acceptable. Use an existing encoder only if it can produce
the required structure; otherwise implement a small dedicated writer.

Full inspection in task 002 found zero file reserved words, planes = 1, colors
used/important = 2, and zero compression. The value `0x73524742` is at file offset 54 (the red
mask), **not** at the V4 color-space field (offset 70, zero). The last CIEXYZ
endpoint component at offset 106 is 2; other masks, endpoints, and gamma
fields are zero. These are unusual but unused for 1-bit BI_RGB. The dedicated
writer preserves them, now accepted in user hardware validation, with 3780 pixels/metre
resolution, a black/white palette, and zero row padding.

Input limits are 16,384 pixels per side, 32 million pixels total, and a
256 MiB decoder allocation budget (the allocation budget is best-effort in
`image`; dimension and pixel-count limits are checked before pixel decoding).
Fit uses centered white margins and crop removes edges symmetrically before
Lanczos3 resizing, avoiding large intermediates on extreme aspect ratios.

Source: https://technical.swissmicros.com/dm42/doc/dm42_user_manual.html#off_images
documents 400×240, 1-bit BMP files in `/OFFIMG/` for DM42/DM42n. This does not
alone prove that a generated file works with the R47 community firmware; the
user's successful device check is recorded in completed task 006.

## Development

Use the repository `flake.nix` and commit `flake.lock`; target the current
machine, x86_64 Linux. Start OpenCode from `nix develop` after environment setup.
Alternatively wrap commands with `nix develop --command ...`. The shell provides
Rust tools and the Wayland/X11/OpenGL runtime libraries for the intended GUI.
The host desktop must provide a working graphics driver and desktop portal for
native file dialogs. Test those once the GUI exists; do not change global NixOS
configuration as part of this project without asking.

The GUI uses optional eframe/egui 0.33.3 (compatible with the declared Rust
1.88 minimum), Glow with Wayland/X11 and accessibility, and rfd 0.17.2 with
the desktop-portal backend. The shell also supplies libdbus for rfd's dynamic
loading. Source thumbnails are bounded to 400×240, while result textures use
the exact export buffer. Integer zoom is measured in physical pixels so
desktop UI scaling does not introduce fractional preview scaling.

The development shell includes `just`. The `justfile` wraps common commands
in `nix develop --command`, so `just gui`, `just convert INPUT OUTPUT`, and
`just check` do not require remembering Cargo flags. Arguments are passed
positionally to preserve spaces and shell characters in image paths.

Once the Cargo package exists:

```sh
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo run --bin offimg -- convert input.jpg output.bmp
cargo run --features gui --bin offimg-gui
```

Defer saved presets, batch conversion, automatic calculator copying,
cross-platform releases, and Nix application packaging. Interactive cropping
is now in scope for task 009, after native-only confirmation and responsive
loading (007–008). Follow with vertical preview layout (010) before hardware
validation 006 in the normal priority order.
