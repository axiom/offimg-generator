# R47 OFF-image generator

A personal Linux Rust project to convert images to 400×240, 1-bit BMP files for
the SwissMicros R47, with a CLI and live-preview desktop GUI. The development
shell, shared conversion library, command-line converter, and optional desktop
GUI are implemented. Desktop and hardware verification status is in the tasks.

## Development

```sh
nix develop
just        # list available commands
just gui    # open the desktop editor
```

The `justfile` wraps the Rust commands below; every recipe uses the locked Nix
environment. Start with `nix develop` to get `just`, or use
`nix develop --command just <recipe>` without entering a shell. Direnv users
can run `direnv allow` after changes to the development environment.

The locked shell targets x86_64 Linux and supplies just, Rust, Cargo, rustfmt, Clippy,
rust-analyzer, compiler/linker tooling, and Wayland/X11/OpenGL libraries.
The GUI also needs working host graphics drivers and a desktop portal for file
dialogs. The shell provides the runtime libraries; it does not configure your
desktop or calculator.

Common development commands:

```sh
just check  # formatting, Clippy, and all tests
just test   # all tests, without opening a window
just fmt    # apply Rust formatting
just build  # build the CLI without GUI dependencies
```

The core accepts PNG, JPEG, BMP, and WebP, applies orientation and white alpha
compositing, and exports genuine 400×240, 1-bit BMP files. R47 hardware
compatibility has not been verified.

## Command line

```sh
just convert photo.jpg off-image.bmp
just convert photo.png off-image.bmp --fit crop --dither bayer --gamma 1.4 --force
just convert photo.jpg off-image.bmp --dither blue-noise
just convert photo.jpg off-image.bmp --dither riemersma
just convert scan.png off-image.bmp --dither sauvola --sharpen
just convert photo.jpg off-image.bmp --serpentine --sharpen
just convert photo.jpg off-image.bmp --dither dbs
just cli convert --help
```

Fit (the default) preserves the whole image with white margins; crop fills the
screen from the center. The CLI and GUI share these dithering choices:

| `--dither` | GUI label | Look / tradeoff |
| --- | --- | --- |
| `floyd-steinberg` (default) | Floyd–Steinberg | Fine error diffusion; can show directional patterns. |
| `threshold` | Threshold | Hard cutoff; useful for text/line art, loses apparent gray. |
| `bayer` | Ordered Bayer | Repeating 4×4 ordered grid; regular texture is visible. |
| `atkinson` | Atkinson | Contrasty classic look; discards a quarter of the error, so faint shadows/highlights can disappear. |
| `stucki` | Stucki | Wider error diffusion for photos/gradients; more work, not always better. |
| `blue-noise` | Blue noise | Organic dispersed dots from a fixed 64×64 tile; repetition can still be visible. |
| `sierra` | Sierra | Full three-row diffusion; a different balance of texture and smooth gradients. |
| `burkes` | Burkes | Two-row diffusion; lighter-weight alternative to Stucki/JJN. |
| `jarvis-judice-ninke` | Jarvis–Judice–Ninke | Wide three-row diffusion for photos; subtle differences from Stucki, more work per pixel. |
| `riemersma` | Riemersma | Hilbert-curve diffusion with a short decaying error history; less raster-direction bias, coarser texture and more work. |
| `clustered-dot` | Clustered dot | Repeating 4×4 growing dot clusters for a newspaper-like texture; can obscure fine detail. |
| `sauvola` | Sauvola · local threshold | Adaptive threshold for text/diagrams with uneven lighting; removes apparent gray and can amplify noise. |
| `ostromoukhov` | Ostromoukhov | Gray-dependent diffusion weights and serpentine scan for photos/gradients; different texture, not always better. |
| `dbs` | DBS · optimized (slower) | Direct binary search improves a blurred-error objective using dot toggles/swaps; slower and can soften fine detail. |

Blue noise uses an embedded [CC0 mask by Christoph Peters](assets/blue-noise/README.md),
with no runtime randomness or downloads. All dithering is static, anchored to
export pixels; preview zoom does not affect it. These are alternative looks,
not verified improvements on the R47's approximately 6×4 cm memory LCD. The user
confirmed desktop acceptance of tasks 011–013; real hardware evaluation remains
pending in task 006.
Riemersma uses a fixed 16-error history (newest weight 1, oldest 1/16), with
off-image Hilbert coordinates skipped. Clustered dot is spatial halftoning,
not an animation. No algorithm adds tuning controls or changes the tone pipeline.

Sauvola uses clipped 15×15 windows, `k=0.2`, and `R=128`; solid black/white stay
unchanged. Ostromoukhov uses a fixed [coefficient table with provenance and MIT
notice](assets/ostromoukhov/README.md) and always scans serpentine. DBS starts
from Floyd–Steinberg and runs at most four deterministic optimization sweeps
using a fixed 3×3 perceptual blur. This is a bounded approximation, not a global
optimum or a calibrated model of the calculator's display.

Conversion runs without a display server. Existing output needs `--force`; the source,
including symlink and hardlink aliases, is always protected. Exports are written
to a temporary file in the destination directory and atomically installed, so
conversion/write failures do not truncate existing images. Temporary files are
cleaned up on errors; replacing a symlink replaces the link, not its target.

