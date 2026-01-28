//! Palette parsing and preset color palettes.
//!
//! This module handles loading color palettes from various sources:
//! - Hex color strings (e.g., "#FF0000")
//! - Palette files (one hex color per line)
//! - Built-in preset palettes (e.g., "gameboy", "cga")
//!
//! # Examples
//!
//! ```
//! use palettize::{parse_hex_color, get_preset_palette, Preset};
//!
//! // Parse a hex color
//! let red = parse_hex_color("#FF0000").unwrap();
//! assert_eq!(red, (255, 0, 0));
//!
//! // Get a preset palette
//! let palette = get_preset_palette(Preset::GameBoy).unwrap();
//! assert_eq!(palette.len(), 4);
//! ```

use anyhow::{Context, Result};
use std::path::Path;
use std::str::FromStr;

/// An RGB color represented as a tuple of (red, green, blue) components.
///
/// Each component is a `u8` value in the range 0-255.
pub type Rgb = (u8, u8, u8);

/// Available preset palettes.
///
/// These palettes are inspired by classic hardware and common use cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// Black and white (2 colors)
    Bw,
    /// 3-bit RGB: black, red, green, blue, yellow, magenta, cyan, white (8 colors)
    Rgb3bit,
    /// 6-level grayscale from black to white
    Grayscale,
    /// Nintendo Game Boy green palette (4 colors)
    GameBoy,
    /// IBM CGA 16-color palette
    Cga,
}

impl Preset {
    /// Returns a list of all available preset names.
    pub fn all_names() -> &'static [&'static str] {
        &["bw", "rgb3bit", "grayscale", "gameboy", "cga"]
    }
}

impl FromStr for Preset {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "bw" => Ok(Preset::Bw),
            "rgb3bit" => Ok(Preset::Rgb3bit),
            "grayscale" => Ok(Preset::Grayscale),
            "gameboy" => Ok(Preset::GameBoy),
            "cga" => Ok(Preset::Cga),
            _ => anyhow::bail!(
                "Unknown preset: '{}'. Available presets: {}",
                s,
                Preset::all_names().join(", ")
            ),
        }
    }
}

impl std::fmt::Display for Preset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Preset::Bw => "bw",
            Preset::Rgb3bit => "rgb3bit",
            Preset::Grayscale => "grayscale",
            Preset::GameBoy => "gameboy",
            Preset::Cga => "cga",
        };
        write!(f, "{}", name)
    }
}

/// Parses a hex color string into an RGB tuple.
///
/// Accepts both formats: with or without the leading `#`.
///
/// # Arguments
///
/// * `hex` - A hex color string like "#FF0000" or "FF0000"
///
/// # Returns
///
/// An RGB tuple on success.
///
/// # Errors
///
/// Returns an error if the string is not a valid 6-character hex color.
///
/// # Examples
///
/// ```
/// use palettize::parse_hex_color;
///
/// assert_eq!(parse_hex_color("#FF0000").unwrap(), (255, 0, 0));
/// assert_eq!(parse_hex_color("00FF00").unwrap(), (0, 255, 0));
/// assert_eq!(parse_hex_color("  #0000ff  ").unwrap(), (0, 0, 255));
///
/// // Invalid colors return errors
/// assert!(parse_hex_color("#GGG").is_err());
/// assert!(parse_hex_color("#12345").is_err());
/// ```
pub fn parse_hex_color(hex: &str) -> Result<Rgb> {
    let trimmed = hex.trim().trim_start_matches('#');
    if trimmed.len() != 6 {
        anyhow::bail!("Invalid hex color: '{}' (expected 6 hex digits)", hex);
    }

    let r = u8::from_str_radix(&trimmed[0..2], 16)
        .with_context(|| format!("Invalid red component in '{}'", hex))?;
    let g = u8::from_str_radix(&trimmed[2..4], 16)
        .with_context(|| format!("Invalid green component in '{}'", hex))?;
    let b = u8::from_str_radix(&trimmed[4..6], 16)
        .with_context(|| format!("Invalid blue component in '{}'", hex))?;

    Ok((r, g, b))
}

/// Loads a palette from a file.
///
/// The file should contain one hex color per line. Lines starting with `//`
/// are treated as comments and ignored. Empty lines are also ignored.
///
/// # Arguments
///
/// * `path` - Path to the palette file
///
/// # Returns
///
/// A vector of RGB colors on success.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - Any line contains an invalid hex color
/// - The file contains no valid colors
///
/// # Examples
///
/// ```no_run
/// use palettize::load_palette_file;
/// use std::path::Path;
///
/// let palette = load_palette_file(Path::new("my-palette.hex")).unwrap();
/// println!("Loaded {} colors", palette.len());
/// ```
pub fn load_palette_file(path: &Path) -> Result<Vec<Rgb>> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read palette file: {}", path.display()))?;

    let mut colors = Vec::new();
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

    Ok(colors)
}

