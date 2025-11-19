# Epic Technical Specification: 2D Image Rendering Pipeline

Date: 2025-11-19
Author: Frosty
Epic ID: 3
Status: Draft

---

## Overview

Epic 3 implements the core image-to-braille rendering pipeline, enabling developers to load and render standard image formats (PNG, JPG, GIF, BMP, WebP, TIFF, SVG) to terminal braille output. This epic delivers the primary use case for dotmax: converting arbitrary images to beautiful terminal graphics with professional quality.

The implementation builds a staged processing pipeline (Load → Resize → Grayscale → Dither → Threshold → Map to Braille) where each stage is independently testable and optimizable. By leveraging Rust's `image` and `imageproc` crates with feature-gated dependencies, we keep the core library lightweight while enabling rich media capabilities for users who need them.

## Objectives and Scope

**In Scope:**
- Image loading from file paths and byte buffers (PNG, JPG, GIF, BMP, WebP, TIFF)
- SVG vector graphics loading with rasterization support
- Automatic and manual image resizing with aspect ratio preservation
- Grayscale conversion from color images
- Otsu thresholding for optimal binary conversion
- Three dithering algorithms: Floyd-Steinberg, Bayer, Atkinson
- Pixel-to-braille dot mapping (2×4 blocks → braille cells)
- Binary image-to-BrailleGrid conversion
- Monochrome and color rendering modes
- High-level `ImageRenderer` API for <100 line integration
- Performance target: <50ms for standard terminals (80×24), <100ms for large terminals (200×50)

**Out of Scope:**
- Video playback (deferred to Phase 2A post-1.0)
- Animation support beyond static frames (Epic 6)
- 3D raytracing (Phase 3)
- Color scheme application (Epic 5)
- Advanced effects pipeline (remains in crabmusic)

## System Architecture Alignment

This epic directly implements **Architecture Pattern 2: Image-to-Braille Conversion Pipeline** from the architecture document. The implementation follows the feature-based module structure defined in ADR 0003 (Feature Flag Architecture), placing all image rendering code in `src/image/*` behind the `image` and `svg` feature flags.

**Architecture Components Used:**
- `src/grid.rs` (Epic 2): `BrailleGrid` as the target output structure
- `src/error.rs` (Epic 2): `DotmaxError` enum with image-specific variants
- `src/image/` (NEW): Complete image processing pipeline module

**Dependencies Activated:**
- `image = "0.25"` - Industry standard raster image loader (PNG, JPG, GIF, BMP, WebP, TIFF)
- `imageproc = "0.24"` - Image processing algorithms (thresholding, filtering)
- `resvg = "0.38"` - SVG rendering engine (rasterization to pixels)
- `usvg = "0.38"` - SVG parsing library (used by resvg)

**Feature Gate Configuration:**
```toml
[features]
image = ["dep:image", "dep:imageproc"]
svg = ["dep:resvg", "dep:usvg"]
```

**Cross-Epic Dependencies:**
- **Epic 2 (Core Rendering)**: Requires `BrailleGrid::set_dot()`, `BrailleGrid::new()`, and error types
- **Epic 5 (Color System)**: Color mode rendering will integrate with ColorScheme (deferred integration)
- **Epic 7 (Performance)**: Pipeline stages must hit <50ms total rendering target

## Detailed Design

### Services and Modules

| Module | Responsibility | Inputs | Outputs | Owner |
|--------|---------------|--------|---------|-------|
| `src/image/mod.rs` | Public API surface, re-exports | Feature gate, module coordination | Public types (`ImageRenderer`, `DitherMethod`) | Story 3.8 |
| `src/image/loader.rs` | Load images from files/memory | File paths, byte buffers | `DynamicImage` (from `image` crate) | Story 3.1 |
| `src/image/resize.rs` | Resize images with aspect ratio | `DynamicImage`, target dimensions | Resized `DynamicImage` | Story 3.2 |
| `src/image/convert.rs` | Color to grayscale conversion | `DynamicImage` | `GrayImage` | Story 3.3 |
| `src/image/threshold.rs` | Otsu thresholding, brightness/contrast | `GrayImage` | `BinaryImage`, threshold value | Story 3.3 |
| `src/image/dither.rs` | Floyd-Steinberg, Bayer, Atkinson | `GrayImage`, `DitheringMethod` | `BinaryImage` | Story 3.4 |
| `src/image/mapper.rs` | Pixels → braille dots (2×4 blocks) | `BinaryImage` | `BrailleGrid` | Story 3.5 |
| `src/image/svg.rs` | SVG loading and rasterization | SVG path/bytes, dimensions | `DynamicImage` | Story 3.6 |
| `src/image/color_mode.rs` | Color-aware rendering | `DynamicImage`, color mapping | `BrailleGrid` with colors | Story 3.7 |

