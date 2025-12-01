# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-11-26

Initial release of dotmax - a high-performance terminal braille rendering library for Rust.

### Added

#### Epic 1: Foundation & Project Setup
- Cargo project with optimal structure and dual MIT/Apache-2.0 licensing
- GitHub Actions CI/CD pipeline with cross-platform testing (Windows, Linux, macOS)
- Feature flags architecture for minimal binary size
- Code quality tooling: Clippy, Rustfmt, cargo-deny, cargo-audit
- Benchmarking infrastructure with Criterion
- MSRV: Rust 1.70

#### Epic 2: Core Braille Rendering Engine
- `BrailleGrid` - Core rendering surface with 2x4 dot matrix per cell
- Unicode braille character conversion (U+2800-U+28FF)
- `TerminalRenderer` with ratatui/crossterm backend
- `TerminalBackend` trait for custom backends
- `DotmaxError` enum with comprehensive error handling (thiserror)
- Terminal resize event handling
- Color support for braille cells (RGB)
- Debug logging with tracing integration
- Proper viewport detection and rendering

#### Epic 3: 2D Image Rendering Pipeline
- Image loading from file paths and byte buffers (PNG, JPG, GIF, BMP, WebP, TIFF)
- `ImageRenderer` with builder pattern
- Resize with aspect ratio preservation
- Multiple resize filters (Lanczos3, Triangle, CatmullRom, Gaussian, Nearest)
- Adaptive filter selection for extreme aspect ratios (45% performance improvement)
- Grayscale conversion using BT.709 coefficients
- Otsu thresholding for automatic binary threshold detection
- Threshold toggle control (auto/manual)
- Dithering algorithms:
  - Floyd-Steinberg (error diffusion)
  - Bayer (ordered dithering, 2x2 to 8x8 matrices)
  - Atkinson (Apple-style limited error diffusion)
- Binary image to braille grid conversion
- SVG vector graphics support with rasterization (resvg/usvg)
- Font handling for SVG text elements
- Color mode image rendering with RGB output
- High-level `render_file()` and `render_bytes()` API

#### Epic 4: Drawing Primitives & Density Rendering
- Line drawing with Bresenham's algorithm
- Line thickness support (1-10 recommended)
- Circle drawing with midpoint circle algorithm (8-way symmetry)
- Filled circles with scanline fill
- Circle thickness support
- Rectangle drawing (outline, filled, thick border)
- Polygon drawing with automatic path closing
- Filled polygons with even-odd scanline fill
- Non-convex and self-intersecting polygon support
- Boundary clipping for out-of-bounds primitives
- Character density rendering (`DensitySet`)
- Predefined density sets: ASCII (69 chars), Simple (10), Blocks (5), Braille (9)
- Custom density set creation
- Colored primitives: `draw_line_colored`, `draw_circle_colored`, `draw_rectangle_colored`, `draw_polygon_colored`

#### Epic 5: Color System & Visual Schemes
- Terminal color capability detection (`ColorCapability` enum)
- Automatic environment-based detection ($COLORTERM, $TERM)
- RGB to ANSI color conversion (16, 256, TrueColor)
- 6 color schemes extracted from crabmusic:
  - Plasma (vibrant pink-to-blue gradient)
  - Neon (cyan to magenta)
  - Sunset (warm yellow to deep red)
  - Ocean (deep blue to light cyan)
  - Forest (earth greens)
  - Monochrome (grayscale gradient)
- `ColorScheme` type for intensity-to-color mapping
- `ColorSchemeBuilder` for custom scheme creation
- `from_colors()` convenience constructor
- Grayscale intensity buffer rendering with color schemes

#### Epic 6: Animation & Frame Management
- `FrameBuffer` with double-buffering for flicker-free updates
- Buffer swap in ~23ns (450,000x faster than 1ms target)
- `FrameTimer` for consistent frame rate control
- Configurable FPS (1-240)
- High-precision timing with spin-wait for accuracy
- `AnimationLoop` high-level animation abstraction
- Builder pattern with `fps()`, `on_frame()`, `run()`
- `PrerenderedAnimation` for cached frame sequences
- Frame caching for looping animations
- `DifferentialRenderer` - only render changed cells (90%+ I/O savings)
- Cell-level change tracking
- 5 animation examples: bouncing_ball, loading_spinner, waveform, fireworks, clock

#### Epic 7: API Design, Performance & Production Readiness
- Public API surface design with organized module re-exports
- `pub type Result<T>` alias for ergonomic error handling
- Comprehensive rustdoc with `#![warn(missing_docs)]`
- Thread safety documentation (Send/Sync bounds)
- Comprehensive benchmarking suite:
  - Core rendering benchmarks
  - Image processing benchmarks
  - Animation benchmarks
- Performance optimization (profiled with flamegraph)
- 925+ tests (66% increase from baseline):
  - Unit tests in all modules
  - Integration tests
  - 28 property-based tests (proptest)
  - 13 visual regression tests
  - 232+ doc tests
- CI coverage reporting with cargo-tarpaulin
- Comprehensive documentation:
  - Updated README with visual demos
  - Getting started guide
  - Performance guide
  - Troubleshooting guide
  - Animation guide
- 49 examples covering all features

### Performance

- Grid creation: ~173ns (80x24), ~743ns (200x50)
- Unicode conversion: ~1.7us (80x24)
- Image pipeline: ~7.9ms (vs 25ms target)
- Animation frame: ~1.6us (10,000x faster than 60fps requirement)
- Frame swap: ~23ns
- Memory baseline: <5MB
- Per-frame overhead: <500KB
- Binary size: <2MB core

### Platform Support

- Windows x86_64
- Linux x86_64
- macOS x86_64 and ARM64

### Links

- Documentation: https://docs.rs/dotmax
- Repository: https://github.com/frosty40/dotmax
- crates.io: https://crates.io/crates/dotmax
