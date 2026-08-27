//! Integration tests for the palettize CLI.

use image::GenericImageView;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

/// Compare two images pixel by pixel.
fn compare_images_exact(
    image1_path: &Path,
    image2_path: &Path,
) -> Result<bool, Box<dyn std::error::Error>> {
    let img1 = image::open(image1_path)?;
    let img2 = image::open(image2_path)?;

    // Check dimensions match
    if img1.dimensions() != img2.dimensions() {
        eprintln!(
            "Dimension mismatch: {:?} vs {:?}",
            img1.dimensions(),
            img2.dimensions()
        );
        return Ok(false);
    }

    let (width, height) = img1.dimensions();
    let rgb1 = img1.to_rgb8();
    let rgb2 = img2.to_rgb8();

    let mut differences = 0;
    let mut first_diff: Option<(u32, u32)> = None;

    for y in 0..height {
        for x in 0..width {
            let pixel1 = rgb1.get_pixel(x, y);
            let pixel2 = rgb2.get_pixel(x, y);

            if pixel1 != pixel2 {
                if first_diff.is_none() {
                    first_diff = Some((x, y));
                    eprintln!(
                        "First difference at ({}, {}): {:?} vs {:?}",
                        x, y, pixel1, pixel2
                    );
                }
                differences += 1;
            }
        }
    }

    if differences > 0 {
        eprintln!(
            "Total differences: {} out of {} pixels ({:.2}%)",
            differences,
            width * height,
            (differences as f64 / (width * height) as f64) * 100.0
        );
        return Ok(false);
    }

    Ok(true)
}

/// Analyze and print differences between two images (for debugging).
#[allow(dead_code)]
fn analyze_image_differences(
    image1_path: &Path,
    image2_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let img1 = image::open(image1_path)?;
    let img2 = image::open(image2_path)?;

    let (width, height) = img1.dimensions();
    println!("Image dimensions: {}x{}", width, height);

    let rgb1 = img1.to_rgb8();
    let rgb2 = img2.to_rgb8();

    println!("\nSample pixel differences (first 10):");
    let mut shown = 0;
    for y in 0..height {
        for x in 0..width {
            let p1 = rgb1.get_pixel(x, y);
            let p2 = rgb2.get_pixel(x, y);
            if p1 != p2 && shown < 10 {
                println!(
                    "  ({:3}, {:3}): RGB({:3},{:3},{:3}) -> RGB({:3},{:3},{:3})",
                    x, y, p1[0], p1[1], p1[2], p2[0], p2[1], p2[2]
                );
                shown += 1;
            }
        }
    }

    // Collect unique colors
    let mut colors1 = HashSet::new();
    let mut colors2 = HashSet::new();

    for y in 0..height {
        for x in 0..width {
            let p1 = rgb1.get_pixel(x, y);
            let p2 = rgb2.get_pixel(x, y);
            colors1.insert((p1[0], p1[1], p1[2]));
            colors2.insert((p2[0], p2[1], p2[2]));
        }
    }

    let mut vec1: Vec<_> = colors1.into_iter().collect();
    let mut vec2: Vec<_> = colors2.into_iter().collect();
    vec1.sort();
    vec2.sort();

    println!("\nUnique colors in image1 ({}): {:?}", vec1.len(), vec1);
    println!("\nUnique colors in image2 ({}): {:?}", vec2.len(), vec2);

    Ok(())
}

/// Parse a palette hex file into a set of RGB colors.
///
/// Skips empty lines and lines starting with `//`, matching the CLI's own
/// palette-file parser.
fn parse_hex_palette(path: &Path) -> HashSet<(u8, u8, u8)> {
    let data = std::fs::read_to_string(path).expect("Failed to read palette file");
    data.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .map(|hex| {
            let r = u8::from_str_radix(&hex[0..2], 16).expect("Invalid hex color");
            let g = u8::from_str_radix(&hex[2..4], 16).expect("Invalid hex color");
            let b = u8::from_str_radix(&hex[4..6], 16).expect("Invalid hex color");
            (r, g, b)
        })
        .collect()
}

#[test]
fn test_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_palettize"))
        .arg("--help")
        .output()
        .expect("Failed to execute palettize --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("palettize"));
    assert!(stdout.contains("--grayscale"));
    assert!(stdout.contains("--palette"));
    assert!(stdout.contains("--auto"));
}