**Module Dependencies:**
```
src/image/mod.rs
  ├── loader.rs → (image crate)
  ├── resize.rs → loader.rs, (image crate)
  ├── convert.rs → (image crate)
  ├── threshold.rs → convert.rs, (imageproc crate)
  ├── dither.rs → convert.rs
  ├── mapper.rs → dither.rs, threshold.rs, src/grid.rs
  ├── svg.rs → (resvg, usvg crates)
  └── color_mode.rs → mapper.rs, src/color.rs (Epic 5)
```

### Data Models and Contracts

**Core Data Types:**

```rust
// src/image/mod.rs

/// Binary image representation (black/white pixels)
pub struct BinaryImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<bool>,  // true = black, false = white
}

/// Dithering algorithm selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DitheringMethod {
    None,              // Direct threshold (no dithering)
    FloydSteinberg,    // Error diffusion (best quality, slower)
    Bayer,             // Ordered dithering (fast, good for gradients)
    Atkinson,          // Error diffusion (Apple-style, softer)
}

/// Thresholding algorithm selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdAlgorithm {
    Otsu,              // Automatic optimal threshold
    Manual(u8),        // User-specified threshold value
}

/// Color rendering mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Monochrome,        // Black/white only
    Grayscale,         // 256 shades using color intensity
    TrueColor,         // Full RGB color per braille cell
}

/// High-level image renderer with builder pattern
pub struct ImageRenderer {
    dither_method: DitheringMethod,
    threshold_algo: ThresholdAlgorithm,
    color_mode: ColorMode,
    preserve_aspect: bool,
    brightness: f32,   // 0.0 to 2.0, default 1.0
    contrast: f32,     // 0.0 to 2.0, default 1.0
    gamma: f32,        // 0.1 to 3.0, default 1.0
}
```

**External Types (from dependencies):**
- `DynamicImage` (from `image` crate): Multi-format image container
- `GrayImage` (from `image` crate): 8-bit grayscale image
- `RgbImage` (from `image` crate): 24-bit RGB image

**Relationships:**
```
DynamicImage (color/grayscale)
    ↓ (resize.rs)
DynamicImage (terminal-sized)
    ↓ (convert.rs)
GrayImage
    ↓ (dither.rs OR threshold.rs)
BinaryImage
    ↓ (mapper.rs)
BrailleGrid (2×4 dot mapping)
```

### APIs and Interfaces

**High-Level API (Builder Pattern):**

```rust
// User-facing API - Simple integration
use dotmax::image::{ImageRenderer, DitheringMethod};

let renderer = ImageRenderer::builder()
    .dithering(DitheringMethod::FloydSteinberg)
    .preserve_aspect_ratio(true)
    .brightness(1.2)
    .build();

let grid = renderer.render_from_path("image.png", 80, 24)?;
terminal.render(&grid)?;
```

**Low-Level Module APIs:**

