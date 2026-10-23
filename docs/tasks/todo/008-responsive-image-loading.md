# 008 — Responsive image loading

Depends on: 007. Do before 009; hardware validation 006 is independent.

## Goal

Keep the window repainting and responding while loading large images and
changing framing, without repeatedly processing unnecessary full-resolution data.

## Scope

The user observed freezing **after selecting an image to load** and subsequently
also **when changing Fit/Fill framing**, especially with large sources such as
4K wallpapers. `Editor::open` currently does file I/O, decoding, orientation,
compositing, framing, rendering, and
thumbnail preparation synchronously. The earlier ~16 ms measurement covered
only cached tone/dither/texture updates, not loading or reframing a
full-resolution image. `Editor::refresh` calls `Source::prepare` synchronously
on framing changes, repeating full-source resampling.

Before adding worker complexity, measure decode/orientation/compositing,
resampling, thumbnail creation, and framing changes on a representative 4K
wallpaper. Investigate downsizing once to a bounded GUI working image and/or
caching prepared Fit/Fill grayscale buffers so toggling framing does not repeat
expensive full-resolution work. Choose the smallest measured fix; do not assume
that dithering is the bottleneck. The current pipeline already resizes to
400×240 before grayscale/tone/dithering; the costly resize itself still reads
the large source.

If using a smaller working image, preserve aspect ratio, orientation, alpha
compositing, and framing semantics; avoid upscaling tiny inputs. Document and
check its size/filter and visible quality on text/line art as well as photos.
Do not permanently replace the original with a 400-pixel-wide copy: task 009
needs original-resolution detail for zoomed crops. Keep crop coordinates tied
to the oriented original, and regenerate a selected crop from that original
when needed. Keep any GUI-only optimization from silently changing CLI output.

Move remaining expensive loading/preparation to a small background worker,
using standard threads/channels unless existing tooling proves insufficient. Keep egui texture
creation and editor-state replacement on the UI thread. Transfer CPU buffers
and owned errors back, wake/repaint the UI when ready, and show a loading state.
Do not move every slider operation to a thread or add an async runtime. Framing
changes must use prepared caches or bounded background preparation if they
still stall; fixing initial loading alone does not complete this task.

Use one in-flight load initially; image-changing actions may be disabled while
busy, but repainting, moving, and closing the window must still work. Do not
accumulate unbounded load requests. Retain the current image/preview until a
complete replacement succeeds. Keep settings consistent with prepared results
if controls are allowed to change during a load. Errors must clear the busy
state and keep the old image available. Closing the app must not wait on a
blocking decode from the UI thread.

Preserve decoder limits and source identity. Avoid cloning the full source just
to cross a thread boundary. Bound working-image/cache memory as well as requests.
Measure loading stages, framing changes, and normal cached updates before and
after optimization; expand background work only where observed stalls justify
it. Synchronous native file dialogs and asynchronous export are outside this
task's scope.

## Acceptance

- [ ] Loading state appears promptly; a large, valid image loads while the
      window continues to repaint, move, and respond to closing on KDE Wayland.
- [x] Decode and initial full-resolution preparation do not run on the UI thread;
      egui texture updates do run there.
- [ ] Repeated Fit/Fill changes on a representative 4K wallpaper remain
      responsive, retain tone settings, and cannot apply stale preparation.
- [x] Evaluate downsizing/caching before expanding threading; record the chosen
      approach and before/after stage timings, including framing changes.
- [x] Any working-image/cache optimization has bounded memory and checks for
      aspect ratio, orientation, transparency, tiny inputs, and image quality;
      the original remains available for task 009's zoomed crops.
- [x] A failed/oversized load keeps the old source and preview, shows the error,
      clears loading state, and allows a subsequent successful load.
- [x] Requests are bounded, and an obsolete result cannot replace newer state
      (or a second load is explicitly prevented while busy).
- [x] Tests exercise pending/success/failure state transitions deterministically,
      without relying on arbitrary sleeps, including settings consistency.
- [x] Preview/export agreement, decoder limits, and CLI-only behavior remain
      covered; `just check` passes.
- [ ] Record a representative image's dimensions/format, load timings, and
      observed desktop responsiveness; do not use a fragile timing assertion.

## Progress / handoff

