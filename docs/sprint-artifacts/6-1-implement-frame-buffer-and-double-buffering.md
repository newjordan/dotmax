# Story 6.1: Implement Frame Buffer and Double Buffering

Status: ready-for-dev

## Story

As a **developer creating flicker-free animations**,
I want **double buffering for smooth frame transitions**,
so that **animations don't flicker or tear during updates**.

## Acceptance Criteria

1. **AC1: FrameBuffer::new() Creates Two BrailleGrid Buffers**
   - `FrameBuffer::new(width: usize, height: usize) -> Self` in `src/animation/frame_buffer.rs`
   - Creates two `BrailleGrid` instances: front buffer (displayed) and back buffer (being prepared)
   - Both buffers initialized with same dimensions
   - Returns `FrameBuffer` struct with owned buffers

2. **AC2: get_back_buffer() Returns Mutable Reference**
   - `pub fn get_back_buffer(&mut self) -> &mut BrailleGrid`
   - Returns mutable reference to back buffer for drawing operations
   - Front buffer remains unchanged while drawing to back buffer
   - User can draw to back buffer using all BrailleGrid methods (set_dot, draw_line, etc.)

3. **AC3: swap_buffers() Exchanges Front/Back in <1ms**
   - `pub fn swap_buffers(&mut self)` performs instant swap
   - Implements pointer swap (not data copy) for O(1) performance
   - After swap: previous back becomes front, previous front becomes back
   - Benchmark validates <1ms swap time

4. **AC4: render() Outputs Front Buffer via TerminalRenderer**
   - `pub fn render(&self, renderer: &mut TerminalRenderer) -> Result<(), DotmaxError>`
   - Renders current front buffer to terminal
   - Supports both colored and non-colored grids
   - Returns error on terminal I/O failure

5. **AC5: Example animation_buffer.rs Demonstrates Bouncing Ball**
   - Create `examples/animation_buffer.rs`
   - Demonstrates complete double-buffering workflow
   - Shows bouncing ball physics simulation
   - Uses: `get_back_buffer()` → draw → `swap_buffers()` → `render()`
   - Includes FPS display and graceful Ctrl+C exit

6. **AC6: Unit Tests Verify Buffer Swap Correctness**
   - Test buffer creation with various dimensions (1x1, 80x24, 200x50)
   - Test swap preserves content (draw to back, swap, verify front has content)
   - Test multiple swaps maintain correct state
   - Test get_back_buffer returns correct buffer after swap
   - Minimum 8 unit tests covering all APIs

7. **AC7: Benchmark Confirms Swap <1ms**
   - Create `benches/animation.rs` with buffer swap microbenchmark
   - Benchmark `swap_buffers()` operation
   - Verify 95th percentile < 1ms
   - Include buffer creation benchmark for baseline

8. **AC8: Zero Clippy Warnings in frame_buffer.rs**
   - `cargo clippy --lib -- -D warnings` passes with zero warnings for animation module
   - No `#[allow(...)]` attributes except where justified with comment
   - Follows Rust naming conventions (snake_case functions, PascalCase types)

9. **AC9: Rustdoc with Examples for All Public Methods**
   - All public functions have `///` doc comments
   - Each method includes at least one `# Examples` code block
   - Examples compile via `cargo test --doc`
   - Module-level documentation explains double-buffering concept
   - Zero rustdoc warnings: `RUSTDOCFLAGS="-D warnings" cargo doc`

## Tasks / Subtasks

