//! Palette distribution remapping.
//!
//! This module matches an image's color distribution to a palette's color
//! distribution in Oklab. The result is a full-color image, not a quantized
//! one. Dithering runs on it afterwards.
//!
//! # Algorithm Overview
//!
//! The remap works on two distributions:
//!
//! 1. Lightness. The stage builds the image's lightness percentiles and the
//!    palette's lightness quantiles. It moves each pixel to the palette
//!    lightness that holds its percentile. The darkest tones land on the
//!    darkest palette entry and the lightest on the lightest.
//! 2. Chroma. The stage scales the image chroma so its high end reaches the
//!    palette's high end. Near-grey pixels have no hue to scale, so they
//!    borrow hue from the palette entries near their new lightness.
//!
//! A `strength` value blends the result with the original color.
//!
//! # Example
//!
//! ```no_run
//! use palettize::{dither, remap_to_palette, Palette};
//!
//! let img = image::open("input.png").unwrap();
//! let palette: Palette = vec![(26, 28, 76), (240, 208, 96)].into();
//!
//! let remapped = remap_to_palette(&img, &palette, 1.0);
//! let output = dither(&image::DynamicImage::ImageRgb8(remapped), &palette);
//! output.save("output.png").unwrap();
//! ```

use crate::oklab::{Lab, chroma, oklab_to_srgb, srgb_to_oklab};
use crate::palette::Palette;
use image::{DynamicImage, ImageBuffer, RgbImage};

/// Maximum number of pixels to sample when measuring the image distribution.
const MAX_SAMPLES: usize = 10_000;

/// Chroma below this counts as sensor or compression noise, not color.
const NOISE_CHROMA: f32 = 0.02;

/// Chroma above this counts as real color the pixel owns.
const REAL_CHROMA: f32 = 0.08;

/// Upper bound on the chroma gain. It stops a near-grey image from amplifying
/// its noise without limit.
const MAX_CHROMA_GAIN: f32 = 4.0;

/// Percentile that sets the image's high chroma end. It ignores a handful of
/// extreme pixels that would otherwise set the scale for the whole image.
const CHROMA_PERCENTILE: f32 = 0.95;

/// Narrowest Gaussian width for the palette arc, in Oklab lightness units.
const MIN_ARC_SIGMA: f32 = 0.03;

/// Widest Gaussian width for the palette arc, in Oklab lightness units.
const MAX_ARC_SIGMA: f32 = 0.15;

