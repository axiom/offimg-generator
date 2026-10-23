use image::{GrayImage, Luma};

/// Pixels are private and can only be constructed from booleans: true is white.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonoImage {
    pixels: GrayImage,
}

impl MonoImage {
    pub fn from_fn(width: u32, height: u32, mut white: impl FnMut(u32, u32) -> bool) -> Self {
        Self {
            pixels: GrayImage::from_fn(width, height, |x, y| {
                Luma([if white(x, y) { 255 } else { 0 }])
            }),
        }
    }

    pub fn pixels(&self) -> &GrayImage {
        &self.pixels
    }

    pub fn invert(&mut self) {
        for pixel in self.pixels.pixels_mut() {
            pixel[0] = 255 - pixel[0];
        }
    }
}
