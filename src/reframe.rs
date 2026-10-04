use crate::model::{Frame, Point, ReframeResult, ReframeTarget};
use image::{DynamicImage, GenericImageView};

pub fn smart_reframe(
    image: &DynamicImage,
    targets: &[ReframeTarget],
    focal_override: Option<Point>,
) -> Vec<ReframeResult> {
    let (width, height) = image.dimensions();
    let (auto_focal, saliency_score) = saliency_focal(image);
    let focal = focal_override.unwrap_or(auto_focal);
    targets
        .iter()
        .map(|target| ReframeResult {
            format: target.id.clone(),
            crop: crop_geometry(width, height, target.width, target.height, focal),
            focal,
            score: saliency_score,
        })
        .collect()
}

fn saliency_focal(image: &DynamicImage) -> (Point, f32) {
    use image::imageops::FilterType;

    let analysis = image.resize(256, 256, FilterType::Triangle).to_rgb8();
    let (width, height) = analysis.dimensions();
    if width < 2 || height < 2 {
        return (Point { x: 0.5, y: 0.5 }, 0.0);
    }

    let luma = |pixel: &image::Rgb<u8>| -> f32 {
        (0.2126 * f32::from(pixel[0]) + 0.7152 * f32::from(pixel[1]) + 0.0722 * f32::from(pixel[2]))
            / 255.0
    };
    let mut samples = Vec::with_capacity((width * height) as usize);
    let mut max_score = 0.0f32;
    for y in 1..height {
        for x in 1..width {
            let pixel = analysis.get_pixel(x, y);
            let current = luma(pixel);
            let edge = (current - luma(analysis.get_pixel(x - 1, y))).abs()
                + (current - luma(analysis.get_pixel(x, y - 1))).abs();
            let max_channel = f32::from(*pixel.0.iter().max().unwrap_or(&0));
            let min_channel = f32::from(*pixel.0.iter().min().unwrap_or(&0));
            let saturation = (max_channel - min_channel) / 255.0;
            let nx = x as f32 / (width - 1) as f32;
            let ny = y as f32 / (height - 1) as f32;
            let third_x = (nx - 1.0 / 3.0).abs().min((nx - 2.0 / 3.0).abs());
            let third_y = (ny - 1.0 / 3.0).abs().min((ny - 2.0 / 3.0).abs());
            let thirds = (1.0 - ((third_x + third_y) * 1.5).min(1.0)) * 0.12 + 1.0;
            let score = (edge * 0.72 + saturation * 0.28) * thirds;
            max_score = max_score.max(score);
            samples.push((nx, ny, score));
        }
    }

    if max_score <= f32::EPSILON {
        return (Point { x: 0.5, y: 0.5 }, 0.0);
    }

    let threshold = max_score * 0.35;
    let mut sum_weight = 0.0f32;
    let mut sum_x = 0.0f32;
    let mut sum_y = 0.0f32;
    for (x, y, score) in samples {
        if score < threshold {
            continue;
        }
        let weight = score - threshold + max_score * 0.02;
        sum_weight += weight;
        sum_x += x * weight;
        sum_y += y * weight;
    }

    if sum_weight <= f32::EPSILON {
        return (Point { x: 0.5, y: 0.5 }, 0.0);
    }
    (
        Point {
            x: (sum_x / sum_weight).clamp(0.0, 1.0),
            y: (sum_y / sum_weight).clamp(0.0, 1.0),
        },
        max_score.min(1.0),
    )
}

