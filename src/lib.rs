//! # Palettize
//!
//! A library for applying ordered Bayer dithering to images with custom color palettes.
//!
//! Ordered dithering is a technique that reduces the number of colors in an image while
//! maintaining visual quality through the use of spatial patterns. This library provides
//! tools to:
//!
//! - Generate Bayer matrices of various sizes
//! - Parse color palettes from hex strings or files
//! - Apply dithering with customizable parameters
//!
//! ## Quick Start
//!
//! ```no_run
//! use palettize::{apply_dithering, generate_bayer_matrix, get_preset_palette, Preset};
//!
//! // Load an image
//! let img = image::open("input.png").unwrap();
//!
//! // Get a preset palette (or define your own)
//! let palette = get_preset_palette(Preset::GameBoy).unwrap();
//!
//! // Generate an 8×8 Bayer matrix
//! let bayer = generate_bayer_matrix(2);
//!
//! // Apply dithering
//! let output = apply_dithering(&img, &palette, &bayer, 1.0);
//!
//! // Save the result
//! output.save("output.png").unwrap();
//! ```
//!
//! ## Custom Palettes
//!
//! You can define custom palettes using hex color strings:
//!
//! ```
//! use palettize::parse_hex_color;
//!
//! let palette = vec![
//!     parse_hex_color("#1a1c2c").unwrap(), // Dark blue
//!     parse_hex_color("#5d275d").unwrap(), // Purple
//!     parse_hex_color("#b13e53").unwrap(), // Red
//!     parse_hex_color("#ef7d57").unwrap(), // Orange
//!     parse_hex_color("#ffcd75").unwrap(), // Yellow
//!     parse_hex_color("#a7f070").unwrap(), // Green
//!     parse_hex_color("#38b764").unwrap(), // Teal
//!     parse_hex_color("#257179").unwrap(), // Dark teal
//! ];
//! ```
//!
//! ## Bayer Matrix Levels
//!
//! The Bayer matrix level controls the size of the dithering pattern:
//!
//! | Level | Matrix Size | Threshold Count |
//! |-------|-------------|-----------------|
//! | 0     | 2×2         | 4               |
//! | 1     | 4×4         | 16              |
//! | 2     | 8×8         | 64              |
//! | 3     | 16×16       | 256             |
//! | 4     | 32×32       | 1024            |
//! | 5     | 64×64       | 4096            |
//!
//! Higher levels produce smoother gradients but larger repeating patterns.
//! Level 2 (8×8) is a good default for most images.

pub mod bayer;
pub mod dither;
pub mod palette;

// Re-export main types and functions for convenience
pub use bayer::generate_bayer_matrix;
pub use dither::{apply_dithering, color_distance_sq, find_two_nearest};
pub use palette::{Preset, Rgb, get_preset_palette, load_palette_file, parse_hex_color};
