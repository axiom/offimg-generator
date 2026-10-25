use image::GrayImage;

use super::Dither;
use crate::mono::MonoImage;

pub struct OrderedBayer;

impl Dither for OrderedBayer {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        const MATRIX: [[u32; 4]; 4] =
            [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
        MonoImage::from_fn(image.width(), image.height(), |x, y| {
            let level = MATRIX[y as usize % 4][x as usize % 4];
            u32::from(image.get_pixel(x, y)[0]) * 32 >= (2 * level + 1) * 255
        })
    }
}
