# Epic Technical Specification: Drawing Primitives & Density Rendering

Date: 2025-11-21
Author: Frosty
Epic ID: 4
Status: Draft

---

## Overview

Epic 4 introduces geometric drawing primitives and character density-based rendering to the dotmax braille graphics library. This epic enables developers to programmatically draw lines, circles, rectangles, and polygons onto braille grids using industry-standard Bresenham algorithms, and to render grayscale intensities using ASCII-art style character density mappings.

These capabilities bridge the gap between image-based rendering (Epic 3) and direct dot manipulation (Epic 2), providing mid-level drawing APIs suitable for data visualization, generative art, terminal UI decorations, and procedural content generation. The drawing primitives complement the existing image pipeline by enabling dynamic, programmatic graphics without requiring pre-rendered image assets.

**Context from PRD**: Functional requirements FR21-FR27 (drawing primitives) and FR28-FR31 (character density rendering). These features extract proven crabmusic code (~500 lines) for line/circle/shape drawing and density-based intensity mapping, refactored into clean, testable, and documented modules.

---

## Objectives and Scope

### In Scope

**Core Drawing Primitives:**
- Bresenham line drawing algorithm (draw lines between two points)
- Bresenham circle drawing algorithm (draw circles with center and radius)
- Rectangle drawing (outline and filled)
- Polygon drawing from vertex lists
- Region filling with solid patterns
- Line thickness control (thin, medium, thick)
- Color support for drawing operations (if color mode enabled)

**Character Density Rendering:**
- Intensity-to-character mapping (0.0 to 1.0 → sparse to dense characters)
- Predefined character density sets (ASCII-art style gradients)
- Custom character density mappings
- Smooth gradients through density character selection
- Integration with grayscale buffers from image pipeline

**Quality & Integration:**
- Comprehensive unit tests (>80% coverage target from Epic 3 standard)
- Integration tests demonstrating primitives + density rendering
- Performance benchmarks (drawing speed, memory usage)
- Interactive examples showing drawing and density APIs
- Rustdoc documentation with algorithm explanations and examples

### Out of Scope (Deferred to Later Epics)

- **Anti-aliasing**: Bresenham produces pixel-perfect lines but no sub-pixel anti-aliasing (acceptable for braille resolution)
- **Bezier curves**: Only straight lines and circular arcs in Epic 4; splines deferred to future
- **Flood fill**: Region filling is solid patterns only; intelligent flood fill deferred
- **Text rendering**: Character density is for gradients, not font rendering
- **3D primitives**: 2D only; 3D raytracer is separate epic
- **Advanced color blending**: Color assignment is solid colors per cell; alpha blending/gradients deferred to Epic 5

---

## System Architecture Alignment

### Architecture Document References

**Module Location** (from architecture.md lines 117-124):
- `src/primitives.rs` - Bresenham line/circle, rectangle, polygon drawing
- `src/density.rs` - Intensity → character mapping, density sets

**Extraction Source** (from architecture.md line 168):
- **Epic 4: Primitives** - Extract ~500 lines from crabmusic (src/primitives.rs, src/density.rs)
- Proven algorithms with demonstrated output quality
- Refactor: Strip audio dependencies, add error handling, comprehensive testing

**Data Flow Integration** (from architecture.md lines 221-227):
```
Drawing Primitives (lines, circles, shapes)
    ↓
Processing Pipeline: Bresenham algorithms → Dot setting
    ↓
BrailleGrid (central state)
    ├── Dots: Vec<u8> (packed bit patterns)
    ├── Colors: Option<Vec<Color>> (per-cell RGB)
    └── Dimensions: (width, height) in cells
    ↓
TerminalRenderer → Terminal Output
```

**Core Constraints**:
- **Zero panics** (architecture.md line 8): All primitives validate inputs, return Results for invalid params
- **Minimal dependencies**: No new external crates required (algorithms are pure Rust)
- **Performance targets**: Drawing operations <1ms for typical use cases (100-pixel lines/circles)
- **Memory discipline**: <500KB per frame overhead (architecture.md line 50)

