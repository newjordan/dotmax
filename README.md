# dotmax

High-performance terminal braille rendering for Rust.

[![Crates.io](https://img.shields.io/crates/v/dotmax.svg)](https://crates.io/crates/dotmax)
[![Documentation](https://docs.rs/dotmax/badge.svg)](https://docs.rs/dotmax)
[![License](https://img.shields.io/crates/l/dotmax.svg)](https://github.com/newjordan/dotmax#license)

## Quick Start

```bash
cargo add dotmax --features image
```

```rust
use dotmax::quick;

fn main() -> Result<(), dotmax::DotmaxError> {
    quick::show_file("photo.png")?;  // Any image, SVG, or animated GIF
    Ok(())
}
```

## Features

| Feature | Description |
|---------|-------------|
| `default` | Core braille rendering (shapes, animations) |
| `image` | PNG, JPG, GIF (static + animated), BMP, WebP, TIFF |
| `svg` | SVG vector graphics |

```toml
[dependencies]
dotmax = { version = "0.1", features = ["image", "svg"] }
```

## Usage

### Prelude (import everything)

```rust
use dotmax::prelude::*;
```

Includes: `BrailleGrid`, `TerminalRenderer`, `Color`, drawing primitives, animation types, color schemes, and quick functions.

### Quick API

| Function | Description |
|----------|-------------|
| `quick::show_file(path)` | Display any media (auto-detects format, plays animations) |
| `quick::show_image(path)` | Display static image |
| `quick::load_file(path)` | Load media into `MediaContent` for manual control |
| `quick::grid()` | Create terminal-sized grid |
| `quick::show(&grid)` | Display grid, wait for keypress |

### One-liner display

```rust
quick::show_file("animation.gif")?;  // Auto-detects format, plays animations
quick::show_image("photo.jpg")?;     // Static image display
```

### Drawing primitives

```rust
use dotmax::prelude::*;

let mut grid = grid()?;
draw_circle(&mut grid, 80, 48, 30)?;
draw_line(&mut grid, 0, 0, 160, 96)?;
show(&grid)?;
```

### Animation

```rust
use dotmax::animation::AnimationLoop;

AnimationLoop::new(80, 24)
    .fps(60)
    .on_frame(|frame, buffer| {
        buffer.set_dot((frame * 2) % 160, 48)?;
        Ok(true)
    })
    .run()?;
```

## Performance

| Operation | Time |
|-----------|------|
| Frame render (80×24) | ~2μs |
| Image pipeline | ~10ms |
| Animation frame budget | 1.6μs (10,000× faster than 60fps requires) |

## Examples

```bash
cargo run --example bouncing_ball
cargo run --example load_image --features image
cargo run --example animated_gif --features image
```

## Documentation

- [API Reference](https://docs.rs/dotmax)
- [Animation Guide](docs/animation_guide.md)

## License

MIT OR Apache-2.0
