# Epic Technical Specification: API Design, Performance & Production Readiness

Date: 2025-11-25
Author: Frosty
Epic ID: 7
Status: Draft

---

## Overview

Epic 7 represents the final milestone before dotmax becomes a production-ready crate on crates.io. With Epics 1-6 complete (Foundation, Core Rendering, Image Pipeline, Drawing Primitives, Color System, and Animation), the library is functionally complete with 557+ tests passing, zero clippy warnings, and comprehensive feature coverage.

This epic transforms working code into a polished, professional library through API refinement, comprehensive benchmarking, performance optimization, enhanced testing, documentation excellence, and finally publication. The goal is ensuring any Rust developer can `cargo add dotmax` and have a working braille renderer in under 5 minutes with less than 100 lines of integration code.

**Current State:**
- 6 epics delivered with all stories marked done
- 557+ tests passing, 232+ doc tests
- Zero clippy warnings across all modules
- Feature-gated optional dependencies (image, svg)
- 5 animation examples, comprehensive color scheme support

**Target State:**
- Production-ready API surface documented on docs.rs
- Performance targets verified (<25ms renders, 60fps sustained)
- Published to crates.io as v0.1.0
- External validation via POC integration

## Objectives and Scope

### In-Scope

1. **API Design & Documentation (Story 7.1)**
   - Review and finalize public API surface in `src/lib.rs`
   - Ensure all public types have comprehensive rustdoc
   - Add module-level documentation explaining purpose
   - Enforce `#![warn(missing_docs)]` across crate

2. **Benchmarking Suite (Story 7.2)**
   - Core rendering benchmarks (grid ops, unicode conversion)
   - Image processing benchmarks (pipeline timing)
   - Animation benchmarks (60fps validation)
   - CI integration with regression detection

3. **Performance Optimization (Story 7.3)**
   - Profile with flamegraph to identify bottlenecks
   - Optimize top 3 hot paths based on benchmark data
   - Target <20ms image renders (beat <25ms by 20%)
   - Target <10% CPU at 60fps sustained

4. **Test Suite Enhancement (Story 7.4)**
   - Property-based tests with proptest
   - Visual regression tests for rendering correctness
   - Integration tests for full pipelines
   - Coverage reporting in CI

5. **Documentation & Examples (Story 7.5)**
   - Comprehensive README.md with GIFs/images
   - Complete docs.rs coverage
   - Example suite (hello_braille, load_image, animation, etc.)
   - Guides: getting_started.md, performance.md, troubleshooting.md

6. **Publication (Story 7.6)**
   - Pre-publication checklist automation
   - Release workflow for GitHub Actions
   - crates.io publication as v0.1.0
   - CHANGELOG.md maintenance

7. **External Validation (Story 7.7)**
   - POC integration with yazi or bat
   - <100 lines integration requirement
   - Collect maintainer feedback
   - Document lessons learned

### Out-of-Scope

- New rendering features (covered in Epics 2-6)
- Async/await API (deferred to post-1.0 per ADR-0006)
- Alternative backends beyond ratatui/crossterm
- Video or 3D rendering capabilities
- Breaking API changes after v1.0 publication

## System Architecture Alignment

This epic aligns with the architecture defined in `docs/architecture.md`:

**Module Structure Validation:**
```
src/
├── lib.rs         # Public API re-exports (Story 7.1 focus)
├── error.rs       # DotmaxError enum (verified complete)
├── grid.rs        # BrailleGrid, Color (core, always available)
├── render.rs      # TerminalRenderer, TerminalBackend trait
├── image/         # Feature-gated (image, svg flags)
├── primitives.rs  # Drawing primitives (Epic 4)
├── density.rs     # Character density rendering
├── color/         # Color schemes, conversion (Epic 5)
├── animation/     # Frame buffers, timing, loops (Epic 6)
└── utils/         # Terminal capability detection
```

**Architecture Constraints Respected:**
- Sync-only API (ADR-0006): No async runtime dependency
- Feature flags (ADR-0003): Core <2MB, opt-in for image/svg
- Terminal abstraction (ADR-0004): TerminalBackend trait maintained
- Measure-first optimization (ADR-0007): Benchmark before optimize

**Performance Architecture:**
- Buffer reuse pattern: BrailleGrid.clear() reuses allocation
- Pipeline profiling: Each image stage benchmarked separately
- Memory targets: <5MB baseline, <500KB per frame

## Detailed Design

### Services and Modules

