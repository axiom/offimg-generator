use image::GrayImage;

use super::{
    Atkinson, BlueNoise, Burkes, ClusteredDot, DirectBinarySearch, Dither, FloydSteinberg,
    JarvisJudiceNinke, OrderedBayer, Ostromoukhov, Riemersma, Sauvola, Sierra, Stucki, Threshold,
    atkinson, burkes, error_diffusion::diffuse, floyd_steinberg, jarvis_judice_ninke, sierra,
    stucki,
};
use crate::mono::MonoImage;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Algorithm {
    Threshold,
    #[default]
    FloydSteinberg,
    Bayer,
    Atkinson,
    Stucki,
    BlueNoise,
    Sierra,
    Burkes,
    JarvisJudiceNinke,
    Riemersma,
    ClusteredDot,
    Sauvola,
    Ostromoukhov,
    Dbs,
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
            Self::Sierra => "sierra",
            Self::Burkes => "burkes",
            Self::JarvisJudiceNinke => "jarvis-judice-ninke",
            Self::Riemersma => "riemersma",
            Self::ClusteredDot => "clustered-dot",
            Self::Sauvola => "sauvola",
            Self::Ostromoukhov => "ostromoukhov",
            Self::Dbs => "dbs",
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
            "sierra" => Ok(Self::Sierra),
            "burkes" => Ok(Self::Burkes),
            "jarvis-judice-ninke" => Ok(Self::JarvisJudiceNinke),
            "riemersma" => Ok(Self::Riemersma),
            "clustered-dot" => Ok(Self::ClusteredDot),
            "sauvola" => Ok(Self::Sauvola),
            "ostromoukhov" => Ok(Self::Ostromoukhov),
            "dbs" => Ok(Self::Dbs),
            _ => Err(
                "dither must be threshold, floyd-steinberg, bayer, atkinson, stucki, blue-noise, sierra, burkes, jarvis-judice-ninke, riemersma, clustered-dot, sauvola, ostromoukhov, or dbs",
            ),
        }
    }
}

impl Algorithm {
    pub const ALL: [Self; 14] = [
        Self::Threshold,
        Self::FloydSteinberg,
        Self::Bayer,
        Self::Atkinson,
        Self::Stucki,
        Self::BlueNoise,
        Self::Sierra,
        Self::Burkes,
        Self::JarvisJudiceNinke,
        Self::Riemersma,
        Self::ClusteredDot,
        Self::Sauvola,
        Self::Ostromoukhov,
        Self::Dbs,
    ];

    pub fn apply(self, image: &GrayImage, threshold: u8) -> MonoImage {
        match self {
            Self::Threshold => Threshold(threshold).dither(image),
            Self::FloydSteinberg => FloydSteinberg.dither(image),
            Self::Bayer => OrderedBayer.dither(image),
            Self::Atkinson => Atkinson.dither(image),
            Self::Stucki => Stucki.dither(image),
            Self::BlueNoise => BlueNoise.dither(image),
            Self::Sierra => Sierra.dither(image),
            Self::Burkes => Burkes.dither(image),
            Self::JarvisJudiceNinke => JarvisJudiceNinke.dither(image),
            Self::Riemersma => Riemersma.dither(image),
            Self::ClusteredDot => ClusteredDot.dither(image),
            Self::Sauvola => Sauvola.dither(image),
            Self::Ostromoukhov => Ostromoukhov.dither(image),
            Self::Dbs => DirectBinarySearch.dither(image),
        }
    }

    pub fn supports_serpentine(self) -> bool {
        matches!(
            self,
            Self::FloydSteinberg
                | Self::Atkinson
                | Self::Stucki
                | Self::Sierra
                | Self::Burkes
                | Self::JarvisJudiceNinke
        )
    }

    pub fn apply_scanned(self, image: &GrayImage, threshold: u8, serpentine: bool) -> MonoImage {
        if !serpentine {
            return self.apply(image, threshold);
        }
        let taps = match self {
            Self::FloydSteinberg => floyd_steinberg::TAPS,
            Self::Atkinson => atkinson::TAPS,
            Self::Stucki => stucki::TAPS,
            Self::Sierra => sierra::TAPS,
            Self::Burkes => burkes::TAPS,
            Self::JarvisJudiceNinke => jarvis_judice_ninke::TAPS,
            _ => return self.apply(image, threshold),
        };
        diffuse(image, taps, true)
    }
}
