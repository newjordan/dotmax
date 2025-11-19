# Epic Technical Specification: Core Braille Rendering Engine

Date: 2025-11-17
Author: Frosty
Epic ID: 2
Status: Draft

---

## Overview

Epic 2 extracts and professionalizes the battle-tested BrailleGrid rendering system from the crabmusic project (~2,000-3,000 lines of proven code). This epic establishes the atomic foundation of dotmax—the core rendering engine that converts 2×4 dot matrix patterns to Unicode braille characters (U+2800-U+28FF) and outputs to any terminal via ratatui/crossterm.

This is the most critical epic in the project. Everything else—image rendering, primitives, color, animation—builds on this foundation. The extraction strategy follows a **copy-refactor-test** approach: preserve the working behavior from crabmusic, strip audio dependencies, refactor into clean modules, lock correctness with tests, and establish the error handling and logging infrastructure that will be used throughout dotmax.

## Objectives and Scope

### In Scope

1. **BrailleGrid Data Structure** - Core 2×4 dot matrix grid with dot manipulation (Story 2.1)
2. **Unicode Braille Conversion** - Dot patterns → Unicode characters U+2800-U+28FF (Story 2.2)
3. **Terminal Rendering Abstraction** - GridBuffer and TerminalRenderer via ratatui/crossterm (Story 2.3)
4. **Comprehensive Error Handling** - DotmaxError enum with thiserror, no panics (Story 2.4)
5. **Terminal Resize Handling** - Dynamic grid adjustment on terminal size changes (Story 2.5)
6. **Color Support Foundation** - Per-cell RGB color assignment infrastructure (Story 2.6)
7. **Debug Logging** - Structured tracing throughout rendering pipeline (Story 2.7)

### Out of Scope

- **Image rendering** (Epic 3) - This epic only creates the grid foundation
- **Drawing primitives** (Epic 4) - Lines, circles, shapes come later
- **Color schemes** (Epic 5) - Only raw RGB support, no intensity mapping yet
- **Animation** (Epic 6) - Frame management builds on this foundation
- **Performance optimization** (Epic 7) - First make it correct, then make it fast
- **Audio-reactive features** - These stay in crabmusic permanently

## System Architecture Alignment

### Architecture Decisions Referenced

- **ADR 0001**: Use Unicode Braille for Terminal Rendering (foundational to this epic)
- **ADR 0002**: Use thiserror for Error Handling (Story 2.4 implements this)
- **ADR 0004**: Terminal Backend Abstraction via Trait (Story 2.3 implements TerminalBackend trait)
- **ADR 0005**: Brownfield Extraction Strategy - Copy-Refactor-Test (entire epic follows this)

### Module Structure Alignment

Per architecture document section "Project Structure", this epic creates:

```
src/
├── lib.rs                    # Re-exports BrailleGrid, TerminalRenderer, DotmaxError
├── error.rs                  # Story 2.4 - DotmaxError enum
├── grid.rs                   # Story 2.1 & 2.2 - BrailleGrid + Unicode conversion
└── render.rs                 # Story 2.3 - TerminalBackend trait + TerminalRenderer
```

### Dependency Constraints

Core dependencies (from Epic 1, Story 1.3):
- `ratatui = "0.29"` - Terminal UI framework
- `crossterm = "0.29"` - Cross-platform terminal I/O
- `thiserror = "2.0"` - Error handling derive macros
- `tracing = "0.1"` - Structured logging

Epic 2 uses **only** these core dependencies. No feature-gated dependencies yet (image/svg come in Epic 3).

## Detailed Design

### Services and Modules

| Module | Responsibility | Key Types | Story | Lines (Est.) |
|--------|---------------|-----------|-------|--------------|
| **src/error.rs** | Error type definitions for all dotmax operations | `DotmaxError` enum | 2.4 | ~150 |
| **src/grid.rs** | BrailleGrid data structure, dot manipulation, Unicode conversion | `BrailleGrid`, `BrailleCell` | 2.1, 2.2, 2.5, 2.6 | ~600 |
| **src/render.rs** | Terminal abstraction and rendering pipeline | `TerminalBackend` trait, `TerminalRenderer`, `TerminalCapabilities` | 2.3, 2.5 | ~500 |
| **src/lib.rs** | Public API re-exports and documentation | N/A (re-exports only) | 2.1-2.7 | ~100 |

**Module Dependencies:**

```
lib.rs (public API)
  ├── error.rs (no internal deps)
  ├── grid.rs → error.rs
  └── render.rs → grid.rs, error.rs
```

**Extraction Mapping from Crabmusic:**

- `crabmusic/src/visualization/braille.rs` (~500 lines) → `dotmax/src/grid.rs`
- `crabmusic/src/visualization/grid_buffer.rs` (~200 lines) → merged into `dotmax/src/grid.rs`
- `crabmusic/src/terminal/renderer.rs` (~450 lines) → `dotmax/src/render.rs`
- Audio/effects/config code → **discarded** (not extracted)

**Total Extraction:** ~1,150 lines from crabmusic, refactored and cleaned to ~1,250 lines in dotmax with improved error handling and logging.

### Data Models and Contracts

#### BrailleGrid (src/grid.rs)

**Core rendering surface** - stores 2×4 dot matrix per terminal cell

```rust
pub struct BrailleGrid {
    width: usize,                // Width in braille cells
    height: usize,               // Height in braille cells
    dots: Vec<Vec<[bool; 8]>>,  // 2D grid, each cell has 8 dots
    colors: Option<Vec<Vec<Color>>>, // Optional per-cell colors (Story 2.6)
}

impl BrailleGrid {
    // Story 2.1: Construction and manipulation
    pub fn new(width: usize, height: usize) -> Result<Self, DotmaxError>;
    pub fn set_dot(&mut self, x: usize, y: usize, dot_index: u8, value: bool) -> Result<(), DotmaxError>;
    pub fn get_dot(&self, x: usize, y: usize, dot_index: u8) -> Result<bool, DotmaxError>;
    pub fn clear(&mut self);
    pub fn clear_region(&mut self, x: usize, y: usize, width: usize, height: usize) -> Result<(), DotmaxError>;
    pub fn dimensions(&self) -> (usize, usize);

    // Story 2.2: Unicode conversion
    pub fn to_unicode_grid(&self) -> Vec<Vec<char>>;
    pub fn cell_to_braille_char(&self, x: usize, y: usize) -> Result<char, DotmaxError>;

    // Story 2.5: Resize support
    pub fn resize(&mut self, new_width: usize, new_height: usize) -> Result<(), DotmaxError>;

    // Story 2.6: Color support
    pub fn set_cell_color(&mut self, x: usize, y: usize, color: Color) -> Result<(), DotmaxError>;
    pub fn get_cell_color(&self, x: usize, y: usize) -> Option<Color>;
    pub fn enable_color_support(&mut self);
}
```

**Dot Indexing Convention** (Unicode Braille Standard):

```
Braille cell (8 dots):
0 3
1 4
2 5
6 7

Unicode formula: U+2800 + bitfield
Bitfield: dots[0]<<0 | dots[1]<<1 | ... | dots[7]<<7
```

#### Color (src/grid.rs, Story 2.6)

