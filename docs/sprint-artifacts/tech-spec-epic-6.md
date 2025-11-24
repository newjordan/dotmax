# Epic Technical Specification: Animation & Frame Management

Date: 2025-11-24
Author: Frosty
Epic ID: 6
Status: Draft

---

## Overview

Epic 6 delivers the animation infrastructure for dotmax, enabling developers to create smooth, flicker-free animations at 60+ fps in terminal applications. This epic extracts and professionalizes animation patterns from the crabmusic project while adding new capabilities for frame timing, double buffering, pre-rendering, and differential rendering optimization.

The animation system builds upon the completed BrailleGrid (Epic 2), image rendering pipeline (Epic 3), drawing primitives (Epic 4), and color system (Epic 5) to provide a unified animation experience. This is a critical epic for achieving the PRD's performance targets (NFR-P2: 60fps minimum, <10% CPU) and enabling real-world use cases like loading indicators, visualizations, and motion graphics.

## Objectives and Scope

**In Scope:**
- Double buffering system for flicker-free rendering (FR41, FR43)
- Frame timing and rate control (30fps, 60fps, custom) (FR39, FR40)
- High-level animation loop abstraction for easy API usage (FR38, FR44)
- Frame pre-rendering and caching for optimized playback (FR42)
- Differential rendering to minimize CPU usage at high frame rates (FR70)
- Comprehensive examples and documentation (FR48, FR66-67)

**Out of Scope:**
- Video playback (Phase 2A - requires FFmpeg integration)
- Sprite systems and easing functions (Phase 2A)
- GIF/APNG animation loading (Phase 2A)
- Timeline-based animation API (Phase 2A)
- Async rendering implementation (deferred per ADR 0006 - sync-only MVP)

## System Architecture Alignment

**Components Referenced:**
- `src/animation/` - New module for all animation functionality
  - `frame_buffer.rs` - Double buffering implementation
  - `timing.rs` - Frame rate control
  - `loop.rs` - High-level animation loop
  - `prerender.rs` - Pre-rendered animation storage
  - `differential.rs` - Optimized differential rendering
  - `mod.rs` - Public API exports

**Architecture Constraints:**
- Must achieve <1ms buffer swap (pointer swap, not data copy)
- Must achieve 60fps with <10% single-core CPU (NFR-P2)
- Must reuse BrailleGrid buffers to stay under <500KB per-frame memory (NFR-P3)
- Must integrate with existing TerminalRenderer from Epic 2
- Must follow measure-first optimization approach (ADR 0007)
- Must maintain sync-only API per ADR 0006

**Dependencies:**
- Epic 2 Story 2.1 (BrailleGrid) - Core buffer structure
- Epic 2 Story 2.3 (TerminalRenderer) - Terminal output
- Epic 2 Story 2.5 (Terminal resize handling) - Resize events during animation

## Detailed Design

### Services and Modules

| Module | Responsibilities | Inputs | Outputs |
|--------|------------------|--------|---------|
| `frame_buffer.rs` | Double buffering, atomic swap | BrailleGrid dimensions | Front/back buffer access |
| `timing.rs` | Frame rate control, FPS measurement | Target FPS, actual frame times | Sleep duration, actual FPS |
| `loop.rs` | High-level animation API | Frame callback, FPS config | Animated terminal output |
| `prerender.rs` | Frame caching, playback | Frame sequence, frame rate | Cached animation playback |
| `differential.rs` | Changed-cell detection, selective render | Current/previous frames | Optimized terminal I/O |

### Data Models and Contracts

```rust
// src/animation/frame_buffer.rs
pub struct FrameBuffer {
    front: BrailleGrid,  // Currently displayed
    back: BrailleGrid,   // Being prepared
}

// src/animation/timing.rs
pub struct FrameTimer {
    target_fps: u32,
    frame_duration: Duration,
    last_frame: Instant,
    frame_times: VecDeque<Duration>,  // Rolling window for FPS calc
}

// src/animation/prerender.rs
pub struct PrerenderedAnimation {
    frames: Vec<BrailleGrid>,
    frame_rate: u32,
}

// src/animation/differential.rs
pub struct DifferentialRenderer {
    last_frame: Option<BrailleGrid>,
}

// src/animation/loop.rs
pub struct AnimationLoop<F>
where
    F: FnMut(u64, &mut BrailleGrid) -> Result<bool, DotmaxError>,
{
    width: usize,
    height: usize,
    target_fps: u32,
    on_frame: F,
}
```

### APIs and Interfaces

**FrameBuffer API:**
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

