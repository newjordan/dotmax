# Story 3.5.2: Fix Resize/Refresh Behavior

Status: ready-for-dev

## Story

As a **user viewing images in the terminal**,
I want **automatic re-rendering when the terminal window is resized**,
so that **the image scales appropriately without requiring manual intervention**.

## Acceptance Criteria

1. **AC1: Terminal Resize Event Detection**
   - Application detects when terminal dimensions change
   - Uses crossterm's `Event::Resize(width, height)` or equivalent
   - Event polling/listening integrated into example applications
   - Terminal dimension changes captured reliably across platforms (Linux, macOS, Windows)

2. **AC2: Automatic Re-Render on Resize**
   - When resize event detected, trigger complete re-render of current image
   - Fetch new terminal dimensions via `crossterm::terminal::size()`
   - Recalculate target dimensions for braille grid (width/2, height/4)
   - Re-run image pipeline: resize → dither → threshold → map → render

3. **AC3: Aspect Ratio Preservation Maintained**
   - Resized output maintains original aspect ratio (Story 3.2 behavior)
   - ImageRenderer respects original resize/fit logic
   - No distortion or stretching regardless of new terminal size
   - Fits within new terminal bounds (both width and height constraints)

4. **AC4: No Manual Refresh Required**
   - User does not need to restart application
   - User does not need to press refresh key
   - Resize is automatic and seamless
   - Application continues running after resize with updated display

5. **AC5: Performance Acceptable During Resize**
   - Re-render completes within 200ms for typical images (<1MB)
   - No flickering or visual artifacts during resize
   - Resize event debouncing if needed (avoid re-rendering on every pixel change)
   - Application remains responsive during resize operation

6. **AC6: Example Applications Updated**
   - At least 2 image examples demonstrate resize behavior:
     - `simple_image.rs` or equivalent basic example
     - `image_browser.rs` or equivalent interactive example
   - Event loop structured to handle resize events
   - Clear code comments showing resize pattern for future examples

7. **AC7: Cross-Platform Compatibility**
   - Resize works on Linux (tested)
   - Resize works on macOS (if available for testing)
   - Resize works on Windows (if available for testing)
   - No platform-specific hacks or workarounds