/// Remaps an image's color distribution onto a palette's color distribution.
///
/// This is an optional stage that runs before dithering. It returns a
/// full-color image, so pass the result to [`crate::dither()`] or
/// [`crate::dither_with_options()`] to quantize it.
///
/// The default dithering path picks the nearest palette color for each pixel,
/// which is a colorimetric match. This function is the perceptual-intent
/// alternative: it stretches the image's tonal range across the palette's
/// tonal range and lends the palette's hues to pixels that have little color
/// of their own. A washed-out photo against a vivid palette gains the full
/// palette instead of collapsing onto the few entries nearest its own colors.
///
/// # Arguments
///
/// * `image` - The input image to remap
/// * `palette` - The color palette whose distribution to match
/// * `strength` - Blend between the original and the match. 0.0 returns the
///   input unchanged, 1.0 applies the full match. Values are clamped to
///   0.0-1.0.
///
/// # Returns
///
/// A new full-color [`RgbImage`] with the remapped distribution.
///
/// # Panics
///
/// Panics if the palette is empty.
///
/// # Examples
///
/// ```no_run
/// use palettize::{dither, remap_to_palette, Palette};
///
/// let img = image::open("input.png").unwrap();
/// let palette: Palette = vec![
///     (26, 28, 76),    // Dark blue
///     (240, 208, 96),  // Light yellow
/// ].into();
///
/// // Match the distribution first, then dither the result
/// let remapped = remap_to_palette(&img, &palette, 1.0);
/// let output = dither(&image::DynamicImage::ImageRgb8(remapped), &palette);
/// output.save("output.png").unwrap();
/// ```
///
/// A half-strength remap keeps more of the original color:
///
/// ```no_run
/// use palettize::{grayscale, remap_to_palette};
///
/// let img = image::open("input.png").unwrap();
/// let palette = grayscale(6);
/// let remapped = remap_to_palette(&img, &palette, 0.5);
/// ```
pub fn remap_to_palette(image: &DynamicImage, palette: &Palette, strength: f32) -> RgbImage {
    let colors = palette.colors();
    assert!(
        !colors.is_empty(),
        "remap_to_palette requires a non-empty palette"
    );

    let strength = strength.clamp(0.0, 1.0);
    let rgb = image.to_rgb8();
    let (width, height) = rgb.dimensions();

    let palette_labs: Vec<Lab> = colors
        .iter()
        .map(|c| srgb_to_oklab(c.r, c.g, c.b))
        .collect();

    // Palette lightness quantiles, ascending.
    let mut palette_ls: Vec<f32> = palette_labs.iter().map(|lab| lab.l).collect();
    palette_ls.sort_by(|a, b| a.total_cmp(b));

    let samples = sample_labs(&rgb, MAX_SAMPLES);

    // Image lightness percentiles, ascending.
    let mut sample_ls: Vec<f32> = samples.iter().map(|lab| lab.l).collect();
    sample_ls.sort_by(|a, b| a.total_cmp(b));

    let mut sample_chromas: Vec<f32> = samples.iter().map(|&lab| chroma(lab)).collect();
    sample_chromas.sort_by(|a, b| a.total_cmp(b));

    let chroma_in_hi = high_chroma(&sample_chromas);
    let chroma_pal_hi = palette_labs
        .iter()
        .map(|&lab| chroma(lab))
        .fold(0.0_f32, f32::max);

    let gain = (chroma_pal_hi / chroma_in_hi.max(1e-3)).clamp(0.0, MAX_CHROMA_GAIN);
    let chroma_ceiling = chroma_pal_hi * 1.05;
    let sigma = arc_sigma(&palette_ls);

    let mut output = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel = rgb.get_pixel(x, y);
            let lab = srgb_to_oklab(pixel[0], pixel[1], pixel[2]);

            let p = percentile(&sample_ls, lab.l);
            let target_l = quantile(&palette_ls, p);

            // The transfer branch keeps the pixel's own hue. The arc branch
            // lends palette hue to pixels that have almost none.
            let (a_transfer, b_transfer) = (lab.a * gain, lab.b * gain);
            let (a_arc, b_arc) = palette_arc(&palette_labs, sigma, target_l);

            let w = smoothstep(NOISE_CHROMA, REAL_CHROMA, chroma(lab));
            let mut a_target = w * a_transfer + (1.0 - w) * a_arc;
            let mut b_target = w * b_transfer + (1.0 - w) * b_arc;

            let target_chroma = a_target.hypot(b_target);
            if target_chroma > chroma_ceiling && target_chroma > 0.0 {
                let scale = chroma_ceiling / target_chroma;
                a_target *= scale;
                b_target *= scale;
            }

            let result = Lab {
                l: lab.l + strength * (target_l - lab.l),
                a: lab.a + strength * (a_target - lab.a),
                b: lab.b + strength * (b_target - lab.b),
            };

            let (r, g, b) = oklab_to_srgb(result);
            output.put_pixel(x, y, image::Rgb([r, g, b]));
        }
    }

    output
}

/// Samples up to `max_samples` pixels from an image as Oklab colors.
fn sample_labs(rgb: &RgbImage, max_samples: usize) -> Vec<Lab> {
    let total_pixels = rgb.width() as usize * rgb.height() as usize;

    if total_pixels <= max_samples {
        rgb.pixels()
            .map(|p| srgb_to_oklab(p[0], p[1], p[2]))
            .collect()
    } else {
        let step = total_pixels / max_samples;
        rgb.pixels()
            .enumerate()
            .filter(|(i, _)| i % step == 0)
            .take(max_samples)
            .map(|(_, p)| srgb_to_oklab(p[0], p[1], p[2]))
            .collect()
    }
}

/// Returns the midrank percentile of `value` in an ascending sorted slice.
///
/// The midrank splits the run of equal values. A flat image therefore maps to
/// the palette's mid-tones instead of collapsing onto its darkest entry.
fn percentile(sorted: &[f32], value: f32) -> f32 {
    if sorted.is_empty() {
        return 0.5;
    }
    let less = sorted.partition_point(|&x| x < value);
    let up_to = sorted.partition_point(|&x| x <= value);
    let equal = up_to - less;
    (less as f32 + 0.5 * equal as f32) / sorted.len() as f32
}