**FrameTimer API:**
```rust
impl FrameTimer {
    pub fn new(target_fps: u32) -> Self;
    pub fn wait_for_next_frame(&mut self);  // Blocks until next frame time
    pub fn actual_fps(&self) -> f32;        // Rolling average FPS
    pub fn frame_time(&self) -> Duration;   // Last frame duration
    pub fn target_fps(&self) -> u32;
    pub fn reset(&mut self);                // Reset timing state
}
```

**AnimationLoop Builder API:**
```rust
impl AnimationLoop {
    pub fn new(width: usize, height: usize) -> AnimationLoopBuilder;
}

impl AnimationLoopBuilder {
    pub fn fps(self, fps: u32) -> Self;
    pub fn on_frame<F>(self, callback: F) -> AnimationLoop<F>
    where
        F: FnMut(u64, &mut BrailleGrid) -> Result<bool, DotmaxError>;
    pub fn build(self) -> Result<AnimationLoop, DotmaxError>;
}

impl<F> AnimationLoop<F> {
    pub fn run(&mut self) -> Result<(), DotmaxError>;  // Main loop, blocks
}
```

**PrerenderedAnimation API:**
```rust
impl PrerenderedAnimation {
    pub fn new(frame_rate: u32) -> Self;
    pub fn add_frame(&mut self, frame: BrailleGrid);
    pub fn play(&self, renderer: &mut TerminalRenderer) -> Result<(), DotmaxError>;
    pub fn play_loop(&self, renderer: &mut TerminalRenderer) -> Result<(), DotmaxError>;
    pub fn frame_count(&self) -> usize;
    pub fn save_to_file(&self, path: &Path) -> Result<(), DotmaxError>;
    pub fn load_from_file(path: &Path) -> Result<Self, DotmaxError>;
}
```

**DifferentialRenderer API:**
```rust
impl DifferentialRenderer {
    pub fn new() -> Self;
    pub fn render_diff(
        &mut self,
        current: &BrailleGrid,
        renderer: &mut TerminalRenderer
    ) -> Result<(), DotmaxError>;
    pub fn invalidate(&mut self);  // Force full render on next call
}
```

### Workflows and Sequencing

**Double Buffering Workflow:**
```
1. User calls get_back_buffer()
2. User draws to back buffer (lines, images, shapes)
3. User calls swap_buffers() - instant pointer swap
4. User calls render() - front buffer rendered to terminal
5. Repeat from step 1
```

**Animation Loop Workflow:**
```
1. AnimationLoop creates FrameBuffer and FrameTimer
2. Loop iteration:
   a. Clear back buffer
   b. Call user's on_frame callback with frame number and back buffer
   c. If callback returns Ok(false), exit loop
   d. Swap buffers
   e. Render front buffer to terminal
   f. Wait for next frame (FrameTimer.wait_for_next_frame())
3. Check for Ctrl+C signal
4. Repeat or exit
```

**Differential Rendering Workflow:**
```
1. Compare current BrailleGrid with cached last_frame
2. For each cell (x, y):
   - If dots differ OR colors differ:
     - Add (x, y) to changed_cells list
3. For each changed cell:
   - Move cursor to position: \x1b[{row};{col}H
   - Output braille character with color
4. Store current frame as last_frame
5. On terminal resize: invalidate() to force full render
```

## Non-Functional Requirements

### Performance

| Metric | Target | Validation Method |
|--------|--------|-------------------|
| Buffer swap time | <1ms | Benchmark in `benches/animation.rs` |
| Frame rate | 60fps sustained | Animation example with FPS display |
| CPU usage at 60fps | <10% single core | Profile with perf/flamegraph |
| Per-frame memory | <500KB | Valgrind heap profiling |
| Timing accuracy | ±2ms at 60fps | Unit test with timing assertions |
| Differential render savings | 60-80% I/O reduction | Benchmark comparing full vs diff |

**Benchmark Requirements:**
- `benches/animation.rs` must include:
  - Buffer swap microbenchmark (target: <1ms)
  - Full frame render vs differential render comparison
  - Animation loop overhead measurement
  - Pre-rendered playback performance

### Security

- No unsafe code required for animation implementation
- All timing uses `std::time` (no external dependencies)
- Input validation on FPS values (min 1, max 240)
- Memory bounds enforced by BrailleGrid (from Epic 2)

### Reliability/Availability

- Graceful Ctrl+C handling in animation loops
- Terminal resize detection during animation (invalidate differential cache)
- No panics in animation code (all errors via Result)
- Frame dropping when behind schedule (no catchup accumulation)

### Observability

- `tracing::debug!` for frame timing information
- `tracing::trace!` for per-frame metrics (only when TRACE enabled)
- FPS counter API for runtime monitoring (`actual_fps()`)
- Frame time API for performance debugging (`frame_time()`)

## Dependencies and Integrations

### Internal Dependencies

