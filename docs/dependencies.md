# Dependency Justifications

This document provides rationale for all dependencies in the dotmax project, as required by NFR-D1 (Minimal Core Dependencies).

## Core Dependencies

These dependencies are **always included** in dotmax builds (no feature flags required).

### ratatui (0.29)

- **Purpose**: Terminal UI framework and rendering abstraction
- **Why Required**:
  - Industry standard for Rust terminal applications
  - Provides cross-platform terminal abstraction layer
  - Epic 2 will build BrailleGrid rendering on top of ratatui's backend traits
  - Handles terminal state management, cursor positioning, and buffer rendering
- **Alternatives Considered**:
  - `crossterm` alone: Too low-level, would require reimplementing TUI abstractions
  - `termion`: Unix-only, lacks Windows support
  - Custom solution: High maintenance burden, reinventing the wheel
- **Rationale for Version**: 0.29 is latest stable, required for modern widget system
- **License**: MIT
- **Transitive Dependency Count**: ~15 (cassowary for layout, unicode-width, etc.)

### crossterm (0.29)

- **Purpose**: Cross-platform terminal I/O (cursor, colors, events)
- **Why Required**:
  - Works seamlessly with ratatui as the terminal backend
  - Handles platform-specific terminal APIs (Windows Console API, Unix termios)
  - Provides event handling for keyboard/mouse input (future interactive features)
  - Only cross-platform Rust terminal library with production-grade Windows support
- **Alternatives Considered**:
  - `termion`: Unix-only, incompatible with Windows
  - `pancurses`: ncurses wrapper, C dependency overhead
- **Rationale for Version**: 0.29 matches ratatui compatibility requirements
- **License**: MIT
- **Transitive Dependency Count**: ~8 (mio for async I/O, signal-hook, etc.)

### thiserror (2.0)

- **Purpose**: Error handling derive macros
- **Why Required**:
  - Library users need typed errors for pattern matching (ImageLoadError vs RenderError)
  - Minimal boilerplate compared to manual Display/Error implementations
  - Generates high-quality error messages with source chain support
  - Industry standard in Rust ecosystem (used by tokio, serde, etc.)
- **Alternatives Considered**:
  - Manual `impl std::error::Error`: Too verbose, error-prone
  - `anyhow`: Type-erasing errors unsuitable for library crates
  - `snafu`: More verbose than thiserror, less widespread adoption
- **Rationale for Version**: 2.0 is latest major version with improved error chains
- **License**: MIT OR Apache-2.0
- **Transitive Dependency Count**: 2 (thiserror-impl proc-macro crate)

### tracing (0.1)

- **Purpose**: Structured logging and instrumentation
- **Why Required**:
  - Critical for debugging Epic 2+ rendering pipeline (log grid updates, performance traces)
  - Supports multiple log levels (trace, debug, info, warn, error)
  - Can instrument functions with span enter/exit timing
  - Ecosystem standard (integrates with tracing-subscriber for output formatting)
  - Zero-cost abstractions when logging disabled (compile-time filtering)
- **Alternatives Considered**:
  - `log` crate: Less structured, no span support for async contexts
  - `println!` debugging: Not controllable at runtime, poor production experience
  - `env_logger`: Built on `log`, lacks structured fields
- **Rationale for Version**: 0.1 is stable, widely adopted, semver-compatible
- **License**: MIT
- **Transitive Dependency Count**: 3 (tracing-core, tracing-attributes)

---

## Optional Dependencies (Feature-Gated)

These dependencies are **only included when explicitly enabled** via Cargo feature flags.

### image (0.25) - Behind `image` Feature

- **Purpose**: Standard Rust image library for loading/decoding raster images
- **Why Required**:
  - Handles PNG, JPEG, GIF, BMP, WebP, TIFF formats out-of-the-box
  - Epic 3 will use for FR31 (load images from file paths/byte buffers)
  - De facto standard for image handling in Rust (used by 10,000+ crates)
- **Why Feature-Gated**:
  - Large dependency tree (adds ~50 transitive deps)
  - Users only rendering primitives (Epic 4) don't need image decoding
  - Keeps core library <2MB for non-image use cases
- **Alternatives Considered**:
  - Format-specific crates (png, jpeg, etc.): Fragmented, no unified API
  - `image` is the industry standard
- **Rationale for Version**: 0.25 is latest stable
- **License**: MIT OR Apache-2.0
- **Estimated Binary Size Impact**: +1.5MB

### imageproc (0.24) - Behind `image` Feature

- **Purpose**: Image processing algorithms (dithering, thresholding)
- **Why Required**:
  - Epic 3 will use for FR33 (Otsu thresholding) and FR34 (Floyd-Steinberg, Bayer, Atkinson dithering)
  - Built on top of `image` crate, shares types (no conversion overhead)
  - Provides optimized implementations of academic algorithms
