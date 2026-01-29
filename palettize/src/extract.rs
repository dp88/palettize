//! Palette extraction algorithms.
//!
//! This module provides automatic color palette extraction from images
//! using either median cut or k-means++ clustering algorithms.

use crate::{Color, Palette};
use image::DynamicImage;

/// Maximum number of pixels to sample from large images.
const MAX_SAMPLES: usize = 10_000;

/// Maximum iterations for k-means convergence.
const MAX_ITERATIONS: usize = 20;

/// Threshold for early termination (squared distance).
const CONVERGENCE_THRESHOLD: f64 = 1.0;

/// Extracts a palette of N distinct colors from an image using k-means clustering.
///
/// Uses k-means++ initialization for better convergence and samples up to 10,000
/// pixels from large images for performance.
///
/// # Arguments
///
/// * `image` - The input image to extract colors from
/// * `n` - Number of colors to extract (1-255)
///
/// # Returns
///
/// A [`Palette`] containing the N extracted colors.
///
/// # Examples
///
/// ```no_run
/// use palettize::extract_palette_kmeans;
///
/// let img = image::open("photo.png").unwrap();
/// let palette = extract_palette_kmeans(&img, 8);
/// assert_eq!(palette.colors().len(), 8);
/// ```
pub fn extract_palette_kmeans(image: &DynamicImage, n: u8) -> Palette {
    let rgb = image.to_rgb8();
    let pixels: Vec<[f64; 3]> = sample_pixels(&rgb, MAX_SAMPLES);

    if pixels.is_empty() {
        return Palette::new(vec![Color::new(0, 0, 0)]);
    }

    let n = n as usize;
    if n == 0 {
        return Palette::new(vec![Color::new(0, 0, 0)]);
    }

    // Initialize centroids using k-means++
    let mut centroids = kmeans_plus_plus_init(&pixels, n);

    // Run k-means iterations
    for _ in 0..MAX_ITERATIONS {
        let assignments = assign_to_centroids(&pixels, &centroids);
        let new_centroids = update_centroids(&pixels, &assignments, n);

        // Check for convergence
        let max_shift = centroids
            .iter()
            .zip(new_centroids.iter())
            .map(|(old, new)| distance_sq(old, new))
            .fold(0.0, f64::max);

        centroids = new_centroids;

        if max_shift < CONVERGENCE_THRESHOLD {
            break;
        }
    }

    // Convert centroids to colors
    let colors: Vec<Color> = centroids
        .into_iter()
        .map(|c| Color::new(c[0].round() as u8, c[1].round() as u8, c[2].round() as u8))
        .collect();

    Palette::new(colors)
}

/// Extracts a palette of N distinct colors from an image using median cut.
///
/// The median cut algorithm recursively divides the color space by splitting
/// boxes of pixels along their longest axis. This typically produces better
/// perceptual results than k-means for many images.
///
/// # Arguments
///
/// * `image` - The input image to extract colors from
/// * `n` - Number of colors to extract (1-255)
///
/// # Returns
///
/// A [`Palette`] containing the N extracted colors.
///
/// # Examples
///
/// ```no_run
/// use palettize::extract_palette_median_cut;
///
/// let img = image::open("photo.png").unwrap();
/// let palette = extract_palette_median_cut(&img, 8);
/// assert_eq!(palette.colors().len(), 8);
/// ```
pub fn extract_palette_median_cut(image: &DynamicImage, n: u8) -> Palette {
    let rgb = image.to_rgb8();
    let pixels: Vec<[u8; 3]> = sample_pixels_u8(&rgb, MAX_SAMPLES);

    if pixels.is_empty() || n == 0 {
        return Palette::new(vec![Color::new(0, 0, 0)]);
    }

    let n = n as usize;

    // Start with one box containing all pixels
    let mut boxes = vec![ColorBox::new(pixels)];

    // Split until we have n boxes (or can't split anymore)
    while boxes.len() < n {
        // Find the box with the largest range to split
        let split_idx = boxes
            .iter()
            .enumerate()
            .filter(|(_, b)| b.pixels.len() > 1)
            .max_by_key(|(_, b)| b.largest_range())
            .map(|(i, _)| i);

        match split_idx {
            Some(idx) => {
                let to_split = boxes.remove(idx);
                let (a, b) = to_split.split();
                boxes.push(a);
                boxes.push(b);
            }
            None => break, // No more boxes can be split
        }
    }

    // Convert boxes to colors by averaging
    let colors: Vec<Color> = boxes.into_iter().map(|b| b.average_color()).collect();

    Palette::new(colors)
}

