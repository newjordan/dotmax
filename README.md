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

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Create a small braille grid (10 cells wide, 5 cells tall)
    let mut grid = BrailleGrid::new(10, 5);

    // Set some dots to spell "Hello"
    grid.set_dot(0, 0, 0, true);
    grid.set_dot(1, 0, 1, true);
    grid.set_dot(2, 0, 2, true);

    // Render to terminal
    grid.render()?;

    Ok(())
}
```

See [examples/](examples/) for more usage patterns.

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

## Examples

See [examples/README.md](examples/README.md) for all available examples.

Run examples with:

```bash
cargo run --example hello_braille
cargo run --example render_image --features image  # Future
```

## Documentation

API documentation: [docs.rs/dotmax](https://docs.rs/dotmax) (coming soon)

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

Dotmax is designed for "efficiency so fast, it's invisible":

- **Image rendering**: <50ms target (25ms goal) for 80×24 terminals
- **Animation**: 60fps minimum, 120fps target
- **Memory**: <5MB baseline, <500KB per frame
- **Binary size**: <2MB addition to your compiled binary

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
