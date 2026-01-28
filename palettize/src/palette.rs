//! Palette parsing and preset color palettes.
//!
//! This module handles loading color palettes from various sources:
//! - Hex color strings (e.g., "#FF0000")
//! - Built-in preset palettes (e.g., "gameboy", "cga")
//!
//! # Examples
//!
//! ```
//! use palettize::{Color, Palette, Preset};
//!
//! // Create a color
//! let red = Color::new(255, 0, 0);
//! assert_eq!(red.r, 255);
//!
//! // Get a preset palette
//! let palette = Palette::from_preset(Preset::GameBoy);
//! assert_eq!(palette.colors().len(), 4);
//!
//! // Create from tuples
//! let palette: Palette = vec![(0, 0, 0), (255, 255, 255)].into();
//! ```

use std::str::FromStr;

/// A color in the palette.
///
/// Represents an RGB color with 8-bit components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
}

impl Color {
    /// Creates a new color from RGB components.
    ///
    /// # Examples
    ///
    /// ```
    /// use palettize::Color;
    ///
    /// let red = Color::new(255, 0, 0);
    /// assert_eq!(red.r, 255);
    /// assert_eq!(red.g, 0);
    /// assert_eq!(red.b, 0);
    /// ```
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Converts the color to a tuple.
    ///
    /// # Examples
    ///
    /// ```
    /// use palettize::Color;
    ///
    /// let color = Color::new(255, 128, 0);
    /// assert_eq!(color.to_tuple(), (255, 128, 0));
    /// ```
    pub const fn to_tuple(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}

impl From<Color> for (u8, u8, u8) {
    fn from(color: Color) -> Self {
        color.to_tuple()
    }
}

/// A collection of colors for dithering.
///
/// The `Palette` struct wraps a vector of colors and provides convenient
/// constructors for creating palettes from presets or custom colors.
///
/// # Examples
///
/// ```
/// use palettize::{Color, Palette, Preset};
///
/// // From a preset
/// let palette = Palette::from_preset(Preset::GameBoy);
///
/// // From a vector of colors
/// let palette = Palette::new(vec![
///     Color::new(0, 0, 0),
///     Color::new(255, 255, 255),
/// ]);
///
/// // From tuples using Into
/// let palette: Palette = vec![(0, 0, 0), (255, 255, 255)].into();
/// ```
#[derive(Debug, Clone)]
pub struct Palette {
    colors: Vec<Color>,
}

impl Palette {
    /// Creates a new palette from a vector of colors.
    ///
    /// # Examples
    ///
    /// ```
    /// use palettize::{Color, Palette};
    ///
    /// let palette = Palette::new(vec![
    ///     Color::new(0, 0, 0),
    ///     Color::new(255, 255, 255),
    /// ]);
    /// assert_eq!(palette.colors().len(), 2);
    /// ```
    pub fn new(colors: Vec<Color>) -> Self {
        Self { colors }
    }

    /// Creates a palette from a preset.
    ///
    /// # Examples
    ///
    /// ```
    /// use palettize::{Palette, Preset};
    ///
    /// let palette = Palette::from_preset(Preset::GameBoy);
    /// assert_eq!(palette.colors().len(), 4);
    /// ```
    pub fn from_preset(preset: Preset) -> Self {
        let colors = get_preset_colors(preset);
        Self { colors }
    }

    /// Returns a slice of the colors in this palette.
    ///
    /// # Examples
    ///
    /// ```
    /// use palettize::{Color, Palette};
    ///
    /// let palette: Palette = vec![(0, 0, 0), (255, 255, 255)].into();
    /// let colors = palette.colors();
    /// assert_eq!(colors.len(), 2);
    /// assert_eq!(colors[0], Color::new(0, 0, 0));
    /// ```
    pub fn colors(&self) -> &[Color] {
        &self.colors
    }
}

impl From<Vec<Color>> for Palette {
    fn from(colors: Vec<Color>) -> Self {
        Self::new(colors)
    }
}

impl From<Vec<(u8, u8, u8)>> for Palette {
    fn from(tuples: Vec<(u8, u8, u8)>) -> Self {
        Self::new(tuples.into_iter().map(Color::from).collect())
    }
}

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

/// Error returned when parsing an unknown preset name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsePresetError {
    name: String,
}

