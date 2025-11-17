# dotmax - Epic Breakdown

**Author:** Frosty
**Date:** 2025-11-14
**Project Level:** Medium-High (Brownfield Extraction + Greenfield Packaging)
**Target Scale:** MVP - Core Rendering + 2D Image (v0.1.0 → v1.0)

---

## Overview

This document provides the complete epic and story breakdown for dotmax, decomposing the requirements from the [PRD](./PRD.md) into implementable stories.

**Living Document Notice:** This is the initial version. It will be updated after UX Design and Architecture workflows add interaction and technical details to stories.

### Epic Summary

**7 Epics** organized by value delivery and technical dependencies:

1. **Foundation & Project Setup** - Establish crate structure, CI/CD, development environment (14 FRs)
2. **Core Braille Rendering Engine** - Extract/clean BrailleGrid system from crabmusic (23 FRs)
3. **2D Image Rendering Pipeline** - Enable image-to-braille conversion with full format support (12 FRs)
4. **Drawing Primitives & Density Rendering** - Programmatic drawing capabilities (11 FRs)
5. **Color System & Visual Schemes** - RGB support and terminal color mapping (8 FRs)
6. **Animation & Frame Management** - Motion and real-time updates (6 FRs)
7. **API Design, Performance & Production Readiness** - Polish, optimize, and ship (26 FRs)

**Sequencing**: Foundation → Core Engine → Images/Primitives → Color → Animation → Production

All 90 functional requirements from PRD are covered across these epics.

---

## Functional Requirements Inventory

### Core Rendering (BrailleGrid System) - 8 FRs
- **FR1**: Create BrailleGrid with specified dimensions
- **FR2**: Set individual dots within grid (2×4 dot matrix)
- **FR3**: Clear entire grid or specific regions
- **FR4**: Render grid to terminal via ratatui/crossterm
- **FR5**: Convert dot patterns to Unicode braille (U+2800-U+28FF)
- **FR6**: Query grid state (get dots, dimensions, buffer)
- **FR7**: Handle terminal resize events
- **FR8**: Create grids with per-cell color support

### 2D Image Rendering - 12 FRs
- **FR9**: Load/render images from file paths (PNG, JPG, GIF, BMP, WebP, TIFF)
- **FR10**: Load/render images from byte buffers
- **FR11**: Load/render SVG vector graphics with rasterization
- **FR12**: Auto-resize images to terminal dimensions (preserve aspect ratio)
- **FR13**: Specify target dimensions (override auto-sizing)
- **FR14**: Convert grayscale/color to braille using threshold algorithms
- **FR15**: Select dithering method (Floyd-Steinberg, Bayer, Atkinson, none)
- **FR16**: Render in monochrome mode
- **FR17**: Render in color mode
- **FR18**: Apply Otsu thresholding for optimal binary conversion
- **FR19**: Adjust brightness/contrast/gamma before rendering
- **FR20**: Handle malformed/unsupported files gracefully

### Drawing Primitives - 7 FRs
- **FR21**: Draw lines using Bresenham algorithm
- **FR22**: Draw circles using Bresenham algorithm
- **FR23**: Draw rectangles (outline/filled)
- **FR24**: Draw polygons from vertex lists
- **FR25**: Fill regions with solid/density patterns
- **FR26**: Set line thickness
- **FR27**: Set color for drawing operations

### Character Density Rendering - 4 FRs
- **FR28**: Map intensity (0.0-1.0) to character densities
- **FR29**: Use predefined character density sets
- **FR30**: Customize character density mappings
- **FR31**: Provide smooth gradients via density selection

### Color Support - 6 FRs
- **FR32**: Assign RGB colors to individual cells
- **FR33**: Convert RGB to terminal color codes (ANSI 256/true color)
- **FR34**: Select from 6+ predefined color schemes
- **FR35**: Create custom color schemes
- **FR36**: Apply color schemes to grayscale intensity buffers
- **FR37**: Query terminal color capabilities

### Animation & Frame Management - 6 FRs
- **FR38**: Render frame-by-frame animations
- **FR39**: Frame timing control
- **FR40**: Create animation loops with frame rates
- **FR41**: Frame buffer management for smooth transitions
- **FR42**: Pre-render frames for optimization
- **FR43**: Clear previous frames (flicker-free)

### API Design & Integration - 7 FRs
- **FR44**: Integrate with <100 lines for basic use
- **FR45**: Follow Rust idioms (Result types, ownership)
- **FR46**: Builder patterns for complex config
- **FR47**: Async/await compatibility
- **FR48**: Examples for common scenarios
- **FR49**: Optional features via Cargo flags
- **FR50**: Minimal, focused API surface

### Terminal Abstraction - 5 FRs
- **FR51**: Work with ratatui/crossterm
- **FR52**: Support custom terminal backends
- **FR53**: Detect terminal capabilities
- **FR54**: Graceful fallback for limited terminals
- **FR55**: Work in standard terminals (PowerShell, bash, zsh, etc.)

### Error Handling & Robustness - 5 FRs
- **FR56**: All operations return Result with meaningful errors
- **FR57**: Never panic in library code
- **FR58**: Query error details for debugging
- **FR59**: Handle edge cases safely
- **FR60**: Debug/trace logging support

### Library Distribution & Packaging - 7 FRs
- **FR61**: Install via `cargo add dotmax`
- **FR62**: Minimize binary size (<2MB)
- **FR63**: Minimal core dependencies
- **FR64**: Feature flags for optional capabilities
- **FR65**: Compile on stable Rust
- **FR66**: API docs via rustdoc with examples
- **FR67**: Examples directory with runnable demos

### Performance & Efficiency - 7 FRs
- **FR68**: Render images <50ms (standard terminals 80×24)
- **FR69**: Render images <100ms (large terminals 200×50)
- **FR70**: 60fps animation with <10% CPU
- **FR71**: <5MB baseline memory
- **FR72**: <500KB per frame overhead
- **FR73**: Prompt memory release (no unbounded growth)
- **FR74**: <5ms initialization (cold start)

### Cross-Platform Compatibility - 6 FRs
- **FR75**: Work on Windows without modification
- **FR76**: Work on Linux without modification
- **FR77**: Work on macOS without modification
- **FR78**: Handle platform-specific terminal quirks
- **FR79**: Consistent visual output across platforms
- **FR80**: Detect/adapt to platform terminal capabilities

### Testing & Validation - 5 FRs
- **FR81**: Unit tests for core rendering
- **FR82**: Integration tests for image loading/rendering
- **FR83**: Benchmark tests (criterion.rs)
- **FR84**: Visual regression tests
- **FR85**: Property-based tests for mathematical correctness

### Documentation & Developer Experience - 5 FRs
- **FR86**: Comprehensive API docs with examples
- **FR87**: Quickstart guide in README
- **FR88**: Migration guide from similar libraries
- **FR89**: Architecture Decision Records (ADRs)
- **FR90**: Troubleshooting guide

**Total: 90 Functional Requirements**

---

## FR Coverage Map

### Epic-to-FR Mapping

**Epic 1: Foundation & Project Setup**
- Covers: FR61-67 (Library Distribution & Packaging), FR75-80 (Cross-Platform Compatibility), FR65 (Stable Rust compilation)
- Total: 14 FRs

**Epic 2: Core Braille Rendering Engine**
- Covers: FR1-8 (Core Rendering/BrailleGrid System), FR51-60 (Terminal Abstraction + Error Handling)
- Total: 23 FRs

**Epic 3: 2D Image Rendering Pipeline**
- Covers: FR9-20 (2D Image Rendering - all formats, algorithms, modes)
- Total: 12 FRs

**Epic 4: Drawing Primitives & Density Rendering**
- Covers: FR21-27 (Drawing Primitives), FR28-31 (Character Density Rendering)
- Total: 11 FRs

**Epic 5: Color System & Visual Schemes**
- Covers: FR32-37 (Color Support), enhances FR8, FR17, FR27 from previous epics
- Total: 8 FRs (6 new + 2 enhancements)

**Epic 6: Animation & Frame Management**
- Covers: FR38-43 (Animation & Frame Management)
- Total: 6 FRs

**Epic 7: API Design, Performance & Production Readiness**
- Covers: FR44-50 (API Design & Integration), FR66-67 (Documentation), FR68-74 (Performance & Efficiency), FR81-90 (Testing & Validation + Documentation)
- Total: 26 FRs

**Coverage Validation**: All 90 unique functional requirements mapped to epics ✓

---

## Epic 1: Foundation & Project Setup

**Goal**: Establish professional Rust crate structure, CI/CD pipeline, and development environment that enables all subsequent development work. This epic transforms dotmax from concept to a working, testable, cross-platform Rust project.

**Value Delivered**: Developers can clone and build immediately; CI/CD catches bugs across platforms; dependency strategy prevents upstream breakage; licensing enables broad adoption; foundation supports solo long-term maintenance.

**FRs Covered**: FR61-67, FR75-80, FR65 (14 FRs)

---

### Story 1.1: Initialize Cargo Project with Optimal Structure

As a **Rust library developer**,
I want a properly initialized Cargo project with clean structure and metadata,
So that the crate is ready for professional development and crates.io publication.

**Acceptance Criteria:**

**Given** an empty project directory
**When** I run `cargo new --lib dotmax`
**Then** the project initializes with:
- `Cargo.toml` with complete metadata (name, version 0.1.0, authors, edition 2021, description, license, repository, keywords, categories)
- `src/lib.rs` as entry point with module structure planning
- `.gitignore` configured for Rust (target/, Cargo.lock for libs)
- Directory structure: `src/`, `examples/`, `tests/`, `benches/`, `docs/`
- `README.md` with placeholder sections (Installation, Quick Start, Features, License)
- `LICENSE` files (MIT and Apache-2.0 dual licensing)
- `CHANGELOG.md` initialized with v0.1.0 (unreleased)

**And** `Cargo.toml` includes metadata:
- `rust-version = "1.70"` (MSRV documented)
- `keywords = ["terminal", "braille", "graphics", "cli", "visualization"]`
- `categories = ["command-line-interface", "graphics", "rendering"]`

**And** project compiles cleanly with `cargo build` and `cargo test` passes (empty test suite)

**Prerequisites:** None (first story)

**Technical Notes:**
- Use `cargo init` or `cargo new --lib dotmax`
- Set edition = "2021" for latest Rust idioms
- Dual license (MIT OR Apache-2.0) maximizes adoption
- MSRV 1.70 balances modern features vs. compatibility
- Structure follows Rust API Guidelines

---

### Story 1.2: Configure GitHub Actions CI/CD Pipeline

As a **solo developer maintaining dotmax long-term**,
I want automated CI/CD that tests across Windows, Linux, and macOS,
So that platform-specific bugs are caught immediately without manual testing.

**Acceptance Criteria:**

**Given** a GitHub repository with Cargo project
**When** I push code to any branch
**Then** GitHub Actions automatically runs:
- `cargo build` on matrix: [windows-latest, ubuntu-latest, macos-latest]
- `cargo test` on all three platforms
- `cargo clippy -- -D warnings` (enforce linting)
- `cargo fmt --check` (enforce formatting)
- `cargo audit` (security vulnerability scanning)

**And** CI fails if any check fails on any platform

**And** CI completes in <5 minutes for clean builds

**And** `.github/workflows/ci.yml` includes:
- Rust stable toolchain (auto-updated)
- MSRV check (rust-version: 1.70)
- Cache for faster builds (`Swatinem/rust-cache@v2`)

**Prerequisites:** Story 1.1 (Cargo project exists)

**Technical Notes:**
- Use GitHub Actions (free for public repos)
- Matrix strategy for cross-platform testing
- `cargo-audit` detects dependency vulnerabilities
- Cache `~/.cargo` and `target/` for speed
- Separate workflow for releases (later story)

---

### Story 1.3: Define Core Dependencies with Feature Flags

