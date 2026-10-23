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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Algorithm {
    #[default]
    Threshold,
}

impl std::str::FromStr for Algorithm {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "threshold" => Ok(Self::Threshold),
            _ => Err("dither must be threshold"),
        }
    }
}

impl Algorithm {
    pub fn apply(self, image: &GrayImage, threshold: u8) -> MonoImage {
        match self {
            Self::Threshold => Threshold(threshold).dither(image),
        }
    }
}