```rust
// src/image/loader.rs
pub fn load_from_path(path: &Path) -> Result<DynamicImage, DotmaxError>;
pub fn load_from_bytes(bytes: &[u8]) -> Result<DynamicImage, DotmaxError>;
pub fn supported_formats() -> Vec<&'static str>;

// src/image/resize.rs
pub fn resize_to_terminal(
    image: &DynamicImage,
    term_width: u16,
    term_height: u16
) -> DynamicImage;

pub fn resize_to_dimensions(
    image: &DynamicImage,
    target_width: u32,
    target_height: u32,
    preserve_aspect: bool
) -> DynamicImage;

// src/image/threshold.rs
pub fn to_grayscale(image: &DynamicImage) -> GrayImage;
pub fn otsu_threshold(gray: &GrayImage) -> u8;
pub fn apply_threshold(gray: &GrayImage, threshold: u8) -> BinaryImage;
pub fn auto_threshold(image: &DynamicImage) -> BinaryImage;
pub fn adjust_brightness(gray: &GrayImage, factor: f32) -> GrayImage;
pub fn adjust_contrast(gray: &GrayImage, factor: f32) -> GrayImage;
pub fn adjust_gamma(gray: &GrayImage, gamma: f32) -> GrayImage;

// src/image/dither.rs
pub fn apply_dithering(gray: &GrayImage, method: DitheringMethod) -> BinaryImage;

// src/image/mapper.rs
pub fn pixels_to_braille(
    binary: &BinaryImage,
    cell_width: usize,
    cell_height: usize
) -> Result<BrailleGrid, DotmaxError>;

// src/image/svg.rs (feature-gated)
#[cfg(feature = "svg")]
pub fn load_svg_from_path(
    path: &Path,
    width: u32,
    height: u32
) -> Result<DynamicImage, DotmaxError>;

#[cfg(feature = "svg")]
pub fn load_svg_from_bytes(
    bytes: &[u8],
    width: u32,
    height: u32
) -> Result<DynamicImage, DotmaxError>;
```

**Error Additions to `src/error.rs`:**

```rust
#[derive(Error, Debug)]
pub enum DotmaxError {
    // ... existing variants ...

    #[error("Failed to load image from {path}: {source}")]
    ImageLoad {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },

    #[error("Unsupported image format: {format}")]
    UnsupportedFormat { format: String },

    #[error("Invalid image dimensions: {width}×{height}")]
    InvalidImageDimensions { width: u32, height: u32 },

    #[error("SVG rendering error: {0}")]
    SvgError(String),
}
```

### Workflows and Sequencing

**Image-to-Braille Pipeline (Monochrome Mode):**

```
1. LOAD IMAGE
   ├─ User provides: file path or byte buffer
   ├─ loader.rs: Decode using `image` crate
   └─ Output: DynamicImage (any format → unified type)

2. RESIZE
   ├─ Input: DynamicImage + terminal dimensions (e.g., 80×24 cells)
   ├─ resize.rs: Calculate pixel dimensions (80×2=160 wide, 24×4=96 tall)
   ├─ Apply aspect ratio preservation (letterbox if needed)
   ├─ Use Lanczos3 filter for quality
   └─ Output: DynamicImage (terminal-sized)

3. CONVERT TO GRAYSCALE
   ├─ Input: DynamicImage (may be color or grayscale)
   ├─ convert.rs: RGB → luminance conversion if needed
   └─ Output: GrayImage (8-bit per pixel)

4. ADJUST IMAGE (Optional)
   ├─ Input: GrayImage
   ├─ threshold.rs: Apply brightness/contrast/gamma adjustments
   └─ Output: GrayImage (adjusted)

5. DITHERING
   ├─ Input: GrayImage + DitheringMethod selection
   ├─ dither.rs: Apply Floyd-Steinberg/Bayer/Atkinson/None
   └─ Output: BinaryImage (boolean pixels, true=black)

6. THRESHOLD (if no dithering)
   ├─ Input: GrayImage
   ├─ threshold.rs: Calculate Otsu threshold, apply binary conversion
   └─ Output: BinaryImage

7. MAP TO BRAILLE
   ├─ Input: BinaryImage
   ├─ mapper.rs: Group pixels into 2×4 blocks
   ├─ For each block: convert pixel pattern to braille dot pattern
   ├─ Call BrailleGrid::set_dot() for each dot position
   └─ Output: BrailleGrid (ready to render)

8. RENDER TO TERMINAL
   ├─ Input: BrailleGrid
   ├─ TerminalRenderer: Convert dots to Unicode braille
   └─ Output: Terminal display
```

