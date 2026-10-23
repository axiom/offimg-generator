use image::GrayImage;

use super::{Dither, FloydSteinberg};
use crate::mono::MonoImage;

pub struct DirectBinarySearch;

// Autocorrelation of the separable [1,2,1]/4 perceptual blur; full convolution,
// including its halo, makes this same objective valid at image edges.
const CORRELATION: [f64; 5] = [1.0 / 16.0, 4.0 / 16.0, 6.0 / 16.0, 4.0 / 16.0, 1.0 / 16.0];

impl Dither for DirectBinarySearch {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        optimize(image, 4)
    }
}

fn optimize(image: &GrayImage, passes: usize) -> MonoImage {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let seed = FloydSteinberg.dither(image);
    let mut bits: Vec<f64> = seed
        .pixels()
        .as_raw()
        .iter()
        .map(|&v| f64::from(v) / 255.0)
        .collect();
    let mut gradient = vec![0.0; bits.len()];
    for y in 0..height {
        for x in 0..width {
            let error =
                bits[y * width + x] - f64::from(image.get_pixel(x as u32, y as u32)[0]) / 255.0;
            update_gradient(&mut gradient, width, height, x, y, error);
        }
    }
    // shortcut: four deterministic sweeps bound interactive work; raise the cap
    // only if measured visual gains justify longer final renders.
    for _ in 0..passes {
        let mut changed = false;
        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;
                let delta = 1.0 - 2.0 * bits[index];
                let mut best = 2.0 * delta * gradient[index] + CORRELATION[2].powi(2);
                let mut swap = None;
                for dy in -1_isize..=1 {
                    for dx in -1_isize..=1 {
                        let (Some(nx), Some(ny)) =
                            (x.checked_add_signed(dx), y.checked_add_signed(dy))
                        else {
                            continue;
                        };
                        if nx >= width || ny >= height || bits[ny * width + nx] == bits[index] {
                            continue;
                        }
                        let change = 2.0 * delta * (gradient[index] - gradient[ny * width + nx])
                            + 2.0
                                * (CORRELATION[2].powi(2)
                                    - CORRELATION[(dx + 2) as usize]
                                        * CORRELATION[(dy + 2) as usize]);
                        if change < best {
                            best = change;
                            swap = Some((nx, ny));
                        }
                    }
                }
                if best < -1e-12 {
                    bits[index] += delta;
                    update_gradient(&mut gradient, width, height, x, y, delta);
                    if let Some((nx, ny)) = swap {
                        bits[ny * width + nx] -= delta;
                        update_gradient(&mut gradient, width, height, nx, ny, -delta);
                    }
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    MonoImage::from_fn(image.width(), image.height(), |x, y| {
        bits[y as usize * width + x as usize] == 1.0
    })
}

fn update_gradient(
    gradient: &mut [f64],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    delta: f64,
) {
    for (ky, wy) in CORRELATION.into_iter().enumerate() {
        for (kx, wx) in CORRELATION.into_iter().enumerate() {
            let (Some(nx), Some(ny)) = (
                x.checked_add_signed(kx as isize - 2),
                y.checked_add_signed(ky as isize - 2),
            ) else {
                continue;
            };
            if nx < width && ny < height {
                gradient[ny * width + nx] += delta * wx * wy;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Independent full 3×3 convolution, including the off-image halo.
    fn energy(gray: &GrayImage, mono: &MonoImage) -> f64 {
        let mut total = 0.0;
        for y in -1..=i64::from(gray.height()) {
            for x in -1..=i64::from(gray.width()) {
                let mut residual = 0.0;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx >= 0
                            && ny >= 0
                            && nx < i64::from(gray.width())
                            && ny < i64::from(gray.height())
                        {
                            let weight = (if dx == 0 { 2.0 } else { 1.0 })
                                * (if dy == 0 { 2.0 } else { 1.0 })
                                / 16.0;
                            residual += weight
                                * (f64::from(mono.pixels().get_pixel(nx as u32, ny as u32)[0])
                                    - f64::from(gray.get_pixel(nx as u32, ny as u32)[0]))
                                / 255.0;
                        }
                    }
                }
                total += residual * residual;
            }
        }
        total
    }

    #[test]
    fn each_sweep_reduces_the_actual_blurred_error_including_at_edges() {
        let mut improved = false;
        for (width, height) in [(1, 1), (1, 17), (17, 1), (3, 5), (16, 13)] {
            let gray = GrayImage::from_fn(width, height, |x, y| {
                image::Luma([((x * 17 + y * 31 + 53) % 256) as u8])
            });
            let mut previous = energy(&gray, &optimize(&gray, 0));
            for passes in 1..=4 {
                let current = energy(&gray, &optimize(&gray, passes));
                assert!(
                    current <= previous + 1e-10,
                    "{width}×{height}: {current} > {previous}"
                );
                improved |= current < previous - 1e-10;
                previous = current;
            }
        }
        assert!(improved);
    }

    #[test]
    fn a_swap_can_improve_when_neither_individual_toggle_can() {
        let gray = GrayImage::from_raw(2, 1, vec![120, 110]).unwrap();
        let seed = optimize(&gray, 0);
        assert_eq!(seed.pixels().as_raw(), &[0, 255]);
        let initial = energy(&gray, &seed);
        for white in [false, true] {
            assert!(energy(&gray, &MonoImage::from_fn(2, 1, |_, _| white)) > initial);
        }
        let optimized = optimize(&gray, 1);
        assert_eq!(optimized.pixels().as_raw(), &[255, 0]);
        assert!(energy(&gray, &optimized) < initial);
    }
}
