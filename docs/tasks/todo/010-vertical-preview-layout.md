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

- [ ] Source/crop editor appears above the monochrome export preview.
- [ ] Startup and Reset controls select 2× preview magnification; 1× and 3×
      remain selectable, and preview zoom never changes exported pixels.
- [ ] Both views remain reachable by scrolling in a smaller window and at
      1×/2×/3× preview zoom, without clipping controls or distorting images.
- [ ] Crop dragging/wheel zoom still works without unintended page scrolling
      when task 009 is present.
- [ ] Visually checked on KDE Wayland; `just check` passes.

## Progress / handoff

Not started. Planning only. The current preview cards are arranged using
`ui.horizontal_top` inside `egui::ScrollArea::both()` in `src/gui.rs`.
The user also requested a 2× default preview: update both `Editor::default`
and Reset controls (currently 1×), cover startup/reset in GUI-state tests,
and update the GUI usage documentation when implemented.
