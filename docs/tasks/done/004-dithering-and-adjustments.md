# 004 — Dithering and tone adjustments

Depends on: 002, 003.

## Goal

Useful shared image controls and three selectable dithering algorithms.

## Scope

Add Floyd–Steinberg (default) and ordered Bayer through the dithering trait.
Add brightness, contrast, gamma, threshold, inversion, and shared validation.
Document ranges, formulas, processing order, and defaults. Expose the settings
through CLI flags without making the core depend on clap.

## Acceptance

- [x] All algorithms produce deterministic, strictly black/white output.
- [x] Tests cover black, white, gradients, tiny images, and image boundaries.
- [x] Neutral adjustments preserve grayscale values; inversion swaps output.
- [x] Invalid/non-finite values are rejected through shared validation.
- [x] CLI flags cover every shared user-facing setting with matching defaults.
- [x] `cargo test`, formatting, and Clippy pass.

## Progress / handoff

Added Floyd–Steinberg (now the default), 4×4 ordered Bayer, shared finite/range
validation, brightness/contrast/gamma LUT adjustment, threshold control, and
post-dither inversion. The CLI takes its defaults directly from `Settings`.
Ranges, formula, and algorithm details are in the README and design document.

Verified with `nix develop --command cargo test` (21 integration tests),
`cargo fmt --check`, and all-target/all-feature Clippy with warnings denied.
Tests include known small patterns, edge diffusion, solids/gradients and empty
or tiny buffers, all 256 neutral gray levels, inversion, rejected non-finite
values, and CLI/core pixel agreement for every algorithm and setting.

Next: task 005. The GUI can cache `Source::prepare(framing)` and call
`Settings::render(&gray)` for tone/algorithm changes without decoding/resizing
again. Use `export::save` for the same atomic/source-safe writes as the CLI.
