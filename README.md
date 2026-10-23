# R47 OFF-image generator

A personal Linux Rust project to convert images to 400×240, 1-bit BMP files for
the SwissMicros R47, with a CLI and live-preview desktop GUI. The development
shell, shared conversion library, and command-line converter are ready. The GUI
is not implemented yet.

## Development

```sh
nix develop
opencode
```

Exit the current OpenCode session before restarting it from the shell above.
Then ask: “Read `docs/design.md` and `docs/tasks/README.md`, then continue the
first unfinished task.”

The locked shell targets x86_64 Linux and supplies Rust, Cargo, rustfmt, Clippy,
rust-analyzer, compiler/linker tooling, and Wayland/X11/OpenGL libraries.
The eventual GUI also needs working host graphics drivers and a desktop portal
for file dialogs; neither has been exercised by an application yet.

Run library tests and checks with:

```sh
nix develop --command cargo test
nix develop --command cargo fmt --check
nix develop --command cargo clippy --all-targets --all-features -- -D warnings
```

The core accepts PNG, JPEG, BMP, and WebP, applies orientation and white alpha
compositing, and exports genuine 400×240, 1-bit BMP files. R47 hardware
compatibility has not been verified.

## Command line

```sh
nix develop --command cargo run --bin offimg -- convert photo.jpg off-image.bmp
nix develop --command cargo run --bin offimg -- convert photo.png off-image.bmp --fit crop --dither threshold --force
nix develop --command cargo run --bin offimg -- convert --help
```

Fit (the default) preserves the whole image with white margins; crop fills the
screen from the center. Threshold dithering is the initial algorithm. Conversion
runs without a display server. Existing output needs `--force`; the source,
including symlink and hardlink aliases, is always protected. Exports are written
to a temporary file in the destination directory and atomically installed, so
conversion/write failures do not truncate existing images. Temporary files are
cleaned up on errors; replacing a symlink replaces the link, not its target.

See [the design](docs/design.md) and [tasks](docs/tasks/README.md). Use `jj` to
commit progress. Keep the two supplied BMP fixtures unchanged.
