//! Palette parsing and color palettes.
//!
//! This module handles loading color palettes from various sources:
//! - Hex color strings (e.g., "#FF0000")
//! - Generated grayscale palettes
//!
//! # Examples
//!
//! ```
//! use palettize::{Color, Palette, grayscale};
//!
//! // Create a color
//! let red = Color::new(255, 0, 0);
//! assert_eq!(red.r, 255);
//!
//! // Generate a grayscale palette
//! let palette = grayscale(2); // black & white
//! assert_eq!(palette.colors().len(), 2);
//!
//! // Create from tuples
//! let palette: Palette = vec![(0, 0, 0), (255, 255, 255)].into();
//! ```

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
/// constructors for creating palettes from custom colors or generators.
///
/// # Examples
///
/// ```
/// use palettize::{Color, Palette, grayscale};
///
/// // From a grayscale generator
/// let palette = grayscale(6);
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

/// Generates a grayscale palette with evenly-spaced stops.
///
/// # Arguments
/// * `stops` - Number of colors (2-256). 2 = black & white, 256 = full grayscale.
///
/// # Panics
/// Panics if stops < 2.
///
/// # Examples
///
/// ```
/// use palettize::{grayscale, Color};
///
/// // Black & white
/// let bw = grayscale(2);
/// assert_eq!(bw.colors().len(), 2);
/// assert_eq!(bw.colors()[0], Color::new(0, 0, 0));
/// assert_eq!(bw.colors()[1], Color::new(255, 255, 255));
///
/// // 6-level grayscale
/// let gray6 = grayscale(6);
/// assert_eq!(gray6.colors().len(), 6);
/// ```
pub fn grayscale(stops: u8) -> Palette {
    assert!(stops >= 2, "grayscale requires at least 2 stops");
    let colors = (0..stops)
        .map(|i| {
            let v = (i as u32 * 255 / (stops as u32 - 1)) as u8;
            Color::new(v, v, v)
        })
        .collect();
    Palette::new(colors)
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
    fn test_grayscale_2_stops() {
        let palette = grayscale(2);
        assert_eq!(palette.colors().len(), 2);
        assert_eq!(palette.colors()[0], Color::new(0, 0, 0));
        assert_eq!(palette.colors()[1], Color::new(255, 255, 255));
    }

    #[test]
    fn test_grayscale_6_stops() {
        let palette = grayscale(6);
        assert_eq!(palette.colors().len(), 6);
        assert_eq!(palette.colors()[0], Color::new(0, 0, 0));
        assert_eq!(palette.colors()[5], Color::new(255, 255, 255));
    }

    #[test]
    fn test_grayscale_256_stops() {
        let palette = grayscale(255);
        assert_eq!(palette.colors().len(), 255);
        assert_eq!(palette.colors()[0], Color::new(0, 0, 0));
        assert_eq!(palette.colors()[254], Color::new(255, 255, 255));
    }

    #[test]
    #[should_panic(expected = "grayscale requires at least 2 stops")]
    fn test_grayscale_1_stop_panics() {
        grayscale(1);
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
}
