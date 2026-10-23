# Ostromoukhov coefficient provenance

The 128 integer coefficient triples in `src/dither/ostromoukhov.rs` are the
low-half table from `dither_ostromoukhov` in:

https://github.com/dalpil/structure-aware-dithering/blob/2f58d6a22608017ccba50c5f69197b5eb28e64da/src/dither/__init__.py

That repository is MIT-licensed; its notice is retained in `LICENSE.txt`.
The pinned source's SHA-256 is
`d66cb0d819be06707ec8b4422692f54127f49ffd48d9636b8c12031f4604623b`;
all 128 triples were checked against its Python literal during implementation.
Only the triples are transcribed. The redundant fourth column (sum) is computed
locally. High gray levels mirror the low half. No table interpolation,
randomness or runtime downloads are used.

Algorithm: Victor Ostromoukhov, *A Simple and Efficient Error-Diffusion
Algorithm*, SIGGRAPH 2001, https://doi.org/10.1145/383259.383326.

This implementation selects coefficients from the original, tone-adjusted gray
byte, uses floating accumulated error without intermediate clamping, and scans
serpentine with white at 127.5. These conventions are explicit: other programs
may index the corrected intensity, clamp propagated error, or use another tie
rule and therefore need not produce identical pixels.
