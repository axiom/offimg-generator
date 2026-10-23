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

pub struct Atkinson;

impl Dither for Atkinson {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        // Six eighths intentionally retain only three quarters of the error.
        diffuse(
            image,
            &[
                (1, 0, 1.0 / 8.0),
                (2, 0, 1.0 / 8.0),
                (-1, 1, 1.0 / 8.0),
                (0, 1, 1.0 / 8.0),
                (1, 1, 1.0 / 8.0),
                (0, 2, 1.0 / 8.0),
            ],
        )
    }
}

pub struct Stucki;

impl Dither for Stucki {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        diffuse(
            image,
            &[
                (1, 0, 8.0 / 42.0),
                (2, 0, 4.0 / 42.0),
                (-2, 1, 2.0 / 42.0),
                (-1, 1, 4.0 / 42.0),
                (0, 1, 8.0 / 42.0),
                (1, 1, 4.0 / 42.0),
                (2, 1, 2.0 / 42.0),
                (-2, 2, 1.0 / 42.0),
                (-1, 2, 2.0 / 42.0),
                (0, 2, 4.0 / 42.0),
                (1, 2, 2.0 / 42.0),
                (2, 2, 1.0 / 42.0),
            ],
        )
    }
}

fn diffuse(image: &GrayImage, taps: &[(isize, usize, f32)]) -> MonoImage {
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
            for &(dx, dy, weight) in taps {
                if let Some(nx) = x.checked_add_signed(dx)
                    && nx < width
                    && y + dy < height
                {
                    values[(y + dy) * width + nx] += error * weight;
                }
            }
        }
    }
    MonoImage::from_fn(image.width(), image.height(), |x, y| {
        values[y as usize * width + x as usize] == 255.0
    })
}

pub struct BlueNoise;

// 64×64, row-major 8-bit thresholds; provenance and CC0 notice in assets/blue-noise/.
const BLUE_NOISE: &[u8; 64 * 64] = include_bytes!("../assets/blue-noise/mask.bin");

impl Dither for BlueNoise {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        MonoImage::from_fn(image.width(), image.height(), |x, y| {
            let level = BLUE_NOISE[(y as usize % 64) * 64 + x as usize % 64];
            // Exact integer form of gray >= (level + 0.5) * 255 / 256.
            u32::from(image.get_pixel(x, y)[0]) * 512 >= (2 * u32::from(level) + 1) * 255
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
    Atkinson,
    Stucki,
    BlueNoise,
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Threshold => "threshold",
            Self::FloydSteinberg => "floyd-steinberg",
            Self::Bayer => "bayer",
            Self::Atkinson => "atkinson",
            Self::Stucki => "stucki",
            Self::BlueNoise => "blue-noise",
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
            "atkinson" => Ok(Self::Atkinson),
            "stucki" => Ok(Self::Stucki),
            "blue-noise" => Ok(Self::BlueNoise),
            _ => Err(
                "dither must be threshold, floyd-steinberg, bayer, atkinson, stucki, or blue-noise",
            ),
        }
    }
}

impl Algorithm {
    pub fn apply(self, image: &GrayImage, threshold: u8) -> MonoImage {
        match self {
            Self::Threshold => Threshold(threshold).dither(image),
            Self::FloydSteinberg => FloydSteinberg.dither(image),
            Self::Bayer => OrderedBayer.dither(image),
            Self::Atkinson => Atkinson.dither(image),
            Self::Stucki => Stucki.dither(image),
            Self::BlueNoise => BlueNoise.dither(image),
        }
    }
}
