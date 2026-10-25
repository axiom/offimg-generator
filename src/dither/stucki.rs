use image::GrayImage;

use super::{Dither, error_diffusion::diffuse};
use crate::mono::MonoImage;

pub struct Stucki;

pub(super) const TAPS: &[(isize, usize, f32)] = &[
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
];

impl Dither for Stucki {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        diffuse(image, TAPS, false)
    }
}
