// grid.rs - Core BrailleGrid data structure extracted from crabmusic
//
// Extracted from: https://github.com/newjordan/crabmusic
// Source files:
//   - crabmusic/src/visualization/braille.rs (BrailleGrid struct, dot manipulation)
//   - crabmusic/src/visualization/mod.rs (Color struct)
//
// Extraction strategy: Copy-Refactor-Test (ADR 0005)
//   1. Copy working code from crabmusic
//   2. Strip audio dependencies
//   3. Add Result-based error handling (zero panics policy)
//   4. Add comprehensive tests

// Import error types from error module
use crate::error::DotmaxError;

/// Maximum grid dimensions to prevent OOM attacks (NFR-S2)
const MAX_GRID_WIDTH: usize = 10_000;
const MAX_GRID_HEIGHT: usize = 10_000;

// ============================================================================
// Color struct - Extracted from crabmusic/src/visualization/mod.rs
// ============================================================================

/// RGB color representation for braille cells
///
/// Extracted from crabmusic. Story 2.6 will implement full color rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a new RGB color
    ///
    /// Extracted from `crabmusic::Color::new()`
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create black color (0, 0, 0)
    #[must_use]
    pub const fn black() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }

    /// Create white color (255, 255, 255)
    #[must_use]
    pub const fn white() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
        }
    }
}

// ============================================================================
// BrailleDot enum - Extracted from crabmusic/src/visualization/braille.rs:16-28
// ============================================================================

/// Braille dot positions
///
/// Extracted from crabmusic. Maps dot positions to bit patterns for Unicode braille.
///
/// Dot positions in a Braille character:
///   1 4
///   2 5
///   3 6
///   7 8
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrailleDot {
    Dot1 = 0b0000_0001,
    Dot2 = 0b0000_0010,
    Dot3 = 0b0000_0100,
    Dot4 = 0b0000_1000,
    Dot5 = 0b0001_0000,
    Dot6 = 0b0010_0000,
    Dot7 = 0b0100_0000,
    Dot8 = 0b1000_0000,
}

// ============================================================================
// dots_to_char - Extracted from crabmusic/src/visualization/braille.rs:52-56
// ============================================================================

/// Convert dot pattern to Braille Unicode character
///
/// Extracted from crabmusic. Story 2.2 will integrate this into the rendering pipeline.
///
/// # Arguments
/// * `dots` - Bit pattern where each bit represents a dot (1 = filled)
///
/// # Returns
/// Unicode Braille character
///
/// # Examples
///
/// ```
/// use dotmax::grid::dots_to_char;
///
/// // Empty pattern
/// assert_eq!(dots_to_char(0b00000000), '⠀');
///
/// // All dots filled
/// assert_eq!(dots_to_char(0b11111111), '⣿');
/// ```
#[inline]
#[must_use]
pub fn dots_to_char(dots: u8) -> char {
    // Braille patterns start at U+2800
    // SAFETY: crabmusic uses unwrap_or here; we keep the same logic
    // since 0x2800 + (0..=255) is always valid Unicode
    char::from_u32(0x2800 + u32::from(dots)).unwrap_or('⠀')
}

// ============================================================================
// BrailleGrid - Extracted from crabmusic/src/visualization/braille.rs:73-369
// ============================================================================

/// High-resolution grid using Braille characters
///
/// **Extracted from crabmusic** - Battle-tested rendering engine.
///
/// Each terminal cell contains a 2×4 dot pattern (8 dots total), giving us
/// high-resolution graphics in any terminal that supports Unicode braille.
///
/// ## Architecture (Preserved from crabmusic)
///
/// - **`patterns: Vec<u8>`** - Flat array, each u8 is a bitfield (8 bits = 8 dots)
/// - **Dot coordinates**: (`dot_x`, `dot_y`) in pixel space (width*2 × height*4)
/// - **Cell coordinates**: (`cell_x`, `cell_y`) in terminal space (width × height)
///
/// ## Dot Indexing (Unicode Braille Standard)
///
/// ```text
/// Braille cell (8 dots):
/// 1 4    (Dot1=0x01, Dot4=0x08)
/// 2 5    (Dot2=0x02, Dot5=0x10)
/// 3 6    (Dot3=0x04, Dot6=0x20)
/// 7 8    (Dot7=0x40, Dot8=0x80)
/// ```
///
/// # Example
///
/// ```
/// use dotmax::BrailleGrid;
///
/// let mut grid = BrailleGrid::new(40, 20).unwrap();
/// // Grid is 40×20 cells = 80×80 dot resolution
/// grid.set_dot(0, 0); // Top-left dot
/// grid.set_dot(1, 0); // Top-right dot of first cell
/// ```
pub struct BrailleGrid {
    /// Width in terminal cells
    width: usize,
    /// Height in terminal cells
    height: usize,
    /// Dot patterns for each cell (binary on/off)
    ///
    /// **Preserved from crabmusic**: Vec<u8> bitfield representation
    /// Each u8 represents one terminal cell with 8 dots
    patterns: Vec<u8>,
    /// Optional colors for each cell
    ///
    /// **Preserved from crabmusic**: Vec<Option<Color>>
    /// Story 2.6 will implement color rendering
    colors: Vec<Option<Color>>,
}

