# 001 — Development environment

## Goal

A locked repository Nix development shell usable by the next OpenCode session.
No Cargo application scaffolding is required for this task.

## Scope

Provide Cargo, rustc, rustfmt, Clippy, rust-analyzer, Rust library sources,
compiler/linker tooling, pkg-config, and intended GUI runtime libraries.
Target x86_64 Linux only. Do not add a Rust overlay or package the application.

## Acceptance

- [x] `flake.nix` evaluates and `flake.lock` pins nixpkgs.
- [x] `nix develop --command ...` runs the compiler, Cargo, formatter, Clippy,
      rust-analyzer, pkg-config, and C compiler/linker tools.
- [x] Rust sources and configured GUI library paths exist inside the shell.
- [x] Restart instructions and GUI runtime limitations are documented.

## Progress / handoff

Completed 2026-10-23. Validated `nix develop --command ...` with Rust/Cargo 1.99.0,
rustfmt 1.10.0, Clippy 0.1.99, rust-analyzer 2026-10-12, pkg-config 0.29.2,
GCC 16.2.0, and GNU ld 2.46. Checked Rust sources and all configured runtime
library directories exist. `pkg-config --exists` passes for wayland-client,
xkbcommon, x11, xcursor, xi, xrandr, and gl.

Next: restart inside the shell and begin task 002. A real GUI launch is
deliberately deferred to task 005, when the application exists. No Rust package
or application tests exist yet.
