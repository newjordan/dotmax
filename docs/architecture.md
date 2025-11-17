# dotmax - Architecture Document

## Executive Summary

Dotmax is a high-performance Rust library crate that provides terminal braille rendering capabilities. The architecture follows a **brownfield extraction + greenfield packaging** strategy, extracting proven braille rendering code (~2,000-3,000 lines) from the crabmusic project and packaging it as a professional, production-ready crate for crates.io distribution.

The architecture prioritizes **performance above all else** (<25ms image rendering, 60-120fps animation), **minimal dependencies** (<10 core), **zero panics**, and **solo long-term maintainability**. The design uses feature flags to keep the core lightweight while enabling opt-in capabilities (image, SVG, future video/3D).

**Core Innovation:** 2×4 braille dot matrix rendering via Unicode (U+2800-U+28FF) provides 4× the resolution of ASCII art while maintaining universal terminal compatibility.

## Project Initialization

Dotmax uses standard Rust tooling with no special starter template:

```bash
# Initialize project (Story 1.1)
cargo new --lib dotmax
cd dotmax

# Standard build
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Build with features
cargo build --features image,svg
```

**Rationale:** Rust library crates are simpler than applications. Standard `cargo new --lib` provides the foundation. Epic 1 stories define the optimal structure, quality tooling, and CI/CD pipeline—better than a generic template because it's customized for dotmax's specific requirements.

## Decision Summary

| Category | Decision | Version | Affects Epics | Rationale |
| -------- | -------- | ------- | ------------- | --------- |
| **Error Handling** | thiserror | 2.0.17 | All | Library users need typed errors for pattern matching (ImageLoadError vs RenderError). thiserror provides minimal boilerplate. |
| **Terminal I/O** | ratatui + crossterm | 0.29.0 + 0.29.0 | Epic 2, 6 | Industry standard for Rust TUIs. Abstracted via TerminalBackend trait to reduce lock-in. |
| **Logging** | tracing | 0.1 | All | Structured logging standard for Rust. Instrument functions, multiple log levels. |
| **Benchmarking** | criterion | 0.7.0 | Epic 7 | Statistics-driven benchmarking with HTML reports. Performance regression detection in CI. |
| **Image Processing** | image + imageproc | 0.25 + 0.24 | Epic 3 | Standard Rust image library. Handles PNG/JPG/GIF/BMP/WebP/TIFF. Feature-gated. |
| **SVG Rendering** | resvg + usvg | 0.38 + 0.38 | Epic 3 | SVG rasterization to bitmap. Feature-gated separately from raster images. |
| **Module Structure** | Feature-based | N/A | All | One module per epic/feature (grid, render, image, primitives, color, animation). Clear boundaries. |
| **Public API** | Minimal surface | N/A | All | Only essential types exported (BrailleGrid, TerminalRenderer, etc.). Internals stay private. |
| **Extraction Strategy** | Copy-refactor-test | N/A | Epic 2 | Copy crabmusic code, strip audio deps, refactor to modules, test to lock behavior, then optimize. |
| **Terminal Abstraction** | Trait-based | N/A | Epic 2 | TerminalBackend trait allows custom backends (testing, alternative terminals). DefaultTerminal uses ratatui. |
| **Image Pipeline** | Staged pipeline | N/A | Epic 3 | Load → Resize → Grayscale → Dither → Threshold → Map. Each stage separate for profiling. |
| **Memory Management** | Buffer reuse | N/A | Epic 6, 7 | BrailleGrid buffers reused across frames. Builder pattern for initial allocation. <500KB per frame. |
| **Performance Approach** | Measure-first | N/A | Epic 7 | No optimization without benchmark proof. criterion guides effort. Flamegraph for hotspots. |
| **Async API** | Sync-only (MVP) | N/A | All | Rendering is CPU-bound, not I/O. Async adds complexity with zero benefit. Defer to post-1.0. |
| **Builder Pattern** | Complex types only | N/A | Epic 2, 3 | BrailleGrid and ImageRenderer get builders (many options). Simple types use constructors. |

## Project Structure