**SVG-to-Braille Pipeline:**

```
1. LOAD SVG
   ├─ User provides: SVG file path or byte buffer + target dimensions
   ├─ svg.rs: Parse with usvg, rasterize with resvg
   └─ Output: DynamicImage (rasterized bitmap)

2-8. FOLLOW STANDARD PIPELINE
   └─ Same as steps 2-8 above (resize, convert, dither, map, render)
```

**Color Mode Pipeline (Epic 3 + Epic 5 integration):**

```
1-2. LOAD + RESIZE (same as monochrome)

3. PRESERVE COLOR INFORMATION
   ├─ Input: DynamicImage (color)
   ├─ color_mode.rs: Extract RGB values per pixel
   └─ Output: RgbImage

4. MAP COLOR TO BRAILLE WITH INTENSITY
   ├─ Input: RgbImage
   ├─ mapper.rs: Convert RGB to grayscale for dot on/off decision
   ├─ mapper.rs: Store RGB color per braille cell
   ├─ Call BrailleGrid::set_dot() + set_color()
   └─ Output: BrailleGrid with color data

5. RENDER TO TERMINAL
   ├─ Input: BrailleGrid with colors
   ├─ TerminalRenderer: Convert to Unicode + ANSI color codes
   └─ Output: Colored terminal display
```

## Non-Functional Requirements

### Performance

**Critical Performance Targets (from PRD NFR-P1):**

- **Image-to-braille conversion**: <25ms target, <50ms maximum for standard terminals (80×24)
- **Large terminal rendering** (200×50): <100ms maximum
- **SVG rasterization**: <100ms for typical graphics

**Per-Stage Budget Allocation (must sum to <50ms):**

| Stage | Target Time | Justification |
|-------|-------------|---------------|
| Load image | <5ms | Disk I/O, minimal processing (cached after first load) |
| Resize | <10ms | Lanczos3 filter is expensive but necessary for quality |
| Grayscale conversion | <2ms | Simple RGB → luminance math |
| Brightness/contrast/gamma | <3ms | Optional adjustments, pixel-wise operations |
| Dithering (Floyd-Steinberg) | <15ms | Error diffusion is most expensive stage, optimize here |
| Thresholding (Otsu) | <5ms | Histogram calculation + single pass |
| Braille mapping | <10ms | 2×4 block iteration, BrailleGrid::set_dot() calls |
| **Total pipeline** | **<50ms** | **Aggressive target, measure each stage with criterion** |

**Memory Efficiency:**

- **Per-frame overhead**: <500KB for typical renders (NFR-P3)
- Image pipeline stages reuse buffers where possible
- DynamicImage → GrayImage → BinaryImage uses progressive conversion (no duplicate large buffers)
- BrailleGrid allocation is final output, sized to terminal dimensions

**Performance Validation Strategy:**

- Benchmark each pipeline stage separately with criterion.rs
- Create performance regression tests in CI (fail if >10% slower)
- Profile with flamegraph to identify hotspots before optimization
- Test with real-world images (photos, diagrams, artwork) of varying sizes

### Security

**Input Validation (NFR-S2):**

- **File path validation**: Check file exists, readable, size limits before loading
- **Maximum image dimensions**: Enforce limits to prevent OOM attacks (e.g., max 10,000×10,000 pixels)
- **Format validation**: Rely on `image` crate's robust format detection and error handling
- **Malformed file handling**: All decode errors caught and converted to `DotmaxError::ImageLoad`

**Memory Safety (NFR-S1):**

- **Zero unsafe code** in image pipeline (rely on safe Rust + `image` crate)
- **No buffer overflows**: Rust prevents out-of-bounds access in pixel iteration
- **Resource limits**: Prevent unbounded memory allocation from adversarial inputs

**Dependency Security (NFR-S3):**

- `image` crate: Well-audited, widely used (100M+ downloads)
- `imageproc` crate: Maintained by image-rs organization
- `resvg`/`usvg`: SVG rendering is complex, monitor for security advisories
- cargo-audit in CI detects known vulnerabilities

### Reliability/Availability

