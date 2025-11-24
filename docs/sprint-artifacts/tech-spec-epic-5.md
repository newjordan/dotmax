# Epic Technical Specification: Color System & Visual Schemes

Date: 2025-11-23
Author: Frosty
Epic ID: 5
Status: Draft

---

## Overview

Epic 5 builds the comprehensive color system that transforms monochrome braille rendering into vibrant visual output. This epic extracts and enhances the proven color scheme system from crabmusic (~150 lines) while adding intelligent terminal capability detection and RGB-to-ANSI color conversion for universal compatibility.

The color system serves as the bridge between raw intensity data (grayscale images, density renders) and colorful terminal output, automatically adapting to each terminal's capabilities (monochrome, 16-color, 256-color, or 24-bit true color). This enables developers to create rich visual experiences that work consistently across diverse terminal environments—from basic Windows CMD to modern terminals like Alacritty and WezTerm.

## Objectives and Scope

**In Scope:**
- Terminal color capability detection (monochrome, ANSI 16, ANSI 256, true color 24-bit)
- RGB-to-ANSI color conversion algorithms (16-color and 256-color palettes)
- True color (24-bit RGB) escape code generation
- Extract and integrate 6+ predefined color schemes from crabmusic
- Custom color scheme creation API with intensity-to-color mapping
- Apply color schemes to grayscale intensity buffers
- Integration with Epic 2 (BrailleGrid color support)
- Integration with Epic 3 (image color mode rendering)
- Integration with Epic 4 (colored drawing primitives)

**Out of Scope:**
- Animation-specific color effects (Epic 6)
- Performance optimization beyond baseline (Epic 7 handles comprehensive benchmarking)
- Color palette generation from images (future feature)
- Custom dithering for color images (Epic 3 handles grayscale dithering)
- HSL/HSV color space support (RGB-only for MVP)

## System Architecture Alignment

**Architecture Document Reference:** `docs/architecture.md` - Decision Summary table, src/color.rs module

**Modules Involved:**
- `src/color.rs` - New module for color system (Epic 5)
- `src/grid.rs` - BrailleGrid already has `colors: Option<Vec<Color>>` field (Epic 2)
- `src/render.rs` - TerminalRenderer will use color escape codes (Epic 2)
- `src/utils/terminal_caps.rs` - New utility for capability detection

**Constraints from Architecture:**
- Use thiserror for `ColorError` types (ADR 0002)
- Follow naming conventions: `ColorScheme`, `ColorCapability`, `rgb_to_ansi256` (snake_case functions)
- All public APIs must have rustdoc with examples
- No unsafe code (memory safety)
- Performance target: Color conversion <100ns per color (from Epic 5 PRD context)

**Integration Points:**
- **Epic 2 (Core Rendering):** BrailleGrid.colors field already exists, TerminalRenderer needs color escape code output
- **Epic 3 (Image Rendering):** Color mode images need intensity→color mapping via ColorScheme
- **Epic 4 (Drawing Primitives):** Drawing functions need Color parameter support
- **Epic 6 (Animation):** Frame buffers will reuse color data structures

## Detailed Design

### Services and Modules

| Module/Service | Responsibilities | Inputs | Outputs | Owner |
|----------------|------------------|--------|---------|-------|
| `src/color.rs` | Core color types and schemes | RGB values, intensity | Color, ColorScheme | Story 5.3, 5.4 |
| `src/utils/terminal_caps.rs` | Terminal capability detection | Environment vars, ANSI queries | ColorCapability enum | Story 5.1 |
| `src/color/convert.rs` | RGB-to-ANSI conversion | RGB (u8, u8, u8), ColorCapability | ANSI codes, escape strings | Story 5.2 |
| `src/color/schemes.rs` | Predefined color schemes | Scheme name | ColorScheme | Story 5.3 |
| `src/color/apply.rs` | Apply schemes to intensity buffers | Grayscale buffer, ColorScheme | Colored BrailleGrid | Story 5.5 |

**Module Structure:**
```
src/color/
├── mod.rs           # Public API, re-exports
├── convert.rs       # RGB-to-ANSI conversion algorithms
├── schemes.rs       # Predefined color schemes from crabmusic
└── apply.rs         # Apply color schemes to intensity data

src/utils/
└── terminal_caps.rs # Terminal capability detection
```

### Data Models and Contracts