As a **library developer focused on minimal bloat**,
I want carefully selected core dependencies with optional features behind flags,
So that users only pay for what they use (binary size <2MB core).

**Acceptance Criteria:**

**Given** `Cargo.toml` exists
**When** I configure dependencies
**Then** core dependencies (no feature flags) include:
- `ratatui = "0.26"` (or latest) - Terminal abstraction
- `crossterm = "0.27"` (or latest) - Cross-platform terminal I/O
- No other required dependencies in MVP

**And** optional dependencies behind feature flags:
```toml
[features]
default = []
image = ["dep:image", "dep:imageproc"]
svg = ["dep:resvg", "dep:usvg"]

[dependencies]
image = { version = "0.25", optional = true }
imageproc = { version = "0.24", optional = true }
resvg = { version = "0.38", optional = true }
usvg = { version = "0.38", optional = true }
```

**And** `cargo build` (no flags) compiles successfully with only ratatui + crossterm

**And** `cargo build --features image,svg` enables image rendering

**And** each dependency is documented with justification in `docs/dependencies.md`

**Prerequisites:** Story 1.1 (Cargo.toml exists)

**Technical Notes:**
- Feature flags prevent bloat (FR62, FR64)
- Pin major versions to prevent breaking changes
- `optional = true` excludes from default builds
- Later epics will add video/raytrace features
- Document why each dependency was chosen (ADR)

---

### Story 1.4: Set Up Code Quality Tooling (Clippy, Rustfmt, Deny)

As a **developer maintaining clean, idiomatic Rust code**,
I want automated linting, formatting, and policy enforcement,
So that code quality remains high even after long gaps in development.

**Acceptance Criteria:**

**Given** a Cargo project with CI configured
**When** I configure quality tools
**Then** the project includes:

**Clippy Configuration** (`clippy.toml` or `Cargo.toml`):
```toml
[lints.clippy]
all = "deny"
pedantic = "warn"
nursery = "warn"
cargo = "warn"
```

**Rustfmt Configuration** (`.rustfmt.toml`):
```toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4
```

**Cargo Deny Configuration** (`.deny.toml`):
- Block GPL/AGPL licenses in dependencies
- Detect duplicate dependencies
- Enforce security advisory scanning

**And** `cargo clippy` runs without warnings

**And** `cargo fmt --check` passes (code is formatted)

**And** `cargo deny check` passes (licenses, advisories, duplicates)

**And** CI enforces all three checks on every PR

**Prerequisites:** Story 1.2 (CI exists)

**Technical Notes:**
- Clippy catches common bugs and anti-patterns
- Rustfmt ensures consistent style
- `cargo-deny` prevents license/security issues
- Pedantic warnings improve code quality
- All checks must pass for CI to succeed

---

### Story 1.5: Create Architecture Decision Records (ADR) System

As a **solo developer who may resume work after months**,
I want documented architecture decisions with context and rationale,
So that I understand WHY choices were made when I return to the project.

**Acceptance Criteria:**

**Given** a project with design decisions being made
**When** I create the ADR system
**Then** `docs/adr/` directory exists with structure:
- `docs/adr/README.md` - ADR index and guidelines
- `docs/adr/template.md` - Standard ADR template
- `docs/adr/0001-use-braille-unicode-for-rendering.md` - First ADR

**And** ADR template includes sections:
1. **Status** (Proposed, Accepted, Deprecated, Superseded)
2. **Context** (Problem being solved, constraints)
3. **Decision** (What was decided)
4. **Consequences** (Trade-offs, implications)
5. **Alternatives Considered** (What was rejected and why)

**And** First ADR (0001) documents:
- **Decision**: Use Unicode braille (U+2800-U+28FF) for terminal rendering
- **Context**: Need cross-platform terminal graphics without protocol dependencies
- **Consequences**: 2×4 resolution per cell, requires Unicode support, text-based output
- **Alternatives**: Sixel (limited support), Kitty protocol (tmux breaks), ASCII art (lower resolution)

**And** `docs/adr/README.md` maintains index of all ADRs with quick summaries

**Prerequisites:** Story 1.1 (docs/ directory exists)

