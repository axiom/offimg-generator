use std::{fs::File, io::BufRead, io::BufReader, io::Seek, path::Path};

use image::{DynamicImage, GrayImage, ImageDecoder, ImageFormat, ImageReader, Limits, RgbImage};

use crate::{HEIGHT, Result, WIDTH, dither::Algorithm, mono::MonoImage};

pub const MAX_DIMENSION: u32 = 16_384;
pub const MAX_PIXELS: u64 = 32_000_000;
pub const MAX_DECODE_BYTES: u64 = 256 * 1024 * 1024;
pub const BRIGHTNESS_RANGE: std::ops::RangeInclusive<f32> = -1.0..=1.0;
pub const CONTRAST_RANGE: std::ops::RangeInclusive<f32> = 0.0..=4.0;
pub const GAMMA_RANGE: std::ops::RangeInclusive<f32> = 0.1..=4.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Framing {
    #[default]
    Fit,
    Crop,
}

impl std::fmt::Display for Framing {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Fit => "fit",
            Self::Crop => "crop",
        })
    }
}

impl std::str::FromStr for Framing {
    type Err = &'static str;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "fit" => Ok(Self::Fit),
            "crop" => Ok(Self::Crop),
            _ => Err("framing must be fit or crop"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub framing: Framing,
    pub algorithm: Algorithm,
    pub threshold: u8,
    pub brightness: f32,
    pub contrast: f32,
    pub gamma: f32,
    pub invert: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            framing: Framing::Fit,
            algorithm: Algorithm::default(),
            threshold: 128,
            brightness: 0.0,
            contrast: 1.0,
            gamma: 1.0,
            invert: false,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<()> {
        for (name, value, range) in [
            ("brightness", self.brightness, BRIGHTNESS_RANGE),
            ("contrast", self.contrast, CONTRAST_RANGE),
            ("gamma", self.gamma, GAMMA_RANGE),
        ] {
            if !range.contains(&value) {
                return Err(format!(
                    "{name} must be finite and in {}..={}",
                    range.start(),
                    range.end()
                )
                .into());
            }
        }
        Ok(())
    }

    /// Apply a fresh LUT to cached grayscale; never mutate a previous preview.
    pub fn adjust(&self, gray: &GrayImage) -> Result<GrayImage> {
        self.validate()?;
        let lut: [u8; 256] = std::array::from_fn(|value| {
            let normalized = value as f32 / 255.0;
            let adjusted =
                ((normalized - 0.5) * self.contrast + 0.5 + self.brightness).clamp(0.0, 1.0);
            (255.0 * adjusted.powf(1.0 / self.gamma)).round() as u8
        });
        Ok(GrayImage::from_fn(gray.width(), gray.height(), |x, y| {
            image::Luma([lut[usize::from(gray.get_pixel(x, y)[0])]])
        }))
    }

    pub fn render(&self, gray: &GrayImage) -> Result<MonoImage> {
        let mut mono = self.algorithm.apply(&self.adjust(gray)?, self.threshold);
        if self.invert {
            mono.invert();
        }
        Ok(mono)
    }
}

/// The oriented, white-composited original; adjustments never mutate it.
pub struct Source {
    image: RgbImage,
}

impl Source {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::decode(BufReader::new(File::open(path)?))
    }

