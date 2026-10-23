# 010 — Stack source and preview vertically

Depends on: 005. Scheduled after 009, but may be implemented independently.

## Goal

Use available window width better by placing the original above the monochrome
preview instead of side by side: both images are wider than they are tall.

## Scope

Change the preview area in `src/gui.rs` to a vertical stack, with the source
(or crop editor from task 009) above the export preview. Keep clear labels,
the controls sidebar, and existing integer nearest-neighbor preview zoom.
Default preview magnification to **2×** on startup and when Reset controls is
used, while retaining the 1×/2×/3× choices. This is display magnification only,
not task 009's crop zoom; centered crop zoom still resets to 1×.
Allow scrolling when the stack exceeds the window; do not shrink images with
fractional scaling just to fit. No layout selector or automatic switching.

This is a presentation change only: preserve crop interaction, settings,
source data, and pixel-for-pixel preview/export agreement.

## Acceptance

- [x] Source/crop editor appears above the monochrome export preview.
- [x] Startup and Reset controls select 2× preview magnification; 1× and 3×
      remain selectable, and preview zoom never changes exported pixels.
- [x] Both views remain reachable by scrolling in a smaller window and at
      1×/2×/3× preview zoom, without clipping controls or distorting images.
- [x] Crop dragging/wheel zoom still works without unintended page scrolling
      when task 009 is present.
- [ ] Visually checked on KDE Wayland; `just check` passes.

## Progress / handoff

Implemented in `src/gui.rs`: vertically stacked labeled cards inside the existing
two-axis scroll area, with the controls sidebar unchanged. Startup and Reset
controls now use 2× display magnification; crop reset remains centered at 1×.
Updated GUI usage and design documentation.

Verified with headless egui tests: source above result at 1×/2×/3×, both Fit and
Fill, and 1.0/1.5 desktop scale; exact display dimensions; horizontal/vertical
overflow scrolling in a 300×200 view; unchanged preview/export pixels. The crop
drag/wheel regression now exercises the actual stacked view and checks that
only wheel events outside the crop frame scroll the page. Existing GUI-state
tests cover 2× startup/reset and independent crop reset.

Commands run: `nix develop --command cargo test --all-features gui::tests::`,
`nix develop --command cargo fmt`, and `just check` (passed: formatting, Clippy,
33 tests; two existing manual timing tests ignored).

Remaining: run `just gui` on KDE Wayland and visually check both images and the
sidebar in a smaller window at 1×/2×/3×, including scrolling and Fill drag/wheel
interaction. Headless tests do not replace that desktop check. Once confirmed,
check the final acceptance item and move this task to `done/`.