fn crop_geometry(
    source_width: u32,
    source_height: u32,
    target_width: u32,
    target_height: u32,
    focal: Point,
) -> Frame {
    if source_width == 0 || source_height == 0 || target_width == 0 || target_height == 0 {
        return Frame {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        };
    }
    let source_ratio = source_width as f32 / source_height as f32;
    let target_ratio = target_width as f32 / target_height as f32;
    let (width, height) = if source_ratio > target_ratio {
        (target_ratio / source_ratio, 1.0)
    } else {
        (1.0, source_ratio / target_ratio)
    };
    let fx = focal.x.clamp(0.0, 1.0);
    let fy = focal.y.clamp(0.0, 1.0);
    let x = (fx - width / 2.0).clamp(0.0, 1.0 - width);
    let y = (fy - height / 2.0).clamp(0.0, 1.0 - height);
    Frame {
        x,
        y,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ReframeTarget;
    use image::{DynamicImage, Rgba, RgbaImage};

    fn salient_image() -> DynamicImage {
        let mut image = RgbaImage::from_pixel(120, 80, Rgba([35, 35, 35, 255]));
        for y in 26..54 {
            for x in 82..112 {
                let v = if (x + y) % 2 == 0 { 255 } else { 20 };
                image.put_pixel(x, y, Rgba([v, 80, 255 - v, 255]));
            }
        }
        DynamicImage::ImageRgba8(image)
    }

    fn assert_bounded(frame: crate::model::Frame) {
        assert!((0.0..=1.0).contains(&frame.x));
        assert!((0.0..=1.0).contains(&frame.y));
        assert!((0.0..=1.0).contains(&frame.width));
        assert!((0.0..=1.0).contains(&frame.height));
        assert!(frame.x + frame.width <= 1.000_001);
        assert!(frame.y + frame.height <= 1.000_001);
    }

    #[test]
    fn crop_geometry_matches_target_ratio() {
        let image = DynamicImage::new_rgb8(120, 80);
        let targets = vec![
            ReframeTarget {
                id: "square".into(),
                width: 1,
                height: 1,
            },
            ReframeTarget {
                id: "vertical".into(),
                width: 9,
                height: 16,
            },
        ];
        let results = smart_reframe(&image, &targets, None);
        assert_eq!(results.len(), 2);
        assert!((results[0].crop.width - (2.0 / 3.0)).abs() < 0.001);
        assert!((results[0].crop.height - 1.0).abs() < 0.001);
        assert!((results[1].crop.width - 0.375).abs() < 0.001);
        assert!((results[1].crop.height - 1.0).abs() < 0.001);
        for result in results {
            assert_bounded(result.crop);
        }
    }

    #[test]
    fn focal_override_is_respected() {
        let image = DynamicImage::new_rgb8(120, 80);
        let targets = vec![ReframeTarget {
            id: "square".into(),
            width: 1,
            height: 1,
        }];
        let focal = Point { x: 0.9, y: 0.1 };
        let result = smart_reframe(&image, &targets, Some(focal));
        assert_eq!(result[0].focal, focal);
        assert_bounded(result[0].crop);
    }

    #[test]
    fn extreme_ratios_stay_bounded_and_deterministic() {
        let image = salient_image();
        let targets = vec![
            ReframeTarget {
                id: "ultra-wide".into(),
                width: 10_000,
                height: 200,
            },
            ReframeTarget {
                id: "ultra-tall".into(),
                width: 200,
                height: 10_000,
            },
        ];
        let first = smart_reframe(&image, &targets, None);
        let second = smart_reframe(&image, &targets, None);
        assert_eq!(first, second);
        for result in first {
            assert_bounded(result.crop);
            assert!(result.crop.width > 0.0);
            assert!(result.crop.height > 0.0);
        }
    }

    #[test]
    fn finds_salient_region() {
        let targets = vec![
            ReframeTarget {
                id: "square".into(),
                width: 1,
                height: 1,
            },
            ReframeTarget {
                id: "vertical".into(),
                width: 9,
                height: 16,
            },
        ];
        let results = smart_reframe(&salient_image(), &targets, None);
        assert_eq!(results.len(), 2);
        for result in &results {
            assert_bounded(result.crop);
            assert!(result.crop.x <= 0.78);
            assert!(result.crop.x + result.crop.width >= 0.78);
            assert!(result.crop.y <= 0.5);
            assert!(result.crop.y + result.crop.height >= 0.5);
        }
        assert!(results[0].focal.x > 0.6);
    }
}