**Color (RGB representation):**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self;
    pub fn black() -> Self;
    pub fn white() -> Self;
    pub fn from_intensity(intensity: f32, scheme: &ColorScheme) -> Self;
}
```

**ColorCapability (terminal support levels):**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCapability {
    Monochrome,      // No color support
    Ansi16,          // 16 colors (basic ANSI)
    Ansi256,         // 256 colors (extended ANSI)
    TrueColor,       // 24-bit RGB (16 million colors)
}

impl ColorCapability {
    pub fn detect() -> Self;  // Detects current terminal capability
    pub fn supports_color(&self) -> bool;
    pub fn supports_truecolor(&self) -> bool;
}
```

**ColorScheme (intensity-to-color mapping):**
```rust
#[derive(Debug, Clone)]
pub struct ColorScheme {
    name: String,
    colors: Vec<Color>,  // Gradient from intensity 0.0 → colors[0] to 1.0 → colors[n-1]
}

impl ColorScheme {
    pub fn new(name: impl Into<String>, colors: Vec<Color>) -> Self;
    pub fn sample(&self, intensity: f32) -> Color;  // Linear interpolation

    // Predefined schemes (from crabmusic)
    pub fn fire() -> Self;       // Black → Red → Orange → Yellow → White
    pub fn ocean() -> Self;      // Black → Dark Blue → Cyan → White
    pub fn forest() -> Self;     // Black → Dark Green → Green → Light Green
    pub fn sunset() -> Self;     // Purple → Red → Orange → Yellow
    pub fn grayscale() -> Self;  // Black → Gray → White
    pub fn neon() -> Self;       // Black → Magenta → Cyan → Yellow
}
```

**Relationships:**
- `Color` is used in `BrailleGrid.colors: Option<Vec<Color>>` (Epic 2)
- `ColorScheme` contains `Vec<Color>` for gradient definition
- `ColorCapability` determines which conversion function to use in `convert.rs`

### APIs and Interfaces

**Terminal Capability Detection API:**
```rust
// src/utils/terminal_caps.rs
pub fn detect_color_capability() -> ColorCapability;
// Checks: $COLORTERM, $TERM, terminal queries
// Returns: Cached result (only detects once per session)
```

**RGB-to-ANSI Conversion API:**
```rust
// src/color/convert.rs
pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8;
// Maps RGB to closest ANSI 256 palette color (0-255 index)

pub fn rgb_to_ansi16(r: u8, g: u8, b: u8) -> u8;
// Maps RGB to basic 16 ANSI colors (0-15)

pub fn rgb_to_truecolor_escape(r: u8, g: u8, b: u8) -> String;
// Returns: "\x1b[38;2;{r};{g};{b}m" for foreground color

pub fn rgb_to_terminal_color(r: u8, g: u8, b: u8, capability: ColorCapability) -> String;
// Smart conversion: automatically chooses best method based on capability
```

**Color Scheme API:**
```rust
// src/color/schemes.rs
impl ColorScheme {
    pub fn sample(&self, intensity: f32) -> Color;
    // intensity: 0.0-1.0, returns interpolated color from gradient

    pub fn apply_to_grid(&self, intensities: &[f32], grid: &mut BrailleGrid) -> Result<(), DotmaxError>;
    // Applies color scheme to intensity buffer, populates grid.colors
}

// Predefined schemes (6+ from crabmusic)
pub fn fire() -> ColorScheme;
pub fn ocean() -> ColorScheme;
pub fn forest() -> ColorScheme;
pub fn sunset() -> ColorScheme;
pub fn grayscale() -> ColorScheme;
pub fn neon() -> ColorScheme;
```

**Integration with BrailleGrid (Epic 2):**
```rust
// src/grid.rs (existing, updated in Epic 5)
impl BrailleGrid {
    pub fn set_cell_color(&mut self, x: usize, y: usize, color: Color);
    // Sets color for specific braille cell

    pub fn apply_color_scheme(&mut self, intensities: &[f32], scheme: &ColorScheme) -> Result<(), DotmaxError>;
    // Convenience method: applies scheme to intensity buffer, updates self.colors
}
```

**Error Handling:**
```rust
// src/error.rs (extend DotmaxError)
pub enum DotmaxError {
    // ... existing variants ...

    #[error("Invalid intensity value: {0}, must be 0.0-1.0")]
    InvalidIntensity(f32),

    #[error("Color scheme '{0}' has no colors")]
    EmptyColorScheme(String),

    #[error("Terminal color capability detection failed: {0}")]
    ColorDetectionFailed(String),
}
```

### Workflows and Sequencing

**Workflow 1: Color Rendering Setup (Developer Perspective)**

1. Application startup → Detect terminal color capability
   ```rust
   let capability = detect_color_capability();  // Cached globally
   ```

