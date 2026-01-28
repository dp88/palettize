//! Bayer matrix generation for ordered dithering.
//!
//! This module provides functionality to generate Bayer matrices of various sizes,
//! which are used for ordered dithering in image processing. The Bayer matrix
//! defines a threshold pattern that determines how pixels are quantized.
//!
//! # Algorithm
//!
//! The Bayer matrix is generated recursively using the following formula:
//!
//! For a 2×2 base matrix:
//! ```text
//! | 0 2 |
//! | 3 1 |
//! ```
//!
//! For larger matrices, each quadrant is constructed by multiplying the previous
//! level's matrix by 4 and adding an offset based on the quadrant position.

/// Generates a Bayer matrix of the specified level.
///
/// The matrix size is determined by `2^(level+1)`, so:
/// - Level 0 produces a 2×2 matrix (4 thresholds)
/// - Level 1 produces a 4×4 matrix (16 thresholds)
/// - Level 2 produces an 8×8 matrix (64 thresholds)
/// - Level 3 produces a 16×16 matrix (256 thresholds)
/// - Level 4 produces a 32×32 matrix (1024 thresholds)
/// - Level 5 produces a 64×64 matrix (4096 thresholds)
///
/// The returned matrix contains normalized floating-point values in the range [0, 1).
///
/// # Arguments
///
/// * `level` - The recursion level (0-5 recommended)
///
/// # Returns
///
/// A 2D vector of normalized threshold values.
///
/// # Examples
///
/// ```
/// use palettize::generate_bayer_matrix;
///
/// // Generate a 2×2 matrix (level 0)
/// let matrix = generate_bayer_matrix(0);
/// assert_eq!(matrix.len(), 2);
/// assert_eq!(matrix[0].len(), 2);
///
/// // Generate an 8×8 matrix (level 2, commonly used)
/// let matrix = generate_bayer_matrix(2);
/// assert_eq!(matrix.len(), 8);
/// ```
///
/// # Panics
///
/// May panic or cause memory issues for very large levels (>10) due to
/// exponential matrix size growth.
pub fn generate_bayer_matrix(level: u32) -> Vec<Vec<f32>> {
    let int_matrix = generate_int(level);
    let size = int_matrix.len();
    let max_value = (size * size) as f32;

    // Convert to float and normalize
    let mut float_matrix = vec![vec![0.0; size]; size];
    for y in 0..size {
        for x in 0..size {
            float_matrix[y][x] = int_matrix[y][x] as f32 / max_value;
        }
    }

    float_matrix
}

/// Generates the integer form of a Bayer matrix recursively.
fn generate_int(level: u32) -> Vec<Vec<u32>> {
    let size = 2_usize.pow(level + 1);
    let mut matrix = vec![vec![0; size]; size];

    if level == 0 {
        // Base case: 2×2 matrix
        matrix[0][0] = 0;
        matrix[0][1] = 2;
        matrix[1][0] = 3;
        matrix[1][1] = 1;
    } else {
        // Recursive case
        let prev_matrix = generate_int(level - 1);
        let prev_size = prev_matrix.len();
        let multiplier = 4;

        for (y, row) in matrix.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                let prev_y = y % prev_size;
                let prev_x = x % prev_size;
                let quadrant_y = y / prev_size;
                let quadrant_x = x / prev_size;
                let quadrant = quadrant_y * 2 + quadrant_x;

                let offset = match quadrant {
                    0 => 0,
                    1 => 2,
                    2 => 3,
                    3 => 1,
                    _ => 0,
                };

                *cell = prev_matrix[prev_y][prev_x] * multiplier + offset;
            }
        }
    }

    matrix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_0_produces_2x2_matrix() {
        let matrix = generate_bayer_matrix(0);
        assert_eq!(matrix.len(), 2);
        assert_eq!(matrix[0].len(), 2);
    }

    #[test]
    fn test_level_2_produces_8x8_matrix() {
        let matrix = generate_bayer_matrix(2);
        assert_eq!(matrix.len(), 8);
        assert_eq!(matrix[0].len(), 8);
    }

    #[test]
    fn test_values_are_normalized() {
        let matrix = generate_bayer_matrix(2);
        for row in &matrix {
            for &value in row {
                assert!(value >= 0.0 && value < 1.0);
            }
        }
    }

    #[test]
    fn test_base_matrix_values() {
        let matrix = generate_bayer_matrix(0);
        // Expected: [[0, 2], [3, 1]] / 4
        assert!((matrix[0][0] - 0.0).abs() < f32::EPSILON);
        assert!((matrix[0][1] - 0.5).abs() < f32::EPSILON);
        assert!((matrix[1][0] - 0.75).abs() < f32::EPSILON);
        assert!((matrix[1][1] - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_all_values_unique() {
        let matrix = generate_bayer_matrix(2);
        let mut values: Vec<f32> = matrix.iter().flatten().copied().collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for i in 1..values.len() {
            assert!(values[i] > values[i - 1], "Values should be unique");
        }
    }
}