#[test]
fn test_cli_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_palettize"))
        .arg("--version")
        .output()
        .expect("Failed to execute palettize --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("palettize"));
    assert!(stdout.contains("0.2.0"));
}

#[test]
fn test_cli_default_palette() {
    // When no palette is specified, should default to black & white
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let input_path = Path::new(manifest_dir).join("tests/fixtures/gismonda.png");

    let output_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = output_dir.path().join("default-output.png");

    let status = Command::new(env!("CARGO_BIN_EXE_palettize"))
        .args([
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute palettize");

    // Should succeed with default black & white palette
    assert!(status.success());
    assert!(output_path.exists());

    // Verify output only contains black and white
    let img = image::open(&output_path).expect("Failed to open output");
    let rgb = img.to_rgb8();
    let bw_colors: HashSet<(u8, u8, u8)> = [(0, 0, 0), (255, 255, 255)].into_iter().collect();

    for pixel in rgb.pixels() {
        let color = (pixel[0], pixel[1], pixel[2]);
        assert!(
            bw_colors.contains(&color),
            "Unexpected color in output: {:?}",
            color
        );
    }
}

#[test]
fn test_custom_palette() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let input_path = Path::new(manifest_dir).join("tests/fixtures/gismonda.png");

    let output_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = output_dir.path().join("custom-output.png");

    let status = Command::new(env!("CARGO_BIN_EXE_palettize"))
        .args([
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
            "-p",
            "#FF0000,#00FF00,#0000FF",
        ])
        .status()
        .expect("Failed to execute palettize");

    assert!(status.success());
    assert!(output_path.exists());

    // Verify output only contains custom palette colors
    let img = image::open(&output_path).expect("Failed to open output");
    let rgb = img.to_rgb8();
    let custom_colors: HashSet<(u8, u8, u8)> = [(255, 0, 0), (0, 255, 0), (0, 0, 255)]
        .into_iter()
        .collect();

    for pixel in rgb.pixels() {
        let color = (pixel[0], pixel[1], pixel[2]);
        assert!(
            custom_colors.contains(&color),
            "Unexpected color in output: {:?}",
            color
        );
    }
}

#[test]
fn test_auto_extract_palette() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let input_path = Path::new(manifest_dir).join("tests/fixtures/gismonda.png");

    let output_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = output_dir.path().join("auto-output.png");
    let hex_path = output_dir.path().join("auto-output.hex");

    let status = Command::new(env!("CARGO_BIN_EXE_palettize"))
        .args([
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
            "-a",
            "8",
        ])
        .status()
        .expect("Failed to execute palettize");

    assert!(status.success());
    assert!(output_path.exists(), "Output image should exist");
    assert!(hex_path.exists(), "Palette file should exist");

    // Verify hex file has 8 colors
    let hex_content = std::fs::read_to_string(&hex_path).expect("Failed to read hex file");
    let color_count = hex_content.lines().filter(|l| !l.is_empty()).count();
    assert_eq!(color_count, 8, "Hex file should contain 8 colors");

    // Verify each line is a valid 6-character hex color
    for line in hex_content.lines() {
        if line.is_empty() {
            continue;
        }
        assert_eq!(line.len(), 6, "Each color should be 6 hex digits");
        assert!(
            line.chars().all(|c| c.is_ascii_hexdigit()),
            "Color should only contain hex digits: {}",
            line
        );
    }

    // Verify output image only contains exactly 8 colors
    let img = image::open(&output_path).expect("Failed to open output");
    let rgb = img.to_rgb8();
    let mut unique_colors = HashSet::new();
    for pixel in rgb.pixels() {
        unique_colors.insert((pixel[0], pixel[1], pixel[2]));
    }
    assert_eq!(
        unique_colors.len(),
        8,
        "Output image should contain exactly 8 colors, found {}",
        unique_colors.len()
    );
}

/// Palette configuration for regression tests
#[derive(Clone, Copy)]
enum Palette {
    Bw,
    Gray16,
    Gray255,
    Cga,
    Dos,
    Nes,
}

