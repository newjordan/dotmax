# Story 3.4: Implement Dithering Algorithms (Floyd-Steinberg, Bayer, Atkinson)

Status: ready-for-dev

## Story

As a **developer rendering images with limited binary output**,
I want **multiple dithering algorithms to optimize visual quality**,
so that **braille images display smooth gradients and detail preservation**.

## Acceptance Criteria

1. **DitheringMethod Enum and API Structure**
   - DitheringMethod enum defined with variants: None, FloydSteinberg, Bayer, Atkinson
   - All dithering functions take `&GrayImage` as input and return `BinaryImage`
   - API supports selecting dithering method at render time
   - Module located at `src/image/dither.rs`
   - Public API exported from `src/image/mod.rs`

2. **Floyd-Steinberg Dithering Implementation**
   - Implements error diffusion algorithm with correct coefficients (7/16, 3/16, 5/16, 1/16)
   - Diffuses quantization error to right and bottom pixels
   - Handles edge cases (image boundaries, no error diffusion outside grid)
   - Produces high-quality output for photographs and complex images
   - Completes in <15ms per tech spec budget for 160×96 image

3. **Bayer Ordered Dithering Implementation**
   - Implements 8×8 Bayer matrix dithering
   - Uses standard Bayer threshold matrix pattern
   - Provides fast, ordered dithering for gradients
   - No error propagation (stateless, parallelizable)
   - Completes in <10ms per tech spec budget for 160×96 image

4. **Atkinson Dithering Implementation**
   - Implements Atkinson error diffusion (Apple-style algorithm)
   - Uses Atkinson coefficients (1/8 diffused to 6 neighbors, 2/8 discarded)
   - Produces softer, more artistic dithering than Floyd-Steinberg
   - Handles edge cases correctly
   - Completes in <12ms per tech spec budget for 160×96 image

5. **Integration with Image Pipeline**
   - Works with `GrayImage` from Story 3.3 grayscale conversion
   - Returns `BinaryImage` compatible with Story 3.5 braille mapper
   - Optional stage: can skip dithering and go directly to thresholding
   - All dithering methods handle same input/output types consistently
   - Feature-gated behind `#[cfg(feature = "image")]`

6. **Error Handling**
   - Zero panics guarantee maintained
   - Invalid dimensions handled gracefully (empty images, 1×1 images)
   - Dithering errors return `DotmaxError::ProcessingError` with context
   - Descriptive error messages for debugging

7. **Testing and Quality Validation**
   - Unit tests for each dithering algorithm with known patterns
   - Visual regression tests comparing dithering outputs
   - Edge case tests (uniform images, all-black, all-white, gradients)
   - Integration test: load → resize → grayscale → dither → verify BinaryImage
   - Test coverage >80% for dither module

8. **Performance Targets**
   - Floyd-Steinberg: <15ms for 160×96 image (most expensive, error diffusion)
   - Bayer: <10ms for 160×96 image (fastest, ordered dithering)
   - Atkinson: <12ms for 160×96 image (moderate, partial error diffusion)
   - Benchmarks created with criterion for all three algorithms
   - Performance validation via `cargo bench --features image`

9. **Documentation and Examples**
   - Rustdoc comments for all public functions with examples
   - Algorithm explanations with references (Floyd-Steinberg paper, Bayer matrix, Atkinson history)
   - Visual comparison example showing all three dithering methods side-by-side
   - Performance characteristics documented (speed vs quality trade-offs)

## Tasks / Subtasks

- [ ] **Task 1: Create module structure and DitheringMethod enum** (AC: 1, 5)
  - [ ] 1.1: Create `src/image/dither.rs` file
  - [ ] 1.2: Add `pub mod dither;` to `src/image/mod.rs`
  - [ ] 1.3: Define `DitheringMethod` enum with variants (None, FloydSteinberg, Bayer, Atkinson)
  - [ ] 1.4: Derive Debug, Clone, Copy, PartialEq, Eq for DitheringMethod
  - [ ] 1.5: Add module-level rustdoc explaining dithering purpose and algorithms
  - [ ] 1.6: Import necessary types (GrayImage, BinaryImage from threshold module)
  - [ ] 1.7: Import DotmaxError from error module
  - [ ] 1.8: Add feature gate `#[cfg(feature = "image")]` to module