| Module/File | Responsibility | Story | Inputs | Outputs |
|-------------|----------------|-------|--------|---------|
| `src/lib.rs` | Public API surface, re-exports, module docs | 7.1 | All public types | Unified API |
| `benches/core_rendering.rs` | Grid operation benchmarks | 7.2 | BrailleGrid | Criterion reports |
| `benches/image_processing.rs` | Image pipeline benchmarks | 7.2 | Test images | Timing data |
| `benches/animation.rs` | Frame rate validation | 7.2 | FrameBuffer, AnimationLoop | FPS metrics |
| `tests/property_tests.rs` | Property-based testing | 7.4 | Proptest strategies | Pass/fail |
| `tests/visual_regression.rs` | Rendering correctness | 7.4 | Test images | Baseline comparisons |
| `examples/hello_braille.rs` | Minimal 10-line example | 7.5 | None | Terminal output |
| `docs/getting_started.md` | Tutorial walkthrough | 7.5 | None | User guide |
| `.github/workflows/release.yml` | Automated publishing | 7.6 | Git tag | crates.io publish |
| `CHANGELOG.md` | Version history | 7.6 | Release notes | User documentation |

### Data Models and Contracts

**Benchmark Result Contract:**
```rust
// Criterion benchmark output structure
struct BenchmarkResult {
    name: String,           // e.g., "grid_creation_80x24"
    mean_ns: f64,           // Mean execution time in nanoseconds
    std_dev_ns: f64,        // Standard deviation
    throughput: Option<f64>, // Operations per second (where applicable)
}
```

**Test Coverage Report:**
```rust
// Coverage reporting structure (tarpaulin output)
struct CoverageReport {
    line_coverage: f64,     // Percentage (target: >70%)
    branch_coverage: f64,   // Percentage
    function_coverage: f64, // Percentage
    uncovered_lines: Vec<(String, u32)>, // (file, line_number)
}
```

**Release Metadata (Cargo.toml):**
```toml
[package]
name = "dotmax"
version = "0.1.0"  # Semantic versioning
edition = "2021"
rust-version = "1.70"  # MSRV
license = "MIT OR Apache-2.0"
repository = "https://github.com/newjordan/dotmax"
documentation = "https://docs.rs/dotmax"
keywords = ["terminal", "braille", "graphics", "cli", "visualization"]
categories = ["command-line-interface", "graphics", "rendering"]
```

**Public API Surface (Story 7.1 target):**
```rust
// Top-level re-exports in src/lib.rs
pub use grid::BrailleGrid;
pub use render::{TerminalBackend, TerminalRenderer, TerminalCapabilities};
pub use color::{Color, ColorScheme, ColorSchemeBuilder};
pub use error::DotmaxError;
pub use animation::{FrameBuffer, FrameTimer, AnimationLoop, DifferentialRenderer};

#[cfg(feature = "image")]
pub use image::ImageRenderer;

pub type Result<T> = std::result::Result<T, DotmaxError>;
```

### APIs and Interfaces

**Story 7.1: API Surface Review Checklist**

| Public Type | Module | Documentation | Thread-Safe | Notes |
|-------------|--------|---------------|-------------|-------|
| `BrailleGrid` | grid | ✓ Complete | Send + Sync | Core type |
| `Color` | grid | ✓ Complete | Copy | RGB tuple |
| `TerminalRenderer` | render | ✓ Complete | !Send | Owns terminal |
| `TerminalBackend` | render | ✓ Complete | trait | Abstraction |
| `DotmaxError` | error | ✓ Complete | Send + Sync | thiserror |
| `ColorScheme` | color | ✓ Complete | Clone | Intensity mapping |
| `FrameBuffer` | animation | ✓ Complete | Send + Sync | Double buffer |
| `AnimationLoop` | animation | ✓ Complete | !Send | Owns renderer |
| `ImageRenderer` | image | ✓ Complete | Send + Sync | Feature-gated |

**Story 7.2: Benchmark API**

```rust
// benches/core_rendering.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_grid_creation(c: &mut Criterion) {
    c.bench_function("grid_creation_80x24", |b| {
        b.iter(|| BrailleGrid::new(80, 24).unwrap())
    });
}

fn bench_dot_operations(c: &mut Criterion) {
    let mut grid = BrailleGrid::new(80, 24).unwrap();
    c.bench_function("set_dot_1000", |b| {
        b.iter(|| {
            for i in 0..1000 {
                grid.set_dot(i % 160, i % 96).unwrap();
            }
        })
    });
}

criterion_group!(benches, bench_grid_creation, bench_dot_operations);
criterion_main!(benches);
```

**Story 7.6: Release Workflow API**

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags: ['v*']

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
      - run: cargo clippy --all-features -- -D warnings
      - run: cargo publish --token ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

### Workflows and Sequencing

**Story Dependency Graph:**
```
Story 7.1 (API Design) ─────┐
                            ├──► Story 7.5 (Documentation)
Story 7.2 (Benchmarking) ───┤
         │                  │
         ▼                  │
Story 7.3 (Optimization) ───┤
                            │
Story 7.4 (Testing) ────────┤
                            ▼
                    Story 7.6 (Publication)
                            │
                            ▼
                    Story 7.7 (POC Integration)
```

**Release Workflow Sequence:**