Implemented; awaiting the user's KDE Wayland check. Kept in `todo/` until that
check passes. All code changes are GUI-only in `src/gui.rs`; the shared pipeline,
decoder limits, CLI behavior, dependencies, and supplied BMP fixtures are unchanged.

Chosen approach: cache both exact original-resolution Fit/Fill preparations
(2 × 400 × 240 grayscale bytes = 192,000 bytes), rather than downsize a working
copy. This removes repeated expensive resampling without another filter pass,
aspect-ratio rounding differences, or lost detail. The full oriented/composited
original is retained for 009; thumbnail CPU storage is at most 288,000 RGB bytes.
No full original is cloned for transfer. Existing source/decoder limits still
bound the originals and resampling intermediates; this is not a new process-wide
256 MiB limit. The old source remains resident while one replacement loads.

One detached thread does file I/O, decode/orientation/compositing, both framing
preparations, and thumbnail creation. A capacity-one channel returns owned CPU
buffers or a string error and wakes egui. UI-thread installation renders with
the latest settings and creates textures before replacing the old loaded state.
Open/export are disabled while busy; controls and preview zoom stay available.
Missing/oversized inputs and disconnected workers clear pending state without
losing the previous image. Closing never joins the worker. There is no queue,
async runtime, per-slider worker, or change to native dialogs/export.

### Measurements (2026-10-23)

Representative existing wallpaper: `ghost-rider-skull-3840x2160-16397.jpg`,
3840×2160 JPEG. Debug/test profile, no timing assertions; normal `just gui`
also uses the debug profile. Decode/orientation/compositing measured together.

| Stage | Before | After |
| --- | --- | --- |
| File decode/orientation/white compositing | 1.629 s on UI | 1.594 s on worker |
| Full-source Fit preparation | 2.360–2.368 s per toggle | cached during loading |
| Full-source Fill preparation | 2.390–2.406 s per toggle | cached during loading |
| Prepare both framing caches | not done up front | 4.719 s on worker |
| Source thumbnail | 336 ms on UI | 331 ms on worker |
| Cached tone/dither/texture update | 8.9–9.2 ms | unchanged small UI work |
| Fit/Fill including render/texture | about 2.4 s | 8.55–8.72 ms |
| UI install/render/both textures | part of blocking load | 10.52 ms |

Tradeoff: initial background work totals about 6.64 s rather than approximately
4.33 s blocking the UI for one mode. Caching prioritizes responsive interaction
and exact quality over shorter initial wall-clock time. No GUI-only working
image was introduced, so tiny inputs, orientation, alpha compositing, framing,
and photo/text quality use the unchanged shared pipeline. Tests compare caches
against original preparation on tiny, transparent, portrait, and line-art-like
inputs; existing core tests cover all EXIF orientations and extreme aspect ratios.
Visible desktop quality/responsiveness is not claimed from headless tests.

Commands passed:

- `nix develop --command just check` (25 tests passed, one manual timing test ignored).
- `nix develop --command cargo test --no-default-features` (21 tests passed).
- `nix develop --command env OFFIMG_BENCH_IMAGE=/path/to/wallpaper.jpg cargo test --all-features image_loading_timings -- --ignored --nocapture`.

Tests exercise empty/pending polling, rejection of a second request, successful
replacement with settings changed during loading, missing/oversized failures,
worker disconnection, subsequent successful loading, receiver drop on closing,
cache agreement, source protection, and preview/export agreement. Synchronization
uses channels, not sleeps. The timing test remains ignored for normal checks.

### Next step: manual KDE Wayland acceptance

Run `just gui` (or `nix develop --command just gui`) and:

1. Open a large valid wallpaper. Confirm the loading indicator animates promptly,
   move/resize the window while loading, and verify closing does not wait. Reopen
   for subsequent checks. Native file dialogs themselves remain synchronous.
2. Load a second image while a preview exists. Check the old preview stays visible;
   adjust tone/Fill during the wait and verify the new preview uses those settings.
3. Toggle Fit/Fill repeatedly on the 4K wallpaper; verify no freezing, settings
   loss, stale framing, or unexpected visual differences on photos and line art.
4. Try an invalid/oversized file, then a valid file; verify error/recovery and
   that export still matches the displayed preview.

Record the user's observed desktop results, check the remaining acceptance items,
and move this file to `done/` only after those pass. Task 009 builds on `Prepared`
and the pending/result model; it must regenerate selected crops from the retained
oriented original rather than either fixed framing cache.
