# Tasks

Read `../design.md` first. Work in the order below. Task IDs stay stable;
the GUI follow-ups 007–009 take priority over hardware validation 006.
Tasks are deliverable-sized handoffs, not function lists.

- Pending tasks live in `todo/`.
- Check acceptance items as they are verified, not merely implemented.
- Before stopping mid-task, update its progress/handoff section with what
  changed, commands run, blockers, and the next step.
- When all acceptance items pass, record the results and move the file to
  `done/`, preserving its name. Do not claim hardware checks from software tests.
- Update the design document when a decision changes. Avoid a second status
  ledger here; the task files are the source of truth.
- Use `jj` to commit verified progress as you go, with descriptive messages.
  Preserve unrelated user changes and include task handoff updates.

Order:

1. 001 — Development environment.
2. 002 — Conversion core and genuine 1-bit BMP export.
3. 003 — Command-line interface.
4. 004 — Dithering algorithms and adjustments.
5. 005 — Live-preview GUI.
6. [007 — Native-only GUI overwrite confirmation](todo/007-native-overwrite-confirmation.md).
7. [008 — Responsive image loading](todo/008-responsive-image-loading.md).
8. [009 — Interactive image crop and zoom](todo/009-interactive-crop-and-zoom.md).
9. [006 — R47 hardware validation](todo/006-r47-validation.md).

Hardware validation can also be performed independently after 005 when the
user and calculator are available; it need not wait for all GUI follow-ups.

To resume: “Read `docs/design.md` and `docs/tasks/README.md`, then continue the
first unfinished task.” No task runner or OpenCode-specific integration needed.
