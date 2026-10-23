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
just cli convert --help
```

Fit (the default) preserves the whole image with white margins; crop fills the
screen from the center. Floyd–Steinberg is the default dither; `threshold` and
4×4 ordered `bayer` are also available. Conversion
runs without a display server. Existing output needs `--force`; the source,
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
BMP…**. The source is shown as a scaled thumbnail; the result shows the exact
400×240 export buffer. Both previews use nearest-neighbor textures at 1×, 2×,
or 3× physical-pixel zoom (scroll for larger previews). **Reset controls** restores
the same defaults as the CLI. Failed loads keep the previous image; errors stay
visible. On the supported KDE Wayland desktop portal, existing destinations
require only the native save dialog's replacement confirmation; there is no
second in-app prompt. Cancellation writes nothing, and the source image and
its symlink/hardlink aliases remain protected even after native approval.
Other Linux portal backends have not been verified. No calculator files are
copied automatically.

The optional `gui` feature uses egui/eframe with Glow, Wayland/X11 and
accessibility support, plus rfd's desktop-portal file dialogs. If a dialog does
not appear, check the host session's desktop portal; cancellation or unavailable
dialogs leave the current image untouched. CLI-only builds do not enable any
GUI dependencies.

See [the design](docs/design.md) and [tasks](docs/tasks/README.md). Use `jj` to
commit progress. Keep the two supplied BMP fixtures unchanged.
