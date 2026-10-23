# 010 — Stack source and preview vertically

Depends on: 005. Scheduled after 009, but may be implemented independently.

## Goal

Use available window width better by placing the original above the monochrome
preview instead of side by side: both images are wider than they are tall.

## Scope

Change the preview area in `src/gui.rs` to a vertical stack, with the source
(or crop editor from task 009) above the export preview. Keep clear labels,
the controls sidebar, and existing integer nearest-neighbor preview zoom.
Allow scrolling when the stack exceeds the window; do not shrink images with
fractional scaling just to fit. No layout selector or automatic switching.

This is a presentation change only: preserve crop interaction, settings,
source data, and pixel-for-pixel preview/export agreement.

## Acceptance

- [ ] Source/crop editor appears above the monochrome export preview.
- [ ] Both views remain reachable by scrolling in a smaller window and at
      1×/2×/3× preview zoom, without clipping controls or distorting images.
- [ ] Crop dragging/wheel zoom still works without unintended page scrolling
      when task 009 is present.
- [ ] Visually checked on KDE Wayland; `just check` passes.

## Progress / handoff

Not started. Planning only. The current preview cards are arranged using
`ui.horizontal_top` inside `egui::ScrollArea::both()` in `src/gui.rs`.