```
dotmax/
├── .github/
│   └── workflows/
│       ├── ci.yml                    # Cross-platform testing (Story 1.2)
│       ├── benchmark.yml             # Performance tracking (Story 1.6)
│       └── release.yml               # crates.io publishing (Epic 7)
│
├── benches/
│   ├── rendering.rs                  # Core rendering benchmarks
│   ├── image_conversion.rs           # Image pipeline benchmarks
│   └── animation.rs                  # Frame rate benchmarks
│
├── docs/
│   ├── adr/                          # Architecture Decision Records
│   │   ├── README.md                 # ADR index
│   │   ├── template.md               # ADR template
│   │   ├── 0001-use-braille-unicode.md
│   │   ├── 0002-thiserror-for-errors.md
│   │   ├── 0003-feature-flag-architecture.md
│   │   └── 0004-terminal-abstraction-layer.md
│   ├── dependencies.md               # Dependency justifications
│   └── performance.md                # Performance targets and results
│
├── examples/
│   ├── README.md                     # Example index
│   ├── hello_braille.rs              # Minimal example (<50 lines)
│   ├── render_image.rs               # Image rendering demo
│   ├── draw_shapes.rs                # Primitives demo
│   ├── color_schemes.rs              # Color system demo
│   └── simple_animation.rs           # Animation demo
│
├── src/
│   ├── lib.rs                        # Public API, re-exports, feature gates
│   │
│   ├── error.rs                      # Error types (thiserror)
│   │   // DotmaxError enum
│   │   // - ImageError (load, format, decode)
│   │   // - RenderError (terminal, buffer, invalid state)
│   │   // - TerminalError (backend, capability detection)
│   │
│   ├── grid.rs                       # BrailleGrid, GridBuffer (Epic 2)
│   │   // struct BrailleGrid
│   │   // struct GridBuffer
│   │   // Dot manipulation, Unicode conversion
│   │
│   ├── render.rs                     # Terminal rendering (Epic 2)
│   │   // trait TerminalBackend
│   │   // struct DefaultTerminal (ratatui/crossterm)
│   │   // struct TerminalRenderer
│   │
│   ├── image/                        # Image rendering (Epic 3, feature-gated)
│   │   ├── mod.rs                    // Public API
│   │   ├── loader.rs                 // Image loading (PNG, JPG, etc.)
│   │   ├── resize.rs                 // Resizing with aspect ratio
│   │   ├── convert.rs                // Color → grayscale
│   │   ├── threshold.rs              // Otsu thresholding
│   │   ├── dither.rs                 // Floyd-Steinberg, Bayer, Atkinson
│   │   └── mapper.rs                 // Pixels → braille dots
│   │
│   ├── primitives.rs                 # Drawing primitives (Epic 4)
│   │   // Bresenham line algorithm
│   │   // Bresenham circle algorithm
│   │   // Rectangle, polygon drawing
│   │
│   ├── density.rs                    # Character density rendering (Epic 4)
│   │   // Intensity → character mapping
│   │   // Predefined density sets
│   │
│   ├── color.rs                      # Color support (Epic 5)
│   │   // struct Color (RGB)
│   │   // struct ColorScheme
│   │   // Terminal color conversion (ANSI 256, true color)
│   │   // 6+ predefined schemes from crabmusic
│   │
│   ├── animation.rs                  # Animation & frames (Epic 6)
│   │   // struct FrameBuffer
│   │   // Frame timing, loop management
│   │   // Flicker-free rendering
│   │
│   └── utils/                        # Internal utilities
│       ├── mod.rs
│       └── terminal_caps.rs          // Terminal capability detection
│
├── tests/
│   ├── integration_tests.rs          # End-to-end tests
│   ├── image_rendering_tests.rs      # Image pipeline tests
│   ├── cross_platform_tests.rs       # Platform-specific behavior
│   └── test_assets/                  # Test images, data
│       ├── sample.png
│       ├── test_svg.svg
│       └── benchmark_image.jpg
│
├── .deny.toml                        # Cargo-deny config (licenses, advisories)
├── .gitignore                        # Rust standard gitignore
├── .rustfmt.toml                     # Formatting config
├── Cargo.toml                        # Package manifest
├── CHANGELOG.md                      # Version history
├── clippy.toml                       # Clippy linting config
├── LICENSE-MIT                       # MIT license
├── LICENSE-APACHE                    # Apache 2.0 license
└── README.md                         # Main documentation
```

## Epic to Architecture Mapping