impl std::fmt::Display for ParsePresetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unknown preset: '{}'. Available presets: {}",
            self.name,
            Preset::all_names().join(", ")
        )
    }
}

impl std::error::Error for ParsePresetError {}

impl FromStr for Preset {
    type Err = ParsePresetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bw" => Ok(Preset::Bw),
            "rgb3bit" => Ok(Preset::Rgb3bit),
            "grayscale" => Ok(Preset::Grayscale),
            "gameboy" => Ok(Preset::GameBoy),
            "cga" => Ok(Preset::Cga),
            _ => Err(ParsePresetError {
                name: s.to_string(),
            }),
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

/// Error returned when parsing an invalid hex color.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseColorError {
    input: String,
    reason: &'static str,
}

impl std::fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid hex color '{}': {}", self.input, self.reason)
    }
}

impl std::error::Error for ParseColorError {}

/// Parses a hex color string into a [`Color`].
///
/// Accepts both formats: with or without the leading `#`.
///
/// # Arguments
///
/// * `hex` - A hex color string like "#FF0000" or "FF0000"
///
/// # Returns
///
/// A [`Color`] on success.
///
/// # Errors
///
/// Returns an error if the string is not a valid 6-character hex color.
///
/// # Examples
///
/// ```
/// use palettize::{parse_hex_color, Color};
///
/// assert_eq!(parse_hex_color("#FF0000").unwrap(), Color::new(255, 0, 0));
/// assert_eq!(parse_hex_color("00FF00").unwrap(), Color::new(0, 255, 0));
/// assert_eq!(parse_hex_color("  #0000ff  ").unwrap(), Color::new(0, 0, 255));
///
/// // Invalid colors return errors
/// assert!(parse_hex_color("#GGG").is_err());
/// assert!(parse_hex_color("#12345").is_err());
/// ```
pub fn parse_hex_color(hex: &str) -> Result<Color, ParseColorError> {
    let trimmed = hex.trim().trim_start_matches('#');
    if trimmed.len() != 6 {
        return Err(ParseColorError {
            input: hex.to_string(),
            reason: "expected 6 hex digits",
        });
    }

    let r = u8::from_str_radix(&trimmed[0..2], 16).map_err(|_| ParseColorError {
        input: hex.to_string(),
        reason: "invalid red component",
    })?;
    let g = u8::from_str_radix(&trimmed[2..4], 16).map_err(|_| ParseColorError {
        input: hex.to_string(),
        reason: "invalid green component",
    })?;
    let b = u8::from_str_radix(&trimmed[4..6], 16).map_err(|_| ParseColorError {
        input: hex.to_string(),
        reason: "invalid blue component",
    })?;

    Ok(Color::new(r, g, b))
}