2. Create/load color scheme
   ```rust
   let scheme = ColorScheme::fire();  // Or custom scheme
   ```

3. Render grayscale image/density data
   ```rust
   let intensities: Vec<f32> = /* ... from image processing ... */;
   ```

4. Apply color scheme to grid
   ```rust
   let mut grid = BrailleGrid::new(80, 24);
   scheme.apply_to_grid(&intensities, &mut grid)?;
   ```

5. Render to terminal with color escape codes
   ```rust
   renderer.render(&grid)?;  // TerminalRenderer handles color output
   ```

**Workflow 2: Color Conversion Pipeline (Internal)**

```
RGB Color (r, g, b)
    ↓
[Detect ColorCapability from cache]
    ↓
Match capability:
    - TrueColor    → rgb_to_truecolor_escape(r, g, b) → "\x1b[38;2;r;g;bm"
    - Ansi256      → rgb_to_ansi256(r, g, b) → index → "\x1b[38;5;{index}m"
    - Ansi16       → rgb_to_ansi16(r, g, b) → code → "\x1b[3{code}m"
    - Monochrome   → Skip color, use default
    ↓
ANSI Escape Code String
    ↓
Output to terminal via TerminalRenderer
```

**Workflow 3: Intensity-to-Color Mapping (ColorScheme.sample)**

```
Intensity (0.0-1.0)
    ↓
[Normalize to scheme gradient range]
    ↓
intensity * (colors.len() - 1) → fractional index
    ↓
Example: intensity=0.5, 5 colors → 0.5 * 4 = 2.0
    ↓
[Linear interpolation between colors[2] and colors[3]]
    ↓
Interpolated RGB Color
```

**Sequence Diagram: Applying Color Scheme to Image**

```
Developer → ImageRenderer: render_to_grid(image, color_mode=Color, scheme=fire())
ImageRenderer → Image Pipeline: Load → Resize → Grayscale
Image Pipeline → ImageRenderer: intensity_buffer (Vec<f32>)
ImageRenderer → ColorScheme: sample(intensity) for each pixel
ColorScheme → ImageRenderer: Color (r, g, b)
ImageRenderer → BrailleGrid: set_cell_color(x, y, color)
BrailleGrid → ImageRenderer: grid with colors populated
ImageRenderer → Developer: BrailleGrid (ready to render)
Developer → TerminalRenderer: render(grid)
TerminalRenderer → Terminal: ANSI escape codes + braille characters
```

## Non-Functional Requirements

### Performance

**Target: Color conversion must be negligible overhead (<100ns per color)**

From PRD NFR-P1 and Epic 5 context:
- RGB-to-ANSI256 conversion: **<100ns per color** (Story 5.2)
- RGB-to-ANSI16 conversion: **<50ns per color** (simpler algorithm)
- True color escape code generation: **<50ns** (string formatting)
- ColorScheme.sample (intensity→RGB): **<100ns per sample** (linear interpolation)
- Terminal capability detection: **<1ms** (cached after first call, one-time cost)

**Rationale:** Color system must not bottleneck the aggressive <25ms image rendering target (NFR-P1). With ~2,000 cells in an 80×24 terminal, color operations across all cells must complete in <200μs total (2000 cells × 100ns).

**Validation:**
- Benchmark suite in `benches/color_conversion.rs` (Story 5.2)
- Profile `ColorScheme.sample` with criterion (Story 5.4)
- Validate no allocations in hot paths (use flamegraph)

### Security

**Memory Safety:**
- No unsafe code in color module (Rust guarantees sufficient)
- Bounds checking on `ColorScheme.colors` vec access
- Intensity values validated (0.0-1.0 range, return error if out of bounds)

**Input Validation:**
- `ColorScheme::new()` validates non-empty colors vec (return `EmptyColorScheme` error)
- Intensity values clamped or validated before indexing
- RGB values inherently safe (u8 range 0-255)

**No Security Concerns:**
- Color system has no file I/O (except terminal env var reads)
- No network operations
- No authentication/authorization
- Library code, not exposed to untrusted inputs

### Reliability/Availability

