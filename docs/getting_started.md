# Getting Started with Dotmax

This guide will walk you through installing dotmax and creating your first terminal graphics application.

## Installation

Add dotmax to your `Cargo.toml`:

```toml
[dependencies]
dotmax = "0.1"
```

For image rendering, enable the `image` feature:

```toml
[dependencies]
dotmax = { version = "0.1", features = ["image"] }
```

For SVG support, enable the `svg` feature:

```toml
[dependencies]
dotmax = { version = "0.1", features = ["image", "svg"] }
```

## Your First Braille Grid (5 minutes)

Create a simple braille rendering in just a few lines:

```rust
use dotmax::{BrailleGrid, TerminalRenderer};

fn main() -> Result<(), dotmax::DotmaxError> {
    // Create a 20x10 grid (40 dots × 40 dots)
    let mut grid = BrailleGrid::new(20, 10)?;

    // Draw a diagonal line
    for i in 0..40 {
        grid.set_dot(i, i)?;
    }

    // Render to terminal
    let mut renderer = TerminalRenderer::new()?;
    renderer.render(&grid)?;

    Ok(())
}
```

Save as `src/main.rs` and run with `cargo run`.

## Understanding the Grid

Dotmax uses Unicode braille characters to render graphics. Each terminal character cell contains a 2×4 matrix of dots:

```
┌─────────┐
│ • •     │  Braille character cell
│ • •     │  2 dots wide × 4 dots tall
│ • •     │  = 8 possible dots per cell
│ • •     │
└─────────┘
```

When you create a `BrailleGrid::new(width, height)`:
- `width` = terminal columns (cells)
- `height` = terminal rows (cells)
- Dot resolution = `width * 2` × `height * 4`

Example: A 20×10 grid gives you 40×40 dot resolution!

## Drawing Basics

### Setting Individual Dots

```rust
// Set a single dot at (x=10, y=20)
grid.set_dot(10, 20)?;

// Clear a dot
grid.clear_dot(10, 20)?;
```

### Drawing Lines

```rust
use dotmax::primitives::draw_line;

// Draw from (0, 0) to (100, 50)
draw_line(&mut grid, 0, 0, 100, 50)?;
```

### Drawing Shapes

```rust
use dotmax::primitives::{draw_circle, draw_rectangle};

// Circle at center (80, 48) with radius 30
draw_circle(&mut grid, 80, 48, 30)?;

// Rectangle from (10, 10) to (60, 40)
draw_rectangle(&mut grid, 10, 10, 50, 30)?;
```

## Adding Color

Enable color support on your grid:

```rust
use dotmax::{BrailleGrid, Color};
use dotmax::primitives::draw_circle_colored;

let mut grid = BrailleGrid::new(80, 24)?;
grid.enable_color_support();

// Draw a red circle
draw_circle_colored(&mut grid, 80, 48, 30, Color::new(255, 0, 0))?;
```

## Simple Animation

Create smooth animations with `AnimationLoop`:

```rust
use dotmax::animation::AnimationLoop;

AnimationLoop::new(80, 24)
    .fps(30)
    .on_frame(|frame, buffer| {
        // frame: current frame number (0, 1, 2, ...)
        // buffer: the grid to draw on (auto-cleared each frame)

        let x = (frame * 2) % 160;  // Move right
        let y = 48;                  // Centered vertically

        buffer.set_dot(x as usize, y)?;

        Ok(true)  // true = continue, false = stop
    })
    .run()
```

Press `q` or `Ctrl+C` to exit.

## Rendering Images

Enable the `image` feature, then:

```rust
use dotmax::image::{load_from_path, render_to_grid};
use dotmax::BrailleGrid;
use std::path::Path;

// Load and auto-resize to fit terminal
let img = load_from_path(Path::new("photo.png"))?;
let grid = render_to_grid(&img, 80, 24)?;

// Render
let mut renderer = TerminalRenderer::new()?;
renderer.render(&grid)?;
```

## Color Schemes

Apply predefined color schemes:

```rust
use dotmax::{get_scheme, apply_color_scheme};

// Get a predefined scheme
let heat_map = get_scheme("heat_map").unwrap();

// Apply to intensity data
let colors = apply_color_scheme(&intensities, &heat_map);
```

Available schemes: `rainbow`, `heat_map`, `blue_purple`, `green_yellow`, `cyan_magenta`, `grayscale`, `monochrome`

## Error Handling

Dotmax uses a unified `DotmaxError` type:

```rust
use dotmax::DotmaxError;

fn main() -> Result<(), DotmaxError> {
    // All dotmax operations return Result<_, DotmaxError>
    let grid = BrailleGrid::new(80, 24)?;
    // ...
}
```

Common error cases:
- Invalid dimensions (zero width/height)
- Dot coordinates out of bounds
- Image loading failures
- Terminal I/O errors

## Next Steps

- **[Examples](../examples/README.md)** - Browse 49 working examples
- **[Animation Guide](animation_guide.md)** - Deep dive into animation
- **[Performance Guide](performance.md)** - Optimization tips
- **[API Reference](https://docs.rs/dotmax)** - Full API documentation

## Quick Reference

### Coordinate System

```
(0,0)────────────────────────► X (dots)
  │
  │    • dot at (x, y)
  │
  ▼
  Y (dots)
```

### Common Grid Sizes

| Terminal | Grid | Dot Resolution |
|----------|------|----------------|
| 80×24 (standard) | `new(80, 24)` | 160×96 dots |
| 120×40 (large) | `new(120, 40)` | 240×160 dots |
| Full terminal | Auto-detect | Varies |

### Feature Flags

| Feature | Description |
|---------|-------------|
| (default) | Core braille rendering |
| `image` | PNG/JPG loading, dithering |
| `svg` | SVG file rendering |

## Troubleshooting

### Nothing displays
- Ensure your terminal supports Unicode (UTF-8)
- Try running in a different terminal (Windows Terminal, iTerm2, etc.)

### Colors not working
- Check terminal color support with `echo $COLORTERM`
- Dotmax auto-detects and adapts to your terminal

### Animation flickering
- Use `AnimationLoop` instead of manual rendering
- Enable double-buffering for custom loops

See [Troubleshooting Guide](troubleshooting.md) for more solutions.
