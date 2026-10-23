use image::{GrayImage, Luma};
use offimg_generator::{dither::Algorithm, pipeline::Settings};

#[test]
fn sauvola_matches_direct_local_statistics_in_clipped_windows() {
    for (width, height) in [(1, 1), (1, 31), (31, 1), (17, 19)] {
        let gray = GrayImage::from_fn(width, height, |x, y| {
            Luma([((x * 17 + y * 31) % 256) as u8])
        });
        let mono = Algorithm::Sauvola.apply(&gray, 0);
        for (x, y, pixel) in mono.pixels().enumerate_pixels() {
            let mut values = Vec::new();
            for ny in y.saturating_sub(7)..(y + 8).min(height) {
                for nx in x.saturating_sub(7)..(x + 8).min(width) {
                    values.push(f64::from(gray.get_pixel(nx, ny)[0]));
                }
            }
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance =
                values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
            let threshold = mean * (1.0 + 0.2 * (variance.sqrt() / 128.0 - 1.0));
            let value = gray.get_pixel(x, y)[0];
            assert_eq!(pixel[0] == 255, value != 0 && f64::from(value) >= threshold);
        }
    }
    let gray = GrayImage::from_raw(2, 1, vec![30, 180]).unwrap();
    assert_eq!(
        Algorithm::Sauvola.apply(&gray, 128).pixels().as_raw(),
        &[0, 255]
    );
}

#[test]
fn ostromoukhov_small_fixtures_use_gray_dependent_weights_and_serpentine_edges() {
    for (width, height, value, expected) in [
        // Gray 96 uses (5,3,2)/10; second corrected value is 144, hence white.
        (3, 1, 96, vec![0, 255, 0]),
        // Only the down coefficient survives in a single column.
        (1, 4, 96, vec![0, 0, 0, 0]),
        (1, 4, 128, vec![255, 0, 255, 0]),
        // Odd row runs right-to-left, retaining the pre-diffusion gray weights.
        (2, 2, 96, vec![0, 255, 0, 0]),
        (2, 2, 128, vec![255, 0, 0, 255]),
    ] {
        let gray = GrayImage::from_pixel(width, height, Luma([value]));
        assert_eq!(
            Algorithm::Ostromoukhov.apply(&gray, 128).pixels().as_raw(),
            &expected
        );
    }
}