```
1. Developer creates release branch
   └── git checkout -b release/v0.1.0

2. Pre-release validation
   ├── cargo test --all-features
   ├── cargo clippy --all-features -- -D warnings
   ├── cargo doc --no-deps --all-features
   ├── cargo bench
   └── cargo publish --dry-run

3. Version bump & changelog
   ├── Update Cargo.toml version
   └── Update CHANGELOG.md

4. Tag and push
   ├── git tag v0.1.0
   └── git push origin v0.1.0

5. CI Release Workflow triggers
   ├── Full test suite (all platforms)
   ├── Benchmark comparison
   ├── cargo publish (automated)
   └── GitHub Release creation

6. Post-release verification
   ├── Verify crates.io page
   ├── Test: cargo add dotmax
   └── Announce release
```

**Benchmark-Optimize Loop (Story 7.2 → 7.3):**

```
1. Run full benchmark suite
   └── cargo bench --all-features

2. Identify bottlenecks (>10% of total time)
   └── cargo flamegraph --example render_image --features image

3. For each bottleneck:
   ├── Document current performance
   ├── Implement optimization
   ├── Re-run benchmark
   ├── Validate improvement (must be measurable)
   └── Document optimization in ADR

4. Repeat until targets met:
   ├── Image render: <20ms (80×24)
   ├── Animation: 60fps @ <10% CPU
   └── Memory: <500KB per frame
```

## Non-Functional Requirements

### Performance

**NFR-P1: Rendering Latency** (FR68-70, Architecture §Performance Considerations)
| Metric | Target | Measurement | Story |
|--------|--------|-------------|-------|
| Image-to-braille (80×24) | <25ms | Criterion benchmark | 7.2, 7.3 |
| Large terminal (200×50) | <100ms | Criterion benchmark | 7.2 |
| SVG rasterization | <100ms | Criterion benchmark | 7.2 |
| Grid creation | <1ms | Criterion benchmark | 7.2 |
| Unicode conversion | <5ms | Criterion benchmark | 7.2 |

**NFR-P2: Animation Performance** (FR71-72, Architecture §Pattern 3)
| Metric | Target | Measurement | Story |
|--------|--------|-------------|-------|
| Sustained FPS | 60fps minimum | Frame timing validation | 7.2 |
| CPU utilization | <10% single core | System profiling | 7.3 |
| Frame drops | 0 over 30 seconds | Animation loop test | 7.2 |
| Buffer swap | <1ms | Criterion benchmark | 7.2 |

**NFR-P3: Memory Efficiency** (FR73-74, Architecture §Performance)
| Metric | Target | Measurement | Story |
|--------|--------|-------------|-------|
| Baseline memory | <5MB | Valgrind/heaptrack | 7.3 |
| Per-frame overhead | <500KB | Memory profiling | 7.3 |
| Memory leaks | Zero | Valgrind CI check | 7.4 |
| Binary size (core) | <2MB | cargo-bloat | 7.3 |

**NFR-P4: Startup Performance**
| Metric | Target | Measurement | Story |
|--------|--------|-------------|-------|
| Library init | <5ms cold start | Criterion benchmark | 7.2 |
| First render | <10ms after init | Criterion benchmark | 7.2 |

### Security

**NFR-S1: Memory Safety** (Architecture §Security Architecture)
- Zero `unsafe` code in MVP (rely on Rust guarantees)
- If unsafe required: isolate in separate module, document invariants, test with MIRI
- No buffer overflows (Rust prevents out-of-bounds access)

**NFR-S2: Input Validation**
- Image files validated by `image` crate (external input handling)
- Dimension checks for zero/overflow before allocation
- Resource limits enforced:
  ```rust
  const MAX_GRID_WIDTH: usize = 10_000;
  const MAX_GRID_HEIGHT: usize = 10_000;
  ```

**NFR-S3: Dependency Security** (Story 7.6)
- `cargo-audit` in CI: Detect known vulnerabilities
- Minimal dependencies: <10 core (reduce attack surface)
- Permissive licenses only: MIT/Apache-2.0 (no viral licenses)
- `cargo-deny` checks on every PR

### Reliability/Availability

**NFR-R1: Error Handling**
- All public functions return `Result<T, DotmaxError>`
- No panics in library code (validated by tests)
- Graceful degradation for unsupported terminals
- Clear error messages with context (thiserror)

**NFR-R2: Platform Compatibility** (FR75-80)
| Platform | Support Level | CI Testing |
|----------|---------------|------------|
| Linux x86_64 | Tier 1 | ubuntu-latest |
| Windows x86_64 | Tier 1 | windows-latest |
| macOS x86_64 | Tier 1 | macos-latest |
| macOS ARM64 | Tier 1 | macos-latest |

**NFR-R3: Rust Version Support**
- MSRV: Rust 1.70 (documented, CI-enforced)
- Stable toolchain only (no nightly features)
- MSRV bumped only with minor version increments

