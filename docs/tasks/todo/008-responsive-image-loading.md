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
- [ ] Decode and initial full-resolution preparation do not run on the UI thread;
      egui texture updates do run there.
- [ ] Repeated Fit/Fill changes on a representative 4K wallpaper remain
      responsive, retain tone settings, and cannot apply stale preparation.
- [ ] Evaluate downsizing/caching before expanding threading; record the chosen
      approach and before/after stage timings, including framing changes.
- [ ] Any working-image/cache optimization has bounded memory and checks for
      aspect ratio, orientation, transparency, tiny inputs, and image quality;
      the original remains available for task 009's zoomed crops.
- [ ] A failed/oversized load keeps the old source and preview, shows the error,
      clears loading state, and allows a subsequent successful load.
- [ ] Requests are bounded, and an obsolete result cannot replace newer state
      (or a second load is explicitly prevented while busy).
- [ ] Tests exercise pending/success/failure state transitions deterministically,
      without relying on arbitrary sleeps, including settings consistency.
- [ ] Preview/export agreement, decoder limits, and CLI-only behavior remain
      covered; `just check` passes.
- [ ] Record a representative image's dimensions/format, load timings, and
      observed desktop responsiveness; do not use a fragile timing assertion.

## Progress / handoff

Not started. Planning updated with the user's 4K loading/framing freezes and
request to consider downsizing before threading. Start with `Editor::open`
and `Editor::refresh` in `src/gui.rs` and
`Source::open`/`Source::prepare` in `src/pipeline.rs`. The current shared error
type is `Box<dyn Error>`; convert to an owned message at the worker boundary
if sufficient, rather than redesigning every error API just for threading.
Task 009 will build on this loading/state model for crop preparation.
