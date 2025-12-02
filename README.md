# dotmax

High-performance terminal braille rendering for Rust

[![Crates.io](https://img.shields.io/crates/v/dotmax.svg)](https://crates.io/crates/dotmax)
[![Documentation](https://docs.rs/dotmax/badge.svg)](https://docs.rs/dotmax)
[![License](https://img.shields.io/crates/l/dotmax.svg)](https://github.com/frosty40/dotmax#license)

## Features

- 🎨 **4× Resolution Advantage** - Braille 2×4 dot matrix provides superior detail over ASCII art
- ⚡ **Blazing Fast** - <50ms image rendering, 60-120fps animation
- 🌍 **Universal Compatibility** - Works in any Unicode-capable terminal
- 🦀 **Zero-Cost Abstractions** - Memory-safe Rust with minimal overhead
- 🎭 **Rich Graphics** - Images, shapes, colors, and animations in your terminal

## Installation

Add dotmax to your Cargo project:

```bash
cargo add dotmax
```

Or add to `Cargo.toml`:

```toml
[dependencies]
dotmax = "0.1"
```

## Quick Start

### One-Liner Image Display

```rust
use dotmax::quick;

fn main() -> Result<(), dotmax::DotmaxError> {
    // Display any image in your terminal - that's it!
    quick::show_image("photo.png")?;
    Ok(())
}
```

### Using the Prelude

```rust
use dotmax::prelude::*;

fn main() -> Result<(), DotmaxError> {
    // Create a terminal-sized grid
    let mut grid = grid()?;

    // Draw shapes using primitives
    draw_circle(&mut grid, 80, 48, 30)?;
    draw_line(&mut grid, 0, 0, 160, 96)?;

    // Display and wait for keypress
    show(&grid)?;
    Ok(())
}
```

### Manual Control

```rust
use dotmax::{BrailleGrid, TerminalRenderer};

fn main() -> Result<(), dotmax::DotmaxError> {
    // Create a 20x10 braille grid (40 dots wide × 40 dots tall)
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

See [examples/](examples/) for more usage patterns including images, animations, and color schemes.

## Visual Demo

<!-- TODO: Add terminal screenshots/GIFs when published -->
```
┌────────────────────────────────────────────────────────────┐
│  ⡀⢀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀    Braille rendering provides    │
│  ⠀⠀⢀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀    4x the resolution of ASCII    │
│  ⠀⠀⠀⠀⢀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀    art. Each character cell      │
│  ⠀⠀⠀⠀⠀⠀⢀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀    contains a 2×4 dot matrix.   │
│  ⠀⠀⠀⠀⠀⠀⠀⠀⢀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀                                     │
└────────────────────────────────────────────────────────────┘
```

## Feature Flags

| Flag | Description | Dependencies |
|------|-------------|--------------|
| `default` | Core braille rendering only | ratatui, crossterm, thiserror |
| `image` | PNG, JPG, GIF, BMP, WebP, TIFF support | image, imageproc |
| `svg` | SVG vector graphics rendering | resvg, usvg |
| `video` | Video playback (Phase 2) | ffmpeg (future) |
| `raytrace` | 3D raytracing (Phase 3) | TBD (future) |

Enable features in your `Cargo.toml`:

```toml
[dependencies]
dotmax = { version = "0.1", features = ["image", "svg"] }
```

## Animation

Dotmax provides a comprehensive animation system for smooth, flicker-free terminal animations:

```rust
use dotmax::animation::AnimationLoop;

fn main() -> Result<(), dotmax::DotmaxError> {
    AnimationLoop::new(80, 24)
        .fps(60)
        .on_frame(|frame, buffer| {
            // Calculate bouncing position
            let x = (frame as usize * 2) % 160;
            let y = 48;
            buffer.set_dot(x, y)?;
            Ok(true) // Continue animation
        })
        .run()
}
```

### Animation Components

| Component | Purpose |
|-----------|---------|
| `AnimationLoop` | High-level animation abstraction (recommended) |
| `FrameTimer` | Consistent frame rate control |
| `FrameBuffer` | Double-buffering for flicker-free updates |
| `PrerenderedAnimation` | Cached frame sequences for looping |
| `DifferentialRenderer` | Only render changed cells (90%+ savings) |

### Animation Examples

```bash
cargo run --example bouncing_ball    # Physics simulation
cargo run --example loading_spinner  # Rotating indicators
cargo run --example waveform         # Sine wave visualization
cargo run --example fireworks        # Particle system
cargo run --example clock            # Real-time analog clock
```

For comprehensive documentation, see [docs/animation_guide.md](docs/animation_guide.md).

## Prelude & Quick API

Dotmax provides two convenience modules for rapid development:

### Prelude Module

Import everything you need with one line:

```rust
use dotmax::prelude::*;
```

Includes: `BrailleGrid`, `TerminalRenderer`, `Color`, drawing primitives (`draw_line`, `draw_circle`, etc.), animation types, color schemes, and quick functions.

### Quick Module

One-liner functions for common tasks:

| Function | Description |
|----------|-------------|
| `quick::grid()` | Create terminal-sized grid |
| `quick::grid_sized(w, h)` | Create grid with explicit size |
| `quick::show(&grid)` | Display grid, wait for keypress |
| `quick::show_image(path)` | Load and display image (one line!) |
| `quick::load_image(path)` | Load image into grid for manipulation |

## Examples

| Example | Description | Features |
|---------|-------------|----------|
| `hello_braille` | Minimal braille demo | - |
| `quick_demo` | Quick API showcase | - |
| `load_image` | Load and display images | `image` |
| `simple_animation` | Basic animation loop | - |
| `color_schemes_demo` | Color scheme showcase | - |
| `shapes_demo` | Drawing primitives | - |

See [examples/README.md](examples/README.md) for all examples.

```bash
# Core examples
cargo run --example hello_braille
cargo run --example simple_animation

# Image rendering (requires feature)
cargo run --example load_image --features image
cargo run --example view_image --features image

# Animation examples
cargo run --example bouncing_ball
cargo run --example fireworks
```

## Documentation

- **API Reference**: [docs.rs/dotmax](https://docs.rs/dotmax)
- **Getting Started**: [docs/getting_started.md](docs/getting_started.md)
- **Animation Guide**: [docs/animation_guide.md](docs/animation_guide.md)
- **Performance Guide**: [docs/performance.md](docs/performance.md)
- **Troubleshooting**: [docs/troubleshooting.md](docs/troubleshooting.md)

## Logging

Dotmax uses the [`tracing`](https://docs.rs/tracing) crate for structured logging. The library does **not** initialize a tracing subscriber - your application must do this if you want to see log output.

### Enabling Logging in Your Application

Add `tracing-subscriber` to your `Cargo.toml`:

```toml
[dependencies]
dotmax = "0.1"
tracing-subscriber = "0.3"
```

Initialize the subscriber in your application:

```rust
use tracing_subscriber;

fn main() {
    // Initialize tracing subscriber (do this once at startup)
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Now dotmax operations will emit trace events
    let grid = dotmax::BrailleGrid::new(80, 24).unwrap();
    // Logs: "Creating BrailleGrid: 80×24"
}
```

### Log Levels

Dotmax uses appropriate log levels for different operations:

| Level | Usage | Examples |
|-------|-------|----------|
| `ERROR` | Operation failures | Out-of-bounds errors, invalid dimensions |
| `WARN` | Degraded operation | Terminal lacks Unicode support (future) |
| `INFO` | Major operations | Grid creation, rendering complete |
| `DEBUG` | Detailed flow | Resize operations, color changes |
| `TRACE` | Hot path internals | Not used by default (performance) |

### Controlling Log Output

Use environment variables to control logging:

```bash
# Show all logs
RUST_LOG=dotmax=trace cargo run

# Show only INFO and above
RUST_LOG=dotmax=info cargo run

# Show logs from multiple crates
RUST_LOG=dotmax=debug,my_app=info cargo run
```

### Performance Considerations

Dotmax is designed for zero-cost logging when disabled:
- Hot paths (`set_dot`, `get_dot`) do **not** emit debug logs
- Logging overhead is compile-time removed when no subscriber is initialized
- Enabling logging has minimal performance impact (~<1%)

### Example

See `examples/logging_demo.rs` for a complete demonstration:

```bash
cargo run --example logging_demo
```

For more information, see the [tracing documentation](https://docs.rs/tracing).

## Performance

Dotmax is designed for "efficiency so fast, it's invisible". Here are the benchmark results:

### Core Rendering

| Operation | 80x24 Terminal | 200x50 Terminal |
|-----------|----------------|-----------------|
| Grid creation | ~173ns | ~743ns |
| Grid clear | ~40ns | ~150ns |
| Unicode conversion | ~1.7μs | ~7.1μs |
| Full frame cycle | ~2.1μs | ~9.1μs |

### Image Processing (`--features image`)

| Operation | 800x600 Source | Target |
|-----------|----------------|--------|
| Resize to 80x24 | ~1.5ms | <10ms |
| Full pipeline (resize + threshold + braille) | ~10ms | <25ms |
| Floyd-Steinberg dithering | ~66μs | <15ms |
| Load + render (with I/O) | ~6ms | <50ms |

### Animation

| Operation | 80x24 | 200x50 |
|-----------|-------|--------|
| Frame swap | ~23ns | ~23ns |
| Single frame preparation | ~2.0μs | ~9.0μs |
| 100 frames sustained | ~164μs | ~766μs |
| **Per-frame budget** | **~1.6μs** | **~7.7μs** |

60fps requires <16.67ms per frame. Dotmax achieves **~1.6μs** (10,000x faster than required).

### Memory

- **Baseline**: <5MB
- **Per frame**: <500KB (80x24 grid)
- **Binary size**: <2MB addition to your compiled binary

### Running Benchmarks

```bash
# Core benchmarks (no features)
cargo bench --bench core_rendering

# Image processing benchmarks
cargo bench --bench image_processing --features image

# Animation benchmarks
cargo bench --bench animation

# All benchmarks
cargo bench --all-features
```

## Comparison to Alternatives

| Feature | dotmax | drawille (Python) | Sixel | Kitty Protocol |
|---------|--------|-------------------|-------|----------------|
| **Resolution** | 2×4 dots/char | 2×4 dots/char | True pixels | True pixels |
| **Terminal Support** | Universal (Unicode) | Universal | Limited | Kitty only |
| **Language** | Rust | Python | Various | Various |
| **Performance** | ~2μs/frame | ~10ms/frame | Varies | Fast |
| **Animation** | Built-in | Manual | Manual | Built-in |
| **Colors** | RGB + Schemes | Limited | Full | Full |
| **Dependencies** | Minimal | Minimal | Native | Native |

**When to use dotmax:**
- Need universal terminal compatibility (SSH, tmux, etc.)
- Want Rust performance with zero-cost abstractions
- Building CLI tools or TUI applications
- Need smooth 60fps animations

**When to use Sixel/Kitty:**
- Control both ends (local terminal)
- Need true pixel-level detail (photos)
- Terminal supports the protocol

## Platform Support

- ✅ Windows (x86_64)
- ✅ Linux (x86_64)
- ✅ macOS (x86_64, ARM64)

## Development

### Code Quality

This project enforces high code quality standards using automated tooling. All checks run in CI and must pass before merging.

#### Linting (Clippy)

Run Clippy to catch common mistakes and enforce Rust idioms:

```bash
cargo clippy --all-targets --all-features
```

**Examples are also checked by Clippy in CI.** All code in `examples/` must pass clippy checks with no warnings. To check examples specifically:

```bash
cargo clippy --examples --all-features -- -D warnings
```

Fix any warnings before committing. For false positives, use `#[allow(clippy::lint_name)]` with a comment explaining why.

#### Formatting (Rustfmt)

Format code before committing:

```bash
cargo fmt
```

Check formatting without modifying files:

```bash
cargo fmt --check
```

#### License and Security (cargo-deny)

Install cargo-deny:

```bash
cargo install cargo-deny
```

Check licenses, advisories, and dependencies:

```bash
cargo deny check
```

This validates that all dependencies use permissive licenses (MIT, Apache-2.0, BSD, etc.) and have no known security vulnerabilities.

#### Running All Checks

Before pushing code, ensure all quality checks pass:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --examples --all-features -- -D warnings  # Examples checked separately
cargo deny check
cargo test
```

CI enforces these checks on every push. Pull requests will fail if any check reports violations.

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines (coming soon).

Before submitting pull requests, ensure all code quality checks pass:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --examples --all-features -- -D warnings  # Examples must also pass
cargo deny check
cargo test
```

## License

Licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

## Acknowledgments

Dotmax extracts and professionalizes the braille rendering system from [crabmusic](https://github.com/newjordan/crabmusic), where it has proven exceptional output quality.

## Repository

https://github.com/frosty40/dotmax