**Technical Notes:**
- ADRs are immutable (new ADRs supersede old ones, don't edit)
- Number format: 0001, 0002, etc. (sortable)
- Keep ADRs concise (1-2 pages max)
- Link ADRs from code comments where relevant
- Critical for NFR-M2 (resumable development)

---

### Story 1.6: Set Up Benchmarking Infrastructure (Criterion.rs)

As a **developer for whom performance is make-or-break**,
I want a benchmarking system that measures and tracks performance over time,
So that I can validate <50ms renders and catch regressions immediately.

**Acceptance Criteria:**

**Given** a Cargo project
**When** I configure benchmarking
**Then** `Cargo.toml` includes:
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "rendering"
harness = false
```

**And** `benches/rendering.rs` exists with placeholder benchmarks:
- `bench_braille_grid_creation` - Measure grid initialization
- `bench_grid_clear` - Measure clear operation
- `bench_unicode_conversion` - Measure dot pattern → Unicode conversion

**And** `cargo bench` runs successfully and generates:
- Console output with timing results
- HTML reports in `target/criterion/`
- Comparison against baseline (if previous run exists)

**And** `.github/workflows/benchmark.yml` tracks performance in CI:
- Runs benchmarks on main branch pushes
- Stores results as artifacts
- Comments on PRs if performance regresses >10%

**Prerequisites:** Story 1.1 (Cargo project), Story 1.2 (CI)

**Technical Notes:**
- Criterion.rs is standard for Rust benchmarking
- `harness = false` required for criterion
- HTML reports show graphs and statistical analysis
- Store baseline results in git (criterion/baseline/)
- Later epics will add image/animation benchmarks
- Critical for FR68-74 (performance requirements)

---

### Story 1.7: Create Example Template and Documentation Structure

As a **developer creating a library with excellent DX**,
I want example code that demonstrates usage and compiles in CI,
So that users can start using dotmax in <5 minutes.

**Acceptance Criteria:**

**Given** a Cargo project with examples/ directory
**When** I create the example structure
**Then** the following examples exist:

**`examples/hello_braille.rs`** - Minimal example:
```rust
// Renders "Hello" using braille dots
// Demonstrates: BrailleGrid creation, dot setting, terminal rendering
// Expected output: ASCII art-style braille text
// Lines of code: <50
```

**`examples/README.md`** - Example index:
- Lists all examples with descriptions
- Shows how to run: `cargo run --example hello_braille`
- Notes which features required (e.g., `--features image`)

**And** `README.md` includes:
- **Installation**: `cargo add dotmax`
- **Quick Start**: Code snippet from hello_braille example
- **Features**: Table of feature flags (image, svg, video, raytrace)
- **Examples**: Link to examples/README.md
- **Documentation**: Link to docs.rs (future)
- **License**: MIT OR Apache-2.0

**And** `cargo run --example hello_braille` executes successfully

**And** CI runs `cargo build --examples` to ensure examples always compile

**Prerequisites:** Story 1.1 (project structure)

**Technical Notes:**
- Examples are integration tests for public API
- Keep examples minimal and focused (one concept each)
- Add more examples in later epics (images, animation, etc.)
- Examples enforce API usability (FR44: <100 lines)
- Critical for FR66-67, FR87 (docs/examples)

---

## Epic 2: Core Braille Rendering Engine

**Goal**: Extract and professionalize the battle-tested BrailleGrid system from crabmusic (~2,000-3,000 lines), creating the foundational rendering engine that converts dot patterns to Unicode braille characters and outputs to terminals. This is the atomic unit - everything else builds on this.

**Value Delivered**: Developers can create braille grids, set individual dots, clear regions, query state, handle terminal events, and render to any terminal via ratatui/crossterm with robust error handling.

**FRs Covered**: FR1-8, FR51-60 (23 FRs)

---

### Story 2.1: Extract BrailleGrid Core from Crabmusic

As a **developer extracting proven code from crabmusic**,
I want the BrailleGrid data structure with dot manipulation capabilities,
So that I have the foundational 2×4 dot matrix rendering system.

**Acceptance Criteria:**

**Given** access to crabmusic source code
**When** I extract the BrailleGrid implementation
**Then** `src/grid/braille_grid.rs` contains:
- `BrailleGrid` struct with fields: `width: usize`, `height: usize`, `dots: Vec<Vec<[bool; 8]>>`
- `new(width, height)` constructor - creates grid with dimensions
- `set_dot(x, y, dot_index, value)` - sets individual dot (0-7 index for 2×4 matrix)
- `get_dot(x, y, dot_index) -> bool` - reads dot value
- `clear()` - resets all dots to false
- `clear_region(x, y, width, height)` - clears rectangular area
- `dimensions() -> (usize, usize)` - returns grid size
- `resize(new_width, new_height)` - adjusts grid dimensions

**And** dot indexing follows Unicode braille standard:
```
Braille cell layout (8 dots):
0 3
1 4
2 5
6 7
```

**And** all methods return `Result<T, DotmaxError>` (no panics)

**And** unit tests cover:
- Grid creation with various dimensions
- Dot setting/getting for all 8 positions
- Clear operations (full and region)
- Edge cases: zero dimensions, out-of-bounds access, resize

**And** code is free of crabmusic audio dependencies

**Prerequisites:** Story 1.1 (project structure), Story 1.3 (dependencies)

**Technical Notes:**
- BrailleGrid is ~500 lines in crabmusic
- Remove audio-reactive features (stay in crabmusic)
- Use `Vec<Vec<[bool; 8]>>` for dot storage (simple, clear)
- Later optimize if benchmarks show issues
- Critical for FR1-3, FR6
- Reference: crabmusic `src/visualization/braille.rs`

---

### Story 2.2: Implement Unicode Braille Character Conversion

As a **developer rendering braille to terminals**,
I want conversion from dot patterns to Unicode braille characters (U+2800-U+28FF),
So that braille grids display correctly in any Unicode-capable terminal.

**Acceptance Criteria:**

**Given** a BrailleGrid with set dot patterns
**When** I convert cells to Unicode
**Then** `src/grid/unicode.rs` contains:
- `dots_to_braille_char(dots: [bool; 8]) -> char` - converts 8-dot array to Unicode braille
- Formula: `base_char = '\u{2800}'` + bitfield from dots
- Bitfield calculation: `dots[0] << 0 | dots[1] << 1 | dots[2] << 2 | ... | dots[7] << 7`

**And** `BrailleGrid` has method:
- `to_unicode_grid() -> Vec<Vec<char>>` - converts entire grid to 2D char array

**And** conversion is correct for all 256 braille patterns (2^8 combinations)

**And** unit tests verify:
- Empty cell (all false) → U+2800 (blank braille)
- Full cell (all true) → U+28FF (all dots)
- Specific patterns match Unicode standard
- Round-trip: dots → char → verify visually correct

**And** benchmarks show conversion <1μs per cell (criterion test)

**Prerequisites:** Story 2.1 (BrailleGrid exists)

**Technical Notes:**
- Unicode braille range: U+2800 to U+28FF (256 characters)
- Bitfield mapping is standards-based (Unicode Braille Patterns block)
- No lookup table needed - direct calculation is fast
- Critical for FR5
- Performance target: <1ms for 80×24 grid conversion

---

### Story 2.3: Implement GridBuffer and Terminal Rendering Abstraction

As a **developer rendering to diverse terminals**,
I want a buffer system that abstracts terminal I/O via ratatui/crossterm,
So that dotmax works on Windows, Linux, macOS without modification.

**Acceptance Criteria:**

**Given** a BrailleGrid converted to Unicode
**When** I render to terminal
**Then** `src/terminal/renderer.rs` contains:

**`TerminalRenderer` struct**:
- `new() -> Result<Self, DotmaxError>` - initializes terminal backend
- `render(grid: &BrailleGrid) -> Result<(), DotmaxError>` - outputs grid to terminal
- `clear() -> Result<(), DotmaxError>` - clears terminal
- `get_terminal_size() -> Result<(u16, u16), DotmaxError>` - queries dimensions

**And** rendering pipeline:
1. Convert BrailleGrid to Unicode via `to_unicode_grid()`
2. Build ratatui `Frame` with braille characters
3. Use crossterm for terminal output
4. Handle terminal resize events

**And** `src/terminal/backend.rs` provides trait:
```rust
pub trait TerminalBackend {
    fn write(&mut self, content: &str) -> Result<(), DotmaxError>;
    fn clear(&mut self) -> Result<(), DotmaxError>;
    fn size(&self) -> Result<(u16, u16), DotmaxError>;
}
```

**And** default implementation uses ratatui + crossterm

**And** rendering handles errors gracefully:
- Terminal not available → descriptive error
- Unicode not supported → warning + fallback info (not crash)
- I/O errors → wrapped in DotmaxError

**And** integration test renders 10×10 grid successfully

**Prerequisites:** Story 2.1 (BrailleGrid), Story 2.2 (Unicode conversion), Story 1.3 (ratatui/crossterm deps)

**Technical Notes:**
- Abstract terminal I/O for future custom backends (FR52)
- Ratatui handles cross-platform differences
- Crossterm provides raw terminal access
- Critical for FR4, FR51, FR53-55
- ~450 lines in crabmusic - needs extraction and cleaning

---

### Story 2.4: Implement Comprehensive Error Handling System

As a **library developer committed to zero panics**,
I want a robust error type hierarchy with meaningful messages,
So that users can debug issues and the library never crashes.

**Acceptance Criteria:**

**Given** dotmax operations that can fail
**When** I define error handling
**Then** `src/error.rs` contains:

**`DotmaxError` enum**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum DotmaxError {
    #[error("Grid operation failed: {0}")]
    GridError(String),

    #[error("Out of bounds: ({x}, {y}) in grid of size ({width}, {height})")]
    OutOfBounds { x: usize, y: usize, width: usize, height: usize },

    #[error("Terminal error: {0}")]
    TerminalError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid dimension: {0}")]
    InvalidDimension(String),
}
```

**And** all public API methods return `Result<T, DotmaxError>` (never panic)

**And** error messages are actionable:
- Include context (what operation failed, why, how to fix)
- Example: "Out of bounds: (100, 50) in grid of size (80, 24). Check grid dimensions before setting dots."

**And** `Debug` and `Display` implementations provide useful info

**And** errors can be chained (preserve error context through layers)

**And** optional logging support via `log` crate (feature-gated):
```rust
#[cfg(feature = "logging")]
log::error!("Grid operation failed: {}", error);
```

**And** unit tests verify:
- Error creation and formatting
- Error propagation through call stack
- Custom error messages

**Prerequisites:** Story 2.1-2.3 (API surface exists)

**Technical Notes:**
- Use `thiserror` crate for error derivation
- Never use `.unwrap()` or `.expect()` in library code
- All panics are bugs (enforced in code review)
- Critical for FR56-58, NFR-R1
- Add `log` crate as optional dependency (feature = "logging")

---

### Story 2.5: Add Terminal Resize Event Handling

As a **developer creating responsive terminal applications**,
I want automatic grid adjustment when terminal size changes,
So that braille output adapts to user window resizing.

**Acceptance Criteria:**

**Given** a terminal that can be resized
**When** terminal dimensions change
**Then** dotmax provides:

**`TerminalRenderer::handle_resize() -> Result<(u16, u16), DotmaxError>`**:
- Detects terminal size change via crossterm
- Returns new dimensions (columns, rows)
- Triggers event for application to handle

**And** `BrailleGrid::resize(new_width, new_height)` handles grid adjustment:
- If growing: preserve existing dots, initialize new cells to false
- If shrinking: truncate excess dots
- Maintains data integrity during resize

**And** example `examples/resize_demo.rs` demonstrates:
```rust
// Renders grid that responds to terminal resize
// Shows: terminal size detection, grid resize, re-render
// User can resize terminal window and see braille adjust
```

**And** resize operation completes in <10ms for typical grids

**And** unit tests cover:
- Grow grid (100×50 → 200×100)
- Shrink grid (200×100 → 80×24)
- Edge cases (resize to zero, massive resize)

**Prerequisites:** Story 2.1 (BrailleGrid), Story 2.3 (TerminalRenderer)

**Technical Notes:**
- Crossterm provides terminal size queries
- Resize is common in terminal apps (user adjusts window)
- Preserve existing content where possible (better UX)
- Critical for FR7
- Later epics will add smart resize for images (preserve aspect ratio)

---

### Story 2.6: Implement Color Support for Braille Cells

As a **developer creating colored terminal graphics**,
I want per-cell RGB color assignment with terminal color conversion,
So that braille output can be vibrant and visually rich.

**Acceptance Criteria:**

**Given** a BrailleGrid with dot patterns
**When** I add color support
**Then** `src/grid/color.rs` provides:

**`Color` struct**:
```rust
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
```

**And** `BrailleGrid` extended with:
- `colors: Vec<Vec<Option<Color>>>` field (parallel to dots)
- `set_color(x, y, color: Color) -> Result<(), DotmaxError>` - assign color to cell
- `get_color(x, y) -> Option<Color>` - read cell color
- `clear_colors()` - reset all to None (monochrome)

**And** `src/terminal/color_conversion.rs` provides:
- `rgb_to_ansi256(r, g, b) -> u8` - converts RGB to ANSI 256-color
- `rgb_to_true_color(r, g, b) -> String` - converts to terminal true color escape codes
- `detect_terminal_color_support() -> ColorCapability` - queries terminal capabilities

**And** `TerminalRenderer` uses colors when rendering:
- If terminal supports true color → use RGB directly
- If terminal supports 256-color → convert to ANSI 256
- If terminal is monochrome → ignore colors (fallback gracefully)

**And** example `examples/color_demo.rs` shows rainbow gradient

**And** color operations add <5% rendering overhead

**Prerequisites:** Story 2.1-2.3 (Core rendering), Story 2.4 (Error handling)

**Technical Notes:**
- Color is optional (None = use terminal default)
- ANSI 256-color is widely supported
- True color (24-bit) requires modern terminals
- Graceful degradation for limited terminals
- Critical for FR8, FR32-33, FR37
- ~20 lines from crabmusic color utilities

---

### Story 2.7: Add Debug Logging and Tracing Support

As a **developer troubleshooting rendering issues**,
I want optional debug logging that traces operations,
So that I can diagnose problems without modifying library code.

**Acceptance Criteria:**

**Given** dotmax operations executing
**When** logging feature is enabled
**Then** `Cargo.toml` includes:
```toml
[dependencies]
log = { version = "0.4", optional = true }

[features]
logging = ["dep:log"]
```

**And** key operations emit log events:
- `log::trace!` - Detailed dot operations (set_dot, get_dot)
- `log::debug!` - Grid operations (create, clear, resize)
- `log::info!` - Terminal operations (render, terminal size change)
- `log::warn!` - Degraded operation (Unicode not supported, color fallback)
- `log::error!` - Errors (but still return Result, don't crash)

**And** logging is zero-cost when disabled (compile-time feature gate)

**And** example `examples/logging_demo.rs` demonstrates:
```rust
// Configure env_logger
env_logger::init();
// Run dotmax operations
// Output shows trace of all operations
```

**And** documentation explains logging setup for users

**And** CI tests with and without logging feature

**Prerequisites:** Story 2.1-2.6 (Core API complete)

**Technical Notes:**
- `log` crate is standard Rust logging facade
- Users choose implementation (env_logger, tracing, etc.)
- Feature-gated to avoid dependency for users who don't need it
- Critical for FR60, NFR-DX4
- Helps with troubleshooting (FR90)

---

## Epic 3: 2D Image Rendering Pipeline

**Goal**: Enable developers to load and render images (PNG, JPG, GIF, BMP, WebP, TIFF, SVG) to braille grids with professional-quality output. Implement resize/scaling, threshold algorithms (Otsu), dithering methods (Floyd-Steinberg, Bayer, Atkinson), and monochrome/color modes.

**Value Delivered**: Developers can convert any standard image format to beautiful braille terminal output with <100 lines of code. This is the primary use case for dotmax.

**FRs Covered**: FR9-20 (12 FRs)

---

### Story 3.1: Implement Image Loading from File Paths and Byte Buffers

As a **developer integrating images into terminal applications**,
I want to load images from files or memory with error handling,
So that I can prepare images for braille conversion.

**Acceptance Criteria:**

**Given** image files in standard formats
**When** I use image loading API
**Then** `src/image/loader.rs` provides:

**`ImageLoader` struct with methods**:
```rust
pub fn load_from_path(path: &Path) -> Result<DynamicImage, DotmaxError>;
pub fn load_from_bytes(bytes: &[u8]) -> Result<DynamicImage, DotmaxError>;
pub fn supported_formats() -> Vec<&'static str>;
```

**And** supported formats (via `image` crate):
- PNG, JPEG, GIF, BMP, WebP, TIFF (all common raster formats)

**And** error handling for:
- File not found → `DotmaxError::IoError`
- Unsupported format → `DotmaxError::ImageError("Unsupported format: .xyz")`
- Corrupted file → `DotmaxError::ImageError("Failed to decode image")`

**And** `Cargo.toml` feature:
```toml
[features]
image = ["dep:image"]

[dependencies]
image = { version = "0.25", optional = true }
```

**And** example `examples/load_image.rs` demonstrates loading PNG/JPG

**And** unit tests cover:
- Load valid images (test fixtures)
- Handle missing files gracefully
- Handle corrupted files gracefully
- Load from byte buffer

**Prerequisites:** Story 1.3 (feature flags), Story 2.4 (error handling)

**Technical Notes:**
- Use `image` crate (de facto standard for Rust image handling)
- `DynamicImage` type supports all formats uniformly
- Feature-gated behind `image` flag (optional dependency)
- Critical for FR9-10
- Test fixtures: add small test images to `tests/fixtures/`

---

### Story 3.2: Implement Image Resize and Aspect Ratio Preservation

As a **developer rendering images to terminals of varying sizes**,
I want automatic and manual image resizing with aspect ratio preservation,
So that images fit terminal dimensions without distortion.

**Acceptance Criteria:**

**Given** a loaded image (DynamicImage)
**When** I resize for terminal rendering
**Then** `src/image/resize.rs` provides:

**`ImageResizer` with methods**:
```rust
pub fn resize_to_terminal(image: &DynamicImage, term_width: u16, term_height: u16) -> DynamicImage;
pub fn resize_to_dimensions(image: &DynamicImage, target_width: u32, target_height: u32, preserve_aspect: bool) -> DynamicImage;
```

**And** `resize_to_terminal` behavior:
- Calculates max dimensions considering braille 2×4 cells (each terminal cell = 2×4 pixels equivalent)
- Preserves aspect ratio automatically
- Uses high-quality resizing (Lanczos3 filter from `image` crate)

**And** `resize_to_dimensions` allows manual control:
- `preserve_aspect = true` → letterbox/pillarbox to fit
- `preserve_aspect = false` → stretch to exact dimensions

**And** edge cases handled:
- Image smaller than terminal → no upscaling (configurable)
- Image larger than terminal → downscale to fit
- Zero or extreme dimensions → error

**And** example `examples/resize_image.rs` shows auto-resize for current terminal

**And** unit tests verify aspect ratio math

**Prerequisites:** Story 3.1 (image loading)

**Technical Notes:**
- Lanczos3 provides best quality (vs. nearest neighbor)
- Braille cells are 2×4 dots, so calculate pixel-to-dot mapping
- Critical for FR12-13
- Performance: resize should be <50ms for typical images

---

### Story 3.3: Implement Grayscale Conversion and Otsu Thresholding

As a **developer converting images to binary braille patterns**,
I want intelligent grayscale conversion with optimal thresholding,
So that braille output has clear contrast and detail.

**Acceptance Criteria:**

**Given** a color or grayscale image
**When** I convert to binary for braille
**Then** `src/image/threshold.rs` provides:

**`Thresholder` with methods**:
```rust
pub fn to_grayscale(image: &DynamicImage) -> GrayImage;
pub fn otsu_threshold(gray: &GrayImage) -> u8;
pub fn apply_threshold(gray: &GrayImage, threshold: u8) -> BinaryImage;
pub fn auto_threshold(image: &DynamicImage) -> BinaryImage;
```

**And** Otsu's method implementation:
- Calculates optimal threshold automatically (maximizes between-class variance)
- Returns threshold value 0-255
- Works on grayscale images (convert color first)

**And** `BinaryImage` type:
```rust
pub struct BinaryImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<bool>, // true = black, false = white
}
```

**And** `auto_threshold` pipeline:
1. Convert to grayscale if needed
2. Calculate Otsu threshold
3. Apply threshold to create binary image

**And** brightness/contrast/gamma adjustments:
```rust
pub fn adjust_brightness(gray: &GrayImage, factor: f32) -> GrayImage;
pub fn adjust_contrast(gray: &GrayImage, factor: f32) -> GrayImage;
pub fn adjust_gamma(gray: &GrayImage, gamma: f32) -> GrayImage;
```

**And** unit tests verify Otsu calculation on known images

**Prerequisites:** Story 3.1 (image loading), Story 3.2 (resize)

**Technical Notes:**
- Otsu's method is industry standard for auto-thresholding
- `imageproc` crate may have implementation (check first)
- Critical for FR14, FR18-19
- Reference: crabmusic threshold implementation

---

### Story 3.4: Implement Dithering Algorithms (Floyd-Steinberg, Bayer, Atkinson)

As a **developer creating high-quality braille images**,
I want multiple dithering methods to convert grayscale to binary,
So that images retain detail and gradients instead of harsh thresholding.

**Acceptance Criteria:**

**Given** a grayscale image
**When** I apply dithering
**Then** `src/image/dither.rs` provides:

**`DitheringMethod` enum**:
```rust
pub enum DitheringMethod {
    None,              // Direct threshold (no dithering)
    FloydSteinberg,    // Error diffusion (best quality, slower)
    Bayer,             // Ordered dithering (fast, good for gradients)
    Atkinson,          // Error diffusion (Apple-style, softer than Floyd-Steinberg)
}
```

**And** `apply_dithering` function:
```rust
pub fn apply_dithering(gray: &GrayImage, method: DitheringMethod) -> BinaryImage;
```

**And** Floyd-Steinberg implementation:
- Error diffusion to neighbors: right (7/16), below-left (3/16), below (5/16), below-right (1/16)
- Serpentine scanning (alternate left-right, right-left per row) for better quality

**And** Bayer implementation:
- 8×8 Bayer matrix for ordered dithering
- Fast and good for gradients/textures

**And** Atkinson implementation:
- Distributes error to 6 neighbors (lighter than Floyd-Steinberg)
- Characteristic of classic Mac graphics

**And** example `examples/dithering_comparison.rs` shows same image with all 4 methods side-by-side

**And** benchmarks compare performance (Floyd-Steinberg slowest but best quality)

**Prerequisites:** Story 3.3 (grayscale conversion)

**Technical Notes:**
- Floyd-Steinberg is most common (default choice)
- Bayer is fastest (good for real-time applications)
- Atkinson has aesthetic appeal (retro Mac look)
- Critical for FR15
- Reference: crabmusic dithering implementation (~100-200 lines)

---

### Story 3.5: Implement Binary Image to BrailleGrid Conversion

As a **developer rendering processed images to terminals**,
I want conversion from binary pixel data to braille dot patterns,
So that images display as braille characters.

**Acceptance Criteria:**

**Given** a BinaryImage (from threshold/dither)
**When** I convert to BrailleGrid
**Then** `src/image/to_braille.rs` provides:

**`BrailleConverter::convert(binary: &BinaryImage) -> BrailleGrid`**:
- Maps every 2×4 pixel region to one braille cell
- Pixel → Dot mapping: pixel (x, y) → dot index based on position in 2×4 block
- Handles images not perfectly divisible by 2×4 (pad or truncate)

**And** mapping algorithm:
```
For each braille cell at (cx, cy):
  For each dot position (dx, dy) in 2×4:
    pixel_x = cx * 2 + dx
    pixel_y = cy * 4 + dy
    dot_value = binary.pixels[pixel_y * width + pixel_x]
    grid.set_dot(cx, cy, dot_index, dot_value)
```

**And** edge handling:
- If image width not multiple of 2 → pad right with white pixels
- If image height not multiple of 4 → pad bottom with white pixels

**And** example `examples/image_to_braille.rs`:
```rust
// Load image → resize → threshold → dither → convert to braille → render
// Full pipeline demonstration in <100 lines
```

**And** conversion completes in <25ms for standard terminal (80×24 cells)

**And** unit tests verify correct pixel-to-dot mapping

**Prerequisites:** Story 2.1 (BrailleGrid), Story 3.3 (BinaryImage), Story 3.4 (dithering)

**Technical Notes:**
- This is the core image→braille conversion
- Pixel indexing must be exact (off-by-one errors ruin output)
- Consider SIMD optimization if benchmarks show bottleneck
- Critical for FR14
- Target: <25ms for 160×96 pixels (80×24 cells)

---

### Story 3.6: Add SVG Vector Graphics Support with Rasterization

As a **developer using vector graphics in terminals**,
I want SVG loading and rasterization to braille,
So that I can render logos, icons, and vector art.

**Acceptance Criteria:**

**Given** an SVG file
**When** I load and rasterize it
**Then** `src/image/svg.rs` provides:

**SVG loading** (feature-gated):
```toml
[features]
svg = ["dep:resvg", "dep:usvg"]

[dependencies]
resvg = { version = "0.38", optional = true }
usvg = { version = "0.38", optional = true }
```

**And** `SvgLoader` struct:
```rust
pub fn load_svg(path: &Path) -> Result<DynamicImage, DotmaxError>;
pub fn load_svg_from_bytes(bytes: &[u8]) -> Result<DynamicImage, DotmaxError>;
pub fn rasterize_svg(svg_data: &[u8], width: u32, height: u32) -> Result<DynamicImage, DotmaxError>;
```

**And** rasterization pipeline:
1. Parse SVG with `usvg`
2. Rasterize to pixel buffer with `resvg`
3. Convert to `DynamicImage`
4. Feed to existing image→braille pipeline

**And** SVG rendering respects:
- Target dimensions (user-specified or auto-calculated for terminal)
- Preserves aspect ratio
- Renders at high quality (anti-aliasing)

**And** example `examples/svg_demo.rs` loads and renders SVG logo

**And** error handling for malformed SVG

**Prerequisites:** Story 3.1-3.5 (image pipeline complete)

**Technical Notes:**
- `resvg` is best Rust SVG renderer
- Rasterize to reasonable resolution (2× terminal size in pixels for quality)
- SVG is important for logos, icons, UI elements
- Critical for FR11
- Feature-gated (not all users need SVG)

---

### Story 3.7: Implement Color Mode Image Rendering

As a **developer creating colored braille images**,
I want to preserve image colors when rendering to terminal,
So that braille output can be vibrant and faithful to original.

**Acceptance Criteria:**

**Given** a color image
**When** I render in color mode
**Then** `src/image/color_render.rs` provides:

**`ColorBrailleConverter::convert(image: &DynamicImage) -> (BrailleGrid, ColorGrid)`**:
- Processes image in color (don't convert to grayscale)
- For each 2×4 pixel region → calculate average color → assign to braille cell
- Returns tuple: (BrailleGrid with dot patterns, ColorGrid with RGB values)

**And** color calculation options:
- `average_color(pixels: &[Rgb])` - mean RGB in 2×4 block
- `dominant_color(pixels: &[Rgb])` - most common color in block
- `sample_color(pixels: &[Rgb])` - center pixel color

**And** `TerminalRenderer::render_with_colors(grid: &BrailleGrid, colors: &ColorGrid)`

**And** example `examples/color_image.rs`:
```rust
// Load color image → resize → convert with color → render to terminal
// Shows vibrant colored braille output
```

**And** graceful degradation:
- Terminal doesn't support color → fallback to monochrome
- Partial color support (ANSI 16) → approximate with available colors

**And** color mode adds <15% overhead vs. monochrome

**Prerequisites:** Story 2.6 (color support), Story 3.5 (image to braille)

**Technical Notes:**
- Average color works well for most images
- Consider perceptual color spaces (LAB) if RGB artifacts
- Critical for FR17
- Reference: crabmusic color scheme application

---

### Story 3.8: Create High-Level Image Rendering API

As a **developer wanting simple image rendering**,
I want a high-level API that handles the full pipeline,
So that I can render images with <10 lines of code.

**Acceptance Criteria:**

**Given** the complete image processing pipeline
**When** I create convenience API
**Then** `src/image/mod.rs` provides:

**`ImageRenderer` builder**:
```rust
ImageRenderer::new()
    .load_from_path("image.png")?
    .resize_to_terminal()?
    .dithering(DitheringMethod::FloydSteinberg)
    .color_mode(ColorMode::TrueColor)
    .render()?;
```

**And** `render_image_simple(path: &Path)` one-liner:
```rust
// Loads, auto-resizes, auto-thresholds, renders with defaults
dotmax::render_image_simple("logo.png")?;
```

**And** example `examples/simple_image.rs` demonstrates <10 line usage

**And** builder pattern allows customization:
```rust
ImageRenderer::new()
    .load_from_path("photo.jpg")?
    .resize(100, 50, true) // manual dimensions
    .brightness(1.2)
    .contrast(1.1)
    .gamma(0.9)
    .threshold(128) // manual threshold
    .dithering(DitheringMethod::Atkinson)
    .render()?;
```

**And** API documentation shows all options

**And** Error messages guide users to fixes

**Prerequisites:** Story 3.1-3.7 (all image features complete)

**Technical Notes:**
- Builder pattern for ergonomics (FR46)
- Sensible defaults (auto-threshold, Floyd-Steinberg, auto-resize)
- Critical for FR44 (<100 lines integration)
- This is the main public API for image rendering

---

## Epic 4: Drawing Primitives & Density Rendering

**Goal**: Provide programmatic drawing capabilities (lines, circles, rectangles, polygons) using Bresenham algorithms and character density-based rendering for ASCII-art style effects. Enable developers to draw graphics procedurally, not just convert images.

**Value Delivered**: Developers can create graphics programmatically, draw UI elements, create loading bars, and render ASCII-art style visualizations with smooth gradients.

**FRs Covered**: FR21-31 (11 FRs)

---

### Story 4.1: Implement Bresenham Line Drawing Algorithm

As a **developer drawing graphics programmatically**,
I want line drawing between two points on braille grid,
So that I can create wireframes, UI borders, and connections.

**Acceptance Criteria:**

**Given** a BrailleGrid
**When** I draw a line
**Then** `src/primitives/line.rs` provides:

**`draw_line(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32) -> Result<(), DotmaxError>`**:
- Uses Bresenham's line algorithm (integer-only, fast)
- Sets dots along line from (x0, y0) to (x1, y1)
- Handles all octants (any angle)
- Clips to grid boundaries (out-of-bounds is not an error, just clips)

**And** line thickness support:
```rust
pub fn draw_line_thick(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, thickness: u32) -> Result<(), DotmaxError>;
```
- thickness = 1: single dot width
- thickness > 1: draws parallel lines offset perpendicular to main line

**And** example `examples/lines_demo.rs`:
```rust
// Draws grid of lines at various angles
// Shows: horizontal, vertical, diagonal, thick lines
```

**And** benchmarks show <1ms for 1000 pixel line

**And** unit tests verify:
- Horizontal/vertical lines
- 45° diagonal
- Arbitrary angles
- Line thickness
- Boundary clipping

**Prerequisites:** Story 2.1 (BrailleGrid)

**Technical Notes:**
- Bresenham is standard for raster line drawing (~50 lines)
- Integer arithmetic only (fast, no floating point)
- Critical for FR21, FR26
- Reference: crabmusic Bresenham implementation

---

### Story 4.2: Implement Bresenham Circle Drawing Algorithm

As a **developer creating circular UI elements**,
I want circle drawing with center and radius,
So that I can create buttons, indicators, and radial graphics.

**Acceptance Criteria:**

**Given** a BrailleGrid
**When** I draw a circle
**Then** `src/primitives/circle.rs` provides:

**`draw_circle(grid: &mut BrailleGrid, center_x: i32, center_y: i32, radius: u32) -> Result<(), DotmaxError>`**:
- Uses Bresenham's circle algorithm (midpoint algorithm)
- Draws circle outline (not filled)
- Handles clipping to grid boundaries

**And** filled circle support:
```rust
pub fn draw_circle_filled(grid: &mut BrailleGrid, center_x: i32, center_y: i32, radius: u32) -> Result<(), DotmaxError>;
```
- Fills interior using horizontal line spans

**And** example `examples/circles_demo.rs`:
```rust
// Draws concentric circles, filled and unfilled
// Shows: various radii, clipping at edges
```

**And** unit tests verify:
- Small circles (radius 1-5)
- Large circles (radius 100+)
- Circles partially off-grid (clipping)
- Filled vs. outline

**Prerequisites:** Story 2.1 (BrailleGrid), Story 4.1 (line drawing for fills)

**Technical Notes:**
- Bresenham/midpoint circle is efficient (~60 lines)
- Draw 8 octants simultaneously using symmetry
- Critical for FR22
- Reference: classic computer graphics algorithm

---

### Story 4.3: Implement Rectangle and Polygon Drawing

As a **developer creating UI layouts and shapes**,
I want rectangle and polygon drawing primitives,
So that I can create boxes, borders, and complex shapes.

**Acceptance Criteria:**

**Given** a BrailleGrid
**When** I draw shapes
**Then** `src/primitives/shapes.rs` provides:

**Rectangle functions**:
```rust
pub fn draw_rectangle(grid: &mut BrailleGrid, x: i32, y: i32, width: u32, height: u32) -> Result<(), DotmaxError>;
pub fn draw_rectangle_filled(grid: &mut BrailleGrid, x: i32, y: i32, width: u32, height: u32) -> Result<(), DotmaxError>;
```

**Polygon functions**:
```rust
pub fn draw_polygon(grid: &mut BrailleGrid, vertices: &[(i32, i32)]) -> Result<(), DotmaxError>;
pub fn draw_polygon_filled(grid: &mut BrailleGrid, vertices: &[(i32, i32)]) -> Result<(), DotmaxError>;
```

**And** rectangle implementation:
- Outline: draw 4 lines (top, right, bottom, left)
- Filled: draw horizontal spans for each row

**And** polygon implementation:
- Outline: draw lines between consecutive vertices (close path)
- Filled: scanline fill algorithm

**And** example `examples/shapes_demo.rs`:
```rust
// Draws various shapes: rectangles, triangles, hexagons
// Shows filled and outline modes
```

**And** unit tests verify:
- Rectangles (various sizes, positions)
- Triangles (all orientations)
- Complex polygons (5+ vertices)
- Clipping behavior

**Prerequisites:** Story 4.1 (line drawing)

**Technical Notes:**
- Rectangle is simple (4 lines)
- Polygon fill uses scanline algorithm (~100 lines)
- Validate polygon has ≥3 vertices
- Critical for FR23-25
- Reference: classic polygon fill algorithms

---

### Story 4.4: Implement Character Density-Based Rendering

As a **developer creating ASCII-art style visualizations**,
I want intensity-to-character mapping for gradients,
So that I can render smooth shading without binary thresholds.

**Acceptance Criteria:**

**Given** intensity values (0.0 to 1.0)
**When** I map to character densities
**Then** `src/density/mod.rs` provides:

**`DensityCharSet` predefined sets**:
```rust
pub const ASCII_DENSITY: &str = " .'`^\",:;Il!i><~+_-?][}{1)(|/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$";
pub const SIMPLE_DENSITY: &str = " .:-=+*#%@";
pub const BLOCKS_DENSITY: &str = " ░▒▓█";
pub const BRAILLE_DENSITY: &str = "⠀⠁⠃⠇⠏⠟⠿⡿⣿"; // Increasing dot density
```

**And** `DensityRenderer` struct:
```rust
pub fn new(charset: &str) -> Self;
pub fn intensity_to_char(&self, intensity: f32) -> char;
pub fn render_grid(&self, intensities: &[Vec<f32>]) -> String;
```

**And** intensity mapping:
- 0.0 → first char (lightest, typically space)
- 1.0 → last char (darkest, typically '@' or '█')
- Linear interpolation for intermediate values

**And** example `examples/density_demo.rs`:
```rust
// Renders gradient using different density character sets
// Shows smooth transitions from light to dark
```

**And** custom character sets supported:
```rust
let custom = DensityRenderer::new("  ..ooOO@@");
```

**And** unit tests verify correct character selection at boundaries (0.0, 0.5, 1.0)

**Prerequisites:** Story 2.1 (BrailleGrid - for integration, but density works standalone)

**Technical Notes:**
- Density rendering is orthogonal to braille (alternative rendering mode)
- Can combine: use density chars to fill braille grid cells
- Critical for FR28-31
- ~150 lines in crabmusic, ~400 for full system with schemes
- Reference: crabmusic character density implementation

---

### Story 4.5: Add Color Support for Drawing Primitives

As a **developer creating colored graphics**,
I want to set colors for line/circle/shape drawing,
So that programmatic graphics can be vibrant.

**Acceptance Criteria:**

**Given** drawing primitive functions
**When** I add color parameter
**Then** all drawing functions accept optional color:

**Updated signatures**:
```rust
pub fn draw_line_colored(grid: &mut BrailleGrid, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) -> Result<(), DotmaxError>;
pub fn draw_circle_colored(grid: &mut BrailleGrid, cx: i32, cy: i32, radius: u32, color: Color) -> Result<(), DotmaxError>;
// ... similar for all primitives
```

**And** color is applied to all dots in the shape

**And** existing non-colored functions remain (use default/no color)

**And** example `examples/colored_shapes.rs`:
```rust
// Draws rainbow of colored shapes
// Shows: red circle, green rectangle, blue polygon
```

**And** unit tests verify color application

**Prerequisites:** Story 2.6 (color support), Story 4.1-4.3 (primitives)

**Technical Notes:**
- Color parameter is `Option<Color>` internally (None = no color override)
- Set both dots and color when drawing
- Critical for FR27
- ~50 lines to add color to all primitive functions

---

## Epic 5: Color System & Visual Schemes

**Goal**: Build comprehensive color system with RGB support, terminal color conversion (ANSI 256, true color), 6+ predefined color schemes from crabmusic, custom scheme creation, and intensity-to-color mapping. Transform monochrome braille into vibrant visual output.

**Value Delivered**: Developers can create colorful terminal graphics with predefined schemes or custom color maps, automatically adapting to terminal capabilities.

**FRs Covered**: FR32-37 (8 FRs including enhancements)

---

### Story 5.1: Implement Terminal Color Capability Detection

As a **developer targeting diverse terminals**,
I want automatic detection of color support levels,
So that color rendering adapts to terminal capabilities.

**Acceptance Criteria:**

**Given** various terminal environments
**When** I query color capabilities
**Then** `src/terminal/color_detect.rs` provides:

**`ColorCapability` enum**:
```rust
pub enum ColorCapability {
    Monochrome,      // No color support
    Ansi16,          // 16 colors (basic ANSI)
    Ansi256,         // 256 colors (extended ANSI)
    TrueColor,       // 24-bit RGB (16 million colors)
}
```

**And** `detect_color_capability() -> ColorCapability`:
- Checks `$COLORTERM` environment variable (truecolor, 256color)
- Checks `$TERM` variable (xterm-256color, etc.)
- Queries terminal via ANSI escape codes (fallback)
- Returns most capable mode supported

**And** detection is cached (only query once per session)

**And** example `examples/color_detection.rs` prints detected capability

**And** unit tests mock env vars to test detection logic

**Prerequisites:** Story 2.3 (terminal abstraction)

**Technical Notes:**
- Most modern terminals support 256-color or true color
- PowerShell/CMD may be limited (ANSI 16 or less)
- $COLORTERM="truecolor" is reliable indicator
- Critical for FR37
- Reference: crossterm color detection helpers

---

### Story 5.2: Implement RGB to ANSI Color Conversion

As a **developer rendering colors to terminals**,
I want RGB-to-ANSI conversion for broad compatibility,
So that colors work even on limited terminals.

**Acceptance Criteria:**

**Given** RGB color values
**When** I convert to ANSI codes
**Then** `src/terminal/color_convert.rs` provides:

**ANSI 256-color conversion**:
```rust
pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8;
```
- Maps RGB (0-255 each) to ANSI 256 palette (0-255 index)
- Uses color cube (216 colors) + grayscale (24 shades)
- Euclidean distance in RGB space for closest match

**ANSI 16-color conversion** (fallback):
```rust
pub fn rgb_to_ansi16(r: u8, g: u8, b: u8) -> u8;
```
- Maps to basic 16 ANSI colors
- Simple thresholding for R, G, B channels

**True color escape code generation**:
```rust
pub fn rgb_to_truecolor_escape(r: u8, g: u8, b: u8) -> String;
// Returns: "\x1b[38;2;{r};{g};{b}m"
```

**And** color conversion respects detected capability:
```rust
pub fn rgb_to_terminal_color(r: u8, g: u8, b: u8, capability: ColorCapability) -> String;
// Automatically chooses conversion based on capability
```

**And** benchmarks show conversion <100ns per color

**And** unit tests verify:
- RGB(255,0,0) → ANSI 196 (bright red) for 256-color
- RGB(128,128,128) → appropriate gray in all modes
- Escape code formatting

**Prerequisites:** Story 5.1 (color detection)

**Technical Notes:**
- ANSI 256 palette: 16 basic + 216 color cube (6×6×6) + 24 grayscale
- Color cube formula: `16 + 36*r + 6*g + b` (r,g,b are 0-5)
- Critical for FR33
- ~100 lines total
- Reference: ANSI color spec, existing color conversion libraries

---

### Story 5.3: Extract and Integrate 6+ Color Schemes from Crabmusic

As a **developer wanting beautiful predefined color palettes**,
I want proven color schemes from crabmusic,
So that I can create vibrant graphics without designing colors.

**Acceptance Criteria:**

**Given** crabmusic color schemes
**When** I extract them to dotmax
**Then** `src/color/schemes.rs` provides at least 6 schemes:

**`ColorScheme` struct**:
```rust
pub struct ColorScheme {
    pub name: String,
    pub colors: Vec<Color>,  // Ordered from low to high intensity
}
```

**Predefined schemes** (from crabmusic):
1. **Rainbow** - Full spectrum (red → orange → yellow → green → blue → purple)
2. **Fire** - Heat map (black → red → orange → yellow → white)
3. **Ocean** - Water theme (deep blue → cyan → teal → white)
4. **Forest** - Nature theme (dark green → lime → yellow-green)
5. **Monochrome** - Grayscale (black → gray → white)
6. **Sunset** - Warm gradient (deep purple → pink → orange → yellow)

**And** schemes accessible via:
```rust
pub fn get_scheme(name: &str) -> Option<ColorScheme>;
pub fn list_schemes() -> Vec<String>;
```

**And** example `examples/color_schemes.rs`:
```rust
// Renders gradient using each predefined scheme
// Shows all 6+ schemes side-by-side
```

**And** unit tests verify all schemes exist and have valid colors

**Prerequisites:** Story 2.6 (Color struct)

**Technical Notes:**
- Extract ~150 lines from crabmusic color scheme definitions
- Each scheme has 10-20 colors typically
- Colors are interpolated for smooth gradients
- Critical for FR34
- Reference: crabmusic `src/visualization/color_schemes.rs`

---

### Story 5.4: Implement Custom Color Scheme Creation and Intensity Mapping

As a **developer creating unique color palettes**,
I want to define custom color schemes and map intensity to colors,
So that I can create brand-specific or artistic effects.

**Acceptance Criteria:**

**Given** intensity values (0.0 to 1.0)
**When** I create custom color scheme
**Then** `src/color/scheme_builder.rs` provides:

**`ColorSchemeBuilder`**:
```rust
ColorSchemeBuilder::new("custom_name")
    .add_color(0.0, Color::rgb(0, 0, 0))      // 0% intensity = black
    .add_color(0.5, Color::rgb(255, 0, 0))    // 50% = red
    .add_color(1.0, Color::rgb(255, 255, 0))  // 100% = yellow
    .build();
```

**And** `ColorScheme::map_intensity(intensity: f32) -> Color`:
- Linear interpolation between adjacent color stops
- 0.0 → first color
- 1.0 → last color
- 0.25 → interpolate between first and second color

**And** validation:
- Intensity values must be 0.0-1.0
- At least 2 colors required
- Colors must be in ascending intensity order

**And** example `examples/custom_scheme.rs`:
```rust
// Creates brand-specific color scheme
// Maps to intensity buffer and renders
```

**And** unit tests verify interpolation math

**Prerequisites:** Story 5.3 (ColorScheme struct)

**Technical Notes:**
- Linear RGB interpolation is simple but may have artifacts
- Consider LAB color space for perceptual uniformity (future enhancement)
- Critical for FR35-36
- ~100 lines for builder and interpolation

---

### Story 5.5: Apply Color Schemes to Grayscale Intensity Buffers

As a **developer colorizing grayscale data**,
I want to apply color schemes to intensity maps,
So that visualizations like heatmaps and gradients are vibrant.

**Acceptance Criteria:**

**Given** grayscale intensity buffer and color scheme
**When** I apply scheme
**Then** `src/color/apply.rs` provides:

**`apply_color_scheme(intensities: &[Vec<f32>], scheme: &ColorScheme) -> Vec<Vec<Color>>`**:
- Maps each intensity value to RGB color via scheme
- Returns 2D color grid matching input dimensions

**And** integration with BrailleGrid:
```rust
pub fn apply_colors_to_grid(grid: &mut BrailleGrid, color_grid: &[Vec<Color>]);
```
- Sets color for each cell in grid

**And** example `examples/heatmap.rs`:
```rust
// Generates 2D intensity data (e.g., sin waves)
// Applies "fire" color scheme
// Renders as colored braille
// Result: vibrant heatmap visualization
```

**And** performance: <10ms to colorize 80×24 grid

**And** unit tests verify:
- Correct color mapping for min/max/mid intensities
- Grid dimensions preserved
- Color assignment to BrailleGrid

**Prerequisites:** Story 5.3 (schemes), Story 5.4 (intensity mapping), Story 2.6 (grid colors)

**Technical Notes:**
- This is where grayscale → color magic happens
- Useful for data visualization (audio, sensor data, math functions)
- Critical for FR36
- ~50 lines (straightforward mapping)

---

## Epic 6: Animation & Frame Management

**Goal**: Enable frame-by-frame animation playback, timing control, frame buffer management, pre-rendering optimization, and flicker-free updates. Support real-time animations at 60+ fps with minimal CPU overhead.

**Value Delivered**: Developers can create smooth animations, loading indicators, visualizations, and motion graphics in terminals at production quality.

**FRs Covered**: FR38-43 (6 FRs)

---

### Story 6.1: Implement Frame Buffer and Double Buffering

As a **developer creating flicker-free animations**,
I want double buffering for smooth frame transitions,
So that animations don't flicker or tear during updates.

**Acceptance Criteria:**

**Given** animated content updating frequently
**When** I use frame buffering
**Then** `src/animation/frame_buffer.rs` provides:

**`FrameBuffer` struct**:
```rust
pub struct FrameBuffer {
    front: BrailleGrid,  // Currently displayed
    back: BrailleGrid,   // Being prepared
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self;
    pub fn get_back_buffer(&mut self) -> &mut BrailleGrid;
    pub fn swap_buffers(&mut self);
    pub fn render(&self, renderer: &mut TerminalRenderer) -> Result<(), DotmaxError>;
}
```

**And** workflow:
1. Draw to back buffer (via `get_back_buffer()`)
2. Swap buffers atomically (via `swap_buffers()`)
3. Render front buffer to terminal
4. Repeat for next frame

**And** swap operation is <1ms (pointer swap, not data copy)

**And** example `examples/animation_buffer.rs`:
```rust
// Animates bouncing ball using double buffering
// Shows smooth, flicker-free motion
```

**And** unit tests verify buffer swap correctness

**Prerequisites:** Story 2.1 (BrailleGrid), Story 2.3 (TerminalRenderer)

**Technical Notes:**
- Double buffering prevents tearing/flicker
- Swap is cheap (swap references, not grid data)
- Critical for FR41, FR43
- Standard technique in graphics programming

---

### Story 6.2: Implement Frame Timing and Rate Control

As a **developer creating animations at specific frame rates**,
I want precise timing control (30fps, 60fps, custom),
So that animations play at consistent speed across systems.

**Acceptance Criteria:**

**Given** an animation loop
**When** I control frame timing
**Then** `src/animation/timing.rs` provides:

**`FrameTimer` struct**:
```rust
pub struct FrameTimer {
    target_fps: u32,
    frame_duration: Duration,
    last_frame: Instant,
}

impl FrameTimer {
    pub fn new(target_fps: u32) -> Self;
    pub fn wait_for_next_frame(&mut self);
    pub fn actual_fps(&self) -> f32;
    pub fn frame_time(&self) -> Duration;
}
```

**And** `wait_for_next_frame()` sleeps until next frame should start:
- Calculates elapsed since last frame
- Sleeps for (target_duration - elapsed) if ahead of schedule
- No sleep if behind schedule (drop frame)

**And** FPS reporting:
- `actual_fps()` calculates real frame rate from recent frames
- Helps diagnose performance issues

**And** example `examples/fps_control.rs`:
```rust
// Renders animation at 60fps
// Displays actual FPS in corner
// Shows timing control working
```

**And** timing accuracy: ±2ms for 60fps (16.67ms target)

**And** unit tests verify frame duration calculation

**Prerequisites:** Story 6.1 (FrameBuffer)

**Technical Notes:**
- Use `std::time::{Instant, Duration}` for high-precision timing
- Sleep granularity depends on OS (Linux ~1ms, Windows ~15ms default)
- Critical for FR39-40, FR70 (60fps requirement)
- Consider spin-wait for very precise timing (advanced)

---

### Story 6.3: Implement Animation Loop Helper

As a **developer wanting easy animation creation**,
I want a high-level animation loop abstraction,
So that I can focus on frame generation, not timing/buffering boilerplate.

**Acceptance Criteria:**

**Given** frame rendering logic
**When** I use animation loop
**Then** `src/animation/loop.rs` provides:

**`AnimationLoop` builder**:
```rust
AnimationLoop::new(width, height)
    .fps(60)
    .on_frame(|frame_num, back_buffer| {
        // User code: update back_buffer for this frame
        // Return Ok(true) to continue, Ok(false) to stop
        Ok(true)
    })
    .run()?;
```

**And** loop handles automatically:
- Frame buffer creation and management
- Double buffering (swap after each frame)
- Timing control (target FPS)
- Terminal rendering
- Ctrl+C detection (graceful exit)

**And** provides frame number to callback (for animation state)

**And** example `examples/simple_animation.rs`:
```rust
// Creates rotating line animation in <30 lines
// Shows: simple API, no manual buffer management
AnimationLoop::new(80, 24)
    .fps(30)
    .on_frame(|frame, buffer| {
        buffer.clear();
        let angle = frame as f32 * 0.1;
        draw_line(buffer, center, angle);
        Ok(true)
    })
    .run()?;
```

**And** supports async rendering (optional):
```rust
.on_frame_async(async |frame, buffer| { ... })
```

**Prerequisites:** Story 6.1 (FrameBuffer), Story 6.2 (FrameTimer)

**Technical Notes:**
- This is the main public animation API
- Hides complexity of buffering and timing
- Critical for FR38, FR44 (<100 lines)
- Builder pattern for ergonomics
- ~150 lines implementation

---

### Story 6.4: Implement Frame Pre-Rendering and Caching

As a **developer optimizing animation playback**,
I want to pre-render frames for known animations,
So that playback is smooth even for complex computations.

**Acceptance Criteria:**

**Given** an animation with known frame count
**When** I pre-render frames
**Then** `src/animation/prerender.rs` provides:

**`PrerenderedAnimation` struct**:
```rust
pub struct PrerenderedAnimation {
    frames: Vec<BrailleGrid>,
    frame_rate: u32,
}

impl PrerenderedAnimation {
    pub fn new(frame_rate: u32) -> Self;
    pub fn add_frame(&mut self, frame: BrailleGrid);
    pub fn play(&self, renderer: &mut TerminalRenderer) -> Result<(), DotmaxError>;
    pub fn play_loop(&self, renderer: &mut TerminalRenderer) -> Result<(), DotmaxError>;
    pub fn frame_count(&self) -> usize;
}
```

**And** `play()` renders frames sequentially at specified frame rate

**And** `play_loop()` repeats animation indefinitely (until Ctrl+C)

**And** memory consideration:
- Each frame is ~width × height bytes
- 80×24 grid ≈ 2KB per frame
- 300 frames (10 seconds at 30fps) ≈ 600KB
- Reasonable for short animations

**And** example `examples/prerendered_demo.rs`:
```rust
// Pre-renders spinning ASCII art (60 frames)
// Plays back at 30fps
// Shows smooth playback with zero computation during play
```

**And** supports loading/saving to file (serialize frames):
```rust
pub fn save_to_file(&self, path: &Path) -> Result<(), DotmaxError>;
pub fn load_from_file(path: &Path) -> Result<Self, DotmaxError>;
```

**Prerequisites:** Story 6.1-6.3 (animation infrastructure)

**Technical Notes:**
- Pre-rendering trades memory for CPU
- Useful for short looping animations (loading spinners, etc.)
- Critical for FR42
- Consider compression for stored animations
- ~100 lines implementation

---

### Story 6.5: Optimize Differential Rendering for Animations

As a **developer optimizing animation performance**,
I want to render only changed cells between frames,
So that CPU usage is minimal even at high frame rates.

**Acceptance Criteria:**

**Given** sequential animation frames
**When** I enable differential rendering
**Then** `src/animation/differential.rs` provides:

**`DifferentialRenderer`**:
```rust
pub struct DifferentialRenderer {
    last_frame: Option<BrailleGrid>,
}

impl DifferentialRenderer {
    pub fn new() -> Self;
    pub fn render_diff(&mut self, current: &BrailleGrid, renderer: &mut TerminalRenderer) -> Result<(), DotmaxError>;
}
```

**And** `render_diff()` algorithm:
1. Compare current frame to last frame
2. Identify changed cells (different dots or colors)
3. Render only changed cells to terminal (via ANSI cursor positioning)
4. Store current as last_frame for next comparison

**And** performance improvement:
- Full render: N cells → N escape codes
- Differential render: typically 10-30% of cells changed
- 60-80% reduction in terminal I/O

**And** example `examples/differential_demo.rs`:
```rust
// Animates small moving object on static background
// Shows CPU/FPS with and without differential rendering
// Demonstrates 50-70% CPU reduction
```

**And** benchmarks compare full vs. differential rendering

**And** gracefully handles:
- First frame (no previous) → render all
- Terminal resize → render all (invalidate cache)

**Prerequisites:** Story 6.1-6.3 (animation basics), Story 2.3 (TerminalRenderer)

**Technical Notes:**
- Differential rendering critical for high FPS (FR70: 60fps <10% CPU)
- ANSI cursor positioning: `\x1b[{row};{col}H`
- Trade-off: diff calculation vs. render savings (usually worth it)
- ~150 lines implementation
- Reference: terminal UI libraries (ratatui) use this technique

---

### Story 6.6: Create High-Level Animation Examples and Documentation

As a **developer learning dotmax animation**,
I want comprehensive examples and documentation,
So that I can create animations quickly without trial-and-error.

**Acceptance Criteria:**

**Given** complete animation API
**When** I create documentation
**Then** documentation includes:

**Examples** (`examples/animations/`):
1. `bouncing_ball.rs` - Simple physics simulation
2. `loading_spinner.rs` - Rotating braille spinner (like npm install)
3. `waveform.rs` - Audio-style waveform visualization
4. `fireworks.rs` - Particle system demo
5. `clock.rs` - Animated analog clock

**And** `docs/animation_guide.md` tutorial:
- Section 1: Simple animation (5-10 lines)
- Section 2: Frame timing and FPS control
- Section 3: Pre-rendering for performance
- Section 4: Differential rendering for efficiency
- Section 5: Handling user input during animation
- Section 6: Best practices (memory, CPU, terminal compatibility)

**And** API documentation (rustdoc):
- All animation types documented with examples
- Performance characteristics noted
- Trade-offs explained (pre-render vs. real-time, full vs. diff rendering)

**And** README updated with animation section and GIF demo

**Prerequisites:** Story 6.1-6.5 (animation features complete)

**Technical Notes:**
- Examples demonstrate real-world use cases
- Loading spinner is especially valuable (common need)
- GIF demo can be created with terminal recording tools (asciinema)
- Critical for FR48, FR66-67, FR87
- Documentation is key to adoption

---

## Epic 7: API Design, Performance & Production Readiness

**Goal**: Transform working code into production-ready crate with clean API, comprehensive documentation, benchmarking, optimization, testing, and crates.io publication. Ensure API usability (<100 lines integration), performance targets (<25ms renders, 60fps), and professional quality.

**Value Delivered**: Dotmax is published, tested, documented, and validated - ready for developers to integrate and for the ecosystem to build upon.

**FRs Covered**: FR44-50, FR66-67, FR68-74, FR81-90 (26 FRs)

---

### Story 7.1: Design and Document Public API Surface

As a **library developer creating clean APIs**,
I want a minimal, focused public API with clear module organization,
So that users understand the library immediately and integration is simple.

**Acceptance Criteria:**

**Given** all feature implementations
**When** I design public API
**Then** `src/lib.rs` exposes:

**Top-level modules**:
```rust
pub mod grid;         // BrailleGrid, GridBuffer
pub mod terminal;     // TerminalRenderer, color detection
pub mod image;        // Image loading, rendering (feature-gated)
pub mod primitives;   // Line, circle, rectangle, polygon
pub mod density;      // Character density rendering
pub mod color;        // Color types, schemes, conversion
pub mod animation;    // Frame buffers, timing, loops
pub mod error;        // DotmaxError
```

**Top-level re-exports** (convenience):
```rust
pub use grid::BrailleGrid;
pub use terminal::TerminalRenderer;
pub use color::{Color, ColorScheme};
pub use error::DotmaxError;

#[cfg(feature = "image")]
pub use image::ImageRenderer;

// Common types for easy access
pub type Result<T> = std::result::Result<T, DotmaxError>;
```

**And** API documentation (`src/lib.rs` module doc):
```rust
//! # Dotmax - Rich Media for Terminals via Braille
//!
//! Dotmax brings images, animations, and graphics to any terminal
//! using Unicode braille characters (U+2800-U+28FF).
//!
//! ## Quick Start
//! [code example]
//!
//! ## Features
//! - Core rendering (always available)
//! - `image` - Image loading (PNG, JPG, GIF, SVG, etc.)
//! - `logging` - Debug logging support
//!
//! ## Examples
//! See `examples/` directory...
```

**And** every public type/function has rustdoc with:
- Summary (one-liner)
- Detailed description (what, why, when to use)
- Code example (where applicable)
- Errors section (if returns Result)
- Performance notes (if relevant)

**And** API review checklist:
- [ ] All public items documented
- [ ] No internal types leaked
- [ ] Error types are specific and actionable
- [ ] Builder patterns for complex config
- [ ] Sync + Send where appropriate (thread-safe)

**Prerequisites:** All previous epics (all features implemented)

**Technical Notes:**
- API surface is critical for usability (FR44-45, FR50)
- Minimize public API (easy to add, hard to remove)
- Re-exports reduce import verbosity
- Follow Rust API Guidelines
- This story is review and organization, not new code

---

### Story 7.2: Implement Comprehensive Benchmarking Suite

As a **developer validating performance targets**,
I want benchmarks for all performance-critical operations,
So that I can verify <25ms renders, 60fps, and catch regressions.

**Acceptance Criteria:**

**Given** performance-critical code paths
**When** I create benchmark suite
**Then** `benches/` directory contains:

**`benches/core_rendering.rs`**:
- `bench_grid_creation` - BrailleGrid::new()
- `bench_dot_operations` - set_dot, get_dot (1000 ops)
- `bench_unicode_conversion` - dots_to_braille_char (entire grid)
- `bench_grid_clear` - clear() operation
- `bench_terminal_render` - full grid render to terminal

**`benches/image_processing.rs`** (feature-gated):
- `bench_image_load` - Load PNG, JPG
- `bench_image_resize` - Resize to terminal dimensions
- `bench_otsu_threshold` - Threshold calculation
- `bench_floyd_steinberg_dither` - Dithering (slowest method)
- `bench_image_to_braille` - Full conversion pipeline
- **Target: <25ms for 80×24 terminal**

**`benches/animation.rs`**:
- `bench_frame_swap` - Double buffer swap
- `bench_differential_render` - Diff calculation + render
- `bench_60fps_loop` - Sustained 60fps for 1000 frames
- **Target: <16ms per frame (60fps)**

**And** CI runs benchmarks on every commit to main:
```yaml
# .github/workflows/benchmark.yml
- Compare against baseline
- Comment on PR if >10% regression
- Store results as artifacts
```

**And** `README.md` includes benchmark results table

**And** benchmarks use realistic data (not trivial cases)

**Prerequisites:** Story 1.6 (criterion setup), all feature epics

**Technical Notes:**
- Criterion.rs provides statistical analysis
- Run benchmarks on dedicated hardware for consistency
- Store baseline in git for comparisons
- Critical for FR68-74, NFR-P1-P7
- Performance is make-or-break for dotmax

---

### Story 7.3: Optimize Hot Paths Based on Benchmark Data

As a **developer meeting aggressive performance targets**,
I want to profile and optimize bottlenecks,
So that dotmax achieves <25ms renders and 60fps at <10% CPU.

**Acceptance Criteria:**

**Given** benchmark results showing bottlenecks
**When** I optimize hot paths
**Then** optimizations applied:

**Profiling process**:
1. Run `cargo flamegraph` on image rendering
2. Identify top 3 bottlenecks (likely: dithering, resize, unicode conversion)
3. Optimize each bottleneck
4. Re-benchmark to validate improvement

**Common optimizations**:
- **SIMD for image processing**: Use `std::simd` for pixel operations (4-8× speedup)
- **Reduce allocations**: Reuse buffers, use object pools
- **Parallel processing**: Use `rayon` for parallel image dithering (multi-core utilization)
- **Optimize Unicode conversion**: Pre-compute lookup table if faster than bitwise ops
- **Cache terminal capabilities**: Don't query on every render

**And** optimization rules:
- Measure first, optimize second (no premature optimization)
- Validate with benchmarks (prove speedup)
- Document why optimization needed (comment code)
- Maintain code clarity (only obscure if significant gain)

**And** target benchmarks after optimization:
- Image rendering: <20ms (beat <25ms target by 20%)
- 60fps animation: 8-10% CPU (beat <10% target)
- Memory usage: <5MB baseline, <500KB per frame

**And** update ADR with optimization decisions

**Prerequisites:** Story 7.2 (benchmarks showing bottlenecks)

**Technical Notes:**
- This story is iterative (optimize, measure, repeat)
- Focus on hot paths (80/20 rule)
- SIMD and parallelism offer biggest gains
- Balance performance vs. code complexity
- Critical for FR68-74, NFR-P1-P7
- Don't optimize until benchmarks show need

---

### Story 7.4: Implement Comprehensive Test Suite

As a **developer ensuring library correctness**,
I want unit, integration, and property-based tests,
So that dotmax is rock-solid and regressions are caught immediately.

**Acceptance Criteria:**

**Given** all library features
**When** I create test suite
**Then** tests include:

**Unit tests** (inline in modules):
- `src/grid/` - Grid operations, bounds checking, resize
- `src/terminal/` - Color conversion, capability detection
- `src/image/` - Threshold algorithms, dithering correctness
- `src/primitives/` - Bresenham correctness, shape drawing
- `src/color/` - RGB interpolation, scheme mapping
- `src/animation/` - Timing calculations, buffer swap

**Integration tests** (`tests/`):
- `tests/image_pipeline.rs` - Full image load → render flow
- `tests/animation_loop.rs` - Animation frame generation
- `tests/cross_platform.rs` - Platform-specific behavior

**Property-based tests** (using `proptest`):
- Grid operations never panic (any width, height, coordinates)
- Color conversion is bijective where possible
- Bresenham line endpoints are always correct

**Visual regression tests** (`tests/visual/`):
- Render known images to braille
- Compare against baseline outputs (stored as text)
- Detect rendering changes (intentional or bugs)

**And** test coverage:
- Core rendering: >80% line coverage
- Critical paths (grid ops, unicode conversion): 100%
- Overall: >70% coverage

**And** CI enforces:
```yaml
- cargo test --all-features
- cargo test --no-default-features
- cargo tarpaulin --out Xml (coverage reporting)
```

**And** no warnings in test code

**Prerequisites:** Story 1.2 (CI), all feature epics

**Technical Notes:**
- Use `#[cfg(test)]` modules in source files
- Integration tests in `tests/` directory
- `proptest` crate for property testing
- Critical for FR81-82, FR84-85, NFR-M4
- Tests prevent regressions during long pauses

---

### Story 7.5: Write Comprehensive Documentation and Examples

As a **developer onboarding new users**,
I want excellent documentation and examples,
So that developers can integrate dotmax in <5 minutes.

**Acceptance Criteria:**

**Given** all library features
**When** I write documentation
**Then** documentation includes:

**README.md**:
- Project description (what, why, who)
- Quick start (install, 10-line example)
- Feature list with explanations
- Performance characteristics
- Browser-rendered example images (GIFs or PNGs)
- Comparison to alternatives (drawille, protocols)
- Links to docs.rs, examples, crates.io
- License, contributing, acknowledgments

**docs.rs API documentation**:
- Every public item documented (enforced by `#![warn(missing_docs)]`)
- Module-level documentation explains purpose
- Examples for common operations
- Links between related types
- Search-friendly descriptions

**Examples** (complete set):
- `hello_braille.rs` - Minimal (10 lines)
- `load_image.rs` - Image rendering
- `animation_simple.rs` - Basic animation
- `color_schemes.rs` - Color demonstration
- `drawing_shapes.rs` - Primitives
- `loading_spinner.rs` - Practical use case
- All in `examples/README.md` with descriptions

**Guides** (`docs/`):
- `docs/getting_started.md` - Tutorial walkthrough
- `docs/performance.md` - Optimization tips
- `docs/troubleshooting.md` - Common issues
- `docs/architecture.md` - High-level design overview
- ADRs (from Epic 1) documenting decisions

**And** documentation checklist:
- [ ] All examples compile and run
- [ ] Code snippets in docs are tested (doctests)
- [ ] No broken links
- [ ] Performance claims backed by benchmarks
- [ ] Clear migration path from similar libraries

**Prerequisites:** All feature epics

**Technical Notes:**
- Documentation is critical for adoption (FR66-67, FR86-90)
- Rustdoc tests ensure examples stay current
- GIFs/images make README compelling
- Critical for NFR-DX2
- Target: 5-minute quick start (FR87)

---

### Story 7.6: Publish to crates.io and Create Release Process

As a **developer releasing dotmax to the ecosystem**,
I want publication to crates.io with versioning and release automation,
So that users can `cargo add dotmax` and the project is discoverable.

**Acceptance Criteria:**

**Given** a production-ready crate
**When** I publish to crates.io
**Then** publication includes:

**Pre-publication checklist**:
- [ ] All tests pass (`cargo test --all-features`)
- [ ] All examples compile (`cargo build --examples`)
- [ ] Benchmarks run successfully
- [ ] Documentation builds (`cargo doc --no-deps`)
- [ ] No clippy warnings (`cargo clippy --all-features`)
- [ ] Version updated in `Cargo.toml` (0.1.0 → 0.2.0 etc.)
- [ ] CHANGELOG.md updated with changes
- [ ] README.md accurate
- [ ] License files present

**Publication command**:
```bash
cargo publish --dry-run  # Verify first
cargo publish            # Actual publish
```

**And** GitHub release automation:
```yaml
# .github/workflows/release.yml
on:
  push:
    tags: ['v*']
steps:
  - Build on all platforms
  - Run full test suite
  - Publish to crates.io (if tests pass)
  - Create GitHub release with changelog
  - Upload binaries (if applicable)
```

**And** versioning strategy:
- 0.1.0 - MVP release (Epic 1-7 complete)
- 0.x.y - Pre-1.0 (breaking changes allowed in minor versions)
- 1.0.0 - Stable API (semver guarantees begin)

**And** post-publication:
- Verify crates.io page looks correct
- Test installation: `cargo add dotmax`
- Announce on Reddit (/r/rust), Discord, etc.

**Prerequisites:** Story 7.1-7.5 (all quality checks), Story 1.1 (metadata configured)

**Technical Notes:**
- crates.io is canonical Rust package registry
- Critical for FR61, NFR-L2
- Publication is one-way (can't delete versions)
- Verify with --dry-run first
- Post-1.0: no breaking changes without major version bump

---

### Story 7.7: Create Proof-of-Concept Integration (yazi or bat)

As a **developer validating real-world usability**,
I want dotmax integrated into an existing Rust CLI tool,
So that I confirm the API works in practice and get external validation.

**Acceptance Criteria:**

**Given** dotmax published to crates.io
**When** I create POC integration
**Then** integration target chosen:

**Option 1: yazi** (file manager):
- Add image preview using dotmax braille rendering
- PR to yazi with feature flag `braille-preview`
- Demonstrates: image loading, terminal rendering, resize handling

**Option 2: bat** (file viewer):
- Add braille image rendering for image files
- PR to bat with feature flag `braille-images`
- Demonstrates: image pipeline, color support

**And** integration requirements:
- <100 lines of code to integrate dotmax
- Clear performance improvement over alternatives (if applicable)
- Documented in README as example integration
- At least 1 maintainer/user feedback collected

**And** integration lessons learned documented:
- API pain points encountered
- Missing features identified
- Usability improvements needed
- Performance in real-world use

**And** based on feedback, refine dotmax API if needed

**Prerequisites:** Story 7.6 (published to crates.io)

**Technical Notes:**
- Real-world integration validates library design
- Critical for FR44 (<100 lines), external validation goal
- Choose target with active maintainers
- yazi and bat are both popular Rust CLI tools
- Feedback loop improves dotmax quality
- This is success criterion from PRD

---

---

## FR Coverage Matrix

### Complete FR-to-Story Traceability

**Epic 1: Foundation & Project Setup (14 FRs)**
- FR61: Story 1.1 (Cargo project), Story 7.6 (crates.io publish)
- FR62: Story 1.3 (feature flags), Story 7.3 (binary size optimization)
- FR63: Story 1.3 (minimal dependencies)
- FR64: Story 1.3 (feature flags for optional capabilities)
- FR65: Story 1.1 (stable Rust compilation), Story 1.2 (MSRV CI check)
- FR66: Story 1.7 (rustdoc), Story 7.5 (comprehensive docs)
- FR67: Story 1.7 (examples directory), Story 7.5 (example suite)
- FR75-80: Story 1.2 (cross-platform CI testing)

**Epic 2: Core Braille Rendering Engine (23 FRs)**
- FR1: Story 2.1 (BrailleGrid creation)
- FR2: Story 2.1 (set_dot)
- FR3: Story 2.1 (clear operations)
- FR4: Story 2.3 (terminal rendering)
- FR5: Story 2.2 (Unicode braille conversion)
- FR6: Story 2.1 (query grid state)
- FR7: Story 2.5 (resize event handling)
- FR8: Story 2.6 (color support for grid)
- FR51-55: Story 2.3 (terminal abstraction with ratatui/crossterm)
- FR56-60: Story 2.4 (error handling), Story 2.7 (logging)

**Epic 3: 2D Image Rendering Pipeline (12 FRs)**
- FR9: Story 3.1 (load from file paths)
- FR10: Story 3.1 (load from byte buffers)
- FR11: Story 3.6 (SVG rendering)
- FR12: Story 3.2 (auto-resize with aspect ratio)
- FR13: Story 3.2 (manual dimension specification)
- FR14: Story 3.3 (grayscale conversion), Story 3.5 (binary to braille)
- FR15: Story 3.4 (dithering methods)
- FR16: Story 3.3 (monochrome mode via threshold)
- FR17: Story 3.7 (color mode rendering)
- FR18: Story 3.3 (Otsu thresholding)
- FR19: Story 3.3 (brightness/contrast/gamma adjustment)
- FR20: Story 3.1 (error handling for malformed images)

**Epic 4: Drawing Primitives & Density Rendering (11 FRs)**
- FR21: Story 4.1 (Bresenham line drawing)
- FR22: Story 4.2 (Bresenham circle drawing)
- FR23: Story 4.3 (rectangle drawing)
- FR24: Story 4.3 (polygon drawing)
- FR25: Story 4.3 (fill operations), Story 4.4 (density patterns)
- FR26: Story 4.1 (line thickness)
- FR27: Story 4.5 (color for drawing operations)
- FR28-31: Story 4.4 (character density rendering, intensity mapping, custom charsets)

**Epic 5: Color System & Visual Schemes (8 FRs)**
- FR32: Story 2.6 (RGB color assignment to cells)
- FR33: Story 5.2 (RGB to ANSI conversion)
- FR34: Story 5.3 (predefined color schemes)
- FR35: Story 5.4 (custom scheme creation)
- FR36: Story 5.5 (apply schemes to intensity buffers)
- FR37: Story 5.1 (terminal capability detection)

**Epic 6: Animation & Frame Management (6 FRs)**
- FR38: Story 6.1 (frame-by-frame rendering)
- FR39: Story 6.2 (frame timing control)
- FR40: Story 6.2 (animation loops with FPS)
- FR41: Story 6.1 (frame buffer management)
- FR42: Story 6.4 (pre-rendering)
- FR43: Story 6.1 (flicker-free via double buffering), Story 6.5 (differential rendering)

**Epic 7: API Design, Performance & Production Readiness (26 FRs)**
- FR44: Story 7.1 (API simplicity), Story 3.8 (high-level image API), Story 6.3 (animation loop helper)
- FR45: Story 7.1 (Rust idioms), Story 2.4 (Result types)
- FR46: Story 7.1 (builder patterns), Story 3.8 (ImageRenderer builder)
- FR47: Story 6.3 (async support in animation loop)
- FR48: Story 7.5 (example suite)
- FR49: Story 1.3 (feature flags)
- FR50: Story 7.1 (minimal API surface)
- FR68-69: Story 7.2 (image rendering benchmarks), Story 7.3 (optimization)
- FR70: Story 7.2 (animation benchmarks), Story 7.3 (CPU optimization)
- FR71-74: Story 7.2 (memory/startup benchmarks), Story 7.3 (optimization)
- FR81-82: Story 7.4 (unit and integration tests)
- FR83: Story 1.6 (criterion setup), Story 7.2 (comprehensive benchmarks)
- FR84-85: Story 7.4 (visual regression tests, property-based tests)
- FR86-90: Story 7.5 (comprehensive documentation, guides, troubleshooting, ADRs)

**Coverage Validation**: All 90 functional requirements traced to specific stories ✓

---

## Summary

### Epic Breakdown Complete

**7 Epics, 47 Stories** - Complete tactical breakdown of dotmax implementation

**Epic Totals:**
- **Epic 1**: Foundation & Project Setup - 7 stories
- **Epic 2**: Core Braille Rendering Engine - 7 stories
- **Epic 3**: 2D Image Rendering Pipeline - 8 stories
- **Epic 4**: Drawing Primitives & Density Rendering - 5 stories
- **Epic 5**: Color System & Visual Schemes - 5 stories
- **Epic 6**: Animation & Frame Management - 6 stories
- **Epic 7**: API Design, Performance & Production Readiness - 7 stories + POC integration

**Total Coverage**: All 90 functional requirements from PRD mapped to specific stories with detailed BDD acceptance criteria.

### Key Implementation Characteristics

**Altitude Shift from PRD:**
- **PRD FRs (Strategic)**: WHAT capabilities exist (high-level, capability-focused)
- **Epic Stories (Tactical)**: HOW they're implemented (detailed AC, edge cases, performance targets, error handling)

**Story Quality:**
- All stories use BDD-style acceptance criteria (Given/When/Then/And)
- Prerequisites ensure sequential dependencies (no forward references)
- Technical notes provide implementation guidance and crabmusic references
- Performance targets specified where critical (<25ms renders, 60fps, <5MB memory)
- Vertically sliced (deliver complete functionality per story, not just layers)

**Next Steps in BMad Method:**

1. **UX Design Workflow** (SKIP - dotmax is a library, no UI)
   - Not applicable for this project

2. **Architecture Workflow** - Run: `/bmad:bmm:workflows:architecture`
   - Technical architecture decisions
   - Dependency management strategy
   - Module structure and boundaries
   - Performance optimization approach
   - Terminal abstraction layer design
   - ADRs for major technical choices
   - **Will UPDATE epics.md** with technical details in story notes

3. **Phase 4: Implementation** - After architecture complete
   - Each story pulls context from: PRD (why) + epics.md (what/how) + Architecture (technical)
   - Use `create-story` workflow to generate individual story implementation plans
   - Stories in epics.md remain single source of truth for requirements

### Living Document Notice

This epic breakdown is the **initial version (v1.0)**. It will be updated:

- **After Architecture**: Technical details added to story notes (module structure, data models, API contracts, tech stack, deployment)
- **During Implementation**: Stories may be refined as edge cases discovered, but core scope remains stable
- **epics.md evolves** through the BMad Method workflow chain before implementation begins

---

_Created: 2025-11-14_
_Author: John (PM Agent) working with Frosty_
_Status: INITIAL VERSION - Ready for Architecture Workflow_
_Next Action: Run architecture workflow to add technical decisions to story technical notes_

---

## Summary

{{epic_breakdown_summary}}

---

_For implementation: Use the `create-story` workflow to generate individual story implementation plans from this epic breakdown._

_This document will be updated after UX Design and Architecture workflows to incorporate interaction details and technical decisions._
