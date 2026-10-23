use image::{GrayImage, Luma};
use offimg_generator::{dither::Algorithm, pipeline::Settings};

const ALGORITHMS: [Algorithm; 3] = [
    Algorithm::Threshold,
    Algorithm::FloydSteinberg,
    Algorithm::Bayer,
];

#[test]
fn algorithms_are_deterministic_binary_and_preserve_solid_black_and_white() {
    for algorithm in ALGORITHMS {
        for (width, height) in [(0, 0), (1, 1), (1, 31), (31, 1), (17, 23), (400, 240)] {
            for value in [0, 255] {
                let gray = GrayImage::from_pixel(width, height, Luma([value]));
                let mono = algorithm.apply(&gray, 128);
                assert_eq!(mono.pixels(), &gray, "{algorithm}: {width}×{height}");
            }
            let gradient = GrayImage::from_fn(width, height, |x, y| {
                Luma([((x * 17 + y * 31) % 256) as u8])
            });
            let mono = algorithm.apply(&gradient, 128);
            assert_eq!(mono, algorithm.apply(&gradient, 128));
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
    for algorithm in ALGORITHMS {
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