impl BrailleGrid {
    /// Create a new Braille grid
    ///
    /// **Extracted from `crabmusic::BrailleGrid::new()`** with added validation.
    ///
    /// # Arguments
    /// * `width` - Width in terminal cells (must be > 0 and <= `MAX_GRID_WIDTH`)
    /// * `height` - Height in terminal cells (must be > 0 and <= `MAX_GRID_HEIGHT`)
    ///
    /// # Returns
    /// * `Ok(BrailleGrid)` if dimensions are valid
    /// * `Err(DotmaxError::InvalidDimensions)` if width/height is 0 or exceeds max
    ///
    /// # Errors
    /// Returns `InvalidDimensions` if width or height is 0 or exceeds max allowed dimensions.
    ///
    /// # Crabmusic Change
    /// Original crabmusic code never validated dimensions.
    /// Dotmax adds validation for security (NFR-S2).
    pub fn new(width: usize, height: usize) -> Result<Self, DotmaxError> {
        // Validate dimensions (NEW - not in crabmusic)
        if width == 0 || height == 0 {
            return Err(DotmaxError::InvalidDimensions { width, height });
        }

        if width > MAX_GRID_WIDTH || height > MAX_GRID_HEIGHT {
            return Err(DotmaxError::InvalidDimensions { width, height });
        }

        // Allocate grid (PRESERVED from crabmusic)
        let size = width * height;
        Ok(Self {
            width,
            height,
            patterns: vec![0; size],
            colors: vec![None; size],
        })
    }

    /// Get width in terminal cells
    ///
    /// **Extracted from crabmusic** (lines 104-106)
    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Get height in terminal cells
    ///
    /// **Extracted from crabmusic** (lines 108-111)
    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Get width in dots (2× terminal width)
    ///
    /// **Extracted from crabmusic** (lines 113-116)
    #[must_use]
    pub const fn dot_width(&self) -> usize {
        self.width * 2
    }

    /// Get height in dots (4× terminal height)
    ///
    /// **Extracted from crabmusic** (lines 118-121)
    #[must_use]
    pub const fn dot_height(&self) -> usize {
        self.height * 4
    }

    /// Get the dimensions of the grid (dotmax addition for AC #7)
    ///
    /// **NEW** - Not in crabmusic. Added to satisfy AC #7 requirement.
    ///
    /// # Returns
    /// A tuple of (width, height) in terminal cells
    #[must_use]
    pub const fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Clear all dots
    ///
    /// **Extracted from crabmusic** (lines 124-127) with minor adaptation
    pub fn clear(&mut self) {
        self.patterns.fill(0);
        self.colors.fill(None);
    }

    /// Set a single dot at the specified position
    ///
    /// **Extracted from crabmusic** (lines 144-172) with added error handling.
    ///
    /// **CRITICAL**: This uses PIXEL coordinates (`dot_x`, `dot_y`), not cell coordinates.
    /// The grid is width*2 × height*4 dots.
    ///
    /// # Arguments
    /// * `dot_x` - X position in dots (0 to width*2-1)
    /// * `dot_y` - Y position in dots (0 to height*4-1)
    ///
    /// # Crabmusic Change
    /// Original crabmusic silently ignored out-of-bounds coordinates.
    /// Dotmax returns an error for explicit bounds checking (zero panics policy).
    ///
    /// # Errors
    /// Returns `OutOfBounds` if dot coordinates exceed grid dimensions.
    pub fn set_dot(&mut self, dot_x: usize, dot_y: usize) -> Result<(), DotmaxError> {
        // Bounds check (MODIFIED from crabmusic - return error instead of silent ignore)
        if dot_x >= self.dot_width() || dot_y >= self.dot_height() {
            return Err(DotmaxError::OutOfBounds {
                x: dot_x,
                y: dot_y,
                width: self.dot_width(),
                height: self.dot_height(),
            });
        }

        // Convert dot coordinates to cell coordinates (PRESERVED from crabmusic)
        let cell_x = dot_x / 2;
        let cell_y = dot_y / 4;
        let cell_index = cell_y * self.width + cell_x;

        // Determine which dot within the cell (0-7) (PRESERVED from crabmusic)
        let local_x = dot_x % 2;
        let local_y = dot_y % 4;

        // Map to Braille dot position (PRESERVED from crabmusic, lines 159-169)
        let dot_bit = match (local_x, local_y) {
            (0, 0) => BrailleDot::Dot1 as u8,
            (0, 1) => BrailleDot::Dot2 as u8,
            (0, 2) => BrailleDot::Dot3 as u8,
            (0, 3) => BrailleDot::Dot7 as u8,
            (1, 0) => BrailleDot::Dot4 as u8,
            (1, 1) => BrailleDot::Dot5 as u8,
            (1, 2) => BrailleDot::Dot6 as u8,
            (1, 3) => BrailleDot::Dot8 as u8,
            _ => unreachable!(),
        };

        // Set the dot (PRESERVED from crabmusic, line 171)
        self.patterns[cell_index] |= dot_bit;
        Ok(())
    }

