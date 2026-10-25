use image::GrayImage;

use super::{Dither, error_diffusion::diffuse};
use crate::mono::MonoImage;

pub struct Burkes;

pub(super) const TAPS: &[(isize, usize, f32)] = &[
    (1, 0, 8.0 / 32.0),
    (2, 0, 4.0 / 32.0),
    (-2, 1, 2.0 / 32.0),
    (-1, 1, 4.0 / 32.0),
    (0, 1, 8.0 / 32.0),
    (1, 1, 4.0 / 32.0),
    (2, 1, 2.0 / 32.0),
];

impl Dither for Burkes {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        diffuse(image, TAPS, false)
    }
}
