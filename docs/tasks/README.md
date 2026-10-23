# Tasks

Read `../design.md` first. Work in the order below. Task IDs stay stable;
the GUI follow-ups 007–010 take priority over hardware validation 006.
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
6. [007 — Native-only GUI overwrite confirmation](done/007-native-overwrite-confirmation.md).
7. [008 — Responsive image loading](done/008-responsive-image-loading.md).
8. [009 — Interactive image crop and zoom](done/009-interactive-crop-and-zoom.md).
9. [010 — Stack source and preview vertically](done/010-vertical-preview-layout.md).
10. [006 — R47 hardware validation](done/006-r47-validation.md).
11. [011 — Additional monochrome dithering algorithms](done/011-additional-dithering-algorithms.md).
12. [012 — Five more dithering algorithms](done/012-five-more-dithering-algorithms.md).
13. [013 — Adaptive/optimized conversion and sharpening](done/013-adaptive-optimized-conversion.md).

Hardware validation can also be performed independently after 005 when the
user and calculator are available; it need not wait for all GUI follow-ups.
Tasks 011–013 alone do not establish hardware compatibility; the user's device
confirmation is recorded in completed task 006.

To resume: “Read `docs/design.md` and `docs/tasks/README.md`, then continue the
first unfinished task.” No task runner or OpenCode-specific integration needed.
