use image::{GrayImage, Luma};
use offimg_generator::{dither::Algorithm, pipeline::Settings};

#[test]
fn algorithms_are_deterministic_binary_and_preserve_solid_black_and_white() {
    for algorithm in Algorithm::ALL {
        for (width, height) in [(0, 0), (1, 1), (1, 31), (31, 1), (17, 23), (400, 240)] {
            for value in [0, 255] {
                let gray = GrayImage::from_pixel(width, height, Luma([value]));
                let mono = algorithm.apply(&gray, 128);
                assert_eq!(mono.pixels(), &gray, "{algorithm}: {width}×{height}");
            }
            let gradient = GrayImage::from_fn(width, height, |x, y| {
                Luma([((x * 17 + y * 31) % 256) as u8])
            });
            let original = gradient.clone();
            let mono = algorithm.apply(&gradient, 128);
            assert_eq!(mono, algorithm.apply(&gradient, 128));
            assert_eq!(gradient, original);
            if algorithm != Algorithm::Threshold {
                assert_eq!(mono, algorithm.apply(&gradient, 0));
                assert_eq!(mono, algorithm.apply(&gradient, 255));
            }
            assert_eq!(mono.pixels().dimensions(), (width, height));
            assert!(
                mono.pixels()
                    .pixels()
                    .all(|pixel| matches!(pixel[0], 0 | 255))
            );
        }
    }
}

#[test]
fn algorithm_names_roundtrip_and_default_stays_floyd_steinberg() {
    assert_eq!(Algorithm::default(), Algorithm::FloydSteinberg);
    assert_eq!(Settings::default().algorithm, Algorithm::FloydSteinberg);
    for (algorithm, name) in Algorithm::ALL.into_iter().zip([
        "threshold",
        "floyd-steinberg",
        "bayer",
        "atkinson",
        "stucki",
        "blue-noise",
        "sierra",
        "burkes",
        "jarvis-judice-ninke",
        "riemersma",
        "clustered-dot",
        "sauvola",
        "ostromoukhov",
        "dbs",
    ]) {
        assert_eq!(algorithm.to_string(), name);
        assert_eq!(name.parse::<Algorithm>().unwrap(), algorithm);
        assert!("unknown".parse::<Algorithm>().unwrap_err().contains(name));
    }
}

#[test]
fn atkinson_and_stucki_match_small_diffusion_fixtures() {
    // Atkinson on a row of 112s: 112, 126, 141.75 -> black, black, white.
    // In a column the same first three values arise via the 1-row/2-row taps;
    // the fourth is 112 + 126/8 - 113.25/8 = 113.59375 -> black.
    // Stucki on a row: 112, 133⅓, 99.492… -> black, white, black.
    // In a column its 8/42 and 4/42 taps also leave the fourth below cutoff.
    for (width, height, value, atkinson, stucki) in [
        (1, 1, 128, vec![255], vec![255]),
        (3, 1, 112, vec![0, 0, 255], vec![0, 255, 0]),
        (1, 4, 112, vec![0, 0, 255, 0], vec![0, 255, 0, 0]),
        (2, 2, 128, vec![255, 0, 0, 255], vec![255, 0, 0, 255]),
        // Atkinson's lost quarter leaves all nine below cutoff; normalizing to
        // sixths makes (1,2) white instead. Stucki retains enough error there.
        (3, 3, 64, vec![0; 9], vec![0, 0, 0, 0, 0, 0, 0, 255, 0]),
        // Both ±2 diagonals and both future rows fit; edges discard error.
        (
            5,
            3,
            96,
            vec![0, 0, 0, 0, 0, 0, 255, 255, 0, 255, 0, 0, 0, 0, 255],
            vec![0, 0, 0, 255, 0, 255, 0, 255, 0, 0, 0, 255, 0, 0, 255],
        ),
    ] {
        let gray = GrayImage::from_pixel(width, height, Luma([value]));
        for (algorithm, expected) in [(Algorithm::Atkinson, atkinson), (Algorithm::Stucki, stucki)]
        {
            assert_eq!(
                algorithm.apply(&gray, 128).pixels().as_raw(),
                &expected,
                "{algorithm}: {width}×{height}, gray {value}"
            );
        }
    }
    // Error from the right edge must not wrap into the next row, nor may
    // down-left from x=0 wrap to the previous row's rightmost pixel.
    let gray = GrayImage::from_raw(2, 2, vec![0, 110, 110, 0]).unwrap();
    for algorithm in [Algorithm::Atkinson, Algorithm::Stucki] {
        assert_eq!(algorithm.apply(&gray, 128).pixels().as_raw(), &[0, 0, 0, 0]);
    }
    // Exactly 127.5 is white, including fractional accumulated error.
    let gray = GrayImage::from_raw(2, 1, vec![124, 112]).unwrap();
    assert_eq!(
        Algorithm::Atkinson.apply(&gray, 128).pixels().as_raw(),
        &[0, 255]
    );
    // -21 error at (0,0); all intervening taps cancel to zero, leaving
    // 128 - 21/42 = 127.5 at (2,2).
    let gray = GrayImage::from_raw(3, 3, vec![234, 4, 2, 4, 2, 1, 2, 1, 128]).unwrap();
    assert_eq!(
        Algorithm::Stucki.apply(&gray, 128).pixels().as_raw(),
        &[255, 0, 0, 0, 0, 0, 0, 0, 255]
    );
}

