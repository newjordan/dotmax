# Story 2.4: Implement Comprehensive Error Handling System

Status: ready-for-dev

## Story

As a **developer using dotmax in applications**,
I want comprehensive, meaningful error types for all operations,
so that I can handle failures gracefully and provide actionable feedback to users.

## Acceptance Criteria

1. `src/error.rs` contains `DotmaxError` enum with variants: `InvalidDimensions`, `OutOfBounds`, `InvalidDotIndex`, `Terminal`, `TerminalBackend`, `UnicodeConversion`
2. All error variants use `#[error("...")]` attribute with meaningful, actionable messages
3. Errors include context: coordinates, dimensions, indices, actual values
4. I/O errors wrapped via `#[from]` for source preservation
5. All public API methods return `Result<T, DotmaxError>` - zero panics contract enforced
6. Unit tests verify error cases: zero dimensions, out-of-bounds access, invalid dot indices

## Tasks / Subtasks

- [ ] Task 1: Create comprehensive DotmaxError enum (AC: #1, #2, #3)
  - [ ] Create `src/error.rs` with full error enum
  - [ ] Add variant: `InvalidDimensions { width: usize, height: usize }`
  - [ ] Add variant: `OutOfBounds { x: usize, y: usize, width: usize, height: usize }`
  - [ ] Add variant: `InvalidDotIndex { index: u8 }`
  - [ ] Add variant: `Terminal(#[from] std::io::Error)`
  - [ ] Add variant: `TerminalBackend(String)`
  - [ ] Add variant: `UnicodeConversion { x: usize, y: usize }`
  - [ ] Each variant has descriptive `#[error("...")]` message
  - [ ] Messages include all context fields for debugging

- [ ] Task 2: Migrate existing error handling to use DotmaxError (AC: #4, #5)
  - [ ] Review `src/grid.rs` - ensure all methods return `Result<T, DotmaxError>`
  - [ ] Replace any remaining basic error types with DotmaxError variants
  - [ ] Update `src/render.rs` error handling (should already use DotmaxError::Terminal)
  - [ ] Verify `#[from]` attribute properly chains I/O errors
  - [ ] Remove any `.unwrap()`, `.expect()`, or `panic!()` from public API paths
  - [ ] Ensure all bounds checking returns `OutOfBounds` error

- [ ] Task 3: Add input validation with proper errors (AC: #5)
  - [ ] `BrailleGrid::new()`: Validate width, height > 0 → `InvalidDimensions`
  - [ ] `BrailleGrid::new()`: Validate max dimensions (10,000) → `InvalidDimensions`
  - [ ] `set_dot()`, `get_dot()`: Validate x < width, y < height → `OutOfBounds`
  - [ ] `set_dot()`, `get_dot()`: Validate dot_index 0-7 → `InvalidDotIndex`
  - [ ] `set_cell_color()`, `get_cell_color()`: Validate bounds → `OutOfBounds`
  - [ ] `clear_region()`: Validate region bounds → `OutOfBounds`

- [ ] Task 4: Write comprehensive error handling tests (AC: #6)
  - [ ] Test: `BrailleGrid::new(0, 10)` → `Err(InvalidDimensions)`
  - [ ] Test: `BrailleGrid::new(10, 0)` → `Err(InvalidDimensions)`
  - [ ] Test: `BrailleGrid::new(20000, 20000)` → `Err(InvalidDimensions)` (exceeds max)
  - [ ] Test: `set_dot(100, 50, 0, true)` on 10×10 grid → `Err(OutOfBounds)`
  - [ ] Test: `get_dot(100, 50, 0)` on 10×10 grid → `Err(OutOfBounds)`
  - [ ] Test: `set_dot(5, 5, 10, true)` → `Err(InvalidDotIndex)` (10 > 7)
  - [ ] Test: `get_dot(5, 5, 255, true)` → `Err(InvalidDotIndex)`
  - [ ] Test: Verify error messages include correct coordinates and dimensions
  - [ ] Test: Verify I/O error chaining preserves source

- [ ] Task 5: Update public API and documentation (AC: #5)
  - [ ] Export `DotmaxError` in `src/lib.rs`
  - [ ] Export `Result<T>` type alias: `pub type Result<T> = std::result::Result<T, DotmaxError>;`
  - [ ] Add rustdoc examples showing error handling patterns
  - [ ] Document zero-panics contract in module-level docs
  - [ ] Update method signatures to use dotmax::Result where appropriate

- [ ] Task 6: Run quality checks and verify zero-panics policy (AC: #5)
  - [ ] `cargo test` - all tests pass (existing + new error tests)
  - [ ] `cargo clippy -- -D warnings` - zero warnings
  - [ ] `cargo fmt --check` - formatted correctly
  - [ ] Code review: Search for `.unwrap()`, `.expect()`, `panic!()` in public API
  - [ ] Verify all public methods return Result or panic-free primitives
  - [ ] CI passes on Windows, Linux, macOS

## Dev Notes

### Learnings from Previous Story (Story 2.3)

**From Story 2-3-implement-gridbuffer-and-terminal-rendering-abstraction (Status: review)**

- **Error Handling Pattern Established**:
  - DotmaxError enum partially exists in src/grid.rs (lines 40-44)
  - Terminal variant uses `#[from] std::io::Error` for automatic error conversion
  - TerminalBackend variant for custom backend errors
  - Story 2.3 uses `?` operator for clean error propagation
  - Zero panics policy already enforced in src/render.rs

- **Existing Error Variants** (from src/grid.rs):
  ```rust
  #[derive(Error, Debug)]
  pub enum DotmaxError {
      #[error("Terminal error: {0}")]
      Terminal(#[from] std::io::Error),
      #[error("Terminal backend error: {0}")]
      TerminalBackend(String),
  }
  ```

- **Story 2.4 Task**: Expand this enum to include all error types from Tech Spec
  - Need to add: InvalidDimensions, OutOfBounds, InvalidDotIndex, UnicodeConversion
  - Move DotmaxError from src/grid.rs to src/error.rs (proper module organization)
  - Update all imports in grid.rs and render.rs to use error module

- **Code Review Insights from Story 2.3**:
  - Using `#[from]` attribute is preferred over manual From impl (avoids conflicts)
  - All 10 integration tests properly use #[ignore = "reason"] syntax
  - Panic handler installed in src/render.rs (lines 185-189) for terminal cleanup
  - Error handling with `?` operator is clean and idiomatic

- **Technical Debt**: None - Story 2.3 error handling was clean

- **Integration Points**:
  - Story 2.4 creates src/error.rs module
  - All existing error handling in grid.rs and render.rs will import from error module
  - Public API (src/lib.rs) will export DotmaxError and Result<T> type alias

[Source: stories/2-3-implement-gridbuffer-and-terminal-rendering-abstraction.md#Dev-Agent-Record]

### Error Handling Design (from Tech Spec)

**DotmaxError Complete Specification** (from tech-spec-epic-2.md):

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

**Error Handling Contract** (from Tech Spec NFR-S3):

- **Zero panics** - All public functions return `Result<T, DotmaxError>`
- **Meaningful context** - Errors include coordinates, dimensions, actual values
- **Source chaining** - `#[from]` preserves underlying I/O errors
- **User-facing messages** - `#[error("...")]` provides actionable feedback

**Input Validation Requirements** (from Tech Spec NFR-S2):

- **Dimensions**: `BrailleGrid::new()` validates width, height > 0 → Err(InvalidDimensions)
- **Coordinates**: All `set_dot()`, `get_dot()`, `set_cell_color()` validate bounds → Err(OutOfBounds)
- **Dot index**: Validate 0-7 range → Err(InvalidDotIndex)
- **Resource limits**: Max grid dimensions to prevent OOM attacks
  ```rust
  const MAX_GRID_WIDTH: usize = 10_000;
  const MAX_GRID_HEIGHT: usize = 10_000;
  // Validated in BrailleGrid::new()
  ```

### Error Flow Example (from Tech Spec)

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

### Current State Analysis

**What exists** (from Stories 2.1, 2.2, 2.3):

1. **src/grid.rs** (Stories 2.1, 2.2):
   - BrailleGrid struct with basic error handling
   - DotmaxError enum (partial - only Terminal and TerminalBackend variants)
   - Some methods may not yet validate all inputs comprehensively

2. **src/render.rs** (Story 2.3):
   - Uses DotmaxError::Terminal for I/O errors
   - Uses `#[from]` attribute for automatic error conversion
   - Zero panics policy enforced
   - Panic handler for terminal cleanup

**What Story 2.4 adds**:

1. **src/error.rs** (NEW):
   - Complete DotmaxError enum with all 6 variants
   - Comprehensive error messages with context
   - Proper thiserror derive usage

2. **Enhanced validation** (grid.rs):
   - Add bounds checking to all coordinate-based methods
   - Add dimension validation to BrailleGrid::new()
   - Add dot index validation (0-7) to set_dot/get_dot
   - Add max dimension limits (MAX_GRID_WIDTH/HEIGHT)

3. **Test coverage** (grid.rs tests module):
   - Comprehensive error case tests
   - Verify error messages include correct context
   - Verify zero-panics policy

### Module Reorganization Plan

**Before (Story 2.3 state)**:
```
src/
├── lib.rs
├── grid.rs (contains DotmaxError enum)
└── render.rs (imports DotmaxError from grid)
```

**After (Story 2.4)**:
```
src/
├── lib.rs (exports error::DotmaxError)
├── error.rs (NEW - contains complete DotmaxError enum)
├── grid.rs (imports DotmaxError from error module)
└── render.rs (imports DotmaxError from error module)
```

**Migration Steps**:
1. Create src/error.rs with complete DotmaxError enum
2. Remove DotmaxError from src/grid.rs
3. Add `use crate::error::DotmaxError;` to grid.rs
4. Update `use crate::grid::DotmaxError;` to `use crate::error::DotmaxError;` in render.rs
5. Add `pub use error::DotmaxError;` to lib.rs
6. Add `pub type Result<T> = std::result::Result<T, DotmaxError>;` to lib.rs

### Testing Strategy (from Tech Spec)

**Error Handling Tests** (new tests in src/error.rs and src/grid.rs):

```rust
// src/error.rs tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages_include_context() {
        let err = DotmaxError::OutOfBounds {
            x: 100,
            y: 50,
            width: 80,
            height: 24,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("100"));
        assert!(msg.contains("50"));
        assert!(msg.contains("80"));
        assert!(msg.contains("24"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let dotmax_err: DotmaxError = io_err.into();
        assert!(matches!(dotmax_err, DotmaxError::Terminal(_)));
    }
}
```

**Grid Validation Tests** (add to src/grid.rs tests):

```rust
#[test]
fn test_new_validates_zero_width() {
    let result = BrailleGrid::new(0, 10);
    assert!(matches!(result, Err(DotmaxError::InvalidDimensions { .. })));
}

#[test]
fn test_new_validates_zero_height() {
    let result = BrailleGrid::new(10, 0);
    assert!(matches!(result, Err(DotmaxError::InvalidDimensions { .. })));
}

#[test]
fn test_new_validates_max_dimensions() {
    let result = BrailleGrid::new(20000, 20000);
    assert!(matches!(result, Err(DotmaxError::InvalidDimensions { .. })));
}

#[test]
fn test_set_dot_validates_bounds() {
    let mut grid = BrailleGrid::new(10, 10).unwrap();
    let result = grid.set_dot(100, 50, 0, true);
    assert!(matches!(result, Err(DotmaxError::OutOfBounds { .. })));
}

#[test]
fn test_set_dot_validates_dot_index() {
    let mut grid = BrailleGrid::new(10, 10).unwrap();
    let result = grid.set_dot(5, 5, 10, true);
    assert!(matches!(result, Err(DotmaxError::InvalidDotIndex { index: 10 })));
}
```

### Security Considerations (from Tech Spec NFR-S1, NFR-S2, NFR-S3)

**NFR-S1: Memory Safety**:
- No unsafe blocks in error handling code
- All validation prevents buffer overflows
- Rust's type system enforces memory safety

**NFR-S2: Input Validation**:
- Dimensions validated before allocation (prevent OOM)
- Coordinates validated before array access (prevent panics)
- Dot indices validated (prevent invalid Unicode generation)
- Max grid dimensions: 10,000 × 10,000

**NFR-S3: Zero Panic Policy**:
- All public methods return Result<T, DotmaxError>
- No `.unwrap()`, `.expect()`, `panic!()` in public API
- Enforcement:
  - Code review checks
  - Unit tests cover all error paths
  - CI enforces clippy warnings

### Architecture Alignment

**ADR References**:
- **ADR 0002**: Use thiserror for Error Handling - Story 2.4 implements this
- **ADR 0005**: Brownfield Extraction Strategy - Not applicable (error handling is new design)

**Module Structure** (from Tech Spec):
```
src/
├── lib.rs                    # Re-exports BrailleGrid, TerminalRenderer, DotmaxError
├── error.rs                  # Story 2.4 - DotmaxError enum (NEW)
├── grid.rs                   # Story 2.1 & 2.2 - BrailleGrid + Unicode conversion
└── render.rs                 # Story 2.3 - TerminalBackend trait + TerminalRenderer
```

Story 2.4 creates **src/error.rs** (~150 lines estimated).

### References

- **[Source: docs/sprint-artifacts/tech-spec-epic-2.md#Story-2.4]** - Complete AC and detailed design
- **[Source: docs/sprint-artifacts/tech-spec-epic-2.md#DotmaxError]** - Error enum specification
- **[Source: docs/sprint-artifacts/tech-spec-epic-2.md#Error-Handling-Flow]** - Validation and error flow
- **[Source: docs/sprint-artifacts/tech-spec-epic-2.md#NFR-S2-Input-Validation]** - Validation requirements
- **[Source: docs/sprint-artifacts/tech-spec-epic-2.md#NFR-S3-Zero-Panic-Policy]** - Zero panics contract
- **[Source: docs/architecture.md#ADR-0002-Error-Handling]** - Why thiserror over anyhow
- **[Source: docs/PRD.md#FR56]** - Functional requirement for comprehensive error handling
- **[Source: docs/PRD.md#FR57]** - Zero panics requirement
- **[Source: docs/PRD.md#FR58]** - Error messages must be actionable

---

## Definition of Done

Story 2.4 is **complete** when:

1. ✅ `src/error.rs` created with complete `DotmaxError` enum (6 variants)
2. ✅ All error variants have descriptive `#[error("...")]` messages with context
3. ✅ I/O errors use `#[from]` attribute for automatic conversion
4. ✅ All public API methods in grid.rs and render.rs return `Result<T, DotmaxError>`
5. ✅ Input validation added to all coordinate/dimension operations
6. ✅ Unit tests cover all error cases (zero dims, out-of-bounds, invalid indices)
7. ✅ Public API exports: `DotmaxError` and `Result<T>` type alias
8. ✅ `cargo test` passes (existing + new error tests)
9. ✅ `cargo clippy -- -D warnings` passes (zero warnings)
10. ✅ `cargo fmt --check` passes (correctly formatted)
11. ✅ Code review: No `.unwrap()`, `.expect()`, or `panic!()` in public API
12. ✅ CI passes on Windows, Linux, macOS
13. ✅ Story moved to **drafted** status in sprint-status.yaml (auto-updated by workflow)

---

## Dev Agent Record

### Context Reference

- docs/sprint-artifacts/2-4-implement-comprehensive-error-handling-system.context.xml (generated 2025-11-18 by story-context workflow)

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

None - Implementation completed without blockers.

### Completion Notes List

**Implementation Summary (2025-11-18)**

All acceptance criteria met and verified:

✅ **AC #1**: Created `src/error.rs` with complete `DotmaxError` enum (6 variants: InvalidDimensions, OutOfBounds, InvalidDotIndex, Terminal, TerminalBackend, UnicodeConversion)

✅ **AC #2**: All error variants use `#[error("...")]` attribute with meaningful, actionable messages

✅ **AC #3**: All errors include context fields for debugging (coordinates, dimensions, indices)

✅ **AC #4**: I/O errors wrapped via `#[from]` for source preservation in Terminal variant

✅ **AC #5**: All public API methods return `Result<T, DotmaxError>` - zero panics contract enforced
- Verified: No `.unwrap()`, `.expect()`, or `panic!()` in public API paths
- Only safe `unwrap_or` used in `dots_to_char` (line 118)

✅ **AC #6**: Comprehensive unit tests added (55 total passing tests, up from 43)
- Error dimension validation tests (zero width, zero height, exceeds max)
- Error bounds checking tests (out-of-bounds access)
- Error message context verification tests (all context fields included)
- Error conversion tests (std::io::Error → DotmaxError::Terminal)

**Quality Verification:**
- ✅ `cargo test`: 55 passed, 0 failed (11 doctests, 8 integration tests)
- ✅ `cargo clippy -- -D warnings`: 0 warnings
- ✅ `cargo fmt --check`: Correctly formatted
- ✅ Zero panics policy verified: All public API methods safe

**Module Reorganization Completed:**
- Moved `DotmaxError` from `src/grid.rs` to new `src/error.rs` module
- Updated imports in `src/grid.rs` and `src/render.rs`
- Added public exports in `src/lib.rs`: `DotmaxError` and `Result<T>` type alias
- Backward compatible for external users (all imports from crate root)

**Tests Added:**
- `src/error.rs`: 7 error-specific tests (message context, I/O conversion, source preservation)
- `src/grid.rs`: 5 additional error context verification tests
- Total: 12 new tests focused on error handling validation

### File List

**Created:**
- `src/error.rs` (~180 lines) - Complete DotmaxError enum with all 6 variants and 7 unit tests

**Modified:**
- `src/lib.rs` - Added error module, exported DotmaxError and Result<T> type alias
- `src/grid.rs` - Removed DotmaxError enum (moved to error.rs), updated imports, added 5 error context verification tests
- `src/render.rs` - Updated imports to use `crate::error::DotmaxError`
