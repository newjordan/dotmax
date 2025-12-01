# Performance Bottleneck Analysis

**Analysis Date:** 2025-11-25
**Benchmark Framework:** Criterion.rs 0.7
**Platform:** WSL2 Linux / Rust 1.91.0

## Executive Summary

**All performance targets are already exceeded.** The codebase demonstrates exceptional performance:

| Target | Requirement | Actual | Margin |
|--------|-------------|--------|--------|
| Image render (80x24) | <20ms | 7.9ms | **60% faster** |
| Frame time (60fps) | <16.67ms | 1.4μs | **11,800x faster** |
| Buffer swap | <1ms | 2.4ns | **416,000x faster** |

Given this exceptional performance, optimization work focuses on **documentation and validation** rather than aggressive code changes.

## Full Pipeline Analysis (80x24 Terminal)

### Image Processing Pipeline: 7.934 ms total

| Stage | Time | % of Pipeline | Notes |
|-------|------|---------------|-------|
| Image Load (PNG) | 4.552 ms | 57% | External dependency (image crate) |
| Image Resize (800x600→160x96) | ~2.0 ms | 25% | Lanczos3 filtering |
| Grayscale Conversion | ~0.1 ms | 1% | Per-pixel operation |
| Dithering (Floyd-Steinberg) | 0.095 ms | 1% | Error diffusion |
| Otsu Threshold | 0.048 ms | <1% | Statistical analysis |
| Braille Mapping | 0.023 ms | <1% | 2x4 blocks → unicode |
| Grid Operations | 0.002 ms | <1% | Buffer management |

**Key Finding:** 82% of pipeline time is in **external dependencies** (image loading + resizing), not dotmax code.

### Animation Pipeline: 1.41 μs/frame

| Stage | Time | Notes |
|-------|------|-------|
| Buffer Swap | 2.4 ns | O(1) std::mem::swap |
| Grid Clear | ~0.2 μs | .fill(0) reuses allocation |
| Unicode Conversion | 1.4 μs | Per-cell conversion |
| **Total** | **1.4 μs** | **11,800x under 16.67ms target** |

## Identified Bottlenecks (Ranked by Time)

### Bottleneck #1: Image Loading (57% of pipeline)

**Location:** External `image` crate
**Time:** 4.552 ms for PNG sample
**Root Cause:** PNG decompression and decoding is inherently CPU-intensive

**Analysis:**
- This is in external dependency code, not dotmax
- PNG decoding involves zlib decompression
- Already optimized by the `image` crate maintainers
- No actionable optimization within dotmax scope

**Recommendation:** Document as external dependency constraint. Users can:
- Use smaller source images
- Pre-load images if rendering multiple frames
- Use JPEG for faster loading (less compression overhead)

### Bottleneck #2: Image Resizing (25% of pipeline)

**Location:** `src/image/resize.rs` (calls `imageproc` crate)
**Time:** ~2.0 ms for 800x600→160x96
**Root Cause:** Lanczos3 filtering requires weighted averaging of multiple source pixels

**Analysis:**
- Uses adaptive filter selection (already implemented in Epic 3.5)
- Lanczos3 for normal images, Triangle for extreme aspect ratios
- External `imageproc` crate handles the actual resize

**Current Optimization:**
```rust
// Already implemented adaptive filter selection
fn select_resize_filter(image_info: &ImageInfo) -> FilterType {
    if image_info.is_extreme_aspect_ratio() {
        FilterType::Triangle  // Faster for unusual shapes
    } else {
        FilterType::Lanczos3  // High quality default
    }
}
```

**Recommendation:** No further optimization needed. Already beats target by 60%.

### Bottleneck #3: Dithering Algorithms (~1% of pipeline)

**Location:** `src/image/dither.rs`
**Time:**
- Bayer: 15.9 μs (fastest)
- Atkinson: 87.0 μs
- Floyd-Steinberg: 94.8 μs

**Analysis:**
- Floyd-Steinberg requires error diffusion to 4 neighbors per pixel
- Bayer is stateless and parallelizable (6x faster)
- All algorithms are sub-millisecond already

**Recommendation:** Document performance characteristics. Users can choose:
- Bayer for real-time/animation (15.9 μs)
- Floyd-Steinberg for quality stills (94.8 μs)

## Optimization Opportunities NOT Pursued

These were identified in the story spec but are **not necessary** given current performance:

### 1. Unicode Conversion Lookup Table

**Spec Suggestion:** Pre-computed 256-entry lookup table for `dots_to_char()`
**Current:** `char::from_u32(0x2800 + dots as u32).unwrap_or(' ')`
**Time:** 1.4 μs for entire 80x24 grid (~0.7ns per cell)

**Decision:** Not implemented. Current performance is 11,800x under target. The optimization would save ~0.5μs total per frame, which is negligible.

### 2. SIMD for Dithering

**Spec Suggestion:** SIMD instructions for pixel operations
**Current:** Scalar per-pixel processing
**Time:** 95 μs for Floyd-Steinberg on 160x96

**Decision:** Not implemented. Already 99% under target. SIMD complexity not justified for sub-millisecond operations.

### 3. Parallel Processing with Rayon

**Spec Suggestion:** Parallel scanlines for image processing
**Current:** Sequential processing
**Time:** 7.9 ms total pipeline

**Decision:** Not implemented. Would add dependency and complexity. Already 60% under target.

## Memory Analysis

### Baseline Memory

Grid memory usage is deterministic:
- 80x24 terminal: `80 * 24 * 8 bytes = 15,360 bytes = 15 KB`
- 200x50 terminal: `200 * 50 * 8 bytes = 80,000 bytes = 80 KB`

**Conclusion:** Far under 5MB baseline target.

### Per-Frame Overhead

Buffer operations:
- `grid.clear()` uses `self.dots.fill(0)` - **no allocation**
- `swap_buffers()` uses `std::mem::swap` - **no allocation**
- Pre-allocated front/back buffers - **no per-frame allocation**

**Conclusion:** Per-frame overhead is ~0 KB (pointer swap only), far under 500KB target.

## Conclusion

**No code optimizations are necessary.** All targets are exceeded by significant margins:

1. Image pipeline: 60% faster than target
2. Animation: 11,800x faster than target
3. Memory: Far under limits

The existing architecture (measure-first, buffer reuse, adaptive algorithms) has produced exceptional performance without needing additional optimization passes.

## Recommendations

1. **Document performance characteristics** in README and API docs
2. **Create ADR** explaining decision not to optimize (targets already met)
3. **Validate targets** with formal benchmark runs
4. **Monitor regressions** via CI benchmark workflow