**Error Handling:**
- All public functions return `Result<T, DotmaxError>` (no panics)
- Graceful degradation: If color detection fails, default to `ColorCapability::Ansi256` (safe fallback)
- Invalid intensity values return `InvalidIntensity` error (don't crash)
- Empty color schemes detected at creation, return `EmptyColorScheme` error

**Cross-Platform Consistency:**
- Color conversion algorithms platform-agnostic (pure math, no OS dependencies)
- Terminal capability detection handles Windows/Linux/macOS environment variables
- ANSI escape codes work consistently across modern terminals

**Robustness:**
- No unbounded allocations (ColorScheme.colors is user-provided, validated at creation)
- No memory leaks (all types are `Drop`-safe, no manual memory management)
- Thread-safe capability detection via `std::sync::OnceLock` for global cache

### Observability

**Logging (via tracing crate):**

**Story 5.1 (Capability Detection):**
```rust
#[instrument]
pub fn detect_color_capability() -> ColorCapability {
    info!("Detecting terminal color capability");
    debug!("COLORTERM={:?}, TERM={:?}", env::var("COLORTERM"), env::var("TERM"));
    // ... detection logic ...
    info!("Detected color capability: {:?}", capability);
}
```

**Story 5.2 (Color Conversion):**
```rust
// Only log at trace level (hot path, avoid overhead)
trace!("Converting RGB({}, {}, {}) to ANSI256 → {}", r, g, b, ansi_code);
```

**Story 5.4 (Color Scheme Application):**
```rust
#[instrument(skip(intensities, grid))]
pub fn apply_to_grid(&self, intensities: &[f32], grid: &mut BrailleGrid) -> Result<()> {
    info!("Applying color scheme '{}' to grid", self.name);
    debug!("Processing {} intensity values", intensities.len());
    // ... application logic ...
    info!("Color scheme applied successfully");
}
```

**Debug Utilities:**
- `ColorScheme::visualize()` method to print gradient preview (dev helper, Story 5.4)
- Example `examples/color_schemes.rs` shows all predefined schemes (visual validation)

## Dependencies and Integrations

**Rust Crates (Dependencies):**

| Crate | Version | Purpose | Epic 5 Usage | Feature Flag |
|-------|---------|---------|--------------|--------------|
| `ratatui` | 0.29 | Terminal UI framework | Color escape code rendering via TerminalRenderer | Core (no flag) |
| `crossterm` | 0.29 | Cross-platform terminal I/O | Terminal capability detection (env vars) | Core (no flag) |
| `thiserror` | 2.0 | Error handling macros | ColorError variants | Core (no flag) |
| `tracing` | 0.1 | Structured logging | Instrument color detection and conversion | Core (no flag) |

**No New Dependencies Required for Epic 5** - All functionality uses existing core dependencies.

**Integration Points:**

**1. Epic 2 (Core Rendering) - CRITICAL INTEGRATION**
- **BrailleGrid.colors field** - Already implemented in Epic 2, Epic 5 populates this field
- **TerminalRenderer** - Needs update to output ANSI color escape codes when `grid.colors.is_some()`
  - Modification in `src/render.rs`: Insert color escape codes before braille characters
  - Example: `"\x1b[38;2;255;0;0m⠿"` (red braille character)
- **Impact:** Epic 2 stories already allocated space for colors, Epic 5 adds the logic to use them

**2. Epic 3 (Image Rendering) - TIGHT INTEGRATION**
- **ImageRenderer** - Add color mode support via ColorScheme parameter
  - New method: `render_to_grid_with_colors(img, width, height, scheme) -> Result<BrailleGrid>`
  - Pipeline: Image → Grayscale → Intensity buffer → ColorScheme.sample → Colored BrailleGrid
- **Integration Story:** Story 5.5 (Apply color schemes to intensity buffers)
- **Impact:** Enables `FR17: Developers can render images in color mode`

**3. Epic 4 (Drawing Primitives) - MODERATE INTEGRATION**
- **Drawing functions** - Add `_colored` variants or optional Color parameter
  - `draw_line_colored(grid, x1, y1, x2, y2, color)`
  - `draw_circle_colored(grid, cx, cy, radius, color)`
- **Integration Story:** Story 4.5 (Add color support for drawing primitives) - depends on Epic 5
- **Impact:** Enables `FR27: Developers can set color for drawing operations`

**4. Epic 6 (Animation) - FUTURE INTEGRATION**
- **AnimationRenderer** - Reuses BrailleGrid with colors, no additional work
- **FrameBuffer** - Color data reused across frames (buffer reuse pattern from architecture)
- **Impact:** Color schemes work automatically in animations

**Dependency Graph:**
```
Epic 5 (Color System)
    ↓ provides Color, ColorScheme, ColorCapability
    ├─→ Epic 2 (Core) - Uses BrailleGrid.colors, TerminalRenderer outputs color codes
    ├─→ Epic 3 (Image) - Uses ColorScheme for colored image rendering
    ├─→ Epic 4 (Primitives) - Uses Color for colored drawing operations
    └─→ Epic 6 (Animation) - Inherits color support from Epic 2/3/4
```

**External System Interfaces:**
- **Terminal Environment Variables:** `$COLORTERM`, `$TERM` (read-only, OS-provided)
- **ANSI Escape Codes:** Standard terminal protocol (output-only, universal compatibility)

## Acceptance Criteria (Authoritative)

**AC1: Terminal Color Capability Detection Works Across All Platforms**
- `detect_color_capability()` function returns correct `ColorCapability` enum value
- Detection checks `$COLORTERM` and `$TERM` environment variables
- Result is cached (only detects once per session via `std::sync::OnceLock`)
- Unit tests verify detection logic with mocked environment variables
- Example `examples/color_detection.rs` runs and displays detected capability
- **Verification:** Run on Windows/Linux/macOS, verify correct capability reported

**AC2: RGB-to-ANSI256 Conversion is Accurate and Fast**
- `rgb_to_ansi256(r, g, b)` returns closest ANSI 256 palette index (0-255)
- Uses Euclidean distance in RGB space for color matching
- Benchmark shows <100ns per conversion (criterion.rs test)
- Unit tests verify known conversions:
  - RGB(255, 0, 0) → ANSI 196 (bright red)
  - RGB(128, 128, 128) → appropriate gray
  - RGB(0, 0, 0) → ANSI 16 (black)
- **Verification:** Run `cargo bench` and check `benches/color_conversion.rs` results

**AC3: RGB-to-ANSI16 Conversion Provides Basic Color Support**
- `rgb_to_ansi16(r, g, b)` maps RGB to basic 16 ANSI colors (0-15)
- Simple thresholding algorithm for R/G/B channels
- Benchmark shows <50ns per conversion
- Unit tests verify primary colors map correctly
- **Verification:** Test output on limited terminals (e.g., basic Windows CMD)

**AC4: True Color Escape Code Generation is Correct**
- `rgb_to_truecolor_escape(r, g, b)` returns correct ANSI escape sequence
- Format: `"\x1b[38;2;{r};{g};{b}m"` for foreground color
- Unit tests verify string formatting
- **Verification:** Visual test in true color terminal (Alacritty, WezTerm)

**AC5: Smart Color Conversion Adapts to Terminal Capability**
- `rgb_to_terminal_color(r, g, b, capability)` chooses correct conversion method
- Routes to `rgb_to_truecolor_escape()` for TrueColor terminals
- Routes to `rgb_to_ansi256()` for Ansi256 terminals
- Routes to `rgb_to_ansi16()` for Ansi16 terminals
- Returns empty string for Monochrome terminals
- **Verification:** Integration tests with mocked capabilities

**AC6: Predefined Color Schemes Extracted from crabmusic**
- At least 6 predefined schemes available: `fire()`, `ocean()`, `forest()`, `sunset()`, `grayscale()`, `neon()`
- Each scheme returns valid `ColorScheme` with 3+ colors
- Visual comparison with crabmusic output confirms color accuracy
- Example `examples/color_schemes.rs` displays all schemes
- **Verification:** Visual inspection of example output, compare to crabmusic

**AC7: Custom Color Scheme Creation API Works**
- `ColorScheme::new(name, colors)` accepts name and Vec<Color>
- Returns `EmptyColorScheme` error if colors vec is empty
- Created scheme can be used with `sample()` method
- Unit tests verify custom scheme creation and sampling
- **Verification:** Create custom scheme in test, verify gradient interpolation

**AC8: Intensity-to-Color Mapping via ColorScheme.sample is Accurate**
- `ColorScheme::sample(intensity)` returns interpolated color for intensity 0.0-1.0
- Linear interpolation between gradient colors
- Benchmark shows <100ns per sample
- Unit tests verify:
  - intensity=0.0 → colors[0]
  - intensity=1.0 → colors[last]
  - intensity=0.5 → interpolated midpoint
- **Verification:** Run unit tests, check benchmark results

**AC9: Apply Color Scheme to Intensity Buffer Populates BrailleGrid.colors**
- `ColorScheme::apply_to_grid(intensities, grid)` populates `grid.colors` field
- Each intensity value (0.0-1.0) maps to a Color via `sample()`
- `grid.colors` vec matches grid dimensions (width × height cells)
- Returns `InvalidIntensity` error for out-of-range values
- Integration test verifies colored grid renders correctly
- **Verification:** Integration test with ImageRenderer color mode

**AC10: TerminalRenderer Outputs Color Escape Codes**
- `TerminalRenderer::render(grid)` outputs ANSI color codes when `grid.colors.is_some()`
- Color escape code inserted before each braille character
- Respects detected terminal capability (uses appropriate conversion)
- No color codes output if `grid.colors.is_none()` (monochrome mode)
- **Verification:** Visual test with colored BrailleGrid, inspect terminal output

**AC11: Integration with Epic 3 (Image Color Mode) Works**
- `ImageRenderer` can render images with color via `ColorScheme`
- Color mode images use intensity buffer + ColorScheme.sample pipeline
- Colored images render correctly with all 6+ predefined schemes
- **Verification:** Story 5.5 integration test, visual comparison

**AC12: Integration with Epic 4 (Colored Primitives) API Ready**
- `BrailleGrid::set_cell_color(x, y, color)` sets individual cell colors
- Drawing primitives can use this API for colored shapes
- Unit tests verify cell color setting and retrieval
- **Verification:** Story 4.5 will validate end-to-end colored drawing

## Traceability Mapping

| AC # | PRD Functional Requirement | Spec Section | Component/API | Story | Test |
|------|----------------------------|--------------|---------------|-------|------|
| AC1 | FR37: Query terminal color capabilities | Terminal Capability Detection API | `src/utils/terminal_caps.rs` → `detect_color_capability()` | 5.1 | Unit tests (mocked env vars), manual cross-platform test |
| AC2 | FR33: Convert RGB to terminal-compatible codes (256) | RGB-to-ANSI Conversion API | `src/color/convert.rs` → `rgb_to_ansi256()` | 5.2 | Unit tests (known conversions), benchmark (<100ns) |
| AC3 | FR33: Convert RGB to terminal-compatible codes (16) | RGB-to-ANSI Conversion API | `src/color/convert.rs` → `rgb_to_ansi16()` | 5.2 | Unit tests (primary colors), benchmark (<50ns) |
| AC4 | FR33: Convert RGB to terminal-compatible codes (true color) | RGB-to-ANSI Conversion API | `src/color/convert.rs` → `rgb_to_truecolor_escape()` | 5.2 | Unit tests (escape format), visual test |
| AC5 | FR37: Adjust rendering to terminal capabilities | RGB-to-ANSI Conversion API | `src/color/convert.rs` → `rgb_to_terminal_color()` | 5.2 | Integration tests (capability routing) |
| AC6 | FR34: Predefined color schemes (6+ built-in) | Color Scheme API | `src/color/schemes.rs` → `fire()`, `ocean()`, etc. | 5.3 | Visual test (compare to crabmusic), example output |
| AC7 | FR35: Create custom color schemes | Color Scheme API | `src/color/schemes.rs` → `ColorScheme::new()` | 5.4 | Unit tests (creation, validation) |
| AC8 | FR36: Apply color schemes to intensity buffers | Color Scheme API | `src/color/schemes.rs` → `ColorScheme::sample()` | 5.4 | Unit tests (interpolation), benchmark (<100ns) |
| AC9 | FR36: Apply color schemes to grayscale buffers | Color Scheme API | `src/color/apply.rs` → `apply_to_grid()` | 5.5 | Integration test (intensity→colored grid) |
| AC10 | FR32: Assign RGB colors to braille cells | Integration with BrailleGrid | `src/render.rs` → `TerminalRenderer::render()` update | 5.5 | Visual test (colored output), terminal inspection |
| AC11 | FR17: Render images in color mode | Integration with Epic 3 | `src/image/mod.rs` → `render_to_grid_with_colors()` | 5.5 | Integration test (colored image rendering) |
| AC12 | FR27: Set color for drawing operations | Integration with Epic 4 | `src/grid.rs` → `set_cell_color()` | 5.5 | Unit test (cell color set/get), deferred to Story 4.5 |

**Cross-Reference to PRD Non-Functional Requirements:**

| NFR | Spec Section | Validation Method |
|-----|--------------|-------------------|
| NFR-P1: <50ms image rendering | Performance (color conversion <100ns per cell) | Benchmark suite (`benches/color_conversion.rs`) |
| NFR-R1: Zero panics, Result types | Reliability/Availability (error handling) | All public APIs return `Result<T, DotmaxError>` |
| NFR-R2: Cross-platform consistency | Reliability/Availability (platform handling) | CI testing on Windows/Linux/macOS |
| NFR-M3: No compiler warnings | Code quality | Clippy in CI (`-D warnings`) |
| NFR-D1: Minimal core dependencies | Dependencies section | No new dependencies for Epic 5 |
| NFR-DX1: API simplicity | APIs and Interfaces section | Examples demonstrate <100 LOC usage |

## Risks, Assumptions, Open Questions

**RISK-1: Terminal Color Capability Detection May Be Unreliable**
- **Type:** Risk (Implementation)
- **Impact:** Medium - Incorrect detection leads to broken color output or missing colors
- **Mitigation:**
  - Check multiple sources: `$COLORTERM`, `$TERM`, terminal queries
  - Default to safe fallback (Ansi256) if detection fails
  - Provide manual override API for users to force capability level
  - Test across diverse terminals (PowerShell, CMD, WSL, iTerm2, Alacritty)
- **Owner:** Story 5.1

**RISK-2: RGB-to-ANSI256 Color Matching May Have Visual Artifacts**
- **Type:** Risk (Quality)
- **Impact:** Low - Colors may not match exactly, but are "close enough"
- **Mitigation:**
  - Use Euclidean distance in RGB space (industry standard)
  - Visual comparison with crabmusic output (proven quality baseline)
  - Unit tests verify primary colors map correctly
  - Benchmark ensures performance (<100ns) doesn't compromise accuracy
- **Owner:** Story 5.2

**RISK-3: Color Scheme Extraction from crabmusic May Lose Fidelity**
- **Type:** Risk (Quality)
- **Impact:** Medium - Color schemes may not look as good as in crabmusic
- **Mitigation:**
  - Extract exact RGB values from crabmusic code (no approximation)
  - Visual comparison test (side-by-side with crabmusic output)
  - Get user (Frosty) validation on color accuracy
- **Owner:** Story 5.3

**ASSUMPTION-1: Most Modern Terminals Support 256-Color or True Color**
- **Type:** Assumption (Environment)
- **Validation:** Industry standard since ~2010, 99%+ of modern terminals support
- **If False:** Graceful degradation to ANSI 16 or monochrome works
- **Impact:** Low - Fallback mechanisms handle limited terminals

**ASSUMPTION-2: Terminal Capability Detection via Environment Variables is Sufficient**
- **Type:** Assumption (Implementation)
- **Validation:** Standard practice in terminal libraries (crossterm, termion)
- **If False:** Add terminal query fallback (ANSI escape code probing)
- **Impact:** Low - Additional detection mechanism adds complexity but is feasible

**ASSUMPTION-3: Linear Interpolation is Sufficient for ColorScheme.sample**
- **Type:** Assumption (Implementation)
- **Validation:** Used in crabmusic, produces good visual results
- **If False:** Add alternative interpolation methods (cubic, spline) as enhancement
- **Impact:** Low - Linear interpolation meets MVP requirements

**ASSUMPTION-4: Epic 2 BrailleGrid.colors Field Already Exists**
- **Type:** Assumption (Dependency)
- **Validation:** Confirmed in architecture.md and Epic 2 implementation
- **If False:** Epic 5 implementation blocked until Epic 2 provides field
- **Impact:** High - Critical dependency, but already validated as complete

**QUESTION-1: Should Color Schemes Support Background Colors?**
- **Status:** Open
- **Context:** Current design only handles foreground colors (braille characters)
- **Options:**
  - MVP: Foreground only (simpler, matches crabmusic)
  - Enhancement: Add background color support (more flexibility)
- **Decision Needed By:** Story 5.4 (ColorScheme API design)
- **Recommendation:** Foreground only for MVP, defer background to post-1.0

**QUESTION-2: Should We Support HSL/HSV Color Spaces?**
- **Status:** Open
- **Context:** RGB-only for MVP, but HSL/HSV may be easier for users to create schemes
- **Options:**
  - MVP: RGB only (simpler, matches architecture)
  - Enhancement: Add HSL/HSV conversion helpers
- **Decision Needed By:** Story 5.4 (custom scheme creation)
- **Recommendation:** RGB only for MVP, add conversion helpers if user feedback requests it

**QUESTION-3: How Should Color Conversion Handle Out-of-Gamut Colors?**
- **Status:** Open
- **Context:** Some RGB colors can't be exactly represented in ANSI palettes
- **Options:**
  - Clamp to nearest available color (current approach)
  - Dithering for better visual approximation (complex)
  - Warn user about gamut issues (too noisy)
- **Decision Needed By:** Story 5.2 (color conversion implementation)
- **Recommendation:** Clamp to nearest (simple, proven), revisit if visual quality issues reported

## Test Strategy Summary

**Test Levels:**

**1. Unit Tests (src/color/*.rs, src/utils/terminal_caps.rs)**
- **Coverage Target:** >80% line coverage for color module
- **Critical Paths:** 100% coverage for color conversion functions
- **Key Tests:**
  - `detect_color_capability()` with mocked environment variables
  - `rgb_to_ansi256()`, `rgb_to_ansi16()`, `rgb_to_truecolor_escape()` with known values
  - `ColorScheme::new()` validation (empty colors vec error)
  - `ColorScheme::sample()` interpolation (boundary and midpoint values)
  - `apply_to_grid()` intensity validation (out-of-range errors)
- **Framework:** Rust `#[cfg(test)]` modules, `#[test]` attribute
- **Run:** `cargo test --lib`

**2. Integration Tests (tests/color_integration_tests.rs)**
- **Scope:** End-to-end color system workflows
- **Key Tests:**
  - Terminal capability detection → color conversion → escape code output
  - Image intensity buffer → ColorScheme.sample → colored BrailleGrid → render
  - Custom scheme creation → apply to grid → verify colors set
  - TerminalRenderer outputs color codes when grid.colors exists
- **Framework:** Rust integration tests (tests/ directory)
- **Run:** `cargo test --test color_integration_tests`

**3. Benchmark Tests (benches/color_conversion.rs)**
- **Performance Validation:**
  - `rgb_to_ansi256()` < 100ns per conversion
  - `rgb_to_ansi16()` < 50ns per conversion
  - `rgb_to_truecolor_escape()` < 50ns per escape code
  - `ColorScheme::sample()` < 100ns per sample
- **Framework:** criterion.rs with HTML reports
- **Run:** `cargo bench --bench color_conversion`
- **CI Integration:** Performance regression detection (>10% slower = fail)

**4. Visual/Manual Tests (examples/)**
- **Examples Required:**
  - `examples/color_detection.rs` - Display detected terminal capability
  - `examples/color_schemes.rs` - Show all 6+ predefined schemes
  - `examples/color_conversion.rs` - Demo RGB → ANSI conversion
  - `examples/colored_image.rs` - Render image with color scheme (Story 5.5)
- **Validation:**
  - Visual inspection on multiple terminals (Alacritty, WezTerm, Windows Terminal, iTerm2, basic CMD)
  - Compare to crabmusic output (color scheme fidelity)
  - User (Frosty) approval on visual quality

**5. Cross-Platform Tests (CI: GitHub Actions)**
- **Platforms:** Windows, Linux, macOS
- **Matrix:**
  - `os: [ubuntu-latest, windows-latest, macos-latest]`
  - `rust: [stable, 1.70]` (stable + MSRV)
- **Tests Run:**
  - Unit tests
  - Integration tests
  - Clippy (no warnings)
  - rustfmt (code formatting)
- **Color-Specific Validation:**
  - Environment variable handling across platforms
  - ANSI escape code compatibility
  - Terminal capability detection consistency

**Test Data:**
- **Sample RGB values:** Primary colors (red, green, blue), grayscale (0-255), edge cases (0, 255)
- **Sample intensity buffers:** Uniform (all 0.0, all 1.0), gradient (0.0→1.0), checkerboard pattern
- **Sample color schemes:** All 6+ predefined schemes + custom schemes (2-color, 10-color)
- **Terminal environments:** Mocked env vars for Monochrome, Ansi16, Ansi256, TrueColor

**Coverage Metrics:**
- **Unit test coverage:** >80% for color module (measured with cargo-tarpaulin if available)
- **Critical path coverage:** 100% for conversion functions
- **Integration test coverage:** All 12 acceptance criteria validated
- **Visual test coverage:** All predefined schemes + colored image rendering

**Test Execution Order:**
1. Unit tests (fast, run on every commit)
2. Integration tests (moderate, run on every commit)
3. Benchmark tests (slow, run on PR + main branch)
4. Visual tests (manual, run before story completion)
5. Cross-platform tests (CI, run on PR + main branch)

**Regression Testing:**
- Benchmark suite tracks performance over time (criterion.rs baselines)
- Visual regression: Save example outputs, compare on changes (manual review)
- Unit/integration tests prevent functional regressions (automated)

**Edge Cases to Test:**
- Empty color scheme (should error)
- Single-color scheme (sample returns that color for all intensities)
- Intensity out of range (< 0.0 or > 1.0, should error)
- Monochrome terminal (should skip color codes)
- Very large color schemes (100+ colors, should still interpolate correctly)
- Terminal capability detection failure (should default to safe fallback)
