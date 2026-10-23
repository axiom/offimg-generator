# Fixed blue-noise threshold mask

`mask.bin` is one tileable 64×64 single-channel, 8-bit mask (4096 bytes),
embedded at compile time. It comes from Christoph Peters' void-and-cluster
texture database, dedicated to the public domain under **CC0-1.0**.
The original notice and legal text are in `LICENSE.txt` and `COPYING.txt`.

- Article: https://momentsingraphics.de/BlueNoise.html
- Archive: https://momentsingraphics.de/Media/BlueNoise/FreeBlueNoiseTextures.zip
- Exact archive member: `Data/64_64/LDR_LLL1_0.png` (64×64 RGBA8).
- Conversion: extract the **red** channel without gamma/color conversion,
  resizing, flipping, or other processing. Its green/blue channels are identical;
  alpha is unused. Store bytes left-to-right, top-to-bottom, without a header.
  With Pillow: `Image.open(source).getchannel("R").tobytes()`.
- Each byte value 0–255 occurs 16 times. Thresholds are centered at
  `(byte + 0.5) * 255 / 256`, strictly between black and white; tile every 64
  output pixels on both axes, anchored at (0,0).
- No generation, randomness, downloads, or PNG decoding at build/runtime.

SHA-256 (downloaded and verified during task 011):

| File | SHA-256 |
| --- | --- |
| `FreeBlueNoiseTextures.zip` | `4279ad66ec851a8cbf37025816ef5187bf1c1c001d46afb8edfa6d3c8e44c644` |
| `Data/64_64/LDR_LLL1_0.png` | `140c55bd97c21b9b22008d7054b492333fec67fbfc1a1b266e5b0209b647eed4` |
| `mask.bin` | `fe602dd3f05c3d79ef1ff01533923c45a9eede45531251901818250445fbe4b1` |

Verify the vendored bytes with `sha256sum assets/blue-noise/mask.bin`.