### Observability

**NFR-O1: Logging** (Architecture §Logging Strategy)
- Uses `tracing` crate for structured logging
- Library does NOT initialize subscriber (application responsibility)
- Log levels:
  | Level | Usage |
  |-------|-------|
  | ERROR | Operation failures |
  | WARN | Unexpected but recoverable |
  | INFO | Major operations (grid creation, render complete) |
  | DEBUG | Detailed flow (resize, color changes) |
  | TRACE | Hot path internals (disabled by default) |

**NFR-O2: Diagnostics**
- Terminal capability detection logged at DEBUG
- Performance timing available via tracing spans
- Error types include context for debugging

**NFR-O3: Benchmark Reporting** (Story 7.2)
- Criterion HTML reports in `target/criterion/`
- CI stores benchmark artifacts
- Regression alerts on >10% slowdown

## Dependencies and Integrations

### Core Dependencies (Always Included)

| Crate | Version | Purpose | Story Impact |
|-------|---------|---------|--------------|
| `ratatui` | 0.29 | Terminal UI framework | 7.1 (API surface) |
| `crossterm` | 0.29 | Cross-platform terminal I/O | 7.1 (API surface) |
| `thiserror` | 2.0 | Error handling derive macros | 7.1 (error types) |
| `tracing` | 0.1 | Structured logging | 7.1 (observability) |

### Optional Dependencies (Feature-Gated)

| Crate | Version | Feature Flag | Purpose | Story Impact |
|-------|---------|--------------|---------|--------------|
| `image` | 0.25 | `image` | Image loading (PNG, JPG, etc.) | 7.2 (benchmarks) |
| `imageproc` | 0.24 | `image` | Image processing algorithms | 7.2 (benchmarks) |
| `resvg` | 0.38 | `svg` | SVG rasterization | 7.2 (benchmarks) |
| `usvg` | 0.38 | `svg` | SVG parsing | 7.2 (benchmarks) |

### Dev Dependencies

| Crate | Version | Purpose | Story Impact |
|-------|---------|---------|--------------|
| `criterion` | 0.7 | Benchmarking framework | 7.2 (primary) |
| `tracing-subscriber` | 0.3 | Log output for tests | 7.4 (testing) |
| `tempfile` | 3.10 | Temporary file handling | 7.4 (testing) |
| `proptest` | 1.4 (proposed) | Property-based testing | 7.4 (new addition) |

### New Dependencies for Epic 7

| Crate | Version | Purpose | Story | Justification |
|-------|---------|---------|-------|---------------|
| `proptest` | 1.4 | Property-based testing | 7.4 | Fuzz testing for grid ops, color conversion |
| `cargo-tarpaulin` | (tool) | Coverage reporting | 7.4 | CI coverage metrics |
| `flamegraph` | (tool) | Performance profiling | 7.3 | Bottleneck identification |

### Feature Flag Configuration

```toml
[features]
default = []
image = ["dep:image", "dep:imageproc"]
svg = ["dep:resvg", "dep:usvg"]
# Future (post-1.0):
# video = ["dep:ffmpeg-next"]
# 3d = ["dep:nalgebra"]
```

### Integration Points

**Story 7.7: POC Integration Targets**

| Target | Type | Integration Complexity | Value |
|--------|------|------------------------|-------|
| **yazi** | File manager | Medium | Image preview in file browser |
| **bat** | File viewer | Low | Braille image rendering |
| **hx/helix** | Text editor | Medium | Image preview in editor |
| **gitui** | Git TUI | Low | Diff visualization |

**Integration Contract (Story 7.7):**
```rust
// Target: <100 lines for basic integration
use dotmax::{BrailleGrid, TerminalRenderer, ImageRenderer};

fn render_image_preview(path: &Path, width: u16, height: u16) -> Result<(), dotmax::DotmaxError> {
    let renderer = ImageRenderer::builder()
        .dither_method(DitherMethod::FloydSteinberg)
        .build();

    let grid = renderer.render_file(path, width as usize, height as usize)?;

    let mut terminal = TerminalRenderer::new()?;
    terminal.render(&grid)?;

    Ok(())
}
```

### External System Interfaces

| System | Interface | Direction | Story |
|--------|-----------|-----------|-------|
| Terminal | ratatui/crossterm | Output | 7.1 |
| Filesystem | std::fs, image crate | Input | 7.2 |
| crates.io | cargo publish | Output | 7.6 |
| GitHub Actions | CI/CD workflows | Bidirectional | 7.2, 7.6 |
| docs.rs | Documentation hosting | Output | 7.5 |

### Dependency Security Validation

**Pre-publication checks (Story 7.6):**
```bash
# Security audit
cargo audit

# License compliance
cargo deny check licenses

# Advisory database check
cargo deny check advisories

# Banned crates check
cargo deny check bans
```

