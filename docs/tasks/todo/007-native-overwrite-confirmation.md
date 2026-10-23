# 007 — Native-only GUI overwrite confirmation

Depends on: 005. Do before 008 and 009; hardware validation 006 is independent.

## Goal

Ask once before replacing an exported BMP, using the native save dialog rather
than a second in-app dialog.

## Scope

The user verified that KDE's portal save dialog already confirms replacement
and that the app's extra Cancel/Replace dialog works but is redundant.
Check the supported rfd/portal save flow, then remove `Editor::pending_export`,
`Editor::confirmation`, and its update call in `src/gui.rs`. An accepted native
save selection should export directly; cancellation must not write anything.

Keep the shared atomic export helper and its source-identity checks. Native
approval never authorizes replacing the source or its symlink/hardlink aliases.
Keep the CLI's `--force` behavior unchanged. Derive replacement permission from
the accepted native save flow, not a blanket change to all export entry points;
retain no-clobber handling for new destinations where applicable.

Update the README/design to describe native confirmation. Target the tested
KDE Wayland portal; do not claim every Linux file-dialog backend confirms
replacement without checking its contract. Do not add a second confirmation
or fallback framework unless a supported backend demonstrably needs it.

## Acceptance

- [ ] Native cancellation leaves existing output and the preview untouched.
- [ ] Native approval replaces a disposable BMP with the current preview, with
      no in-app confirmation; manually verified on KDE Wayland.
- [x] Source paths and symlink/hardlink aliases remain rejected after approval.
- [x] Atomic writes, failure cleanup, and CLI no-overwrite/`--force` tests pass.
- [x] Automated GUI/export checks cover accepted saves and failure preservation.
- [x] Documentation describes the single native confirmation and its supported
      backend, without claiming the source can be replaced.
- [x] `just check` passes.

## Progress / handoff

Implemented native-only export in `src/gui.rs`: removed pending-export state,
the in-app modal, and its update call. The toolbar passes the native selection
to `save_selection`; `None` writes nothing and retains the preview. An accepted
existing destination permits replacement; new paths retain no-clobber writes.
The shared atomic exporter and CLI are unchanged.

Inspected the pinned rfd 0.17.2 implementation: portal `SaveFile` returns a
path only on success; cancellation returns an empty selection, not its Zenity
fallback. That fallback explicitly uses `--confirm-overwrite`. KDE's native
replacement prompt was previously confirmed by the user in task 005; other
portal backends are not claimed as verified. Updated README and design.

Verified `nix develop --command just check`: formatting, all-feature Clippy,
and all 23 tests pass. Headless GUI tests cover new and existing accepted
destinations, exact preview/export agreement, cancellation preserving output
and preview, rejection of source/symlink/hardlink aliases, failed loads, and
encoder-failure preservation/temporary-file cleanup. The supplied fixtures
and calculator files are untouched. Headless tests do not exercise native UI.

Next: user runs `just gui` on KDE Wayland with a disposable output BMP. Change
a control so the preview differs, select that existing output, and cancel the
native save/replacement dialog: output and preview must remain unchanged.
Repeat and approve replacement: exported pixels must match the current preview,
with no in-app confirmation. Record results, check the first two acceptance
items, remove the pending-manual-check note in the design, and move this task
to `done/` only after confirmation. Then continue task 008.
