# dotmax - Product Requirements Document

**Author:** Frosty
**Date:** 2025-11-14
**Version:** 1.0

---

## Executive Summary

**Dotmax** is a high-performance Rust library that brings rich media capabilities to any terminal through braille-based character rendering. By extracting and professionalizing the proven braille rendering system from the crabmusic project, dotmax enables developers to add images, animations, video playback, and 3D visualization to their terminal applications with a simple `cargo add dotmax`.

The project addresses the fundamental limitation of text-only terminals by leveraging braille characters' 2×4 dot matrix structure to achieve 4× the effective resolution of ASCII art, while maintaining universal compatibility across all terminal environments. Unlike terminal-specific graphics protocols (Sixel, Kitty, iTerm2), dotmax's text-based approach works everywhere - from PowerShell to SSH sessions to embedded systems.

Dotmax serves two primary markets: (1) Rust CLI tool developers seeking to enhance their applications with visual capabilities, and (2) AI coding assistants (Claude Code, GitHub Copilot CLI, etc.) through MCP (Model Context Protocol) server integration, enabling AI agents to generate and manipulate visual content directly in the terminal.

### What Makes This Special

**"A window to discovery"** - Dotmax extends artistic elements to the terminal medium, opening new and novel experiences. When AI agents can intelligently manipulate the system through image generation or even 3D worlds, the terminal transforms from a text-only interface into a canvas for creativity, better data presentation, and enhanced branding elements. This library represents the foundation for AI-driven visual terminal experiences.

**Core Innovation:**
- **4× Resolution Advantage**: Braille characters provide 2×4 dot matrix per terminal cell vs. 1 pixel per character in ASCII
- **Universal Compatibility**: Text-based rendering works on ANY terminal (no protocol support required)
- **Proven Superior Quality**: Outputs demonstrably better than existing solutions (drawille, Python-based alternatives)
- **Comprehensive Media Support**: Images + Video + 3D Raytracer + Animations in a single unified library
- **Performance**: Rust-native, memory-safe, optimized for real-time rendering

---

## Project Classification

**Technical Type:** Rust Library (Crate) + Developer Tools
**Domain:** Terminal Graphics / Developer Tooling / Graphics Rendering
**Complexity:** Medium-High
**Project Nature:** Brownfield Extraction + Greenfield Packaging

This is a **brownfield extraction** project. Dotmax extracts the battle-tested braille rendering system (~2,000-3,000 lines) from the crabmusic audio visualization project, where it has proven exceptional output quality. The extraction transforms working-but-unfocused "vibe coded" components into a professional, focused, and maintainable Rust crate with clean APIs, comprehensive documentation, and production-ready quality.

**Key Technical Characteristics:**
- **Language**: Rust (modern systems programming language)
- **Distribution**: crates.io (Rust package registry)
- **Target Platforms**: Desktop terminals (Windows, Linux, macOS) - primary; Embedded systems (Arduino + display) - future
- **Integration Model**: Library crate (`cargo add dotmax`) + future MCP server for AI tool integration
- **Dependencies**: Minimal core (ratatui/crossterm for terminal abstraction), optional feature-flagged dependencies (image, video, 3D)

