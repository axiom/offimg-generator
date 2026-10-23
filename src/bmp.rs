use std::io::{self, Write};

use crate::{HEIGHT, WIDTH, mono::MonoImage};

pub const PIXEL_OFFSET: usize = 130;
pub const ROW_BYTES: usize = 52;
pub const FILE_SIZE: usize = PIXEL_OFFSET + ROW_BYTES * HEIGHT as usize;

/// Write the samples' V4 layout, with palette index 0 black and index 1 white.
pub fn write(image: &MonoImage, mut output: impl Write) -> io::Result<()> {
    if image.pixels().dimensions() != (WIDTH, HEIGHT) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "OFF-images must be exactly 400×240 pixels",
        ));
    }
    let mut bytes = vec![0; FILE_SIZE];
    bytes[0..2].copy_from_slice(b"BM");
    for (offset, value) in [
        (2, FILE_SIZE as u32),
        (10, PIXEL_OFFSET as u32),
        (14, 108),
        (18, WIDTH),
        (22, HEIGHT),
        (34, (ROW_BYTES * HEIGHT as usize) as u32),
        (38, 3780),
        (42, 3780),
        (46, 2),
        (50, 2),
        // Preserve the samples' unusual, unused V4 fields under BI_RGB.
        (54, 0x7352_4742), // Red mask contains the sRGB tag, not the color-space field.
        (106, 2),          // Last CIEXYZ endpoint component; gamma fields are all zero.
    ] {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes[26..28].copy_from_slice(&1u16.to_le_bytes());
    bytes[28..30].copy_from_slice(&1u16.to_le_bytes());
    bytes[126..129].fill(255);
    for y in 0..HEIGHT {
        let row = PIXEL_OFFSET + (HEIGHT - 1 - y) as usize * ROW_BYTES;
        for x in 0..WIDTH {
            if image.pixels().get_pixel(x, y)[0] == 255 {
                bytes[row + x as usize / 8] |= 0x80 >> (x % 8);
            }
        }
    }
    output.write_all(&bytes)
}
