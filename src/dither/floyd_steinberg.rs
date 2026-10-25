use image::GrayImage;

use super::{Dither, error_diffusion::diffuse};
use crate::mono::MonoImage;

pub struct FloydSteinberg;

pub(super) const TAPS: &[(isize, usize, f32)] = &[
    (1, 0, 7.0 / 16.0),
    (-1, 1, 3.0 / 16.0),
    (0, 1, 5.0 / 16.0),
    (1, 1, 1.0 / 16.0),
];

impl Dither for FloydSteinberg {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        diffuse(image, TAPS, false)
    }
}