- **Why Feature-Gated**:
  - Only needed for image-to-braille conversion pipeline
  - Users rendering primitives/text don't need dithering algorithms
- **Alternatives Considered**:
  - Manual implementation: Would require reimplementing Floyd-Steinberg, Bayer matrices, Otsu's method
  - `imageproc` is battle-tested and actively maintained
- **Rationale for Version**: 0.24 matches `image` 0.25 compatibility
- **License**: MIT
- **Estimated Binary Size Impact**: +500KB

### gif (0.13) - Behind `image` Feature

- **Purpose**: Animated GIF decoding for frame-by-frame playback
- **Why Required**:
  - Story 9.2 requires animated GIF support with timing/disposal methods
  - Provides streaming decode (memory efficient for large animations)
  - Handles NETSCAPE loop extension for loop count
- **Why Feature-Gated**:
  - Part of animation/media system, not needed for static image rendering
  - Bundled with `image` feature since it extends image capabilities
- **Alternatives Considered**:
  - `image` crate alone: Only decodes first frame of animated GIFs
  - Manual parsing: GIF format is complex (LZW, disposal methods, extensions)
- **Rationale for Version**: 0.13 is latest stable
- **License**: MIT OR Apache-2.0
- **Estimated Binary Size Impact**: +150KB

### png (0.18) - Behind `image` Feature

- **Purpose**: PNG decoding with APNG (Animated PNG) animation support
- **Why Required**:
  - Story 9.3 requires animated PNG (APNG) support with frame timing and blend operations
  - Provides APNG-specific APIs: animation_control(), frame_control(), BlendOp, DisposeOp
  - Streaming decode for memory-efficient multi-frame playback
  - Already a transitive dependency of `image` crate; direct dependency needed for APNG APIs
- **Why Feature-Gated**:
  - Part of animation/media system, not needed for static image rendering
  - Bundled with `image` feature since it extends image capabilities
- **Alternatives Considered**:
  - `image` crate alone: No APNG animation API, only reads first frame
  - `apng` crate: Encoder-focused, limited decoder support
  - Direct `png` usage provides full APNG specification compliance
- **Rationale for Version**: 0.18 is latest stable with full APNG support
- **License**: MIT OR Apache-2.0
- **Estimated Binary Size Impact**: ~0KB (already included via image crate dependency tree)

### resvg (0.38) - Behind `svg` Feature

- **Purpose**: SVG rasterization (vector graphics to bitmap conversion)
- **Why Required**:
  - Epic 3 will use for FR36 (SVG support with rasterization)
  - Only production-grade pure-Rust SVG renderer
  - Handles complex SVG features (gradients, transforms, filters)
- **Why Feature-Gated**:
  - Separate feature from raster images (users may want one or the other)
  - Large dependency tree (adds ~30 transitive deps)
  - SVG support is advanced use case, not core functionality
- **Alternatives Considered**:
  - `svg` crate: Only parses SVG, doesn't rasterize
  - `librsvg` (C library): FFI overhead, not pure Rust
  - `resvg` is the only viable pure-Rust option
- **Rationale for Version**: 0.38 is latest stable
- **License**: MPL-2.0 (compatible with MIT/Apache-2.0)
- **Estimated Binary Size Impact**: +1.2MB

### usvg (0.38) - Behind `svg` Feature

- **Purpose**: SVG parsing and simplification
- **Why Required**:
  - Required by `resvg` for SVG processing
  - Parses SVG XML into internal representation
  - Simplifies complex SVG paths for efficient rendering
- **Why Feature-Gated**:
  - Dependency of `resvg`, gated together under `svg` feature
  - No value without SVG support
- **Alternatives Considered**:
  - None - `usvg` is tightly coupled to `resvg` (same author/ecosystem)
- **Rationale for Version**: 0.38 matches `resvg` compatibility
- **License**: MPL-2.0
- **Estimated Binary Size Impact**: Included in `resvg` estimate

### ffmpeg-next (7.0) - Behind `video` Feature

- **Purpose**: FFmpeg bindings for video decoding and playback
- **Why Required**:
  - Story 9.4 requires video playback support (MP4, MKV, AVI, WebM)
  - Provides frame-by-frame video decoding with timing information
  - Supports all major video codecs: H.264, H.265/HEVC, VP9, AV1
  - Industry standard for video processing (FFmpeg powers VLC, YouTube, etc.)
- **Why Feature-Gated**:
  - Requires system FFmpeg libraries (not pure Rust)
  - Large dependency - users must explicitly opt-in
  - Video support is advanced use case, not core functionality
- **Alternatives Considered**:
  - `gstreamer-rs`: Also requires system libraries, less ubiquitous than FFmpeg
  - `video-rs`: Higher-level but less flexible
  - Pure Rust decoders: No comprehensive option exists for all codecs
  - `ffmpeg-next` is well-maintained fork with modern API
