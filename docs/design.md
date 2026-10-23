# R47 OFF-image generator

## Scope

A small, Linux-only Rust utility for personal use. Convert images to 400×240,
1-bit BMP OFF-images for the SwissMicros R47, which uses DM42n hardware with
community firmware. Provide both a command-line converter and a simple desktop
GUI with a live preview. Confirm compatibility on an actual R47.

## Architecture

Use one Cargo package: a shared conversion library and two binaries, `offimg`
and `offimg-gui`. Keep GUI dependencies optional behind a `gui` feature so a
CLI-only build does not need the desktop stack. No multi-crate workspace,
plugin system, or background workers initially.

Suggested modules: `pipeline`, `dither`, `mono`, and `bmp`. Use `image` for
decoding and resizing, `clap` for CLI parsing, `eframe`/`egui` for the GUI,
and `rfd` with its desktop-portal backend for file dialogs. Check compatible
crate versions and feature names when adding dependencies. Start with PNG,
JPEG, BMP, and WebP input rather than every decoder.

Both front ends share settings, defaults, validation, and conversion code:

1. Decode, apply orientation, and composite transparency onto white.
2. Fit with white margins or center-crop to 400×240, preserving aspect ratio.
3. Convert to grayscale.
4. Apply brightness, contrast, and gamma adjustments.
5. Dither to black/white.
6. Optionally invert.
7. Preview or export the same monochrome buffer.

Keep the original input and cache the resized grayscale image. Never apply
adjustments cumulatively to a previous result. Start with processing on the
UI thread at this small output size; measure before introducing concurrency.
Enforce input decoder limits and retain the current image if loading fails.
The user has since observed stalls while loading images: task 008 will move
expensive loading/preparation off the UI thread, without assuming every cached
400×240 tone update needs background processing. This follow-up is not yet
implemented.

## Dithering and controls

Define a small Rust `Dither` trait operating on a grayscale image and returning
a monochrome image whose API enforces black/white pixels. An enum selects the
built-in algorithm. Initial algorithms: threshold, Floyd–Steinberg (default),
and ordered Bayer. No dynamic plugin registry.

Controls: framing, algorithm, brightness, contrast, gamma (neutral 1.0),
threshold for the threshold algorithm, inversion, and reset. Specify numeric
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
The CLI refuses to overwrite output without `--force`. The native KDE portal
save dialog already confirms replacement; task 007 will remove the redundant
in-app confirmation (currently both are shown). Never overwrite the source
image accidentally: identity checks and atomic writes remain mandatory even
after native approval.

Task 009 will add GUI-only crop editing: a fixed 400:240 frame, drag-to-pan,
slider/wheel crop zoom, and keyboard alternatives. Keep the selection inside
the oriented original; existing Fit mode retains white margins. Extend Fill
mode, starting at centered 1× crop, and distinguish crop zoom from preview
magnification. CLI flags/defaults stay unchanged. Retain selection when toggling
Fit/Fill; new images and reset restore centered 1×. These controls are planned,
not implemented; use shared crop math and preserve preview/export agreement.

Task 010 will stack the source/crop editor above the monochrome export preview,
rather than side by side, to better fit the landscape images within the window.
Keep the controls sidebar, integer preview zoom, and scrolling for overflow.
This layout change is planned, not implemented.

## BMP findings

The supplied `numbers.bmp` and `numbers2.bmp` remain unchanged in the repository
root and serve as format fixtures. Header inspection found:

| Field | Value in both files |
| --- | --- |
| Dimensions | 400×240 |
| Bits per pixel | 1 |
| Compression | BI_RGB (uncompressed) |
| DIB header | 108-byte BITMAPV4HEADER |
| Pixel offset | 130 bytes |
| Row layout | Bottom-up; 50 pixel bytes + 2 padding bytes |
| Pixel data size | 12,480 bytes |
| Total size | 12,610 bytes |

`numbers.bmp` has palette index 0 black, index 1 white. `numbers2.bmp` reverses
that palette. Interpret pixel values through the palette, not a hardcoded bit
polarity. Pixels within each byte use the standard BMP most-significant-bit
first order. Resolution metadata differs and is not the image dimensions.

Initially export the same V4 structure with a consistent black/white palette,
standard bit order, bottom-up rows, and zeroed row padding. Inspect the rest of
the sample V4 fields before implementing the writer. A visually monochrome
24-bit BMP is not acceptable. Use an existing encoder only if it can produce
the required structure; otherwise implement a small dedicated writer.

Full inspection in task 002 found identical reserved/unused V4 fields in both
samples: zero file reserved words, planes = 1, colors used/important = 2,
and zero compression. The value `0x73524742` is at file offset 54 (the red
mask), **not** at the V4 color-space field (offset 70, zero). The last CIEXYZ
endpoint component at offset 106 is 2; other masks, endpoints, and gamma
fields are zero. These are unusual but unused for 1-bit BI_RGB. The dedicated
writer preserves them pending hardware validation, with 3780 pixels/metre
resolution, a black/white palette, and zero row padding.

Input limits are 16,384 pixels per side, 32 million pixels total, and a
256 MiB decoder allocation budget (the allocation budget is best-effort in
`image`; dimension and pixel-count limits are checked before pixel decoding).
Fit uses centered white margins and crop removes edges symmetrically before
Lanczos3 resizing, avoiding large intermediates on extreme aspect ratios.

Source: https://technical.swissmicros.com/dm42/doc/dm42_user_manual.html#off_images
documents 400×240, 1-bit BMP files in `/OFFIMG/` for DM42/DM42n. This does not
prove that a generated file works with the R47 community firmware.

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
