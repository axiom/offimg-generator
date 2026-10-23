# R47 OFF-image generator

A personal Linux Rust project to convert images to 400×240, 1-bit BMP files for
the SwissMicros R47, with a CLI and live-preview desktop GUI. The development
shell and shared conversion library are ready; the front ends are next.

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

See [the design](docs/design.md) and [tasks](docs/tasks/README.md). Use `jj` to
commit progress. Keep the two supplied BMP fixtures unchanged.