| Epic | Modules/Files | Key Types | Extracted from crabmusic? |
|------|---------------|-----------|---------------------------|
| **Epic 1: Foundation** | Cargo.toml, CI, tooling, docs/adr/ | N/A | No (new infrastructure) |
| **Epic 2: Core Rendering** | src/grid.rs, src/render.rs, src/error.rs | BrailleGrid, GridBuffer, TerminalRenderer, DotmaxError | Yes (~1,500 lines) |
| **Epic 3: 2D Image** | src/image/* (feature-gated) | ImageRenderer, DitherMethod, ThresholdAlgorithm | Partial (~500 lines + new) |
| **Epic 4: Primitives** | src/primitives.rs, src/density.rs | Line, Circle, DensitySet | Yes (~500 lines) |
| **Epic 5: Color** | src/color.rs | Color, ColorScheme | Yes (~150 lines) |
| **Epic 6: Animation** | src/animation.rs | AnimationRenderer, FrameTimer | Partial (new design) |
| **Epic 7: Production** | tests/, benches/, examples/, CI | N/A | No (new quality work) |

**Total Extraction:** ~2,000-3,000 lines from crabmusic, refactored and enhanced.

## Technology Stack Details

### Core Technologies

**Language & Edition:**
- Rust 2021 edition
- MSRV: 1.70 (documented in Cargo.toml, enforced in CI)
- Stable Rust only (no nightly features)

**Core Dependencies (always included):**
```toml
[dependencies]
ratatui = "0.29"         # Terminal UI framework
crossterm = "0.29"       # Cross-platform terminal I/O
thiserror = "2.0"        # Error handling derive macros
tracing = "0.1"          # Structured logging
```

**Optional Dependencies (feature-gated):**
```toml
[dependencies]
image = { version = "0.25", optional = true }
imageproc = { version = "0.24", optional = true }
resvg = { version = "0.38", optional = true }
usvg = { version = "0.38", optional = true }

[features]
default = []
image = ["dep:image", "dep:imageproc"]
svg = ["dep:resvg", "dep:usvg"]
```

**Dev Dependencies:**
```toml
[dev-dependencies]
criterion = { version = "0.7", features = ["html_reports"] }
tracing-subscriber = "0.3"
```

### Integration Points

**Data Flow Architecture:**

```
Input Sources
    ├── Image Files (PNG, JPG, GIF, BMP, WebP, TIFF, SVG)
    ├── Drawing Primitives (lines, circles, shapes)
    └── Direct Grid Manipulation (set_dot)
         ↓
    Processing Pipeline
         ├── Image: Load → Resize → Grayscale → Dither → Threshold → Map
         ├── Primitives: Bresenham algorithms → Dot setting
         └── Direct: Dot manipulation
         ↓
    BrailleGrid (central state)
         ├── Dots: Vec<u8> (packed bit patterns)
         ├── Colors: Option<Vec<Color>> (per-cell RGB)
         └── Dimensions: (width, height) in cells
         ↓
    TerminalRenderer
         ├── Convert dots to Unicode braille (U+2800 + pattern)
         ├── Apply colors (ANSI 256 or true color)
         └── Output via TerminalBackend
         ↓
    Terminal Output (ratatui/crossterm)
```

**Module Dependencies:**

```
lib.rs (public API re-exports)
   ├── error.rs (no dependencies)
   ├── grid.rs → error.rs
   ├── render.rs → grid.rs, error.rs, (ratatui, crossterm)
   ├── image/* → grid.rs, error.rs, (image, imageproc, resvg, usvg)
   ├── primitives.rs → grid.rs, error.rs
   ├── density.rs → grid.rs, error.rs
   ├── color.rs → error.rs
   └── animation.rs → grid.rs, render.rs, error.rs
```

**External System Interfaces:**
- **Terminal:** Via ratatui/crossterm (abstracted by TerminalBackend trait)
- **Filesystem:** Image loading from disk (via `image` crate)
- **crates.io:** Distribution and dependency management

## Novel Pattern Designs

### Pattern 1: Braille Dot Matrix Mapping

**Purpose:** Convert pixel data to 2×4 braille dot patterns efficiently while maintaining visual quality.

**Components:**

1. **BrailleGrid** - Core rendering surface
   - Stores dots as packed bits (8 dots per byte)
   - Handles 2×4 dot matrix per terminal cell
   - Converts to Unicode braille characters (U+2800-U+28FF)

2. **Dot Coordinate System**
   - Cell coordinates: (cell_x, cell_y)
   - Dot coordinates: (x, y) where x/y are in dot space
   - Mapping: cell_x = x / 2, cell_y = y / 4

**Data Flow:**

```
Pixel Grid (input)          Braille Cell (output)
┌─┬─┬─┬─┐                  ┌─┬─┐
│ │ │ │ │  → threshold →   │•│ │  Dot positions:
├─┼─┼─┼─┤                  ├─┼─┤  0 3
│ │ │ │ │  → map       →   │•│•│  1 4
├─┼─┼─┼─┤                  ├─┼─┤  2 5
│ │ │ │ │                  │ │•│  6 7
├─┼─┼─┼─┤                  └─┴─┘
│ │ │ │ │
└─┴─┴─┴─┘
```

**Implementation Guide:**

```rust
// src/grid.rs
pub struct BrailleGrid {
    width: usize,   // Width in braille cells
    height: usize,  // Height in braille cells
    dots: Vec<u8>,  // Packed dot data (8 dots per byte)
    colors: Option<Vec<Color>>,  // Optional color per cell
}

impl BrailleGrid {
    /// Set a single dot at (x, y) coordinates
    /// x, y are in DOT coordinates (not cell coordinates)
    pub fn set_dot(&mut self, x: usize, y: usize, value: bool) {
        let cell_x = x / 2;  // 2 dots wide per cell
        let cell_y = y / 4;  // 4 dots tall per cell
        let dot_x = x % 2;   // Which column within cell (0 or 1)
        let dot_y = y % 4;   // Which row within cell (0-3)

        // Braille Unicode uses specific bit pattern
        let dot_index = dot_x * 4 + dot_y;
        let cell_index = cell_y * self.width + cell_x;

        if value {
            self.dots[cell_index] |= 1 << dot_index;  // Set bit
        } else {
            self.dots[cell_index] &= !(1 << dot_index);  // Clear bit
        }
    }

    /// Convert dot pattern to Unicode braille character
    pub fn to_unicode(&self, cell_index: usize) -> char {
        let pattern = self.dots[cell_index];
        char::from_u32(0x2800 + pattern as u32).unwrap()
    }
}
```

**Affects Epics:** Epic 2 (Core), Epic 3 (Image mapping), Epic 4 (Drawing)

---

### Pattern 2: Image-to-Braille Conversion Pipeline

**Purpose:** Convert arbitrary resolution images to low-resolution braille grids while maximizing visual quality through optimal dithering and thresholding.

**Components:**

1. **loader.rs** - Load images from disk/memory
2. **resize.rs** - Resize to terminal dimensions, preserve aspect ratio
3. **convert.rs** - RGB → grayscale conversion
4. **dither.rs** - Floyd-Steinberg, Bayer, Atkinson algorithms
5. **threshold.rs** - Otsu algorithm for optimal binary threshold
6. **mapper.rs** - Pixel blocks (2×4) → braille dots

**Data Flow:**

```
Image (any size)
    ↓
[1. Resize to terminal dimensions with aspect ratio preservation]
    ↓ (e.g., 800x600 → 160x96 dots = 80x24 cells)
[2. Convert to grayscale if needed]
    ↓ (RGB → luminance)
[3. Apply dithering for quality]
    ↓ (Floyd-Steinberg distributes quantization error)
[4. Threshold to binary (black/white)]
    ↓ (Otsu algorithm finds optimal threshold)
[5. Map pixels to braille dots]
    ↓ (2x4 pixel blocks → single braille cell)
BrailleGrid (ready to render)
```

**Implementation Guide:**

```rust
// src/image/mod.rs
pub struct ImageRenderer {
    dither_method: DitherMethod,
    threshold_algo: ThresholdAlgorithm,
    preserve_aspect: bool,
}

impl ImageRenderer {
    pub fn render_to_grid(
        &self,
        img: DynamicImage,
        target_width: usize,  // In braille cells
        target_height: usize  // In braille cells
    ) -> Result<BrailleGrid, DotmaxError> {
        // Pipeline stages (each in separate module for profiling)
        let resized = resize::resize_with_aspect(
            img,
            target_width * 2,   // Convert cells to dots (width)
            target_height * 4   // Convert cells to dots (height)
        )?;

        let grayscale = convert::to_grayscale(resized);
        let dithered = dither::apply(grayscale, self.dither_method)?;
        let binary = threshold::apply(dithered, self.threshold_algo)?;
        let grid = mapper::pixels_to_braille(binary, target_width, target_height)?;

        Ok(grid)
    }
}
```

**Performance Critical:** This entire pipeline must complete in <50ms (<25ms target). Each stage needs separate benchmarking.

**Affects Epics:** Epic 3 (Image Rendering), Epic 7 (Performance optimization)

---

### Pattern 3: Buffer Reuse for Animation

**Purpose:** Achieve 60fps animation without allocating new grids each frame. Target <500KB memory overhead per frame.

**Components:**

1. **AnimationRenderer** - Manages grid reuse
2. **FrameTimer** - Frame timing control
3. **BrailleGrid** - Stateful buffer (reused, not reallocated)

**Data Flow:**

```
AnimationRenderer initialization
    ↓
Allocate BrailleGrid once
    ↓
Loop (60+ fps):
    ├─ Clear grid (reuse buffer, no dealloc)
    ├─ User draws frame (closure)
    ├─ Render to terminal
    └─ Wait for frame timing
```

**Implementation Guide:**

```rust
// src/animation.rs
pub struct AnimationRenderer {
    grid: BrailleGrid,           // Reused buffer
    terminal: TerminalRenderer,   // Reused terminal
    frame_timer: FrameTimer,      // Timing control
}

impl AnimationRenderer {
    /// Render frame without allocation
    pub fn render_frame<F>(&mut self, frame_fn: F) -> Result<(), DotmaxError>
    where
        F: FnOnce(&mut BrailleGrid) -> Result<(), DotmaxError>
    {
        self.grid.clear();  // Reuse buffer
        frame_fn(&mut self.grid)?;
        self.terminal.render(&self.grid)?;
        self.frame_timer.wait_for_next_frame();
        Ok(())
    }

    /// Animation loop
    pub fn animate<F>(&mut self, mut frame_fn: F) -> Result<(), DotmaxError>
    where
        F: FnMut(u64, &mut BrailleGrid) -> Result<bool, DotmaxError>
    {
        let mut frame_num = 0;
        loop {
            self.grid.clear();
            let should_continue = frame_fn(frame_num, &mut self.grid)?;
            if !should_continue { break; }

            self.terminal.render(&self.grid)?;
            self.frame_timer.wait_for_next_frame();
            frame_num += 1;
        }
        Ok(())
    }
}
```

**Memory Target:** <500KB per frame overhead (NFR-P3). Reusing grid buffer is critical.

**Affects Epics:** Epic 6 (Animation), Epic 7 (Performance)

---

### Pattern 4: Terminal Backend Abstraction

**Purpose:** Reduce lock-in to ratatui/crossterm while keeping the common case (99% of users) simple.

**Components:**

1. **TerminalBackend trait** - Minimal abstraction
2. **DefaultTerminal** - ratatui/crossterm implementation
3. **TerminalCapabilities** - Feature detection

**Implementation Guide:**

```rust
// src/render.rs
pub trait TerminalBackend {
    fn size(&self) -> Result<(u16, u16), DotmaxError>;
    fn render(&mut self, content: &str) -> Result<(), DotmaxError>;
    fn capabilities(&self) -> TerminalCapabilities;
}

pub struct TerminalCapabilities {
    pub supports_color: bool,
    pub supports_truecolor: bool,
    pub supports_unicode: bool,
}

// Default implementation (99% of users)
pub struct DefaultTerminal {
    backend: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
}

impl TerminalBackend for DefaultTerminal {
    // Implementation using ratatui/crossterm
}
```

**Usage:**

```rust
// Easy path (default)
let terminal = DefaultTerminal::new()?;

// Custom path (testing, alternative backends)
struct MockTerminal { /* ... */ }
impl TerminalBackend for MockTerminal { /* ... */ }
```

**Affects Epics:** Epic 2 (Core Rendering)

## Implementation Patterns

These patterns ensure consistent implementation across all AI agents:

### Naming Conventions

**Files and Modules:** `snake_case`
```rust
src/braille_grid.rs
src/terminal_backend.rs
src/image/floyd_steinberg.rs
```

**Types (structs, enums, traits):** `PascalCase`
```rust
pub struct BrailleGrid { }
pub enum DotmaxError { }
pub trait TerminalBackend { }
```

**Functions, Variables, Methods:** `snake_case`
```rust
pub fn render_to_terminal(&self) -> Result<()> { }
let image_width = 100;
```

**Constants:** `SCREAMING_SNAKE_CASE`
```rust
const BRAILLE_UNICODE_BASE: u32 = 0x2800;
const MAX_GRID_WIDTH: usize = 1000;
```

### Code Organization

**File Organization:** One primary type per file, matching filename
```rust
// src/grid.rs - Primary type: BrailleGrid
pub struct BrailleGrid { }
pub struct GridBuffer { }  // Helper type, closely related
```

**Module Organization:** Feature-based modules
```rust
// src/image/mod.rs
pub mod loader;
pub mod resize;
pub mod dither;

