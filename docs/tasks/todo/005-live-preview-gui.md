# 005 — Live-preview GUI

Depends on: 004.

## Goal

A simple Linux desktop editor whose exported image matches the preview.

## Scope

Add optional eframe/egui and portal-backed rfd dependencies, and an
`offimg-gui` binary requiring the `gui` feature. Provide open/export controls,
source/result views, framing/algorithm selectors, sliders, invert, and reset.
Use integer nearest-neighbor preview zoom. Cache resized grayscale input and
only recalculate when relevant settings change. Keep errors visible and retain
the previous valid input after failed loads.

## Acceptance

- [x] GUI builds and opens from `nix develop` on the user's Linux desktop.
- [x] Open/save dialogs work with the host desktop portal.
- [x] Controls update the preview live without cumulative image degradation.
- [x] Preview pixels and exported BMP pixels match in an automated check.
- [x] Defaults/ranges match CLI settings; reset restores those defaults.
- [ ] Existing output requires confirmation; source images stay protected.
- [x] Decode/export errors are shown without crashing or losing the preview.
- [x] CLI-only build still works without GUI dependencies enabled.
- [x] Formatting, tests, and all-feature Clippy pass.

## Progress / handoff

Implemented optional eframe 0.33.3 (Rust 1.88 compatible),
Glow/Wayland/X11/accessibility, and portal-backed rfd 0.17.2. Added libdbus to
the Nix GUI runtime path because this rfd version loads it dynamically.

The editor has open/export, all shared settings and ranges, reset, nearest
integer physical-pixel zoom, source/result views, cached grayscale, visible
errors, failed-load preservation, and modal overwrite confirmation. It exports
the actual preview buffer through the CLI's atomic/source-safe helper.
Added a headless GUI-state test for preview/export agreement, cache reuse,
reset, and failure preservation, including a timing observation.

Verified `nix develop --command cargo test --all-features` (22 tests),
`cargo test --no-default-features` (21 tests), `cargo fmt --check`, and
`cargo clippy --all-targets --all-features -- -D warnings`. The no-default
`cargo tree -e normal` contains no GUI dependencies. A cached debug-build
tone/dither/texture update measured about 16 ms; no workers needed.

Launched from `nix develop` on KDE Wayland (`wayland-0`, X11 also available).
The user confirmed the window, dialogs, and controls worked, but did not try
replacing existing output. Source protection and failed-export preservation
are automated-tested. In response to visual feedback, replaced the cramped
toolbar controls with a spaced sidebar, larger action buttons, and preview
cards; relaunched for a second look.

Next: ask the user to export twice to a disposable destination and verify
Cancel/Replace. This is the only unchecked item. Then move this task to done
and proceed to task 006; do not claim R47 hardware compatibility yet.