**Error Handling (NFR-R1):**

- **Zero panics**: All image operations return `Result<T, DotmaxError>`
- **Graceful degradation**: Unsupported formats return clear error messages, not crashes
- **Edge case handling**: Zero-size images, extreme dimensions, corrupted files all handled safely

**Cross-Platform Consistency (NFR-R2):**

- `image` crate provides platform-agnostic image loading
- Same visual output on Windows, Linux, macOS
- CI tests verify consistency across all platforms

**Robustness (NFR-R3):**

- Handle malformed images (corrupted PNG headers, truncated JPEGs)
- Handle edge cases: 1×1 pixel images, extremely wide/tall images
- Validate all inputs before processing (no assumptions about file correctness)

### Observability

**Logging Strategy (NFR-R4, using `tracing` crate):**

```rust
use tracing::{debug, info, instrument};

#[instrument]
pub fn render_from_path(path: &Path) -> Result<BrailleGrid, DotmaxError> {
    info!("Loading image from {:?}", path);
    let img = load_from_path(path)?;
    debug!("Image dimensions: {}×{}", img.width(), img.height());

    // ... pipeline stages ...

    info!("Image rendered successfully to {}×{} braille grid", width, height);
    Ok(grid)
}
```

**Log Levels:**
- `error!`: Failed operations (file not found, decode errors)
- `warn!`: Suboptimal conditions (image downscaled significantly, unusual aspect ratios)
- `info!`: Major operations (image loaded, pipeline stage completed)
- `debug!`: Detailed flow (Otsu threshold value, dithering method applied, resize calculations)
- `trace!`: Hot path internals (only if TRACE enabled, pixel iteration details)

**Metrics for Observability:**
- Image load time
- Resize time
- Dithering time
- Total pipeline duration
- Input image dimensions
- Output grid dimensions
- Format type (PNG, JPG, SVG, etc.)

## Dependencies and Integrations

**Direct Dependencies (from Cargo.toml):**

| Dependency | Version | Purpose | License | Feature Gate |
|------------|---------|---------|---------|--------------|
| `image` | 0.25 | Load PNG, JPG, GIF, BMP, WebP, TIFF | MIT/Apache-2.0 | `image` |
| `imageproc` | 0.24 | Image processing algorithms | MIT | `image` |
| `resvg` | 0.38 | SVG rendering engine | MPL-2.0 | `svg` |
| `usvg` | 0.38 | SVG parsing library | MPL-2.0 | `svg` |

**Transitive Dependencies (notable):**
- `png`, `jpeg-decoder`, `gif`, `webp`: Format-specific decoders (pulled by `image` crate)
- `tiny-skia`: 2D rendering backend for resvg
- `fontdb`: Font handling for SVG text (if SVG contains text)

**Integration Points:**

1. **Epic 2 (Core Rendering)**:
   - `BrailleGrid::new(width, height)` - Create output grid
   - `BrailleGrid::set_dot(x, y, value)` - Set individual dots from pixel mapping
   - `BrailleGrid::set_color(cell_x, cell_y, color)` - For color mode (Epic 5 integration)

2. **Epic 5 (Color System)**:
   - `Color::rgb(r, g, b)` - Store per-cell colors in color mode
   - `ColorScheme::apply()` - Apply color schemes to intensity buffers (future)

3. **Epic 7 (Performance)**:
   - Benchmark suite measures each pipeline stage
   - Performance regression tests ensure <50ms target maintained

4. **External Systems**:
   - **Filesystem**: Image loading via standard library `std::fs`
   - **Terminal**: Output via `TerminalRenderer` (Epic 2, uses ratatui/crossterm)

**Dependency Justifications (from Architecture docs/dependencies.md):**

- **image**: De facto standard Rust image library, 100M+ downloads, well-maintained
- **imageproc**: Companion library for image processing, same maintainers as `image`
- **resvg**: Best-in-class SVG rendering for Rust, high-quality rasterization
- **usvg**: Required by resvg for SVG parsing, minimal overhead

## Acceptance Criteria (Authoritative)

**Epic-Level Acceptance Criteria:**