/// Returns the palette lightness at percentile `p`.
///
/// The result is piecewise linear through the sorted palette lightnesses, so
/// p = 0.0 gives the darkest entry and p = 1.0 gives the lightest.
fn quantile(sorted: &[f32], p: f32) -> f32 {
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    let position = p.clamp(0.0, 1.0) * (n - 1) as f32;
    let index = position.floor() as usize;
    if index >= n - 1 {
        return sorted[n - 1];
    }
    let fraction = position - index as f32;
    sorted[index] + (sorted[index + 1] - sorted[index]) * fraction
}

/// Returns the high end of an ascending sorted chroma slice.
fn high_chroma(sorted: &[f32]) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let index = (CHROMA_PERCENTILE * (sorted.len() - 1) as f32).round() as usize;
    sorted[index]
}

/// Returns the Gaussian width used by [`palette_arc`].
fn arc_sigma(palette_ls: &[f32]) -> f32 {
    let n = palette_ls.len();
    if n < 2 {
        return MIN_ARC_SIGMA;
    }
    let spread = palette_ls[n - 1] - palette_ls[0];
    (2.0 * spread / (n - 1) as f32).clamp(MIN_ARC_SIGMA, MAX_ARC_SIGMA)
}

/// Returns the palette's average chroma axes at lightness `l`.
///
/// Palette entries near `l` dominate the average. Entries far from `l` fade
/// out. For a palette with several hues at one lightness the hues partly
/// cancel toward neutral, which is the safe result.
fn palette_arc(palette_labs: &[Lab], sigma: f32, l: f32) -> (f32, f32) {
    if palette_labs.len() == 1 {
        return (palette_labs[0].a, palette_labs[0].b);
    }

    let denominator = 2.0 * sigma * sigma;
    let exponent = |lab: &Lab| {
        let d = l - lab.l;
        -(d * d) / denominator
    };

    // Shift by the largest exponent. Without it every weight can underflow to
    // zero when the palette sits far from this lightness.
    let peak = palette_labs.iter().map(exponent).fold(f32::MIN, f32::max);

    let mut sum_a = 0.0;
    let mut sum_b = 0.0;
    let mut sum_w = 0.0;
    for lab in palette_labs {
        let w = (exponent(lab) - peak).exp();
        sum_a += w * lab.a;
        sum_b += w * lab.b;
        sum_w += w;
    }

    (sum_a / sum_w, sum_b / sum_w)
}