Controls (shared defaults and validation):

| Flag | Range | Default |
| --- | --- | --- |
| `--brightness` | −1 to 1 | 0 |
| `--contrast` | 0 to 4 | 1 |
| `--gamma` | 0.1 to 4 | 1 |
| `--threshold` | 0 to 255; threshold algorithm only | 128 |
| `--invert` | swap output black/white | off |
| `--serpentine` | alternate scan direction for fixed-kernel raster diffusion | off |
| `--sharpen` | fixed mild output-size sharpening before tone/dithering | off |

**Serpentine scan** works with Floyd–Steinberg, Atkinson, Stucki, Sierra, Burkes,
and Jarvis–Judice–Ninke: odd rows run right-to-left with mirrored weights. Other
choices ignore the flag; Ostromoukhov already scans this way. The GUI disables
the checkbox where it does not apply while retaining its value.

**Mild sharpening** uses a fixed 3×3 binomial blur and unsharp amount 0.5, with
clamped borders and byte rounding/clamping. It runs on fresh output-resolution
grayscale before brightness/contrast/gamma. It may help thin details but can
amplify noise/halos; it never changes framing, resolution or the cached source.

On normalized grayscale `s`, tone adjustment is
`255 × clamp((s − 0.5) × contrast + 0.5 + brightness, 0, 1)^(1/gamma)`,
rounded to the nearest byte. Gamma greater than 1 brightens. Dithering follows
tone adjustment; inversion is last. Every preview/conversion starts from fresh
grayscale, never from a previously adjusted result. Non-finite values are rejected.

## Desktop GUI

```sh
just gui
```

Use **Open…** to select an image, adjust framing/dithering/tone, then **Export
BMP…**. **Fit** shows the whole source with white margins. **Fill** uses a fixed
400:240 frame: drag the image behind it to pan and use the mouse wheel to zoom
(1×–16×). There are no crop zoom/pan sliders. Wheel scrolling zooms only over the
crop frame; elsewhere it scrolls normally. For keyboard access, Tab to the frame:
arrow keys pan the image and +/− zoom. At crop zoom 1× the selection starts
as the largest centered crop; zoom keeps its center unless clamped at an edge.
Selections stay inside the oriented original, rounded to whole source pixels
(at least one per axis, even for tiny images).

The source/crop editor sits above the export preview and uses a thumbnail as a
framing guide; the result
shows the exact 400×240 export buffer, resampled from the full-resolution original.
**Preview zoom** (1×/2×/3× physical pixels, nearest-neighbor) only magnifies the
views; it defaults to 2× and never changes exported content. Scroll vertically
or horizontally to reach views larger than the window; images do not shrink to fit.
Fit/Fill toggling retains the crop and tone settings. A new image centers crop
zoom at 1×; **Reset controls** also restores the CLI's tone/framing defaults and
2× preview magnification. Failed loads keep the previous image; errors stay
visible. On the supported KDE Wayland desktop portal, existing destinations
require only the native save dialog's replacement confirmation; there is no
second in-app prompt. Cancellation writes nothing, and the source image and
its symlink/hardlink aliases remain protected even after native approval.
Other Linux portal backends have not been verified. No calculator files are
copied automatically.

Image decoding, full-resolution framing, and thumbnail preparation run in one
background load, with a loading indicator and the previous preview retained.
Open/export are disabled during loading; tone and framing controls remain live,
and a successful load uses their latest values. Fit/Fill switches between
cached grayscale buffers; tone/dither changes do not resample the original.
Crop changes also run off-thread, with one preparation at a time and no request
queue: drag updates coalesce to the latest selection, and obsolete results are
discarded. **Updating crop…** means the monochrome result still shows the last
completed selection. Export always saves that displayed result, not a pending
crop. The original is
retained at full resolution; no extra downsizing changes image quality or CLI
output. Preparing both modes can make the initial wait longer, but closing the
window does not wait for the loader. The user confirmed that the GUI no longer
freezes on KDE Wayland (task 008).

DBS optimization also runs off-thread. **Optimizing DBS…** retains the last
displayed result; export saves those exact pixels, not a pending optimization.
When a new image loads with DBS selected, its initial displayed result is the
Floyd–Steinberg starting image until optimization completes. There is at most
one DBS worker, even across image replacements; edits coalesce to the latest
settings/crop and stale results are discarded. Other algorithms remain immediate.
Worker results are polled at 50 ms intervals while pending; closing does not
wait for optimization. In the debug build, a generated 400×240 gray-pattern
render/texture update averaged about 172 ms for DBS and 16–17 ms for
Sauvola/Ostromoukhov (headless CPU measurements, not desktop frame timings).

The optional `gui` feature uses egui/eframe with Glow, Wayland/X11 and
accessibility support, plus rfd's desktop-portal file dialogs. If a dialog does
not appear, check the host session's desktop portal; cancellation or unavailable
dialogs leave the current image untouched. CLI-only builds do not enable any
GUI dependencies.

See [the design](docs/design.md) and [tasks](docs/tasks/README.md). Use `jj` to
commit progress. Keep the two supplied BMP fixtures unchanged.