#[test]
fn blue_noise_uses_centered_thresholds_and_tiles_the_fixed_64_square_mask() {
    let mask = include_bytes!("../assets/blue-noise/mask.bin");
    assert_eq!(mask.len(), 64 * 64);
    assert_eq!(&mask[..8], &[65, 247, 203, 177, 54, 149, 96, 135]);
    let mut histogram = [0; 256];
    for &byte in mask {
        histogram[byte as usize] += 1;
    }
    assert_eq!(histogram, [16; 256]);
    for value in 0..=255 {
        let gray = GrayImage::from_pixel(64, 64, Luma([value]));
        let mono = Algorithm::BlueNoise.apply(&gray, 128);
        for (&byte, pixel) in mask.iter().zip(mono.pixels().pixels()) {
            let threshold = (f64::from(byte) + 0.5) * 255.0 / 256.0;
            assert!(threshold > 0.0 && threshold < 255.0);
            assert_eq!(pixel[0] == 255, f64::from(value) >= threshold);
        }
    }
    let gray = GrayImage::from_pixel(129, 131, Luma([128]));
    let mono = Algorithm::BlueNoise.apply(&gray, 128);
    for (x, y, pixel) in mono.pixels().enumerate_pixels() {
        assert_eq!(pixel, mono.pixels().get_pixel(x % 64, y % 64));
    }
    assert_eq!(mono, Algorithm::BlueNoise.apply(&gray, 128));
}

#[test]
fn sierra_burkes_and_jjn_match_diffusion_fixtures() {
    // On a row of 112s, the second pixel is respectively 129.5, 140,
    // and 128⅓: all white; all three third pixels remain below cutoff.
    for (width, height, value, sierra, burkes, jjn) in [
        (3, 1, 112, vec![0, 255, 0], vec![0, 255, 0], vec![0, 255, 0]),
        (
            1,
            4,
            112,
            vec![0, 255, 0, 0],
            vec![0, 255, 0, 255],
            vec![0, 255, 0, 0],
        ),
        (
            2,
            2,
            128,
            vec![255, 0, 0, 255],
            vec![255, 0, 0, 255],
            vec![255, 0, 0, 255],
        ),
        (
            3,
            3,
            64,
            vec![0, 0, 0, 0, 0, 0, 0, 255, 0],
            vec![0, 0, 0, 0, 0, 255, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 255, 0],
        ),
        (
            5,
            3,
            96,
            vec![0, 0, 0, 0, 0, 255, 255, 255, 0, 255, 0, 0, 0, 255, 0],
            vec![0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0, 255],
            vec![0, 0, 0, 0, 0, 255, 0, 255, 255, 0, 0, 0, 255, 0, 0],
        ),
    ] {
        let gray = GrayImage::from_pixel(width, height, Luma([value]));
        for (algorithm, expected) in [
            (Algorithm::Sierra, sierra),
            (Algorithm::Burkes, burkes),
            (Algorithm::JarvisJudiceNinke, jjn),
        ] {
            assert_eq!(
                algorithm.apply(&gray, 128).pixels().as_raw(),
                &expected,
                "{algorithm}: {width}×{height}, gray {value}"
            );
        }
    }
    let gray = GrayImage::from_raw(2, 2, vec![0, 100, 100, 0]).unwrap();
    for algorithm in [
        Algorithm::Sierra,
        Algorithm::Burkes,
        Algorithm::JarvisJudiceNinke,
    ] {
        assert_eq!(algorithm.apply(&gray, 128).pixels().as_raw(), &[0, 0, 0, 0]);
    }
}

