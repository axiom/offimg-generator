use image::GrayImage;

use super::Dither;
use crate::mono::MonoImage;

pub struct Ostromoukhov;

impl Dither for Ostromoukhov {
    fn dither(&self, image: &GrayImage) -> MonoImage {
        let width = image.width() as usize;
        let height = image.height() as usize;
        let mut values: Vec<f32> = image.as_raw().iter().map(|&v| f32::from(v)).collect();
        for y in 0..height {
            let direction = if y % 2 == 0 { 1 } else { -1 };
            for step in 0..width {
                let x = if direction == 1 {
                    step
                } else {
                    width - 1 - step
                };
                let index = y * width + x;
                let source = image.as_raw()[index];
                let [right, down_left, down] = COEFFICIENTS[usize::from(source.min(255 - source))];
                let divisor = f32::from(right + down_left + down);
                let new = if values[index] >= 127.5 { 255.0 } else { 0.0 };
                let error = values[index] - new;
                values[index] = new;
                for (dx, dy, weight) in [
                    (direction, 0, right),
                    (-direction, 1, down_left),
                    (0, 1, down),
                ] {
                    if let Some(nx) = x.checked_add_signed(dx)
                        && nx < width
                        && y + dy < height
                    {
                        values[(y + dy) * width + nx] += error * (f32::from(weight) / divisor);
                    }
                }
            }
        }
        MonoImage::from_fn(image.width(), image.height(), |x, y| {
            values[y as usize * width + x as usize] == 255.0
        })
    }
}

// Original 128 low-half triples, mirrored for 128..255. Table source and MIT
// notice: assets/ostromoukhov/. Index by pre-diffusion input gray, not error.
const COEFFICIENTS: [[u16; 3]; 128] = [
    [13, 0, 5],
    [13, 0, 5],
    [21, 0, 10],
    [7, 0, 4],
    [8, 0, 5],
    [47, 3, 28],
    [23, 3, 13],
    [15, 3, 8],
    [22, 6, 11],
    [43, 15, 20],
    [7, 3, 3],
    [501, 224, 211],
    [249, 116, 103],
    [165, 80, 67],
    [123, 62, 49],
    [489, 256, 191],
    [81, 44, 31],
    [483, 272, 181],
    [60, 35, 22],
    [53, 32, 19],
    [237, 148, 83],
    [471, 304, 161],
    [3, 2, 1],
    [459, 304, 161],
    [38, 25, 14],
    [453, 296, 175],
    [225, 146, 91],
    [149, 96, 63],
    [111, 71, 49],
    [63, 40, 29],
    [73, 46, 35],
    [435, 272, 217],
    [108, 67, 56],
    [13, 8, 7],
    [213, 130, 119],
    [423, 256, 245],
    [5, 3, 3],
    [281, 173, 162],
    [141, 89, 78],
    [283, 183, 150],
    [71, 47, 36],
    [285, 193, 138],
    [13, 9, 6],
    [41, 29, 18],
    [36, 26, 15],
    [289, 213, 114],
    [145, 109, 54],
    [291, 223, 102],
    [73, 57, 24],
    [293, 233, 90],
    [21, 17, 6],
    [295, 243, 78],
    [37, 31, 9],
    [27, 23, 6],
    [149, 129, 30],
    [299, 263, 54],
    [75, 67, 12],
    [43, 39, 6],
    [151, 139, 18],
    [303, 283, 30],
    [38, 36, 3],
    [305, 293, 18],
    [153, 149, 6],
    [307, 303, 6],
    [1, 1, 0],
    [101, 105, 2],
    [49, 53, 2],
    [95, 107, 6],
    [23, 27, 2],
    [89, 109, 10],
    [43, 55, 6],
    [83, 111, 14],
    [5, 7, 1],
    [172, 181, 37],
    [97, 76, 22],
    [72, 41, 17],
    [119, 47, 29],
    [4, 1, 1],
    [4, 1, 1],
    [4, 1, 1],
    [4, 1, 1],
    [4, 1, 1],
    [4, 1, 1],
    [4, 1, 1],
    [4, 1, 1],
    [4, 1, 1],
    [65, 18, 17],
    [95, 29, 26],
    [185, 62, 53],
    [30, 11, 9],
    [35, 14, 11],
    [85, 37, 28],
    [55, 26, 19],
    [80, 41, 29],
    [155, 86, 59],
    [5, 3, 2],
    [5, 3, 2],
    [5, 3, 2],
    [5, 3, 2],
    [5, 3, 2],
    [5, 3, 2],
    [5, 3, 2],
    [5, 3, 2],
    [5, 3, 2],
    [5, 3, 2],
    [5, 3, 2],
    [5, 3, 2],
    [5, 3, 2],
    [305, 176, 119],
    [155, 86, 59],
    [105, 56, 39],
    [80, 41, 29],
    [65, 32, 23],
    [55, 26, 19],
    [335, 152, 113],
    [85, 37, 28],
    [115, 48, 37],
    [35, 14, 11],
    [355, 136, 109],
    [30, 11, 9],
    [365, 128, 107],
    [185, 62, 53],
    [25, 8, 7],
    [95, 29, 26],
    [385, 112, 103],
    [65, 18, 17],
    [395, 104, 101],
    [4, 1, 1],
];