/// Returns the colors for a preset palette.
///
/// This is the internal function that returns a `Vec<Color>`.
/// For public use, prefer [`Palette::from_preset`].
fn get_preset_colors(preset: Preset) -> Vec<Color> {
    match preset {
        Preset::Bw => vec![Color::new(0, 0, 0), Color::new(255, 255, 255)],
        Preset::Rgb3bit => vec![
            Color::new(0, 0, 0),
            Color::new(255, 0, 0),
            Color::new(0, 255, 0),
            Color::new(0, 0, 255),
            Color::new(255, 255, 0),
            Color::new(255, 0, 255),
            Color::new(0, 255, 255),
            Color::new(255, 255, 255),
        ],
        Preset::Grayscale => vec![
            Color::new(0, 0, 0),
            Color::new(51, 51, 51),
            Color::new(102, 102, 102),
            Color::new(153, 153, 153),
            Color::new(204, 204, 204),
            Color::new(255, 255, 255),
        ],
        Preset::GameBoy => vec![
            Color::new(15, 56, 15),
            Color::new(48, 98, 48),
            Color::new(139, 172, 15),
            Color::new(155, 188, 15),
        ],
        Preset::Cga => vec![
            Color::new(0, 0, 0),
            Color::new(0, 0, 170),
            Color::new(0, 170, 0),
            Color::new(0, 170, 170),
            Color::new(170, 0, 0),
            Color::new(170, 0, 170),
            Color::new(170, 85, 0),
            Color::new(170, 170, 170),
            Color::new(85, 85, 85),
            Color::new(85, 85, 255),
            Color::new(85, 255, 85),
            Color::new(85, 255, 255),
            Color::new(255, 85, 85),
            Color::new(255, 85, 255),
            Color::new(255, 255, 85),
            Color::new(255, 255, 255),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_new() {
        let color = Color::new(255, 128, 0);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_color_to_tuple() {
        let color = Color::new(255, 128, 0);
        assert_eq!(color.to_tuple(), (255, 128, 0));
    }

    #[test]
    fn test_color_from_tuple() {
        let color: Color = (255, 128, 0).into();
        assert_eq!(color, Color::new(255, 128, 0));
    }

    #[test]
    fn test_tuple_from_color() {
        let tuple: (u8, u8, u8) = Color::new(255, 128, 0).into();
        assert_eq!(tuple, (255, 128, 0));
    }

    #[test]
    fn test_palette_new() {
        let palette = Palette::new(vec![Color::new(0, 0, 0), Color::new(255, 255, 255)]);
        assert_eq!(palette.colors().len(), 2);
    }

    #[test]
    fn test_palette_from_colors() {
        let palette: Palette = vec![Color::new(0, 0, 0), Color::new(255, 255, 255)].into();
        assert_eq!(palette.colors().len(), 2);
    }

    #[test]
    fn test_palette_from_tuples() {
        let palette: Palette = vec![(0, 0, 0), (255, 255, 255)].into();
        assert_eq!(palette.colors().len(), 2);
        assert_eq!(palette.colors()[0], Color::new(0, 0, 0));
        assert_eq!(palette.colors()[1], Color::new(255, 255, 255));
    }

    #[test]
    fn test_palette_from_preset() {
        let palette = Palette::from_preset(Preset::GameBoy);
        assert_eq!(palette.colors().len(), 4);
    }

    #[test]
    fn test_parse_hex_with_hash() {
        assert_eq!(parse_hex_color("#FF0000").unwrap(), Color::new(255, 0, 0));
        assert_eq!(parse_hex_color("#00FF00").unwrap(), Color::new(0, 255, 0));
        assert_eq!(parse_hex_color("#0000FF").unwrap(), Color::new(0, 0, 255));
    }

    #[test]
    fn test_parse_hex_without_hash() {
        assert_eq!(parse_hex_color("FF0000").unwrap(), Color::new(255, 0, 0));
        assert_eq!(parse_hex_color("000000").unwrap(), Color::new(0, 0, 0));
        assert_eq!(parse_hex_color("FFFFFF").unwrap(), Color::new(255, 255, 255));
    }

    #[test]
    fn test_parse_hex_with_whitespace() {
        assert_eq!(parse_hex_color("  #FF0000  ").unwrap(), Color::new(255, 0, 0));
    }

    #[test]
    fn test_parse_hex_lowercase() {
        assert_eq!(parse_hex_color("#ff0000").unwrap(), Color::new(255, 0, 0));
        assert_eq!(parse_hex_color("#aabbcc").unwrap(), Color::new(170, 187, 204));
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
    fn test_preset_palette_bw() {
        let palette = Palette::from_preset(Preset::Bw);
        assert_eq!(palette.colors().len(), 2);
        assert_eq!(palette.colors()[0], Color::new(0, 0, 0));
        assert_eq!(palette.colors()[1], Color::new(255, 255, 255));
    }

    #[test]
    fn test_preset_palette_gameboy() {
        let palette = Palette::from_preset(Preset::GameBoy);
        assert_eq!(palette.colors().len(), 4);
    }

    #[test]
    fn test_preset_palette_cga() {
        let palette = Palette::from_preset(Preset::Cga);
        assert_eq!(palette.colors().len(), 16);
    }
}
