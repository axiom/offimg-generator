use std::io::Cursor;

use image::{DynamicImage, GrayImage, ImageFormat, Luma, Rgb, RgbImage, Rgba, RgbaImage};
use offimg_generator::{
    HEIGHT, WIDTH, bmp,
    dither::{Dither, Threshold},
    mono::MonoImage,
    pipeline::{Crop, CropBounds, Framing, MAX_DIMENSION, Settings, Source},
};

fn encoded(image: impl Into<DynamicImage>, format: ImageFormat) -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());
    image.into().write_to(&mut bytes, format).unwrap();
    bytes.into_inner()
}

fn source(image: impl Into<DynamicImage>) -> Source {
    Source::decode(Cursor::new(encoded(image, ImageFormat::Png))).unwrap()
}

#[test]
fn fit_has_white_margins_and_crop_uses_the_center() {
    let image = source(RgbImage::from_fn(800, 240, |x, _| {
        Rgb([if (200..600).contains(&x) { 0 } else { 255 }; 3])
    }));
    let fit = image.prepare(Framing::Fit);
    assert_eq!(fit.dimensions(), (WIDTH, HEIGHT));
    assert_eq!(fit.get_pixel(200, 0)[0], 255);
    assert_eq!(fit.get_pixel(200, 239)[0], 255);
    assert_eq!(fit.get_pixel(200, 120)[0], 0);
    let crop = image.prepare(Framing::Crop);
    assert!(crop.pixels().all(|pixel| pixel[0] == 0));

    let portrait = source(RgbImage::from_pixel(240, 800, Rgb([0; 3])));
    assert_eq!(portrait.prepare(Framing::Fit).get_pixel(0, 120)[0], 255);
    assert!(portrait.prepare(Framing::Crop).pixels().all(|p| p[0] == 0));
    for (width, height) in [(1, MAX_DIMENSION), (MAX_DIMENSION, 1), (1, 1)] {
        let thin = source(RgbImage::from_pixel(width, height, Rgb([0; 3])));
        assert_eq!(thin.prepare(Framing::Crop).dimensions(), (WIDTH, HEIGHT));
    }
}

#[test]
fn crop_bounds_clamp_pan_and_zoom_without_empty_selections() {
    assert_eq!(
        Crop::default().bounds(800, 240).unwrap(),
        CropBounds {
            x: 200,
            y: 0,
            width: 400,
            height: 240
        }
    );
    assert_eq!(
        Crop::default().bounds(240, 800).unwrap(),
        CropBounds {
            x: 0,
            y: 328,
            width: 240,
            height: 144
        }
    );
    for (width, height) in [
        (800, 240),
        (240, 800),
        (3, 7),
        (1, 1),
        (1, MAX_DIMENSION),
        (MAX_DIMENSION, 1),
    ] {
        for zoom in [1.0, 1.5, 2.0, 16.0] {
            for (center_x, center_y) in [(-10.0, -10.0), (0.5, 0.5), (10.0, 10.0)] {
                let crop = Crop {
                    center_x,
                    center_y,
                    zoom,
                };
                let bounds = crop.bounds(width, height).unwrap();
                assert!(bounds.width > 0 && bounds.height > 0);
                assert!(bounds.x + bounds.width <= width && bounds.y + bounds.height <= height);
                let panned = crop
                    .pan(1_000_000.0, 1_000_000.0, width, height)
                    .unwrap()
                    .bounds(width, height)
                    .unwrap();
                assert_eq!((panned.x, panned.y), (0, 0));
                let panned = crop
                    .pan(-1_000_000.0, -1_000_000.0, width, height)
                    .unwrap()
                    .bounds(width, height)
                    .unwrap();
                assert_eq!(
                    (panned.x + panned.width, panned.y + panned.height),
                    (width, height)
                );
            }
        }
    }
    let crop = Crop {
        zoom: 2.0,
        ..Crop::default()
    };
    let before = crop.bounds(800, 240).unwrap();
    assert_eq!((before.width, before.height), (200, 120));
    let panned = crop
        .pan(0.25, -0.5, 800, 240)
        .unwrap()
        .bounds(800, 240)
        .unwrap();
    assert_eq!((panned.x, panned.y), (before.x - 50, before.y + 60));
    let zoomed = Crop { zoom: 4.0, ..crop }.bounds(800, 240).unwrap();
    assert_eq!(
        (zoomed.x + zoomed.width / 2, zoomed.y + zoomed.height / 2),
        (400, 120)
    );
}

#[test]
fn invalid_crop_inputs_are_rejected_at_shared_boundaries() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for crop in [
            Crop {
                center_x: value,
                ..Crop::default()
            },
            Crop {
                center_y: value,
                ..Crop::default()
            },
            Crop {
                zoom: value,
                ..Crop::default()
            },
        ] {
            assert!(crop.bounds(800, 240).is_err());
        }
        assert!(Crop::default().pan(value, 0.0, 800, 240).is_err());
        assert!(Crop::default().pan(0.0, value, 800, 240).is_err());
    }
    for zoom in [0.0, 0.99, 16.01] {
        assert!(
            Crop {
                zoom,
                ..Crop::default()
            }
            .bounds(800, 240)
            .is_err()
        );
    }
    assert!(Crop::default().bounds(0, 240).is_err());
    assert!(Crop::default().bounds(400, 0).is_err());
}

