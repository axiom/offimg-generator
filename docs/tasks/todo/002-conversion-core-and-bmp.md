# 002 — Conversion core and BMP export

Depends on: 001.

## Goal

A tested shared Rust library that converts an image to a real 400×240, 1-bit BMP.

## Scope

Create one Cargo package and library modules as needed. Implement limited input
decoders, orientation handling, decoder limits, white alpha compositing,
aspect-preserving fit/center-crop, grayscale, a small dithering trait with
threshold implementation, and a monochrome type with an enforced invariant.
Keep settings independent of clap and egui. Add the V4 BMP writer after examining
the remaining sample header fields. Preserve both supplied BMP fixtures.

## Acceptance

- [ ] `cargo test` and `cargo fmt --check` pass.
- [ ] Tests cover fit/crop, transparency, orientation, and invalid input.
- [ ] Both supplied fixtures decode with their palette interpreted correctly.
- [ ] Output headers report 400×240, 1 bpp, no compression, and correct sizes.
- [ ] A test pattern verifies bit order, row order, palette, and padding.
- [ ] An independent decoder reads exported BMP pixels back identically.
- [ ] Invalid or oversized input fails with an actionable error.

## Progress / handoff

Not started. No hardware compatibility claim until task 006.
