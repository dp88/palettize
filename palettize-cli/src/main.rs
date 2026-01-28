//! Command-line interface for the palettize image dithering tool.
//!
//! This binary provides a user-friendly interface to the palettize library,
//! allowing users to apply ordered Bayer dithering to images from the command line.
//!
//! # Usage
//!
//! ```text
//! palettize -i input.png -o output.png --preset gameboy
//! palettize -i input.png -o output.png -p '#000000,#FFFFFF'
//! palettize -i input.png -o output.png --palette-file colors.hex -b 3 -n 0.5
//! ```

use anyhow::{Context, Result};
use clap::Parser;
use palettize::{Color, Palette, Preset, dither_with_options, parse_hex_color};
use std::path::{Path, PathBuf};

/// Apply ordered Bayer dithering to images with custom color palettes.
///
/// This tool converts full-color images to a limited palette while maintaining
/// visual quality through ordered dithering. The result mimics the look of
/// classic hardware with limited color capabilities.
#[derive(Parser, Debug)]
#[command(name = "palettize")]
#[command(version)]
#[command(about = "Apply ordered Bayer dithering to images with custom color palettes")]
#[command(long_about = "
Palettize converts full-color images to a limited color palette using ordered
Bayer dithering. This technique creates the characteristic cross-hatch patterns
seen in classic video games and pixel art.

EXAMPLES:
    # Use a preset palette
    palettize -i photo.png -o output.png --preset gameboy

    # Use custom colors
    palettize -i photo.png -o output.png -p '#000000,#555555,#AAAAAA,#FFFFFF'

    # Use a palette file
    palettize -i photo.png -o output.png --palette-file my-colors.hex

    # Adjust dithering parameters
    palettize -i photo.png -o output.png --preset bw -b 3 -n 0.5
")]
struct Args {
    /// Input image path (supports PNG, JPEG, GIF, BMP, etc.)
    #[arg(short, long)]
    input: PathBuf,

    /// Output image path
    #[arg(short, long)]
    output: PathBuf,

    /// Custom palette as comma-separated hex colors.
    ///
    /// Example: "#000000,#FFFFFF,#FF0000"
    #[arg(short, long, conflicts_with_all = ["preset", "palette_file"])]
    palette: Option<String>,

    /// Path to a palette file containing hex colors (one per line).
    ///
    /// Lines starting with '//' are treated as comments.
    #[arg(long, conflicts_with_all = ["preset", "palette"], value_name = "FILE")]
    palette_file: Option<PathBuf>,

    /// Use a preset palette.
    ///
    /// Available presets: bw, rgb3bit, grayscale, gameboy, cga
    #[arg(long, conflicts_with_all = ["palette", "palette_file"], value_name = "NAME")]
    preset: Option<Preset>,

    /// Bayer matrix level (0-5).
    ///
    /// Controls the dithering pattern size:
    ///   0 = 2×2,  1 = 4×4,  2 = 8×8 (default),
    ///   3 = 16×16, 4 = 32×32, 5 = 64×64
    #[arg(short, long, default_value = "2", value_parser = clap::value_parser!(u32).range(0..=5))]
    bayer_level: u32,

    /// Dither strength (0.0-2.0).
    ///
    /// Controls the contrast of the dithering pattern.
    /// Lower values produce subtler dithering, higher values increase contrast.
    /// Default is 1.0.
    #[arg(short, long, default_value = "1.0", value_name = "STRENGTH")]
    noise: f32,
}

/// Loads a palette from a file.
///
/// The file should contain one hex color per line. Lines starting with `//`
/// are treated as comments and ignored. Empty lines are also ignored.
fn load_palette_file(path: &Path) -> Result<Palette> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read palette file: {}", path.display()))?;

    let mut colors: Vec<Color> = Vec::new();
    for (line_num, line) in data.lines().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        // Skip lines that look like comments (# followed by text, not a hex color)
        if trimmed.starts_with('#') && trimmed.len() > 7 {
            continue;
        }

        colors.push(parse_hex_color(trimmed).with_context(|| {
            format!(
                "Invalid color on line {} of {}",
                line_num + 1,
                path.display()
            )
        })?);
    }

    if colors.is_empty() {
        anyhow::bail!("Palette file '{}' contained no colors", path.display());
    }

    Ok(Palette::new(colors))
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate arguments
    if args.palette.is_none() && args.palette_file.is_none() && args.preset.is_none() {
        anyhow::bail!(
            "A palette must be specified. Use one of:\n  \
             --preset <NAME>      (available: {})\n  \
             --palette <COLORS>   (comma-separated hex colors)\n  \
             --palette-file <FILE>",
            Preset::all_names().join(", ")
        );
    }

    if args.noise < 0.0 || args.noise > 2.0 {
        anyhow::bail!("Dither strength (--noise) must be between 0.0 and 2.0");
    }

    // Parse palette
    let palette: Palette = if let Some(palette_str) = args.palette {
        let colors = palette_str
            .split(',')
            .map(|s| parse_hex_color(s.trim()))
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse palette colors")?;
        Palette::new(colors)
    } else if let Some(palette_path) = args.palette_file {
        load_palette_file(&palette_path)?
    } else if let Some(preset) = args.preset {
        Palette::from_preset(preset)
    } else {
        unreachable!()
    };

    if palette.colors().is_empty() {
        anyhow::bail!("Palette cannot be empty");
    }

    // Load input image
    let img = image::open(&args.input)
        .with_context(|| format!("Failed to open input image: {}", args.input.display()))?;

    // Calculate matrix size for display
    let matrix_size = 2_usize.pow(args.bayer_level + 1);

    eprintln!("Applying dithering:");
    eprintln!("  Palette: {} colors", palette.colors().len());
    eprintln!(
        "  Bayer level: {} ({}×{} matrix)",
        args.bayer_level, matrix_size, matrix_size
    );
    eprintln!("  Noise intensity: {}", args.noise);

    // Apply dithering
    let output = dither_with_options(&img, &palette, args.bayer_level, args.noise);

    // Save output image
    output
        .save(&args.output)
        .with_context(|| format!("Failed to save output image: {}", args.output.display()))?;

    eprintln!("Output saved to: {}", args.output.display());

    Ok(())
}