pub use loader::ImageLoader;
pub use dither::DitherMethod;
```

**Test Organization:** Tests in same file under `#[cfg(test)]`
```rust
// src/grid.rs
impl BrailleGrid {
    pub fn new(width: usize, height: usize) -> Self { }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let grid = BrailleGrid::new(10, 10);
        assert_eq!(grid.width(), 10);
    }
}
```

### Error Handling

**All errors are `DotmaxError` enum with thiserror:**

```rust
// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DotmaxError {
    #[error("Failed to load image from {path}: {source}")]
    ImageLoad {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },

    #[error("Invalid grid dimensions: {width}x{height}")]
    InvalidDimensions { width: usize, height: usize },

    #[error("Terminal error: {0}")]
    Terminal(#[from] std::io::Error),
}
```

**All public functions return `Result<T, DotmaxError>`:**
```rust
pub fn load_image(path: &Path) -> Result<Image, DotmaxError> { }
pub fn render(&mut self) -> Result<(), DotmaxError> { }
```

### Logging Strategy

**Use `tracing` crate for structured logging:**

```rust
use tracing::{debug, info, warn, error, instrument};

#[instrument]
pub fn render_image(&mut self, path: &Path) -> Result<()> {
    info!("Loading image from {:?}", path);
    debug!("Image dimensions: {}x{}", img.width(), img.height());
    // ... work ...
    info!("Image rendered successfully");
}
```