- **System Requirements**:
  - Linux: `libavcodec-dev libavformat-dev libavutil-dev libswscale-dev`
  - macOS: `brew install ffmpeg`
  - Windows: FFmpeg binaries in PATH
- **Rationale for Version**: 7.0 supports FFmpeg 6.x/7.x
- **License**: LGPL-2.1+ (FFmpeg itself), crate is MIT
- **Estimated Binary Size Impact**: +2MB (mostly from FFmpeg system libs)

---

## Dev Dependencies

These dependencies are **only used during development/testing** and do NOT impact library users.

### criterion (0.7)

- **Purpose**: Statistics-driven benchmarking with HTML reports
- **Why Required**:
  - Story 1.6 will set up benchmarking infrastructure
  - Provides statistical analysis of performance (mean, variance, outlier detection)
  - Generates HTML reports for visualizing performance trends
  - Industry standard for Rust benchmarking (used by tokio, serde, rayon)
- **Rationale for Version**: 0.7 is latest stable with HTML report features
- **License**: Apache-2.0 OR MIT
- **Impact**: Dev-only, not included in library builds

### tracing-subscriber (0.3)

- **Purpose**: Logging output formatting for tests and examples
- **Why Required**:
  - Formats `tracing` logs for human-readable console output
  - Needed in tests to debug failures (enable with RUST_LOG=debug)
  - Examples will use to demonstrate logging capabilities
- **Rationale for Version**: 0.3 is stable, compatible with `tracing` 0.1
- **License**: MIT
- **Impact**: Dev-only, not included in library builds

---

## Dependency Count Summary

| Category | Count | Counts Toward Core Limit? |
|----------|-------|---------------------------|
| Core (always included) | **4** | ✅ Yes |
| Optional (feature-gated) | 7 | ❌ No |
| Dev (tests/benchmarks) | 2 | ❌ No |
| **Total Direct Dependencies** | **13** | **4 count toward NFR-D1** |

**NFR-D1 Compliance**: ✅ Core library has exactly 4 direct dependencies (<10 limit)

---

## Version Pinning Strategy

All dependency versions use **major version locking**:

```toml
ratatui = "0.29"  # Allows 0.29.x patches, blocks 0.30.x
```

**Rationale** (per Architecture Document ADR 0003):
- Semantic versioning in Rust: Patch/minor updates are non-breaking
- Major version updates reviewed manually via dependabot PRs
- Security updates applied automatically within major version (via cargo-audit + dependabot)
- Reduces breakage risk while staying up-to-date on security patches

**Example**:
- `ratatui = "0.29"` → Cargo resolves to latest 0.29.x (e.g., 0.29.3)
- If 0.30.0 releases, dependabot creates PR for manual review
- If 0.29.4 releases with security patch, next `cargo update` pulls it automatically

---

## Security Considerations

**Dependency Scanning** (per Architecture Document Section 6.3):
- `cargo audit` runs in CI (configured in Story 1.2)
- Checks RustSec Advisory Database for known vulnerabilities
- CI fails if ANY dependency has known CVE
- Manual review required to accept/mitigate vulnerabilities

**License Compliance** (planned for Story 1.4):
- `cargo-deny` will enforce MIT/Apache-2.0 compatible licenses
- No GPL/AGPL dependencies (incompatible with dual licensing strategy)
- All current dependencies verified as permissive licenses

**Supply Chain Risk Mitigation**:
- Major version pinning prevents unexpected breaking changes
- Dependabot monitors for security advisories
- High-profile dependencies (ratatui, crossterm, image) have large communities

---

## Binary Size Impact Analysis

**Core-only build** (`cargo build --release`):
- Measured: **4.1KB** libdotmax.rlib
- Includes: 4 core dependencies + ~30 transitive dependencies
- **Well under 2MB target** (NFR-P6) ✅

**Feature-gated builds**:
- `--features image`: +1.5MB (image decoding/processing)
- `--features svg`: +1.2MB (SVG rasterization)
- `--features image,svg`: ~2.7MB total

**Conclusion**: Feature flag architecture successfully keeps core minimal while supporting advanced use cases.

---

## References

- [Architecture Document - Technology Stack Details](architecture.md#Technology-Stack-Details)
- [Architecture Document - ADR 0003: Feature Flag Architecture](architecture.md#ADR-0003)
- [PRD - NFR-D1: Minimal Core Dependencies](PRD.md#NFR-D1)
- [PRD - NFR-P6: Binary Size Impact](PRD.md#NFR-P6)
- [Story 1.2: CI/CD Pipeline (cargo-audit configuration)](../sprint-artifacts/1-2-configure-github-actions-cicd-pipeline.md)
- [Cargo Book - Features Documentation](https://doc.rust-lang.org/cargo/reference/features.html)

---

**Last Updated**: 2025-12-02 (Story 9.4 - Added ffmpeg-next for video playback)