    /// Get an individual dot value
    ///
    /// **NEW** - Not in crabmusic. Added to match AC #4 requirement.
    ///
    /// # Arguments
    /// * `dot_x` - X position in dots (0 to width*2-1)
    /// * `dot_y` - Y position in dots (0 to height*4-1)
    /// * `dot_index` - Dot position 0-7 in the cell
    ///
    /// # Returns
    /// * `Ok(bool)` - The dot value (true = enabled, false = disabled)
    /// * `Err(DotmaxError::OutOfBounds)` if coordinates exceed grid dimensions
    /// * `Err(DotmaxError::InvalidDotIndex)` if `dot_index` > 7
    ///
    /// # Errors
    /// Returns `OutOfBounds` if dot coordinates exceed grid dimensions, or `InvalidDotIndex` if dot index > 7.
    pub fn get_dot(&self, x: usize, y: usize, dot_index: u8) -> Result<bool, DotmaxError> {
        // Validate cell bounds
        if x >= self.width || y >= self.height {
            return Err(DotmaxError::OutOfBounds {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }

        // Validate dot index
        if dot_index > 7 {
            return Err(DotmaxError::InvalidDotIndex { index: dot_index });
        }

        // Calculate cell index
        let cell_index = y * self.width + x;
        let pattern = self.patterns[cell_index];

        // Check if dot is set using bit mask
        let dot_bit = 1u8 << dot_index;
        Ok((pattern & dot_bit) != 0)
    }

    /// Clear a rectangular region of the grid
    ///
    /// **NEW** - Not in crabmusic. Added to satisfy AC #6 requirement.
    ///
    /// # Arguments
    /// * `x` - Starting column in cells (0-indexed)
    /// * `y` - Starting row in cells (0-indexed)
    /// * `width` - Width of region to clear in cells
    /// * `height` - Height of region to clear in cells
    ///
    /// # Returns
    /// * `Ok(())` if region was cleared successfully
    /// * `Err(DotmaxError::OutOfBounds)` if region extends beyond grid bounds
    ///
    /// # Errors
    /// Returns `OutOfBounds` if the specified region extends beyond grid dimensions.
    pub fn clear_region(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<(), DotmaxError> {
        // Validate bounds - check if region fits within grid
        let end_x = x.saturating_add(width);
        let end_y = y.saturating_add(height);

        if end_x > self.width || end_y > self.height {
            return Err(DotmaxError::OutOfBounds {
                x: end_x.saturating_sub(1),
                y: end_y.saturating_sub(1),
                width: self.width,
                height: self.height,
            });
        }

        // Clear the specified region
        for row_idx in y..end_y {
            for col_idx in x..end_x {
                let cell_index = row_idx * self.width + col_idx;
                self.patterns[cell_index] = 0;
                self.colors[cell_index] = None;
            }
        }

        Ok(())
    }

    /// Get the Braille character at a cell position
    ///
    /// **Extracted from crabmusic** (lines 338-347)
    ///
    /// # Arguments
    /// * `cell_x` - X position in cells
    /// * `cell_y` - Y position in cells
    ///
    /// # Returns
    /// Braille character representing the dot pattern
    #[must_use]
    pub fn get_char(&self, cell_x: usize, cell_y: usize) -> char {
        if cell_x >= self.width || cell_y >= self.height {
            return '⠀';
        }

        let index = cell_y * self.width + cell_x;
        dots_to_char(self.patterns[index])
    }

    /// Get the color at a cell position
    ///
    /// **Extracted from crabmusic** (lines 350-357)
    #[must_use]
    pub fn get_color(&self, cell_x: usize, cell_y: usize) -> Option<Color> {
        if cell_x >= self.width || cell_y >= self.height {
            return None;
        }

        let index = cell_y * self.width + cell_x;
        self.colors[index]
    }

    /// Check if a cell has any dots set
    ///
    /// **Extracted from crabmusic** (lines 360-368)
    #[must_use]
    pub fn is_empty(&self, cell_x: usize, cell_y: usize) -> bool {
        if cell_x >= self.width || cell_y >= self.height {
            return true;
        }

        let index = cell_y * self.width + cell_x;
        self.patterns[index] == 0
    }

    // ========================================================================
    // Story 2.2: Unicode Braille Character Conversion
    // ========================================================================

    /// Convert entire grid to 2D array of Unicode braille characters
    ///
    /// **Story 2.2** - Batch conversion for rendering pipeline.
    ///
    /// This method converts the entire grid from dot patterns to Unicode braille
    /// characters, producing a 2D array that matches the grid dimensions.
    ///
    /// Uses the proven `dots_to_char()` function extracted from crabmusic
    /// (lines 53-56) which applies the Unicode Braille standard formula:
    /// `U+2800 + bitfield`
    ///
    /// # Returns
    /// A 2D vector of Unicode braille characters, where `result[y][x]` corresponds
    /// to cell `(x, y)` in the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use dotmax::BrailleGrid;
    ///
    /// let mut grid = BrailleGrid::new(5, 5).unwrap();
    /// grid.set_dot(0, 0).unwrap(); // Top-left dot of cell (0,0)
    ///
    /// let chars = grid.to_unicode_grid();
    /// assert_eq!(chars.len(), 5); // 5 rows
    /// assert_eq!(chars[0].len(), 5); // 5 columns
    /// assert_eq!(chars[0][0], '⠁'); // Cell (0,0) has dot 1 set
    /// ```
    ///
    /// # Performance
    /// Time complexity: O(width × height) - processes each cell once
    /// Allocates: `Vec<Vec<char>>` with dimensions matching grid size
    #[must_use]
    pub fn to_unicode_grid(&self) -> Vec<Vec<char>> {
        let mut result = Vec::with_capacity(self.height);

        for y in 0..self.height {
            let mut row = Vec::with_capacity(self.width);
            for x in 0..self.width {
                let index = y * self.width + x;
                // Use extracted crabmusic conversion function
                row.push(dots_to_char(self.patterns[index]));
            }
            result.push(row);
        }

        result
    }

    /// Convert single cell at (x, y) to Unicode braille character
    ///
    /// **Story 2.2** - Single-cell conversion with bounds validation.
    ///
    /// Returns the Unicode braille character for a specific cell, or an error
    /// if coordinates are out of bounds.
    ///
    /// # Arguments
    /// * `x` - X position in cells (0 to width-1)
    /// * `y` - Y position in cells (0 to height-1)
    ///
    /// # Returns
    /// * `Ok(char)` - Unicode braille character (U+2800 to U+28FF)
    /// * `Err(DotmaxError::OutOfBounds)` - If coordinates exceed grid dimensions
    ///
    /// # Examples
    ///
    /// ```
    /// use dotmax::BrailleGrid;
    ///
    /// let mut grid = BrailleGrid::new(10, 10).unwrap();
    /// grid.set_dot(2, 4).unwrap(); // Set a dot in cell (1,1)
    ///
    /// let ch = grid.cell_to_braille_char(1, 1).unwrap();
    /// assert!(ch >= '\u{2800}' && ch <= '\u{28FF}'); // Valid braille range
    /// ```
    ///
    /// # Errors
    /// Returns `OutOfBounds` if x >= width or y >= height.
    pub fn cell_to_braille_char(&self, x: usize, y: usize) -> Result<char, DotmaxError> {
        // Validate bounds
        if x >= self.width || y >= self.height {
            return Err(DotmaxError::OutOfBounds {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }

        // Convert cell pattern to Unicode
        let index = y * self.width + x;
        Ok(dots_to_char(self.patterns[index]))
    }
}

// ============================================================================
// STRIPPED from crabmusic - Not in Story 2.1 scope:
// ============================================================================
// - set_dot_with_color() → Story 2.6 (Color Support)
// - draw_line() / draw_line_with_color() → Epic 4 (Drawing Primitives)
// - draw_circle() → Epic 4 (Drawing Primitives)
// - Anti-aliasing logic → Out of scope (audio-reactive feature)
// - FFT/spectrum integration → Audio dependencies (discarded)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Grid Creation Tests (AC #2) - Adapted from crabmusic tests
    // ========================================================================

    #[test]
    fn test_new_valid_dimensions() {
        let grid = BrailleGrid::new(80, 24);
        assert!(grid.is_ok());
        let grid = grid.unwrap();
        assert_eq!(grid.dimensions(), (80, 24));
        assert_eq!(grid.width(), 80);
        assert_eq!(grid.height(), 24);
    }

    #[test]
    fn test_braille_grid_creation() {
        // Ported from crabmusic test_braille_grid_creation (line 389)
        let grid = BrailleGrid::new(40, 20).unwrap();
        assert_eq!(grid.width(), 40);
        assert_eq!(grid.height(), 20);
        assert_eq!(grid.dot_width(), 80);
        assert_eq!(grid.dot_height(), 80);
    }

    #[test]
    fn test_new_minimal_dimensions() {
        let grid = BrailleGrid::new(1, 1);
        assert!(grid.is_ok());
        assert_eq!(grid.unwrap().dimensions(), (1, 1));
    }

    #[test]
    fn test_new_large_dimensions() {
        let grid = BrailleGrid::new(200, 50);
        assert!(grid.is_ok());
        assert_eq!(grid.unwrap().dimensions(), (200, 50));
    }

    #[test]
    fn test_new_max_dimensions() {
        let grid = BrailleGrid::new(10_000, 10_000);
        assert!(grid.is_ok());
    }

    #[test]
    fn test_new_zero_width_returns_error() {
        let result = BrailleGrid::new(0, 24);
        assert!(matches!(
            result,
            Err(DotmaxError::InvalidDimensions {
                width: 0,
                height: 24
            })
        ));
    }

    #[test]
    fn test_new_zero_height_returns_error() {
        let result = BrailleGrid::new(80, 0);
        assert!(matches!(
            result,
            Err(DotmaxError::InvalidDimensions {
                width: 80,
                height: 0
            })
        ));
    }

    #[test]
    fn test_new_both_zero_returns_error() {
        let result = BrailleGrid::new(0, 0);
        assert!(matches!(
            result,
            Err(DotmaxError::InvalidDimensions {
                width: 0,
                height: 0
            })
        ));
    }

    #[test]
    fn test_new_exceeds_max_width() {
        let result = BrailleGrid::new(10_001, 100);
        assert!(matches!(result, Err(DotmaxError::InvalidDimensions { .. })));
    }

    #[test]
    fn test_new_exceeds_max_height() {
        let result = BrailleGrid::new(100, 10_001);
        assert!(matches!(result, Err(DotmaxError::InvalidDimensions { .. })));
    }

    // ========================================================================
    // Dot Manipulation Tests (AC #3, #4, #8) - Adapted from crabmusic
    // ========================================================================

    #[test]
    fn test_set_dot() {
        // Ported from crabmusic test_set_dot (line 398)
        let mut grid = BrailleGrid::new(10, 10).unwrap();

        // Set top-left dot of first cell (dot coordinate 0,0)
        grid.set_dot(0, 0).unwrap();
        assert_eq!(grid.get_char(0, 0), '⠁');

        // Set top-right dot of first cell (dot coordinate 1,0)
        grid.set_dot(1, 0).unwrap();
        assert_eq!(grid.get_char(0, 0), '⠉');
    }

    #[test]
    fn test_dot_positions() {
        // Ported from crabmusic test_dot_positions (line 476)
        let mut grid = BrailleGrid::new(10, 10).unwrap();

        // Test all 8 dot positions in first cell
        grid.clear();
        grid.set_dot(0, 0).unwrap(); // Dot 1
        assert_eq!(grid.patterns[0], 0b0000_0001);

        grid.clear();
        grid.set_dot(0, 1).unwrap(); // Dot 2
        assert_eq!(grid.patterns[0], 0b0000_0010);

        grid.clear();
        grid.set_dot(0, 2).unwrap(); // Dot 3
        assert_eq!(grid.patterns[0], 0b0000_0100);

        grid.clear();
        grid.set_dot(0, 3).unwrap(); // Dot 7
        assert_eq!(grid.patterns[0], 0b0100_0000);

        grid.clear();
        grid.set_dot(1, 0).unwrap(); // Dot 4
        assert_eq!(grid.patterns[0], 0b0000_1000);

        grid.clear();
        grid.set_dot(1, 1).unwrap(); // Dot 5
        assert_eq!(grid.patterns[0], 0b0001_0000);

        grid.clear();
        grid.set_dot(1, 2).unwrap(); // Dot 6
        assert_eq!(grid.patterns[0], 0b0010_0000);

        grid.clear();
        grid.set_dot(1, 3).unwrap(); // Dot 8
        assert_eq!(grid.patterns[0], 0b1000_0000);
    }

    #[test]
    fn test_get_dot_all_positions() {
        let mut grid = BrailleGrid::new(10, 10).unwrap();

        // Set all 8 dots in cell (5,5) using set_dot pixel API
        for dot_y in 0..4 {
            for dot_x in 0..2 {
                // Cell (5,5) corresponds to dot range (10-11, 20-23)
                grid.set_dot(5 * 2 + dot_x, 5 * 4 + dot_y).unwrap();
            }
        }

        // Verify all 8 dots are set using get_dot
        for dot_index in 0..8 {
            assert!(
                grid.get_dot(5, 5, dot_index).unwrap(),
                "Dot {dot_index} should be set"
            );
        }
    }

    #[test]
    fn test_set_dot_out_of_bounds() {
        let mut grid = BrailleGrid::new(10, 10).unwrap();
        // Grid is 10×10 cells = 20×40 dots
        let result = grid.set_dot(100, 5);
        assert!(matches!(result, Err(DotmaxError::OutOfBounds { .. })));
    }

    #[test]
    fn test_get_dot_out_of_bounds() {
        let grid = BrailleGrid::new(10, 10).unwrap();
        let result = grid.get_dot(100, 100, 0);
        assert!(matches!(result, Err(DotmaxError::OutOfBounds { .. })));
    }

    #[test]
    fn test_get_dot_invalid_dot_index() {
        let grid = BrailleGrid::new(10, 10).unwrap();
        let result = grid.get_dot(5, 5, 8);
        assert!(matches!(result, Err(DotmaxError::InvalidDotIndex { .. })));
    }

    #[test]
    fn test_bounds_checking() {
        // Ported from crabmusic test_bounds_checking (line 514)
        let mut grid = BrailleGrid::new(10, 10).unwrap();

        // Should return error, not panic
        let result = grid.set_dot(1000, 1000);
        assert!(result.is_err());

        assert_eq!(grid.get_char(1000, 1000), '⠀');
        assert!(grid.is_empty(1000, 1000));
    }

    // ========================================================================
    // Clear Operations Tests (AC #5, #6) - Adapted from crabmusic
    // ========================================================================

    #[test]
    fn test_clear() {
        // Ported from crabmusic test_clear (line 411)
        let mut grid = BrailleGrid::new(10, 10).unwrap();

        grid.set_dot(0, 0).unwrap();
        grid.set_dot(5, 5).unwrap();

        grid.clear();

        assert_eq!(grid.get_char(0, 0), '⠀');
        assert!(grid.is_empty(0, 0));
    }

    #[test]
    fn test_clear_empty_grid() {
        let mut grid = BrailleGrid::new(5, 5).unwrap();
        grid.clear(); // Should not panic on empty grid
        assert!(grid.is_empty(0, 0));
    }

    #[test]
    fn test_clear_region_basic() {
        let mut grid = BrailleGrid::new(20, 20).unwrap();

        // Set dots in various cells
        grid.set_dot(5 * 2, 5 * 4).unwrap(); // Cell (5,5)
        grid.set_dot(6 * 2, 6 * 4).unwrap(); // Cell (6,6)
        grid.set_dot(10 * 2, 10 * 4).unwrap(); // Cell (10,10)

        // Clear region (5, 5, 2, 2) - clears cells (5,5), (6,5), (5,6), (6,6)
        grid.clear_region(5, 5, 2, 2).unwrap();

        // Verify region cleared
        assert!(grid.is_empty(5, 5));
        assert!(grid.is_empty(6, 6));

        // Verify outside region unchanged
        assert!(!grid.is_empty(10, 10));
    }

    #[test]
    fn test_clear_region_out_of_bounds() {
        let mut grid = BrailleGrid::new(10, 10).unwrap();

        // Region extends beyond grid
        let result = grid.clear_region(5, 5, 10, 10);
        assert!(matches!(result, Err(DotmaxError::OutOfBounds { .. })));
    }

    #[test]
    fn test_clear_region_zero_size() {
        let mut grid = BrailleGrid::new(10, 10).unwrap();

        // Zero-size region should succeed (clears nothing)
        let result = grid.clear_region(5, 5, 0, 0);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Dimensions Test (AC #7)
    // ========================================================================

    #[test]
    fn test_dimensions_returns_correct_size() {
        let grid1 = BrailleGrid::new(80, 24).unwrap();
        assert_eq!(grid1.dimensions(), (80, 24));

        let grid2 = BrailleGrid::new(100, 50).unwrap();
        assert_eq!(grid2.dimensions(), (100, 50));

        let grid3 = BrailleGrid::new(1, 1).unwrap();
        assert_eq!(grid3.dimensions(), (1, 1));
    }

    // ========================================================================
    // Unicode Conversion Tests - Ported from crabmusic
    // ========================================================================

    #[test]
    fn test_dots_to_char() {
        // Ported from crabmusic test_dots_to_char (line 376)
        // Empty pattern
        assert_eq!(dots_to_char(0b0000_0000), '⠀');

        // All dots
        assert_eq!(dots_to_char(0b1111_1111), '⣿');

        // Single dots
        assert_eq!(dots_to_char(0b0000_0001), '⠁'); // Dot 1
        assert_eq!(dots_to_char(0b0000_1000), '⠈'); // Dot 4
    }

    // ========================================================================
    // Color Tests (AC #1 - Color struct)
    // ========================================================================

    #[test]
    fn test_color_rgb() {
        let color = Color::rgb(255, 128, 64);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
    }

    #[test]
    fn test_color_black() {
        let color = Color::black();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_color_white() {
        let color = Color::white();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 255);
    }

    #[test]
    fn test_color_equality() {
        let color1 = Color::rgb(100, 150, 200);
        let color2 = Color::rgb(100, 150, 200);
        let color3 = Color::rgb(100, 150, 201);

        assert_eq!(color1, color2);
        assert_ne!(color1, color3);
    }

    // ========================================================================
    // Story 2.2: Unicode Braille Character Conversion Tests (AC #4, #5)
    // ========================================================================

    /// Test all 256 braille patterns (exhaustive coverage for AC #4)
    ///
    /// This test verifies correctness of the Unicode Braille conversion
    /// for ALL possible 8-dot patterns (2^8 = 256 combinations).
    ///
    /// Tests the bitfield formula: U+2800 + (dots[0]<<0 | dots[1]<<1 | ... | dots[7]<<7)
    #[test]
    fn test_all_256_braille_patterns() {
        for bitfield in 0u8..=255 {
            let ch = dots_to_char(bitfield);
            let expected = char::from_u32(0x2800 + u32::from(bitfield)).unwrap();
            assert_eq!(
                ch, expected,
                "Failed for bitfield {bitfield:08b} (decimal {bitfield})"
            );
        }
    }

    /// Test empty cell → U+2800 (AC #5)
    #[test]
    fn test_empty_cell_is_u2800() {
        let ch = dots_to_char(0b0000_0000);
        assert_eq!(ch, '\u{2800}', "Empty cell should be blank braille U+2800");
    }

    /// Test full cell → U+28FF (AC #5)
    #[test]
    fn test_full_cell_is_u28ff() {
        let ch = dots_to_char(0b1111_1111);
        assert_eq!(ch, '\u{28FF}', "Full cell should be U+28FF (all dots)");
    }

    /// Test specific patterns match Unicode standard (AC #5)
    #[test]
    fn test_specific_braille_patterns() {
        // Pattern: dots [true, false, true, false, false, false, false, false]
        // Bitfield: 0b00000101 = 5
        // Expected: U+2805 = '⠅'
        assert_eq!(dots_to_char(0b0000_0101), '\u{2805}');

        // Pattern: dots [true, true, true, true, false, false, false, false]
        // Bitfield: 0b00001111 = 15
        // Expected: U+280F = '⠏'
        assert_eq!(dots_to_char(0b0000_1111), '\u{280F}');

        // Single dot patterns
        assert_eq!(dots_to_char(0b0000_0001), '\u{2801}'); // Dot 1 only
        assert_eq!(dots_to_char(0b0000_1000), '\u{2808}'); // Dot 4 only
        assert_eq!(dots_to_char(0b0100_0000), '\u{2840}'); // Dot 7 only
        assert_eq!(dots_to_char(0b1000_0000), '\u{2880}'); // Dot 8 only
    }

    /// Test `to_unicode_grid()` dimensions (AC #2)
    #[test]
    fn test_to_unicode_grid_dimensions() {
        // 5×5 grid → verify result is 5×5 Vec<Vec<char>>
        let grid = BrailleGrid::new(5, 5).unwrap();
        let chars = grid.to_unicode_grid();

        assert_eq!(chars.len(), 5, "Grid should have 5 rows");
        for row in &chars {
            assert_eq!(row.len(), 5, "Each row should have 5 columns");
        }
    }

    /// Test `to_unicode_grid()` with various grid sizes
    #[test]
    fn test_to_unicode_grid_various_sizes() {
        // 80×24 (standard terminal)
        let grid1 = BrailleGrid::new(80, 24).unwrap();
        let chars1 = grid1.to_unicode_grid();
        assert_eq!(chars1.len(), 24);
        assert_eq!(chars1[0].len(), 80);

        // 1×1 (minimal)
        let grid2 = BrailleGrid::new(1, 1).unwrap();
        let chars2 = grid2.to_unicode_grid();
        assert_eq!(chars2.len(), 1);
        assert_eq!(chars2[0].len(), 1);

        // 100×50 (large terminal)
        let grid3 = BrailleGrid::new(100, 50).unwrap();
        let chars3 = grid3.to_unicode_grid();
        assert_eq!(chars3.len(), 50);
        assert_eq!(chars3[0].len(), 100);
    }

    /// Test `to_unicode_grid()` with empty grid (all blank braille)
    #[test]
    fn test_to_unicode_grid_empty() {
        let grid = BrailleGrid::new(3, 3).unwrap();
        let chars = grid.to_unicode_grid();

        // All cells should be blank braille U+2800
        for row in chars {
            for ch in row {
                assert_eq!(ch, '\u{2800}', "Empty grid should have blank braille");
            }
        }
    }

    /// Test `to_unicode_grid()` with dots set
    #[test]
    fn test_to_unicode_grid_with_dots() {
        let mut grid = BrailleGrid::new(5, 5).unwrap();

        // Set top-left dot of cell (0,0)
        grid.set_dot(0, 0).unwrap();

        // Set all dots of cell (2,2)
        for dot_y in 0..4 {
            for dot_x in 0..2 {
                grid.set_dot(2 * 2 + dot_x, 2 * 4 + dot_y).unwrap();
            }
        }

        let chars = grid.to_unicode_grid();

        // Cell (0,0) should have dot 1 set → '⠁'
        assert_eq!(chars[0][0], '\u{2801}');

        // Cell (2,2) should have all dots → '⣿'
        assert_eq!(chars[2][2], '\u{28FF}');

        // Other cells should be blank
        assert_eq!(chars[1][1], '\u{2800}');
        assert_eq!(chars[4][4], '\u{2800}');
    }

    /// Test `cell_to_braille_char()` bounds validation (AC #3)
    #[test]
    fn test_cell_to_braille_char_out_of_bounds() {
        let grid = BrailleGrid::new(10, 10).unwrap();

        // Out of bounds → Err(OutOfBounds)
        let result1 = grid.cell_to_braille_char(100, 5);
        assert!(matches!(result1, Err(DotmaxError::OutOfBounds { .. })));

        let result2 = grid.cell_to_braille_char(5, 100);
        assert!(matches!(result2, Err(DotmaxError::OutOfBounds { .. })));

        let result3 = grid.cell_to_braille_char(10, 10); // Exactly at boundary
        assert!(matches!(result3, Err(DotmaxError::OutOfBounds { .. })));
    }

    /// Test `cell_to_braille_char()` returns correct character
    #[test]
    fn test_cell_to_braille_char_correct_conversion() {
        let mut grid = BrailleGrid::new(10, 10).unwrap();

        // Set specific pattern in cell (5,5)
        // Set dots to create pattern 0b00001111 (bitfield 15) → '⠏'
        grid.set_dot(5 * 2, 5 * 4).unwrap(); // Dot 1
        grid.set_dot(5 * 2, 5 * 4 + 1).unwrap(); // Dot 2
        grid.set_dot(5 * 2, 5 * 4 + 2).unwrap(); // Dot 3
        grid.set_dot(5 * 2 + 1, 5 * 4).unwrap(); // Dot 4

        let ch = grid.cell_to_braille_char(5, 5).unwrap();
        assert_eq!(ch, '\u{280F}');
    }

    /// Test `cell_to_braille_char()` for empty cells
    #[test]
    fn test_cell_to_braille_char_empty_cells() {
        let grid = BrailleGrid::new(10, 10).unwrap();

        // All cells should start as blank braille
        for y in 0..10 {
            for x in 0..10 {
                let ch = grid.cell_to_braille_char(x, y).unwrap();
                assert_eq!(ch, '\u{2800}', "Empty cell ({x}, {y}) should be blank");
            }
        }
    }

    /// Test that conversion is correct after clearing
    #[test]
    fn test_unicode_conversion_after_clear() {
        let mut grid = BrailleGrid::new(5, 5).unwrap();

        // Set some dots
        grid.set_dot(0, 0).unwrap();
        grid.set_dot(5, 5).unwrap();

        // Verify they're set
        assert_ne!(grid.cell_to_braille_char(0, 0).unwrap(), '\u{2800}');

        // Clear grid
        grid.clear();

        // Verify all cells are blank braille
        let chars = grid.to_unicode_grid();
        for row in chars {
            for ch in row {
                assert_eq!(ch, '\u{2800}');
            }
        }
    }

    /// Test Unicode range validity (all conversions produce valid braille)
    #[test]
    fn test_unicode_range_validity() {
        let grid = BrailleGrid::new(5, 5).unwrap();
        let chars = grid.to_unicode_grid();

        for row in chars {
            for ch in row {
                assert!(
                    ('\u{2800}'..='\u{28FF}').contains(&ch),
                    "Character {ch} is outside braille range U+2800-U+28FF"
                );
            }
        }
    }

    // ========================================================================
    // Story 2.4: Error Context Verification Tests (AC #3)
    // ========================================================================

    /// Test InvalidDimensions error message includes context (AC #3)
    #[test]
    fn test_invalid_dimensions_error_message_includes_context() {
        let result = BrailleGrid::new(0, 10);
        match result {
            Err(DotmaxError::InvalidDimensions { width, height }) => {
                let msg = format!("{}", DotmaxError::InvalidDimensions { width, height });
                assert!(msg.contains("0"), "Error message should include width=0");
                assert!(msg.contains("10"), "Error message should include height=10");
                assert!(
                    msg.contains("width") && msg.contains("height"),
                    "Error message should label dimensions"
                );
            }
            _ => panic!("Expected InvalidDimensions error"),
        }
    }

    /// Test OutOfBounds error message includes all context (AC #3)
    #[test]
    fn test_out_of_bounds_error_message_includes_all_context() {
        let mut grid = BrailleGrid::new(10, 10).unwrap();
        let result = grid.set_dot(100, 50);
        match result {
            Err(DotmaxError::OutOfBounds {
                x,
                y,
                width,
                height,
            }) => {
                let msg = format!(
                    "{}",
                    DotmaxError::OutOfBounds {
                        x,
                        y,
                        width,
                        height
                    }
                );
                assert!(msg.contains("100"), "Error message should include x=100");
                assert!(msg.contains("50"), "Error message should include y=50");
                // width*2=20 and height*4=40 for dot coordinates
                assert!(
                    msg.contains("20"),
                    "Error message should include dot_width=20"
                );
                assert!(
                    msg.contains("40"),
                    "Error message should include dot_height=40"
                );
            }
            _ => panic!("Expected OutOfBounds error"),
        }
    }

    /// Test InvalidDotIndex error message includes index (AC #3)
    #[test]
    fn test_invalid_dot_index_error_message_includes_index() {
        let grid = BrailleGrid::new(10, 10).unwrap();
        let result = grid.get_dot(5, 5, 10);
        match result {
            Err(DotmaxError::InvalidDotIndex { index }) => {
                let msg = format!("{}", DotmaxError::InvalidDotIndex { index });
                assert!(msg.contains("10"), "Error message should include index=10");
                assert!(
                    msg.contains("0-7"),
                    "Error message should specify valid range"
                );
            }
            _ => panic!("Expected InvalidDotIndex error"),
        }
    }

    /// Test exceeding maximum dimensions returns proper error (AC #1, #3)
    #[test]
    fn test_new_exceeds_both_max_dimensions() {
        let result = BrailleGrid::new(20_000, 20_000);
        assert!(
            matches!(result, Err(DotmaxError::InvalidDimensions { .. })),
            "Grid exceeding MAX_GRID_WIDTH and MAX_GRID_HEIGHT should return InvalidDimensions"
        );
    }

    /// Test set_dot with invalid dot index returns InvalidDotIndex (AC #1)
    #[test]
    fn test_set_dot_invalid_dot_index_high() {
        let grid = BrailleGrid::new(10, 10).unwrap();
        // set_dot uses dot coordinates, not dot_index, so we test via get_dot
        let result = grid.get_dot(5, 5, 255);
        assert!(
            matches!(result, Err(DotmaxError::InvalidDotIndex { index: 255 })),
            "Dot index 255 should return InvalidDotIndex error"
        );
    }
}
