//! Core dithering algorithm implementation.
//!
//! This module provides the ordered (Bayer) dithering algorithm that converts
//! full-color images to a limited color palette while maintaining the illusion
//! of more colors through spatial patterns.
//!
//! # Algorithm Overview
//!
//! Ordered dithering works by comparing each pixel's "distance" between two
//! palette colors against a threshold from the Bayer matrix. The threshold
//! varies spatially in a regular pattern, creating the characteristic
//! cross-hatch patterns of dithered images.
//!
//! For each pixel:
//! 1. Find the two nearest colors in the palette
//! 2. Calculate a weight based on how close the pixel is to each color
//! 3. Compare this weight against the Bayer threshold at that position
//! 4. Choose the nearest or second-nearest color based on the comparison

use crate::palette::Rgb;
use image::{DynamicImage, GenericImageView, ImageBuffer, RgbImage};

/// Calculates the squared Euclidean distance between two colors in RGB space.
///
/// Using squared distance avoids an expensive square root operation while
/// maintaining the same ordering for comparisons.
///
/// # Arguments
///
/// * `c1` - First color as floating-point RGB
/// * `c2` - Second color as integer RGB tuple
///
/// # Returns
///
/// The squared distance between the colors.
///
/// # Examples
///
/// ```
/// use palettize::color_distance_sq;
///
/// // Distance from black to white
/// let dist = color_distance_sq((0.0, 0.0, 0.0), (255, 255, 255));
/// assert!((dist - 195075.0).abs() < 0.001); // 255^2 * 3
///
/// // Distance from a color to itself
/// let dist = color_distance_sq((128.0, 128.0, 128.0), (128, 128, 128));
/// assert!(dist < 0.001);
/// ```
pub fn color_distance_sq(c1: (f32, f32, f32), c2: Rgb) -> f32 {
    let dr = c1.0 - c2.0 as f32;
    let dg = c1.1 - c2.1 as f32;
    let db = c1.2 - c2.2 as f32;
    dr * dr + dg * dg + db * db
}

/// Finds the two nearest palette colors to a source color.
///
/// Returns both colors along with their squared distances, which can be used
/// to weight the dithering decision.
///
/// # Arguments
///
/// * `color` - The source color as floating-point RGB
/// * `palette` - The available palette colors
///
/// # Returns
///
/// A tuple of `(nearest, second_nearest, nearest_distance, second_distance)`.
///
/// # Panics
///
/// Panics if the palette is empty.
///
/// # Examples
///
/// ```
/// use palettize::find_two_nearest;
///
/// let palette = vec![(0, 0, 0), (255, 255, 255)];
/// let (nearest, second, d1, d2) = find_two_nearest((64.0, 64.0, 64.0), &palette);
///
/// // Dark gray is closer to black
/// assert_eq!(nearest, (0, 0, 0));
/// assert_eq!(second, (255, 255, 255));
/// assert!(d1 < d2);
/// ```
pub fn find_two_nearest(color: (f32, f32, f32), palette: &[Rgb]) -> (Rgb, Rgb, f32, f32) {
    let mut best = palette[0];
    let mut second = palette[0];
    let mut best_dist = f32::MAX;
    let mut second_dist = f32::MAX;

    for &pal_color in palette {
        let dist = color_distance_sq(color, pal_color);
        if dist < best_dist {
            second = best;
            second_dist = best_dist;
            best = pal_color;
            best_dist = dist;
        } else if dist < second_dist {
            second = pal_color;
            second_dist = dist;
        }
    }

    (best, second, best_dist, second_dist)
}

