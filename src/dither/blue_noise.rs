use image::GrayImage;

use super::Dither;
use crate::mono::MonoImage;

pub struct BlueNoise;

// 64×64, row-major 8-bit thresholds; provenance and CC0 notice in assets/blue-noise/.
const BLUE_NOISE: &[u8; 64 * 64] = include_bytes!("../../assets/blue-noise/mask.bin");

impl Dither for BlueNoise {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        MonoImage::from_fn(image.width(), image.height(), |x, y| {
            let level = BLUE_NOISE[(y as usize % 64) * 64 + x as usize % 64];
            // Exact integer form of gray >= (level + 0.5) * 255 / 256.
            u32::from(image.get_pixel(x, y)[0]) * 512 >= (2 * u32::from(level) + 1) * 255
        })
    }
}
