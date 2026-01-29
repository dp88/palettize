//! # Palettize
//!
//! A library for applying ordered Bayer dithering to images with custom color palettes.
//!
//! Ordered dithering is a technique that reduces the number of colors in an image while
//! maintaining visual quality through the use of spatial patterns. This library provides
//! tools to:
//!
//! - Generate Bayer matrices of various sizes
//! - Parse color palettes from hex strings
//! - Apply dithering with customizable parameters
//!
//! ## Quick Start
//!
//! The simplest way to dither an image is with the [`dither()`] function:
//!
//! ```no_run
//! use palettize::{dither, grayscale};
//!
//! // Load an image
//! let img = image::open("input.png").unwrap();
//!
//! // Generate a grayscale palette (black & white)
//! let palette = grayscale(2);
//!
//! // Apply dithering with default settings
//! let output = dither(&img, &palette);
//!
//! // Save the result
//! output.save("output.png").unwrap();
//! ```
//!
//! For more control over dithering parameters, use [`dither_with_options()`]:
//!
//! ```no_run
//! use palettize::{dither_with_options, grayscale};
//!
//! let img = image::open("input.png").unwrap();
//! let palette = grayscale(6); // 6-level grayscale
//!
//! // Bayer level 3 = 16×16 matrix, noise 0.5 = subtle dithering
//! let output = dither_with_options(&img, &palette, 3, 0.5);
//! output.save("output.png").unwrap();
//! ```
//!
//! ## Custom Palettes
//!
//! You can create palettes from custom colors in several ways:
//!
//! ```
//! use palettize::{Color, Palette, parse_hex_color};
//!
//! // From Color structs
//! let palette = Palette::new(vec![
//!     Color::new(0, 0, 0),
//!     Color::new(255, 255, 255),
//! ]);
//!
//! // From tuples using Into
//! let palette: Palette = vec![
//!     (0, 0, 0),
//!     (255, 255, 255),
//! ].into();
//!
//! // From hex color strings
//! let palette = Palette::new(vec![
//!     parse_hex_color("#1a1c2c").unwrap(),
//!     parse_hex_color("#5d275d").unwrap(),
//!     parse_hex_color("#b13e53").unwrap(),
//! ]);
//! ```
//!
//! ## Grayscale Palettes
//!
//! Use [`grayscale()`] to generate evenly-spaced grayscale palettes:
//!
//! - `grayscale(2)` - Black and white
//! - `grayscale(6)` - 6-level grayscale
//! - `grayscale(256)` - Full 8-bit grayscale
//!
//! ## Bayer Matrix Levels
//!
//! The Bayer matrix level controls the size of the dithering pattern:
//!
//! | Level | Matrix Size | Threshold Count |
//! |-------|-------------|-----------------|
//! | 0     | 2×2         | 4               |
//! | 1     | 4×4         | 16              |
//! | 2     | 8×8         | 64 (default)    |
//! | 3     | 16×16       | 256             |
//! | 4     | 32×32       | 1024            |
//! | 5     | 64×64       | 4096            |
//!
//! Higher levels produce smoother gradients but larger repeating patterns.
//! Level 2 (8×8) is a good default for most images.

pub mod bayer;
pub mod dither;
pub mod extract;
pub mod palette;

use image::{DynamicImage, RgbImage};

// Re-export main types and functions for convenience
pub use bayer::generate_bayer_matrix;
pub use dither::{apply_dithering, color_distance_sq, find_two_nearest};
pub use extract::{extract_palette_kmeans, extract_palette_median_cut};
pub use palette::{Color, Palette, ParseColorError, grayscale, parse_hex_color};

/// Default Bayer matrix level used by [`dither()`].
///
/// Level 2 produces an 8×8 matrix, which provides a good balance between
/// smooth gradients and visible dithering patterns for most images.
pub const DEFAULT_BAYER_LEVEL: u32 = 2;

/// Default noise intensity used by [`dither()`].
///
/// A value of 1.0 provides neutral dithering strength. Lower values (0.0-1.0)
/// produce subtler dithering, while higher values (1.0-2.0) increase contrast.
pub const DEFAULT_NOISE: f32 = 1.0;

/// Dithers an image using the given palette with default settings.
///
/// This is the simplest way to apply ordered Bayer dithering to an image.
/// It uses default parameters: Bayer level 2 (8×8 matrix) and noise intensity 1.0.
///
/// For more control over dithering parameters, use [`dither_with_options()`].
///
/// # Arguments
///
/// * `image` - The input image to dither
/// * `palette` - The color palette to quantize to
///
/// # Returns
///
/// A new [`RgbImage`] with the dithering applied.
///
/// # Examples
///
/// ```no_run
/// use palettize::{dither, grayscale};
///
/// let img = image::open("input.png").unwrap();
/// let palette = grayscale(2); // black & white
/// let output = dither(&img, &palette);
/// output.save("output.png").unwrap();
/// ```
///
/// Using a custom palette:
///
/// ```no_run
/// use palettize::{dither, Color, Palette};
///
/// let img = image::open("input.png").unwrap();
/// let palette: Palette = vec![
///     (0, 0, 0),       // Black
///     (255, 255, 255), // White
/// ].into();
/// let output = dither(&img, &palette);
/// ```
pub fn dither(image: &DynamicImage, palette: &Palette) -> RgbImage {
    dither_with_options(image, palette, DEFAULT_BAYER_LEVEL, DEFAULT_NOISE)
}

/// Dithers an image using the given palette with custom settings.
///
/// This function provides full control over the dithering parameters.
/// For default settings, use [`dither()`] instead.
///
/// # Arguments
///
/// * `image` - The input image to dither
/// * `palette` - The color palette to quantize to
/// * `bayer_level` - The Bayer matrix level (0-5). Controls the size of the
///   dithering pattern. Level 0 = 2×2, level 5 = 64×64. Use 2 as a good default.
/// * `noise` - Dither strength (0.0-2.0). Controls the contrast of the dithering
///   pattern. 0.0 = minimal dithering, 1.0 = neutral, 2.0 = maximum contrast.
///
/// # Returns
///
/// A new [`RgbImage`] with the dithering applied.
///
/// # Bayer Matrix Levels
///
/// | Level | Matrix Size | Effect |
/// |-------|-------------|--------|
/// | 0     | 2×2         | Very coarse dithering |
/// | 1     | 4×4         | Coarse dithering |
/// | 2     | 8×8         | Good default |
/// | 3     | 16×16       | Smooth gradients |
/// | 4     | 32×32       | Very smooth |
/// | 5     | 64×64       | Extremely smooth |
///
/// # Examples
///
/// ```no_run
/// use palettize::{dither_with_options, grayscale};
///
/// let img = image::open("input.png").unwrap();
/// let palette = grayscale(6); // 6-level grayscale
///
/// // Smooth gradients with subtle dithering
/// let output = dither_with_options(&img, &palette, 3, 0.5);
/// output.save("smooth.png").unwrap();
///
/// // Coarse retro look with strong dithering
/// let output = dither_with_options(&img, &palette, 1, 1.5);
/// output.save("retro.png").unwrap();
/// ```
pub fn dither_with_options(
    image: &DynamicImage,
    palette: &Palette,
    bayer_level: u32,
    noise: f32,
) -> RgbImage {
    let bayer_matrix = generate_bayer_matrix(bayer_level);
    apply_dithering(image, palette.colors(), &bayer_matrix, noise)
}