**Log Levels:**
- `error!` - Operation failed (user needs to know)
- `warn!` - Something unexpected but recoverable
- `info!` - Major operations (image loaded, render complete)
- `debug!` - Detailed flow (algorithm steps, buffer sizes)
- `trace!` - Hot path internals (only if TRACE enabled)

## Consistency Rules

### Function Signatures

**References for read, mutable references for modify, owned for consume:**

```rust
// Read-only
pub fn render(&self, grid: &BrailleGrid) -> Result<()> { }

// Modify
pub fn clear(&mut self) -> Result<()> { }

// Consume
pub fn into_buffer(self) -> Vec<u8> { }
```

### Option vs Result

**`Option` for "may not exist", `Result` for "may fail":**

```rust
// Good - color might not be set
pub fn get_color(&self, x: usize, y: usize) -> Option<Color> { }

// Good - loading can fail
pub fn load_image(path: &Path) -> Result<Image, DotmaxError> { }
```

### Builder Pattern

**Use `new()` for simple construction, `builder()` for complex:**

```rust
// Simple
impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

// Complex
impl BrailleGrid {
    pub fn builder() -> BrailleGridBuilder {
        BrailleGridBuilder::default()
    }
}

let grid = BrailleGrid::builder()
    .size(80, 24)
    .with_color_support()
    .build()?;
```

### Import Organization

**Group imports: std, external crates, internal modules**

```rust
use std::path::Path;
use std::io::Write;

use thiserror::Error;
use tracing::{debug, info};

use crate::grid::BrailleGrid;
use crate::error::DotmaxError;
```

### Feature Gates

**Consistent feature gate syntax:**

```rust
// At module level
#[cfg(feature = "image")]
pub mod image;

// At item level
#[cfg(feature = "image")]
pub use image::ImageRenderer;

// In Cargo.toml
[features]
default = []
image = ["dep:image", "dep:imageproc"]
svg = ["dep:resvg", "dep:usvg"]
```

### Documentation Format

**Every public item needs rustdoc with this structure:**