1. ✅ **Image Loading Works**: Developers can load PNG, JPG, GIF, BMP, WebP, TIFF from file paths and byte buffers
2. ✅ **SVG Rendering Works**: Developers can load and rasterize SVG files to braille output
3. ✅ **Automatic Resizing Works**: Images automatically resize to terminal dimensions with aspect ratio preservation
4. ✅ **Manual Resizing Works**: Developers can specify custom dimensions with/without aspect ratio preservation
5. ✅ **Grayscale Conversion Works**: Color images convert to grayscale for binary thresholding
6. ✅ **Otsu Thresholding Works**: System calculates optimal threshold automatically for binary conversion
7. ✅ **Brightness/Contrast/Gamma Adjustments Work**: Developers can adjust image properties before rendering
8. ✅ **Three Dithering Methods Work**: Floyd-Steinberg, Bayer, Atkinson algorithms produce quality output
9. ✅ **Braille Mapping Works**: Binary images map to BrailleGrid with correct 2×4 dot patterns
10. ✅ **Monochrome Mode Works**: Black/white braille rendering displays correctly in terminal
11. ✅ **Color Mode Works**: RGB color preserved per braille cell, renders with ANSI codes
12. ✅ **High-Level API Works**: `ImageRenderer` builder pattern enables <100 line integration
13. ✅ **Performance Target Met**: Standard terminal renders in <50ms, large terminals in <100ms
14. ✅ **Error Handling Works**: Malformed files, missing files, unsupported formats return clear errors (no panics)
15. ✅ **Cross-Platform Consistency**: Same visual output on Windows, Linux, macOS
16. ✅ **Feature Gates Work**: `image` and `svg` features compile independently, core stays lightweight
17. ✅ **Examples Work**: All example programs compile and demonstrate key features
18. ✅ **Tests Pass**: Unit tests cover all modules, integration tests verify end-to-end pipeline

**Story-Specific Acceptance Criteria (from epics.md):**

- **Story 3.1**: Image loading from paths/bytes, supported formats list, error handling for missing/corrupted files
- **Story 3.2**: Resize to terminal dimensions, manual resize with aspect ratio control, edge cases handled
- **Story 3.3**: Grayscale conversion, Otsu threshold calculation, binary image output, brightness/contrast/gamma adjustments
- **Story 3.4**: Floyd-Steinberg, Bayer, Atkinson dithering implementations, quality comparison tests
- **Story 3.5**: 2×4 pixel block → braille cell mapping, correct Unicode braille output
- **Story 3.6**: SVG loading from path/bytes, rasterization to specified dimensions, integration with image pipeline
- **Story 3.7**: Color preservation per cell, RGB → intensity mapping, ANSI color code output
- **Story 3.8**: High-level `ImageRenderer` API, builder pattern, <100 line integration examples

## Traceability Mapping