**Allowed Licenses:**
- MIT
- Apache-2.0
- BSD-2-Clause
- BSD-3-Clause
- ISC
- Zlib

## Acceptance Criteria (Authoritative)

### Story 7.1: Design and Document Public API Surface

| AC# | Criteria | Testable Condition |
|-----|----------|-------------------|
| AC1 | `src/lib.rs` exposes organized module structure | Modules: grid, terminal, image, primitives, density, color, animation, error |
| AC2 | Top-level re-exports for convenience types | `BrailleGrid`, `TerminalRenderer`, `Color`, `ColorScheme`, `DotmaxError` accessible from `dotmax::` |
| AC3 | `pub type Result<T>` alias defined | `dotmax::Result<T>` equivalent to `Result<T, DotmaxError>` |
| AC4 | Module-level documentation complete | Every `pub mod` has `//!` doc explaining purpose |
| AC5 | All public items have rustdoc | `#![warn(missing_docs)]` passes with zero warnings |
| AC6 | Code examples in rustdoc | Key types have `# Examples` section with working code |
| AC7 | Errors section documented | Result-returning functions document error conditions |
| AC8 | No internal types leaked | Only intentional public API visible |
| AC9 | Thread safety documented | `Send`/`Sync` bounds noted where applicable |

### Story 7.2: Implement Comprehensive Benchmarking Suite

| AC# | Criteria | Testable Condition |
|-----|----------|-------------------|
| AC1 | Core rendering benchmarks exist | `benches/core_rendering.rs` with grid_creation, dot_ops, unicode_conversion |
| AC2 | Image processing benchmarks exist | `benches/image_processing.rs` with load, resize, dither, threshold, full_pipeline |
| AC3 | Animation benchmarks exist | `benches/animation.rs` with frame_swap, differential_render, 60fps_sustained |
| AC4 | All benchmarks run successfully | `cargo bench --all-features` completes without errors |
| AC5 | Image render < 25ms target | 80×24 terminal benchmark mean < 25ms |
| AC6 | 60fps animation achievable | Frame timing benchmark shows < 16.67ms per frame |
| AC7 | CI benchmark integration | `.github/workflows/benchmark.yml` runs on main branch |
| AC8 | Regression detection configured | CI comments on PR if > 10% regression |
| AC9 | Benchmark results documented | README.md includes performance table |

### Story 7.3: Optimize Hot Paths Based on Benchmark Data

| AC# | Criteria | Testable Condition |
|-----|----------|-------------------|
| AC1 | Profiling completed | Flamegraph generated for image rendering pipeline |
| AC2 | Top 3 bottlenecks identified | Documented list of hotspots with % of total time |
| AC3 | Each optimization measured | Before/after benchmark comparison for each change |
| AC4 | Image render < 20ms | Beat 25ms target by 20% margin |
| AC5 | Animation CPU < 10% | 60fps sustained at < 10% single-core utilization |
| AC6 | Memory baseline < 5MB | Heap profiling confirms < 5MB for core operations |
| AC7 | Per-frame overhead < 500KB | Animation frame allocation < 500KB |
| AC8 | No performance regressions | All existing benchmarks maintain or improve |
| AC9 | Optimization ADR created | `docs/adr/NNNN-performance-optimizations.md` documents decisions |

### Story 7.4: Implement Comprehensive Test Suite

| AC# | Criteria | Testable Condition |
|-----|----------|-------------------|
| AC1 | Unit tests for all modules | Each `src/*.rs` has `#[cfg(test)]` module |
| AC2 | Integration tests exist | `tests/` directory with pipeline tests |
| AC3 | Property-based tests added | `proptest` tests for grid ops, color conversion |
| AC4 | Visual regression tests exist | `tests/visual/` with baseline comparisons |
| AC5 | Core coverage > 80% | `cargo tarpaulin` reports > 80% line coverage for grid, render |
| AC6 | Overall coverage > 70% | Total project coverage > 70% |
| AC7 | All tests pass | `cargo test --all-features` exits 0 |
| AC8 | No test warnings | Tests compile without warnings |
| AC9 | CI coverage reporting | Coverage report generated and stored as artifact |

### Story 7.5: Write Comprehensive Documentation and Examples

| AC# | Criteria | Testable Condition |
|-----|----------|-------------------|
| AC1 | README.md complete | Includes: description, quick start, features, performance, examples, badges |
| AC2 | docs.rs coverage 100% | `cargo doc` with `#![warn(missing_docs)]` passes |
| AC3 | Example suite complete | `hello_braille.rs`, `load_image.rs`, `animation_simple.rs`, `color_schemes.rs`, `drawing_shapes.rs` |
| AC4 | All examples compile | `cargo build --examples --all-features` succeeds |
| AC5 | Getting started guide exists | `docs/getting_started.md` with tutorial |
| AC6 | Performance guide exists | `docs/performance.md` with optimization tips |
| AC7 | Troubleshooting guide exists | `docs/troubleshooting.md` with common issues |
| AC8 | Doctests pass | `cargo test --doc` exits 0 |
| AC9 | 5-minute integration achievable | New user can render image in < 5 minutes following docs |

