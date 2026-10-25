use image::GrayImage;

use crate::mono::MonoImage;

mod algorithm;
mod atkinson;
mod blue_noise;
mod burkes;
mod clustered_dot;
mod dbs;
mod error_diffusion;
mod floyd_steinberg;
mod jarvis_judice_ninke;
mod ordered_bayer;
mod ostromoukhov;
mod riemersma;
mod sauvola;
mod sierra;
mod stucki;
mod threshold;

pub use algorithm::Algorithm;
pub use atkinson::Atkinson;
pub use blue_noise::BlueNoise;
pub use burkes::Burkes;
pub use clustered_dot::ClusteredDot;
pub use dbs::DirectBinarySearch;
pub use floyd_steinberg::FloydSteinberg;
pub use jarvis_judice_ninke::JarvisJudiceNinke;
pub use ordered_bayer::OrderedBayer;
pub use ostromoukhov::Ostromoukhov;
pub use riemersma::Riemersma;
pub use sauvola::Sauvola;
pub use sierra::Sierra;
pub use stucki::Stucki;
pub use threshold::Threshold;

pub trait Dither {
    fn dither(&self, image: &GrayImage) -> MonoImage;
}
