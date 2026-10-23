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

- [x] `cargo test` and `cargo fmt --check` pass.
- [x] Tests cover fit/crop, transparency, orientation, and invalid input.
- [x] Both supplied fixtures decode with their palette interpreted correctly.
- [x] Output headers report 400×240, 1 bpp, no compression, and correct sizes.
- [x] A test pattern verifies bit order, row order, palette, and padding.
- [x] An independent decoder reads exported BMP pixels back identically.
- [x] Invalid or oversized input fails with an actionable error.

## Progress / handoff

Implemented one Cargo library with limited image features, bounded decoding,
EXIF orientation, alpha compositing, cached-grayscale preparation, fit/crop,
threshold dithering, a read-only monochrome buffer, and a dedicated V4 writer.
The supplied fixtures are now explicitly unignored so tests are reproducible;
their bytes were not modified. Full V4 findings are in `docs/design.md`.

Verified with `nix develop --command cargo test` (9 integration tests),
`cargo fmt --check`, and `cargo clippy --all-targets --all-features -- -D warnings`
(the latter commands also wrapped in `nix develop --command`). The round-trip
uses the independent general-purpose `image` BMP decoder, not writer code.

Next: task 003, the CLI and safe output handling. No hardware compatibility
claim until task 006. Threshold is the temporary default until task 004 adds
Floyd–Steinberg and tone controls.