8. **AC8: Edge Cases Handled**
   - Very small terminal (<10×10 chars) handled gracefully (don't panic)
   - Very large terminal (>300×100 chars) handled without memory spikes
   - Rapid resize events (user dragging window) don't crash application
   - Terminal size query failures return appropriate errors (no panics)

9. **AC9: Documentation Updated**
   - Example code includes comments explaining resize handling
   - README or docs note automatic resize capability
   - Known limitations documented (if any)
   - Developers understand pattern for implementing resize in their apps

## Tasks / Subtasks

- [ ] **Task 1: Research Current Event Handling in Examples** (AC: #6)
  - [ ] 1.1: Review `examples/simple_image.rs` event loop structure
  - [ ] 1.2: Review `examples/image_browser.rs` event loop structure
  - [ ] 1.3: Check if crossterm Event::Resize is already handled anywhere
  - [ ] 1.4: Identify which examples currently use event polling
  - [ ] 1.5: Document current state (baseline before changes)

- [ ] **Task 2: Implement Resize Event Detection** (AC: #1, #7)
  - [ ] 2.1: Add crossterm Event::Resize handling to example event loops
  - [ ] 2.2: Use `crossterm::event::poll()` or `read()` to capture events
  - [ ] 2.3: Log resize events via tracing for debugging (info level)
  - [ ] 2.4: Test on Linux to confirm events are captured
  - [ ] 2.5: Test on other platforms if available (macOS, Windows)

- [ ] **Task 3: Fetch New Terminal Dimensions** (AC: #2)
  - [ ] 3.1: After resize event, call `crossterm::terminal::size()?`
  - [ ] 3.2: Handle potential errors (no panics, return Result)
  - [ ] 3.3: Convert terminal (cols, rows) to braille grid dimensions (cols/2, rows/4)
  - [ ] 3.4: Log new dimensions for debugging

- [ ] **Task 4: Trigger Image Re-Render** (AC: #2, #3)
  - [ ] 4.1: Store original image path/buffer for re-rendering
  - [ ] 4.2: Re-instantiate ImageRenderer with new terminal dimensions
  - [ ] 4.3: Re-run full pipeline: load (or use cached) → resize → dither → threshold → map → render
  - [ ] 4.4: Verify aspect ratio preservation (use Story 3.2 resize logic)
  - [ ] 4.5: Clear terminal before re-render to avoid artifacts

- [ ] **Task 5: Implement Resize Debouncing (if needed)** (AC: #5)
  - [ ] 5.1: Test rapid resize events (drag terminal window continuously)
  - [ ] 5.2: If performance poor, add debouncing (e.g., 100ms delay after last resize)
  - [ ] 5.3: Use `std::time::Instant` to track last resize time
  - [ ] 5.4: Only re-render if >100ms since last resize event
  - [ ] 5.5: Document debouncing strategy in code comments

- [ ] **Task 6: Update simple_image.rs Example** (AC: #4, #6)
  - [ ] 6.1: Add event loop with resize handling after initial render
  - [ ] 6.2: Structure: loop { poll events → if Resize → re-render → if Quit → break }
  - [ ] 6.3: Keep example simple (<100 lines if possible)
  - [ ] 6.4: Add comments explaining resize pattern
  - [ ] 6.5: Test manually: run example, resize terminal, verify re-render

- [ ] **Task 7: Update image_browser.rs Example** (AC: #4, #6)
  - [ ] 7.1: Integrate resize handling into existing event loop
  - [ ] 7.2: Preserve interactive features (navigation, zoom, etc.)
  - [ ] 7.3: Ensure resize works while browsing multiple images
  - [ ] 7.4: Add comments explaining resize integration with other controls
  - [ ] 7.5: Test manually: run example, navigate images, resize terminal multiple times

- [ ] **Task 8: Edge Case Testing** (AC: #8)
  - [ ] 8.1: Test with very small terminal (10×10) - verify no panic, graceful degradation
  - [ ] 8.2: Test with very large terminal (300×100) - verify no memory spikes
  - [ ] 8.3: Test rapid resize (drag window continuously) - verify stability
  - [ ] 8.4: Simulate terminal size query failure (if possible) - verify error handling
  - [ ] 8.5: Document any edge cases that need graceful failure

- [ ] **Task 9: Performance Validation** (AC: #5)
  - [ ] 9.1: Measure re-render time for typical image (<1MB PNG)
  - [ ] 9.2: Verify <200ms target met
  - [ ] 9.3: Test with large image (4K PNG) - document performance if slower
  - [ ] 9.4: Use RUST_LOG=debug to measure pipeline stages
  - [ ] 9.5: If performance poor, profile and optimize bottleneck

- [ ] **Task 10: Cross-Platform Testing** (AC: #7)
  - [ ] 10.1: Run examples on Linux and verify resize works
  - [ ] 10.2: Run examples on macOS (if available) and verify resize works
  - [ ] 10.3: Run examples on Windows (if available) and verify resize works
  - [ ] 10.4: Document any platform-specific quirks discovered
  - [ ] 10.5: Confirm no #[cfg(target_os = "...")] hacks needed

- [ ] **Task 11: Documentation Updates** (AC: #9)
  - [ ] 11.1: Add "Automatic Resize" section to README.md or docs/
  - [ ] 11.2: Update example README.md with resize behavior notes
  - [ ] 11.3: Add inline comments to examples explaining resize pattern
  - [ ] 11.4: Document any known limitations (e.g., extreme terminal sizes)
  - [ ] 11.5: Provide code snippet template for resize handling

- [ ] **Task 12: Final Integration & Testing** (AC: #1-9)
  - [ ] 12.1: Run `cargo build --examples --all-features` - verify compilation
  - [ ] 12.2: Run `cargo clippy --examples --all-features` - verify zero warnings
  - [ ] 12.3: Run all updated examples manually and test resize on each
  - [ ] 12.4: Verify behavior matches all 9 acceptance criteria
  - [ ] 12.5: Create demo recording or screenshot sequence (optional, for documentation)

## Dev Notes

### Context from Epic 3 Retrospective

**Issue Origin (Story 3.9 Manual Testing):**
From Epic 3 retrospective line 106:
> ⚠️ **Minor Issues Discovered:**
>   - Resize doesn't refresh on terminal window size change

**Priority:** HIGH (Story 3.5.1 in retrospective, renumbered to 3.5.2 after CI story)

**User Impact:**
When user resizes terminal window, the image remains at original size. This breaks expected UX—most terminal applications automatically adjust to new dimensions.

**Estimated Effort:** 1-2 days (per retrospective)

**Epic 3.5 Goal:** Quick polish item to improve UX before Epic 4

### Current Architecture

**Terminal Backend (Story 2.3):**
From `src/render.rs`:
- Uses ratatui + crossterm for terminal I/O
- `TerminalBackend` trait abstracts terminal operations
- `DefaultTerminal` implementation uses crossterm

**Event Handling:**
Crossterm provides `Event` enum with variants:
- `Event::Key(KeyEvent)` - keyboard input
- `Event::Mouse(MouseEvent)` - mouse input
- `Event::Resize(u16, u16)` - terminal dimension change
- `Event::FocusGained` / `Event::FocusLost` - focus events

**Event Polling:**
```rust
use crossterm::event::{poll, read, Event};
use std::time::Duration;

// Check if event available (non-blocking with timeout)
if poll(Duration::from_millis(100))? {
    if let Event::Resize(width, height) = read()? {
        // Handle resize
    }
}
```

**Terminal Size Query:**
```rust
use crossterm::terminal;

let (cols, rows) = terminal::size()?; // Returns (u16, u16)
```

### Technical Approach

**Pattern 1: Event Loop with Resize Handling (Recommended)**
```rust
use crossterm::event::{poll, read, Event, KeyCode};
use std::time::Duration;

// Initial render
let grid = renderer.render_to_grid()?;
terminal.draw_grid(&grid)?;

// Event loop
loop {
    // Poll for events with timeout
    if poll(Duration::from_millis(100))? {
        match read()? {
            Event::Resize(width, height) => {
                // Re-fetch terminal size (more reliable than event args)
                let (cols, rows) = terminal::size()?;

                // Recalculate braille dimensions
                let grid_width = cols / 2;
                let grid_height = rows / 4;

                // Re-render with new dimensions
                let grid = renderer
                    .with_dimensions(grid_width as u32, grid_height as u32)
                    .render_to_grid()?;

                terminal.clear()?;
                terminal.draw_grid(&grid)?;
            }
            Event::Key(key) if key.code == KeyCode::Char('q') => {
                break;
            }
            _ => {}
        }
    }
}
```

**Pattern 2: Debounced Resize (if rapid resizes cause performance issues)**
```rust
use std::time::{Duration, Instant};

let mut last_resize = Instant::now();
const DEBOUNCE_MS: u64 = 100;

loop {
    if poll(Duration::from_millis(100))? {
        match read()? {
            Event::Resize(_, _) => {
                last_resize = Instant::now();
            }
            _ => {}
        }
    }

    // Re-render only if 100ms passed since last resize
    if last_resize.elapsed() >= Duration::from_millis(DEBOUNCE_MS) {
        // Render logic here
        last_resize = Instant::now() + Duration::from_secs(1000); // Prevent re-trigger
    }
}
```

**Recommended:** Start with Pattern 1 (simple event handling). Only add debouncing if performance testing reveals issues.

### Example Structure Changes

**Current: simple_image.rs (one-shot render)**
```rust
fn main() -> Result<()> {
    let renderer = ImageRenderer::new("image.png")?;
    let grid = renderer.render_to_grid()?;

    let mut terminal = DefaultTerminal::new()?;
    terminal.draw_grid(&grid)?;

    Ok(())
}
```

**Updated: simple_image.rs (with resize handling)**
```rust
fn main() -> Result<()> {
    let image_path = "image.png";
    let mut terminal = DefaultTerminal::new()?;

    // Initial render
    render_image(&image_path, &mut terminal)?;

    // Event loop for resize handling
    loop {
        if poll(Duration::from_millis(100))? {
            match read()? {
                Event::Resize(_, _) => {
                    render_image(&image_path, &mut terminal)?;
                }
                Event::Key(key) if key.code == KeyCode::Char('q') => {
                    break;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn render_image(path: &str, terminal: &mut DefaultTerminal) -> Result<()> {
    let (cols, rows) = terminal::size()?;
    let grid_width = cols / 2;
    let grid_height = rows / 4;

    let renderer = ImageRenderer::from_path(path)?
        .with_dimensions(grid_width as u32, grid_height as u32);

    let grid = renderer.render_to_grid()?;
    terminal.clear()?;
    terminal.draw_grid(&grid)?;

    Ok(())
}
```

### Integration with ImageRenderer (Story 3.8)

**High-Level API (from Story 3.8):**
ImageRenderer uses builder pattern:
```rust
let renderer = ImageRenderer::from_path("image.png")?
    .with_dimensions(width, height)
    .with_dither(DitherMethod::FloydSteinberg)
    .with_threshold_auto(); // Otsu
```

**Resize Strategy:**
- Store original image path or buffer
- On resize event, recreate ImageRenderer with new dimensions
- Re-run pipeline (resize → dither → threshold → map)
- Clear terminal and draw new grid

**Aspect Ratio (Story 3.2):**
ImageRenderer already handles aspect ratio preservation. No changes needed—just provide new terminal dimensions and it will fit appropriately.

### Performance Considerations

**Target:** <200ms re-render for typical images

**Pipeline Stages (from Story 3.8):**
1. Load (cached if image already loaded)
2. Resize (ImageMagick-like algorithm, O(n) where n = pixels)
3. Grayscale conversion (O(n))
4. Otsu threshold (<5ms per Story 3.3)
5. Dithering (<15ms per Story 3.4)
6. Map to braille (O(n/8) where n = pixels)
7. Render to terminal (O(m) where m = grid cells)

**Optimization Strategies:**
- Cache loaded image (DynamicImage) to avoid reloading from disk
- Re-use BrailleGrid buffer if dimensions unchanged
- Skip resize if aspect ratio calculation results in same dimensions
- Add debouncing for rapid resize events

**Memory:**
Per architecture (FR71), maintain <5MB baseline. Resize should not spike memory.

### Testing Strategy

**Manual Testing (Primary):**
1. Run example with image
2. Resize terminal window (drag corner)
3. Verify image re-renders automatically
4. Verify aspect ratio maintained
5. Verify no crashes or visual artifacts

**Cross-Platform Testing:**
- Linux: Primary development platform
- macOS: Test if available
- Windows: Test if available

**Edge Cases:**
- Tiny terminal (10×10) - should show degraded but stable image
- Huge terminal (300×100) - should work without memory spike
- Rapid resize (drag continuously) - should remain stable

### Learnings from Previous Story

**From Story 3.5.1 (Add Examples to CI Clippy Gate) - Status: drafted**

Story 3.5.1 added CI checks for examples. This story (3.5.2) will benefit:
- Examples must compile without warnings (clippy enforced)
- Example code held to same quality as src/
- Changes to examples will be caught by CI

**Key Learnings to Apply:**
- Keep examples simple and well-commented
- Test examples manually before committing
- Ensure examples demonstrate best practices
- Use proper error handling (no unwrap in examples)

**Files Modified by Story 3.5.1:**
- `.github/workflows/ci.yml` - CI configuration
- `examples/*.rs` - Fixed clippy warnings

**No Direct Dependencies:** Story 3.5.2 doesn't depend on 3.5.1 code changes, but benefits from CI discipline.

[Source: docs/sprint-artifacts/3-5-1-add-examples-to-ci-clippy-gate.md]

### Zero Panics Discipline

Per ADR and Epic 3 discipline:
- All operations return `Result<T, DotmaxError>`
- Terminal size query: `terminal::size()? `returns Result
- Event reading: `read()?` returns Result
- No unwrap, no expect, no panic!

**Error Handling Pattern:**
```rust
let (cols, rows) = terminal::size().map_err(|e| DotmaxError::TerminalError(e))?;
```

### Known Limitations (to document)

**Potential Limitations:**
1. Extreme terminal sizes (<10×10 or >300×100) may degrade gracefully
2. Very large images (>10MB) may take >200ms to re-render
3. SSH/remote terminals may have resize event delays
4. Terminal emulators may vary in resize event behavior

**Documentation Strategy:**
Document known limitations in README or example comments. Set user expectations.

### Code Quality Standards

**From architecture and ADRs:**
- **Zero Panics:** All code returns `Result<T, DotmaxError>`
- **Clippy Clean:** All code must pass `cargo clippy -- -D warnings`
- **Rustfmt:** All code formatted with rustfmt
- **Documentation:** Examples demonstrate best practices

**Examples Held to Same Standard:** Yes (per Story 3.5.1)

### Project Structure Notes

**Files to Modify:**
- `examples/simple_image.rs` - Add resize handling
- `examples/image_browser.rs` - Add resize handling (if exists)
- `README.md` or `examples/README.md` - Document resize capability

**Files to Read:**
- `src/render.rs` - Understand TerminalBackend API
- `src/image/mod.rs` - Understand ImageRenderer API
- `examples/simple_image.rs` - Current example structure

**No src/ Changes Expected:**
This is primarily an example/documentation story. Core library already supports necessary operations.

### References

- [Source: docs/sprint-artifacts/epic-3-retro-2025-11-21.md#Story-3-5-1] - Resize refresh issue identified in retrospective
- [Source: docs/sprint-artifacts/epic-3-retro-2025-11-21.md#Action-Items-for-Epic-3-5] - Story 3.5.1 (renumbered 3.5.2) acceptance criteria and effort estimate
- [Source: docs/sprint-artifacts/3-9-manual-testing-validation-and-feedback-refinement.md] - Manual testing that discovered resize issue
- [Source: docs/architecture.md#Technology-Stack-Details] - crossterm and ratatui usage
- [Source: docs/architecture.md#Decision-Summary] - Terminal abstraction via TerminalBackend trait
- [Source: src/render.rs] - Terminal rendering implementation (TerminalBackend, DefaultTerminal)
- [Source: src/image/mod.rs] - ImageRenderer high-level API (Story 3.8)
- [Source: examples/simple_image.rs] - Current example structure (baseline)

## Dev Agent Record

### Context Reference

- `docs/sprint-artifacts/3-5-2-fix-resize-refresh-behavior.context.xml`

### Agent Model Used

<!-- Will be filled during implementation -->

### Debug Log References

<!-- Will be added during implementation -->

### Completion Notes List

<!-- Will be filled after story completion -->

### File List

<!-- Will be populated during implementation -->
