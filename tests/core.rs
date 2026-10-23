use std::io::Cursor;

use image::{DynamicImage, GrayImage, ImageFormat, Luma, Rgb, RgbImage, Rgba, RgbaImage};
use offimg_generator::{
    HEIGHT, WIDTH, bmp,
    dither::{Dither, Threshold},
    mono::MonoImage,
    pipeline::{Framing, MAX_DIMENSION, Settings, Source},
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
        assert_eq!(decoded.image(), &expected.into_rgb8());
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
fn sample_palettes_are_interpreted_instead_of_assuming_bit_polarity() {
    for bytes in [
        include_bytes!("../numbers.bmp"),
        include_bytes!("../numbers2.bmp"),
    ] {
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