#[test]
fn clustered_dot_grows_a_centered_cluster_and_tiles_without_endpoint_bias() {
    let gray = GrayImage::from_pixel(4, 4, Luma([64]));
    assert_eq!(
        Algorithm::ClusteredDot.apply(&gray, 128).pixels().as_raw(),
        &[0, 0, 0, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 0, 0, 0]
    );
    for value in 0..=255 {
        let gray = GrayImage::from_pixel(9, 11, Luma([value]));
        let mono = Algorithm::ClusteredDot.apply(&gray, 128);
        for (x, y, pixel) in mono.pixels().enumerate_pixels() {
            assert_eq!(pixel, mono.pixels().get_pixel(x % 4, y % 4));
        }
        let white = (0..4)
            .flat_map(|y| (0..4).map(move |x| (x, y)))
            .filter(|&(x, y)| mono.pixels().get_pixel(x, y)[0] == 255)
            .count();
        let expected = (0..16)
            .filter(|&rank| f64::from(value) >= (f64::from(rank) + 0.5) * 255.0 / 16.0)
            .count();
        assert_eq!(white, expected);
    }
    let gray = GrayImage::from_pixel(4, 4, Luma([128]));
    assert_ne!(
        Algorithm::ClusteredDot.apply(&gray, 128),
        Algorithm::Bayer.apply(&gray, 128)
    );
}

#[test]
fn riemersma_uses_hilbert_order_raw_error_and_a_finite_decaying_history() {
    let gray = GrayImage::from_pixel(2, 2, Luma([110]));
    // Hilbert visits (0,0), (0,1), (1,1), (1,0). Raw errors are
    // 110, -145, 110, -145; feeding adjusted errors back gives another pattern.
    assert_eq!(
        Algorithm::Riemersma.apply(&gray, 128).pixels().as_raw(),
        &[0, 255, 255, 0]
    );

    // Independent iterative Hilbert index decoding and oldest-first f64 history.
    // More than 16 pixels exercises expiration; clipping must not advance history.
    for (width, height) in [(8, 8), (5, 7), (1, 31), (31, 1)] {
        let gray = GrayImage::from_fn(width, height, |x, y| {
            Luma([((x * 17 + y * 31) % 256) as u8])
        });
        let side = width.max(height).next_power_of_two() as i32;
        let mut expected = GrayImage::new(width, height);
        let mut history = std::collections::VecDeque::new();
        for distance in 0..side * side {
            let (mut x, mut y, mut t, mut scale) = (0, 0, distance, 1);
            while scale < side {
                let rx = 1 & (t / 2);
                let ry = 1 & (t ^ rx);
                if ry == 0 {
                    if rx == 1 {
                        x = scale - 1 - x;
                        y = scale - 1 - y;
                    }
                    std::mem::swap(&mut x, &mut y);
                }
                x += scale * rx;
                y += scale * ry;
                t /= 4;
                scale *= 2;
            }
            if x >= width as i32 || y >= height as i32 {
                continue;
            }
            let source = f64::from(gray.get_pixel(x as u32, y as u32)[0]);
            let correction: f64 = history
                .iter()
                .rev()
                .enumerate()
                .map(|(age, error)| error * 16.0_f64.powf(-(age as f64) / 15.0))
                .sum();
            let new = if source + correction >= 127.5 { 255 } else { 0 };
            expected.put_pixel(x as u32, y as u32, Luma([new]));
            history.push_back(source - f64::from(new));
            if history.len() > 16 {
                history.pop_front();
            }
        }
        assert_eq!(
            Algorithm::Riemersma.apply(&gray, 128).pixels(),
            &expected,
            "{width}×{height}"
        );
    }
}

#[test]
fn floyd_steinberg_distributes_error_without_wrapping_at_edges() {
    for (width, height) in [(2, 1), (1, 2)] {
        let gray = GrayImage::from_pixel(width, height, Luma([128]));
        let mono = Algorithm::FloydSteinberg.apply(&gray, 128);
        assert_eq!(mono.pixels().as_raw(), &[255, 0]);
    }
    // Only the downward 5/16 term can reach the next pixel in a single column.
    let gray = GrayImage::from_pixel(1, 3, Luma([96]));
    assert_eq!(
        Algorithm::FloydSteinberg
            .apply(&gray, 128)
            .pixels()
            .as_raw(),
        &[0, 0, 255]
    );
    let gray = GrayImage::from_pixel(2, 2, Luma([128]));
    assert_eq!(
        Algorithm::FloydSteinberg
            .apply(&gray, 128)
            .pixels()
            .as_raw(),
        &[255, 0, 0, 255]
    );
}