| Acceptance Criteria | PRD Functional Requirements | Spec Section | Components/APIs | Test Strategy |
|---------------------|----------------------------|--------------|-----------------|---------------|
| AC1: Image Loading | FR9, FR10 | APIs/loader.rs | `load_from_path()`, `load_from_bytes()` | Unit tests with valid/invalid files, integration test with all formats |
| AC2: SVG Rendering | FR11 | APIs/svg.rs | `load_svg_from_path()`, resvg integration | Unit test SVG→bitmap, integration test SVG→braille |
| AC3: Auto Resize | FR12 | APIs/resize.rs | `resize_to_terminal()` | Unit test aspect ratio math, integration test with various images |
| AC4: Manual Resize | FR13 | APIs/resize.rs | `resize_to_dimensions()` | Unit test preserve vs stretch, edge case tests |
| AC5: Grayscale Conversion | FR14 | APIs/threshold.rs | `to_grayscale()` | Unit test RGB→luminance conversion formula |
| AC6: Otsu Thresholding | FR18 | APIs/threshold.rs | `otsu_threshold()`, `apply_threshold()` | Unit test against known images with expected thresholds |
| AC7: Image Adjustments | FR19 | APIs/threshold.rs | `adjust_brightness()`, `adjust_contrast()`, `adjust_gamma()` | Unit test pixel value transformations |
| AC8: Dithering Methods | FR15 | APIs/dither.rs | `apply_dithering()`, `DitheringMethod` enum | Visual regression tests (save output, compare), quality tests |
| AC9: Braille Mapping | FR14 (binary conversion) | APIs/mapper.rs | `pixels_to_braille()` | Unit test 2×4 block→dot pattern conversion, edge cases |
| AC10: Monochrome Mode | FR16 | APIs/ImageRenderer | Builder with `ColorMode::Monochrome` | Integration test: image→grid→terminal output |
| AC11: Color Mode | FR17 | APIs/color_mode.rs | `ColorMode::TrueColor`, `set_color()` | Integration test with color terminal output |
| AC12: High-Level API | FR44, FR46 | APIs/ImageRenderer | Builder pattern, `render_from_path()` | Example programs verify <100 line integration |
| AC13: Performance Target | FR68, FR69, NFR-P1 | Performance/Budget Allocation | All pipeline stages | Criterion benchmarks for each stage, total <50ms |
| AC14: Error Handling | FR20, FR56-59, NFR-R1 | APIs/Error Types | `DotmaxError` variants | Unit tests for all error paths (missing file, corrupt, etc.) |
| AC15: Cross-Platform | FR75-77, NFR-R2 | N/A (integration) | All modules | CI tests on Windows, Linux, macOS |
| AC16: Feature Gates | FR49, FR64, NFR-D3 | Architecture/Feature Gates | Cargo.toml features | CI compiles with/without features separately |
| AC17: Examples Work | FR48, FR67 | N/A (examples) | examples/*.rs | CI compiles and runs all examples |
| AC18: Tests Pass | FR81-82, NFR-M4 | N/A (testing) | All modules | CI runs `cargo test --all-features` |

## Risks, Assumptions, Open Questions

**Risks:**

1. **RISK: Performance target (<50ms) may be difficult to achieve**
   - *Mitigation*: Profile each stage separately, optimize dithering (most expensive), consider SIMD for hot paths
   - *Fallback*: Relax to <100ms if necessary, but document as known issue for future optimization

2. **RISK: SVG rendering quality may vary with complex SVGs**
   - *Mitigation*: Test with diverse SVG samples (text, gradients, paths), document limitations
   - *Fallback*: Recommend rasterizing complex SVGs externally if quality issues arise

3. **RISK: Color mode may increase memory usage beyond <500KB target**
   - *Mitigation*: Profile memory with color grids, optimize color storage (u32 vs struct)
   - *Fallback*: Document increased memory usage for color mode, keep monochrome mode optimized

4. **RISK: Dithering algorithms may be slower than expected**
   - *Mitigation*: Benchmark Floyd-Steinberg vs Bayer early, consider making Floyd-Steinberg opt-in "high quality" mode
   - *Fallback*: Default to faster Bayer dithering, offer Floyd-Steinberg as explicit choice

**Assumptions:**

1. **ASSUMPTION: `image` crate handles all format edge cases correctly**
   - *Validation*: Rely on `image` crate's extensive test suite and community usage (100M+ downloads)

2. **ASSUMPTION: Aspect ratio preservation is always desired (default behavior)**
   - *Validation*: Provide `preserve_aspect: false` option for users who want exact dimensions

3. **ASSUMPTION: Otsu thresholding produces acceptable results for most images**
   - *Validation*: Test with diverse image types (photos, diagrams, line art), offer manual threshold override

4. **ASSUMPTION: Terminal dimensions won't change during rendering**
   - *Validation*: Epic 2 (Story 2.5) handles resize events, Epic 3 renders to static dimensions provided

5. **ASSUMPTION: 2×4 braille mapping is sufficient resolution for image quality**
   - *Validation*: Compare output quality to crabmusic baseline (proven acceptable)

**Open Questions:**

1. **QUESTION: Should we support image caching to avoid re-loading?**
   - *Answer*: Not in MVP. Users can cache `DynamicImage` themselves if needed. Defer to post-1.0.

2. **QUESTION: Should dithering be applied before or after resize?**
   - *Answer*: After resize. Dithering on high-res images wastes computation, dither on terminal-sized image for optimal quality.

3. **QUESTION: Should we extract dithering code from crabmusic or implement from scratch?**
   - *Answer*: Check crabmusic for working dithering implementation. If quality is proven, extract. Otherwise implement from algorithm specs.

4. **QUESTION: How do we handle SVGs with text when fonts aren't available?**
   - *Answer*: `resvg` handles font fallbacks. Document requirement for system fonts, test with text-heavy SVGs.

5. **QUESTION: Should color mode be part of Epic 3 or deferred to Epic 5?**
   - *Answer*: Basic color mode (RGB per cell) in Epic 3 (Story 3.7). ColorScheme application deferred to Epic 5.

## Test Strategy Summary

**Test Levels:**

1. **Unit Tests** (in-module `#[cfg(test)]` blocks):
   - Test individual functions in isolation
   - `loader.rs`: Load valid/invalid images, handle errors
   - `resize.rs`: Aspect ratio math, edge cases (0-size, extreme ratios)
   - `threshold.rs`: Otsu calculation on known images, brightness/contrast formulas
   - `dither.rs`: Dithering algorithms on test patterns
   - `mapper.rs`: 2×4 block→braille dot conversion correctness
   - `svg.rs`: SVG parsing and rasterization
   - **Coverage Target**: >80% line coverage for image pipeline modules

2. **Integration Tests** (`tests/image_rendering_tests.rs`):
   - Test complete pipeline: load → resize → dither → map → render
   - Test all image formats (PNG, JPG, GIF, BMP, WebP, TIFF, SVG)
   - Test all dithering methods produce valid output
   - Test error handling end-to-end (missing files, corrupted images)
   - Test feature gates (`cargo test --features image`, `cargo test --features svg`)
   - **Coverage Target**: All user-facing APIs exercised

3. **Visual Regression Tests** (`tests/visual_regression/`):
   - Render test images to BrailleGrid, serialize to string
   - Compare output against baseline snapshots (golden files)
   - Detect visual regressions from code changes
   - Test images: photos, diagrams, line art, gradients
   - **Tooling**: Save grid output as text, use `insta` crate for snapshot testing

4. **Performance Tests** (`benches/image_conversion.rs`):
   - Benchmark each pipeline stage separately with criterion.rs
   - Benchmark full pipeline (load→render) for various image sizes
   - Track performance over time, fail CI if >10% regression
   - Test images: small (100×100), medium (800×600), large (1920×1080)
   - **Targets**: <50ms for 80×24 terminal, <100ms for 200×50 terminal

5. **Cross-Platform Tests** (GitHub Actions CI):
   - Run all tests on Windows, Linux, macOS
   - Verify consistent behavior across platforms
   - Test with stable Rust and MSRV (1.70)

**Test Fixtures:**

```
tests/fixtures/
  ├── images/
  │   ├── sample.png           # Standard test image
  │   ├── test_photo.jpg       # Photographic content
  │   ├── test_diagram.png     # Line art / diagram
  │   ├── test_gradient.png    # Gradient for dithering tests
  │   ├── test_svg.svg         # SVG test case
  │   ├── corrupted.png        # Invalid file for error testing
  │   └── extreme_wide.png     # Edge case: very wide image
  └── baselines/
      ├── sample_fs.txt        # Floyd-Steinberg baseline output
      ├── sample_bayer.txt     # Bayer dithering baseline
      └── sample_atkinson.txt  # Atkinson dithering baseline
```

**Test Execution:**

```bash
# Run all tests
cargo test --all-features

# Run only unit tests
cargo test --lib --features image,svg

# Run integration tests
cargo test --test image_rendering_tests --features image,svg

# Run benchmarks
cargo bench --features image,svg

# Run with coverage
cargo tarpaulin --all-features --out Html
```

**Continuous Integration:**

- **On every PR**: Run full test suite on all platforms
- **On merge to main**: Run tests + benchmarks, track performance
- **Before release**: Run visual regression tests, validate examples compile
