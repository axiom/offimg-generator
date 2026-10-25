use image::GrayImage;

use crate::mono::MonoImage;

pub(super) fn diffuse(
    image: &GrayImage,
    taps: &[(isize, usize, f32)],
    serpentine: bool,
) -> MonoImage {
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
