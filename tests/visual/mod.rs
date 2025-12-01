//! Visual regression testing framework for dotmax
//!
//! This module provides utilities for capturing braille output as text and
//! comparing it against known baselines to detect rendering changes.
//!
//! ## How it works
//!
//! 1. **Baseline Generation**: Create known-good output files
//! 2. **Comparison**: Compare current output against baselines
//! 3. **Failure Handling**: Report differences, optionally update baselines
//!
//! ## Usage
//!
//! ```ignore
//! use dotmax::tests::visual::{capture_grid, compare_with_baseline};
//!
//! let grid = BrailleGrid::new(10, 5).unwrap();
//! // ... draw to grid ...
//! let output = capture_grid(&grid);
//! compare_with_baseline("test_name", &output).unwrap();
//! ```
//!
//! ## Baseline Update Procedure
//!
//! To update baselines when intentional rendering changes occur:
//!
//! 1. Run tests with `UPDATE_BASELINES=1 cargo test visual`
//! 2. Review changes in `tests/visual/baselines/`
//! 3. Commit new baselines if changes are intentional

use dotmax::BrailleGrid;
use std::fs;
use std::path::Path;

/// Baseline directory location (relative to tests/)
const BASELINE_DIR: &str = "tests/visual/baselines";

/// Capture a BrailleGrid as a string of braille characters
///
/// Converts the grid to Unicode braille and joins rows with newlines.
/// This produces a human-readable representation suitable for comparison.
pub fn capture_grid(grid: &BrailleGrid) -> String {
    let unicode = grid.to_unicode_grid();
    unicode
        .into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n")
}

/// Compare output against a stored baseline
///
/// # Arguments
/// * `test_name` - Unique identifier for this test's baseline
/// * `actual` - The current output to compare
///
/// # Returns
/// * `Ok(())` if output matches baseline
/// * `Err(message)` if output differs or baseline missing
///
/// # Environment Variables
/// * `UPDATE_BASELINES=1` - Update baseline files instead of comparing
pub fn compare_with_baseline(test_name: &str, actual: &str) -> Result<(), String> {
    let baseline_path = format!("{}/{}.txt", BASELINE_DIR, test_name);
    let path = Path::new(&baseline_path);

    // Check if we should update baselines
    if std::env::var("UPDATE_BASELINES").is_ok() {
        fs::create_dir_all(BASELINE_DIR)
            .map_err(|e| format!("Failed to create baseline directory: {}", e))?;
        fs::write(path, actual)
            .map_err(|e| format!("Failed to write baseline: {}", e))?;
        return Ok(());
    }

    // Read existing baseline
    let expected = fs::read_to_string(path)
        .map_err(|_| format!("Baseline not found: {}. Run with UPDATE_BASELINES=1 to create.", baseline_path))?;

    // Compare
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "Visual regression detected in '{}'\n\n--- Expected ---\n{}\n\n--- Actual ---\n{}\n",
            test_name, expected, actual
        ))
    }
}

/// Generate a baseline for a test
///
/// This is a helper for explicitly creating/updating baselines.
/// Typically called once when adding new visual tests.
pub fn generate_baseline(test_name: &str, content: &str) -> Result<(), String> {
    let baseline_path = format!("{}/{}.txt", BASELINE_DIR, test_name);
    let path = Path::new(&baseline_path);

    fs::create_dir_all(BASELINE_DIR)
        .map_err(|e| format!("Failed to create baseline directory: {}", e))?;
    fs::write(path, content)
        .map_err(|e| format!("Failed to write baseline: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_grid_empty() {
        let grid = BrailleGrid::new(3, 2).unwrap();
        let output = capture_grid(&grid);
        // Empty grid should be braille blank characters (U+2800)
        assert!(output.contains('\u{2800}'));
        assert_eq!(output.lines().count(), 2);
    }

    #[test]
    fn test_capture_grid_with_dots() {
        let mut grid = BrailleGrid::new(2, 2).unwrap();
        grid.set_dot(0, 0).unwrap(); // Top-left dot
        let output = capture_grid(&grid);
        // Should have non-blank character in top-left
        assert!(!output.is_empty());
    }
}