#[test]
fn bayer_has_the_expected_repeating_four_by_four_pattern() {
    let gray = GrayImage::from_pixel(8, 8, Luma([128]));
    let mono = Algorithm::Bayer.apply(&gray, 128);
    for (x, y, pixel) in mono.pixels().enumerate_pixels() {
        assert_eq!(pixel[0], if (x + y) % 2 == 0 { 255 } else { 0 });
    }
    let gray = GrayImage::from_pixel(4, 4, Luma([32]));
    let mono = Algorithm::Bayer.apply(&gray, 128);
    assert_eq!(
        mono.pixels()
            .pixels()
            .filter(|pixel| pixel[0] == 255)
            .count(),
        2
    );
    assert_eq!(mono.pixels().get_pixel(0, 0)[0], 255);
    assert_eq!(mono.pixels().get_pixel(2, 2)[0], 255);
}

#[test]
fn neutral_adjustments_preserve_every_gray_level_and_do_not_mutate_the_input() {
    let gray = GrayImage::from_fn(256, 1, |x, _| Luma([x as u8]));
    let original = gray.clone();
    let defaults = Settings::default();
    assert_eq!(defaults.adjust(&gray).unwrap(), gray);
    let changed = Settings {
        brightness: 0.2,
        contrast: 1.4,
        gamma: 1.8,
        ..defaults.clone()
    };
    changed.render(&gray).unwrap();
    assert_eq!(defaults.adjust(&gray).unwrap(), original);
    assert_eq!(gray, original);
    for algorithm in Algorithm::ALL {
        let settings = Settings {
            algorithm,
            ..defaults.clone()
        };
        let normal = settings.render(&gray).unwrap();
        let inverse = Settings {
            invert: true,
            ..settings
        }
        .render(&gray)
        .unwrap();
        for (a, b) in normal.pixels().pixels().zip(inverse.pixels().pixels()) {
            assert_eq!(a[0], 255 - b[0]);
        }
    }
}

#[test]
fn tone_formula_ranges_and_threshold_endpoints() {
    let gray = GrayImage::from_pixel(1, 1, Luma([64]));
    let settings = Settings {
        brightness: 0.1,
        contrast: 2.0,
        gamma: 2.0,
        ..Settings::default()
    };
    // 64/255 -> contrast around 0.5 -> brightness -> clamp -> sqrt -> byte.
    let expected = (255.0_f32
        * (((64.0_f32 / 255.0 - 0.5) * 2.0 + 0.5 + 0.1).clamp(0.0, 1.0)).sqrt())
    .round() as u8;
    assert_eq!(settings.adjust(&gray).unwrap().get_pixel(0, 0)[0], expected);
    for brightness in [-1.0, 1.0] {
        for contrast in [0.0, 4.0] {
            for gamma in [0.1, 4.0] {
                assert!(
                    Settings {
                        brightness,
                        contrast,
                        gamma,
                        ..Settings::default()
                    }
                    .validate()
                    .is_ok()
                );
            }
        }
    }
    let gradient = GrayImage::from_fn(256, 1, |x, _| Luma([x as u8]));
    assert!(
        Algorithm::Threshold
            .apply(&gradient, 0)
            .pixels()
            .pixels()
            .all(|p| p[0] == 255)
    );
    let mono = Algorithm::Threshold.apply(&gradient, 255);
    assert_eq!(mono.pixels().pixels().filter(|p| p[0] == 255).count(), 1);
}

#[test]
fn invalid_and_nonfinite_controls_fail_at_shared_boundaries() {
    for (name, values) in [
        (
            "brightness",
            vec![-1.01, 1.01, f32::NAN, f32::INFINITY, f32::NEG_INFINITY],
        ),
        (
            "contrast",
            vec![-0.01, 4.01, f32::NAN, f32::INFINITY, f32::NEG_INFINITY],
        ),
        (
            "gamma",
            vec![0.0, 0.09, 4.01, f32::NAN, f32::INFINITY, f32::NEG_INFINITY],
        ),
    ] {
        for value in values {
            let mut settings = Settings::default();
            match name {
                "brightness" => settings.brightness = value,
                "contrast" => settings.contrast = value,
                "gamma" => settings.gamma = value,
                _ => unreachable!(),
            }
            assert!(settings.validate().unwrap_err().to_string().contains(name));
            assert!(settings.render(&GrayImage::new(1, 1)).is_err());
            assert!(settings.adjust(&GrayImage::new(1, 1)).is_err());
        }
    }
}