    pub fn decode(reader: impl BufRead + Seek) -> Result<Self> {
        let mut reader = ImageReader::new(reader).with_guessed_format()?;
        if !matches!(
            reader.format(),
            Some(ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Bmp | ImageFormat::WebP)
        ) {
            return Err("unsupported image; use PNG, JPEG, BMP, or WebP".into());
        }
        let mut limits = Limits::default();
        limits.max_image_width = Some(MAX_DIMENSION);
        limits.max_image_height = Some(MAX_DIMENSION);
        limits.max_alloc = Some(MAX_DECODE_BYTES);
        reader.limits(limits);
        let mut decoder = reader.into_decoder().map_err(decode_error)?;
        let (width, height) = decoder.dimensions();
        if width == 0 || height == 0 || u64::from(width) * u64::from(height) > MAX_PIXELS {
            return Err(format!("input must have 1–{MAX_PIXELS} pixels; resize it first").into());
        }
        if decoder.total_bytes() > MAX_DECODE_BYTES {
            return Err("decoded input exceeds 256 MiB; resize it first".into());
        }
        let orientation = decoder.orientation().map_err(decode_error)?;
        let mut image = DynamicImage::from_decoder(decoder).map_err(decode_error)?;
        image.apply_orientation(orientation);
        let rgba = image.into_rgba8();
        let image = RgbImage::from_fn(rgba.width(), rgba.height(), |x, y| {
            let pixel = rgba.get_pixel(x, y);
            let alpha = u32::from(pixel[3]);
            image::Rgb(std::array::from_fn(|c| {
                ((u32::from(pixel[c]) * alpha + 255 * (255 - alpha) + 127) / 255) as u8
            }))
        });
        Ok(Self { image })
    }

    pub fn image(&self) -> &RgbImage {
        &self.image
    }

    /// Cache this result in interactive callers until source or framing changes.
    pub fn prepare(&self, framing: Framing) -> GrayImage {
        use image::imageops::{FilterType::Lanczos3, crop_imm, overlay, resize};

        let (width, height) = self.image.dimensions();
        let framed = match framing {
            Framing::Fit => {
                let (fit_width, fit_height) = if u64::from(width) * u64::from(HEIGHT)
                    > u64::from(height) * u64::from(WIDTH)
                {
                    (
                        WIDTH,
                        ((u64::from(height) * u64::from(WIDTH) / u64::from(width)) as u32).max(1),
                    )
                } else {
                    (
                        ((u64::from(width) * u64::from(HEIGHT) / u64::from(height)) as u32).max(1),
                        HEIGHT,
                    )
                };
                let resized = resize(&self.image, fit_width, fit_height, Lanczos3);
                let mut canvas = RgbImage::from_pixel(WIDTH, HEIGHT, image::Rgb([255; 3]));
                overlay(
                    &mut canvas,
                    &resized,
                    i64::from((WIDTH - resized.width()) / 2),
                    i64::from((HEIGHT - resized.height()) / 2),
                );
                canvas
            }
            Framing::Crop => {
                // Crop before resizing so extreme aspect ratios cannot create huge intermediates.
                let (crop_width, crop_height) = if u64::from(width) * u64::from(HEIGHT)
                    > u64::from(height) * u64::from(WIDTH)
                {
                    (
                        (u64::from(height) * u64::from(WIDTH) / u64::from(HEIGHT)) as u32,
                        height,
                    )
                } else {
                    (
                        width,
                        ((u64::from(width) * u64::from(HEIGHT) / u64::from(WIDTH)) as u32).max(1),
                    )
                };
                let cropped = crop_imm(
                    &self.image,
                    (width - crop_width) / 2,
                    (height - crop_height) / 2,
                    crop_width,
                    crop_height,
                );
                resize(&cropped.to_image(), WIDTH, HEIGHT, Lanczos3)
            }
        };
        DynamicImage::ImageRgb8(framed).into_luma8()
    }

    pub fn convert(&self, settings: &Settings) -> Result<MonoImage> {
        settings.validate()?;
        let gray = self.prepare(settings.framing);
        settings.render(&gray)
    }
}

fn decode_error(error: image::ImageError) -> Box<dyn std::error::Error> {
    if matches!(error, image::ImageError::Limits(_)) {
        format!(
            "input exceeds decoder limits ({MAX_DIMENSION} pixels per side, 256 MiB); resize it first: {error}"
        )
        .into()
    } else {
        format!("cannot decode input image: {error}").into()
    }
}
