use image::GrayImage;

use super::{Dither, error_diffusion::diffuse};
use crate::mono::MonoImage;

pub struct JarvisJudiceNinke;

pub(super) const TAPS: &[(isize, usize, f32)] = &[
    (1, 0, 7.0 / 48.0),
    (2, 0, 5.0 / 48.0),
    (-2, 1, 3.0 / 48.0),
    (-1, 1, 5.0 / 48.0),
    (0, 1, 7.0 / 48.0),
    (1, 1, 5.0 / 48.0),
    (2, 1, 3.0 / 48.0),
    (-2, 2, 1.0 / 48.0),
    (-1, 2, 3.0 / 48.0),
    (0, 2, 5.0 / 48.0),
    (1, 2, 3.0 / 48.0),
    (2, 2, 1.0 / 48.0),
];

impl Dither for JarvisJudiceNinke {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        diffuse(image, TAPS, false)
    }
}