**RGB color representation**

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
}
```

#### DotmaxError (src/error.rs, Story 2.4)

**Comprehensive error type for all operations**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DotmaxError {
    #[error("Invalid grid dimensions: width={width}, height={height}")]
    InvalidDimensions { width: usize, height: usize },

    #[error("Out of bounds access: ({x}, {y}) in grid of size ({width}, {height})")]
    OutOfBounds { x: usize, y: usize, width: usize, height: usize },

    #[error("Invalid dot index: {index} (must be 0-7)")]
    InvalidDotIndex { index: u8 },

    #[error("Terminal error: {0}")]
    Terminal(#[from] std::io::Error),

    #[error("Terminal backend error: {0}")]
    TerminalBackend(String),

    #[error("Unicode conversion failed for cell ({x}, {y})")]
    UnicodeConversion { x: usize, y: usize },
}
```

**Error Handling Contract:**
- **Zero panics** - All public functions return `Result<T, DotmaxError>`
- **Meaningful context** - Errors include coordinates, dimensions, actual values
- **Source chaining** - `#[from]` preserves underlying I/O errors
- **User-facing messages** - `#[error("...")]` provides actionable feedback

#### TerminalBackend Trait (src/render.rs, Story 2.3)

**Abstraction for terminal I/O** - reduces ratatui lock-in

```rust
pub trait TerminalBackend {
    fn size(&self) -> Result<(u16, u16), DotmaxError>;
    fn render(&mut self, content: &str) -> Result<(), DotmaxError>;
    fn clear(&mut self) -> Result<(), DotmaxError>;
    fn capabilities(&self) -> TerminalCapabilities;
}

pub struct TerminalCapabilities {
    pub supports_color: bool,
    pub supports_truecolor: bool,
    pub supports_unicode: bool,
}
```

#### TerminalRenderer (src/render.rs, Story 2.3)

**High-level rendering API** - converts BrailleGrid to terminal output

```rust
pub struct TerminalRenderer {
    backend: Box<dyn TerminalBackend>,
}

impl TerminalRenderer {
    pub fn new() -> Result<Self, DotmaxError>;
    pub fn with_backend(backend: Box<dyn TerminalBackend>) -> Self;
    pub fn render(&mut self, grid: &BrailleGrid) -> Result<(), DotmaxError>;
    pub fn clear(&mut self) -> Result<(), DotmaxError>;
    pub fn get_terminal_size(&self) -> Result<(u16, u16), DotmaxError>;
}
```

### APIs and Interfaces

#### Public API Surface (src/lib.rs)

**Exported Types:**

```rust
// Core types (always available)
pub use grid::BrailleGrid;
pub use grid::Color;
pub use render::{TerminalBackend, TerminalRenderer, TerminalCapabilities};
pub use error::DotmaxError;

// Re-export Result type for convenience
pub type Result<T> = std::result::Result<T, DotmaxError>;
```

**Usage Example (Basic):**

```rust
use dotmax::{BrailleGrid, TerminalRenderer};

fn main() -> dotmax::Result<()> {
    // Create grid
    let mut grid = BrailleGrid::new(80, 24)?;

    // Set some dots
    grid.set_dot(10, 10, 0, true)?; // Top-left dot
    grid.set_dot(10, 10, 4, true)?; // Middle-right dot

    // Render to terminal
    let mut renderer = TerminalRenderer::new()?;
    renderer.render(&grid)?;

    Ok(())
}
```

**Usage Example (With Color):**

```rust
use dotmax::{BrailleGrid, Color, TerminalRenderer};

fn main() -> dotmax::Result<()> {
    let mut grid = BrailleGrid::new(80, 24)?;
    grid.enable_color_support();

    // Set colored cell
    grid.set_cell_color(5, 5, Color::rgb(255, 0, 0))?; // Red
    grid.set_dot(5, 5, 0, true)?;

    let mut renderer = TerminalRenderer::new()?;
    renderer.render(&grid)?;

    Ok(())
}
```

#### Method Signatures (Contract)

**BrailleGrid Core Operations:**

| Method | Input | Output | Purpose | Story |
|--------|-------|--------|---------|-------|
| `new(width, height)` | `usize, usize` | `Result<Self>` | Create grid with dimensions | 2.1 |
| `set_dot(x, y, dot_index, value)` | `usize, usize, u8, bool` | `Result<()>` | Set individual dot (0-7 index) | 2.1 |
| `get_dot(x, y, dot_index)` | `usize, usize, u8` | `Result<bool>` | Read dot value | 2.1 |
| `clear()` | N/A | N/A | Reset all dots to false | 2.1 |
| `clear_region(x, y, w, h)` | `usize × 4` | `Result<()>` | Clear rectangular area | 2.1 |
| `dimensions()` | N/A | `(usize, usize)` | Get grid size | 2.1 |
| `to_unicode_grid()` | N/A | `Vec<Vec<char>>` | Convert entire grid to chars | 2.2 |
| `cell_to_braille_char(x, y)` | `usize, usize` | `Result<char>` | Convert single cell to Unicode | 2.2 |
| `resize(new_w, new_h)` | `usize, usize` | `Result<()>` | Adjust grid dimensions | 2.5 |
| `enable_color_support()` | N/A | N/A | Allocate color buffer | 2.6 |
| `set_cell_color(x, y, color)` | `usize, usize, Color` | `Result<()>` | Assign RGB to cell | 2.6 |
| `get_cell_color(x, y)` | `usize, usize` | `Option<Color>` | Read cell color if set | 2.6 |

**TerminalRenderer Operations:**

| Method | Input | Output | Purpose | Story |
|--------|-------|--------|---------|-------|
| `new()` | N/A | `Result<Self>` | Create with default backend | 2.3 |
| `render(grid)` | `&BrailleGrid` | `Result<()>` | Output grid to terminal | 2.3 |
| `clear()` | N/A | `Result<()>` | Clear terminal | 2.3 |
| `get_terminal_size()` | N/A | `Result<(u16, u16)>` | Query terminal dimensions | 2.3 |

**Error Contract:**

All methods return `Result<T, DotmaxError>` or panic-free primitives. No method in the public API may panic under any circumstances (panics are bugs).

### Workflows and Sequencing

#### Rendering Pipeline (Primary Flow)

```
1. Application Code
   ↓
2. Create BrailleGrid
   → BrailleGrid::new(width, height)
   → Allocates dots: Vec<Vec<[bool; 8]>>
   ↓
3. Set Dots (application draws)
   → grid.set_dot(x, y, dot_index, value)
   → Updates internal dots array
   ↓
4. Convert to Unicode
   → grid.to_unicode_grid()
   → For each cell: apply bitfield formula
   → Bitfield → U+2800 + value
   ↓
5. Render to Terminal
   → renderer.render(&grid)
   → Calls to_unicode_grid() internally
   → Builds ratatui Frame with braille chars
   → Outputs via crossterm to terminal
   ↓
6. Terminal Display
   → User sees braille characters
```

#### Unicode Conversion Algorithm (Story 2.2)

```
Input: BrailleCell with 8 boolean dots [d0, d1, d2, d3, d4, d5, d6, d7]

Step 1: Calculate bitfield
  bitfield = 0
  for i in 0..8:
    if dots[i] == true:
      bitfield |= (1 << i)

Step 2: Convert to Unicode
  unicode_value = 0x2800 + bitfield
  braille_char = char::from_u32(unicode_value).unwrap()

Output: Single Unicode braille character

Example:
  dots = [true, false, true, false, false, false, false, false]
  bitfield = 0b00000101 = 5
  unicode_value = 0x2800 + 5 = 0x2805 = '⠅'
```