{{#if domain_context_summary}}

### Domain Context

{{domain_context_summary}}
{{/if}}

---

## Success Criteria

### Primary Success Factors (Critical)

**1. Performance Excellence** ⚡ **HIGHEST PRIORITY - AGGRESSIVE TARGETS**

Dotmax is foundational technology for an entire suite of future products (loading bars, git animation tools, terminal utilities). **Performance is make-or-break** - if dotmax doesn't meet industry standards, it fails. The goal: **efficiency so fast, it's invisible**.

**Adoption depends entirely on performance**: Developers will only integrate dotmax if memory and CPU usage are negligible. Any bloat reduces experience quality and kills adoption. We must meet **AND BEAT** existing performance benchmarks wherever possible.

**Performance Targets:**

- **Image Rendering**: <50ms for standard terminals (80×24), <100ms for large terminals (200×50) - **Target: <25ms** (beat competitors)
- **Animation Playback**: 60 fps minimum, targeting 120 fps capability on modern hardware
- **Video Playback**: Real-time 30fps for standard resolution without frame drops - **Target: 60fps capability**
- **Memory Footprint**:
  - Core library: <5MB baseline memory usage
  - Per-frame overhead: <500KB for typical renders
  - Zero memory leaks (valgrind/MIRI validated)
  - Bounded allocations (no unbounded growth)
- **CPU Usage**:
  - Idle/static renders: <1% CPU on modern hardware
  - Active animation: <10% single-core usage at 60fps
  - Efficient multi-threading where beneficial
- **Startup Time**: <5ms library initialization (cold start)
- **Binary Size**: Core library adds <2MB to compiled binaries (feature flags minimize bloat)

**Validation Method**:
- Comprehensive benchmark suite (criterion.rs) with CI/CD performance regression testing
- Competitive benchmarking against drawille, ascii-image-converter, and terminal graphics protocols
- Memory profiling (heaptrack, valgrind) in CI pipeline
- CPU profiling (perf, flamegraph) for hotspot identification
- **Numbers guide ALL optimization decisions** - No premature optimization, but aggressive optimization where benchmarks demand it

**2. Production-Ready Quality**

- Clean extraction from crabmusic with zero audio dependencies
- Visual output quality matches or exceeds crabmusic baseline (proven superior to drawille)
- Cross-platform compatibility: Windows, Linux, macOS work without modification
- API usability: Developers can integrate rich media in <100 lines of code
- Zero breaking changes after 1.0 (semantic versioning commitment)

**3. Resumable Development**

- Documentation quality enables pickup after months/years gap
- Clear architecture decision records (ADRs)
- Modular milestones allowing pause between phases
- Code structure supports solo long-term maintenance

**4. External Validation**

- Published to crates.io as professional Rust crate
- At least ONE proof-of-concept integration (yazi, bat, or similar Rust CLI tool)
- API tested by external developer (even if just one person)

### Secondary Success Indicators (Tracking Only)

These metrics are tracked but **not** success criteria for technical work:

- **Community Adoption**: crates.io downloads, GitHub stars/forks
- **Integration Reach**: Number of projects using dotmax
- **Feedback Volume**: Issues, PRs, community engagement

**Philosophy**: "I can only do my best, I cannot demand the world respond to my work." These metrics measure market response, not technical quality.

### Long-Term Vision (Post-MVP)

- **Foundation for Product Suite**: Dotmax successfully powers loading bar tools, git animation tools, and other terminal utilities
- **MCP Integration**: MCP server enables AI tool adoption (Claude Code, GitHub Copilot CLI, etc.)
- **Community Standard**: Dotmax becomes the de facto Rust solution for terminal graphics
- **Sustainable Maintenance**: Library can be maintained solo or community-driven when needed

---

## Product Scope

### MVP - Minimum Viable Product (v0.1.0 → v1.0)

**Core Extraction & Foundation:**

The MVP extracts and professionalizes the battle-tested braille rendering system from crabmusic, delivering a production-ready Rust crate that developers can integrate with `cargo add dotmax`.

**Essential Capabilities:**

1. **Core Rendering Engine**
   - BrailleGrid system (~500 lines) - 2×4 dot matrix rendering
   - GridBuffer - character grid abstraction (~200 lines)
   - TerminalRenderer - ratatui/crossterm integration (~450 lines)
   - Color utilities and RGB support (~20 lines)

2. **Rich Media - 2D (Core)** (MINIMUM REQUIRED)
   - **Image-to-Braille Conversion** - Core functionality, not optional
   - **Industry Standard Formats**: PNG, JPG, GIF, BMP, WebP, TIFF
   - **Vector Format**: SVG import and rasterization (core capability)
   - Otsu thresholding and dithering algorithms
   - Resize/scale to terminal dimensions
   - Color and monochrome rendering modes
   - Aspect ratio preservation
   - Multiple dithering methods (Floyd-Steinberg, Bayer, Atkinson)

3. **Drawing Primitives**
   - Bresenham line and circle algorithms (~100 lines)
   - Basic shapes (rectangles, filled polygons)
   - Character density-based rendering (~400 lines)
   - Color schemes for intensity mapping (~150 lines)

4. **Developer Experience**
   - Clean, minimal API surface
   - Feature flags for optional capabilities (video, raytrace as opt-in)
   - Comprehensive API documentation (rustdoc)
   - Examples directory with common use cases
   - Integration requires <100 lines of code

5. **Quality & Performance**
   - Cross-platform: Windows, Linux, macOS
   - Performance benchmarks (criterion.rs)
   - Memory profiling validated
   - Zero audio dependencies (clean extraction)
   - Visual output quality matches/exceeds crabmusic baseline

6. **Distribution & Validation**
   - Published to crates.io
   - MIT or Apache-2.0 license (not AGPLv3)
   - At least ONE proof-of-concept integration (yazi or bat)
   - Resumable documentation (ADRs, architecture docs)

**What's Included in MVP:**
- Core: ~2,000-3,000 lines from crabmusic extraction
- **Rich Media 2D**: All industry-standard raster formats (PNG, JPG, GIF, BMP, WebP, TIFF) + SVG vector
- Basic animations: Simple frame-by-frame rendering
- Test suite and benchmarks
- README with visual examples

**What's Explicitly Excluded from MVP:**
- Video playback (FFmpeg dependency - Phase 2: Rich Media 2D++)
- 3D raytracer (Phase 3: Rich Media 3D)
- Audio-reactive features (stays in crabmusic)
- Effects pipeline (crabmusic-specific)
- Configuration system (crabmusic-specific)
- MCP server (Phase 2B)
- Language bindings (Phase 2C)
- Embedded systems support (future)

### Growth Features (Post-v1.0)

**Phase 2A: Rich Media 2D++ (Video & Advanced Animation)**
- **Video Playback Support** (feature flag: `video`)
  - FFmpeg integration for frame extraction
  - Real-time playback at 30-60fps
  - Frame caching and buffering
  - Multiple codec support (H.264, VP9, AV1)
- **Advanced Animation**
  - Easing functions
  - Sprite systems
  - Frame interpolation
  - GIF/APNG animation support
  - Timeline-based animation API

**Phase 2B: AI Tool Integration (MCP Server)**
- **MCP Server** (`dotmax-mcp-server`)
  - JSON-RPC protocol implementation
  - Tool definitions: render_image, render_video, render_animation
  - Claude Code integration
  - GitHub Copilot CLI integration
  - Universal AI tool compatibility

**Phase 2C: Developer Ecosystem**
- Integration examples for popular Rust CLI tools
- Tutorial documentation site (beyond rustdoc)
- Blog posts / showcase demos
- Community outreach (Reddit, Rust forums)

**Phase 3: Rich Media 3D (Raytracing & 3D Graphics)**
- **3D Raytracer** (feature flag: `raytrace`)
  - Extract from crabmusic (~1,500 lines)
  - OBJ/glTF model loading
  - Sphere and mesh rendering
  - Camera controls and lighting
  - Material system (PBR basics)
- **Real-time 3D Rendering**
  - Wireframe mode
  - Simple shading models
  - Depth-based braille intensity

### Vision Features (Long-term / Future)

**Platform Expansion:**
- **Embedded Systems**: Arduino + display support (memory-optimized)
- **WASM Target**: Web terminal compatibility
- **Mobile Terminals**: Termux, iSH support

**Product Suite Foundation:**
- Loading bar tools (using dotmax)
- Git animation tools (using dotmax)
- Terminal utility suite (all powered by dotmax core)

**Advanced Capabilities:**
- Vector graphics import (SVG rendering)
- Advanced effects pipeline extraction
- Hardware acceleration exploration (GPU)
- Plugin/extension system

**Language Bindings** (if MCP insufficient):
- Node.js bindings (napi-rs)
- Python bindings (PyO3)
- Direct embedding vs. MCP service calls

**Community & Sustainability:**
- Dotmax becomes de facto Rust terminal graphics standard
- Community contributions and maintenance
- Corporate sponsorship or adoption

---

{{#if domain_considerations}}

## Domain-Specific Requirements

{{domain_considerations}}

This section shapes all functional and non-functional requirements below.
{{/if}}

---

{{#if innovation_patterns}}

## Innovation & Novel Patterns

{{innovation_patterns}}

### Validation Approach

{{validation_approach}}
{{/if}}

---

{{#if project_type_requirements}}

## {{project_type}} Specific Requirements

{{project_type_requirements}}

{{#if endpoint_specification}}

### API Specification

{{endpoint_specification}}
{{/if}}

{{#if authentication_model}}

### Authentication & Authorization

{{authentication_model}}
{{/if}}

{{#if platform_requirements}}

### Platform Support

{{platform_requirements}}
{{/if}}

{{#if device_features}}

### Device Capabilities

{{device_features}}
{{/if}}

{{#if tenant_model}}

### Multi-Tenancy Architecture

{{tenant_model}}
{{/if}}

{{#if permission_matrix}}

### Permissions & Roles

{{permission_matrix}}
{{/if}}
{{/if}}

---

{{#if ux_principles}}

## User Experience Principles

{{ux_principles}}

### Key Interactions

{{key_interactions}}
{{/if}}

---

## Functional Requirements

This section defines WHAT capabilities dotmax must have. These FRs are the complete inventory of features - if a capability is not listed here, it will NOT exist in the final product. UX designers, architects, and epic breakdown all rely on this as the authoritative capability list.

### Core Rendering (BrailleGrid System)

**FR1**: Developers can create a BrailleGrid with specified dimensions (width × height in braille cells)

**FR2**: Developers can set individual dots within the braille grid (2×4 dot matrix per cell)

**FR3**: Developers can clear the entire grid or specific regions

**FR4**: Developers can render the grid to terminal output via standard terminal abstractions (ratatui/crossterm)

**FR5**: The system converts braille dot patterns to Unicode braille characters (U+2800 to U+28FF range)

**FR6**: Developers can query grid state (get dot values, dimensions, buffer contents)

**FR7**: The system handles terminal resize events and adjusts grid dimensions accordingly

**FR8**: Developers can create grids with color support (per-cell color assignment)

### 2D Image Rendering

**FR9**: Developers can load and render images from file paths (PNG, JPG, GIF, BMP, WebP, TIFF formats)

**FR10**: Developers can load and render images from byte buffers (in-memory image data)

**FR11**: Developers can load and render SVG vector graphics with rasterization to braille grid

**FR12**: The system automatically resizes images to fit terminal dimensions while preserving aspect ratio

**FR13**: Developers can specify target dimensions for image rendering (override auto-sizing)

**FR14**: The system converts grayscale/color images to braille dot patterns using threshold algorithms

**FR15**: Developers can select dithering method (Floyd-Steinberg, Bayer, Atkinson, or none)

**FR16**: Developers can render images in monochrome mode (black & white only)

**FR17**: Developers can render images in color mode (using terminal color capabilities)

**FR18**: The system applies Otsu thresholding for optimal binary conversion

**FR19**: Developers can adjust brightness/contrast/gamma of images before rendering

**FR20**: The system handles malformed or unsupported image files gracefully (error handling)

### Drawing Primitives

**FR21**: Developers can draw lines between two points using Bresenham algorithm

**FR22**: Developers can draw circles with specified center and radius using Bresenham algorithm

**FR23**: Developers can draw rectangles (outline or filled)

**FR24**: Developers can draw polygons from a list of vertices

**FR25**: Developers can fill regions with solid patterns or density-based characters

**FR26**: Developers can set line thickness for drawing operations

**FR27**: Developers can set color for drawing operations (if color mode enabled)

### Character Density Rendering

**FR28**: The system maps intensity values (0.0 to 1.0) to character densities (sparse to dense)

**FR29**: Developers can use predefined character density sets for ASCII-art style rendering

**FR30**: Developers can customize character density mappings

**FR31**: The system provides smooth gradients through density character selection

### Color Support

**FR32**: Developers can assign RGB colors to individual braille cells

**FR33**: The system converts RGB values to terminal-compatible color codes (ANSI 256-color or true color)

**FR34**: Developers can select from predefined color schemes (6+ built-in schemes from crabmusic)

**FR35**: Developers can create custom color schemes with intensity-to-color mappings

**FR36**: The system applies color schemes to grayscale intensity buffers

**FR37**: Developers can query terminal color capabilities and adjust rendering accordingly

### Animation & Frame Management

**FR38**: Developers can render frame-by-frame animations by updating grid contents

**FR39**: The system supports frame timing control for animation playback

**FR40**: Developers can create animation loops with specified frame rates

**FR41**: The system provides frame buffer management for smooth animation transitions

**FR42**: Developers can pre-render animation frames for optimized playback

**FR43**: The system clears previous frames between animation updates (flicker-free rendering)

### API Design & Integration

**FR44**: Developers can integrate dotmax with <100 lines of code for basic use cases

**FR45**: The API follows Rust idioms (ownership, borrowing, error handling via Result types)

**FR46**: The system provides builder patterns for complex configuration

**FR47**: Developers can use dotmax with async/await patterns (async-compatible API)

**FR48**: The system provides examples for common integration scenarios (image viewer, animation player, etc.)

**FR49**: Developers can enable optional features via Cargo feature flags (image, video, raytrace)

**FR50**: The API surface is minimal and focused (no bloat, clear purpose for each method)

### Terminal Abstraction

**FR51**: The system works with ratatui/crossterm for terminal I/O

**FR52**: Developers can provide custom terminal backends (abstraction layer exists)

**FR53**: The system detects terminal capabilities (size, color support, Unicode support)

**FR54**: The system handles terminal environments without braille support gracefully (fallback behavior)

**FR55**: The system works in standard terminals (PowerShell, bash, zsh, fish, etc.)

### Error Handling & Robustness

**FR56**: All operations return Result types with meaningful error messages

**FR57**: The system never panics in library code (all panics are bugs)

**FR58**: Developers can query error details for debugging (error context preserved)

**FR59**: The system handles edge cases (zero-size grids, out-of-bounds access, invalid formats) safely

**FR60**: The system provides debug/trace logging for troubleshooting (via log crate or similar)

### Library Distribution & Packaging

**FR61**: Developers can install dotmax via `cargo add dotmax` from crates.io

**FR62**: The system minimizes binary size impact (<2MB addition to compiled binaries)

**FR63**: Core library has minimal dependencies (ratatui, crossterm, essential only)

**FR64**: Optional features are behind Cargo feature flags (image, video, raytrace)

**FR65**: The system compiles cleanly on stable Rust (no nightly features required)

**FR66**: The system provides API documentation via rustdoc with examples

**FR67**: The system includes examples/ directory with runnable demonstrations

### Performance & Efficiency

**FR68**: The system renders images to braille in <50ms for standard terminals (80×24)

**FR69**: The system renders images to braille in <100ms for large terminals (200×50)

**FR70**: The system achieves 60fps animation playback with <10% CPU usage

**FR71**: The system uses <5MB baseline memory for core operations

**FR72**: The system uses <500KB additional memory per typical rendered frame

**FR73**: The system releases memory promptly (no unbounded growth)

**FR74**: The system initializes in <5ms (cold start library load)

### Cross-Platform Compatibility

**FR75**: The system works on Windows without modification

**FR76**: The system works on Linux without modification

**FR77**: The system works on macOS without modification

**FR78**: The system handles platform-specific terminal quirks (color codes, escape sequences)

**FR79**: The system provides consistent visual output across platforms

**FR80**: The system detects and adapts to platform-specific terminal capabilities

### Testing & Validation

**FR81**: The system includes unit tests for all core rendering functions

**FR82**: The system includes integration tests for image loading and rendering

**FR83**: The system includes benchmark tests (criterion.rs) for performance validation

**FR84**: The system includes visual regression tests (save output, compare against baseline)

**FR85**: The system includes property-based tests for mathematical correctness (proptest or similar)

### Documentation & Developer Experience

**FR86**: The system provides comprehensive API documentation with examples

**FR87**: The system includes quickstart guide in README

**FR88**: The system provides migration guide from similar libraries (drawille, etc.)

**FR89**: The system includes architecture decision records (ADRs) for major design choices

**FR90**: The system includes troubleshooting guide for common issues

---

## Non-Functional Requirements

These define HOW dotmax must perform, not WHAT it does. NFRs specify quality attributes critical for adoption and long-term success.

### Performance (CRITICAL - Make or Break)

**Performance is the highest priority NFR.** Dotmax must be "efficiency so fast, it's invisible" or it fails.

**NFR-P1: Rendering Latency**
- Image-to-braille conversion: <25ms target, <50ms maximum for standard terminals (80×24)
- Large terminal rendering (200×50): <100ms maximum
- SVG rasterization: <100ms for typical graphics
- Must beat or match competitor performance (drawille, ascii-image-converter)

**NFR-P2: Animation Performance**
- Sustained 60fps minimum with <10% single-core CPU usage on modern hardware (2020+ desktop)
- Target capability: 120fps on modern hardware
- No dropped frames during sustained playback (30+ seconds)
- Frame buffer management adds <5ms overhead

**NFR-P3: Memory Efficiency**
- Baseline memory footprint: <5MB for core library operations
- Per-frame memory overhead: <500KB for typical renders
- Zero memory leaks (validated by valgrind/MIRI in CI)
- Bounded memory allocations (no unbounded growth patterns)
- Efficient memory reuse (object pooling where beneficial)

**NFR-P4: CPU Efficiency**
- Idle/static renders: <1% CPU usage on modern hardware
- Active animation at 60fps: <10% single-core usage
- Multi-threading where beneficial (image processing, batch operations)
- SIMD optimization exploration for hot paths

**NFR-P5: Startup Performance**
- Library initialization (cold start): <5ms
- First render after initialization: <10ms additional
- No blocking I/O during initialization

**NFR-P6: Binary Size Impact**
- Core library adds <2MB to compiled binary size
- Feature flags minimize bloat (opt-in for image/video/raytrace)
- Link-time optimization (LTO) compatible

**NFR-P7: Competitive Benchmarking**
- Continuous benchmarking against drawille (Python), ascii-image-converter (Go)
- Performance regression detection in CI pipeline
- Public benchmark results for transparency

### Reliability & Stability

**NFR-R1: Error Handling**
- Zero panics in library code (all panics are bugs requiring immediate fix)
- All operations return Result types with actionable error messages
- Graceful degradation for unsupported features (fallback behavior)

**NFR-R2: Cross-Platform Consistency**
- Identical behavior on Windows, Linux, macOS
- Platform-specific quirks isolated to platform abstraction layer
- Automated cross-platform testing in CI (GitHub Actions: Windows, Linux, macOS)

**NFR-R3: Robustness**
- Handles malformed inputs safely (corrupted images, invalid formats)
- Handles edge cases (zero-size grids, extreme dimensions, out-of-bounds access)
- No undefined behavior (validated by MIRI for unsafe code blocks if any)

**NFR-R4: Long-Running Stability**
- No memory leaks during sustained operation (hours/days)
- No performance degradation over time
- Resource cleanup on drop (RAII patterns)

### Maintainability & Resumability

**NFR-M1: Solo Developer Sustainability**
- Code must be maintainable by one person long-term
- Clear module boundaries and separation of concerns
- Minimal external dependencies to reduce upstream breakage risk

**NFR-M2: Resumable Development**
- Documentation quality enables project pickup after months/years gap
- Architecture Decision Records (ADRs) document major design choices
- Comprehensive inline documentation (rustdoc comments)
- Clear TODO/FIXME markers with context

**NFR-M3: Code Quality**
- Rust idioms followed (ownership, borrowing, explicit lifetimes)
- No compiler warnings on stable Rust
- Clippy lints enforced (pedantic level where reasonable)
- Formatted with rustfmt (CI enforced)

**NFR-M4: Test Coverage**
- Core rendering: >80% line coverage
- Critical paths: 100% coverage (braille conversion, grid operations)
- Integration tests for all public APIs
- Visual regression tests for output validation

### Dependency Management

**NFR-D1: Minimal Core Dependencies**
- Core library: <10 direct dependencies
- Each dependency justified and documented
- Prefer std library over external crates where reasonable

**NFR-D2: Upstream Risk Mitigation**
- Pin major versions in Cargo.toml
- Monitor dependency security advisories (cargo-audit in CI)
- Abstract terminal backend (reduce ratatui/crossterm lock-in risk)
- Document dependency migration paths if upstream breaks

**NFR-D3: Feature Flag Discipline**
- Core rendering has zero optional dependencies
- Image support: opt-in via `image` feature flag
- Video support: opt-in via `video` feature flag (Phase 2)
- Raytrace support: opt-in via `raytrace` feature flag (Phase 3)

### Developer Experience

**NFR-DX1: API Simplicity**
- Basic integration requires <100 lines of code
- API follows principle of least surprise
- Common tasks are trivial, complex tasks are possible

**NFR-DX2: Documentation Quality**
- Every public API has rustdoc with examples
- Examples compile and run (tested in CI)
- Quickstart guide in README (<5 minutes to first render)
- Migration guides from similar libraries (drawille)

**NFR-DX3: Build Experience**
- Clean build from `cargo build --release`
- Compiles on stable Rust (no nightly required)
- Build time: <2 minutes for full project on modern hardware
- Incremental builds: <10 seconds for single-file changes

**NFR-DX4: Error Messages**
- Actionable error messages (explain what went wrong and how to fix)
- Error context preserved through error chains
- Debug mode provides detailed diagnostics

### Compatibility & Standards

**NFR-C1: Rust Version Policy**
- Minimum Supported Rust Version (MSRV): Rust 1.70+ (or latest stable - 6 months)
- MSRV documented in README and enforced in CI
- MSRV only bumped with minor version increments (not patches)

**NFR-C2: Semantic Versioning**
- Strict semver compliance (breaking changes = major version bump)
- 1.0.0 = stable API, no breaking changes in 1.x line
- Pre-1.0: minor version bumps may break compatibility (documented)

**NFR-C3: Terminal Compatibility**
- Works in standard terminals: PowerShell, bash, zsh, fish, iTerm2, Windows Terminal, Alacritty
- Detects and adapts to terminal capabilities (color depth, Unicode support)
- Graceful fallback for limited terminals

**NFR-C4: Unicode Braille Support**
- Assumes terminal supports Unicode braille range (U+2800 to U+28FF)
- Detects lack of braille support and warns user (doesn't crash)
- Future: ASCII fallback mode (not MVP)

### Licensing & Distribution

**NFR-L1: Open Source License**
- MIT or Apache-2.0 dual licensing (NOT AGPLv3)
- Maximizes adoption potential
- Corporate-friendly licensing

**NFR-L2: Distribution**
- Published to crates.io as canonical source
- Source code on GitHub (public repository)
- Releases follow semantic versioning
- Changelog maintained (CHANGELOG.md)

**NFR-L3: Dependency Licenses**
- All dependencies use permissive licenses (MIT, Apache-2.0, BSD)
- No viral licenses (GPL, AGPL) in dependency tree
- License compatibility verified before adding dependencies

### Security (Minimal for Graphics Library)

**NFR-S1: Memory Safety**
- Zero unsafe code in MVP (Rust memory safety guarantees)
- If unsafe required: isolated, audited, documented, MIRI-validated
- No buffer overflows or out-of-bounds access

**NFR-S2: Input Validation**
- All external inputs validated (file paths, image data, dimensions)
- No arbitrary code execution through malformed inputs
- Resource limits enforced (max image size, max grid dimensions)

**NFR-S3: Dependency Security**
- cargo-audit in CI pipeline (detect known vulnerabilities)
- Dependencies reviewed before adoption
- Security advisories monitored and patched promptly

---

## Implementation Planning

### Epic Breakdown Required

Requirements must be decomposed into epics and bite-sized stories (200k context limit).

**Next Step:** Run `workflow epics-stories` to create the implementation breakdown.

---

## References

### Research & Planning Documents

- **Technical Research**: `docs/research-technical-2025-11-14.md` - Comprehensive AI CLI tool integration research, competitive analysis (drawille, Sixel/Kitty protocols), MCP server discovery, Rust CLI tool targets (yazi, bat), and three-phase hybrid deployment model recommendation

- **Brainstorming Session**: `docs/bmm-brainstorming-session-2025-11-14.md` - Mind mapping, assumption reversal, chaos engineering, and first principles thinking applied to dotmax extraction strategy, market positioning, and implementation sequencing

- **Source Repository**: https://github.com/newjordan/crabmusic - Working implementation with proven braille rendering system (~2,000-3,000 lines to extract)

### Key Technical References

- **Braille Unicode Range**: U+2800 to U+28FF (256 braille patterns for 2×4 dot matrix)
- **Competitive Alternatives**: drawille (Python, AGPLv3), ascii-image-converter (Go), python-termgraphics
- **Terminal Graphics Protocols**: Sixel (1980s, limited color), Kitty Graphics Protocol (24-bit, doesn't work in tmux), iTerm2 Protocol
- **Integration Targets**: Claude Code (TypeScript/Bun), GitHub Copilot CLI (npm), Zed (Rust), yazi (Rust file manager), bat (Rust file viewer)
- **MCP (Model Context Protocol)**: https://spec.modelcontextprotocol.io/ - JSON-RPC protocol for AI tool extensibility

---

## Next Steps

### Immediate: Epic & Story Breakdown

**Run**: `/bmad:bmm:workflows:create-epics-and-stories` or manual epic creation

This PRD must be decomposed into implementable epics and bite-sized stories. Recommended epic structure:

1. **Epic 1: Core Extraction** - Extract BrailleGrid, GridBuffer, TerminalRenderer from crabmusic
2. **Epic 2: Rich Media 2D** - Image loading (PNG/JPG/GIF/BMP/WebP/TIFF), SVG support, dithering
3. **Epic 3: Drawing & Primitives** - Lines, circles, shapes, density rendering
4. **Epic 4: Color & Schemes** - RGB support, terminal color mapping, predefined schemes
5. **Epic 5: Animation Support** - Frame management, timing, buffer optimization
6. **Epic 6: API Design & DX** - Clean API surface, builder patterns, examples
7. **Epic 7: Performance & Benchmarking** - criterion.rs suite, profiling, optimization
8. **Epic 8: Cross-Platform Testing** - Windows/Linux/macOS CI, terminal compatibility
9. **Epic 9: Documentation** - rustdoc, README, examples, ADRs
10. **Epic 10: Distribution** - crates.io publish, licensing, POC integration (yazi/bat)

### Subsequent Workflows

**Architecture Design**: `/bmad:bmm:workflows:architecture`
- Technical architecture decisions
- Dependency management strategy
- Terminal abstraction layer design
- Performance optimization approach
- ADRs for major design choices

**UX Design**: Not applicable (library has no UI)

**Sprint Planning**: `/bmad:bmm:workflows:sprint-planning`
- After epics/stories created
- Phase 1 focus: Extraction + validation (Weeks 1-8)
- Phase 2 focus: Rich media expansion + MCP (Weeks 9-16+)

---

## Product Value Summary

**Dotmax is "a window to discovery"** - extending artistic elements to the terminal medium and opening new experiences for developers and AI agents alike.

**What Makes It Special:**
- **Performance so fast, it's invisible** - Aggressive optimization targeting <25ms renders, 60-120fps animation
- **Universal compatibility** - Text-based braille rendering works on ANY terminal, no protocol support required
- **Foundation for ecosystem** - Powers future product suite (loading bars, git animation tools, terminal utilities)
- **Proven superior quality** - Outputs demonstrably better than existing solutions (drawille, competitors)
- **4× resolution advantage** - Braille's 2×4 dot matrix vs. ASCII single-character rendering
- **AI-ready integration** - MCP server path enables Claude Code, GitHub Copilot CLI, and future AI tools

**Strategic Positioning**: Modern Rust alternative to fragmented Python/Go braille libraries, purpose-built for AI coding assistant integration and high-performance terminal graphics.

**Mission**: Transform terminals from text-only interfaces into canvases for creativity, data visualization, and AI-driven visual experiences.

---

_This PRD captures the complete requirements for the dotmax Rust library - from brownfield extraction through production deployment._

_Created through collaborative discovery between Frosty (artist/developer) and John (Product Manager AI), drawing on comprehensive technical research and systematic brainstorming._

_**Next Action**: Run epic breakdown workflow to decompose requirements into implementable stories._