/// Smooth cubic ramp from 0.0 at `edge0` to 1.0 at `edge1`.
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bayer::generate_bayer_matrix;
    use crate::dither::apply_dithering;
    use crate::palette::parse_hex_color;
    use std::collections::HashSet;

    /// Builds a horizontal ramp where each column has one color.
    fn ramp_image(width: u32, height: u32, color_at: impl Fn(u32) -> [u8; 3]) -> DynamicImage {
        let mut img = RgbImage::new(width, height);
        for x in 0..width {
            let c = color_at(x);
            for y in 0..height {
                img.put_pixel(x, y, image::Rgb(c));
            }
        }
        DynamicImage::ImageRgb8(img)
    }

    fn hex_palette(hexes: &[&str]) -> Palette {
        Palette::new(hexes.iter().map(|h| parse_hex_color(h).unwrap()).collect())
    }

    fn grey_ramp() -> DynamicImage {
        ramp_image(256, 8, |x| {
            let v = x as u8;
            [v, v, v]
        })
    }

    /// A DOS-like palette. Neither endpoint is pure black or pure white, so an
    /// endpoint assertion cannot pass by accident.
    fn dos_palette() -> Palette {
        hex_palette(&[
            "#101010", "#2a3b6b", "#3f7d4f", "#7a3030", "#8a6a2a", "#5d5d5d", "#b0b0c0", "#f0e0a0",
        ])
    }

    fn palette_l_range(palette: &Palette) -> (f32, f32) {
        let ls: Vec<f32> = palette
            .colors()
            .iter()
            .map(|c| srgb_to_oklab(c.r, c.g, c.b).l)
            .collect();
        (
            ls.iter().copied().fold(f32::MAX, f32::min),
            ls.iter().copied().fold(f32::MIN, f32::max),
        )
    }

    #[test]
    fn test_strength_zero_is_identity() {
        let img = ramp_image(64, 4, |x| [x as u8 * 4, 255 - x as u8 * 4, 128]);
        let palette = dos_palette();

        let output = remap_to_palette(&img, &palette, 0.0);
        let source = img.to_rgb8();

        for y in 0..source.height() {
            for x in 0..source.width() {
                let a = source.get_pixel(x, y);
                let b = output.get_pixel(x, y);
                for c in 0..3 {
                    let diff = (a[c] as i32 - b[c] as i32).abs();
                    assert!(
                        diff <= 1,
                        "channel {} at ({}, {}) drifted by {}",
                        c,
                        x,
                        y,
                        diff
                    );
                }
            }
        }
    }

    #[test]
    fn test_endpoints_reach_palette_extremes() {
        let img = grey_ramp();
        let palette = dos_palette();
        let (min_l, max_l) = palette_l_range(&palette);

        let output = remap_to_palette(&img, &palette, 1.0);

        let darkest = srgb_to_oklab(
            output.get_pixel(0, 0)[0],
            output.get_pixel(0, 0)[1],
            output.get_pixel(0, 0)[2],
        );
        let lightest = srgb_to_oklab(
            output.get_pixel(255, 0)[0],
            output.get_pixel(255, 0)[1],
            output.get_pixel(255, 0)[2],
        );

        assert!(
            (darkest.l - min_l).abs() < 0.05,
            "darkest output L {} vs palette min {}",
            darkest.l,
            min_l
        );
        assert!(
            (lightest.l - max_l).abs() < 0.05,
            "lightest output L {} vs palette max {}",
            lightest.l,
            max_l
        );
    }

    #[test]
    fn test_lightness_stays_monotone() {
        let img = grey_ramp();
        let palette = dos_palette();

        let output = remap_to_palette(&img, &palette, 1.0);

        let mut previous = f32::MIN;
        for x in 0..output.width() {
            let p = output.get_pixel(x, 0);
            let l = srgb_to_oklab(p[0], p[1], p[2]).l;
            assert!(
                l >= previous - 1e-3,
                "L dropped from {} to {} at column {}",
                previous,
                l,
                x
            );
            previous = l;
        }
    }

    #[test]
    fn test_grey_input_gains_palette_chroma() {
        let img = grey_ramp();
        let palette = hex_palette(&["#1a1c4c", "#f0d060"]);

        let output = remap_to_palette(&img, &palette, 1.0);

        let mut total = 0.0;
        for p in output.pixels() {
            total += chroma(srgb_to_oklab(p[0], p[1], p[2]));
        }
        let mean = total / output.pixels().len() as f32;

        assert!(mean > 0.03, "mean output chroma is {}", mean);
    }

    #[test]
    fn test_remap_widens_palette_use() {
        // A washed-out gradient: channels close together, tones between 60 and 190.
        let img = ramp_image(192, 64, |x| {
            let v = 60 + (x as f32 * 130.0 / 191.0) as u8;
            [v, v.saturating_add(6), v.saturating_sub(4)]
        });
        let palette = dos_palette();
        let bayer = generate_bayer_matrix(2);

        let direct = apply_dithering(&img, palette.colors(), &bayer, 1.0);
        let remapped = remap_to_palette(&img, &palette, 1.0);
        let via_remap = apply_dithering(
            &DynamicImage::ImageRgb8(remapped),
            palette.colors(),
            &bayer,
            1.0,
        );

        let count = |img: &RgbImage| -> usize {
            img.pixels()
                .map(|p| (p[0], p[1], p[2]))
                .collect::<HashSet<_>>()
                .len()
        };

        let direct_count = count(&direct);
        let remap_count = count(&via_remap);

        assert!(
            remap_count > direct_count,
            "remap used {} palette entries, direct used {}",
            remap_count,
            direct_count
        );
        assert!(
            remap_count >= 6,
            "remap used only {} palette entries",
            remap_count
        );
    }

    #[test]
    fn test_single_color_palette_still_dithers() {
        let img = grey_ramp();
        let palette = hex_palette(&["#3f7d4f"]);

        let remapped = remap_to_palette(&img, &palette, 1.0);
        let output = apply_dithering(
            &DynamicImage::ImageRgb8(remapped),
            palette.colors(),
            &generate_bayer_matrix(2),
            1.0,
        );

        assert_eq!(output.width(), 256);
        for p in output.pixels() {
            assert_eq!((p[0], p[1], p[2]), (0x3f, 0x7d, 0x4f));
        }
    }

    #[test]
    #[should_panic(expected = "remap_to_palette requires a non-empty palette")]
    fn test_empty_palette_panics() {
        let img = grey_ramp();
        remap_to_palette(&img, &Palette::new(vec![]), 1.0);
    }
}
