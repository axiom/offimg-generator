use image::GrayImage;

use super::Dither;
use crate::mono::MonoImage;

pub struct ClusteredDot;

impl Dither for ClusteredDot {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        const MATRIX: [[u32; 4]; 4] =
            [[12, 5, 6, 13], [4, 0, 1, 7], [11, 3, 2, 8], [15, 10, 9, 14]];
        MonoImage::from_fn(image.width(), image.height(), |x, y| {
            let level = MATRIX[y as usize % 4][x as usize % 4];
            u32::from(image.get_pixel(x, y)[0]) * 32 >= (2 * level + 1) * 255
        })
    }
}