- [ ] **Task 2: Implement Floyd-Steinberg dithering** (AC: 2)
  - [ ] 2.1: Implement `floyd_steinberg(gray: &GrayImage) -> Result<BinaryImage, DotmaxError>` signature
  - [ ] 2.2: Create mutable error buffer (Vec<i16> or Vec<f32> for accumulated errors)
  - [ ] 2.3: Iterate pixels left-to-right, top-to-bottom
  - [ ] 2.4: For each pixel: calculate new_value = old_value + accumulated_error
  - [ ] 2.5: Apply threshold (127 or middle gray): new_pixel = if new_value >= 127 { true } else { false }
  - [ ] 2.6: Calculate quantization error: error = new_value - output_value (where output is 0 or 255)
  - [ ] 2.7: Diffuse error to neighbors with Floyd-Steinberg coefficients:
    - Right pixel (x+1, y): 7/16 of error
    - Bottom-left (x-1, y+1): 3/16 of error
    - Bottom (x, y+1): 5/16 of error
    - Bottom-right (x+1, y+1): 1/16 of error
  - [ ] 2.8: Handle boundary conditions (don't diffuse outside image)
  - [ ] 2.9: Return BinaryImage with dithered pixels
  - [ ] 2.10: Add tracing logs: `debug!("Floyd-Steinberg dithering {}×{} image", width, height)`
  - [ ] 2.11: Add rustdoc with algorithm explanation and example
  - [ ] 2.12: Cite original Floyd-Steinberg paper (1976) in documentation

- [ ] **Task 3: Implement Bayer matrix dithering** (AC: 3)
  - [ ] 3.1: Implement `bayer(gray: &GrayImage) -> Result<BinaryImage, DotmaxError>` signature
  - [ ] 3.2: Define 8×8 Bayer threshold matrix as constant (standard Bayer pattern)
  - [ ] 3.3: Bayer matrix values: 0-63 pattern, normalized to 0.0-1.0 range
  - [ ] 3.4: Iterate pixels (any order, stateless)
  - [ ] 3.5: For each pixel at (x, y):
    - Get Bayer threshold: bayer_matrix[x % 8][y % 8]
    - Apply threshold: if (pixel_value / 255.0) > bayer_threshold { true } else { false }
  - [ ] 3.6: Return BinaryImage with dithered pixels
  - [ ] 3.7: Add tracing logs: `debug!("Bayer dithering {}×{} image", width, height)`
  - [ ] 3.8: Add rustdoc explaining Bayer ordered dithering
  - [ ] 3.9: Document that Bayer is fastest (stateless, parallelizable in future)
  - [ ] 3.10: Unit test: verify Bayer pattern applied correctly (check specific pixels)

- [ ] **Task 4: Implement Atkinson dithering** (AC: 4)
  - [ ] 4.1: Implement `atkinson(gray: &GrayImage) -> Result<BinaryImage, DotmaxError>` signature
  - [ ] 4.2: Create error buffer (same as Floyd-Steinberg pattern)
  - [ ] 4.3: Iterate pixels left-to-right, top-to-bottom
  - [ ] 4.4: For each pixel: apply threshold with accumulated error
  - [ ] 4.5: Calculate quantization error
  - [ ] 4.6: Diffuse error to 6 neighbors with Atkinson coefficients (1/8 each):
    - Right pixel (x+1, y): 1/8
    - Two-right pixel (x+2, y): 1/8
    - Bottom-left (x-1, y+1): 1/8
    - Bottom (x, y+1): 1/8
    - Bottom-right (x+1, y+1): 1/8
    - Bottom pixel two rows down (x, y+2): 1/8
    - Note: Only 6/8 of error diffused, 2/8 discarded (Atkinson's signature)
  - [ ] 4.7: Handle boundary conditions
  - [ ] 4.8: Return BinaryImage
  - [ ] 4.9: Add tracing logs: `debug!("Atkinson dithering {}×{} image", width, height)`
  - [ ] 4.10: Add rustdoc explaining Atkinson history (Apple MacPaint algorithm)
  - [ ] 4.11: Document artistic quality (softer than Floyd-Steinberg)

- [ ] **Task 5: Create unified apply_dithering() function** (AC: 1, 5)
  - [ ] 5.1: Implement `apply_dithering(gray: &GrayImage, method: DitheringMethod) -> Result<BinaryImage, DotmaxError>`
  - [ ] 5.2: Match on DitheringMethod enum:
    - DitheringMethod::None → return auto_threshold (from threshold module)
    - DitheringMethod::FloydSteinberg → call floyd_steinberg()
    - DitheringMethod::Bayer → call bayer()
    - DitheringMethod::Atkinson → call atkinson()
  - [ ] 5.3: Return result from selected algorithm
  - [ ] 5.4: Add tracing logs: `debug!("Applying {:?} dithering", method)`
  - [ ] 5.5: Add rustdoc with examples showing how to select dithering method
  - [ ] 5.6: Export apply_dithering and DitheringMethod from src/image/mod.rs

- [ ] **Task 6: Error handling and validation** (AC: 6)
  - [ ] 6.1: Add `ProcessingError` variant to DotmaxError enum (if not exists)
  - [ ] 6.2: Error message format: "Failed to apply {algorithm} dithering: {reason}"
  - [ ] 6.3: Handle empty images (0 width or height) → return error
  - [ ] 6.4: Handle 1×1 images gracefully (edge case, no diffusion needed)
  - [ ] 6.5: Verify zero panics guarantee (no .unwrap() / .expect() in production code)
  - [ ] 6.6: Add unit tests for error paths (empty image, invalid dimensions)
  - [ ] 6.7: Test that all algorithms handle edge cases consistently

- [ ] **Task 7: Unit tests for Floyd-Steinberg** (AC: 7)
  - [ ] 7.1: Create uniform gray (128) test image → verify dithered output pattern
  - [ ] 7.2: Create gradient test image (0 to 255 smooth gradient) → verify smooth transition
  - [ ] 7.3: Create all-black image (0) → verify all output pixels are black (false)
  - [ ] 7.4: Create all-white image (255) → verify all output pixels are white (true)
  - [ ] 7.5: Test edge diffusion: verify error doesn't propagate outside boundaries
  - [ ] 7.6: Test 1×1 image edge case
  - [ ] 7.7: Test very small image (2×2, 3×3) for boundary handling
  - [ ] 7.8: Visual test: verify Floyd-Steinberg produces expected pattern on known image

- [ ] **Task 8: Unit tests for Bayer dithering** (AC: 7)
  - [ ] 8.1: Create uniform gray (128) test → verify Bayer pattern visible
  - [ ] 8.2: Create gradient test → verify smooth transition with ordered pattern
  - [ ] 8.3: Create all-black/all-white tests
  - [ ] 8.4: Test that Bayer threshold matrix is applied correctly (check specific pixels)
  - [ ] 8.5: Verify Bayer output is deterministic (same input → same output)
  - [ ] 8.6: Test edge cases (1×1, small images)
  - [ ] 8.7: Compare Bayer vs Floyd-Steinberg visually (different patterns expected)

- [ ] **Task 9: Unit tests for Atkinson dithering** (AC: 7)
  - [ ] 9.1: Create uniform gray test → verify Atkinson pattern (softer than Floyd-Steinberg)
  - [ ] 9.2: Create gradient test → verify smooth transition
  - [ ] 9.3: Create all-black/all-white tests
  - [ ] 9.4: Test error diffusion to 6 neighbors (verify coefficients applied)
  - [ ] 9.5: Test boundary conditions (errors don't propagate outside grid)
  - [ ] 9.6: Test edge cases (1×1, small images)
  - [ ] 9.7: Visual comparison: Atkinson should be softer than Floyd-Steinberg

- [ ] **Task 10: Integration tests** (AC: 7)
  - [ ] 10.1: Integration test: load image (Story 3.1) → resize (Story 3.2) → grayscale (Story 3.3) → dither → verify BinaryImage
  - [ ] 10.2: Test all three dithering methods produce valid BinaryImage output
  - [ ] 10.3: Test DitheringMethod::None falls back to auto_threshold
  - [ ] 10.4: Test that all algorithms produce same dimensions as input
  - [ ] 10.5: Test chaining: apply_dithering works in full pipeline
  - [ ] 10.6: Error handling integration test: invalid inputs return errors, not panics
  - [ ] 10.7: Cross-platform test: same output on Windows, Linux, macOS (deterministic algorithms)

- [ ] **Task 11: Performance benchmarks** (AC: 8)
  - [ ] 11.1: Benchmark Floyd-Steinberg dithering for 160×96 image
  - [ ] 11.2: Benchmark Bayer dithering for 160×96 image
  - [ ] 11.3: Benchmark Atkinson dithering for 160×96 image
  - [ ] 11.4: Benchmark apply_dithering() with method selection overhead
  - [ ] 11.5: Benchmark larger image (320×192) to test scalability
  - [ ] 11.6: Verify Floyd-Steinberg <15ms, Bayer <10ms, Atkinson <12ms targets met
  - [ ] 11.7: Add benchmarks to `benches/image_conversion.rs` or create new file
  - [ ] 11.8: Document actual performance results in completion notes

- [ ] **Task 12: Visual comparison example** (AC: 9)
  - [ ] 12.1: Create `examples/dither_comparison.rs` example program
  - [ ] 12.2: Load sample image (use existing test fixture or bundled image)
  - [ ] 12.3: Apply all three dithering methods to same image
  - [ ] 12.4: Display side-by-side comparison in terminal (3 columns)
  - [ ] 12.5: Add labels: "Floyd-Steinberg", "Bayer", "Atkinson"
  - [ ] 12.6: Add performance timing output (how long each method took)
  - [ ] 12.7: Add feature gate `#![cfg(feature = "image")]`
  - [ ] 12.8: Test example compiles: `cargo run --example dither_comparison --features image`
  - [ ] 12.9: Verify visual quality differences are apparent

- [ ] **Task 13: Documentation and algorithm references** (AC: 9)
  - [ ] 13.1: Module-level rustdoc explaining dithering purpose and trade-offs
  - [ ] 13.2: Document Floyd-Steinberg algorithm with reference:
    - Floyd, R. W.; Steinberg, L. (1976). "An Adaptive Algorithm for Spatial Grey Scale"
    - Explain error diffusion concept
  - [ ] 13.3: Document Bayer matrix dithering:
    - Explain ordered dithering vs error diffusion
    - Include 8×8 Bayer matrix pattern reference
  - [ ] 13.4: Document Atkinson dithering:
    - History: Bill Atkinson, Apple MacPaint (1984)
    - Explain artistic quality (2/8 error discarded → softer output)
  - [ ] 13.5: Add performance comparison table in docs:
    | Method | Speed | Quality | Best For |
    | Floyd-Steinberg | Slower | Highest | Photos, complex images |
    | Bayer | Fastest | Good | Gradients, simple images |
    | Atkinson | Moderate | Artistic | Line art, artistic renders |
  - [ ] 13.6: Add rustdoc examples for each algorithm showing usage
  - [ ] 13.7: Document when to use None (skip dithering, direct threshold)

- [ ] **Task 14: Export public API** (AC: 1)
  - [ ] 14.1: Export `DitheringMethod` enum from `src/image/mod.rs`
  - [ ] 14.2: Export `apply_dithering()` function from dither module
  - [ ] 14.3: Optionally export individual algorithms (floyd_steinberg, bayer, atkinson) or keep private
  - [ ] 14.4: Verify all exports behind `#[cfg(feature = "image")]`
  - [ ] 14.5: Update module documentation in `src/image/mod.rs` with dithering capabilities
  - [ ] 14.6: Add DitheringMethod to re-exports in lib.rs if needed

- [ ] **Task 15: Validation and cleanup** (AC: All)
  - [ ] 15.1: Run `cargo test --features image` - all tests pass
  - [ ] 15.2: Run `cargo clippy --features image -- -D warnings` - zero warnings
  - [ ] 15.3: Run `cargo fmt` - code formatted
  - [ ] 15.4: Verify zero panics guarantee (no .unwrap() / .expect() in production code)
  - [ ] 15.5: Run benchmarks, verify performance targets met (<15ms Floyd, <10ms Bayer, <12ms Atkinson)
  - [ ] 15.6: Visual check: example program shows all three dithering methods clearly
  - [ ] 15.7: Integration check: pipeline works (load → resize → grayscale → dither)
  - [ ] 15.8: Cross-platform check: CI tests pass on Windows, Linux, macOS

## Dev Notes

### Learnings from Previous Story (Story 3.3 - Grayscale and Otsu Thresholding)

**From Story 3.3 (Grayscale Conversion and Otsu Thresholding) - Status: done, Review: APPROVED**

**Exceptional Quality Standards to Maintain:**

1. **Comprehensive Testing Discipline**: Story 3.3 had 27 unit tests with >80% coverage - aim for similar rigor
   - Known-value testing verified exact calculations
   - Edge cases thoroughly tested (all-black, all-white, uniform)
   - Error paths fully covered
   - Continue this pattern for dithering algorithms

2. **Algorithm Implementation Excellence**:
   - Otsu's method implemented from scratch with proper histogram and variance calculations
   - Algorithm referenced with academic citation (Otsu 1979 paper)
   - Edge cases handled correctly (uniform images, boundaries)
   - Apply same rigor to Floyd-Steinberg, Bayer, Atkinson implementations

3. **Performance Discipline**:
   - Benchmarks created early (Task 12 in previous story)
   - Actual timing validation confirmed targets met
   - Individual operation benchmarks separate from pipeline benchmarks
   - Create dithering benchmarks with same structure

4. **Documentation Quality**:
   - Every public function had rustdoc with examples
   - Algorithms explained with formulas and references
   - Visual examples demonstrated features
   - Maintain this standard for dithering algorithms

**Technical Patterns to Reuse:**

1. **Module Structure**: Story 3.3 separated convert.rs and threshold.rs cleanly
   - This story: dither.rs will follow same pattern
   - Clean separation of concerns, single responsibility

2. **Error Handling**: `InvalidParameter` variant added to DotmaxError with descriptive messages
   - Add `ProcessingError` variant for dithering failures
   - Maintain zero panics guarantee with proper Result returns

3. **Test Helpers**: Story 3.3 created helper functions for test image generation
   - Reuse or create similar helpers for dithering tests
   - Generate gradients, uniform images programmatically

4. **BinaryImage Type**: Defined in Story 3.3 (threshold.rs:71)
   - Input: `&GrayImage` (from convert.rs)
   - Output: `BinaryImage` (Vec<bool> pixels)
   - This story consumes GrayImage and produces BinaryImage (same contract)

**Performance Insights from Story 3.3:**

- Grayscale conversion achieved <2ms target
- Otsu threshold achieved <5ms target
- Total threshold pipeline <10ms
- **Implication**: Dithering must stay within <15ms budget to keep total pipeline <50ms

**Code Quality Metrics from Story 3.3 to Match:**

- ✅ Zero clippy warnings (mandatory)
- ✅ Rustfmt formatted
- ✅ >80% test coverage
- ✅ All doctests compile and pass
- ✅ Example program executes successfully
- ✅ Feature gate compiles independently
- ✅ Zero panics in production code

**Architectural Decisions from Story 3.3:**

- **BT.709 luminance formula**: Specification updated to use modern standard (superior device compatibility)
  - Lesson: Use modern industry standards, not legacy
  - Apply to dithering: Use standard Floyd-Steinberg coefficients, standard 8×8 Bayer matrix

- **Helper functions for complex math**: Otsu used histogram(), variance calculation helpers
  - Apply to dithering: Consider error diffusion helpers, Bayer matrix lookup helpers

- **Const values for clarity**: MIN_BRIGHTNESS, MAX_BRIGHTNESS made parameters clear
  - Apply to dithering: BAYER_MATRIX as const, error diffusion coefficients as consts

**Integration Points from Story 3.3:**

- `to_grayscale()` provides `GrayImage` → input to this story's dithering algorithms
- `BinaryImage` struct defined → output of this story's dithering algorithms
- `auto_threshold()` pipeline → alternative to dithering (DitheringMethod::None fallback)

**Files Created in Story 3.3 (for reference):**

- `src/image/convert.rs` (208 lines) - Provides GrayImage input to dithering
- `src/image/threshold.rs` (741 lines) - Defines BinaryImage type, provides auto_threshold fallback
- `examples/threshold_demo.rs` (169 lines) - Pattern for our dither_comparison.rs example
- `benches/image_conversion.rs` (160 lines) - Add dithering benchmarks here

**Technical Debt to Avoid:**

- ❌ Don't skip visual validation (Story 3.3 had visual example, we need dither comparison)
- ❌ Don't hardcode test patterns - generate programmatically
- ❌ Don't skip boundary condition testing (error diffusion at edges is critical)
- ❌ Don't assume algorithms are correct - validate with known patterns

**Process Lessons:**

- Run benchmarks early to detect performance issues
- Create visual examples to validate quality (not just unit tests)
- Test edge cases systematically (1×1, small images, boundaries)
- Cross-platform CI testing ensures determinism

### Architecture Patterns and Constraints

**Image Pipeline Integration (Tech Spec Section: Workflows and Sequencing)**

- Dithering is Step 5 of the image-to-braille pipeline (optional stage)
- Pipeline flow:
  1. Load (Story 3.1)
  2. Resize (Story 3.2)
  3. Grayscale (Story 3.3)
  4. **Optional: Dither (Story 3.4) OR Skip to Threshold**
  5. Threshold (Story 3.3) - if dithering skipped
  6. Map to Braille (Story 3.5)

- **Input**: `&GrayImage` from Story 3.3 convert module
- **Output**: `BinaryImage` (boolean pixels) → passes to Story 3.5 mapper
- **Alternative Path**: DitheringMethod::None → call auto_threshold() directly

**Performance Budget (Tech Spec: Per-Stage Budget Allocation)**

From tech spec, dithering budget is **<15ms target** (most expensive stage in pipeline):

| Algorithm | Target Time | Justification |
|-----------|-------------|---------------|
| Floyd-Steinberg | <15ms | Error diffusion is most expensive, but highest quality |
| Bayer | <10ms | Ordered dithering, no error propagation (fast) |
| Atkinson | <12ms | Partial error diffusion (6/8), moderate cost |

**Total Pipeline Budget Remaining**:
- Load: <5ms
- Resize: <10ms
- Grayscale: <2ms
- **Dithering: <15ms** (this story)
- Threshold: <5ms (if no dithering, or validation step)
- Braille mapping: <10ms
- **Total: <47ms** (within <50ms target, 3ms margin)

**Dithering Algorithm Details (Tech Spec: APIs and Interfaces)**

1. **Floyd-Steinberg (1976)**:
   - Error diffusion pattern (serpentine or left-to-right)
   - Coefficients: pixel + 7/16 (right) + 3/16 (bottom-left) + 5/16 (bottom) + 1/16 (bottom-right)
   - Quantization error = actual_value - output_value (where output is 0 or 255)
   - Best quality for photographs and complex images

2. **Bayer Ordered Dithering**:
   - 8×8 threshold matrix (standard Bayer pattern)
   - Stateless: each pixel decided independently based on matrix
   - Threshold: `pixel > bayer_matrix[x % 8][y % 8]`
   - Fast, parallelizable, good for gradients

3. **Atkinson Dithering (1984)**:
   - Apple MacPaint algorithm by Bill Atkinson
   - Diffuses 1/8 error to 6 neighbors (right, two-right, bottom-left, bottom, bottom-right, two-down)
   - Only 6/8 error diffused, 2/8 discarded (signature of Atkinson)
   - Produces softer, more artistic results

**Data Types and Contracts (Tech Spec: Data Models)**

```rust
// From Tech Spec
pub enum DitheringMethod {
    None,              // Direct threshold (no dithering)
    FloydSteinberg,    // Error diffusion (best quality, slower)
    Bayer,             // Ordered dithering (fast, good for gradients)
    Atkinson,          // Error diffusion (Apple-style, softer)
}
```

**Error Handling Pattern (ADR 0002)**

- Zero panics guarantee: All public functions return `Result<T, DotmaxError>`
- Add `ProcessingError` variant to `DotmaxError` for dithering failures (if not exists)
- Descriptive errors: "Failed to apply Floyd-Steinberg dithering: image dimensions invalid"
- Handle edge cases gracefully: empty images, 1×1 images, boundary conditions

**Dependency Integration**

- `image::GrayImage`: 8-bit grayscale image input (from Story 3.3 convert module)
- `BinaryImage`: Custom struct from Story 3.3 threshold module
- No external dithering libraries needed - implement from scratch for full control

### Project Structure Alignment

From architecture.md and tech-spec-epic-3.md, Epic 3 structure:

```
src/image/
  ├── mod.rs                    # Public API surface (modify to add dithering exports)
  ├── loader.rs                 # Image loading (Story 3.1) ✅
  ├── resize.rs                 # Resizing (Story 3.2) ✅
  ├── convert.rs                # Grayscale conversion (Story 3.3) ✅
  ├── threshold.rs              # Otsu, binary conversion (Story 3.3) ✅
  ├── dither.rs                 # Dithering algorithms - THIS STORY (NEW)
  ├── mapper.rs                 # Pixels → braille (Story 3.5)
  ├── svg.rs                    # SVG support (Story 3.6)
  └── color_mode.rs             # Color rendering (Story 3.7)
```

**This Story Scope**:
- Create `src/image/dither.rs` for all three dithering algorithms
- Add exports to `src/image/mod.rs` (DitheringMethod enum, apply_dithering function)
- Create `examples/dither_comparison.rs` for visual validation
- Add benchmarks to `benches/image_conversion.rs` (or create new dither-specific bench file)

**Module Responsibilities**:
- `dither.rs`: Convert `GrayImage` → `BinaryImage` using one of three algorithms
- `threshold.rs`: Provides BinaryImage type and auto_threshold fallback (DitheringMethod::None)
- `convert.rs`: Provides GrayImage input to dithering

### Performance Targets (from Tech Spec)

**Stage Budget Breakdown:**

| Operation | Target Time | Budget Allocation | Validation Strategy |
|-----------|-------------|-------------------|---------------------|
| Floyd-Steinberg dithering | <15ms | Error diffusion to 4 neighbors per pixel | Benchmark with criterion on 160×96 image |
| Bayer dithering | <10ms | Ordered dithering, no error propagation | Benchmark with criterion on 160×96 image |
| Atkinson dithering | <12ms | Error diffusion to 6 neighbors, 2/8 discarded | Benchmark with criterion on 160×96 image |
| **Total Story 3.4** | **<15ms** | **Maximum time budget for dithering stage** | **Benchmark all algorithms separately** |

**Memory Efficiency:**

- Dithering operates on GrayImage input: 160×96 pixels × 1 byte = ~15KB
- Produces BinaryImage output: 160×96 pixels × 1 byte (bool) = ~15KB
- Floyd-Steinberg/Atkinson: Error buffer (Vec<f32> or Vec<i16>) = ~61KB (160×96 × 4 bytes)
- Total memory overhead: ~91KB for error diffusion, ~30KB for Bayer (no error buffer)
- Ensure buffers are dropped after use (no leaks)

**Optimization Opportunities (deferred to Epic 7 unless targets missed):**

- Floyd-Steinberg: Serpentine scan (alternate row direction) reduces artifacts
- Bayer: Matrix lookup can be precomputed or inlined
- Atkinson: Sparse error diffusion (only 6 neighbors) may be faster than Floyd-Steinberg
- SIMD: Vectorize Bayer threshold comparison (future optimization)

### Cross-Epic Dependencies

**Depends on Story 3.3 (Grayscale and Otsu Thresholding):**

- `GrayImage` type (input to all dithering algorithms)
- `BinaryImage` struct (output type, defined in threshold.rs)
- `auto_threshold()` function (fallback for DitheringMethod::None)
- Established module structure and testing patterns

**Depends on Story 3.2 (Resize):**

- `DynamicImage` type (indirect dependency, resized images feed grayscale)
- Performance budget allocation pattern (measure each stage separately)

**Depends on Story 3.1 (Image Loading):**

- `load_from_path()` for integration tests and examples
- Feature gate pattern (`#[cfg(feature = "image")]`)

**Enables Story 3.5 (Braille Mapper):**

- `BinaryImage` output from dithering (pixels map to braille dots)
- Dithered binary data provides high-quality input to braille mapping
- Improved visual quality in braille output (gradients, detail preservation)

**Integrates with Epic 2:**

- No direct integration (dithering is preprocessing for braille)
- Eventually feeds `BrailleGrid` via Story 3.5 mapper

### Technical Notes

**Floyd-Steinberg Algorithm Implementation**

Reference: Floyd, R. W.; Steinberg, L. (1976). "An Adaptive Algorithm for Spatial Grey Scale". Proceedings of the Society of Information Display. 17: 75–77.

Pseudocode:
```
function floyd_steinberg(grayscale_image):
    create error_buffer (initialized to 0)
    create binary_output

    for y in 0..height:
        for x in 0..width:
            old_pixel = grayscale_image[x, y]
            new_pixel = old_pixel + error_buffer[x, y]

            output_value = if new_pixel >= 127 { 255 } else { 0 }
            binary_output[x, y] = output_value == 255

            quantization_error = new_pixel - output_value

            # Diffuse error to neighbors
            if x + 1 < width:
                error_buffer[x+1, y] += quantization_error * 7/16
            if y + 1 < height:
                if x - 1 >= 0:
                    error_buffer[x-1, y+1] += quantization_error * 3/16
                error_buffer[x, y+1] += quantization_error * 5/16
                if x + 1 < width:
                    error_buffer[x+1, y+1] += quantization_error * 1/16

    return binary_output
```

**Bayer Matrix (8×8)**

Standard 8×8 Bayer threshold matrix (values 0-63, normalized to 0.0-1.0):

```rust
const BAYER_MATRIX_8X8: [[u8; 8]; 8] = [
    [ 0, 32,  8, 40,  2, 34, 10, 42],
    [48, 16, 56, 24, 50, 18, 58, 26],
    [12, 44,  4, 36, 14, 46,  6, 38],
    [60, 28, 52, 20, 62, 30, 54, 22],
    [ 3, 35, 11, 43,  1, 33,  9, 41],
    [51, 19, 59, 27, 49, 17, 57, 25],
    [15, 47,  7, 39, 13, 45,  5, 37],
    [63, 31, 55, 23, 61, 29, 53, 21],
];

// Normalize to 0.0-1.0 range for threshold comparison
fn bayer_threshold(x: usize, y: usize) -> f32 {
    (BAYER_MATRIX_8X8[y % 8][x % 8] as f32) / 64.0
}
```

**Atkinson Dithering Algorithm**

Bill Atkinson, Apple MacPaint (1984)

Pseudocode:
```
function atkinson(grayscale_image):
    create error_buffer (initialized to 0)
    create binary_output

    for y in 0..height:
        for x in 0..width:
            old_pixel = grayscale_image[x, y]
            new_pixel = old_pixel + error_buffer[x, y]

            output_value = if new_pixel >= 127 { 255 } else { 0 }
            binary_output[x, y] = output_value == 255

            quantization_error = new_pixel - output_value

            # Diffuse error to 6 neighbors (1/8 each), 2/8 discarded
            if x + 1 < width:
                error_buffer[x+1, y] += quantization_error / 8
            if x + 2 < width:
                error_buffer[x+2, y] += quantization_error / 8

            if y + 1 < height:
                if x - 1 >= 0:
                    error_buffer[x-1, y+1] += quantization_error / 8
                error_buffer[x, y+1] += quantization_error / 8
                if x + 1 < width:
                    error_buffer[x+1, y+1] += quantization_error / 8

            if y + 2 < height:
                error_buffer[x, y+2] += quantization_error / 8

    return binary_output
```

**Algorithm Comparison**

| Algorithm | Error Diffusion | Neighbors | Error Preserved | Visual Quality | Speed |
|-----------|-----------------|-----------|-----------------|----------------|-------|
| Floyd-Steinberg | Yes | 4 (right, bottom-left, bottom, bottom-right) | 100% (7+3+5+1)/16 | Highest | Moderate |
| Bayer | No (ordered) | 0 (stateless) | N/A | Good | Fastest |
| Atkinson | Yes | 6 (right, 2-right, bottom-left, bottom, bottom-right, 2-down) | 75% (6×1/8) | Artistic | Moderate |

**Test Strategy:**

- **Unit Tests**: Test each algorithm with known patterns (uniform gray, gradients, all-black/white)
- **Visual Tests**: Generate dithered outputs, compare against expected patterns
- **Edge Cases**: 1×1 images, small images (2×2, 3×3), boundary conditions
- **Integration Tests**: Full pipeline (load → resize → grayscale → dither)
- **Benchmarks**: Validate <15ms Floyd-Steinberg, <10ms Bayer, <12ms Atkinson

**Implementation Tips:**

1. **Error Buffer Type**: Use `Vec<i16>` or `Vec<f32>` for error accumulation
   - i16: -255 to +255 range sufficient, faster than f32
   - f32: More precision, easier math with fractional coefficients
   - Recommendation: Use f32 for clarity, optimize to i16 later if needed

2. **Boundary Handling**: Always check x+1, y+1, x-1 before diffusing error
   - Use `if` guards, not `min()` clamping (clearer intent)
   - Example: `if x + 1 < width { error_buffer[...] += ... }`

3. **Threshold Value**: Use 127 (middle gray) or 128 for binary decision
   - Recommendation: 127 (< 128 is common threshold midpoint)

4. **Bayer Matrix Lookup**: Precompute normalized values or use const array
   - Avoid runtime division (divide by 64.0 once in const)

### References

**Tech Spec Sections:**

- Section: Services and Modules (Table row: src/image/dither.rs - Story 3.4)
- Section: APIs and Interfaces (dither.rs function signatures: floyd_steinberg, bayer, atkinson, apply_dithering)
- Section: Data Models and Contracts (DitheringMethod enum definition)
- Section: Workflows and Sequencing (Step 5: Dithering - optional stage in pipeline)
- Section: Performance/Per-Stage Budget (Dithering: <15ms target from 50ms total pipeline)
- Section: Acceptance Criteria (AC8: Three Dithering Methods Work)

**Architecture Document:**

- Novel Pattern 2: Image-to-Braille Conversion Pipeline [Source: docs/architecture.md#Pattern-2]
- Performance Considerations [Source: docs/architecture.md#Performance-Considerations]
- Error Handling Pattern (ADR 0002) [Source: docs/architecture.md#ADR-0002]
- Zero Panics Guarantee [Source: docs/architecture.md#Error-Handling]

**Epic 3 Tech Spec:**

- Image-to-Braille Pipeline (Monochrome Mode) [Source: docs/sprint-artifacts/tech-spec-epic-3.md#Workflows-and-Sequencing]
- Performance Targets [Source: docs/sprint-artifacts/tech-spec-epic-3.md#Performance]
- Per-Stage Budget Allocation [Source: docs/sprint-artifacts/tech-spec-epic-3.md#Performance, lines 360-368]
- DitheringMethod Data Type [Source: docs/sprint-artifacts/tech-spec-epic-3.md#Data-Models-and-Contracts, lines 108-116]

**Story 3.3 Implementation:**

- BinaryImage struct [Source: docs/sprint-artifacts/3-3-implement-grayscale-conversion-and-otsu-thresholding.md, src/image/threshold.rs:71]
- GrayImage type [Source: docs/sprint-artifacts/3-3-implement-grayscale-conversion-and-otsu-thresholding.md, src/image/convert.rs]
- auto_threshold() function [Source: src/image/threshold.rs:312]
- Performance Benchmarks Pattern [Source: benches/image_conversion.rs]

**External References:**

- Floyd-Steinberg Paper: Floyd, R. W.; Steinberg, L. (1976). "An Adaptive Algorithm for Spatial Grey Scale". Proceedings of the Society of Information Display. 17: 75–77.
- Bayer Ordered Dithering: Bayer, B. E. (1973). "An optimum method for two-level rendition of continuous-tone pictures". IEEE International Conference on Communications. 1: 2611-15.
- Atkinson Dithering: Bill Atkinson, Apple Computer (1984). Algorithm used in MacPaint and HyperCard.
- Wikipedia: Dithering - https://en.wikipedia.org/wiki/Dithering (comprehensive algorithm overview)

## Dev Agent Record

### Context Reference

- docs/sprint-artifacts/3-4-implement-dithering-algorithms-floyd-steinberg-bayer-atkinson.context.xml

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

## Change Log

**2025-11-19 - v1.0 - Story Drafted**

- Created story document for Story 3.4: Implement Dithering Algorithms (Floyd-Steinberg, Bayer, Atkinson)
- Defined 9 acceptance criteria covering all three dithering algorithms
- Created 15 tasks with 123 subtasks for systematic implementation
- Added comprehensive Dev Notes with learnings from Story 3.3
- Documented architecture patterns, performance targets, and algorithm pseudocode
- Included Floyd-Steinberg, Bayer, Atkinson algorithm references and implementation details
- Added references to tech spec, architecture, and external academic papers
- Ready for development with clear implementation path