impl Palette {
    fn suffix(&self) -> &'static str {
        match self {
            Palette::Bw => "bw",
            Palette::Gray16 => "gray16",
            Palette::Gray255 => "gray255",
            Palette::Cga => "cga",
            Palette::Dos => "dos",
            Palette::Nes => "nes",
        }
    }

    fn cli_args(&self, manifest_dir: &Path) -> Vec<String> {
        match self {
            Palette::Bw => vec![],
            Palette::Gray16 => vec!["-g".to_string(), "16".to_string()],
            Palette::Gray255 => vec!["-g".to_string(), "255".to_string()],
            Palette::Cga => vec![
                "--palette-file".to_string(),
                manifest_dir
                    .join("tests/fixtures/cga.hex")
                    .to_str()
                    .unwrap()
                    .to_string(),
            ],
            Palette::Dos => vec![
                "--palette-file".to_string(),
                manifest_dir
                    .join("tests/fixtures/dos.hex")
                    .to_str()
                    .unwrap()
                    .to_string(),
            ],
            Palette::Nes => vec![
                "--palette-file".to_string(),
                manifest_dir
                    .join("tests/fixtures/nes.hex")
                    .to_str()
                    .unwrap()
                    .to_string(),
            ],
        }
    }
}

const INPUT_IMAGES: &[&str] = &[
    "gismonda",
    "spring-landscape",
    "the-departure",
    "villa-farnese",
];

const PALETTES: &[Palette] = &[
    Palette::Bw,
    Palette::Gray16,
    Palette::Gray255,
    Palette::Cga,
    Palette::Dos,
    Palette::Nes,
];

#[test]
fn test_regression_matrix() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    for image in INPUT_IMAGES {
        for palette in PALETTES {
            let input_path = manifest_dir.join(format!("tests/fixtures/{}.png", image));
            let expected_path =
                manifest_dir.join(format!("tests/fixtures/{}-{}.png", image, palette.suffix()));

            let output_dir = tempfile::tempdir().expect("Failed to create temp dir");
            let output_path =
                output_dir
                    .path()
                    .join(format!("{}-{}-generated.png", image, palette.suffix()));

            let mut args = vec![
                "-i".to_string(),
                input_path.to_str().unwrap().to_string(),
                "-o".to_string(),
                output_path.to_str().unwrap().to_string(),
            ];
            args.extend(palette.cli_args(manifest_dir));

            let status = Command::new(env!("CARGO_BIN_EXE_palettize"))
                .args(&args)
                .status()
                .expect("Failed to execute palettize");

            assert!(
                status.success(),
                "Palettize command failed for {} with {} palette",
                image,
                palette.suffix()
            );

            let matches = compare_images_exact(&output_path, &expected_path)
                .expect("Failed to compare images");

            assert!(
                matches,
                "Generated image does not match expected output for {} with {} palette",
                image,
                palette.suffix()
            );
        }
    }
}

#[test]
fn test_remap_flag() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let input_path = manifest_dir.join("tests/fixtures/gismonda.png");
    let palette_path = manifest_dir.join("tests/fixtures/dos.hex");

    let output_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let remap_output_path = output_dir.path().join("remap-output.png");
    let plain_output_path = output_dir.path().join("no-remap-output.png");

    let status = Command::new(env!("CARGO_BIN_EXE_palettize"))
        .args([
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            remap_output_path.to_str().unwrap(),
            "--palette-file",
            palette_path.to_str().unwrap(),
            "--remap",
        ])
        .status()
        .expect("Failed to execute palettize");

    assert!(status.success());
    assert!(remap_output_path.exists());

    let status = Command::new(env!("CARGO_BIN_EXE_palettize"))
        .args([
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            plain_output_path.to_str().unwrap(),
            "--palette-file",
            palette_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute palettize");

    assert!(status.success());
    assert!(plain_output_path.exists());

    // Every output pixel is a palette color
    let palette_colors = parse_hex_palette(&palette_path);
    let img = image::open(&remap_output_path).expect("Failed to open output");
    let rgb = img.to_rgb8();
    for pixel in rgb.pixels() {
        let color = (pixel[0], pixel[1], pixel[2]);
        assert!(
            palette_colors.contains(&color),
            "Unexpected color in output: {:?}",
            color
        );
    }

    // The flag changes the output
    let matches = compare_images_exact(&remap_output_path, &plain_output_path)
        .expect("Failed to compare images");
    assert!(!matches, "--remap should change the output");
}

#[test]
fn test_remap_strength_out_of_range() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let input_path = manifest_dir.join("tests/fixtures/gismonda.png");

    let output_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = output_dir.path().join("remap-out-of-range-output.png");

    let status = Command::new(env!("CARGO_BIN_EXE_palettize"))
        .args([
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
            "--remap",
            "1.5",
        ])
        .status()
        .expect("Failed to execute palettize");

    assert!(!status.success());
}