- [ ] **Task 1: Create Animation Module Structure** (AC: #1, #8, #9)
  - [ ] 1.1: Create `src/animation/` directory
  - [ ] 1.2: Create `src/animation/mod.rs` with module-level documentation
  - [ ] 1.3: Create `src/animation/frame_buffer.rs` file
  - [ ] 1.4: Add `pub mod animation;` to `src/lib.rs` (always enabled, no feature flag)
  - [ ] 1.5: Add rustdoc explaining double-buffering pattern at module level

- [ ] **Task 2: Implement FrameBuffer Struct** (AC: #1)
  - [ ] 2.1: Define `FrameBuffer` struct with `front: BrailleGrid` and `back: BrailleGrid` fields
  - [ ] 2.2: Implement `FrameBuffer::new(width, height)` constructor
  - [ ] 2.3: Initialize both grids with `BrailleGrid::new(width, height)`
  - [ ] 2.4: Add width/height accessor methods: `pub fn width(&self) -> usize`, `pub fn height(&self) -> usize`
  - [ ] 2.5: Add rustdoc with struct-level documentation

- [ ] **Task 3: Implement get_back_buffer()** (AC: #2)
  - [ ] 3.1: Add `pub fn get_back_buffer(&mut self) -> &mut BrailleGrid`
  - [ ] 3.2: Returns `&mut self.back`
  - [ ] 3.3: Add rustdoc with example showing drawing to back buffer

- [ ] **Task 4: Implement get_front_buffer()** (AC: #4)
  - [ ] 4.1: Add `pub fn get_front_buffer(&self) -> &BrailleGrid`
  - [ ] 4.2: Returns `&self.front` (immutable reference)
  - [ ] 4.3: Add rustdoc explaining this is read-only access to displayed buffer

- [ ] **Task 5: Implement swap_buffers()** (AC: #3)
  - [ ] 5.1: Add `pub fn swap_buffers(&mut self)`
  - [ ] 5.2: Use `std::mem::swap(&mut self.front, &mut self.back)` for O(1) swap
  - [ ] 5.3: Add rustdoc explaining the pointer-swap semantics
  - [ ] 5.4: Document that previous back becomes new front

- [ ] **Task 6: Implement render()** (AC: #4)
  - [ ] 6.1: Add `pub fn render(&self, renderer: &mut TerminalRenderer) -> Result<(), DotmaxError>`
  - [ ] 6.2: Call `renderer.render(&self.front)`
  - [ ] 6.3: Propagate any errors from TerminalRenderer
  - [ ] 6.4: Add rustdoc with complete workflow example

- [ ] **Task 7: Write Unit Tests** (AC: #6)
  - [ ] 7.1: Create `#[cfg(test)] mod tests` in `frame_buffer.rs`
  - [ ] 7.2: Test `new()` creates buffers with correct dimensions
  - [ ] 7.3: Test `get_back_buffer()` returns mutable reference
  - [ ] 7.4: Test `swap_buffers()` exchanges buffers correctly
  - [ ] 7.5: Test content preservation: draw → swap → verify front has content
  - [ ] 7.6: Test multiple sequential swaps maintain correct state
  - [ ] 7.7: Test dimensions (1x1, 80x24, 200x50)
  - [ ] 7.8: Test `width()` and `height()` return correct values

- [ ] **Task 8: Create Performance Benchmark** (AC: #7)
  - [ ] 8.1: Create `benches/animation.rs`
  - [ ] 8.2: Add `[[bench]] name = "animation" harness = false` to Cargo.toml
  - [ ] 8.3: Benchmark `FrameBuffer::new(80, 24)` creation
  - [ ] 8.4: Benchmark `swap_buffers()` operation (target: <1ms)
  - [ ] 8.5: Benchmark `swap_buffers()` for large buffer (200x50)
  - [ ] 8.6: Add benchmark group with proper measurement settings

- [ ] **Task 9: Create Visual Example** (AC: #5)
  - [ ] 9.1: Create `examples/animation_buffer.rs`
  - [ ] 9.2: Implement bouncing ball physics (position, velocity, bounce on edges)
  - [ ] 9.3: Use double-buffering workflow: clear back → draw ball → swap → render
  - [ ] 9.4: Add FPS calculation and display
  - [ ] 9.5: Add Ctrl+C handler for graceful exit using `ctrlc` crate or crossterm events
  - [ ] 9.6: Add comments explaining each step of the animation loop
  - [ ] 9.7: Target 60fps with frame timing

- [ ] **Task 10: Update Module Exports** (AC: #9)
  - [ ] 10.1: Export `FrameBuffer` from `src/animation/mod.rs`
  - [ ] 10.2: Re-export from `src/lib.rs`: `pub use animation::FrameBuffer;`
  - [ ] 10.3: Verify public API is accessible from crate root

- [ ] **Task 11: Final Validation** (AC: All)
  - [ ] 11.1: Run full test suite: `cargo test --lib --all-features`
  - [ ] 11.2: Run clippy: `cargo clippy --lib --example animation_buffer --bench animation -- -D warnings`
  - [ ] 11.3: Run rustdoc: `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`
  - [ ] 11.4: Run doc tests: `cargo test --doc`
  - [ ] 11.5: Run benchmark: `cargo bench --bench animation`
  - [ ] 11.6: Manual test: Run example and verify smooth animation
  - [ ] 11.7: All ACs verified with evidence

## Dev Notes

### Context and Purpose

**Epic 6 Goal:** Enable frame-by-frame animation playback, timing control, frame buffer management, pre-rendering optimization, and flicker-free updates. Support real-time animations at 60+ fps with minimal CPU overhead.

**Story 6.1 Focus:** Double buffering is the foundation of flicker-free animation. By maintaining two buffers (front and back), we can prepare the next frame while displaying the current one, then instantly swap them. This eliminates visual tearing and flickering.

**Value Delivered:** Developers get the core infrastructure for building smooth, professional-quality terminal animations.

### Learnings from Previous Story

**From Story 5.5 (Apply Color Schemes to Grayscale Intensity Buffers) - Status: done**

**Relevant APIs to REUSE:**
- `BrailleGrid` - Core buffer structure from Epic 2
- `BrailleGrid::new(width, height)` - Grid creation
- `BrailleGrid::clear()` - Clear grid between frames
- `BrailleGrid::set_dot()`, `draw_line()`, etc. - Drawing primitives from Epic 4
- `BrailleGrid::apply_color_scheme()` - For colorized animations
- `TerminalRenderer::render()` - Terminal output (supports colors)

**Testing Infrastructure:**
- 353+ library tests passing
- criterion.rs benchmarks established
- Clippy/rustdoc validation patterns established

**No New Files to Integrate From:**
- Story 5.5 was the final Epic 5 story (color system)
- All color APIs are complete and available for animations

[Source: docs/sprint-artifacts/5-5-apply-color-schemes-to-grayscale-intensity-buffers.md#Dev-Agent-Record]

### Architecture Alignment

**From docs/architecture.md:**

**Module Location:**
- Create `src/animation/` directory (new module for Epic 6)
- `src/animation/frame_buffer.rs` - Double buffering implementation
- `src/animation/mod.rs` - Module root with re-exports

**Pattern 3: Buffer Reuse for Animation** (from architecture.md):
- BrailleGrid buffers are reused, not reallocated each frame
- Target <500KB memory overhead per frame
- Critical for 60fps performance

**Error Handling:**
- Use `DotmaxError` for all fallible operations
- `render()` returns `Result<(), DotmaxError>`
- Follow `thiserror` patterns (ADR 0002)

[Source: docs/architecture.md#Pattern-3]

**From docs/sprint-artifacts/tech-spec-epic-6.md:**

**FrameBuffer API (Authoritative):**
```rust
impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self;
    pub fn get_back_buffer(&mut self) -> &mut BrailleGrid;
    pub fn get_front_buffer(&self) -> &BrailleGrid;
    pub fn swap_buffers(&mut self);  // <1ms - pointer swap only
    pub fn render(&self, renderer: &mut TerminalRenderer) -> Result<(), DotmaxError>;
    pub fn width(&self) -> usize;
    pub fn height(&self) -> usize;
}
```

**Performance Requirements:**
- Buffer swap time: <1ms (pointer swap, not data copy)
- Frame rate: 60fps sustained
- Per-frame memory: <500KB

[Source: docs/sprint-artifacts/tech-spec-epic-6.md#Story-6.1]

### Technical Design

**File Structure After Story 6.1:**
```
src/animation/
├── mod.rs            # Module root, re-exports FrameBuffer
└── frame_buffer.rs   # FrameBuffer struct and implementation [NEW]

benches/
└── animation.rs      # Animation benchmarks [NEW]

examples/
└── animation_buffer.rs  # Double-buffering demo [NEW]
```

**FrameBuffer Implementation:**
```rust
// src/animation/frame_buffer.rs
use crate::grid::BrailleGrid;
use crate::render::TerminalRenderer;
use crate::error::DotmaxError;

/// Double-buffered frame management for flicker-free animation.
pub struct FrameBuffer {
    front: BrailleGrid,  // Currently displayed
    back: BrailleGrid,   // Being prepared
}

impl FrameBuffer {
    /// Creates a new double-buffered frame system.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            front: BrailleGrid::new(width, height),
            back: BrailleGrid::new(width, height),
        }
    }

    /// Returns mutable access to the back buffer for drawing.
    pub fn get_back_buffer(&mut self) -> &mut BrailleGrid {
        &mut self.back
    }

    /// Returns immutable access to the front buffer.
    pub fn get_front_buffer(&self) -> &BrailleGrid {
        &self.front
    }

    /// Atomically swaps front and back buffers (O(1) pointer swap).
    pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
    }

    /// Renders the front buffer to the terminal.
    pub fn render(&self, renderer: &mut TerminalRenderer) -> Result<(), DotmaxError> {
        renderer.render(&self.front)
    }

    pub fn width(&self) -> usize { self.front.width() }
    pub fn height(&self) -> usize { self.front.height() }
}
```

**Animation Workflow:**
1. Clear back buffer: `buffer.get_back_buffer().clear()`
2. Draw to back buffer: `draw_circle()`, `set_dot()`, etc.
3. Swap buffers: `buffer.swap_buffers()` (instant)
4. Render front buffer: `buffer.render(&mut renderer)?`
5. Wait for next frame timing (Story 6.2)
6. Repeat

### Project Structure Notes

**New Files:**
```
src/animation/mod.rs          # Created: module root
src/animation/frame_buffer.rs # Created: FrameBuffer struct
benches/animation.rs          # Created: animation benchmarks
examples/animation_buffer.rs  # Created: bouncing ball demo
```

**Modified Files:**
```
src/lib.rs   # Updated: add `pub mod animation;` and re-exports
Cargo.toml   # Updated: add [[bench]] for animation.rs
```

**No Changes To:**
```
src/grid.rs      # BrailleGrid unchanged (reused)
src/render.rs    # TerminalRenderer unchanged (reused)
src/error.rs     # DotmaxError unchanged (reused)
src/color/       # Color system unchanged (available for colorized animations)
src/primitives/  # Drawing primitives unchanged (available for animation)
```

### Dependencies

**Internal Dependencies (from Epic 2):**
- `BrailleGrid` - Core buffer structure
- `TerminalRenderer` - Terminal output
- `DotmaxError` - Error handling

**External Dependencies:**
- `std::mem::swap` - For O(1) buffer swap (stdlib)
- `crossterm` - Terminal events for Ctrl+C handling (already in dependencies)

**No new external dependencies required.**

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-6.md#Story-6.1] - Authoritative acceptance criteria
- [Source: docs/sprint-artifacts/tech-spec-epic-6.md#Detailed-Design] - FrameBuffer API specification
- [Source: docs/sprint-artifacts/tech-spec-epic-6.md#Performance] - Performance targets (<1ms swap)
- [Source: docs/architecture.md#Pattern-3] - Buffer reuse pattern
- [Source: docs/epics.md#Story-6.1] - Epic story definition
- [Source: docs/sprint-artifacts/5-5-apply-color-schemes-to-grayscale-intensity-buffers.md] - Previous story context

## Dev Agent Record

### Context Reference

- docs/sprint-artifacts/6-1-implement-frame-buffer-and-double-buffering.context.xml

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

## Change Log

**2025-11-24 - Story Drafted**
- Story created by SM agent (claude-opus-4-5-20251101)
- Status: drafted (from backlog)
- Epic 6: Animation & Frame Management
- Story 6.1: Implement Frame Buffer and Double Buffering
- Automated workflow execution: /bmad:bmm:workflows:create-story
- Previous story learnings integrated from Story 5.5 (done)
- Ready for story-context workflow to generate technical context XML