/// Applies ordered Bayer dithering to an image.
///
/// This is the main entry point for dithering an image. It processes each pixel,
/// comparing it against the Bayer matrix threshold to decide which of the two
/// nearest palette colors to use.
///
/// # Arguments
///
/// * `img` - The input image
/// * `palette` - The color palette to quantize to
/// * `bayer_matrix` - The Bayer threshold matrix
/// * `noise_intensity` - Dither strength (0.0-2.0). Higher values increase contrast
///   in the dithering pattern. A value of 1.0 is neutral.
///
/// # Returns
///
/// A new RGB image with the dithering applied.
///
/// # Examples
///
/// ```no_run
/// use palettize::{apply_dithering, generate_bayer_matrix, get_preset_palette, Preset};
///
/// let img = image::open("input.png").unwrap();
/// let palette = get_preset_palette(Preset::GameBoy).unwrap();
/// let bayer = generate_bayer_matrix(2);
///
/// let output = apply_dithering(&img, &palette, &bayer, 1.0);
/// output.save("output.png").unwrap();
/// ```
pub fn apply_dithering(
    img: &DynamicImage,
    palette: &[Rgb],
    bayer_matrix: &[Vec<f32>],
    noise_intensity: f32,
) -> RgbImage {
    let (width, height) = img.dimensions();
    let matrix_size = bayer_matrix.len();

    let mut output = ImageBuffer::new(width, height);
    let rgb_img = img.to_rgb8();

    for y in 0..height {
        for x in 0..width {
            let pixel = rgb_img.get_pixel(x, y);

            // Get threshold from Bayer matrix
            let raw_threshold =
                bayer_matrix[(y % matrix_size as u32) as usize][(x % matrix_size as u32) as usize];

            // Scale threshold contrast with the noise parameter (acts like strength)
            let threshold =
                (((raw_threshold - 0.5) * (1.0 + noise_intensity)).clamp(-0.5, 0.5)) + 0.5;

            // Find two closest palette colors
            let src = (pixel[0] as f32, pixel[1] as f32, pixel[2] as f32);
            let (best, second, best_dist, second_dist) = find_two_nearest(src, palette);

            // If only one palette color exists, short-circuit
            let chosen = if palette.len() == 1 || second_dist == f32::MAX {
                best
            } else {
                // Weight choosing nearest vs second-nearest; closer colors win more often
                let weight_nearest = second_dist / (best_dist + second_dist);
                if threshold <= weight_nearest {
                    best
                } else {
                    second
                }
            };

            output.put_pixel(x, y, image::Rgb([chosen.0, chosen.1, chosen.2]));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_distance_sq_same_color() {
        let dist = color_distance_sq((100.0, 100.0, 100.0), (100, 100, 100));
        assert!(dist.abs() < f32::EPSILON);
    }

    #[test]
    fn test_color_distance_sq_black_to_white() {
        let dist = color_distance_sq((0.0, 0.0, 0.0), (255, 255, 255));
        let expected = 255.0 * 255.0 * 3.0;
        assert!((dist - expected).abs() < 0.001);
    }

    #[test]
    fn test_find_two_nearest_simple() {
        let palette = vec![(0, 0, 0), (255, 255, 255)];
        let (nearest, second, _, _) = find_two_nearest((50.0, 50.0, 50.0), &palette);
        assert_eq!(nearest, (0, 0, 0)); // Dark gray closer to black
        assert_eq!(second, (255, 255, 255));
    }

    #[test]
    fn test_find_two_nearest_bright() {
        let palette = vec![(0, 0, 0), (255, 255, 255)];
        let (nearest, second, _, _) = find_two_nearest((200.0, 200.0, 200.0), &palette);
        assert_eq!(nearest, (255, 255, 255)); // Light gray closer to white
        assert_eq!(second, (0, 0, 0));
    }

    #[test]
    fn test_find_two_nearest_exact_match() {
        let palette = vec![(0, 0, 0), (128, 128, 128), (255, 255, 255)];
        let (nearest, _, dist, _) = find_two_nearest((128.0, 128.0, 128.0), &palette);
        assert_eq!(nearest, (128, 128, 128));
        assert!(dist < 0.001);
    }

    #[test]
    fn test_apply_dithering_output_dimensions() {
        use image::DynamicImage;

        // Create a small test image
        let img = DynamicImage::new_rgb8(10, 10);
        let palette = vec![(0, 0, 0), (255, 255, 255)];
        let bayer = crate::bayer::generate_bayer_matrix(0);

        let output = apply_dithering(&img, &palette, &bayer, 1.0);

        assert_eq!(output.width(), 10);
        assert_eq!(output.height(), 10);
    }

    #[test]
    fn test_apply_dithering_output_uses_palette_colors() {
        use image::{DynamicImage, GenericImage, Rgba};

        // Create a gray image
        let mut img = DynamicImage::new_rgb8(4, 4);
        for y in 0..4 {
            for x in 0..4 {
                img.put_pixel(x, y, Rgba([128, 128, 128, 255]));
            }
        }

        let palette = vec![(0, 0, 0), (255, 255, 255)];
        let bayer = crate::bayer::generate_bayer_matrix(0);

        let output = apply_dithering(&img, &palette, &bayer, 1.0);

        // All output pixels should be either black or white
        for y in 0..4 {
            for x in 0..4 {
                let pixel = output.get_pixel(x, y);
                let is_black = pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0;
                let is_white = pixel[0] == 255 && pixel[1] == 255 && pixel[2] == 255;
                assert!(
                    is_black || is_white,
                    "Pixel at ({}, {}) is {:?}, expected black or white",
                    x,
                    y,
                    pixel
                );
            }
        }
    }
}