### Story 7.6: Publish to crates.io and Create Release Process

| AC# | Criteria | Testable Condition |
|-----|----------|-------------------|
| AC1 | Pre-publication checklist passes | All tests, clippy, docs, benchmarks green |
| AC2 | Version set to 0.1.0 | `Cargo.toml` version = "0.1.0" |
| AC3 | CHANGELOG.md updated | Contains 0.1.0 release notes |
| AC4 | Dry-run succeeds | `cargo publish --dry-run` exits 0 |
| AC5 | Release workflow exists | `.github/workflows/release.yml` triggers on tag |
| AC6 | Published to crates.io | `cargo add dotmax` works |
| AC7 | GitHub release created | Tag `v0.1.0` has release notes |
| AC8 | docs.rs builds | Documentation available at docs.rs/dotmax |
| AC9 | Post-release verification | Installation tested in clean environment |

### Story 7.7: Create Proof-of-Concept Integration

| AC# | Criteria | Testable Condition |
|-----|----------|-------------------|
| AC1 | Integration target selected | yazi or bat chosen with rationale |
| AC2 | Integration < 100 lines | Core dotmax usage in < 100 LOC |
| AC3 | Feature works end-to-end | Image preview renders correctly in target app |
| AC4 | PR submitted (optional) | Draft PR or fork demonstrating integration |
| AC5 | Performance acceptable | No visible lag in target application |
| AC6 | Feedback collected | At least 1 external user/maintainer feedback |
| AC7 | Lessons documented | `docs/integration-lessons.md` with findings |
| AC8 | API pain points identified | List of usability issues discovered |
| AC9 | Improvements backlogged | Issues created for identified improvements |

## Traceability Mapping

| AC | Spec Section | Component/API | Test Approach |
|----|--------------|---------------|---------------|
| **Story 7.1** |
| 7.1-AC1 | §Detailed Design | `src/lib.rs` | Manual review |
| 7.1-AC2 | §APIs and Interfaces | Top-level re-exports | Compile test |
| 7.1-AC3 | §Data Models | `dotmax::Result<T>` | Compile test |
| 7.1-AC4 | §APIs and Interfaces | Module docs | `cargo doc` |
| 7.1-AC5 | §APIs and Interfaces | All public items | `#![warn(missing_docs)]` |
| 7.1-AC6 | §APIs and Interfaces | Rustdoc examples | `cargo test --doc` |
| 7.1-AC7 | §APIs and Interfaces | Error documentation | Manual review |
| 7.1-AC8 | §APIs and Interfaces | API surface | `cargo doc --document-private-items` comparison |
| 7.1-AC9 | §APIs and Interfaces | Thread safety | Type bounds check |
| **Story 7.2** |
| 7.2-AC1 | §Workflows | `benches/core_rendering.rs` | `cargo bench` |
| 7.2-AC2 | §Workflows | `benches/image_processing.rs` | `cargo bench --features image` |
| 7.2-AC3 | §Workflows | `benches/animation.rs` | `cargo bench` |
| 7.2-AC4 | §NFR Performance | All benchmarks | `cargo bench --all-features` |
| 7.2-AC5 | §NFR-P1 | Image pipeline | Criterion report |
| 7.2-AC6 | §NFR-P2 | Animation timing | Frame timing test |
| 7.2-AC7 | §Workflows | CI integration | GitHub Actions |
| 7.2-AC8 | §Workflows | Regression detection | CI workflow |
| 7.2-AC9 | §Documentation | README.md | Manual review |
| **Story 7.3** |
| 7.3-AC1 | §Workflows | Flamegraph | `cargo flamegraph` |
| 7.3-AC2 | §Workflows | Profiling analysis | Documentation |
| 7.3-AC3 | §Workflows | Optimization validation | Benchmark comparison |
| 7.3-AC4 | §NFR-P1 | Image render timing | Criterion report |
| 7.3-AC5 | §NFR-P2 | CPU utilization | System profiler |
| 7.3-AC6 | §NFR-P3 | Memory baseline | Valgrind/heaptrack |
| 7.3-AC7 | §NFR-P3 | Frame overhead | Memory profiler |
| 7.3-AC8 | §NFR Performance | All benchmarks | Regression test |
| 7.3-AC9 | §Architecture | ADR | File existence |
| **Story 7.4** |
| 7.4-AC1 | §Services | Unit tests | `cargo test` |
| 7.4-AC2 | §Services | Integration tests | `cargo test` |
| 7.4-AC3 | §Dependencies | Property tests | `cargo test` with proptest |
| 7.4-AC4 | §Services | Visual regression | Baseline comparison |
| 7.4-AC5 | §NFR Reliability | Coverage | `cargo tarpaulin` |
| 7.4-AC6 | §NFR Reliability | Coverage | `cargo tarpaulin` |
| 7.4-AC7 | §NFR Reliability | All tests | CI workflow |
| 7.4-AC8 | §NFR Reliability | Test quality | Compiler warnings |
| 7.4-AC9 | §Observability | Coverage reporting | CI artifacts |
| **Story 7.5** |
| 7.5-AC1 | §Documentation | README.md | Manual review |
| 7.5-AC2 | §APIs | docs.rs | `cargo doc` |
| 7.5-AC3 | §Services | Examples | `cargo build --examples` |
| 7.5-AC4 | §Services | Examples | Compile test |
| 7.5-AC5 | §Documentation | Getting started | File existence |
| 7.5-AC6 | §Documentation | Performance guide | File existence |
| 7.5-AC7 | §Documentation | Troubleshooting | File existence |
| 7.5-AC8 | §Documentation | Doctests | `cargo test --doc` |
| 7.5-AC9 | §Scope | User experience | Manual timing test |
| **Story 7.6** |
| 7.6-AC1 | §Workflows | Pre-publication | Checklist script |
| 7.6-AC2 | §Data Models | Cargo.toml | Version check |
| 7.6-AC3 | §Documentation | CHANGELOG.md | File content |
| 7.6-AC4 | §Workflows | Dry-run | `cargo publish --dry-run` |
| 7.6-AC5 | §Workflows | Release workflow | GitHub Actions |
| 7.6-AC6 | §Dependencies | crates.io | `cargo add dotmax` |
| 7.6-AC7 | §Workflows | GitHub release | Tag verification |
| 7.6-AC8 | §Dependencies | docs.rs | URL check |
| 7.6-AC9 | §Workflows | Installation | Clean env test |
| **Story 7.7** |
| 7.7-AC1 | §Integration Points | Target selection | Documentation |
| 7.7-AC2 | §Integration Points | LOC count | Code review |
| 7.7-AC3 | §Integration Points | Functionality | Manual test |
| 7.7-AC4 | §Integration Points | PR | GitHub |
| 7.7-AC5 | §NFR Performance | Perceived performance | Manual test |
| 7.7-AC6 | §Integration Points | External feedback | Documentation |
| 7.7-AC7 | §Documentation | Lessons learned | File existence |
| 7.7-AC8 | §Integration Points | Usability analysis | Documentation |
| 7.7-AC9 | §Integration Points | Backlog | GitHub issues |