```rust
/// Brief one-line summary.
///
/// Longer explanation if needed. Describe what it does, not how.
///
/// # Examples
///
/// ```
/// use dotmax::BrailleGrid;
///
/// let grid = BrailleGrid::new(80, 24);
/// grid.render()?;
/// # Ok::<(), dotmax::DotmaxError>(())
/// ```
///
/// # Errors
///
/// Returns [`DotmaxError::InvalidDimensions`] if width or height is 0.
///
/// # Panics
///
/// Does not panic. (Or describe when it panics)
pub fn new(width: usize, height: usize) -> Self { }
```

## Data Architecture

### Core Data Models

**BrailleGrid - Central State:**
```rust
pub struct BrailleGrid {
    width: usize,               // Width in braille cells
    height: usize,              // Height in braille cells
    dots: Vec<u8>,              // Packed dot data (1 byte = 8 dots per cell)
    colors: Option<Vec<Color>>, // Optional per-cell RGB colors
}
```

**Color - RGB Representation:**
```rust
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
```

**ColorScheme - Intensity Mapping:**
```rust
pub struct ColorScheme {
    name: String,
    colors: Vec<Color>,  // Intensity 0.0 → colors[0], 1.0 → colors[n-1]
}
```

**DitherMethod - Algorithm Selection:**
```rust
pub enum DitherMethod {
    None,
    FloydSteinberg,
    Bayer,
    Atkinson,
}
```

### Data Relationships

```
BrailleGrid
    ├── dots: Vec<u8>              (1:1 per cell, packed bits)
    └── colors: Option<Vec<Color>> (1:1 per cell, optional)

ColorScheme
    └── colors: Vec<Color>         (intensity mapping)

ImageRenderer
    ├── dither_method: DitherMethod
    └── threshold_algo: ThresholdAlgorithm
```

No complex relationships—data is primarily flat structures optimized for performance.

## API Contracts

### Public API Surface

**Re-exported from lib.rs:**

```rust
// Core types (always available)
pub use grid::BrailleGrid;
pub use render::{TerminalBackend, TerminalRenderer};
pub use error::DotmaxError;
pub use color::{Color, ColorScheme};

// Feature-gated types
#[cfg(feature = "image")]
pub use image::ImageRenderer;

#[cfg(feature = "svg")]
pub use image::SvgRenderer;
```

### Key Type Signatures

**BrailleGrid:**
```rust
impl BrailleGrid {
    pub fn new(width: usize, height: usize) -> Self;
    pub fn builder() -> BrailleGridBuilder;
    pub fn set_dot(&mut self, x: usize, y: usize, value: bool);
    pub fn clear(&mut self);
    pub fn width(&self) -> usize;
    pub fn height(&self) -> usize;
}
```

**TerminalRenderer:**
```rust
impl TerminalRenderer {
    pub fn new() -> Result<Self, DotmaxError>;
    pub fn render(&mut self, grid: &BrailleGrid) -> Result<(), DotmaxError>;
}
```

**ImageRenderer (feature-gated):**
```rust
impl ImageRenderer {
    pub fn builder() -> ImageRendererBuilder;
    pub fn render_to_grid(&self, img: DynamicImage, width: usize, height: usize)
        -> Result<BrailleGrid, DotmaxError>;
}
```

## Security Architecture

Dotmax is a graphics rendering library with minimal security surface:

### Memory Safety
- **Zero unsafe code in MVP** - Rely on Rust's memory safety guarantees
- **If unsafe required later:** Isolate in separate module, document invariants, test with MIRI
- **No buffer overflows** - Rust prevents out-of-bounds access

### Input Validation
- **Image files:** Validated by `image` crate (external input)
- **Dimensions:** Checked for zero/overflow before allocation
- **Resource limits:** Max grid dimensions to prevent OOM attacks
  ```rust
  const MAX_GRID_WIDTH: usize = 10_000;
  const MAX_GRID_HEIGHT: usize = 10_000;
  ```

### Dependency Security
- **cargo-audit in CI:** Detect known vulnerabilities
- **Minimal dependencies:** <10 core, reduce attack surface
- **Permissive licenses only:** MIT/Apache-2.0 (no viral licenses)

### No Authentication/Authorization
Dotmax is a library, not a service. No users, no auth, no network.

## Performance Considerations

**Performance is the highest priority NFR.** Dotmax must be "efficiency so fast, it's invisible."

### Performance Targets

**Rendering Latency:**
- Image-to-braille: **<25ms target** (50ms max) for 80×24 terminals
- Large terminals (200×50): <100ms max
- SVG rasterization: <100ms for typical graphics

**Animation Performance:**
- Sustained **60fps minimum** with <10% CPU (single core)
- Target capability: **120fps** on modern hardware
- No dropped frames during 30+ second playback

**Memory Efficiency:**
- Baseline: **<5MB** for core operations
- Per-frame overhead: **<500KB** for typical renders
- **Zero memory leaks** (valgrind/MIRI validated in CI)
- Bounded allocations (no unbounded growth)

**Startup Performance:**
- Library initialization: **<5ms** (cold start)
- First render after init: <10ms additional

**Binary Size:**
- Core library: **<2MB** addition to compiled binaries
- Feature flags minimize bloat (opt-in for image/svg/etc.)

### Performance Strategies

**1. Buffer Reuse Pattern:**
```rust
// Good - reuse allocation
pub fn clear(&mut self) {
    self.dots.fill(0);  // Reuse Vec
}