| Dependency | Module | Version | Purpose |
|------------|--------|---------|---------|
| BrailleGrid | `src/grid.rs` | Epic 2 | Core buffer structure |
| TerminalRenderer | `src/render.rs` | Epic 2 | Terminal output |
| Color | `src/color/mod.rs` | Epic 5 | Color support for cells |
| DotmaxError | `src/error.rs` | Epic 2 | Error handling |

### External Dependencies

| Crate | Version | Purpose | Feature Gate |
|-------|---------|---------|--------------|
| std::time | stdlib | High-precision timing | None (always) |
| std::thread | stdlib | Sleep for frame timing | None (always) |
| crossterm | 0.29 | Terminal I/O, cursor control | None (core) |

**No new external dependencies required for Epic 6.** All functionality implementable with stdlib and existing crossterm/ratatui.

## Acceptance Criteria (Authoritative)

### Story 6.1: Frame Buffer and Double Buffering
1. **AC6.1.1**: `FrameBuffer::new(width, height)` creates two BrailleGrid buffers
2. **AC6.1.2**: `get_back_buffer()` returns mutable reference to back buffer
3. **AC6.1.3**: `swap_buffers()` exchanges front/back in <1ms (pointer swap)
4. **AC6.1.4**: `render()` outputs front buffer via TerminalRenderer
5. **AC6.1.5**: Example `examples/animation_buffer.rs` demonstrates bouncing ball
6. **AC6.1.6**: Unit tests verify buffer swap correctness (content preserved)
7. **AC6.1.7**: Benchmark confirms swap <1ms
8. **AC6.1.8**: Zero clippy warnings in `frame_buffer.rs`
9. **AC6.1.9**: Rustdoc with examples for all public methods

### Story 6.2: Frame Timing and Rate Control
1. **AC6.2.1**: `FrameTimer::new(fps)` initializes with target frame rate
2. **AC6.2.2**: `wait_for_next_frame()` sleeps appropriate duration
3. **AC6.2.3**: `actual_fps()` returns rolling average of real frame rate
4. **AC6.2.4**: `frame_time()` returns duration of last frame
5. **AC6.2.5**: Timing accuracy within ±2ms at 60fps
6. **AC6.2.6**: Example `examples/fps_control.rs` displays real-time FPS
7. **AC6.2.7**: Handles frame drops gracefully (no sleep if behind)
8. **AC6.2.8**: Zero clippy warnings in `timing.rs`
9. **AC6.2.9**: Rustdoc with examples for all public methods

### Story 6.3: Animation Loop Helper
1. **AC6.3.1**: Builder pattern: `AnimationLoop::new(w, h).fps(60).on_frame(cb).run()`
2. **AC6.3.2**: Callback receives frame number and mutable back buffer
3. **AC6.3.3**: Loop handles buffer management automatically
4. **AC6.3.4**: Loop handles timing automatically
5. **AC6.3.5**: Loop handles terminal rendering automatically
6. **AC6.3.6**: Ctrl+C detection for graceful exit
7. **AC6.3.7**: Example `examples/simple_animation.rs` in <30 lines
8. **AC6.3.8**: Zero clippy warnings in `loop.rs`
9. **AC6.3.9**: Rustdoc with examples for all public methods

### Story 6.4: Frame Pre-Rendering and Caching
1. **AC6.4.1**: `PrerenderedAnimation::new(fps)` creates empty animation
2. **AC6.4.2**: `add_frame()` stores BrailleGrid in sequence
3. **AC6.4.3**: `play()` renders frames at specified rate
4. **AC6.4.4**: `play_loop()` repeats indefinitely until Ctrl+C
5. **AC6.4.5**: `frame_count()` returns number of stored frames
6. **AC6.4.6**: `save_to_file()` serializes animation to disk
7. **AC6.4.7**: `load_from_file()` deserializes animation from disk
8. **AC6.4.8**: Example `examples/prerendered_demo.rs` shows usage
9. **AC6.4.9**: Zero clippy warnings in `prerender.rs`

### Story 6.5: Differential Rendering Optimization
1. **AC6.5.1**: `DifferentialRenderer::new()` creates renderer
2. **AC6.5.2**: `render_diff()` compares frames and renders only changes
3. **AC6.5.3**: First frame renders fully (no previous to compare)
4. **AC6.5.4**: `invalidate()` forces full render on next call
5. **AC6.5.5**: Terminal resize triggers automatic invalidation
6. **AC6.5.6**: 60-80% I/O reduction verified in benchmarks
7. **AC6.5.7**: Example `examples/differential_demo.rs` shows CPU savings
8. **AC6.5.8**: Zero clippy warnings in `differential.rs`
9. **AC6.5.9**: Rustdoc with examples for all public methods

