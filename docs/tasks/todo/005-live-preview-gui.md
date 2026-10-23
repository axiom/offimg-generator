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

- [ ] GUI builds and opens from `nix develop` on the user's Linux desktop.
- [ ] Open/save dialogs work with the host desktop portal.
- [ ] Controls update the preview live without cumulative image degradation.
- [ ] Preview pixels and exported BMP pixels match in an automated check.
- [ ] Defaults/ranges match CLI settings; reset restores those defaults.
- [ ] Existing output requires confirmation; source images stay protected.
- [ ] Decode/export errors are shown without crashing or losing the preview.
- [ ] CLI-only build still works without GUI dependencies enabled.
- [ ] Formatting, tests, and all-feature Clippy pass.

## Progress / handoff

Not started. Record tested display backend and any host portal/graphics issues.
Measure responsiveness before adding workers or other concurrency.
