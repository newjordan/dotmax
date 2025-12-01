# Dotmax Performance Guide

This guide covers performance characteristics, optimization techniques, and benchmarking results for dotmax.

## Performance Targets

Dotmax is designed to meet these targets on typical hardware:

| Operation | Target | Actual |
|-----------|--------|--------|
| Grid creation (80×24) | <1ms | ~50μs |
| Frame render | <16.67ms (60fps) | ~2-5ms |
| Buffer swap | <1ms | ~2.4ns |
| Image pipeline (1024×1024) | <25ms | ~7.9ms |
| Differential render (10% change) | <2ms | <1ms |

## Core Performance Characteristics

### Grid Operations

```rust
// O(1) - Constant time operations
grid.set_dot(x, y)?;     // ~10ns
grid.clear_dot(x, y)?;   // ~10ns
grid.get_dot(x, y);      // ~5ns

// O(n) - Linear in grid size
grid.clear();            // ~1μs for 80×24
grid.to_string();        // ~10μs for 80×24
```

### Memory Usage

| Grid Size | Memory | Dots |
|-----------|--------|------|
| 80×24 | ~2KB | 160×96 |
| 120×40 | ~5KB | 240×160 |
| 240×80 | ~19KB | 480×320 |

Formula: `size ≈ width × height × (1 byte + color overhead if enabled)`

### Color Support Overhead

When `enable_color_support()` is called:
- Memory: +3 bytes per cell (RGB)
- Render time: +20-30% for terminal escape codes

Only enable color when needed.

## Optimization Techniques

### 1. Choose the Right FPS

```rust
// UI elements: 10-15 FPS is sufficient
AnimationLoop::new(80, 24).fps(15)

// Smooth motion: 30-60 FPS
AnimationLoop::new(80, 24).fps(60)

// Real-time clock: 1 FPS saves CPU
AnimationLoop::new(80, 24).fps(1)
```

### 2. Use Differential Rendering

When most of the screen is static, only update changed cells:

```rust
use dotmax::animation::DifferentialRenderer;

let mut diff = DifferentialRenderer::new(80, 24);

// Initial full render
diff.full_render(&grid, &mut renderer)?;

// Only render differences
let changes = diff.diff_render(&new_grid, &mut renderer)?;
// Typical: 90%+ fewer terminal writes
```

**When to use:**
- UI with static elements
- Games with fixed backgrounds
- Visualizations with updating regions

**When NOT to use:**
- Full-screen animations
- Particle systems covering entire screen

### 3. Pre-render Repeating Animations

```rust
use dotmax::animation::PrerenderedAnimation;

// Pre-compute all frames once
let frames: Vec<BrailleGrid> = (0..60)
    .map(|i| generate_frame(i))
    .collect();

let anim = PrerenderedAnimation::new(frames);

// Playback is just memory access
loop {
    let frame = anim.get_current_frame();
    renderer.render(frame)?;
    anim.advance();
}
```

Memory cost: ~2KB per frame (80×24 grid)

### 4. Minimize Drawing Operations

```rust
// Bad: Check every pixel
for y in 0..96 {
    for x in 0..160 {
        if should_draw(x, y) {
            grid.set_dot(x, y)?;
        }
    }
}

// Good: Only draw what you need
let points = calculate_points();
for (x, y) in points {
    grid.set_dot(x, y)?;
}
```

### 5. Use Appropriate Image Filters

For image rendering, filter choice affects speed:

| Filter | Speed | Quality |
|--------|-------|---------|
| Nearest | Fastest | Lowest |
| Triangle | Fast | Low |
| CatmullRom | Medium | Good |
| Lanczos3 | Slowest | Best |

Dotmax auto-selects `CatmullRom` for extreme aspect ratios (>10:1).

### 6. Batch Terminal Operations

```rust
// Bad: Many small renders
for shape in shapes {
    draw_shape(&mut grid, shape)?;
    renderer.render(&grid)?;  // Expensive!
}

// Good: Single render after all drawing
for shape in shapes {
    draw_shape(&mut grid, shape)?;
}
renderer.render(&grid)?;  // Once
```

## Profiling Your Application

### Enable Tracing

```rust
// In your main()
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

Then run with:
```bash
RUST_LOG=dotmax=debug cargo run --release
```

### Using Criterion Benchmarks

Run the included benchmarks:

```bash
# All benchmarks
cargo bench

# Specific benchmark
cargo bench --bench rendering

# With HTML report
cargo bench -- --save-baseline main
```

Benchmark categories:
- `rendering` - Core grid operations
- `animation` - Frame timing and buffers
- `image_processing` - Image pipeline (requires `image` feature)
- `color_conversion` - ANSI/RGB conversion
- `core_rendering` - Full rendering pipeline

### Flame Graphs

For detailed profiling:

```bash
cargo install flamegraph
cargo flamegraph --example simple_animation
```

## Performance Bottlenecks

### Common Issues

1. **Terminal I/O** (most common)
   - Solution: Reduce render frequency
   - Solution: Use differential rendering

2. **Image Resizing**
   - Solution: Cache resized images
   - Solution: Use faster filter for previews

3. **Color Escape Codes**
   - Solution: Disable color when not needed
   - Solution: Batch color changes

4. **Grid Allocation**
   - Solution: Reuse grids instead of creating new ones
   - Solution: Use `clear()` instead of `new()`

### Memory Profiling

Check memory usage with:

```bash
cargo build --release
valgrind --tool=massif ./target/release/your_app
ms_print massif.out.*
```

## Benchmark Results

Representative benchmarks on typical hardware (M1 MacBook):

```
Grid Operations:
  set_dot (single)      10.2 ns
  set_dot (100 dots)    1.02 μs
  clear (80x24)         0.89 μs
  to_string (80x24)     8.4 μs

Animation:
  buffer_swap           2.4 ns
  frame_render (80x24)  2.1 ms
  diff_render (10%)     0.3 ms

Image Pipeline:
  load_png (1024x1024)  4.2 ms
  resize (to 160x96)    1.8 ms
  to_braille            1.9 ms
  total_pipeline        7.9 ms

Color:
  rgb_to_ansi256        15 ns
  scheme_sample         8 ns
  apply_scheme (1000)   12 μs
```

## Best Practices Summary

1. **Start simple** - Profile before optimizing
2. **Match FPS to content** - Don't animate faster than needed
3. **Use differential rendering** - For mostly-static screens
4. **Pre-render loops** - For repeating animations
5. **Batch operations** - Minimize render calls
6. **Enable color only when needed** - It adds overhead
7. **Test on target hardware** - Performance varies by terminal

## Further Reading

- [Animation Guide](animation_guide.md) - Animation-specific optimizations
- [Troubleshooting](troubleshooting.md) - Debugging performance issues
- [API Reference](https://docs.rs/dotmax) - Detailed API documentation
