use image::GrayImage;

use crate::mono::MonoImage;

mod dbs;
mod ostromoukhov;
mod sauvola;
pub use dbs::DirectBinarySearch;
pub use ostromoukhov::Ostromoukhov;
pub use sauvola::Sauvola;

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

impl Atkinson {
    // Six eighths intentionally retain only three quarters of the error.
    const TAPS: &'static [(isize, usize, f32)] = &[
        (1, 0, 1.0 / 8.0),
        (2, 0, 1.0 / 8.0),
        (-1, 1, 1.0 / 8.0),
        (0, 1, 1.0 / 8.0),
        (1, 1, 1.0 / 8.0),
        (0, 2, 1.0 / 8.0),
    ];
}

impl Dither for Atkinson {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        diffuse(image, Self::TAPS, false)
    }
}

pub struct Stucki;

impl Stucki {
    const TAPS: &'static [(isize, usize, f32)] = &[
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
}

impl Dither for Stucki {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        diffuse(image, Self::TAPS, false)
    }
}

pub struct Sierra;

impl Sierra {
    const TAPS: &'static [(isize, usize, f32)] = &[
        (1, 0, 5.0 / 32.0),
        (2, 0, 3.0 / 32.0),
        (-2, 1, 2.0 / 32.0),
        (-1, 1, 4.0 / 32.0),
        (0, 1, 5.0 / 32.0),
        (1, 1, 4.0 / 32.0),
        (2, 1, 2.0 / 32.0),
        (-1, 2, 2.0 / 32.0),
        (0, 2, 3.0 / 32.0),
        (1, 2, 2.0 / 32.0),
    ];
}

impl Dither for Sierra {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        diffuse(image, Self::TAPS, false)
    }
}

pub struct Burkes;

impl Burkes {
    const TAPS: &'static [(isize, usize, f32)] = &[
        (1, 0, 8.0 / 32.0),
        (2, 0, 4.0 / 32.0),
        (-2, 1, 2.0 / 32.0),
        (-1, 1, 4.0 / 32.0),
        (0, 1, 8.0 / 32.0),
        (1, 1, 4.0 / 32.0),
        (2, 1, 2.0 / 32.0),
    ];
}

impl Dither for Burkes {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        diffuse(image, Self::TAPS, false)
    }
}

pub struct JarvisJudiceNinke;

impl JarvisJudiceNinke {
    const TAPS: &'static [(isize, usize, f32)] = &[
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
}

impl Dither for JarvisJudiceNinke {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        diffuse(image, Self::TAPS, false)
    }
}

