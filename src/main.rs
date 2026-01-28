//! Command-line interface for the palettize image dithering tool.

use anyhow::{Context, Result};
use clap::Parser;
use palettize::{
    Preset, Rgb, apply_dithering, generate_bayer_matrix, get_preset_palette, load_palette_file,
    parse_hex_color,
};
use std::path::PathBuf;

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
    let palette: Vec<Rgb> = if let Some(palette_str) = args.palette {
        palette_str
            .split(',')
            .map(|s| parse_hex_color(s.trim()))
            .collect::<Result<Vec<_>>>()
            .context("Failed to parse palette colors")?
    } else if let Some(palette_path) = args.palette_file {
        load_palette_file(&palette_path)?
    } else if let Some(preset) = args.preset {
        get_preset_palette(preset)?
    } else {
        unreachable!()
    };

    if palette.is_empty() {
        anyhow::bail!("Palette cannot be empty");
    }

    // Load input image
    let img = image::open(&args.input)
        .with_context(|| format!("Failed to open input image: {}", args.input.display()))?;

    // Generate Bayer matrix
    let bayer_matrix = generate_bayer_matrix(args.bayer_level);

    eprintln!("Applying dithering:");
    eprintln!("  Palette: {} colors", palette.len());
    eprintln!(
        "  Bayer level: {} ({}×{} matrix)",
        args.bayer_level,
        bayer_matrix.len(),
        bayer_matrix.len()
    );
    eprintln!("  Noise intensity: {}", args.noise);

    // Apply dithering
    let output = apply_dithering(&img, &palette, &bayer_matrix, args.noise);

    // Save output image
    output
        .save(&args.output)
        .with_context(|| format!("Failed to save output image: {}", args.output.display()))?;

    eprintln!("Output saved to: {}", args.output.display());

    Ok(())
}