### Story 6.6: Animation Examples and Documentation
1. **AC6.6.1**: `examples/animations/bouncing_ball.rs` - physics simulation
2. **AC6.6.2**: `examples/animations/loading_spinner.rs` - rotating spinner
3. **AC6.6.3**: `examples/animations/waveform.rs` - waveform visualization
4. **AC6.6.4**: `examples/animations/fireworks.rs` - particle system
5. **AC6.6.5**: `examples/animations/clock.rs` - animated analog clock
6. **AC6.6.6**: `docs/animation_guide.md` comprehensive tutorial
7. **AC6.6.7**: All animation types have rustdoc with examples
8. **AC6.6.8**: README updated with animation section
9. **AC6.6.9**: All examples pass clippy and compile

## Traceability Mapping

| AC | Spec Section | Component/API | Test Idea |
|----|--------------|---------------|-----------|
| AC6.1.1-6.1.4 | Detailed Design - FrameBuffer API | `FrameBuffer` | Unit test: create, draw, swap, verify |
| AC6.1.5 | Workflows | `examples/animation_buffer.rs` | Example compiles and runs |
| AC6.1.7 | NFR Performance | `benches/animation.rs` | Benchmark: swap < 1ms |
| AC6.2.1-6.2.4 | Detailed Design - FrameTimer API | `FrameTimer` | Unit test: timing accuracy |
| AC6.2.5 | NFR Performance | `timing.rs` | Test: 100 frames at 60fps, check variance |
| AC6.3.1-6.3.6 | Detailed Design - AnimationLoop API | `AnimationLoop` | Integration test: full animation run |
| AC6.3.7 | APIs | `examples/simple_animation.rs` | Line count < 30, compiles |
| AC6.4.1-6.4.7 | Detailed Design - PrerenderedAnimation | `PrerenderedAnimation` | Unit test: add, play, save/load |
| AC6.5.1-6.5.5 | Detailed Design - DifferentialRenderer | `DifferentialRenderer` | Unit test: diff calculation |
| AC6.5.6 | NFR Performance | `benches/animation.rs` | Benchmark: full vs diff |
| AC6.6.1-6.6.5 | Examples | `examples/animations/*` | All examples compile and run |
| AC6.6.6 | Documentation | `docs/animation_guide.md` | Manual review: comprehensive |

## Risks, Assumptions, Open Questions

### Risks
- **R1**: Windows timer granularity (~15ms default) may affect 60fps timing
  - *Mitigation*: Document Windows behavior; consider `timeBeginPeriod` for advanced users
- **R2**: Terminal I/O may become bottleneck at high FPS
  - *Mitigation*: Differential rendering (Story 6.5) addresses this
- **R3**: Pre-rendered animations may consume significant memory for long animations
  - *Mitigation*: Document memory characteristics; recommend max 10 seconds at 30fps

### Assumptions
- **A1**: Terminal supports ANSI cursor positioning (`\x1b[row;colH`)
- **A2**: BrailleGrid from Epic 2 is performant for animation workloads
- **A3**: crossterm handles Ctrl+C signal detection adequately
- **A4**: `std::thread::sleep` provides sufficient precision for frame timing

### Open Questions
- **Q1**: Should we provide async animation API in this epic or defer to post-1.0?
  - *Resolution*: Defer per ADR 0006 (sync-only MVP)
- **Q2**: Should `PrerenderedAnimation` use compression for file storage?
  - *Recommendation*: No compression in MVP; simple binary format sufficient

## Test Strategy Summary

### Test Levels

| Level | Focus | Framework | Coverage Target |
|-------|-------|-----------|-----------------|
| Unit | Individual functions | `#[cfg(test)]` | 80%+ line coverage |
| Integration | Module interactions | `tests/` | All public APIs |
| Benchmark | Performance validation | criterion | All NFR targets |
| Example | API usability | `examples/` | All examples compile |

### Test Categories

**Unit Tests (per story):**
- Story 6.1: Buffer creation, swap correctness, dimensions
- Story 6.2: Timer initialization, sleep duration calculation, FPS measurement
- Story 6.3: Builder pattern, callback invocation, loop termination
- Story 6.4: Frame storage, playback order, file I/O
- Story 6.5: Diff calculation, changed cell detection, invalidation

**Integration Tests:**
- Full animation loop with drawing primitives
- Animation with image rendering (requires `image` feature)
- Terminal resize during animation
- Ctrl+C handling

**Benchmark Tests (`benches/animation.rs`):**
- Buffer swap microbenchmark (target: <1ms)
- Full render vs differential render comparison
- Animation loop overhead (target: <1ms per iteration)
- FPS accuracy measurement at 30fps, 60fps, 120fps

**Edge Cases:**
- Zero-size buffers
- Extremely high FPS (240fps)
- Very long animations (1000+ frames pre-rendered)
- Terminal resize mid-animation
- Frame callback returning error
