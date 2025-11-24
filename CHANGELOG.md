# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure and setup
- **Terminal Color Capability Detection** (`src/utils/terminal_caps.rs`) - Epic 5 foundation (Story 5.1)
  - `ColorCapability` enum with 4 variants: `Monochrome`, `Ansi16`, `Ansi256`, `TrueColor`
  - `detect_color_capability()` function for automatic environment-based detection
  - Checks `$COLORTERM` for "truecolor"/"24bit" → `TrueColor`
  - Checks `$TERM` for "256color" → `Ansi256`, "color" → `Ansi16`
  - Safe fallback to `Ansi256` (widely supported)
  - `OnceLock` caching for <1ns repeated access after first detection
  - Helper methods: `supports_color()`, `supports_truecolor()`
  - Cross-platform support (Windows, Linux, macOS)
  - 39 unit tests with comprehensive coverage
  - `color_detection` example for visual validation
  - Zero clippy warnings, zero rustdoc warnings
- **Color Support for Drawing Primitives** (Story 4.5) - All drawing primitives now support RGB color
  - `draw_line_colored()` - Colored line drawing with optional thickness
  - `draw_circle_colored()` - Colored circles (outline or filled)
  - `draw_rectangle_colored()` - Colored rectangles (outline or filled)
  - `draw_polygon_colored()` - Colored polygons (open or closed paths)
  - All colored functions use per-cell color storage (2×4 dots per colored cell)
  - `colored_shapes` example demonstrating all colored primitive capabilities
  - Full backward compatibility (non-colored functions unchanged)
- **Character Density Rendering** (`src/density/`) - ASCII-art style intensity-to-character mapping
  - `DensitySet` type for intensity-to-character mapping with validation
  - Predefined density sets: ASCII (69 chars), Simple (10 chars), Blocks (5 chars), Braille (9 chars)
  - Custom density set creation with validation (1-256 characters)
  - `BrailleGrid::render_density()` API for rendering intensity buffers
  - Comprehensive unit tests (13 test cases, 100% coverage)
  - Integration tests for gradient patterns (horizontal, vertical, radial)
  - `density_demo` example showcasing all density sets and gradient types
  - Performance benchmarks for density mapping and grid rendering
- Line drawing primitives using Bresenham's algorithm (`draw_line`, `draw_line_thick`)
- Circle drawing primitives using Bresenham's midpoint circle algorithm (`draw_circle`, `draw_circle_filled`, `draw_circle_thick`)
- **Rectangle and Polygon Drawing Primitives** (`src/primitives/shapes.rs`)
  - Rectangle outline drawing (`draw_rectangle`)
  - Filled rectangle support (`draw_rectangle_filled`) with scanline fill
  - Thick rectangle borders (`draw_rectangle_thick`) with concentric approach
  - Polygon outline drawing (`draw_polygon`) with automatic path closing
  - Filled polygon support (`draw_polygon_filled`) using scanline fill algorithm with even-odd rule
  - Support for arbitrary polygon vertex counts (≥3 vertices required)
  - Handles non-convex and self-intersecting polygons correctly
  - Comprehensive unit tests (25 test cases, 100% coverage)
  - `shapes_demo` example demonstrating 21 shape variations
  - Performance benchmarks for all rectangle and polygon operations
- Primitives module (`src/primitives/`) with line, circle, and shape drawing capabilities
- Support for all octants (horizontal, vertical, diagonal, arbitrary angles) for lines
- 8-way symmetry for circles with integer-only arithmetic
- Line and circle thickness support (thickness 1-10 recommended for braille resolution)
- Filled circle support using scanline fill approach
- Boundary clipping for primitives extending beyond grid bounds
- Comprehensive unit tests for line drawing (8 test cases) and circle drawing (9 test cases)
- `lines_demo` and `circles_demo` examples demonstrating various drawing techniques
- Performance benchmarks for line and circle drawing primitives
- Adaptive resize filter selection for extreme aspect ratio images
- Comprehensive performance benchmarks for extreme image sizes
- Integration tests for large and extreme aspect ratio images

### Changed
- Image resize algorithm now uses Triangle filter (3x faster) for extreme aspect ratios (>2.5:1)
- Improved resize performance by 45% for extreme aspect ratio images (501ms → 276ms for 10000×4000 images)

### Performance
- Character density mapping: O(1) per cell, ~1μs per cell typical
- Density grid rendering (80×24 terminal): <2ms measured, <10ms target
- Density rendering scales linearly with grid size
- Large images (4000×4000): 222ms total
- Extreme wide (10000×4000): 725ms total (449ms load + 276ms resize)
- Extreme tall (4000×10000): ~700ms total
- All image sizes now meet <5s performance targets

## [0.1.0] - Unreleased

### Added
- Initial Cargo project initialization
- Project directory structure (src/, examples/, tests/, benches/, docs/)
- Dual MIT/Apache-2.0 licensing
- Basic README and documentation placeholders
