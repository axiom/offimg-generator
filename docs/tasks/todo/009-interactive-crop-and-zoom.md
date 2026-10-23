# 009 — Interactive image crop and zoom

Depends on: 008. Do before hardware validation 006 in the normal work order.

## Goal

Let the user choose a portion of the original image for the exported OFF screen,
instead of being limited to fit or a centered crop.

## Confirmed interaction

- GUI-only initially; no new CLI flags or changed CLI defaults.
- A fixed **400:240** crop frame, with the image dragged behind it to pan.
- A zoom slider and mouse-wheel zoom over the crop editor.
- Keep the crop entirely inside the source; no new blank margins in crop mode.
- Existing Fit mode still shows the whole image with white margins.

## Scope

Extend the GUI's existing Fill/crop mode rather than adding a separate editing
application or arbitrary-aspect rectangle tool. Use the source area as the
framing editor and keep a separate monochrome export preview. Distinguish
**crop zoom** (changes exported content) from existing **preview zoom**
(1×/2×/3× display magnification only).

At crop zoom 1×, use the largest centered output-aspect crop inside the image.
Zoom around the current crop center, clamp panning at image edges, and validate
a finite, documented zoom range that cannot produce an empty selection. Wheel
input should zoom only while hovering the editor, not hijack other scrolling.
Provide keyboard-adjustable zoom/pan controls as an alternative to dragging.

Keep crop math/resampling in the shared conversion core, independent of egui;
GUI-only editing state need not become new CLI-facing settings. Map display
coordinates back to the oriented source, not the thumbnail. Preserve the
original and regenerate grayscale from the selected region, not a previously
resized/adjusted preview. Crop before resizing to avoid large intermediates.

Retain crop selection when toggling Fit/Fill. A new image and Reset controls
restore centered 1× crop; reset also retains its existing tone/default behavior.
Changing framing must not discard tone settings. Cache by source and crop;
tone/dither changes should not re-decode or re-crop the original.

Measure crop preparation on a large input. Reuse the loading worker approach
where needed to avoid restoring UI freezes during dragging; bound/coalesce
requests and apply only the latest applicable result. Export the displayed
monochrome buffer, never an invisible pending result; make pending updates clear.
Do not add presets, rotation, free-aspect crop handles, or CLI crop controls.

## Acceptance

- [ ] On KDE Wayland, dragging and slider/wheel zoom select an off-center portion
      of a real image, with a fixed output-aspect frame and live result preview.
- [ ] Crop zoom and preview magnification are clearly labeled and independent;
      preview magnification alone never changes exported pixels.
- [ ] Panning/zoom remain inside the source, including portrait, landscape,
      tiny images, and extreme aspect ratios; invalid/non-finite input is handled.
- [ ] Keyboard controls can adjust selection without mouse dragging/wheel use.
- [ ] Tests cover crop bounds, coordinate mapping, centered defaults, edge
      clamping, pan/zoom changes, Fit/Fill toggling, and reset/new-image behavior.
- [ ] Tests verify selected source pixels and preview/export agreement using
      an asymmetric pattern, with orientation applied before crop coordinates.
- [ ] Loading/dragging stays responsive on a representative large image, with
      no stale results or unbounded request queues if preparation is asynchronous.
- [ ] Existing CLI output/defaults and Fit behavior are unchanged; `just check`
      passes, and controls/zoom semantics are documented.

## Progress / handoff

Not started. Planning only. Current `Source::prepare(Framing::Crop)` always
center-crops; the GUI source is only a thumbnail and `Editor::zoom` is display
magnification. Relevant files: `src/pipeline.rs`, `src/gui.rs`, `tests/core.rs`,
and GUI-state tests. Preserve both supplied BMP fixtures unchanged.
