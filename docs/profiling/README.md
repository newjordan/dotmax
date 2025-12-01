# Dotmax Performance Profiling

## Profiling Environment

**Hardware/Software:**
- Platform: WSL2 (Windows Subsystem for Linux)
- OS: Linux 6.6.87.2-microsoft-standard-WSL2
- Rust: 1.91.0 (stable)
- Cargo: 1.91.0

**Benchmark Framework:**
- Criterion.rs 0.7 with statistical analysis
- Multiple iterations for statistical significance
- Warm-up runs to eliminate cold-start effects

## Available Profiling Tools

### 1. Criterion Benchmarks (Primary)

Located in `benches/` directory:

| Benchmark File | Coverage | Features |
|----------------|----------|----------|
| `core_rendering.rs` | Grid creation, dot operations, unicode conversion | Core |
| `image_processing.rs` | Image load, resize, dithering, full pipeline | `image` |
| `animation.rs` | Frame timing, buffer swap, 60fps validation | Core |
| `rendering.rs` | Grid clear, unicode grid conversion | Core |
| `dithering.rs` | Floyd-Steinberg, Bayer, Atkinson comparison | `image` |
| `color_schemes.rs` | Color scheme sampling and lookup | Core |
| `density.rs` | Character density rendering | Core |

**Running benchmarks:**
```bash
# All benchmarks
cargo bench --all-features

# Specific benchmark
cargo bench --bench core_rendering

# Image benchmarks only
cargo bench --features image --bench image_processing

# Save baseline for comparison
cargo bench -- --save-baseline before

# Compare against baseline
cargo bench -- --baseline before
```

### 2. Memory Profiling (Linux)

**Valgrind (not available in WSL2 by default):**
```bash
# Memory leak check
valgrind --tool=memcheck cargo run --example view_image --features image

# Heap profiling
valgrind --tool=massif cargo run --example bouncing_ball
ms_print massif.out.*
```

**Heaptrack (alternative):**
```bash
heaptrack cargo run --example view_image --features image
heaptrack_print heaptrack.*.gz
```

### 3. CPU Profiling with Flamegraphs

**Prerequisites (Linux with perf):**
```bash
cargo install flamegraph
```

**Generating flamegraphs:**
```bash
# Image rendering
cargo flamegraph --example view_image --features image -- test_images/photo.jpg

# Animation
cargo flamegraph --example bouncing_ball
```

**Note:** Flamegraph requires `perf` which may not be available in WSL2 environments.

## Benchmark Results Summary (2025-11-25)

### Image Pipeline Performance

| Benchmark | Time | Target | Status |
|-----------|------|--------|--------|
| Full pipeline (800x600 → 80x24) | 7.934 ms | <20ms | ✅ 60% faster |
| With dithering | 7.990 ms | <20ms | ✅ 60% faster |
| Original (25ms target) | 7.9 ms | <25ms | ✅ 68% faster |

### Animation Performance

| Benchmark | Time | Target | Status |
|-----------|------|--------|--------|
| 100 frames at 80x24 | 141 μs | 1.67s (100×16.67ms) | ✅ 11,800x faster |
| Per frame | 1.41 μs | <16.67ms | ✅ 11,800x faster |
| Buffer swap | 2.4 ns | <1ms | ✅ 416,000x faster |

### Core Operations

| Benchmark | Time |
|-----------|------|
| Grid creation (80x24) | 0.174 μs |
| Unicode conversion (80x24) | 1.426 μs |
| Combined ops (clear+redraw+convert) | 1.817 μs |

### Dithering Algorithms (160x96 image)

| Algorithm | Time | Relative |
|-----------|------|----------|
| Bayer | 15.9 μs | 1.0x (fastest) |
| Atkinson | 87.0 μs | 5.5x |
| Floyd-Steinberg | 94.8 μs | 6.0x |

## Profiling Methodology

### Measure-First Optimization (ADR-0007)

1. **No optimization without benchmark proof** - All changes must be justified by profiler data
2. **Use criterion for microbenchmarks** - Statistical analysis ensures reliable measurements
3. **Use flamegraph for hotspot identification** - Visual representation of CPU time distribution
4. **Document before/after measurements** - Track improvement percentages

### Benchmark Best Practices

1. **Avoid microbenchmark pitfalls:**
   - Use `black_box()` to prevent dead code elimination (now `std::hint::black_box()`)
   - Run multiple iterations for statistical significance
   - Account for cache effects with warm-up runs

2. **Test realistic scenarios:**
   - Use actual image sizes (800x600, 1920x1080)
   - Test terminal sizes matching real usage (80x24, 160x48, 200x50)
   - Include full pipeline benchmarks, not just isolated functions

3. **Track regressions:**
   - CI workflow runs benchmarks on every push
   - Criterion's comparison features detect >10% regressions
   - Baseline comparisons before/after changes

## CI Integration

Benchmarks run automatically via `.github/workflows/benchmark.yml`:
- Triggered on push to main and PRs
- Stores results as artifacts for historical comparison
- Detects regressions >10% and comments on PRs
