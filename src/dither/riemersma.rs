use image::GrayImage;

use super::Dither;
use crate::mono::MonoImage;

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