## Risks, Assumptions, Open Questions

### Risks

| ID | Risk | Likelihood | Impact | Mitigation | Story |
|----|------|------------|--------|------------|-------|
| R1 | Performance targets not met after optimization | Low | High | Early profiling in Story 7.2; iterative optimization in 7.3; targets have 20% margin | 7.2, 7.3 |
| R2 | POC integration rejected by target project | Medium | Medium | Multiple targets (yazi, bat); focus on demonstrating value; fork if needed | 7.7 |
| R3 | crates.io name collision | Low | High | Verify name availability early; have backup names (dotmax-braille) | 7.6 |
| R4 | Breaking API changes discovered during POC | Medium | Medium | POC feedback loop before 1.0; semver allows breaking changes in 0.x | 7.7 |
| R5 | CI benchmark flakiness | Medium | Low | Use statistical analysis (Criterion); run multiple iterations; dedicated hardware | 7.2 |
| R6 | Documentation becomes stale | Low | Medium | Doctests enforce example correctness; CI checks docs build | 7.5 |
| R7 | Platform-specific issues in release | Low | High | Full CI matrix (Windows, Linux, macOS); MSRV testing | 7.6 |

### Assumptions

| ID | Assumption | Validation | Impact if Wrong |
|----|------------|------------|-----------------|
| A1 | Current test suite (557+ tests) provides adequate base coverage | Coverage reporting in 7.4 | Additional test writing needed |
| A2 | Existing benchmarks (rendering, color, animation) are representative | Profiling in 7.3 | New benchmarks needed |
| A3 | API surface from Epics 1-6 is stable and well-designed | API review in 7.1 | Refactoring before publication |
| A4 | yazi or bat maintainers are receptive to braille preview feature | Early outreach before 7.7 | Choose alternative target |
| A5 | docs.rs will build documentation without issues | Dry-run in 7.6 | Fix doc generation issues |
| A6 | Performance targets (<25ms, 60fps) are already met | Benchmark validation in 7.2 | Optimization work in 7.3 expands |
| A7 | No security advisories exist for current dependencies | `cargo audit` in 7.6 | Dependency updates needed |

### Open Questions