/// A box of pixels in RGB color space for median cut algorithm.
struct ColorBox {
    pixels: Vec<[u8; 3]>,
}

impl ColorBox {
    fn new(pixels: Vec<[u8; 3]>) -> Self {
        Self { pixels }
    }

    /// Returns the range (max - min) for a given channel.
    fn channel_range(&self, channel: usize) -> u8 {
        if self.pixels.is_empty() {
            return 0;
        }
        let min = self.pixels.iter().map(|p| p[channel]).min().unwrap_or(0);
        let max = self.pixels.iter().map(|p| p[channel]).max().unwrap_or(0);
        max - min
    }

    /// Returns the index of the channel with the largest range.
    fn largest_channel(&self) -> usize {
        let r_range = self.channel_range(0);
        let g_range = self.channel_range(1);
        let b_range = self.channel_range(2);

        if r_range >= g_range && r_range >= b_range {
            0
        } else if g_range >= b_range {
            1
        } else {
            2
        }
    }

    /// Returns the largest range across all channels.
    fn largest_range(&self) -> u8 {
        self.channel_range(self.largest_channel())
    }

    /// Splits the box along the channel with the largest range at the median.
    fn split(mut self) -> (ColorBox, ColorBox) {
        let channel = self.largest_channel();

        // Sort pixels by the chosen channel
        self.pixels.sort_by_key(|p| p[channel]);

        // Split at the median
        let mid = self.pixels.len() / 2;
        let right = self.pixels.split_off(mid);

        (ColorBox::new(self.pixels), ColorBox::new(right))
    }

    /// Returns the average color of all pixels in the box.
    fn average_color(&self) -> Color {
        if self.pixels.is_empty() {
            return Color::new(0, 0, 0);
        }

        let (r_sum, g_sum, b_sum) = self.pixels.iter().fold((0u64, 0u64, 0u64), |acc, p| {
            (acc.0 + p[0] as u64, acc.1 + p[1] as u64, acc.2 + p[2] as u64)
        });

        let count = self.pixels.len() as u64;
        Color::new(
            (r_sum / count) as u8,
            (g_sum / count) as u8,
            (b_sum / count) as u8,
        )
    }
}

/// Samples up to `max_samples` pixels from an image as u8 values.
fn sample_pixels_u8(rgb: &image::RgbImage, max_samples: usize) -> Vec<[u8; 3]> {
    let total_pixels = rgb.width() as usize * rgb.height() as usize;

    if total_pixels <= max_samples {
        rgb.pixels().map(|p| [p[0], p[1], p[2]]).collect()
    } else {
        let step = total_pixels / max_samples;
        rgb.pixels()
            .enumerate()
            .filter(|(i, _)| i % step == 0)
            .take(max_samples)
            .map(|(_, p)| [p[0], p[1], p[2]])
            .collect()
    }
}

/// Samples up to `max_samples` pixels from an image.
fn sample_pixels(rgb: &image::RgbImage, max_samples: usize) -> Vec<[f64; 3]> {
    let total_pixels = rgb.width() as usize * rgb.height() as usize;

    if total_pixels <= max_samples {
        // Use all pixels
        rgb.pixels()
            .map(|p| [p[0] as f64, p[1] as f64, p[2] as f64])
            .collect()
    } else {
        // Sample evenly across the image
        let step = total_pixels / max_samples;
        rgb.pixels()
            .enumerate()
            .filter(|(i, _)| i % step == 0)
            .take(max_samples)
            .map(|(_, p)| [p[0] as f64, p[1] as f64, p[2] as f64])
            .collect()
    }
}