fn diffuse(image: &GrayImage, taps: &[(isize, usize, f32)], serpentine: bool) -> MonoImage {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let mut values: Vec<f32> = image.as_raw().iter().map(|&v| f32::from(v)).collect();
    for y in 0..height {
        let reverse = serpentine && y % 2 == 1;
        for step in 0..width {
            let x = if reverse { width - 1 - step } else { step };
            let index = y * width + x;
            let old = values[index];
            let new = if old >= 127.5 { 255.0 } else { 0.0 };
            values[index] = new;
            let error = old - new;
            for &(dx, dy, weight) in taps {
                if let Some(nx) = x.checked_add_signed(if reverse { -dx } else { dx })
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

pub struct ClusteredDot;

impl Dither for ClusteredDot {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        const MATRIX: [[u32; 4]; 4] =
            [[12, 5, 6, 13], [4, 0, 1, 7], [11, 3, 2, 8], [15, 10, 9, 14]];
        MonoImage::from_fn(image.width(), image.height(), |x, y| {
            let level = MATRIX[y as usize % 4][x as usize % 4];
            u32::from(image.get_pixel(x, y)[0]) * 32 >= (2 * level + 1) * 255
        })
    }
}

pub struct Riemersma;

impl Dither for Riemersma {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        let mut pixels = GrayImage::new(image.width(), image.height());
        // Newest first: 16 raw quantization errors, exponentially decaying to 1/16.
        let weights: [f32; 16] = std::array::from_fn(|age| 16.0_f32.powf(-(age as f32) / 15.0));
        let mut errors = [0.0; 16];
        let side = u64::from(image.width().max(image.height())).next_power_of_two() as i64;
        visit_hilbert(
            (0, 0),
            (1, 0),
            (0, 1),
            side,
            image.dimensions(),
            &mut |x, y| {
                let source = f32::from(image.get_pixel(x, y)[0]);
                let adjusted = source + errors.iter().zip(weights).map(|(e, w)| e * w).sum::<f32>();
                let new = if adjusted >= 127.5 { 255 } else { 0 };
                pixels.put_pixel(x, y, image::Luma([new]));
                errors.rotate_right(1);
                // Store source - output, not adjusted - output: history is already weighted.
                errors[0] = source - f32::from(new);
            },
        );
        MonoImage::from_fn(image.width(), image.height(), |x, y| {
            pixels.get_pixel(x, y)[0] == 255
        })
    }
}

// Clip whole quadrants, so a narrow rectangle does not traverse a huge padded square.
fn visit_hilbert(
    (x, y): (i64, i64),
    a: (i64, i64),
    b: (i64, i64),
    side: i64,
    bounds: (u32, u32),
    visit: &mut impl FnMut(u32, u32),
) {
    let far_x = x + (a.0 + b.0) * (side - 1);
    let far_y = y + (a.1 + b.1) * (side - 1);
    if x.min(far_x) >= i64::from(bounds.0) || y.min(far_y) >= i64::from(bounds.1) {
        return;
    }
    if side == 1 {
        visit(x as u32, y as u32);
        return;
    }
    let half = side / 2;
    visit_hilbert((x, y), b, a, half, bounds, visit);
    visit_hilbert((x + b.0 * half, y + b.1 * half), a, b, half, bounds, visit);
    visit_hilbert(
        (x + (a.0 + b.0) * half, y + (a.1 + b.1) * half),
        a,
        b,
        half,
        bounds,
        visit,
    );
    visit_hilbert(
        (
            x + a.0 * (side - 1) + b.0 * (half - 1),
            y + a.1 * (side - 1) + b.1 * (half - 1),
        ),
        (-b.0, -b.1),
        (-a.0, -a.1),
        half,
        bounds,
        visit,
    );
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
            Self::FloydSteinberg => &[
                (1, 0, 7.0 / 16.0),
                (-1, 1, 3.0 / 16.0),
                (0, 1, 5.0 / 16.0),
                (1, 1, 1.0 / 16.0),
            ],
            Self::Atkinson => Atkinson::TAPS,
            Self::Stucki => Stucki::TAPS,
            Self::Sierra => Sierra::TAPS,
            Self::Burkes => Burkes::TAPS,
            Self::JarvisJudiceNinke => JarvisJudiceNinke::TAPS,
            _ => return self.apply(image, threshold),
        };
        diffuse(image, taps, true)
    }
}

#[cfg(test)]
mod tests {
    use super::visit_hilbert;

    #[test]
    fn hilbert_order_clipping_and_coverage() {
        let mut points = Vec::new();
        visit_hilbert((0, 0), (1, 0), (0, 1), 4, (4, 4), &mut |x, y| {
            points.push((x, y))
        });
        assert_eq!(
            points,
            vec![
                (0, 0),
                (1, 0),
                (1, 1),
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 3),
                (1, 2),
                (2, 2),
                (2, 3),
                (3, 3),
                (3, 2),
                (3, 1),
                (2, 1),
                (2, 0),
                (3, 0),
            ]
        );
        let mut clipped = Vec::new();
        visit_hilbert((0, 0), (1, 0), (0, 1), 4, (3, 2), &mut |x, y| {
            clipped.push((x, y))
        });
        assert_eq!(
            clipped,
            points
                .into_iter()
                .filter(|&(x, y)| x < 3 && y < 2)
                .collect::<Vec<_>>()
        );
        for (width, height) in [
            (0, 0),
            (0, 7),
            (7, 0),
            (1, 1),
            (2, 2),
            (8, 8),
            (400, 240),
            (1, 16384),
            (16384, 1),
        ] {
            let mut visited = vec![false; width as usize * height as usize];
            let side = u64::from(width.max(height)).next_power_of_two() as i64;
            let mut previous = None;
            visit_hilbert(
                (0, 0),
                (1, 0),
                (0, 1),
                side,
                (width, height),
                &mut |x, y| {
                    assert!(x < width && y < height);
                    let index = y as usize * width as usize + x as usize;
                    assert!(!visited[index]);
                    visited[index] = true;
                    if width == height
                        && let Some((px, py)) = previous
                    {
                        assert_eq!(x.abs_diff(px) + y.abs_diff(py), 1);
                    }
                    previous = Some((x, y));
                },
            );
            assert!(visited.into_iter().all(|pixel| pixel));
        }
    }
}
