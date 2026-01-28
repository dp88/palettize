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

#[test]
fn test_david_image_exact_match() {
    // Get the manifest directory (palettize-cli directory)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let input_path = Path::new(manifest_dir).join("tests/fixtures/david-in.png");
    let expected_path = Path::new(manifest_dir).join("tests/fixtures/david-out.png");

    // Use tempfile for output
    let output_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = output_dir.path().join("david-generated.png");

    // Run palettize command via cargo run -p palettize-cli
    let status = Command::new("cargo")
        .args([
            "run",
            "-p",
            "palettize-cli",
            "--",
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
            "-g",
            "6",
            "-b",
            "2",
            "-n",
            "0.05",
        ])
        .current_dir(Path::new(manifest_dir).parent().unwrap())
        .status()
        .expect("Failed to execute palettize");

    assert!(status.success(), "Palettize command failed");

    // Analyze differences if test will fail (for debugging)
    if let Ok(false) = compare_images_exact(&output_path, &expected_path) {
        println!("\n=== Analyzing image differences ===");
        let _ = analyze_image_differences(&output_path, &expected_path);
    }

    // Compare output with expected
    let matches =
        compare_images_exact(&output_path, &expected_path).expect("Failed to compare images");

    assert!(
        matches,
        "Generated image does not match expected output pixel-by-pixel"
    );
}

#[test]
fn test_cli_help() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let output = Command::new("cargo")
        .args(["run", "-p", "palettize-cli", "--", "--help"])
        .current_dir(Path::new(manifest_dir).parent().unwrap())
        .output()
        .expect("Failed to execute palettize --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("palettize"));
    assert!(stdout.contains("--grayscale"));
    assert!(stdout.contains("--palette"));
}

#[test]
fn test_cli_version() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let output = Command::new("cargo")
        .args(["run", "-p", "palettize-cli", "--", "--version"])
        .current_dir(Path::new(manifest_dir).parent().unwrap())
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
    let input_path = Path::new(manifest_dir).join("tests/fixtures/david-in.png");

    let output_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = output_dir.path().join("default-output.png");

    let status = Command::new("cargo")
        .args([
            "run",
            "-p",
            "palettize-cli",
            "--",
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .current_dir(Path::new(manifest_dir).parent().unwrap())
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
    let input_path = Path::new(manifest_dir).join("tests/fixtures/david-in.png");

    let output_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = output_dir.path().join("custom-output.png");

    let status = Command::new("cargo")
        .args([
            "run",
            "-p",
            "palettize-cli",
            "--",
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
            "-p",
            "#FF0000,#00FF00,#0000FF",
        ])
        .current_dir(Path::new(manifest_dir).parent().unwrap())
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