/// Initializes centroids using k-means++ algorithm.
fn kmeans_plus_plus_init(pixels: &[[f64; 3]], k: usize) -> Vec<[f64; 3]> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Deterministic seed based on pixel data
    let mut hasher = DefaultHasher::new();
    pixels.len().hash(&mut hasher);
    if let Some(first) = pixels.first() {
        (first[0] as u64).hash(&mut hasher);
        (first[1] as u64).hash(&mut hasher);
        (first[2] as u64).hash(&mut hasher);
    }
    let mut seed = hasher.finish();

    // Simple LCG for deterministic random numbers
    let mut next_random = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (seed >> 33) as f64 / (1u64 << 31) as f64
    };

    let mut centroids: Vec<[f64; 3]> = Vec::with_capacity(k);

    // Choose first centroid randomly
    let first_idx = (next_random() * pixels.len() as f64) as usize % pixels.len();
    centroids.push(pixels[first_idx]);

    // Choose remaining centroids with probability proportional to D(x)^2
    for _ in 1..k {
        let distances: Vec<f64> = pixels
            .iter()
            .map(|p| {
                centroids
                    .iter()
                    .map(|c| distance_sq(p, c))
                    .fold(f64::MAX, f64::min)
            })
            .collect();

        let total: f64 = distances.iter().sum();
        if total == 0.0 {
            // All remaining pixels are at existing centroids, just pick any
            centroids.push(pixels[0]);
            continue;
        }

        let threshold = next_random() * total;
        let mut cumulative = 0.0;
        let mut chosen_idx = 0;

        for (i, &d) in distances.iter().enumerate() {
            cumulative += d;
            if cumulative >= threshold {
                chosen_idx = i;
                break;
            }
        }

        centroids.push(pixels[chosen_idx]);
    }

    centroids
}

/// Assigns each pixel to its nearest centroid.
fn assign_to_centroids(pixels: &[[f64; 3]], centroids: &[[f64; 3]]) -> Vec<usize> {
    pixels
        .iter()
        .map(|p| {
            centroids
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    distance_sq(p, a)
                        .partial_cmp(&distance_sq(p, b))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
                .unwrap_or(0)
        })
        .collect()
}

/// Updates centroids based on current assignments.
fn update_centroids(pixels: &[[f64; 3]], assignments: &[usize], k: usize) -> Vec<[f64; 3]> {
    let mut sums = vec![[0.0, 0.0, 0.0]; k];
    let mut counts = vec![0usize; k];

    for (pixel, &cluster) in pixels.iter().zip(assignments.iter()) {
        sums[cluster][0] += pixel[0];
        sums[cluster][1] += pixel[1];
        sums[cluster][2] += pixel[2];
        counts[cluster] += 1;
    }

    sums.into_iter()
        .zip(counts.into_iter())
        .map(|(sum, count)| {
            if count > 0 {
                [
                    sum[0] / count as f64,
                    sum[1] / count as f64,
                    sum[2] / count as f64,
                ]
            } else {
                // Empty cluster, keep at origin (will be overwritten or ignored)
                [0.0, 0.0, 0.0]
            }
        })
        .collect()
}

/// Squared Euclidean distance between two RGB points.
fn distance_sq(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dr = a[0] - b[0];
    let dg = a[1] - b[1];
    let db = a[2] - b[2];
    dr * dr + dg * dg + db * db
}

#[cfg(test)]
mod tests {
    use super::*;

    // K-means tests

    #[test]
    fn test_kmeans_extract_single_color() {
        let mut img = image::RgbImage::new(10, 10);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgb([255, 0, 0]);
        }
        let dynamic = DynamicImage::ImageRgb8(img);

