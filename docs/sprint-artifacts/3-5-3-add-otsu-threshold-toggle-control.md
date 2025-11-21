# Story 3.5.3: Add Otsu Threshold Toggle Control

Status: drafted

## Story

As a **user interactively adjusting image rendering**,
I want **the ability to toggle between automatic Otsu thresholding and manual threshold values**,
so that **I can fine-tune image contrast for optimal braille output in different lighting scenarios**.

## Acceptance Criteria

1. **AC1: Add Threshold Toggle Control to Interactive Examples**
   - `image_browser.rs` example adds keyboard controls for threshold mode:
     - Press `O` (capital O) to toggle between Auto (Otsu) and Manual modes
     - Press `+` to increase manual threshold by 10 (range: 0-255)
     - Press `-` to decrease manual threshold by 10 (range: 0-255)
   - UI displays current threshold mode and value (e.g., "Threshold: Auto (Otsu)" or "Threshold: Manual (128)")
   - Default mode remains Auto (Otsu) for backward compatibility

2. **AC2: Manual Threshold Value Control**
   - When in Manual mode, `+` and `-` keys adjust threshold value
   - Threshold value clamped to valid range [0, 255]
   - Value changes trigger immediate re-render with new threshold
   - Changes are smooth and responsive (<200ms per re-render)

3. **AC3: Threshold Mode State Management**
   - `RenderSettings` struct in `image_browser.rs` tracks:
     - `threshold_mode: ThresholdMode` enum (Auto vs Manual)
     - `manual_threshold_value: u8` (default: 128)
   - Toggle between modes preserves last manual threshold value
   - Switching to Auto mode ignores manual value, uses Otsu calculation
   - Switching back to Manual mode restores previous manual value

4. **AC4: ImageRenderer API Integration**
   - `ImageRenderer` builder pattern already has `.threshold(u8)` method
   - Manual mode calls `.threshold(value)` before `.render()`
   - Auto mode omits `.threshold()` call, allowing Otsu to execute
   - No changes needed to ImageRenderer itself (API already supports both modes)

5. **AC5: UI Feedback and Display**
   - Status footer shows current threshold mode clearly
   - Format examples:
     - `Threshold: Auto (Otsu)`
     - `Threshold: Manual (128)`
     - `Threshold: Manual (200)`
   - UI updates immediately when mode or value changes
   - Help text in footer explains threshold controls

6. **AC6: Reset Functionality**
   - Existing `R` (reset) key resets threshold to Auto (Otsu) mode
   - Reset also sets manual threshold value back to default (128)
   - Threshold reset included in RenderSettings::reset() implementation

7. **AC7: Edge Cases and Validation**
   - Manual threshold value clamped to [0, 255] range
   - Value 0 = all pixels white (no dots)
   - Value 255 = all pixels black (all dots)
   - Threshold adjustments work correctly with all dithering modes
   - Threshold adjustments work correctly with all color modes

8. **AC8: Documentation**
   - `image_browser.rs` comments explain threshold mode toggle
   - Example README.md updated with threshold control instructions
   - Help text in UI footer includes threshold controls:
     - `O (toggle Otsu/Manual)`
     - `+/- (adjust manual threshold)`
   - Known limitation documented: Threshold only affects binary conversion (not relevant for color sampling)

9. **AC9: Testing and Validation**
   - Manual testing with `image_browser.rs`:
     - Toggle between Auto and Manual modes works
     - Manual threshold adjustments visible in output
     - Otsu mode produces optimal contrast automatically
     - Reset (`R`) correctly resets threshold mode
   - Test with various image types (photos, line art, diagrams)
   - Verify threshold control works with all dithering methods

## Tasks / Subtasks

