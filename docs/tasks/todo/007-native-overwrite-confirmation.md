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
- [ ] Source paths and symlink/hardlink aliases remain rejected after approval.
- [ ] Atomic writes, failure cleanup, and CLI no-overwrite/`--force` tests pass.
- [ ] Automated GUI/export checks cover accepted saves and failure preservation.
- [ ] Documentation describes the single native confirmation and its supported
      backend, without claiming the source can be replaced.
- [ ] `just check` passes.

## Progress / handoff

Not started. Planning only. Current code still shows both dialogs.
Relevant code: `src/gui.rs` toolbar/save/confirmation, `src/export.rs`,
`tests/cli.rs`, and the GUI-state test in `src/gui.rs`.
Do not modify the two supplied BMP fixtures or any calculator files.