        let palette = extract_palette_kmeans(&dynamic, 1);
        assert_eq!(palette.colors().len(), 1);
        assert_eq!(palette.colors()[0], Color::new(255, 0, 0));
    }

    #[test]
    fn test_kmeans_extract_two_colors() {
        let mut img = image::RgbImage::new(10, 10);
        for (i, pixel) in img.pixels_mut().enumerate() {
            if i < 50 {
                *pixel = image::Rgb([0, 0, 0]);
            } else {
                *pixel = image::Rgb([255, 255, 255]);
            }
        }
        let dynamic = DynamicImage::ImageRgb8(img);

        let palette = extract_palette_kmeans(&dynamic, 2);
        assert_eq!(palette.colors().len(), 2);

        let colors: Vec<_> = palette.colors().iter().collect();
        let has_black = colors.iter().any(|c| c.r < 10 && c.g < 10 && c.b < 10);
        let has_white = colors
            .iter()
            .any(|c| c.r > 245 && c.g > 245 && c.b > 245);
        assert!(has_black, "Should find black");
        assert!(has_white, "Should find white");
    }

    #[test]
    fn test_kmeans_respects_count() {
        let mut img = image::RgbImage::new(10, 10);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgb([128, 128, 128]);
        }
        let dynamic = DynamicImage::ImageRgb8(img);

        for n in [1, 3, 5, 8] {
            let palette = extract_palette_kmeans(&dynamic, n);
            assert_eq!(palette.colors().len(), n as usize);
        }
    }

    // Median cut tests

    #[test]
    fn test_median_cut_extract_single_color() {
        let mut img = image::RgbImage::new(10, 10);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgb([255, 0, 0]);
        }
        let dynamic = DynamicImage::ImageRgb8(img);

        let palette = extract_palette_median_cut(&dynamic, 1);
        assert_eq!(palette.colors().len(), 1);
        assert_eq!(palette.colors()[0], Color::new(255, 0, 0));
    }

    #[test]
    fn test_median_cut_extract_two_colors() {
        let mut img = image::RgbImage::new(10, 10);
        for (i, pixel) in img.pixels_mut().enumerate() {
            if i < 50 {
                *pixel = image::Rgb([0, 0, 0]);
            } else {
                *pixel = image::Rgb([255, 255, 255]);
            }
        }
        let dynamic = DynamicImage::ImageRgb8(img);

        let palette = extract_palette_median_cut(&dynamic, 2);
        assert_eq!(palette.colors().len(), 2);

        let colors: Vec<_> = palette.colors().iter().collect();
        let has_black = colors.iter().any(|c| c.r < 10 && c.g < 10 && c.b < 10);
        let has_white = colors
            .iter()
            .any(|c| c.r > 245 && c.g > 245 && c.b > 245);
        assert!(has_black, "Should find black");
        assert!(has_white, "Should find white");
    }

    #[test]
    fn test_median_cut_respects_count() {
        let mut img = image::RgbImage::new(10, 10);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgb([128, 128, 128]);
        }
        let dynamic = DynamicImage::ImageRgb8(img);

        for n in [1, 3, 5, 8] {
            let palette = extract_palette_median_cut(&dynamic, n);
            assert_eq!(palette.colors().len(), n as usize);
        }
    }

    #[test]
    fn test_color_box_channel_range() {
        let pixels = vec![[0, 50, 100], [10, 60, 200], [20, 70, 150]];
        let color_box = ColorBox::new(pixels);

        assert_eq!(color_box.channel_range(0), 20); // R: 20 - 0
        assert_eq!(color_box.channel_range(1), 20); // G: 70 - 50
        assert_eq!(color_box.channel_range(2), 100); // B: 200 - 100
    }

    #[test]
    fn test_color_box_largest_channel() {
        let pixels = vec![[0, 50, 100], [10, 60, 200], [20, 70, 150]];
        let color_box = ColorBox::new(pixels);

        assert_eq!(color_box.largest_channel(), 2); // Blue has largest range
    }

    #[test]
    fn test_color_box_split() {
        let pixels = vec![[0, 0, 0], [100, 0, 0], [200, 0, 0], [255, 0, 0]];
        let color_box = ColorBox::new(pixels);

        let (a, b) = color_box.split();
        assert_eq!(a.pixels.len(), 2);
        assert_eq!(b.pixels.len(), 2);
    }

    #[test]
    fn test_color_box_average_color() {
        let pixels = vec![[0, 0, 0], [100, 100, 100]];
        let color_box = ColorBox::new(pixels);

        let avg = color_box.average_color();
        assert_eq!(avg, Color::new(50, 50, 50));
    }

    #[test]
    fn test_distance_sq() {
        assert_eq!(distance_sq(&[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0]), 0.0);
        assert_eq!(distance_sq(&[0.0, 0.0, 0.0], &[1.0, 0.0, 0.0]), 1.0);
        assert_eq!(distance_sq(&[0.0, 0.0, 0.0], &[1.0, 1.0, 1.0]), 3.0);
    }
}