- [ ] **Task 1: Define Threshold Mode Types** (AC: #3)
  - [ ] 1.1: Add `ThresholdMode` enum to `RenderSettings` in `image_browser.rs`
  - [ ] 1.2: Define enum variants: `Auto` (Otsu) and `Manual(u8)`
  - [ ] 1.3: Add `threshold_mode: ThresholdMode` field to `RenderSettings`
  - [ ] 1.4: Initialize default to `ThresholdMode::Auto`
  - [ ] 1.5: Update `RenderSettings::reset()` to reset threshold mode to Auto

- [ ] **Task 2: Implement Threshold Mode Toggle Logic** (AC: #1, #3)
  - [ ] 2.1: Add `toggle_threshold_mode()` method to `RenderSettings`
  - [ ] 2.2: Toggle logic: Auto → Manual(128) → Auto (cycle)
  - [ ] 2.3: When switching to Manual, initialize to 128 or last manual value
  - [ ] 2.4: When switching to Auto, preserve manual value for next toggle
  - [ ] 2.5: Add comments explaining toggle behavior

- [ ] **Task 3: Implement Manual Threshold Adjustment** (AC: #2, #7)
  - [ ] 3.1: Add `adjust_threshold()` method to `RenderSettings`
  - [ ] 3.2: Accept delta parameter (e.g., +10 or -10)
  - [ ] 3.3: Only adjust if current mode is Manual (no-op if Auto)
  - [ ] 3.4: Clamp result to [0, 255] range
  - [ ] 3.5: Update manual threshold value in state

- [ ] **Task 4: Add Keyboard Controls to image_browser.rs** (AC: #1)
  - [ ] 4.1: Update `handle_key()` function to handle new keys
  - [ ] 4.2: Add `KeyCode::Char('O')` case for toggle threshold mode
  - [ ] 4.3: Add `KeyCode::Char('+')` case for increase threshold
  - [ ] 4.4: Add `KeyCode::Char('-')` case for decrease threshold
  - [ ] 4.5: Call appropriate `RenderSettings` methods for each key

- [ ] **Task 5: Integrate Threshold Mode with ImageRenderer** (AC: #4)
  - [ ] 5.1: Update `try_render_image()` function in `ImageBrowser`
  - [ ] 5.2: Check current threshold mode before building image
  - [ ] 5.3: If `ThresholdMode::Manual(value)`, call `builder.threshold(value)`
  - [ ] 5.4: If `ThresholdMode::Auto`, do not call `.threshold()` (use Otsu)
  - [ ] 5.5: Verify integration works correctly

- [ ] **Task 6: Update UI Display** (AC: #5, #8)
  - [ ] 6.1: Update `display_string()` method in `RenderSettings`
  - [ ] 6.2: Add threshold mode to output string
  - [ ] 6.3: Format: "Threshold: Auto (Otsu)" or "Threshold: Manual (128)"
  - [ ] 6.4: Update `display_footer()` to include threshold controls in help text
  - [ ] 6.5: Add line: "│ Threshold: O (toggle) | +/- (adjust manual)"

- [ ] **Task 7: Manual Testing** (AC: #9)
  - [ ] 7.1: Run `image_browser.rs` with sample images
  - [ ] 7.2: Test toggle between Auto and Manual modes (press `O`)
  - [ ] 7.3: Test manual threshold adjustment (press `+` and `-`)
  - [ ] 7.4: Verify threshold value displayed correctly in UI
  - [ ] 7.5: Test with different image types (photos, line art, diagrams)
  - [ ] 7.6: Test with all dithering methods (Floyd-Steinberg, Bayer, Atkinson, None)
  - [ ] 7.7: Test with all color modes (Monochrome, Grayscale, TrueColor)
  - [ ] 7.8: Verify reset (`R`) resets threshold to Auto mode
  - [ ] 7.9: Test edge cases (threshold 0, threshold 255)

- [ ] **Task 8: Update Documentation** (AC: #8)
  - [ ] 8.1: Update `image_browser.rs` module-level doc comments
  - [ ] 8.2: Add threshold controls to Controls section
  - [ ] 8.3: Update `examples/README.md` with threshold toggle feature
  - [ ] 8.4: Add inline comments explaining threshold mode logic
  - [ ] 8.5: Document known limitations (threshold only for binary conversion)

- [ ] **Task 9: Code Quality Checks** (AC: all)
  - [ ] 9.1: Run `cargo clippy --examples --all-features -- -D warnings`
  - [ ] 9.2: Fix any clippy warnings in modified examples
  - [ ] 9.3: Run `cargo fmt` to format all code
  - [ ] 9.4: Verify examples compile: `cargo build --examples --all-features`
  - [ ] 9.5: No compiler warnings or errors

## Dev Notes

### Context from Epic 3 Retrospective

**Issue Origin (Story 3.9 Manual Testing):**
From Epic 3 retrospective line 376-388:
> ### Story 3.5.2: Add Otsu Threshold Toggle Control ⭐ MEDIUM PRIORITY
>
> **Issue:** No way to switch between auto (Otsu) and manual threshold
>
> **Acceptance Criteria:**
> - `ImageRenderer::threshold_mode(ThresholdMode::Auto | ThresholdMode::Manual(u8))`
> - Default remains Auto (Otsu) for backward compatibility
> - Manual mode accepts 0-255 threshold value
> - Example program demonstrates both modes
>
> **Estimated Effort:** Small (1 day)
>
> **Rationale:** Power-user feature for fine-tuning

**Note:** This is Story 3.5.2 in the retrospective but renumbered to 3.5.3 in sprint-status.yaml after CI story (3.5.1) was prioritized first and story 3.5.2 became resize/refresh.

**Priority:** MEDIUM (from retrospective)

**User Impact:**
Power users and developers testing image rendering need the ability to override automatic Otsu thresholding. This is especially useful for:
- Images where Otsu produces suboptimal results (e.g., high-key or low-key images)
- Testing and comparing different threshold values
- Fine-tuning contrast for specific terminal/display combinations
- Educational purposes (demonstrating effect of threshold on output)

**Epic 3.5 Goal:** Polish Epic 3 image rendering with UX improvements before Epic 4

### Current Implementation Analysis

**Existing ImageRenderer API (src/image/mod.rs:690-693):**
```rust
pub fn threshold(mut self, value: u8) -> Self {
    self.threshold = Some(value);
    self
}
```

**Threshold Logic in render() (src/image/mod.rs:824-830):**
```rust
let binary = if let Some(threshold_value) = self.threshold {
    debug!("Applying manual threshold: {}", threshold_value);
    apply_threshold(&gray, threshold_value)
} else if self.dithering == DitheringMethod::None {
    debug!("Applying automatic Otsu thresholding");
    auto_threshold(&gray)
} else {
    // Dithering path...
}
```

**Key Insight:** ImageRenderer already supports both modes:
- `.threshold(value)` sets manual threshold (Some(value))
- Omitting `.threshold()` uses Auto/Otsu mode (None)

**This story does NOT need to modify ImageRenderer API.** It only needs to add interactive UI controls to `image_browser.rs` to toggle between calling `.threshold(value)` or not.

### Current image_browser.rs Structure

**RenderSettings struct (lines 34-40):**
```rust
struct RenderSettings {
    color_mode: ColorMode,
    dithering: DitheringMethod,
    brightness: f32,
    contrast: f32,
    gamma: f32,
}
```

**Need to add:**
```rust
threshold_mode: ThresholdMode,  // New field
```

**Where ThresholdMode is:**
```rust
#[derive(Debug, Clone, Copy)]
enum ThresholdMode {
    Auto,          // Use Otsu automatic thresholding
    Manual(u8),    // Use manual threshold value
}
```

**Existing Controls (lines 8-15):**
- **Left/Right Arrow**: Previous/Next image
- **C**: Cycle color mode
- **D**: Cycle dithering algorithm
- **b/B**: Adjust brightness (+/-)
- **t/T**: Adjust contrast (+/-)
- **g/G**: Adjust gamma (+/-)
- **R**: Reset all adjustments
- **Q or Esc**: Quit

**New Controls to Add:**
- **O**: Toggle threshold mode (Auto ↔ Manual)
- **+**: Increase manual threshold by 10
- **-**: Decrease manual threshold by 10

### Implementation Strategy

**Step 1: Add ThresholdMode to RenderSettings**
```rust
#[derive(Debug, Clone, Copy)]
enum ThresholdMode {
    Auto,          // Use Otsu
    Manual(u8),    // Use specified value
}

impl Default for ThresholdMode {
    fn default() -> Self {
        Self::Auto  // Default to Otsu for backward compatibility
    }
}

struct RenderSettings {
    // ... existing fields ...
    threshold_mode: ThresholdMode,
}
```

**Step 2: Add Methods to RenderSettings**
```rust
impl RenderSettings {
    fn toggle_threshold_mode(&mut self) {
        self.threshold_mode = match self.threshold_mode {
            ThresholdMode::Auto => ThresholdMode::Manual(128),  // Default mid-point
            ThresholdMode::Manual(val) => ThresholdMode::Auto,
        };
    }

    fn adjust_threshold(&mut self, delta: i16) {
        if let ThresholdMode::Manual(ref mut val) = self.threshold_mode {
            let new_val = (*val as i16 + delta).clamp(0, 255) as u8;
            *val = new_val;
        }
        // If Auto mode, do nothing
    }

    fn display_string(&self) -> String {
        let threshold_str = match self.threshold_mode {
            ThresholdMode::Auto => "Auto (Otsu)".to_string(),
            ThresholdMode::Manual(val) => format!("Manual ({})", val),
        };
        format!(
            "Color: {:?} | Dither: {:?} | Threshold: {} | Brightness: {:.1} | ...",
            self.color_mode, self.dithering, threshold_str, self.brightness, ...
        )
    }
}
```

**Step 3: Update try_render_image() Integration**
```rust
fn try_render_image(&mut self, is_svg: bool) -> Result<...> {
    let mut builder = ImageRenderer::new()
        .dithering(self.settings.dithering)
        .color_mode(self.settings.color_mode);

    // Apply manual threshold if in Manual mode
    builder = match self.settings.threshold_mode {
        ThresholdMode::Manual(value) => builder.threshold(value),
        ThresholdMode::Auto => builder,  // Don't call .threshold(), use Otsu
    };

    // ... rest of pipeline ...
}
```

**Step 4: Add Keyboard Handlers**
```rust
fn handle_key(&mut self, key: KeyEvent) -> Result<ControlFlow, ...> {
    match key.code {
        KeyCode::Char('O') => {
            self.settings.toggle_threshold_mode();
            Ok(ControlFlow::Continue)
        }
        KeyCode::Char('+') => {
            self.settings.adjust_threshold(10);
            Ok(ControlFlow::Continue)
        }
        KeyCode::Char('-') => {
            self.settings.adjust_threshold(-10);
            Ok(ControlFlow::Continue)
        }
        // ... existing handlers ...
    }
}
```

**Step 5: Update UI Footer**
```rust
println!("│ Controls: ← → (prev/next) | C (color) | D (dither) | O (threshold) | R (reset)");
println!("│ Adjust:   b/B (brightness) | t/T (contrast) | g/G (gamma) | +/- (threshold)");
```

### Technical Considerations

**Performance:**
- Threshold changes trigger full re-render (same as brightness/contrast/gamma)
- Target: <200ms per re-render (same as AC5 in Story 3.5.2)
- ImageRenderer already has caching for resized images (Issue #3 fix in Story 3.9)
- Threshold only affects binary conversion stage (fast: ~5ms per Story 3.3)

**Threshold Value Range:**
- Valid range: [0, 255] (u8 grayscale range)
- Value 0 = all pixels white (no dots) - extreme case
- Value 255 = all pixels black (all dots) - extreme case
- Value 128 = mid-point (good default for manual mode)
- Otsu typically calculates values in range [50, 200] depending on image

**Interaction with Dithering:**
- Dithering and thresholding are alternative approaches:
  - If `dithering != None`: Uses dithering (ignores threshold)
  - If `dithering == None`: Uses threshold (Auto Otsu or Manual)
- This story's threshold toggle only matters when `dithering == None`
- **Design Decision:** Allow threshold toggle regardless of dither mode, but document that threshold only applies when dithering is None

**Interaction with Color Mode:**
- Threshold only affects binary conversion (monochrome mode)
- Color modes (Grayscale, TrueColor) use different rendering path
- Threshold control may not be relevant for color modes (document limitation)

### Learnings from Previous Story

**From Story 3.5.2 (Fix Resize/Refresh Behavior) - Status: drafted**

Story 3.5.2 added terminal resize event handling to examples. This story (3.5.3) will benefit:
- Examples already have event loop infrastructure
- Keyboard event handling patterns established
- RenderSettings pattern for managing state
- Re-render on state change pattern proven

**Key Learnings to Apply:**
- Keep keyboard controls simple and intuitive
- Use existing RenderSettings pattern for state management
- Trigger re-render immediately on control changes
- Update UI footer to show current state clearly
- Test manually with various images

**Files Modified by Story 3.5.2:**
- `examples/simple_image.rs` - Added resize handling
- `examples/image_browser.rs` - Added resize handling to event loop
- `README.md` or `examples/README.md` - Documented resize capability

**No Direct Dependencies:** Story 3.5.3 doesn't depend on 3.5.2 code, but both modify `image_browser.rs` event loop.

[Source: docs/sprint-artifacts/3-5-2-fix-resize-refresh-behavior.md]

### Zero Panics Discipline

Per ADR and Epic 3 discipline:
- All operations return `Result<T, DotmaxError>`
- Threshold value validated and clamped (no panic on invalid input)
- No unwrap, no expect, no panic!

**Validation Pattern:**
```rust
fn adjust_threshold(&mut self, delta: i16) {
    if let ThresholdMode::Manual(ref mut val) = self.threshold_mode {
        // Clamp to valid range [0, 255] - no panic possible
        let new_val = (*val as i16 + delta).clamp(0, 255) as u8;
        *val = new_val;
    }
}
```

### Testing Strategy

**Manual Testing (Primary):**
1. Run `cargo run --example image_browser --features image,svg`
2. Load various images (photos, line art, diagrams)
3. Test threshold toggle:
   - Press `O` to switch to Manual mode (should show "Threshold: Manual (128)")
   - Press `+` multiple times (threshold value should increase)
   - Press `-` multiple times (threshold value should decrease)
   - Press `O` again to switch to Auto mode (should show "Threshold: Auto (Otsu)")
4. Verify visual differences:
   - Manual low threshold (e.g., 50) → darker image (more black dots)
   - Manual high threshold (e.g., 200) → lighter image (fewer black dots)
   - Auto (Otsu) → optimal contrast automatically
5. Test with different dithering modes:
   - Set dithering to `None` (press `D` until None)
   - Verify threshold control works
   - Set dithering to `FloydSteinberg` (press `D`)
   - Threshold control should be ignored (dithering takes precedence)
6. Test reset:
   - Press `R` to reset
   - Verify threshold mode resets to Auto

**Image Types to Test:**
- **High-key image** (mostly bright): Otsu may threshold too high, manual lower value helps
- **Low-key image** (mostly dark): Otsu may threshold too low, manual higher value helps
- **Line art/diagrams**: Threshold very important for clean lines
- **Photographs**: Otsu usually optimal, manual for special effects

### Known Limitations (to document)

1. **Threshold Only Affects Binary Mode:**
   - Threshold control only matters when `dithering == None`
   - When dithering is enabled (Floyd-Steinberg, Bayer, Atkinson), dithering algorithm is used instead of threshold
   - Document in UI or example comments

2. **Color Mode Interaction:**
   - Threshold primarily affects monochrome mode
   - Color modes (Grayscale, TrueColor) use different rendering pipeline
   - Threshold may have limited effect or no effect in color modes
   - Document expected behavior

3. **No Persistent State:**
   - Threshold mode and value reset when changing images
   - Could add persistence in future (save per-image settings)
   - Not a priority for this story (Epic 3.5 is quick polish)

### Code Quality Standards

**From architecture and ADRs:**
- **Zero Panics:** All code returns `Result<T, DotmaxError>` or uses validated values
- **Clippy Clean:** All code must pass `cargo clippy -- -D warnings`
- **Rustfmt:** All code formatted with rustfmt
- **Documentation:** Examples demonstrate best practices
- **Examples Held to Same Standard:** Yes (per Story 3.5.1)

### Project Structure Notes

**Files to Modify:**
- `examples/image_browser.rs` - Add threshold mode toggle and controls
- `examples/README.md` or main `README.md` - Document threshold toggle feature

**Files to Read:**
- `src/image/mod.rs` - Understand ImageRenderer API and threshold behavior
- `src/image/threshold.rs` - Understand Otsu and manual threshold functions
- `examples/image_browser.rs` - Current implementation (RenderSettings, event loop)

**No src/ Changes Needed:**
ImageRenderer API already supports both automatic and manual threshold modes. This story only adds interactive UI controls to examples.

### References

- [Source: docs/sprint-artifacts/epic-3-retro-2025-11-21.md#Story-3-5-2] - Otsu threshold toggle issue identified in retrospective
- [Source: docs/sprint-artifacts/epic-3-retro-2025-11-21.md#Action-Items-for-Epic-3-5] - Story 3.5.2 acceptance criteria and effort estimate
- [Source: docs/sprint-artifacts/3-9-manual-testing-validation-and-feedback-refinement.md] - Manual testing that identified need for threshold toggle
- [Source: src/image/mod.rs:690-693] - ImageRenderer `.threshold(u8)` method
- [Source: src/image/mod.rs:824-830] - Threshold logic in render() pipeline
- [Source: src/image/threshold.rs:180-320] - Otsu threshold and manual threshold implementations
- [Source: examples/image_browser.rs] - Current interactive example structure
- [Source: docs/architecture.md#Technology-Stack-Details] - crossterm event handling
- [Source: docs/sprint-artifacts/3-5-2-fix-resize-refresh-behavior.md] - Similar example modification pattern

### Epic 3 Context

**Story Position:** Story 3.5.3 in Epic 3.5 (Polish & Refinement Sprint)

**Dependencies:**
- **Story 3.3 (Otsu Thresholding):** Provides `otsu_threshold()` and `apply_threshold()` functions
- **Story 3.8 (High-Level API):** Provides `ImageRenderer` builder pattern with `.threshold()` method
- **Story 3.5.1 (CI Clippy Gate):** Examples held to same quality standards as src/
- **Story 3.5.2 (Resize/Refresh):** Examples have event loop infrastructure

**Enables:**
- Better user control for fine-tuning image contrast
- Educational value (demonstrating threshold effect)
- Testing and comparison of Otsu vs manual threshold

**Epic 3.5 Goal:** Polish Epic 3 with UX improvements before Epic 4

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Will be filled during implementation -->

### Debug Log References

<!-- Will be added during implementation -->

### Completion Notes List

<!-- Will be filled after story completion -->

### File List

<!-- Will be populated during implementation -->