#[test]
fn crop_samples_selected_original_pixels_and_preserves_legacy_centered_output() {
    let original = source(RgbImage::from_fn(1200, 720, |x, y| {
        Rgb([if (400..800).contains(&x) && (120..360).contains(&y) {
            0
        } else {
            255
        }; 3])
    }));
    let crop = Crop {
        center_x: 0.5,
        center_y: 1.0 / 3.0,
        zoom: 3.0,
    };
    assert_eq!(
        crop.bounds(1200, 720).unwrap(),
        CropBounds {
            x: 400,
            y: 120,
            width: 400,
            height: 240
        }
    );
    assert!(
        original
            .prepare_crop(crop)
            .unwrap()
            .pixels()
            .all(|p| p[0] == 0)
    );
    assert!(
        original
            .prepare_crop(Crop {
                center_x: 0.0,
                center_y: 0.0,
                ..crop
            })
            .unwrap()
            .pixels()
            .all(|p| p[0] == 255)
    );
    for (width, height) in [(801, 241), (241, 801), (3, 7), (1, 1)] {
        let original = source(RgbImage::from_fn(width, height, |x, y| {
            Rgb([((x * 17 + y * 29) % 256) as u8; 3])
        }));
        let (cw, ch) =
            if u64::from(width) * u64::from(HEIGHT) > u64::from(height) * u64::from(WIDTH) {
                (height * WIDTH / HEIGHT, height)
            } else {
                (width, (width * HEIGHT / WIDTH).max(1))
            };
        let region = image::imageops::crop_imm(
            original.image(),
            (width - cw) / 2,
            (height - ch) / 2,
            cw,
            ch,
        )
        .to_image();
        let legacy = DynamicImage::ImageRgb8(image::imageops::resize(
            &region,
            WIDTH,
            HEIGHT,
            image::imageops::FilterType::Lanczos3,
        ))
        .into_luma8();
        assert_eq!(original.prepare_crop(Crop::default()).unwrap(), legacy);
        assert_eq!(original.prepare(Framing::Crop), legacy);
    }
}

#[test]
fn transparency_is_composited_on_white_before_resizing() {
    let image = source(RgbaImage::from_fn(3, 1, |x, _| {
        [
            Rgba([0, 0, 0, 0]),
            Rgba([0, 0, 0, 128]),
            Rgba([255, 0, 0, 255]),
        ][x as usize]
    }));
    assert_eq!(image.image().get_pixel(0, 0).0, [255; 3]);
    assert_eq!(image.image().get_pixel(1, 0).0, [127; 3]);
    assert_eq!(image.image().get_pixel(2, 0).0, [255, 0, 0]);
}

#[test]
fn jpeg_exif_orientation_is_applied_during_decode() {
    let image = RgbImage::from_fn(16, 8, |x, _| Rgb([if x < 8 { 0 } else { 255 }; 3]));
    let jpeg = encoded(image, ImageFormat::Jpeg);
    for orientation in 1..=8u8 {
        // Minimal little-endian TIFF IFD with the EXIF orientation tag.
        let mut exif = b"Exif\0\0II\x2a\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0".to_vec();
        exif.extend_from_slice(&[orientation, 0, 0, 0, 0, 0, 0, 0]);
        let mut oriented = vec![0xff, 0xd8, 0xff, 0xe1];
        oriented.extend_from_slice(&((exif.len() + 2) as u16).to_be_bytes());
        oriented.extend(exif);
        oriented.extend_from_slice(&jpeg[2..]);
        let decoded = Source::decode(Cursor::new(oriented)).unwrap();
        let mut expected = image::load_from_memory(&jpeg).unwrap();
        expected.apply_orientation(image::metadata::Orientation::from_exif(orientation).unwrap());
        let expected = expected.into_rgb8();
        assert_eq!(decoded.image(), &expected);
        let crop = Crop {
            center_x: 0.0,
            center_y: 1.0,
            zoom: 2.0,
        };
        let bounds = crop.bounds(expected.width(), expected.height()).unwrap();
        let region =
            image::imageops::crop_imm(&expected, bounds.x, bounds.y, bounds.width, bounds.height)
                .to_image();
        let selected = DynamicImage::ImageRgb8(image::imageops::resize(
            &region,
            WIDTH,
            HEIGHT,
            image::imageops::FilterType::Lanczos3,
        ))
        .into_luma8();
        assert_eq!(decoded.prepare_crop(crop).unwrap(), selected);
    }
    let rotated = {
        let mut expected = image::load_from_memory(&jpeg).unwrap();
        expected.apply_orientation(image::metadata::Orientation::Rotate90);
        expected.into_rgb8()
    };
    assert_eq!(rotated.dimensions(), (8, 16));
    assert!(rotated.get_pixel(0, 0)[0] < 10);
    assert!(rotated.get_pixel(0, 15)[0] > 245);
}