/// Returns the colors for a preset palette.
///
/// # Arguments
///
/// * `preset` - The preset to retrieve
///
/// # Returns
///
/// A vector of RGB colors for the preset.
///
/// # Errors
///
/// This function is infallible for valid `Preset` values.
///
/// # Examples
///
/// ```
/// use palettize::{get_preset_palette, Preset};
///
/// let bw = get_preset_palette(Preset::Bw).unwrap();
/// assert_eq!(bw, vec![(0, 0, 0), (255, 255, 255)]);
///
/// let gameboy = get_preset_palette(Preset::GameBoy).unwrap();
/// assert_eq!(gameboy.len(), 4);
/// ```
pub fn get_preset_palette(preset: Preset) -> Result<Vec<Rgb>> {
    let colors = match preset {
        Preset::Bw => vec![(0, 0, 0), (255, 255, 255)],
        Preset::Rgb3bit => vec![
            (0, 0, 0),
            (255, 0, 0),
            (0, 255, 0),
            (0, 0, 255),
            (255, 255, 0),
            (255, 0, 255),
            (0, 255, 255),
            (255, 255, 255),
        ],
        Preset::Grayscale => vec![
            (0, 0, 0),
            (51, 51, 51),
            (102, 102, 102),
            (153, 153, 153),
            (204, 204, 204),
            (255, 255, 255),
        ],
        Preset::GameBoy => vec![(15, 56, 15), (48, 98, 48), (139, 172, 15), (155, 188, 15)],
        Preset::Cga => vec![
            (0, 0, 0),
            (0, 0, 170),
            (0, 170, 0),
            (0, 170, 170),
            (170, 0, 0),
            (170, 0, 170),
            (170, 85, 0),
            (170, 170, 170),
            (85, 85, 85),
            (85, 85, 255),
            (85, 255, 85),
            (85, 255, 255),
            (255, 85, 85),
            (255, 85, 255),
            (255, 255, 85),
            (255, 255, 255),
        ],
    };

    Ok(colors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_with_hash() {
        assert_eq!(parse_hex_color("#FF0000").unwrap(), (255, 0, 0));
        assert_eq!(parse_hex_color("#00FF00").unwrap(), (0, 255, 0));
        assert_eq!(parse_hex_color("#0000FF").unwrap(), (0, 0, 255));
    }

    #[test]
    fn test_parse_hex_without_hash() {
        assert_eq!(parse_hex_color("FF0000").unwrap(), (255, 0, 0));
        assert_eq!(parse_hex_color("000000").unwrap(), (0, 0, 0));
        assert_eq!(parse_hex_color("FFFFFF").unwrap(), (255, 255, 255));
    }

    #[test]
    fn test_parse_hex_with_whitespace() {
        assert_eq!(parse_hex_color("  #FF0000  ").unwrap(), (255, 0, 0));
    }

    #[test]
    fn test_parse_hex_lowercase() {
        assert_eq!(parse_hex_color("#ff0000").unwrap(), (255, 0, 0));
        assert_eq!(parse_hex_color("#aabbcc").unwrap(), (170, 187, 204));
    }

    #[test]
    fn test_parse_hex_invalid_length() {
        assert!(parse_hex_color("#FFF").is_err());
        assert!(parse_hex_color("#FFFFFFF").is_err());
    }

    #[test]
    fn test_parse_hex_invalid_chars() {
        assert!(parse_hex_color("#GGGGGG").is_err());
    }

    #[test]
    fn test_preset_from_str() {
        assert_eq!("bw".parse::<Preset>().unwrap(), Preset::Bw);
        assert_eq!("BW".parse::<Preset>().unwrap(), Preset::Bw);
        assert_eq!("gameboy".parse::<Preset>().unwrap(), Preset::GameBoy);
        assert_eq!("GameBoy".parse::<Preset>().unwrap(), Preset::GameBoy);
    }

    #[test]
    fn test_preset_from_str_unknown() {
        assert!("unknown".parse::<Preset>().is_err());
    }

    #[test]
    fn test_preset_display() {
        assert_eq!(Preset::Bw.to_string(), "bw");
        assert_eq!(Preset::GameBoy.to_string(), "gameboy");
    }

    #[test]
    fn test_get_preset_palette_bw() {
        let palette = get_preset_palette(Preset::Bw).unwrap();
        assert_eq!(palette.len(), 2);
        assert_eq!(palette[0], (0, 0, 0));
        assert_eq!(palette[1], (255, 255, 255));
    }

    #[test]
    fn test_get_preset_palette_gameboy() {
        let palette = get_preset_palette(Preset::GameBoy).unwrap();
        assert_eq!(palette.len(), 4);
    }

    #[test]
    fn test_get_preset_palette_cga() {
        let palette = get_preset_palette(Preset::Cga).unwrap();
        assert_eq!(palette.len(), 16);
    }
}