| ID | Question | Owner | Due | Resolution Path |
|----|----------|-------|-----|-----------------|
| Q1 | Should we add `async` feature flag for post-1.0? | Frosty | Story 7.1 | Document decision in ADR; defer implementation |
| Q2 | Which POC target (yazi vs bat) has better maintainer responsiveness? | Frosty | Before 7.7 | Research GitHub activity, Discord presence |
| Q3 | Should benchmarks run on every PR or only on main? | Frosty | Story 7.2 | Balance CI cost vs regression detection |
| Q4 | What is the minimum acceptable coverage threshold? | Frosty | Story 7.4 | 70% overall proposed; validate feasibility |
| Q5 | Should we create GIF demos for README or use static images? | Frosty | Story 7.5 | GIFs more compelling but larger; consider both |
| Q6 | Version 0.1.0 vs 1.0.0 for initial release? | Frosty | Story 7.6 | 0.1.0 allows breaking changes; recommended |
| Q7 | Should property tests use `proptest` or `quickcheck`? | Dev | Story 7.4 | `proptest` preferred (better shrinking, strategies) |

## Test Strategy Summary

### Test Levels

| Level | Scope | Tools | Coverage Target | Story |
|-------|-------|-------|-----------------|-------|
| **Unit** | Individual functions/modules | `#[test]`, `assert!` | >80% for core | 7.4 |
| **Integration** | Cross-module pipelines | `tests/` directory | Full pipeline coverage | 7.4 |
| **Property** | Invariant validation | `proptest` | Grid ops, color conversion | 7.4 |
| **Visual Regression** | Rendering correctness | Baseline comparison | Known image outputs | 7.4 |
| **Performance** | Timing validation | `criterion` | All NFR-P targets | 7.2 |
| **Documentation** | Example correctness | `cargo test --doc` | 100% doctest pass | 7.5 |

### Test Categories

**Story 7.4 Test Matrix:**

| Category | Files | Test Count (Target) | Priority |
|----------|-------|---------------------|----------|
| Grid operations | `src/grid.rs` | 50+ | Critical |
| Terminal rendering | `src/render.rs` | 20+ | Critical |
| Image pipeline | `src/image/*.rs` | 40+ | High |
| Drawing primitives | `src/primitives.rs` | 30+ | High |
| Color system | `src/color/*.rs` | 40+ | High |
| Animation | `src/animation/*.rs` | 30+ | High |
| Error handling | `src/error.rs` | 10+ | Medium |
| Integration | `tests/*.rs` | 20+ | High |
| Property-based | `tests/property_tests.rs` | 15+ | Medium |
| Visual regression | `tests/visual/*.rs` | 10+ | Medium |

### Test Frameworks

```toml
[dev-dependencies]
# Existing
criterion = { version = "0.7", features = ["html_reports"] }
tracing-subscriber = "0.3"
tempfile = "3.10"

# New for Epic 7
proptest = "1.4"  # Property-based testing
```

### CI Test Pipeline

```yaml
# .github/workflows/ci.yml (enhanced for Epic 7)
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, 1.70]  # stable + MSRV
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      - run: cargo test --all-features
      - run: cargo test --no-default-features
      - run: cargo clippy --all-features -- -D warnings
      - run: cargo doc --no-deps --all-features

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@cargo-tarpaulin
      - run: cargo tarpaulin --out Xml --all-features
      - uses: codecov/codecov-action@v3

  benchmark:
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo bench --all-features
      - uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion/
```

### Acceptance Criteria Coverage

| Story | ACs | Test Approach |
|-------|-----|---------------|
| 7.1 | 9 | Manual review + compile tests + `cargo doc` |
| 7.2 | 9 | `cargo bench` + CI workflow validation |
| 7.3 | 9 | Benchmark comparison + profiler output |
| 7.4 | 9 | `cargo test` + `cargo tarpaulin` + CI artifacts |
| 7.5 | 9 | `cargo doc` + `cargo test --doc` + manual review |
| 7.6 | 9 | `cargo publish --dry-run` + CI workflow + post-release verification |
| 7.7 | 9 | Manual integration test + code review + feedback collection |

### Edge Cases and Boundary Testing

| Area | Edge Cases to Test |
|------|--------------------|
| Grid | Zero dimensions, max dimensions (10,000×10,000), single cell |
| Image | Corrupt files, unsupported formats, zero-byte files, huge images |
| Color | Edge RGB values (0,0,0), (255,255,255), out-of-gamut |
| Animation | 0 FPS, very high FPS (1000+), single frame, empty frames |
| Terminal | No Unicode support, no color support, resize to 1×1 |

### Definition of Done (Testing)

- [ ] All unit tests pass (`cargo test`)
- [ ] All integration tests pass (`cargo test --test '*'`)
- [ ] All doctests pass (`cargo test --doc`)
- [ ] Property tests pass (`cargo test` with proptest)
- [ ] Coverage > 70% overall, > 80% core modules
- [ ] Zero clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Benchmarks complete without errors
- [ ] CI pipeline green on all platforms