#[test]
fn malformed_unsupported_and_oversized_inputs_fail_actionably() {
    let error = Source::decode(Cursor::new(b"not an image"))
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("PNG, JPEG, BMP, or WebP"));
    let mut broken_png = b"\x89PNG\r\n\x1a\n".to_vec();
    broken_png.extend_from_slice(&[0; 32]);
    assert!(Source::decode(Cursor::new(broken_png)).is_err());
    let mut bytes = Vec::new();
    bmp::write(&MonoImage::from_fn(WIDTH, HEIGHT, |_, _| false), &mut bytes).unwrap();
    for (width, height) in [(MAX_DIMENSION + 1, 1u32), (6000, 6000)] {
        bytes[18..22].copy_from_slice(&width.to_le_bytes());
        bytes[22..26].copy_from_slice(&height.to_le_bytes());
        let error = Source::decode(Cursor::new(&bytes))
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("resize it first"), "{error}");
    }
}

#[test]
fn enabled_decoders_accept_all_four_input_formats() {
    for format in [
        ImageFormat::Png,
        ImageFormat::Jpeg,
        ImageFormat::Bmp,
        ImageFormat::WebP,
    ] {
        let bytes = encoded(RgbImage::from_pixel(2, 3, Rgb([0; 3])), format);
        let decoded = Source::decode(Cursor::new(bytes)).unwrap();
        assert_eq!(decoded.image().dimensions(), (2, 3));
    }
}

#[test]
fn threshold_is_deterministic_and_only_produces_black_and_white() {
    let gray = GrayImage::from_fn(256, 1, |x, _| Luma([x as u8]));
    let output = Threshold(128).dither(&gray);
    assert_eq!(output, Threshold(128).dither(&gray));
    for x in 0..256 {
        assert_eq!(
            output.pixels().get_pixel(x, 0)[0],
            if x < 128 { 0 } else { 255 }
        );
    }
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

#[test]
fn sample_palette_is_interpreted_instead_of_assuming_bit_polarity() {
    let bytes = include_bytes!("fixtures/magic-numbers.bmp");
    let decoded = Source::decode(Cursor::new(bytes)).unwrap();
    assert_eq!(decoded.image().dimensions(), (WIDTH, HEIGHT));
    let offset = u32_at(bytes, 10) as usize;
    let mut counts = [0; 2];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let byte = bytes[offset + (HEIGHT - 1 - y) as usize * 52 + x as usize / 8];
            let index = ((byte >> (7 - x % 8)) & 1) as usize;
            let palette = &bytes[122 + 4 * index..125 + 4 * index];
            assert_eq!(
                decoded.image().get_pixel(x, y).0,
                [palette[2], palette[1], palette[0]]
            );
            counts[index] += 1;
        }
    }
    assert!(counts.into_iter().all(|count| count > 0));
}

#[test]
fn bmp_headers_bit_order_row_order_palette_padding_and_round_trip() {
    let pattern = MonoImage::from_fn(WIDTH, HEIGHT, |x, y| {
        (y == 0 && [0, 7, 8, 399].contains(&x)) || (y == 239 && x == 1) || (x == 73 && y == 91)
    });
    let mut bytes = Vec::new();
    bmp::write(&pattern, &mut bytes).unwrap();
    assert_eq!(bytes.len(), 12_610);
    assert_eq!(&bytes[..2], b"BM");
    for (offset, value) in [
        (2, 12_610),
        (10, 130),
        (14, 108),
        (18, 400),
        (22, 240),
        (30, 0),
        (34, 12_480),
        (46, 2),
    ] {
        assert_eq!(u32_at(&bytes, offset), value);
    }
    assert_eq!(&bytes[26..30], &[1, 0, 1, 0]);
    assert_eq!(&bytes[122..130], &[0, 0, 0, 0, 255, 255, 255, 0]);
    assert_eq!(bytes[130], 0x40);
    let top = 130 + 239 * 52;
    assert_eq!(&bytes[top..top + 2], &[0x81, 0x80]);
    assert_eq!(bytes[top + 49], 1);
    for row in bytes[130..].as_chunks::<52>().0 {
        assert_eq!(&row[50..], &[0, 0]);
    }
    // The general-purpose image decoder is independent of our dedicated writer.
    let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::Bmp)
        .unwrap()
        .into_luma8();
    assert_eq!(&decoded, pattern.pixels());
    assert!(bmp::write(&MonoImage::from_fn(1, 1, |_, _| true), Vec::new()).is_err());
}

#[test]
fn conversion_matches_the_prepared_grayscale() {
    let original = source(RgbImage::from_fn(400, 240, |x, _| Rgb([x as u8; 3])));
    let settings = Settings::default();
    assert_eq!(
        original.convert(&settings).unwrap(),
        settings
            .render(&original.prepare(settings.framing))
            .unwrap()
    );
}