#### Terminal Resize Event Flow (Story 2.5)

```
1. Terminal resize event detected
   → crossterm emits Event::Resize(new_width, new_height)
   ↓
2. Application queries new size
   → renderer.get_terminal_size()
   ↓
3. Application resizes grid
   → grid.resize(new_width_in_cells, new_height_in_cells)
   → Preserves existing dots where possible
   → Clears new cells if grid expanded
   → Truncates if grid shrunk
   ↓
4. Re-render
   → renderer.render(&grid)
   → Grid now matches terminal dimensions
```

#### Color Rendering Flow (Story 2.6)

```
1. Enable color support
   → grid.enable_color_support()
   → Allocates colors: Option<Vec<Vec<Color>>> → Some(vec![...])
   ↓
2. Set cell colors
   → grid.set_cell_color(x, y, Color::rgb(r, g, b))
   ↓
3. Render with colors
   → renderer.render(&grid)
   → For each cell:
       - Convert dots to braille char
       - If color exists: apply ANSI color code
       - Output colored braille to terminal
```

#### Error Handling Flow (Story 2.4)

```
User calls API method
   ↓
Input validation
   → Dimensions valid? (width, height > 0)
   → Coordinates in bounds? (x < width, y < height)
   → Dot index valid? (0-7)
   ↓
   [Invalid] → Return Err(DotmaxError::InvalidDimensions | OutOfBounds | InvalidDotIndex)
   [Valid] → Continue
   ↓
Execute operation
   → May call underlying libraries (ratatui, crossterm)
   → I/O errors wrapped via #[from] → DotmaxError::Terminal
   ↓
Return Result
   → Ok(value) on success
   → Err(DotmaxError::...) on failure with context
```

## Non-Functional Requirements

### Performance

Epic 2 focuses on **correctness first, performance later**. The primary goal is to extract working code and lock behavior with tests. Performance optimization comes in Epic 7 after benchmarking infrastructure is established.

**However, some baseline targets must be met:**

**NFR-P1: Unicode Conversion Speed**
- Target: <1μs per cell conversion (dots → braille char)
- Rationale: 80×24 grid = 1,920 cells. At 1μs/cell = 1.92ms total (well under budget)
- Measurement: Microbenchmark in Story 2.2 acceptance criteria
- Implementation: Direct bitfield calculation (no lookup tables needed)

**NFR-P2: Grid Operations**
- `set_dot()`: O(1) - direct array access
- `get_dot()`: O(1) - direct array access
- `clear()`: O(n) where n = total cells - acceptable for infrequent operation
- `resize()`: O(n) - allocates new storage, copies existing data

**NFR-P3: Memory Efficiency (Baseline)**
- BrailleGrid storage: `width × height × 8 bools` + optional color buffer
- For 80×24 grid: 1,920 cells × 8 bools = 15,360 bytes (~15KB) for dots
- With colors: + (1,920 cells × 3 bytes RGB) = additional 5,760 bytes (~6KB)
- **Total: ~21KB for standard terminal** - well under <5MB baseline target (NFR-P3)

**NFR-P4: No Premature Optimization**
- Story 2.1 uses `Vec<Vec<[bool; 8]>>` - simple, clear, not optimized
- Could pack dots into `Vec<u8>` later (8 bools → 1 byte) if benchmarks show issues
- Defer optimization to Epic 7 after measuring actual hotspots

**Performance Validation:**
- Story 2.2 includes criterion benchmark for Unicode conversion
- Target: Pass benchmark showing <1μs per cell
- No broader performance work until Epic 7

### Security

Epic 2 establishes the security foundation for dotmax.

**NFR-S1: Memory Safety (Zero Unsafe Code)**
- Epic 2 contains **no unsafe blocks** - rely entirely on Rust's memory safety guarantees
- All array access bounds-checked via Result types (no panics on out-of-bounds)
- No buffer overflows possible (Rust prevents this)

**NFR-S2: Input Validation**
- **Dimensions**: `BrailleGrid::new()` validates width, height > 0 → Err(InvalidDimensions) if violated
- **Coordinates**: All `set_dot()`, `get_dot()`, `set_cell_color()` validate bounds → Err(OutOfBounds)
- **Dot index**: Validate 0-7 range → Err(InvalidDotIndex)
- **Resource limits**: Max grid dimensions to prevent OOM attacks
  ```rust
  const MAX_GRID_WIDTH: usize = 10_000;
  const MAX_GRID_HEIGHT: usize = 10_000;
  // Validated in BrailleGrid::new()
  ```

**NFR-S3: No Panics (Zero Panic Policy)**
- **Contract**: No public API method may panic under any circumstances
- **Implementation**: All operations return `Result<T, DotmaxError>`
- **Enforcement**:
  - Code review checks for `.unwrap()`, `.expect()`, `panic!()` in public API paths
  - Unit tests cover edge cases: zero dimensions, out-of-bounds, invalid indices
  - Integration tests stress-test boundary conditions

**NFR-S4: Dependency Security**
- All dependencies (ratatui, crossterm, thiserror, tracing) use permissive licenses (MIT/Apache-2.0)
- No viral licenses (GPL, AGPL) in dependency tree
- cargo-deny configured in Epic 1 (Story 1.4) - runs in CI to detect vulnerabilities

**Security Non-Concerns for Epic 2:**
- No authentication/authorization (library, not service)
- No network I/O (terminal rendering only)
- No cryptography required
- No sensitive data handling (just braille dots)

### Reliability/Availability

**NFR-R1: Cross-Platform Consistency**
- BrailleGrid behavior identical on Windows, Linux, macOS
- Terminal I/O differences abstracted by ratatui/crossterm (they handle platform quirks)
- CI testing on all three platforms (Epic 1, Story 1.2 established this)
- Unicode braille characters (U+2800-U+28FF) are standards-based - work everywhere