**Integration Points**:
- `grid.rs` - Primitives call `BrailleGrid::set_dot(x, y)` to manipulate grid state
- `error.rs` - New error variants: `PrimitivesError` for invalid coords/dimensions
- `color.rs` (Epic 5) - Forward compatibility: primitives accept `Option<Color>` for future use

---

## Detailed Design

### Services and Modules

| Module | Responsibilities | Inputs | Outputs | Owner |
|--------|------------------|--------|---------|-------|
| **primitives.rs** | Line, circle, rectangle, polygon drawing using Bresenham | Coordinates, dimensions, thickness | Modified BrailleGrid | Epic 4 |
| **density.rs** | Intensity → character mapping for ASCII-art style rendering | Intensity values (f32), density sets | Character selection | Epic 4 |
| **grid.rs** (Epic 2) | Provides `set_dot(x, y)` API for primitives to manipulate dots | Dot coordinates | Braille grid state | Epic 2 (existing) |
| **color.rs** (Epic 5) | Future: Provides `Color` type for colored primitives | RGB values | Color assignments | Epic 5 (future) |

**Module Dependencies**:
```
primitives.rs → grid.rs (set_dot), error.rs (PrimitivesError)
density.rs → grid.rs (set_dot), error.rs (DensityError)
```

**Public API Surface** (exported from `lib.rs`):
```rust
// primitives.rs exports
pub struct LineDrawer;
pub struct CircleDrawer;
pub struct RectangleDrawer;
pub struct PolygonDrawer;

// density.rs exports
pub struct DensitySet;
pub struct DensityRenderer;
pub const ASCII_DENSITY_LIGHT: DensitySet;
pub const ASCII_DENSITY_MEDIUM: DensitySet;
pub const ASCII_DENSITY_HEAVY: DensitySet;
```

---

### Data Models and Contracts

**1. Drawing Primitives Data Model**

```rust
// src/primitives.rs

/// Represents a 2D point in braille grid coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Line thickness for drawing operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineThickness {
    Thin = 1,    // Single pixel width
    Medium = 2,  // 2-pixel width
    Thick = 3,   // 3-pixel width
}

/// Drawing configuration (color support forward-compatible with Epic 5)
#[derive(Debug, Clone)]
pub struct DrawConfig {
    pub thickness: LineThickness,
    pub color: Option<Color>,  // None = monochrome, Some = colored (Epic 5)
}

impl Default for DrawConfig {
    fn default() -> Self {
        Self {
            thickness: LineThickness::Thin,
            color: None,
        }
    }
}
```

**2. Character Density Data Model**

```rust
// src/density.rs

/// Character density set for intensity-based rendering
/// Maps intensity range [0.0, 1.0] to characters (sparse → dense)
#[derive(Debug, Clone)]
pub struct DensitySet {
    /// Characters ordered from sparse (low intensity) to dense (high intensity)
    /// Example: [' ', '.', ':', '-', '=', '+', '*', '#', '@']
    pub characters: Vec<char>,
    pub name: String,
}

impl DensitySet {
    /// Create custom density set with validation
    pub fn new(name: String, characters: Vec<char>) -> Result<Self, DensityError> {
        if characters.is_empty() {
            return Err(DensityError::EmptyDensitySet);
        }
        if characters.len() > 256 {
            return Err(DensityError::TooManyCharacters);
        }
        Ok(Self { characters, name })
    }

    /// Map intensity [0.0, 1.0] to character
    pub fn map(&self, intensity: f32) -> char {
        let clamped = intensity.clamp(0.0, 1.0);
        let index = (clamped * (self.characters.len() - 1) as f32).round() as usize;
        self.characters[index]
    }
}

// Predefined density sets (extracted from crabmusic)
pub const ASCII_DENSITY_LIGHT: &str = " .:-=+*#";
pub const ASCII_DENSITY_MEDIUM: &str = " .'`^\",:;Il!i><~+_-?][}{1)(|/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$";
pub const ASCII_DENSITY_HEAVY: &str = " ░▒▓█";
```

**3. Error Types**

```rust
// src/error.rs additions