// Bad - new allocation
pub fn clear(&mut self) {
    self.dots = vec![0; self.width * self.height];
}
```

**2. Measure-First Optimization:**
- No optimization without benchmark proof
- Use criterion for microbenchmarks
- Use flamegraph to identify hotspots
- Profile after Epic 2, Epic 3, and Epic 7

**3. Zero-Copy Where Possible:**
```rust
// Good - pass by reference
pub fn render(&self, grid: &BrailleGrid) -> Result<()> { }

// Bad - forces clone/move
pub fn render(&self, grid: BrailleGrid) -> Result<()> { }
```

**4. Pipeline Profiling:**

Each stage of image pipeline benchmarked separately:
- Resize stage: <10ms
- Dithering: <15ms
- Thresholding: <5ms
- Braille mapping: <10ms
- **Total: <50ms** (with margin)

**5. Profiling Tools:**
- **criterion.rs** - Microbenchmarks with statistical analysis
- **flamegraph** - Visual profiling (find hotspots)
- **valgrind/heaptrack** - Memory profiling
- **cargo-bloat** - Binary size analysis

### Performance Validation

**CI Performance Checks:**
- Benchmark suite runs on main branch
- Performance regression detected if >10% slower
- Results stored as artifacts for tracking

**Competitive Benchmarking:**
- Compare against drawille (Python)
- Compare against ascii-image-converter (Go)
- Publish results for transparency

## Deployment Architecture

Dotmax is a **library crate**, not a deployed service. Deployment means **publishing to crates.io**.

### Distribution Model

**Primary Distribution: crates.io**
```bash
# Users install via Cargo
cargo add dotmax

# Or specify in Cargo.toml
[dependencies]
dotmax = "0.1"  # Semantic versioning
```

**Source Code: GitHub**
- Public repository
- GitHub Actions for CI/CD
- Releases tagged with semantic versions

### Release Process

**Automated Release Workflow (.github/workflows/release.yml):**

1. **Trigger:** Git tag push (`v0.1.0`, `v1.0.0`, etc.)
2. **CI Validation:**
   - Run full test suite (all platforms)
   - Run benchmarks (ensure no regressions)
   - Run clippy (no warnings)
   - Verify docs build
3. **crates.io Publish:**
   - `cargo publish` (automated)
   - Update CHANGELOG.md
4. **GitHub Release:**
   - Create release notes
   - Attach artifacts (benchmarks, docs)

### Versioning Strategy

**Semantic Versioning (semver):**
- `0.x.y` - Pre-1.0: Minor version may break compatibility
- `1.x.y` - Post-1.0: Major = breaking, Minor = features, Patch = fixes
- **1.0.0 = Stable API** - No breaking changes in 1.x line

**MSRV Policy:**
- Minimum Supported Rust Version: **1.70**
- MSRV only bumped with minor version increments
- Documented in README and enforced in CI

### Platform Support

**Tier 1 (Fully Supported):**
- Windows (x86_64)
- Linux (x86_64)
- macOS (x86_64, ARM64)

**CI Testing Matrix:**
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
    rust: [stable, 1.70]  # Stable + MSRV
```

## Development Environment

### Prerequisites

**Required:**
- **Rust 1.70 or later** (stable toolchain)
  ```bash
  rustup update stable
  ```
- **Git** (for version control)
- **Terminal with Unicode support** (for testing braille output)

**Optional (for full development):**
- **cargo-deny** - License/security checking
  ```bash
  cargo install cargo-deny
  ```
- **cargo-audit** - Security vulnerability scanning
  ```bash
  cargo install cargo-audit
  ```
- **flamegraph** - Performance profiling
  ```bash
  cargo install flamegraph
  ```

### Setup Commands

```bash
# Clone repository
git clone https://github.com/newjordan/dotmax.git
cd dotmax

# Build (core only)
cargo build

# Build with all features
cargo build --features image,svg

# Run tests
cargo test

# Run tests with features
cargo test --features image,svg

# Run benchmarks
cargo bench

# Run examples
cargo run --example hello_braille
cargo run --example render_image --features image

# Check code quality
cargo clippy -- -D warnings
cargo fmt --check
cargo deny check

# Generate documentation
cargo doc --open --features image,svg
```

### Development Workflow

**1. Before starting work:**
```bash
git checkout main
git pull
cargo test  # Ensure clean state
```

**2. During development:**
```bash
# Run tests frequently
cargo test

# Check for issues
cargo clippy

# Format code
cargo fmt
```

**3. Before committing:**
```bash
cargo test --all-features
cargo clippy -- -D warnings
cargo fmt --check
cargo deny check
```

**4. Performance work:**
```bash
# Run benchmarks
cargo bench

# Profile hot paths
cargo flamegraph --example render_image --features image
```

## Architecture Decision Records (ADRs)

### ADR 0001: Use Unicode Braille for Terminal Rendering

**Status:** Accepted

