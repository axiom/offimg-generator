# 009 — Interactive image crop and zoom

Depends on: 008. Do before hardware validation 006 in the normal work order.

## Goal

Let the user choose a portion of the original image for the exported OFF screen,
instead of being limited to fit or a centered crop.

## Confirmed interaction

- GUI-only initially; no new CLI flags or changed CLI defaults.
- A fixed **400:240** crop frame, with the image dragged behind it to pan.
- Mouse-wheel zoom over the crop editor; no zoom/pan sliders, per the user's
  follow-up after confirming cropping and zooming works.
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
Keep keyboard access on the focusable frame (arrows pan the image, +/− zoom),
without separate zoom/pan controls.

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

- [x] On KDE Wayland, dragging and wheel zoom select an off-center portion
      of a real image, with a fixed output-aspect frame and live result preview.
- [x] Crop zoom and preview magnification are clearly labeled and independent;
      preview magnification alone never changes exported pixels.
- [x] Panning/zoom remain inside the source, including portrait, landscape,
      tiny images, and extreme aspect ratios; invalid/non-finite input is handled.
- [x] Keyboard controls can adjust selection without mouse dragging/wheel use.
- [x] Tests cover crop bounds, coordinate mapping, centered defaults, edge
      clamping, pan/zoom changes, Fit/Fill toggling, and reset/new-image behavior.
- [x] Tests verify selected source pixels and preview/export agreement using
      an asymmetric pattern, with orientation applied before crop coordinates.
- [ ] Loading/dragging stays responsive on a representative large image, with
      no stale results or unbounded request queues if preparation is asynchronous.
- [x] Existing CLI output/defaults and Fit behavior are unchanged; `just check`
      passes, and controls/zoom semantics are documented.

## Progress / handoff

Implemented. The user confirms cropping/zooming works and has also confirmed the
updated slider-free UI. Remaining: explicit representative-large-image
responsiveness confirmation before moving to `done/`.

- `src/pipeline.rs`: shared `Crop`/`CropBounds`, finite validation, 1×–16× zoom,
  clamped normalized centers, frame-to-source drag mapping, crop-before-resize.
  Centered 1× output remains pixel-identical to the previous integer crop.
- `src/gui.rs`: fixed-frame Fill editor using thumbnail UVs, drag/wheel input,
  keyboard access on the frame instead of zoom/pan sliders, independent preview magnification,
  retained Fit/Fill selection, reset/new-image behavior. Source pixels are shared
  with one detached crop worker, with coalescing, stale-result rejection and a
  visible pending label. Export uses the displayed monochrome buffer even pending.
- Core/GUI tests cover bounds, tiny/extreme aspect ratios, invalid/non-finite
  inputs, coordinate mapping, EXIF-before-crop, asymmetric selected pixels,
  preview/export agreement, keyboard input, drag/wheel scroll isolation,
  cache reuse, toggling, resets, new sources and stale/coalesced work.
- Both supplied BMP fixtures remain unchanged. No CLI controls, new dependencies,
  presets or task 010 layout/default-magnification changes added.

Measurements: `nix develop --command cargo test --all-features
large_image_crop_timings -- --ignored --nocapture` passed on a generated
3840×2160 RGB PNG. Debug crop preparation: 1× 2.423 s, 2× 692 ms, 16× 62 ms.
200 nonblocking request updates during preparation: 6.3 µs; this measures request
coalescing, not drawing or real desktop responsiveness.

Validation: `nix develop --command just check` passed (formatting, Clippy with
warnings denied, 32 tests; two manual timing tests skipped). The crop timing
test above was run separately. `nix develop --command cargo test
--no-default-features` also passed all 24 CLI/core tests.

Follow-up: removed the crop zoom and both pan sliders as requested. Mouse-wheel
zoom and dragging are unchanged; a read-only crop zoom hint distinguishes it
from preview magnification. Tab to the frame for arrow-key pan and +/− zoom.
Updated the headless keyboard test to exercise the frame, not removed sliders.
Follow-up validation: `nix develop --command just check` passes again (32 tests,
formatting and Clippy; two manual timing tests skipped). The user subsequently
confirmed the updated slider-free UI. No code changed after those checks.

Remaining desktop check: confirm loading/dragging responsiveness with a large real
image, without freezes or stale updates. The UI confirmation did not explicitly
identify the test image size. Record that result before moving to `done/`;
do not claim hardware compatibility from desktop checks.
