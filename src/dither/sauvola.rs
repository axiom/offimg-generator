use image::GrayImage;

use super::Dither;
use crate::mono::MonoImage;

pub struct Sauvola;

impl Dither for Sauvola {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        let stride = image.width() as usize + 1;
        let mut sums = vec![[0_u64; 2]; stride * (image.height() as usize + 1)];
        for y in 0..image.height() as usize {
            let mut row = [0; 2];
            for x in 0..image.width() as usize {
                let value = u64::from(image.get_pixel(x as u32, y as u32)[0]);
                row[0] += value;
                row[1] += value * value;
                for c in 0..2 {
                    sums[(y + 1) * stride + x + 1][c] = sums[y * stride + x + 1][c] + row[c];
                }
            }
        }
        MonoImage::from_fn(image.width(), image.height(), |x, y| {
            let (left, top) = (x.saturating_sub(7) as usize, y.saturating_sub(7) as usize);
            let right = (u64::from(x) + 8).min(u64::from(image.width())) as usize;
            let bottom = (u64::from(y) + 8).min(u64::from(image.height())) as usize;
            let count = ((right - left) * (bottom - top)) as f64;
            let stats: [f64; 2] = std::array::from_fn(|c| {
                (sums[bottom * stride + right][c] + sums[top * stride + left][c]
                    - sums[top * stride + right][c]
                    - sums[bottom * stride + left][c]) as f64
                    / count
            });
            let deviation = (stats[1] - stats[0] * stats[0]).max(0.0).sqrt();
            let threshold = stats[0] * (1.0 + 0.2 * (deviation / 128.0 - 1.0));
            let value = image.get_pixel(x, y)[0];
            // A zero-mean black window has a zero threshold, not a white background.
            value != 0 && f64::from(value) >= threshold
        })
    }
}