#[derive(Debug, thiserror::Error)]
pub enum PrimitivesError {
    #[error("Invalid coordinates: ({x}, {y}) outside grid bounds (width={width}, height={height})")]
    OutOfBounds { x: i32, y: i32, width: usize, height: usize },

    #[error("Invalid radius: {radius} (must be > 0)")]
    InvalidRadius { radius: i32 },

    #[error("Invalid dimensions: width={width}, height={height} (must be > 0)")]
    InvalidDimensions { width: i32, height: i32 },

    #[error("Invalid polygon: {reason}")]
    InvalidPolygon { reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum DensityError {
    #[error("Density set cannot be empty")]
    EmptyDensitySet,

    #[error("Density set has too many characters: {count} (max 256)")]
    TooManyCharacters { count: usize },

    #[error("Invalid intensity: {value} (must be 0.0 to 1.0)")]
    InvalidIntensity { value: f32 },
}
```

---

### APIs and Interfaces

**Drawing Primitives API**

```rust
// src/primitives.rs

impl BrailleGrid {
    /// Draw line from (x0, y0) to (x1, y1) using Bresenham algorithm
    pub fn draw_line(&mut self, p0: Point, p1: Point, config: &DrawConfig) -> Result<(), PrimitivesError> {
        // Bresenham line algorithm implementation
        // Validates coordinates against grid bounds
        // Applies thickness by drawing parallel lines
        // Sets dots via self.set_dot(x, y)
    }

    /// Draw circle centered at (cx, cy) with given radius
    pub fn draw_circle(&mut self, center: Point, radius: i32, config: &DrawConfig) -> Result<(), PrimitivesError> {
        // Bresenham circle algorithm (midpoint circle)
        // Validates center and radius
        // Applies thickness by drawing concentric circles
    }

    /// Draw rectangle outline or filled
    pub fn draw_rectangle(&mut self, top_left: Point, width: i32, height: i32, filled: bool, config: &DrawConfig) -> Result<(), PrimitivesError> {
        // Rectangle outline: 4 lines
        // Rectangle filled: scan line fill
        // Validates dimensions > 0
    }

    /// Draw polygon from vertex list
    pub fn draw_polygon(&mut self, vertices: &[Point], closed: bool, config: &DrawConfig) -> Result<(), PrimitivesError> {
        // Draws lines between consecutive vertices
        // If closed=true, connects last vertex to first
        // Validates: vertices.len() >= 2 (or 3 for closed)
    }

    /// Fill region with solid pattern (simple scan-line fill)
    pub fn fill_region(&mut self, top_left: Point, width: i32, height: i32, config: &DrawConfig) -> Result<(), PrimitivesError> {
        // Fill rectangular region by setting all dots
        // Used for filled rectangles, solid backgrounds
    }
}
```

**Character Density API**

```rust
// src/density.rs

impl BrailleGrid {
    /// Render intensity buffer using character density mapping
    /// intensity_buffer: Row-major array of f32 values [0.0, 1.0]
    /// density_set: Character mapping for intensity → char
    pub fn render_density(&mut self, intensity_buffer: &[f32], density_set: &DensitySet) -> Result<(), DensityError> {
        // Validates intensity_buffer.len() == grid.width() * grid.height()
        // Maps each intensity to character via density_set.map()
        // Renders character at corresponding grid position
        // Returns error if intensity values out of range
    }
}

/// Predefined density sets
pub fn light_density() -> DensitySet {
    DensitySet::new("Light".into(), ASCII_DENSITY_LIGHT.chars().collect()).unwrap()
}

pub fn medium_density() -> DensitySet {
    DensitySet::new("Medium".into(), ASCII_DENSITY_MEDIUM.chars().collect()).unwrap()
}

pub fn heavy_density() -> DensitySet {
    DensitySet::new("Heavy".into(), ASCII_DENSITY_HEAVY.chars().collect()).unwrap()
}
```

---

### Workflows and Sequencing

**Workflow 1: Drawing Shapes Programmatically**

```
1. Developer creates BrailleGrid (Epic 2 API)
   grid = BrailleGrid::builder()
            .dimensions(80, 24)
            .build()

2. Configure drawing parameters
   config = DrawConfig {
       thickness: LineThickness::Medium,
       color: None,  // Monochrome for Epic 4
   }

3. Draw primitives
   grid.draw_line(Point{x:10, y:10}, Point{x:70, y:20}, &config)?
   grid.draw_circle(Point{x:40, y:12}, 5, &config)?
   grid.draw_rectangle(Point{x:5, y:5}, 20, 10, false, &config)?

4. Render to terminal (Epic 2 API)
   renderer.render(&grid)?
```

**Workflow 2: Rendering Density-Based Gradients**

```
1. Developer generates intensity buffer (e.g., gradient, heatmap, terrain)
   let intensities = generate_gradient(width, height);  // Vec<f32>

2. Create or select density set
   let density = medium_density();

3. Render intensity buffer as characters
   grid.render_density(&intensities, &density)?

4. Display result
   renderer.render(&grid)?
```

**Workflow 3: Combining Primitives + Image Rendering**

```
1. Load and render image (Epic 3 API)
   let image_renderer = ImageRenderer::builder()
                          .image_path("photo.jpg")
                          .build()?;
   image_renderer.render(&mut grid)?;

2. Overlay drawing primitives (annotations, UI elements)
   let config = DrawConfig::default();
   grid.draw_rectangle(Point{x:0, y:0}, grid.width(), 3, false, &config)?;  // Border
   grid.draw_line(Point{x:10, y:10}, Point{x:70, y:20}, &config)?;  // Annotation

3. Render combined result
   renderer.render(&grid)?
```

---

## Non-Functional Requirements

### Performance

**Drawing Speed Targets** (from PRD aggressive performance targets):
- **Line drawing**: <1ms for lines up to 200 pixels length
- **Circle drawing**: <2ms for circles up to 100 pixels radius
- **Rectangle fill**: <5ms for full terminal-sized rectangles (80×24 cells)
- **Density rendering**: <10ms for full terminal-sized intensity buffer (80×24 cells)
- **Benchmark validation**: All targets measured with criterion.rs in CI pipeline

**Rationale**: Drawing primitives are often used in real-time scenarios (animations, interactive visualizations). Performance must not bottleneck frame rates (60fps = 16.6ms budget per frame).

**Measurement Approach**:
- Criterion benchmarks in `benches/primitives.rs` and `benches/density.rs`
- Measure across different sizes: small (10px), medium (50px), large (200px)
- Profile hot paths with flamegraph if targets not met

---

### Security

**No Unsafe Code**: All algorithms implemented in safe Rust (no `unsafe` blocks required).

**Input Validation**:
- All coordinate/dimension parameters validated against grid bounds
- Out-of-bounds coordinates return `PrimitivesError::OutOfBounds`, no panics
- Intensity values clamped to [0.0, 1.0] range (no arithmetic errors)
- Polygon vertex lists validated for minimum length (no indexing errors)

**Integer Overflow Protection**:
- Bresenham algorithms use checked arithmetic (`i32::checked_add`, `i32::saturating_mul`)
- No risk of overflow in coordinate calculations (max grid size bounded by terminal size ~200×50)

---

### Reliability/Availability

**Error Handling Discipline** (from architecture.md zero-panics constraint):
- All drawing functions return `Result<(), PrimitivesError>` or `Result<(), DensityError>`
- Invalid inputs (out of bounds, negative radius, empty polygon) are errors, not panics
- Graceful degradation: If single primitive fails, other operations continue

**Edge Case Handling**:
- Zero-length lines: No-op (valid but draws nothing)
- Zero-radius circles: No-op (valid but draws nothing)
- Degenerate polygons (2 vertices): Draws single line
- Intensity buffer size mismatch: Returns `DensityError::InvalidBuffer` with diagnostic message

---

### Observability

**Tracing Integration** (from architecture.md decision table line 41):
- Instrument drawing functions with `tracing::instrument`
- Log level: `DEBUG` for function entry/exit, `TRACE` for per-pixel operations
- Example: `tracing::debug!("draw_line: p0={:?}, p1={:?}, thickness={:?}", p0, p1, config.thickness)`

**Performance Telemetry**:
- Optional feature flag `perf-metrics`: Record drawing operation durations
- Emits metrics: `dotmax.draw_line.duration_ms`, `dotmax.draw_circle.duration_ms`
- Useful for profiling complex scenes with many primitives

**Error Context**:
- All error variants include diagnostic context (coordinates, dimensions, reasons)
- Example: `PrimitivesError::OutOfBounds` includes both invalid coords and grid bounds for debugging

---

## Dependencies and Integrations

### Internal Dependencies

| Dependency | Version | Purpose | Relationship |
|------------|---------|---------|--------------|
| `src/grid.rs` (Epic 2) | In-tree | BrailleGrid state manipulation via `set_dot(x, y)` | Core dependency - primitives call grid methods |
| `src/error.rs` (Epic 2) | In-tree | Error type definitions (`PrimitivesError`, `DensityError`) | Error propagation |
| `src/color.rs` (Epic 5) | In-tree (future) | Forward-compatible `Option<Color>` support in `DrawConfig` | Future integration |

### External Dependencies

**No new external crates required** for Epic 4 core functionality. Bresenham algorithms are pure Rust implementations (no math libraries needed).

**Existing Dependencies** (from Epic 1-3):
- `thiserror = "2.0"` - Error type derivation (already in Cargo.toml)
- `tracing = "0.1"` - Logging/instrumentation (already in Cargo.toml)
- `criterion = "0.7"` (dev dependency) - Benchmarking (already in Cargo.toml)

### Integration Points

**1. BrailleGrid Integration** (Epic 2):
- Primitives call `BrailleGrid::set_dot(x: usize, y: usize) -> Result<(), RenderError>`
- Validates coordinates within grid bounds
- Modifies internal `Vec<u8>` dot buffer

**2. Image Pipeline Integration** (Epic 3):
- Density rendering consumes intensity buffers (f32 arrays) from Epic 3 grayscale conversion
- Example: `ImageRenderer` outputs grayscale buffer → `DensitySet::render_density()` renders as characters
- Enables hybrid rendering: image + density overlay

**3. Color System Integration** (Epic 5, future):
- `DrawConfig` includes `Option<Color>` field (None for Epic 4, Some for Epic 5)
- When Epic 5 complete, primitives automatically support colored drawing with no API changes
- Example: `DrawConfig { thickness: Medium, color: Some(Color::rgb(255, 0, 0)) }` draws red lines

**4. Animation Integration** (Epic 6, future):
- Primitives used to draw per-frame graphics (progress bars, moving shapes)
- Performance targets ensure <1ms drawing overhead per primitive (leaves 15ms for other frame work at 60fps)

---

## Acceptance Criteria (Authoritative)

**Epic-Level Acceptance Criteria** (all stories must collectively satisfy):

1. **AC1: Bresenham Line Drawing**
   - Developers can draw lines between any two points using Bresenham algorithm
   - Line drawing handles all octants (horizontal, vertical, diagonal, steep/shallow)
   - Lines respect configurable thickness (thin, medium, thick)
   - Invalid coordinates return `PrimitivesError::OutOfBounds`, no panics
   - Line drawing performance: <1ms for 200-pixel lines (criterion validated)

2. **AC2: Bresenham Circle Drawing**
   - Developers can draw circles with specified center and radius using Bresenham midpoint algorithm
   - Circles are symmetric (8-way symmetry) and pixel-perfect
   - Circle drawing handles configurable thickness (thin = outline, thick = concentric circles)
   - Invalid radius (<=0) or out-of-bounds center returns error, no panics
   - Circle drawing performance: <2ms for 100-pixel radius circles (criterion validated)

3. **AC3: Rectangle and Polygon Drawing**
   - Developers can draw rectangles (outline or filled) with specified dimensions
   - Developers can draw polygons from vertex lists (open or closed paths)
   - Rectangles and polygons validate dimensions/vertices, return errors for invalid inputs
   - Filled rectangles use efficient scan-line fill algorithm
   - Polygon drawing supports arbitrary vertex counts (minimum 2 for open, 3 for closed)

4. **AC4: Line Thickness Support**
   - All primitives (lines, circles, rectangles) support LineThickness enum (Thin, Medium, Thick)
   - Thickness implementation draws parallel/concentric shapes (no pixel doubling)
   - Thick lines/circles maintain algorithm correctness (no gaps or artifacts)
   - DrawConfig struct provides ergonomic thickness control

5. **AC5: Character Density Rendering**
   - Developers can render intensity buffers [0.0, 1.0] using character density mapping
   - System provides 3+ predefined density sets (light, medium, heavy ASCII gradients)
   - Developers can create custom density sets with validation (1-256 characters)
   - Density rendering produces smooth visual gradients (no banding or artifacts)
   - Density rendering performance: <10ms for full terminal (80×24) intensity buffers

6. **AC6: Zero Panics Guarantee**
   - All drawing functions validate inputs and return Result types
   - Out-of-bounds coordinates, invalid dimensions, negative radii return typed errors
   - Edge cases (zero-length lines, empty polygons, size mismatches) handled gracefully
   - Comprehensive error messages include diagnostic context (coords, bounds, reasons)
   - 100% test coverage on error paths (invalid inputs verified to return errors, not panic)

7. **AC7: Integration with Existing System**
   - Primitives integrate seamlessly with Epic 2 BrailleGrid (call set_dot API)
   - Density rendering consumes intensity buffers compatible with Epic 3 image pipeline
   - Forward-compatible with Epic 5 color system (DrawConfig includes Option<Color>)
   - No breaking changes to Epic 2 or Epic 3 public APIs
   - Examples demonstrate combining primitives + images + density rendering

8. **AC8: Comprehensive Testing**
   - Unit tests achieve >80% code coverage (matching Epic 3 quality standard)
   - Integration tests demonstrate all primitives working together
   - Known-value tests verify Bresenham algorithms produce expected pixel patterns
   - Edge case tests: boundaries, zero dimensions, extreme coordinates
   - Performance benchmarks in CI pipeline validate all performance targets

9. **AC9: Production-Quality Documentation**
   - Rustdoc on all public functions explaining algorithms and usage
   - Code examples in rustdoc demonstrate common use cases
   - Algorithm explanations reference Bresenham papers/resources
   - Error types documented with recovery strategies
   - Interactive examples (`draw_shapes.rs`, `density_demo.rs`) in examples/ directory

---

## Traceability Mapping

| AC | PRD Req | Spec Section | Components | Test Idea |
|----|---------|--------------|------------|-----------|
| AC1 | FR21 | APIs - draw_line() | primitives.rs::bresenham_line | Unit tests: all octants, boundaries, thickness |
| AC2 | FR22 | APIs - draw_circle() | primitives.rs::bresenham_circle | Unit tests: symmetry, radii 1-100, thickness |
| AC3 | FR23, FR24 | APIs - draw_rectangle(), draw_polygon() | primitives.rs::rectangle, polygon | Unit tests: outline/filled, vertex validation |
| AC4 | FR26 | Data Models - LineThickness | primitives.rs::DrawConfig | Unit tests: thin/medium/thick all primitives |
| AC5 | FR28-FR31 | APIs - render_density() | density.rs::DensitySet | Unit tests: intensity mapping, smooth gradients |
| AC6 | NFR (zero panics) | Security, Reliability | error.rs, all functions | Integration tests: invalid inputs return errors |
| AC7 | FR27 (color), Epic 3 integration | Workflows - combining primitives + images | primitives.rs, density.rs, grid.rs | Integration test: image + overlay primitives |
| AC8 | NFR (quality) | Test Strategy | tests/, benches/ | CI pipeline: coverage report, benchmark gates |
| AC9 | NFR (DX) | Test Strategy | rustdoc, examples/ | Manual review: docs clarity, example completeness |

---

## Risks, Assumptions, Open Questions

### Risks

**RISK: Bresenham Algorithm Extraction from Crabmusic**
- **Severity**: MEDIUM
- **Description**: Crabmusic code may have implicit dependencies on audio timing or state that aren't obvious until extraction
- **Mitigation**:
  1. Copy exact crabmusic algorithms first (lock behavior with tests)
  2. Incrementally strip dependencies while maintaining test pass
  3. Reference crabmusic visual output as quality baseline
- **Owner**: Dev agent (Story 4.1, 4.2)

**RISK: Performance Targets May Require Optimization**
- **Severity**: MEDIUM
- **Description**: Bresenham algorithms are O(n) where n=pixels, but large shapes could miss <1ms target
- **Mitigation**:
  1. Implement naive algorithm first, measure with criterion
  2. If targets missed, profile with flamegraph to identify hotspots
  3. Optimize hot paths (e.g., batch set_dot calls, SIMD for density mapping)
  4. Epic 3 precedent: adaptive algorithms (Lanczos3 vs Triangle) - apply if needed
- **Owner**: Dev agent + architect (Story 4.1-4.4 benchmarking)

**RISK: Thick Lines May Have Visual Artifacts**
- **Severity**: LOW
- **Description**: Parallel line strategy for thickness may create gaps on steep angles
- **Mitigation**:
  1. Test thick lines across all octants with visual inspection
  2. If artifacts, use perpendicular offset algorithm (Xiaolin Wu style)
  3. Document any known limitations in rustdoc
- **Owner**: Dev agent (Story 4.1 implementation + manual testing)

### Assumptions

**ASSUMPTION: Bresenham Sufficient for Braille Resolution**
- **Rationale**: Braille grids are low resolution (40×24 cells = 80×96 dots typical terminal). Pixel-perfect Bresenham is appropriate; anti-aliasing adds complexity for negligible benefit at this resolution.
- **Validation**: Epic 3 precedent (no anti-aliasing in image pipeline), proven crabmusic quality
- **Impact if Wrong**: If users request smoother curves, defer anti-aliasing to Epic 4.5 or future (not MVP blocker)

**ASSUMPTION: No External Math Libraries Needed**
- **Rationale**: Bresenham uses only integer arithmetic (addition, subtraction, bit shifts). No floating point or trigonometry.
- **Validation**: Review crabmusic implementation (confirmed integer-only)
- **Impact if Wrong**: If circle algorithm needs sqrt or trig, add `libm` crate (minimal dependency)

**ASSUMPTION: Density Rendering Complements (Not Replaces) Braille**
- **Rationale**: Density rendering is for ASCII-art style effects (gradients, heatmaps). Primary rendering remains braille dots for maximum resolution.
- **Validation**: Crabmusic uses density for specific visualizations, braille for main graphics
- **Impact if Wrong**: If users prefer density-only mode, expose as rendering option (API extension, not redesign)

### Open Questions

**QUESTION: Should Line Thickness Use Brush Patterns or Parallel Lines?**
- **Context**: Medium/thick lines can be implemented via (1) parallel lines offset perpendicular, or (2) brush patterns (e.g., thick = draw 3×3 square at each point)
- **Decision Needed By**: Story 4.1 (line drawing implementation)
- **Recommendation**: Start with parallel lines (simpler, proven in graphics libs). If visual quality issues, experiment with brush patterns in Story 4.1 testing.

**QUESTION: Should Density Sets Support Unicode Block Characters?**
- **Context**: ASCII density uses 7-bit chars (` .:-=+*#`). Unicode has block characters (`░▒▓█`) for smoother gradients.
- **Decision Needed By**: Story 4.4 (density implementation)
- **Recommendation**: Support both. Predefined sets include ASCII (universal compatibility) and Unicode blocks (better quality on modern terminals). Let users choose.

**QUESTION: Forward Compatibility with Epic 5 Color System?**
- **Context**: DrawConfig includes `Option<Color>` for Epic 5. Should Epic 4 validate/ignore color parameter?
- **Decision Needed By**: Story 4.1 (DrawConfig design)
- **Recommendation**: Accept `Option<Color>` but ignore if Some (log warning with tracing). Epic 5 implements color application. This ensures API stability (no breaking changes when Epic 5 merges).

---

## Test Strategy Summary

### Test Levels

**1. Unit Tests** (`tests/primitives_tests.rs`, `tests/density_tests.rs`)
- Target: >80% code coverage (Epic 3 quality standard)
- Focus: Individual functions, algorithm correctness, error handling
- Examples:
  - Bresenham line: Test all octants (horizontal, vertical, 8 diagonals)
  - Bresenham circle: Test symmetry (8-way), radii 1-100
  - Density mapping: Known intensity values → expected characters
  - Error paths: Out of bounds, negative dimensions, empty polygons

**2. Integration Tests** (`tests/integration_drawing_tests.rs`)
- Focus: Combining primitives, primitives + images, density + primitives
- Examples:
  - Draw multiple shapes on same grid (no overlap conflicts)
  - Render image, overlay primitives (Story 3.8 + Epic 4 integration)
  - Density render intensity buffer from Epic 3 grayscale conversion

**3. Performance Tests** (`benches/primitives.rs`, `benches/density.rs`)
- Criterion benchmarks for all primitives across sizes (small/medium/large)
- CI regression detection: Fail if performance >10% slower than baseline
- Flamegraph profiling for hot paths if targets missed

**4. Visual Validation Tests** (`examples/draw_shapes.rs`, `examples/density_demo.rs`)
- Interactive examples for manual visual inspection
- Compare output quality to crabmusic baseline (screenshot comparisons)
- Test thick lines for artifacts, density gradients for smoothness

### Edge Cases to Test

- **Boundary conditions**: Lines starting/ending at grid edges, circles centered at corners
- **Degenerate inputs**: Zero-length lines, zero-radius circles, 2-vertex polygons
- **Extreme coordinates**: Negative coords, coords far outside grid bounds
- **Size mismatches**: Intensity buffer size != grid size (density rendering)
- **Empty inputs**: Empty polygon vertex list, empty density character set

### Test Coverage Goals

| Category | Target | Measurement |
|----------|--------|-------------|
| Unit test coverage | >80% | cargo tarpaulin in CI |
| Integration scenarios | 100% (all AC combinations) | Manual checklist |
| Performance benchmarks | 100% (all primitives) | Criterion CI gate |
| Error paths | 100% | Explicit invalid input tests |

### Acceptance Testing

**Manual Testing Protocol** (from Epic 3.5 learnings):
- Story 4.5 (or 4.6): Manual testing and validation story
- Test across terminal types (xterm, Konsole, Windows Terminal, iTerm2)
- Visual quality assessment: Compare to crabmusic, check for artifacts
- Performance feel: Drawing should feel instant in interactive examples
- Document findings and create Epic 4.5 polish sprint if needed (Epic 3 → 3.5 precedent)

---

**Tech Spec Completed By**: SM Agent (Bob)
**Date**: 2025-11-21
**Next Step**: Draft Story 4.1 - Implement Bresenham Line Drawing Algorithm
