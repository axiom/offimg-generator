use image::GrayImage;

use super::{Dither, error_diffusion::diffuse};
use crate::mono::MonoImage;

pub struct Atkinson;

// Six eighths intentionally retain only three quarters of the error.
pub(super) const TAPS: &[(isize, usize, f32)] = &[
    (1, 0, 1.0 / 8.0),
    (2, 0, 1.0 / 8.0),
    (-1, 1, 1.0 / 8.0),
    (0, 1, 1.0 / 8.0),
    (1, 1, 1.0 / 8.0),
    (0, 2, 1.0 / 8.0),
];

impl Dither for Atkinson {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        diffuse(image, TAPS, false)
    }
}
