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

- [ ] All algorithms produce deterministic, strictly black/white output.
- [ ] Tests cover black, white, gradients, tiny images, and image boundaries.
- [ ] Neutral adjustments preserve grayscale values; inversion swaps output.
- [ ] Invalid/non-finite values are rejected through shared validation.
- [ ] CLI flags cover every shared user-facing setting with matching defaults.
- [ ] `cargo test`, formatting, and Clippy pass.

## Progress / handoff

Not started. Do not generalize to a plugin framework.