**Context:** Need cross-platform terminal graphics without protocol dependencies. Options include Sixel, Kitty Graphics Protocol, iTerm2 Protocol, or text-based approaches.

**Decision:** Use Unicode braille characters (U+2800-U+28FF) for rendering. Each braille character represents a 2×4 dot matrix, providing 4× the resolution of ASCII art.

**Consequences:**
- ✅ Universal compatibility (any terminal with Unicode support)
- ✅ Works over SSH, in tmux/screen, all environments
- ✅ 4× resolution advantage over ASCII
- ❌ Requires Unicode terminal (99%+ of modern terminals)
- ❌ Lower resolution than graphics protocols (but works everywhere)

**Alternatives Considered:**
- Sixel: Limited terminal support, doesn't work in tmux
- Kitty Protocol: Modern but limited adoption, breaks in tmux
- iTerm2 Protocol: macOS only
- ASCII art: Lower resolution (1 char = 1 pixel)

---

### ADR 0002: Use thiserror for Error Handling

**Status:** Accepted

**Context:** Library needs clear error types for users to handle different error conditions. Options include thiserror (derive macros), anyhow (type-erased), or manual std::error::Error implementation.

**Decision:** Use thiserror for custom error types with DotmaxError enum.

**Consequences:**
- ✅ Type-safe error matching for users
- ✅ Minimal boilerplate (derive macro)
- ✅ Clear error context and chaining
- ❌ Slightly more verbose than anyhow

**Alternatives Considered:**
- anyhow: Better for applications, but type-erased (bad for library users)
- Manual std::error::Error: Too verbose, reinventing wheel

---

### ADR 0003: Feature Flag Architecture

**Status:** Accepted

**Context:** Core rendering should be lightweight, but users need image/SVG support. Must balance binary size with functionality.

**Decision:** Use Cargo feature flags for optional capabilities. Core has zero optional dependencies. Image/SVG/video/3D are opt-in.

**Consequences:**
- ✅ Core library stays minimal (<2MB binary size)
- ✅ Users only pay for what they use
- ✅ Easy to add new features without bloating core
- ❌ More complex build matrix (test with/without features)

**Alternatives Considered:**
- All-in-one: Simple but bloated (10+ MB binary)
- Separate crates: Complex dependency management, harder to maintain

---

### ADR 0004: Terminal Backend Abstraction via Trait

**Status:** Accepted

**Context:** Need to reduce lock-in to ratatui/crossterm while keeping common case simple. May want alternative backends (testing, embedded).

**Decision:** Define minimal TerminalBackend trait. Provide DefaultTerminal using ratatui/crossterm. Users can implement custom backends.

**Consequences:**
- ✅ Reduces vendor lock-in
- ✅ Enables testing with mock terminals
- ✅ Future-proof for alternative backends
- ❌ Small abstraction overhead (negligible)

**Alternatives Considered:**
- Direct ratatui dependency: Simpler but locked in
- Zero abstraction: Harder to test and swap backends

---

### ADR 0005: Brownfield Extraction Strategy - Copy-Refactor-Test

**Status:** Accepted

**Context:** Extracting ~2,000-3,000 lines from crabmusic. Could rewrite from scratch or extract incrementally.

**Decision:** Copy exact working code, strip audio dependencies, refactor to modules, add tests, then optimize.

**Consequences:**
- ✅ Preserves proven working behavior
- ✅ Tests lock in correctness before optimization
- ✅ Lower risk than rewrite
- ❌ Initial code may not be optimal (refactor later)

**Alternatives Considered:**
- Rewrite from scratch: Higher risk, may lose quality
- Direct port: Messy codebase, hard to maintain

---

### ADR 0006: Sync-Only API for MVP

**Status:** Accepted

**Context:** PRD mentions async compatibility (FR47), but braille rendering is CPU-bound computation, not I/O.

**Decision:** MVP has sync-only API. Users can wrap in `tokio::spawn_blocking` if needed. Defer async wrappers to post-1.0.

**Consequences:**
- ✅ Simpler implementation and testing
- ✅ No async runtime dependency
- ✅ CPU-bound work doesn't benefit from async
- ❌ Users in async contexts must wrap manually

**Alternatives Considered:**
- Async-first: Complexity with zero performance benefit
- Dual API: More code to maintain, confusing for users

---

### ADR 0007: Measure-First Performance Optimization

**Status:** Accepted

**Context:** Performance is make-or-break (<50ms render target). Risk of premature optimization vs. missing targets.

**Decision:** No optimization without benchmark proof. Use criterion for all performance work. Profile with flamegraph before optimizing.

**Consequences:**
- ✅ Data-driven optimization (no guessing)
- ✅ criterion tracks regressions automatically
- ✅ Avoids wasted effort on non-hotspots
- ❌ Requires benchmark infrastructure (Story 1.6)

**Alternatives Considered:**
- Optimize everything upfront: Wastes time on non-critical paths
- Trust intuition: Risky for aggressive <25ms target

---

**Version:** 1.0
**Date:** 2025-11-14
**Author:** Winston (Architect AI)
**For:** Frosty

---

_This architecture document serves as the consistency contract for all AI agents implementing dotmax. All architectural decisions, patterns, and conventions defined here are MANDATORY for successful project completion._
