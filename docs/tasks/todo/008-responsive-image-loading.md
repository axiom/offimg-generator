# 008 — Responsive image loading

Depends on: 007. Do before 009; hardware validation 006 is independent.

## Goal

Keep the window repainting and responding while an image loads.

## Scope

The user observed freezing specifically **after selecting an image to load**,
not during tone changes, export, or the native dialog. `Editor::open` currently
does file I/O, decoding, orientation, compositing, framing, rendering, and
thumbnail preparation synchronously. The earlier ~16 ms measurement covered
only cached tone/dither/texture updates, not loading a full-resolution image.

Move expensive loading/preparation to a small background worker, using standard
threads/channels unless existing tooling proves insufficient. Keep egui texture
creation and editor-state replacement on the UI thread. Transfer CPU buffers
and owned errors back, wake/repaint the UI when ready, and show a loading state.
Do not move every slider operation to a thread or add an async runtime.

Use one in-flight load initially; image-changing actions may be disabled while
busy, but repainting, moving, and closing the window must still work. Do not
accumulate unbounded load requests. Retain the current image/preview until a
complete replacement succeeds. Keep settings consistent with prepared results
if controls are allowed to change during a load. Errors must clear the busy
state and keep the old image available. Closing the app must not wait on a
blocking decode from the UI thread.

Preserve decoder limits and source identity. Avoid cloning the full source just
to cross a thread boundary. Measure loading stages and normal cached updates;
expand background work only where observed stalls justify it. Synchronous
native file dialogs and asynchronous export are outside this task's scope.

## Acceptance

- [ ] Loading state appears promptly; a large, valid image loads while the
      window continues to repaint, move, and respond to closing on KDE Wayland.
- [ ] Decode and initial full-resolution preparation do not run on the UI thread;
      egui texture updates do run there.
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

Not started. Planning only. Start with `Editor::open` in `src/gui.rs` and
`Source::open`/`Source::prepare` in `src/pipeline.rs`. The current shared error
type is `Box<dyn Error>`; convert to an owned message at the worker boundary
if sufficient, rather than redesigning every error API just for threading.
Task 009 will build on this loading/state model for crop preparation.
