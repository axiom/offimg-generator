use image::GrayImage;

use crate::mono::MonoImage;

pub trait Dither {
    fn dither(&self, image: &GrayImage) -> MonoImage;
}

pub struct Threshold(pub u8);

impl Dither for Threshold {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        MonoImage::from_fn(image.width(), image.height(), |x, y| {
            image.get_pixel(x, y)[0] >= self.0
        })
    }
}

pub struct FloydSteinberg;

impl Dither for FloydSteinberg {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        let width = image.width() as usize;
        let height = image.height() as usize;
        let mut values: Vec<f32> = image.as_raw().iter().map(|&v| f32::from(v)).collect();
        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;
                let old = values[index];
                let new = if old >= 127.5 { 255.0 } else { 0.0 };
                values[index] = new;
                let error = old - new;
                if x + 1 < width {
                    values[index + 1] += error * (7.0 / 16.0);
                }
                if y + 1 < height {
                    if x > 0 {
                        values[index + width - 1] += error * (3.0 / 16.0);
                    }
                    values[index + width] += error * (5.0 / 16.0);
                    if x + 1 < width {
                        values[index + width + 1] += error * (1.0 / 16.0);
                    }
                }
            }
        }
        MonoImage::from_fn(image.width(), image.height(), |x, y| {
            values[y as usize * width + x as usize] == 255.0
        })
    }
}

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Algorithm {
    Threshold,
    #[default]
    FloydSteinberg,
    Bayer,
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Threshold => "threshold",
            Self::FloydSteinberg => "floyd-steinberg",
            Self::Bayer => "bayer",
        })
    }
}

impl std::str::FromStr for Algorithm {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "threshold" => Ok(Self::Threshold),
            "floyd-steinberg" => Ok(Self::FloydSteinberg),
            "bayer" => Ok(Self::Bayer),
            _ => Err("dither must be threshold, floyd-steinberg, or bayer"),
        }
    }
}

impl Algorithm {
    pub fn apply(self, image: &GrayImage, threshold: u8) -> MonoImage {
        match self {
            Self::Threshold => Threshold(threshold).dither(image),
            Self::FloydSteinberg => FloydSteinberg.dither(image),
            Self::Bayer => OrderedBayer.dither(image),
        }
    }
}
