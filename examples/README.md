# Dotmax Examples

This directory contains 49 examples demonstrating dotmax's features. All examples can be run with `cargo run --example <name>`.

## Quick Start

### Minimal Examples (No Features Required)

| Example | Description | Lines |
|---------|-------------|-------|
| `hello_braille` | Basic braille grid rendering | ~65 |
| `simple_animation` | Bouncing dot animation with `AnimationLoop` | ~30 |
| `shapes_demo` | Drawing rectangles, polygons, triangles | ~160 |
| `color_schemes_demo` | All predefined color schemes | ~90 |

```bash
cargo run --example hello_braille
cargo run --example simple_animation
```

## Example Categories

### Core Rendering
- **`hello_braille.rs`** - Basic braille grid and dots
- **`braille_mapping_demo.rs`** - Unicode braille character mapping
- **`terminal_debug.rs`** - Debug terminal detection and viewport
- **`logging_demo.rs`** - Enable tracing/logging

### Image Rendering (`--features image`)

| Example | Description |
|---------|-------------|
| `load_image` | Load images from files and byte buffers |
| `view_image` | Interactive image viewer with resize support |
| `simple_image` | Minimal image rendering |
| `color_image` | Colored image rendering with schemes |
| `resize_image` | Image resizing with different filters |
| `dither_comparison` | Compare dithering algorithms |
| `threshold_demo` | Otsu vs fixed threshold |
| `image_browser` | Full-featured image browser |

```bash
cargo run --example load_image --features image
cargo run --example view_image --features image -- path/to/image.png
cargo run --example dither_comparison --features image
```

### SVG Rendering (`--features svg`)

- **`svg_demo.rs`** - Load and render SVG files
- **`svg_font_quality.rs`** - SVG text rendering examples
- **`save_svg_raster.rs`** - Export SVG to raster image

```bash
cargo run --example svg_demo --features svg
```

### Animation

| Example | Description | Location |
|---------|-------------|----------|
| `simple_animation` | Basic bouncing dot | `examples/` |
| `animation_buffer` | Double-buffering | `examples/` |
| `fps_control` | Frame timing | `examples/` |
| `prerendered_demo` | Cached sequences | `examples/` |
| `differential_demo` | Optimized rendering | `examples/` |

**Animation Gallery** (`examples/animations/`):
- **`bouncing_ball.rs`** - Physics simulation with gravity
- **`loading_spinner.rs`** - Multiple spinner styles
- **`waveform.rs`** - Sine wave visualization
- **`fireworks.rs`** - Particle system demo
- **`clock.rs`** - Real-time analog clock

```bash
cargo run --example bouncing_ball
cargo run --example fireworks
cargo run --example clock
```

### Drawing Primitives

- **`lines_demo.rs`** - Bresenham line drawing
- **`circles_demo.rs`** - Circle algorithm
- **`circles_demo_simple.rs`** - Simplified circles
- **`shapes_demo.rs`** - Rectangles, polygons, filled shapes
- **`shapes_demo_simple.rs`** - Simplified shapes
- **`colored_shapes.rs`** - Colored primitives
- **`density_demo.rs`** - Character density rendering

```bash
cargo run --example shapes_demo
cargo run --example circles_demo
cargo run --example lines_demo
```

### Color System

- **`color_demo.rs`** - Basic color support
- **`color_schemes_demo.rs`** - All 6 predefined schemes
- **`custom_scheme.rs`** - Create custom color schemes
- **`color_detection.rs`** - Terminal color capability detection
- **`color_conversion_demo.rs`** - RGB to ANSI conversion
- **`heatmap.rs`** - Heatmap visualization

```bash
cargo run --example color_schemes_demo
cargo run --example custom_scheme
cargo run --example heatmap
```

## Feature Requirements

| Feature | Examples | Enable With |
|---------|----------|-------------|
| Core | `hello_braille`, `simple_animation`, `shapes_demo` | (default) |
| Image | `load_image`, `view_image`, `dither_comparison` | `--features image` |
| SVG | `svg_demo`, `svg_font_quality` | `--features svg` |
| All | `image_browser`, `color_image` | `--all-features` |

## Running Examples

```bash
# Core (no features needed)
cargo run --example hello_braille
cargo run --example simple_animation
cargo run --example shapes_demo

# Image rendering
cargo run --example load_image --features image
cargo run --example view_image --features image -- image.png

# SVG rendering
cargo run --example svg_demo --features svg

# All features
cargo run --example image_browser --all-features
```

## Interactive Controls

Most interactive examples support:
- **`q`** or **`Ctrl+C`** - Exit
- **Terminal resize** - Auto re-render
- **Arrow keys** - Navigate (where applicable)

## Debugging

Enable debug logging to see internal operations:

```bash
RUST_LOG=dotmax=debug cargo run --example hello_braille
```

## Adding New Examples

Examples should follow these guidelines:

1. **Minimal** - Focus on demonstrating one feature
2. **Documented** - Include `//!` doc comments at the top
3. **Runnable** - Must compile with `cargo clippy --examples`
4. **Practical** - Show real-world usage patterns

## Test Fixtures

Image examples use fixtures in `tests/fixtures/images/`:
- `sample.png` - Standard test image
- `large.jpg` - High-resolution test
- `wide.png` - Extreme aspect ratio
- `tall.png` - Extreme aspect ratio

## See Also

- [Animation Guide](../docs/animation_guide.md) - Comprehensive animation documentation
- [Getting Started](../docs/getting_started.md) - Tutorial walkthrough
- [API Reference](https://docs.rs/dotmax) - Full API documentation
