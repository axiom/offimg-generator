# Tasks

Read `../design.md` first. Work in numbered order unless a task explicitly
permits otherwise. Tasks are deliverable-sized handoffs, not function lists.

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

1. Development environment.
2. Conversion core and genuine 1-bit BMP export.
3. Command-line interface.
4. Dithering algorithms and adjustments.
5. Live-preview GUI.
6. R47 hardware validation.

To resume: “Read `docs/design.md` and `docs/tasks/README.md`, then continue the
first unfinished task.” No task runner or OpenCode-specific integration needed.