#[test]
fn serpentine_mirrors_every_forward_tap_and_keeps_legacy_raster_output() {
    type Kernel = (Algorithm, f64, &'static [(i32, i32, f64)]);
    let kernels: &[Kernel] = &[
        (
            Algorithm::FloydSteinberg,
            16.0,
            &[(1, 0, 7.0), (-1, 1, 3.0), (0, 1, 5.0), (1, 1, 1.0)],
        ),
        (
            Algorithm::Atkinson,
            8.0,
            &[
                (1, 0, 1.0),
                (2, 0, 1.0),
                (-1, 1, 1.0),
                (0, 1, 1.0),
                (1, 1, 1.0),
                (0, 2, 1.0),
            ],
        ),
        (
            Algorithm::Stucki,
            42.0,
            &[
                (1, 0, 8.0),
                (2, 0, 4.0),
                (-2, 1, 2.0),
                (-1, 1, 4.0),
                (0, 1, 8.0),
                (1, 1, 4.0),
                (2, 1, 2.0),
                (-2, 2, 1.0),
                (-1, 2, 2.0),
                (0, 2, 4.0),
                (1, 2, 2.0),
                (2, 2, 1.0),
            ],
        ),
        (
            Algorithm::Sierra,
            32.0,
            &[
                (1, 0, 5.0),
                (2, 0, 3.0),
                (-2, 1, 2.0),
                (-1, 1, 4.0),
                (0, 1, 5.0),
                (1, 1, 4.0),
                (2, 1, 2.0),
                (-1, 2, 2.0),
                (0, 2, 3.0),
                (1, 2, 2.0),
            ],
        ),
        (
            Algorithm::Burkes,
            32.0,
            &[
                (1, 0, 8.0),
                (2, 0, 4.0),
                (-2, 1, 2.0),
                (-1, 1, 4.0),
                (0, 1, 8.0),
                (1, 1, 4.0),
                (2, 1, 2.0),
            ],
        ),
        (
            Algorithm::JarvisJudiceNinke,
            48.0,
            &[
                (1, 0, 7.0),
                (2, 0, 5.0),
                (-2, 1, 3.0),
                (-1, 1, 5.0),
                (0, 1, 7.0),
                (1, 1, 5.0),
                (2, 1, 3.0),
                (-2, 2, 1.0),
                (-1, 2, 3.0),
                (0, 2, 5.0),
                (1, 2, 3.0),
                (2, 2, 1.0),
            ],
        ),
    ];
    let mut differs = false;
    for &(algorithm, divisor, taps) in kernels {
        assert!(algorithm.supports_serpentine());
        for (width, height) in [(0, 0), (1, 9), (9, 1), (3, 5), (17, 13)] {
            let gray = GrayImage::from_fn(width, height, |x, y| {
                Luma([((x * 17 + y * 31 + 53) % 256) as u8])
            });
            assert_eq!(
                algorithm.apply_scanned(&gray, 128, false),
                algorithm.apply(&gray, 128)
            );
            let mut values: Vec<f64> = gray.as_raw().iter().map(|&v| f64::from(v)).collect();
            for y in 0..height as i32 {
                for step in 0..width as i32 {
                    let direction = if y % 2 == 0 { 1 } else { -1 };
                    let x = if direction == 1 {
                        step
                    } else {
                        width as i32 - 1 - step
                    };
                    let index = y as usize * width as usize + x as usize;
                    let new = if values[index] >= 127.5 { 255.0 } else { 0.0 };
                    let error = values[index] - new;
                    values[index] = new;
                    for &(dx, dy, weight) in taps {
                        let (nx, ny) = (x + direction * dx, y + dy);
                        if nx >= 0 && nx < width as i32 && ny < height as i32 {
                            values[ny as usize * width as usize + nx as usize] +=
                                error * weight / divisor;
                        }
                    }
                }
            }
            let actual = algorithm.apply_scanned(&gray, 128, true);
            assert_eq!(
                actual.pixels().as_raw(),
                &values.into_iter().map(|v| v as u8).collect::<Vec<_>>(),
                "{algorithm}: {width}×{height}"
            );
            differs |= actual != algorithm.apply(&gray, 128);
        }
    }
    assert!(differs);
}

#[test]
fn mild_sharpening_is_opt_in_fresh_and_precedes_tone_adjustment() {
    let defaults = Settings::default();
    assert!(!defaults.sharpen && !defaults.serpentine);
    let gray = GrayImage::from_raw(3, 1, vec![64, 128, 64]).unwrap();
    let original = gray.clone();
    let sharp = Settings {
        sharpen: true,
        ..defaults.clone()
    };
    assert_eq!(sharp.adjust(&gray).unwrap().as_raw(), &[56, 144, 56]);
    let fractional = GrayImage::from_raw(3, 1, vec![64, 129, 64]).unwrap();
    assert_eq!(sharp.adjust(&fractional).unwrap().as_raw(), &[56, 145, 56]);
    let extremes = GrayImage::from_raw(3, 1, vec![0, 255, 0]).unwrap();
    assert_eq!(sharp.adjust(&extremes).unwrap(), extremes);
    assert_eq!(sharp.adjust(&gray).unwrap(), sharp.adjust(&gray).unwrap());
    let changed = Settings {
        brightness: -0.1,
        contrast: 1.3,
        gamma: 1.7,
        ..sharp.clone()
    };
    let tone_only = Settings {
        sharpen: false,
        ..changed.clone()
    };
    assert_eq!(
        changed.adjust(&gray).unwrap(),
        tone_only.adjust(&sharp.adjust(&gray).unwrap()).unwrap()
    );
    assert_eq!(gray, original);
    assert_eq!(defaults.adjust(&gray).unwrap(), original);
    for value in [0, 64, 128, 255] {
        let gray = GrayImage::from_pixel(3, 5, Luma([value]));
        assert_eq!(sharp.adjust(&gray).unwrap(), gray);
    }
    assert_eq!(
        sharp.adjust(&GrayImage::new(0, 0)).unwrap().dimensions(),
        (0, 0)
    );
    for algorithm in Algorithm::ALL {
        let settings = Settings {
            algorithm,
            serpentine: true,
            sharpen: true,
            ..defaults.clone()
        };
        assert_eq!(
            settings.render(&gray).unwrap(),
            algorithm.apply_scanned(&settings.adjust(&gray).unwrap(), 128, true)
        );
        if !algorithm.supports_serpentine() {
            assert_eq!(
                algorithm.apply_scanned(&gray, 128, true),
                algorithm.apply(&gray, 128)
            );
        }
    }
}