**NFR-R2: Error Handling Robustness**
- **Graceful degradation**: If terminal lacks Unicode support, return clear error (don't crash)
- **Actionable errors**: `DotmaxError` variants include context (coordinates, dimensions, actual values)
- **Source preservation**: I/O errors wrapped via `#[from]` - users can inspect underlying cause
- **No silent failures**: All operations return Result - caller must handle explicitly

**NFR-R3: Terminal Compatibility**
- **Assumption**: Terminal supports Unicode braille range (U+2800-U+28FF)
- **Detection**: `TerminalCapabilities.supports_unicode` flag (Story 2.3)
- **Fallback**: If Unicode not supported, warn user with clear message (don't render garbage)
- **Tested terminals**: PowerShell, bash, zsh, Windows Terminal, Alacritty, iTerm2

**NFR-R4: Resize Handling (Story 2.5)**
- Terminal resize events handled gracefully
- `grid.resize()` preserves existing dots where possible
- No data corruption on resize
- Application can query new size via `renderer.get_terminal_size()`

**NFR-R5: State Consistency**
- BrailleGrid internal state always valid after any public method call
- Invariants maintained:
  - `dots.len() == height`, `dots[i].len() == width` for all rows
  - If `colors.is_some()`, then color buffer matches grid dimensions
  - All dot indices in range 0-7
- Enforced via private methods and validation in constructors/mutators

### Observability

**NFR-O1: Structured Logging with `tracing`**

Epic 2 Story 2.7 adds comprehensive debug logging throughout the rendering pipeline.

**Log Levels Used:**

| Level | Usage | Example |
|-------|-------|---------|
| `error!` | Operation failed, user needs to know | "Failed to render grid: terminal error" |
| `warn!` | Unexpected but recoverable | "Terminal lacks Unicode support" |
| `info!` | Major operations | "Grid created: 80×24", "Rendered 1920 cells" |
| `debug!` | Detailed flow | "Converting cell (10, 5) to braille", "Resize: 80×24 → 100×30" |
| `trace!` | Hot path internals | "set_dot(5, 5, 3, true)" - only if needed |

**Instrumented Functions (Story 2.7):**

```rust
#[instrument]
pub fn new(width: usize, height: usize) -> Result<Self, DotmaxError> {
    info!("Creating BrailleGrid: {}×{}", width, height);
    // ... implementation
}

#[instrument(skip(self))]
pub fn render(&mut self, grid: &BrailleGrid) -> Result<(), DotmaxError> {
    debug!("Rendering grid: {}×{}", grid.width(), grid.height());
    // ... implementation
}
```

**NFR-O2: Error Context for Debugging**

All `DotmaxError` variants include actionable context:

```rust
// Good - tells user exactly what went wrong
Err(DotmaxError::OutOfBounds {
    x: 100,
    y: 50,
    width: 80,
    height: 24
})

// Message: "Out of bounds access: (100, 50) in grid of size (80, 24)"
```

**NFR-O3: No Logging in Hot Paths (Without TRACE)**

- `set_dot()` and `get_dot()` do NOT log at debug level (called thousands of times)
- Use `trace!` level only if profiling shows no performance impact
- Rendering pipeline logs at `debug!` level (called once per frame)

**NFR-O4: User Control Over Logging**

- Library uses `tracing` crate - does NOT initialize subscriber
- Application controls log level via `tracing-subscriber`:
  ```rust
  // User code (not in library)
  use tracing_subscriber;

  tracing_subscriber::fmt()
      .with_max_level(tracing::Level::DEBUG)
      .init();
  ```
- Default: No logging output unless user initializes subscriber

## Dependencies and Integrations

### Core Dependencies (Epic 2 Uses Only These)

All dependencies established in Epic 1, Story 1.3. Epic 2 uses **only** core dependencies - no feature-gated deps yet.

| Dependency | Version | Purpose | License | Used In |
|------------|---------|---------|---------|---------|
| **ratatui** | 0.29 | Terminal UI framework | MIT | Story 2.3 (TerminalRenderer) |
| **crossterm** | 0.29 | Cross-platform terminal I/O | MIT | Story 2.3 (backend impl) |
| **thiserror** | 2.0 | Error handling derive macros | MIT/Apache-2.0 | Story 2.4 (DotmaxError) |
| **tracing** | 0.1 | Structured logging | MIT | Story 2.7 (debug logging) |

**Dev Dependencies (Testing Only):**

| Dependency | Version | Purpose | Used In |
|------------|---------|---------|---------|
| **criterion** | 0.7 | Benchmarking with statistics | Story 2.2 (Unicode benchmark) |
| **tracing-subscriber** | 0.3 | Log output in tests | Story 2.7 (test logging) |

### Integration Points

**External Systems:**

1. **Terminal (via ratatui/crossterm)**
   - **Purpose**: Output braille characters to user's terminal
   - **Interface**: TerminalBackend trait (Story 2.3)
   - **Data Flow**: BrailleGrid → Unicode chars → ratatui Frame → crossterm → terminal stdout
   - **Error Handling**: I/O errors wrapped in DotmaxError::Terminal

2. **Crabmusic Source Code (extraction)**
   - **Purpose**: Copy proven braille rendering code
   - **Files**:
     - `crabmusic/src/visualization/braille.rs` → `dotmax/src/grid.rs`
     - `crabmusic/src/visualization/grid_buffer.rs` → merged into `dotmax/src/grid.rs`
     - `crabmusic/src/terminal/renderer.rs` → `dotmax/src/render.rs`
   - **Process**: Copy → strip audio deps → refactor → test → commit
   - **Reference**: https://github.com/newjordan/crabmusic

**Internal Module Integrations:**

```
src/lib.rs (public API)
  ├─ Exports: BrailleGrid, TerminalRenderer, DotmaxError, Color
  └─ Feature gates: None (Epic 2 has no features)

src/error.rs (Story 2.4)
  ├─ Consumed by: grid.rs, render.rs
  └─ Exports: DotmaxError enum

src/grid.rs (Stories 2.1, 2.2, 2.5, 2.6)
  ├─ Uses: error::DotmaxError
  ├─ Exports: BrailleGrid, Color
  └─ Consumed by: render.rs, lib.rs

src/render.rs (Stories 2.3, 2.5)
  ├─ Uses: grid::BrailleGrid, error::DotmaxError, ratatui, crossterm
  ├─ Exports: TerminalBackend trait, TerminalRenderer, TerminalCapabilities
  └─ Consumed by: lib.rs
```

### Dependency Justifications (per Architecture)

**ratatui (0.29):**
- **Why**: Industry standard for Rust TUI applications
- **Alternatives considered**: termion (less active), ncurses bindings (C FFI complexity)
- **Lock-in mitigation**: TerminalBackend trait abstracts terminal I/O
- **Risk**: Low - stable API, active maintenance, large ecosystem

**crossterm (0.29):**
- **Why**: Cross-platform terminal manipulation (Windows, Linux, macOS)
- **Alternatives considered**: termion (Unix-only), platform-specific APIs
- **Lock-in mitigation**: Used only in DefaultTerminal implementation
- **Risk**: Low - companion to ratatui, well-tested

**thiserror (2.0):**
- **Why**: Zero-boilerplate error types for library (ADR 0002)
- **Alternatives considered**: anyhow (wrong for libraries), manual impl (verbose)
- **Lock-in mitigation**: Could manually impl std::error::Error if needed
- **Risk**: Minimal - simple derive macro, stable

**tracing (0.1):**
- **Why**: Structured logging standard in Rust ecosystem
- **Alternatives considered**: log crate (less structured), println! (not production-ready)
- **Lock-in mitigation**: Could swap for `log` facade if needed
- **Risk**: Low - widely adopted, stable API

### Version Constraints and MSRV

**Minimum Supported Rust Version (MSRV): 1.70**
- Documented in Cargo.toml: `rust-version = "1.70"`
- Enforced in CI (Epic 1, Story 1.2)
- Dependencies compatible with Rust 1.70

**Dependency Pinning:**
- Using `^` semver ranges (e.g., `ratatui = "0.29"` → `^0.29`)
- Allows patch updates automatically (0.29.1, 0.29.2)
- Blocks breaking minor updates (0.30 requires explicit bump)

**Security Monitoring:**
- cargo-deny configured (Epic 1, Story 1.4)
- CI runs `cargo audit` to detect known vulnerabilities
- cargo-deny checks licenses, bans, sources

### External References

**Crabmusic Repository:**
- URL: https://github.com/newjordan/crabmusic
- Extraction source for Epic 2
- License: Check before extraction (ensure compatible)

**Unicode Braille Standard:**
- Range: U+2800 to U+28FF (256 characters)
- Reference: Unicode Standard Section 10.3
- Dot pattern mapping defined in Story 2.2

**Terminal Compatibility:**
- Tested terminals: PowerShell, bash, zsh, Windows Terminal, Alacritty, iTerm2
- Assumption: Unicode support in target terminals
- Fallback: Detect and warn if Unicode braille unavailable

## Acceptance Criteria (Authoritative)

These are the complete, atomic acceptance criteria extracted from all Epic 2 stories. Each criterion is testable and traceable to PRD functional requirements.

### Story 2.1: Extract BrailleGrid Core from Crabmusic

**AC 2.1.1**: `src/grid.rs` contains `BrailleGrid` struct with fields: `width: usize`, `height: usize`, `dots: Vec<Vec<[bool; 8]>>`, `colors: Option<Vec<Vec<Color>>>`

**AC 2.1.2**: `BrailleGrid::new(width, height) -> Result<Self, DotmaxError>` creates grid with specified dimensions, validates width/height > 0

**AC 2.1.3**: `BrailleGrid::set_dot(x, y, dot_index, value) -> Result<(), DotmaxError>` sets individual dot (0-7 index), validates bounds and dot index

**AC 2.1.4**: `BrailleGrid::get_dot(x, y, dot_index) -> Result<bool, DotmaxError>` reads dot value, validates bounds and dot index

**AC 2.1.5**: `BrailleGrid::clear()` resets all dots to false without reallocation

**AC 2.1.6**: `BrailleGrid::clear_region(x, y, width, height) -> Result<(), DotmaxError>` clears rectangular area, validates bounds

**AC 2.1.7**: `BrailleGrid::dimensions() -> (usize, usize)` returns grid size

**AC 2.1.8**: Dot indexing follows Unicode braille standard (0-7 mapping to braille positions)

**AC 2.1.9**: All methods return `Result<T, DotmaxError>` - zero panics in public API

**AC 2.1.10**: Unit tests cover: grid creation, dot setting/getting for all 8 positions, clear operations, edge cases (zero dimensions, out-of-bounds, invalid dot index)

**AC 2.1.11**: Code is free of crabmusic audio dependencies

### Story 2.2: Implement Unicode Braille Character Conversion

**AC 2.2.1**: `src/grid.rs` contains function to convert 8-dot array to Unicode braille using bitfield formula: `U+2800 + (dots[0]<<0 | dots[1]<<1 | ... | dots[7]<<7)`

**AC 2.2.2**: `BrailleGrid::to_unicode_grid() -> Vec<Vec<char>>` converts entire grid to 2D char array

**AC 2.2.3**: `BrailleGrid::cell_to_braille_char(x, y) -> Result<char, DotmaxError>` converts single cell to Unicode

**AC 2.2.4**: Conversion is correct for all 256 braille patterns (2^8 combinations)

**AC 2.2.5**: Unit tests verify: empty cell → U+2800, full cell → U+28FF, specific patterns match Unicode standard

**AC 2.2.6**: Benchmark (`benches/rendering.rs`) shows conversion <1μs per cell (criterion test passes)

### Story 2.3: Implement GridBuffer and Terminal Rendering Abstraction

**AC 2.3.1**: `src/render.rs` contains `TerminalRenderer` struct with methods: `new()`, `render(grid)`, `clear()`, `get_terminal_size()`

**AC 2.3.2**: `TerminalBackend` trait defined with methods: `size()`, `render(content)`, `clear()`, `capabilities()`

**AC 2.3.3**: Default implementation uses ratatui + crossterm for terminal I/O

**AC 2.3.4**: Rendering pipeline: BrailleGrid → `to_unicode_grid()` → ratatui Frame → crossterm → terminal output

**AC 2.3.5**: Terminal I/O errors wrapped in `DotmaxError::Terminal`

**AC 2.3.6**: `TerminalCapabilities` struct includes: `supports_color`, `supports_truecolor`, `supports_unicode` flags

**AC 2.3.7**: Integration test renders 10×10 grid successfully to terminal

### Story 2.4: Implement Comprehensive Error Handling System

**AC 2.4.1**: `src/error.rs` contains `DotmaxError` enum with variants: `InvalidDimensions`, `OutOfBounds`, `InvalidDotIndex`, `Terminal`, `TerminalBackend`, `UnicodeConversion`

**AC 2.4.2**: All error variants use `#[error("...")]` attribute with meaningful, actionable messages

**AC 2.4.3**: Errors include context: coordinates, dimensions, indices, actual values

**AC 2.4.4**: I/O errors wrapped via `#[from]` for source preservation

**AC 2.4.5**: All public API methods return `Result<T, DotmaxError>` - zero panics contract enforced

**AC 2.4.6**: Unit tests verify error cases: zero dimensions, out-of-bounds access, invalid dot indices

### Story 2.5: Add Terminal Resize Event Handling

**AC 2.5.1**: `TerminalRenderer::get_terminal_size() -> Result<(u16, u16), DotmaxError>` queries current terminal dimensions via crossterm

**AC 2.5.2**: `BrailleGrid::resize(new_width, new_height) -> Result<(), DotmaxError>` adjusts grid dimensions

**AC 2.5.3**: Resize preserves existing dots when grid grows (new cells initialized to false)

**AC 2.5.4**: Resize truncates excess dots when grid shrinks (no data corruption)

**AC 2.5.5**: Color buffer (if enabled) resizes in sync with dots

**AC 2.5.6**: Unit tests verify resize behavior: grow, shrink, preserve data, maintain invariants

### Story 2.6: Implement Color Support for Braille Cells

**AC 2.6.1**: `src/grid.rs` contains `Color` struct with fields: `r: u8`, `g: u8`, `b: u8`

**AC 2.6.2**: `Color::rgb(r, g, b)`, `Color::black()`, `Color::white()` constructors provided

**AC 2.6.3**: `BrailleGrid::enable_color_support()` allocates color buffer (`Option<Vec<Vec<Color>>>` → `Some`)

**AC 2.6.4**: `BrailleGrid::set_cell_color(x, y, color) -> Result<(), DotmaxError>` assigns RGB to cell (validates bounds)

**AC 2.6.5**: `BrailleGrid::get_cell_color(x, y) -> Option<Color>` reads cell color (returns None if no color set or color support disabled)

**AC 2.6.6**: `TerminalRenderer::render()` applies ANSI color codes when rendering colored cells

**AC 2.6.7**: Unit tests verify: color assignment, retrieval, rendering with colors, monochrome fallback

### Story 2.7: Add Debug Logging and Tracing Support

**AC 2.7.1**: `tracing` dependency used for structured logging (already in Cargo.toml from Story 1.3)

**AC 2.7.2**: Key functions instrumented with `#[instrument]` attribute: `BrailleGrid::new()`, `TerminalRenderer::render()`

**AC 2.7.3**: Log levels used appropriately: `error!` (failures), `warn!` (degraded operation), `info!` (major ops), `debug!` (detailed flow)

**AC 2.7.4**: Hot paths (`set_dot`, `get_dot`) do NOT log at debug level (only trace if needed)

**AC 2.7.5**: Library does NOT initialize tracing subscriber (user controls logging)

**AC 2.7.6**: Tests can enable logging via `tracing-subscriber` for debugging test failures

### Epic-Level Acceptance Criteria

**AC EPIC.1**: All 7 stories (2.1-2.7) completed with acceptance criteria met

**AC EPIC.2**: Core modules exist: `src/error.rs`, `src/grid.rs`, `src/render.rs`, `src/lib.rs`

**AC EPIC.3**: Public API exports: `BrailleGrid`, `Color`, `TerminalRenderer`, `TerminalBackend`, `DotmaxError`

**AC EPIC.4**: Zero panics - all public methods return `Result` or panic-free primitives

**AC EPIC.5**: Cross-platform CI passes on Windows, Linux, macOS

**AC EPIC.6**: Code free of crabmusic audio dependencies (~1,150 lines extracted cleanly)

**AC EPIC.7**: Unit test coverage >80% for core rendering logic

**AC EPIC.8**: Integration test renders braille to terminal successfully

## Traceability Mapping

This table maps Epic 2 acceptance criteria to PRD functional requirements, spec sections, implementation components, and test strategies.

| AC ID | PRD FR(s) | Spec Section | Component(s) | Test Approach |
|-------|-----------|--------------|--------------|---------------|
| **Story 2.1: BrailleGrid Core** |
| AC 2.1.1 | FR1, FR2 | Data Models → BrailleGrid | `src/grid.rs` struct definition | Unit: struct fields match spec |
| AC 2.1.2 | FR1 | APIs → `new()` | `src/grid.rs::BrailleGrid::new()` | Unit: various dimensions, validate errors |
| AC 2.1.3 | FR2 | APIs → `set_dot()` | `src/grid.rs::BrailleGrid::set_dot()` | Unit: all 8 dot positions, bounds checks |
| AC 2.1.4 | FR6 | APIs → `get_dot()` | `src/grid.rs::BrailleGrid::get_dot()` | Unit: read after set, bounds checks |
| AC 2.1.5 | FR3 | APIs → `clear()` | `src/grid.rs::BrailleGrid::clear()` | Unit: all dots false, no realloc |
| AC 2.1.6 | FR3 | APIs → `clear_region()` | `src/grid.rs::BrailleGrid::clear_region()` | Unit: region boundaries, partial clear |
| AC 2.1.7 | FR6 | APIs → `dimensions()` | `src/grid.rs::BrailleGrid::dimensions()` | Unit: returns correct size |
| AC 2.1.8 | FR5 | Data Models → Dot indexing | `src/grid.rs` internal logic | Unit: verify 0-7 mapping to braille positions |
| AC 2.1.9 | FR56, FR57 | NFR-S3 Zero Panics | All public methods | Unit: edge cases return Err, never panic |
| AC 2.1.10 | FR81 | Test Strategy | `src/grid.rs` tests module | Unit: comprehensive test suite |
| AC 2.1.11 | Architecture | Extraction Strategy | Entire `src/grid.rs` | Code review: no audio imports |
| **Story 2.2: Unicode Conversion** |
| AC 2.2.1 | FR5 | Workflows → Unicode Algorithm | `src/grid.rs` conversion fn | Unit: bitfield calculation correct |
| AC 2.2.2 | FR5 | APIs → `to_unicode_grid()` | `src/grid.rs::BrailleGrid::to_unicode_grid()` | Unit: grid → 2D char array |
| AC 2.2.3 | FR5 | APIs → `cell_to_braille_char()` | `src/grid.rs::BrailleGrid::cell_to_braille_char()` | Unit: single cell conversion |
| AC 2.2.4 | FR5 | Workflows → Unicode Algorithm | Unicode conversion logic | Unit: all 256 patterns (exhaustive) |
| AC 2.2.5 | FR5 | Workflows → Unicode Algorithm | Unicode conversion logic | Unit: specific patterns (U+2800, U+28FF) |
| AC 2.2.6 | FR68, NFR-P1 | Performance → Unicode speed | Unicode conversion | Benchmark: <1μs per cell (criterion) |
| **Story 2.3: Terminal Rendering** |
| AC 2.3.1 | FR4, FR51 | APIs → TerminalRenderer | `src/render.rs::TerminalRenderer` | Unit: methods exist and callable |
| AC 2.3.2 | FR52, ADR-0004 | Data Models → TerminalBackend | `src/render.rs::TerminalBackend` trait | Unit: trait methods defined |
| AC 2.3.3 | FR51 | Dependencies → ratatui/crossterm | `src/render.rs` DefaultTerminal | Integration: renders to actual terminal |
| AC 2.3.4 | FR4 | Workflows → Rendering Pipeline | `src/render.rs::TerminalRenderer::render()` | Integration: end-to-end render |
| AC 2.3.5 | FR56 | NFR-R2 Error Handling | Error wrapping in render.rs | Unit: I/O errors → DotmaxError::Terminal |
| AC 2.3.6 | FR53, FR37 | Data Models → TerminalCapabilities | `src/render.rs::TerminalCapabilities` | Unit: capability detection |
| AC 2.3.7 | FR82 | Test Strategy | Integration test | Integration: 10×10 grid renders |
| **Story 2.4: Error Handling** |
| AC 2.4.1 | FR56 | Data Models → DotmaxError | `src/error.rs::DotmaxError` | Unit: all variants defined |
| AC 2.4.2 | FR58 | NFR-O2 Error Context | Error messages | Manual: read error messages |
| AC 2.4.3 | FR58 | NFR-R2 Actionable Errors | Error variant fields | Unit: errors include context |
| AC 2.4.4 | FR56 | Data Models → DotmaxError | `#[from]` on I/O errors | Unit: source chain preserved |
| AC 2.4.5 | FR57, NFR-S3 | Zero Panics Policy | All public API | Unit: exhaustive error cases |
| AC 2.4.6 | FR81 | Test Strategy | Error handling tests | Unit: InvalidDimensions, OutOfBounds, etc. |
| **Story 2.5: Resize Handling** |
| AC 2.5.1 | FR7, FR53 | APIs → `get_terminal_size()` | `src/render.rs::TerminalRenderer::get_terminal_size()` | Integration: query actual terminal |
| AC 2.5.2 | FR7 | APIs → `resize()` | `src/grid.rs::BrailleGrid::resize()` | Unit: dimension changes |
| AC 2.5.3 | FR7 | Workflows → Resize Flow | Resize implementation | Unit: preserve dots on grow |
| AC 2.5.4 | FR7 | Workflows → Resize Flow | Resize implementation | Unit: truncate on shrink |
| AC 2.5.5 | FR7, FR8 | Workflows → Resize Flow | Color buffer resize | Unit: colors sync with dots |
| AC 2.5.6 | FR81 | Test Strategy | Resize tests | Unit: grow/shrink/preserve |
| **Story 2.6: Color Support** |
| AC 2.6.1 | FR8, FR32 | Data Models → Color | `src/grid.rs::Color` | Unit: struct definition |
| AC 2.6.2 | FR32 | APIs → Color constructors | `src/grid.rs::Color` methods | Unit: rgb(), black(), white() |
| AC 2.6.3 | FR8 | APIs → `enable_color_support()` | `src/grid.rs::BrailleGrid::enable_color_support()` | Unit: allocates color buffer |
| AC 2.6.4 | FR32 | APIs → `set_cell_color()` | `src/grid.rs::BrailleGrid::set_cell_color()` | Unit: assign color, validate bounds |
| AC 2.6.5 | FR32 | APIs → `get_cell_color()` | `src/grid.rs::BrailleGrid::get_cell_color()` | Unit: retrieve color or None |
| AC 2.6.6 | FR17, FR33 | Workflows → Color Rendering | `src/render.rs` color output | Integration: colored terminal output |
| AC 2.6.7 | FR81 | Test Strategy | Color tests | Unit: assignment, retrieval, rendering |
| **Story 2.7: Debug Logging** |
| AC 2.7.1 | FR60 | Dependencies → tracing | Cargo.toml | Verify: dependency exists |
| AC 2.7.2 | FR60, NFR-O1 | Observability → Instrumentation | `#[instrument]` on key fns | Manual: check instrumentation |
| AC 2.7.3 | FR60, NFR-O1 | Observability → Log Levels | Log calls in code | Code review: appropriate levels |
| AC 2.7.4 | NFR-O3 | Observability → Hot Paths | set_dot/get_dot logging | Code review: no debug logs in hot paths |
| AC 2.7.5 | NFR-O4 | Observability → User Control | Library logging design | Manual: no subscriber init in lib |
| AC 2.7.6 | FR60 | Test Strategy | Test logging | Manual: enable logging in tests |
| **Epic-Level Criteria** |
| AC EPIC.1 | All FRs | Overview | All Epic 2 stories | Story completion tracking |
| AC EPIC.2 | Architecture | Module Structure | src/ directory | Manual: files exist |
| AC EPIC.3 | Architecture | Public API Surface | `src/lib.rs` | Manual: exports correct |
| AC EPIC.4 | FR57, NFR-S3 | Zero Panics Policy | All public API | Code review + unit tests |
| AC EPIC.5 | FR75-77, NFR-R1 | Cross-Platform | CI pipeline | CI: Windows/Linux/macOS pass |
| AC EPIC.6 | Architecture | Extraction Strategy | All source files | Code review: no audio deps |
| AC EPIC.7 | NFR-M4 | Test Coverage | Test suite | Coverage report: >80% |
| AC EPIC.8 | FR82 | Integration Testing | Integration tests | Integration: renders successfully |

## Risks, Assumptions, Open Questions

### Risks

**RISK-2.1: Crabmusic Code Quality Unknown**
- **Description**: Extracting ~1,150 lines from crabmusic - code quality may require significant refactoring
- **Impact**: Medium - Could extend Story 2.1 timeline if code is messy
- **Mitigation**:
  - Review crabmusic code before extraction (assess quality early)
  - Budget extra time for refactoring in Story 2.1
  - Write tests immediately after extraction to lock behavior
  - Refactor incrementally with test coverage
- **Status**: Open - Will assess during Story 2.1 kickoff

**RISK-2.2: Performance Below Targets**
- **Description**: Unicode conversion might not meet <1μs per cell target on all platforms
- **Impact**: Low - Epic 2 defers optimization to Epic 7, but benchmark establishes baseline
- **Mitigation**:
  - Story 2.2 benchmark establishes actual performance
  - If target missed, document actual timing and defer optimization to Epic 7
  - Direct bitfield calculation is already fast - unlikely to miss target
- **Status**: Open - Will measure in Story 2.2

**RISK-2.3: Terminal Compatibility Issues**
- **Description**: Some terminals may not support Unicode braille (U+2800-U+28FF) or colors
- **Impact**: Medium - Library may not work on older/limited terminals
- **Mitigation**:
  - Story 2.3 implements TerminalCapabilities detection
  - Graceful error messages if Unicode unsupported (don't crash)
  - Document tested terminals in README (PowerShell, Windows Terminal, Alacritty, etc.)
  - Future: ASCII fallback mode (not in Epic 2)
- **Status**: Open - Will test on multiple terminals during Story 2.3

**RISK-2.4: Cross-Platform CI Flakiness**
- **Description**: Integration tests rendering to terminal may be flaky in CI environments
- **Impact**: Medium - Could block merges if tests fail intermittently
- **Mitigation**:
  - Mock TerminalBackend in most tests (don't rely on actual terminal)
  - Limit actual terminal rendering tests to manual verification
  - Use headless terminal emulation in CI if needed
  - Document known CI limitations
- **Status**: Open - Will address if CI proves flaky

### Assumptions

**ASSUMPTION-2.1: Terminal Unicode Support**
- **Assumption**: Target terminals support Unicode braille range (U+2800-U+28FF)
- **Rationale**: 99%+ of modern terminals support Unicode
- **Validation**: Story 2.3 implements detection, warns if unsupported
- **Impact if False**: Library won't work on very old terminals (acceptable trade-off)

**ASSUMPTION-2.2: Crabmusic License Compatible**
- **Assumption**: Crabmusic license allows extraction and reuse in dotmax (MIT/Apache-2.0)
- **Rationale**: Frosty owns crabmusic code
- **Validation**: Check crabmusic LICENSE file before extraction
- **Impact if False**: BLOCKER - Cannot proceed with extraction without compatible license

**ASSUMPTION-2.3: Rust 1.70 Sufficient**
- **Assumption**: MSRV 1.70 supports all features needed (no newer Rust features required)
- **Rationale**: Dependencies compatible with 1.70, no advanced features needed
- **Validation**: CI enforces MSRV testing
- **Impact if False**: Bump MSRV (minor issue, document in README)

**ASSUMPTION-2.4: Vec<Vec<[bool; 8]>> Adequate**
- **Assumption**: Simple `Vec<Vec<[bool; 8]>>` storage is adequate for MVP (not optimized)
- **Rationale**: Epic 2 prioritizes correctness; Epic 7 handles optimization
- **Validation**: Story 2.2 benchmark measures actual performance
- **Impact if False**: Refactor to packed `Vec<u8>` in Epic 7 (planned optimization path)

**ASSUMPTION-2.5: ratatui/crossterm Stable**
- **Assumption**: ratatui 0.29 and crossterm 0.29 APIs are stable (no breaking changes expected)
- **Rationale**: Both are mature, widely-used crates with stable APIs
- **Validation**: Monitor dependency updates, pin versions in Cargo.toml
- **Impact if False**: Upstream breaking change requires migration (TerminalBackend abstraction mitigates)

### Open Questions

**QUESTION-2.1: Crabmusic Extraction Specifics**
- **Question**: Which exact files/functions should be extracted from crabmusic?
- **Context**: Need to map crabmusic code to dotmax modules precisely
- **Resolution**: Story 2.1 developer will review crabmusic repo and document extraction map
- **Blocking**: Story 2.1 start
- **Owner**: Dev Agent

**QUESTION-2.2: Test Coverage Target**
- **Question**: Is >80% line coverage sufficient, or should we target higher?
- **Context**: AC EPIC.7 specifies >80%, but critical code might need 100%
- **Resolution**: Start with 80%, push critical paths (BrailleGrid core, Unicode conversion) to 100%
- **Blocking**: No - defer to Story 2.1 test writing
- **Owner**: TEA Agent (test strategy)

**QUESTION-2.3: Color Conversion Algorithm**
- **Question**: Should RGB → ANSI conversion be in Epic 2 or Epic 5?
- **Context**: Story 2.6 adds basic color support, but conversion algorithm might be complex
- **Resolution**: Epic 2 stores raw RGB only; Epic 5 implements conversion algorithms (cleaner separation)
- **Blocking**: No - clarifies scope boundary
- **Owner**: SM Agent (epic scoping)

**QUESTION-2.4: Benchmark CI Integration**
- **Question**: Should criterion benchmarks run in CI or only locally?
- **Context**: CI environments may have inconsistent performance
- **Resolution**: Run benchmarks in CI but don't fail on performance regression (track trends only). Detailed performance work in Epic 7.
- **Blocking**: No - can decide during Story 1.6 (benchmarking infrastructure)
- **Owner**: Architect Agent

## Test Strategy Summary

Epic 2 establishes comprehensive test coverage for the core rendering engine. All code must be tested before merging.

### Test Levels

**Unit Tests (Primary Focus)**

Scope: All public API methods and internal logic
- **Location**: `src/grid.rs`, `src/render.rs`, `src/error.rs` - each file has `#[cfg(test)] mod tests`
- **Framework**: Built-in Rust `#[test]` with `assert!`, `assert_eq!`
- **Coverage Target**: >80% line coverage (100% for critical paths)
- **Run Command**: `cargo test`

**Critical Unit Test Cases:**

| Module | Test Cases | Purpose |
|--------|------------|---------|
| `grid.rs` | Grid creation (various dimensions, zero dimensions, large dimensions) | Validate construction |
| `grid.rs` | Dot manipulation (all 8 positions, out-of-bounds, invalid indices) | Validate core operations |
| `grid.rs` | Clear operations (full grid, regions, edge cases) | Validate state management |
| `grid.rs` | Unicode conversion (all 256 patterns, specific chars U+2800/U+28FF) | Validate braille mapping |
| `grid.rs` | Resize (grow, shrink, preserve data, color buffer sync) | Validate dynamic sizing |
| `grid.rs` | Color support (enable, set, get, None cases) | Validate color infrastructure |
| `render.rs` | TerminalRenderer methods (new, render, clear, size) | Validate rendering API |
| `error.rs` | Error variants (all types, context fields, messages) | Validate error handling |

**Integration Tests**

Scope: End-to-end workflows across module boundaries
- **Location**: `tests/integration_tests.rs`
- **Framework**: Rust integration test (`tests/` directory)
- **Run Command**: `cargo test --test integration_tests`

**Integration Test Cases:**

1. **Basic Rendering Flow**
   - Create BrailleGrid → set dots → render to terminal → verify output
   - Validates: grid.rs + render.rs integration

2. **Color Rendering Flow**
   - Create grid → enable colors → set dots + colors → render → verify colored output
   - Validates: color support + rendering

3. **Resize Flow**
   - Create grid → set dots → resize → verify dots preserved → render
   - Validates: resize + rendering

4. **Error Propagation**
   - Trigger errors in different modules → verify proper error wrapping
   - Validates: DotmaxError handling across modules

**Benchmark Tests (Performance Validation)**

Scope: Performance-critical operations
- **Location**: `benches/rendering.rs`
- **Framework**: Criterion.rs (statistical benchmarking)
- **Run Command**: `cargo bench`

**Benchmark Cases:**

1. **Unicode Conversion** (Story 2.2 AC 2.2.6)
   - Measure: Convert single cell (8 dots → char)
   - Target: <1μs per cell
   - Validates: NFR-P1

2. **Grid Operations**
   - Measure: `set_dot()` throughput
   - Target: O(1) constant time
   - Validates: NFR-P2

3. **Full Grid Rendering**
   - Measure: 80×24 grid → Unicode → render
   - Baseline: Establish actual timing for Epic 7 optimization
   - Validates: End-to-end performance

**Mock Testing Strategy**

Use mocks for terminal I/O to avoid CI flakiness:

```rust
// Mock TerminalBackend for testing
struct MockTerminal {
    rendered_content: Vec<String>,
    size: (u16, u16),
}

impl TerminalBackend for MockTerminal {
    fn render(&mut self, content: &str) -> Result<(), DotmaxError> {
        self.rendered_content.push(content.to_string());
        Ok(())
    }
    // ... other trait methods
}
```

**Benefits:**
- No actual terminal required (works in CI)
- Deterministic output (no flakiness)
- Can verify exact rendered content

### Test Execution

**Local Development:**
```bash
# Run all tests
cargo test

# Run with coverage (requires tarpaulin)
cargo tarpaulin --out Html

# Run benchmarks
cargo bench

# Run specific test
cargo test test_grid_creation
```

**CI Pipeline:**
```yaml
# .github/workflows/ci.yml (from Epic 1, Story 1.2)
- name: Run tests
  run: cargo test --all-features

- name: Run benchmarks (non-blocking)
  run: cargo bench --no-fail-fast
  continue-on-error: true  # Don't fail CI on benchmark issues
```

### Coverage Requirements

**Overall Target**: >80% line coverage for Epic 2 code

**Critical Path Target**: 100% coverage
- `BrailleGrid::new()`, `set_dot()`, `get_dot()` - core operations
- Unicode conversion logic - correctness critical
- Error handling paths - must validate all error cases

**Exclusions from Coverage**:
- `#[cfg(test)]` test modules
- Example code (`examples/`)
- Unreachable code (properly documented with `unreachable!()`)

### Test Data Strategy

**Fixtures:**
- Small grids: 1×1, 5×5 (edge cases, fast)
- Standard grid: 80×24 (typical terminal)
- Large grid: 200×50 (stress test)

**Dot Patterns:**
- All zeros (U+2800 blank)
- All ones (U+28FF full)
- Specific patterns from epics.md Story 2.2
- Randomized patterns (property-based if time permits)

**Colors:**
- Black (0,0,0)
- White (255,255,255)
- Primary colors (R, G, B)
- Random RGB values

### Regression Prevention

**Practices:**
1. **Lock Behavior with Tests**: After extracting from crabmusic, write tests immediately to lock proven behavior
2. **Test-Driven Refactoring**: When refactoring extracted code, ensure tests pass before and after
3. **CI Enforcement**: All tests must pass before merge (no exceptions)
4. **Benchmark Tracking**: Track performance over time (don't regress)

### Test Documentation

Each test includes:
- **Descriptive name**: `test_grid_creation_validates_dimensions`
- **Comment**: Explains what's being tested and why
- **Arrange-Act-Assert** structure: Clear test phases

Example:
```rust
#[test]
fn test_set_dot_validates_bounds() {
    // Arrange: Create 10×10 grid
    let mut grid = BrailleGrid::new(10, 10).unwrap();

    // Act: Try to set dot out of bounds
    let result = grid.set_dot(100, 100, 0, true);

    // Assert: Error with correct context
    assert!(matches!(result, Err(DotmaxError::OutOfBounds { .. })));
}
```

### Definition of Done (Testing)

A story is **not done** until:
1. ✅ All acceptance criteria have corresponding tests
2. ✅ Unit tests pass locally (`cargo test`)
3. ✅ Integration tests pass locally
4. ✅ CI tests pass (all platforms)
5. ✅ Coverage meets target (>80% overall, 100% critical paths)
6. ✅ Benchmarks run successfully (even if performance not optimized yet)
7. ✅ No compiler warnings (`cargo clippy` clean)
8. ✅ Code formatted (`cargo fmt` clean)
