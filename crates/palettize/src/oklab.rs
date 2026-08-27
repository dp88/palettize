//! sRGB to Oklab color conversion.
//!
//! Oklab is a perceptual color space by Björn Ottosson. Equal steps in Oklab
//! look like equal steps to the eye. The remap stage uses it to compare and
//! move lightness and chroma.
//!
//! See <https://bottosson.github.io/posts/oklab/> for the reference constants.

/// A color in the Oklab space.
///
/// `l` is perceptual lightness, 0.0 at black and 1.0 at white.
/// `a` and `b` are the opponent chroma axes.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Lab {
    /// Perceptual lightness (0.0-1.0).
    pub l: f32,
    /// Green-red opponent axis.
    pub a: f32,
    /// Blue-yellow opponent axis.
    pub b: f32,
}

/// Tolerance for the gamut test on linear RGB channels.
const GAMUT_TOLERANCE: f32 = 1e-4;

/// Iterations of the binary search that pulls a color back into gamut.
const GAMUT_STEPS: u32 = 12;

/// Converts one sRGB channel to linear light.
fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// Converts one linear light channel to sRGB.
fn linear_to_srgb(v: f32) -> f32 {
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// Converts an sRGB color to Oklab.
pub(crate) fn srgb_to_oklab(r: u8, g: u8, b: u8) -> Lab {
    let lr = srgb_to_linear(r as f32 / 255.0);
    let lg = srgb_to_linear(g as f32 / 255.0);
    let lb = srgb_to_linear(b as f32 / 255.0);

    let l = 0.412_221_47 * lr + 0.536_332_54 * lg + 0.051_445_995 * lb;
    let m = 0.211_903_5 * lr + 0.680_699_5 * lg + 0.107_396_96 * lb;
    let s = 0.088_302_46 * lr + 0.281_718_85 * lg + 0.629_978_7 * lb;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    Lab {
        l: 0.210_454_26 * l_ + 0.793_617_8 * m_ - 0.004_072_047 * s_,
        a: 1.977_998_5 * l_ - 2.428_592_2 * m_ + 0.450_593_7 * s_,
        b: 0.025_904_037 * l_ + 0.782_771_77 * m_ - 0.808_675_77 * s_,
    }
}

/// Converts an Oklab color to linear RGB.
fn oklab_to_linear(lab: Lab) -> (f32, f32, f32) {
    let l_ = lab.l + 0.396_337_78 * lab.a + 0.215_803_76 * lab.b;
    let m_ = lab.l - 0.105_561_346 * lab.a - 0.063_854_17 * lab.b;
    let s_ = lab.l - 0.089_484_18 * lab.a - 1.291_485_5 * lab.b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    (
        4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
        -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
        -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
    )
}

/// Reports whether all linear channels sit inside `[0, 1]` within tolerance.
fn in_gamut(rgb: (f32, f32, f32)) -> bool {
    let inside = |v: f32| (-GAMUT_TOLERANCE..=1.0 + GAMUT_TOLERANCE).contains(&v);
    inside(rgb.0) && inside(rgb.1) && inside(rgb.2)
}

/// Converts an Oklab color to sRGB.
///
/// Colors outside the sRGB gamut keep their lightness and hue. A binary search
/// scales the chroma down until the color fits. A residual clamp catches
/// lightness that no chroma scale can rescue.
pub(crate) fn oklab_to_srgb(lab: Lab) -> (u8, u8, u8) {
    let mut linear = oklab_to_linear(lab);

    if !in_gamut(linear) {
        let mut lo = 0.0_f32;
        let mut hi = 1.0_f32;
        for _ in 0..GAMUT_STEPS {
            let mid = 0.5 * (lo + hi);
            let scaled = Lab {
                l: lab.l,
                a: lab.a * mid,
                b: lab.b * mid,
            };
            if in_gamut(oklab_to_linear(scaled)) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        linear = oklab_to_linear(Lab {
            l: lab.l,
            a: lab.a * lo,
            b: lab.b * lo,
        });
    }

    let encode = |v: f32| (linear_to_srgb(v.clamp(0.0, 1.0)) * 255.0).round() as u8;
    (encode(linear.0), encode(linear.1), encode(linear.2))
}

/// Returns the chroma of an Oklab color, the distance from the neutral axis.
pub(crate) fn chroma(lab: Lab) -> f32 {
    lab.a.hypot(lab.b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip_within_one_step() {
        let levels = [0u8, 51, 102, 153, 204, 255];
        for &r in &levels {
            for &g in &levels {
                for &b in &levels {
                    let (r2, g2, b2) = oklab_to_srgb(srgb_to_oklab(r, g, b));
                    let diff = |a: u8, b: u8| (a as i32 - b as i32).abs();
                    assert!(
                        diff(r, r2) <= 1 && diff(g, g2) <= 1 && diff(b, b2) <= 1,
                        "round trip of ({}, {}, {}) gave ({}, {}, {})",
                        r,
                        g,
                        b,
                        r2,
                        g2,
                        b2
                    );
                }
            }
        }
    }

    #[test]
    fn test_lightness_ordering() {
        let black = srgb_to_oklab(0, 0, 0);
        let grey = srgb_to_oklab(128, 128, 128);
        let white = srgb_to_oklab(255, 255, 255);

        assert!(black.l < grey.l);
        assert!(grey.l < white.l);
        assert!(black.l.abs() < 0.01, "black L is {}", black.l);
        assert!((white.l - 1.0).abs() < 0.01, "white L is {}", white.l);
    }

    #[test]
    fn test_chroma_separates_color_from_grey() {
        let red = srgb_to_oklab(255, 0, 0);
        let grey = srgb_to_oklab(128, 128, 128);

        assert!(chroma(red) > 0.15, "red chroma is {}", chroma(red));
        assert!(chroma(grey) < 1e-3, "grey chroma is {}", chroma(grey));
    }

    #[test]
    fn test_matches_published_reference_values() {
        // Reference Oklab values for the sRGB primaries, from the CSS Color 4
        // specification. A round trip cannot catch a wrong matrix pair, because
        // a consistent pair of wrong matrices round trips perfectly. These
        // values pin the constants to the real color space.
        let cases = [
            ((255u8, 0u8, 0u8), (0.627_955, 0.224_863, 0.125_846)),
            ((0, 255, 0), (0.866_440, -0.233_888, 0.179_498)),
            ((0, 0, 255), (0.452_014, -0.032_457, -0.311_528)),
            ((255, 255, 255), (1.0, 0.0, 0.0)),
            ((0, 0, 0), (0.0, 0.0, 0.0)),
        ];

        for ((r, g, b), (l, a, bb)) in cases {
            let got = srgb_to_oklab(r, g, b);
            assert!(
                (got.l - l).abs() < 1e-3 && (got.a - a).abs() < 1e-3 && (got.b - bb).abs() < 1e-3,
                "({}, {}, {}) gave L {} a {} b {}, expected L {} a {} b {}",
                r,
                g,
                b,
                got.l,
                got.a,
                got.b,
                l,
                a,
                bb
            );
        }
    }

    #[test]
    fn test_greys_have_no_chroma() {
        for v in [0u8, 32, 64, 128, 192, 255] {
            let lab = srgb_to_oklab(v, v, v);
            assert!(
                lab.a.abs() < 1e-4 && lab.b.abs() < 1e-4,
                "grey {} has a {} b {}",
                v,
                lab.a,
                lab.b
            );
        }
    }
}
