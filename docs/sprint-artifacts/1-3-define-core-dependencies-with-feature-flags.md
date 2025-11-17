# Story 1.3: Define Core Dependencies with Feature Flags

Status: ready-for-dev

## Story

As a **developer building with dotmax**,
I want core dependencies to be minimal (<10 total) with optional features behind feature flags,
so that my binary size stays under 2MB for the core library while enabling opt-in capabilities (image, SVG) when needed.

## Acceptance Criteria

1. `Cargo.toml` defines exactly 4 core dependencies (always included): `ratatui`, `crossterm`, `thiserror`, `tracing`
2. Optional dependencies are feature-gated: `image`, `imageproc` behind `image` feature; `resvg`, `usvg` behind `svg` feature
3. `[features]` section defines: `default = []`, `image = ["dep:image", "dep:imageproc"]`, `svg = ["dep:resvg", "dep:usvg"]`
4. `cargo build` (no features) compiles successfully with only 4 core dependencies
5. `cargo build --features image` adds image/imageproc dependencies
6. `cargo build --features svg` adds resvg/usvg dependencies
7. `cargo build --features image,svg` includes all optional dependencies
8. Binary size for core-only build is <2MB (measured with `cargo build --release`)
9. Dependency versions match Architecture Document specifications (ratatui 0.29, crossterm 0.29, thiserror 2.0, tracing 0.1, image 0.25, imageproc 0.24, resvg 0.38, usvg 0.38)
10. CI pipeline (from Story 1.2) passes for all feature combinations

## Tasks / Subtasks

- [ ] Task 1: Add core dependencies to Cargo.toml (AC: #1, #9)
  - [ ] Add `ratatui = "0.29"` to [dependencies]
  - [ ] Add `crossterm = "0.29"` to [dependencies]
  - [ ] Add `thiserror = "2.0"` to [dependencies]
  - [ ] Add `tracing = "0.1"` to [dependencies]
  - [ ] Verify exactly 4 core dependencies (no more, no less)

- [ ] Task 2: Add optional dependencies with feature gates (AC: #2, #9)
  - [ ] Add `image = { version = "0.25", optional = true }` to [dependencies]
  - [ ] Add `imageproc = { version = "0.24", optional = true }` to [dependencies]
  - [ ] Add `resvg = { version = "0.38", optional = true }` to [dependencies]
  - [ ] Add `usvg = { version = "0.38", optional = true }` to [dependencies]
  - [ ] Verify `optional = true` is set for all non-core dependencies

- [ ] Task 3: Define feature flags (AC: #3)
  - [ ] Add `[features]` section to Cargo.toml
  - [ ] Define `default = []` (no features enabled by default)
  - [ ] Define `image = ["dep:image", "dep:imageproc"]` feature
  - [ ] Define `svg = ["dep:resvg", "dep:usvg"]` feature
  - [ ] Verify feature syntax uses weak dependency features (`dep:`)

- [ ] Task 4: Add dev dependencies (AC: #9)
  - [ ] Add `criterion = { version = "0.7", features = ["html_reports"] }` to [dev-dependencies]
  - [ ] Add `tracing-subscriber = "0.3"` to [dev-dependencies]
  - [ ] Verify dev dependencies are separate from main dependencies

- [ ] Task 5: Test core-only build (AC: #4, #8)
  - [ ] Run `cargo clean` to clear previous builds
  - [ ] Run `cargo build --release` (no features)
  - [ ] Verify build succeeds
  - [ ] Measure binary size of `target/release/libdotmax.rlib` or compiled example
  - [ ] Confirm size is <2MB
  - [ ] Verify Cargo.lock shows only 4 core dependencies + transitive deps

- [ ] Task 6: Test feature-gated builds (AC: #5, #6, #7)
  - [ ] Run `cargo clean && cargo build --features image`
  - [ ] Verify image/imageproc are included in Cargo.lock
  - [ ] Run `cargo clean && cargo build --features svg`
  - [ ] Verify resvg/usvg are included in Cargo.lock
  - [ ] Run `cargo clean && cargo build --features image,svg`
  - [ ] Verify all optional dependencies are included
  - [ ] Confirm all feature combinations compile successfully

- [ ] Task 7: Validate CI with new dependencies (AC: #10)
  - [ ] Push changes to GitHub (trigger CI from Story 1.2)
  - [ ] Verify CI runs on all platforms (Windows, Linux, macOS)
  - [ ] Verify CI tests all feature combinations:
    - Core only (no features)
    - `--features image`
    - `--features svg`
    - `--features image,svg`
  - [ ] Confirm cargo audit passes (no vulnerabilities in new dependencies)
  - [ ] Verify build times are reasonable (<5 min with warm cache)

- [ ] Task 8: Document dependency justifications (AC: #9, implied)
  - [ ] Create or update `docs/dependencies.md`
  - [ ] Document why each core dependency is required
  - [ ] Document why each optional dependency is feature-gated
  - [ ] Include version pinning rationale (major version lock strategy)
  - [ ] Reference Architecture Document ADR 0003 (feature flag architecture)

## Dev Notes

### Learnings from Previous Story

**From Story 1.2 (Status: done)**

Story 1.2 established the CI/CD pipeline that will validate this story's dependency configuration across all platforms. The CI is already configured to test multiple feature combinations, which is critical for this story.

**CI Infrastructure Available:**
- **Cross-platform testing**: CI runs on Windows, Linux, macOS - ensures dependencies work on all platforms
- **Caching enabled**: Swatinem/rust-cache@v2 configured - will speed up builds with new dependencies
- **Security scanning**: cargo-audit runs on every push - will catch vulnerabilities in new dependencies we add
- **Multiple toolchains**: Stable + MSRV 1.70 tested - ensures dependency versions are MSRV-compatible

**Important Continuity:**
- MSRV 1.70 constraint: Must verify all dependencies support Rust 1.70 (check their Cargo.toml or docs)
- CI triggers on every push: Changes to Cargo.toml will automatically trigger full CI validation
- No dependencies added yet: This story is the first to add real dependencies - CI will become slower but caching should keep it <5 min
- Basic test exists in src/lib.rs: CI will validate dependencies compile correctly

**CI Workflow from 1.2 to leverage:**
- Test matrix job: Will build core + all feature combinations on all platforms
- cargo audit job: Will scan new dependencies for known vulnerabilities
- MSRV check job: Will validate all dependencies are compatible with Rust 1.70

[Source: docs/sprint-artifacts/1-2-configure-github-actions-cicd-pipeline.md#Dev-Agent-Record]

### Project Structure Notes

**Cargo.toml Structure Alignment:**
Following Architecture Document (Section: Technology Stack Details), the Cargo.toml must have this exact structure:

```toml
[package]
# (already exists from Story 1.1)

[dependencies]
# Core dependencies (always included) - Story 1.3 adds these
ratatui = "0.29"
crossterm = "0.29"
thiserror = "2.0"
tracing = "0.1"

# Optional dependencies (feature-gated) - Story 1.3 adds these
image = { version = "0.25", optional = true }
imageproc = { version = "0.24", optional = true }
resvg = { version = "0.38", optional = true }
usvg = { version = "0.38", optional = true }

[features]
default = []
image = ["dep:image", "dep:imageproc"]
svg = ["dep:resvg", "dep:usvg"]

[dev-dependencies]
criterion = { version = "0.7", features = ["html_reports"] }
tracing-subscriber = "0.3"
```

**Rationale for Each Dependency (from Architecture Document):**

| Dependency | Rationale | Source |
|------------|-----------|--------|
| **ratatui 0.29** | Industry standard for Rust TUIs. Provides cross-platform terminal abstraction. Epic 2 will use TerminalBackend trait built on ratatui. | docs/architecture.md#Decision-Summary |
| **crossterm 0.29** | Cross-platform terminal I/O (cursor, colors, events). Works with ratatui as backend. | docs/architecture.md#Decision-Summary |
| **thiserror 2.0** | Library users need typed errors for pattern matching (ImageLoadError vs RenderError). Minimal boilerplate. | docs/architecture.md#Decision-Summary, ADR 0002 |
| **tracing 0.1** | Structured logging standard for Rust. Instrument functions, multiple log levels. Critical for debugging Epic 2+ rendering pipeline. | docs/architecture.md#Decision-Summary |
| **image 0.25** | Standard Rust image library. Handles PNG/JPG/GIF/BMP/WebP/TIFF. Epic 3 will use for image loading. Feature-gated to avoid bloat. | docs/architecture.md#Decision-Summary |
| **imageproc 0.24** | Image processing (dithering algorithms, thresholding). Epic 3 will use for Floyd-Steinberg, Bayer, Atkinson dithering. | docs/architecture.md#Decision-Summary |
| **resvg 0.38** | SVG rasterization to bitmap. Epic 3 will use for vector graphics support. Feature-gated separately from raster images. | docs/architecture.md#Decision-Summary |
| **usvg 0.38** | SVG parsing. Required by resvg for SVG handling. | docs/architecture.md#Decision-Summary |
| **criterion 0.7** | Statistics-driven benchmarking with HTML reports. Story 1.6 will use for performance tracking. Only dev dependency. | docs/architecture.md#Decision-Summary |
| **tracing-subscriber 0.3** | Logging in tests and examples. Only dev dependency. | docs/architecture.md#Technology-Stack-Details |

[Source: docs/architecture.md#Technology-Stack-Details, #Decision-Summary]

### Architecture Patterns to Follow

**From Architecture Document:**

**1. Feature Flag Architecture (ADR 0003):**
- **Decision**: Core has zero optional dependencies. Image/SVG/video/3D are opt-in.
- **Consequences**: Core library stays minimal (<2MB binary size), users only pay for what they use, easier to add new features without bloating core.
- **Implementation**: Use Cargo `optional = true` syntax + `[features]` section with weak dependency features (`dep:image` syntax).

[Source: docs/architecture.md#ADR-0003-Feature-Flag-Architecture]

**2. Minimal Core Dependencies (NFR-D1):**
- **Requirement**: Core library <10 direct dependencies.
- **Target for MVP**: Exactly 4 core dependencies (ratatui, crossterm, thiserror, tracing).
- **Future**: Video/3D features (Epic 2A, 3) will add more optional dependencies, but core remains at 4.

[Source: docs/architecture.md#Non-Functional-Requirements, docs/PRD.md#NFR-D1]

**3. Version Pinning Strategy (NFR-D2):**
- **Strategy**: Major version locked (ratatui = "0.29" allows 0.29.x patches, blocks 0.30.x).
- **Rationale**: Semantic versioning in Rust allows patch/minor updates within major version. Major version updates reviewed manually via dependabot PRs.
- **Security**: cargo-audit monitors dependency advisories, CI fails on known vulnerabilities.

[Source: docs/architecture.md#Dependencies-and-Integrations, #ADR-0003]

**4. Binary Size Target (NFR-P6):**
- **Target**: Core library adds <2MB to compiled binaries.
- **Measurement**: `cargo build --release` (no features) → check size of `target/release/libdotmax.rlib` or a minimal example binary.
- **Validation**: AC #8 enforces this. If over 2MB, investigate heavy transitive dependencies.

[Source: docs/architecture.md#Performance-Considerations, docs/PRD.md#NFR-P6]

### Testing Standards

**Dependency Testing Strategy:**

**1. Feature Combination Matrix (AC #10):**
CI from Story 1.2 must test all valid feature combinations:
- `cargo build` (core only)
- `cargo build --features image` (core + image processing)
- `cargo build --features svg` (core + vector graphics)
- `cargo build --features image,svg` (core + all optional)

**2. MSRV Compatibility Check:**
MSRV job in CI uses Rust 1.70. All dependencies must support rust-version 1.70 or newer.
- Verify this by checking each dependency's Cargo.toml or docs.rs before adding.
- If dependency requires newer Rust, either find alternative or update MSRV (requires ADR).

**3. Security Audit (AC #10):**
cargo-audit runs in CI. New dependencies must have zero known vulnerabilities.
- If audit fails, check RustSec advisory database for details.
- Options: Pin to older secure version, find alternative dependency, or accept risk with justification (document in docs/dependencies.md).

**4. Binary Size Validation (AC #8):**
Manual test required:
```bash
cargo clean
cargo build --release  # No features
ls -lh target/release/libdotmax.rlib  # Should be <2MB

# Or build minimal example:
cargo run --release --example hello_braille
ls -lh target/release/examples/hello_braille  # Should be small
```

**5. Cross-Platform Compilation:**
CI matrix (Windows, Linux, macOS) validates dependencies compile on all platforms.
- Some dependencies have platform-specific code (e.g., crossterm handles Windows console differently).
- CI from Story 1.2 will catch platform-specific build failures.

### References

- [Source: docs/architecture.md#Technology-Stack-Details] - Exact dependency versions and justifications
- [Source: docs/architecture.md#Decision-Summary] - Dependency decision table
- [Source: docs/architecture.md#ADR-0003-Feature-Flag-Architecture] - Feature flag design rationale
- [Source: docs/architecture.md#Dependencies-and-Integrations] - Version pinning strategy
- [Source: docs/architecture.md#Performance-Considerations] - Binary size target (<2MB)
- [Source: docs/PRD.md#NFR-D1-Minimal-Core-Dependencies] - <10 dependency requirement
- [Source: docs/PRD.md#NFR-P6-Binary-Size-Impact] - Binary size NFR
- [Source: docs/PRD.md#FR61-67] - Library distribution and packaging FRs
- [Source: docs/epics.md#Epic-1-Story-1.3] - Original story definition
- [Source: docs/sprint-artifacts/tech-spec-epic-1.md#Dependencies-and-Integrations] - Epic 1 dependency specifications
- [Source: docs/sprint-artifacts/1-2-configure-github-actions-cicd-pipeline.md#Dev-Notes] - CI infrastructure from previous story

### Implementation Guidance

**Step-by-Step Implementation:**

**Step 1: Update Cargo.toml Dependencies Section**

Open `Cargo.toml` and add to `[dependencies]`:

```toml
[dependencies]
# Core dependencies (always included)
ratatui = "0.29"         # Terminal UI framework
crossterm = "0.29"       # Cross-platform terminal I/O
thiserror = "2.0"        # Error handling derive macros
tracing = "0.1"          # Structured logging

# Optional dependencies (feature-gated)
image = { version = "0.25", optional = true }
imageproc = { version = "0.24", optional = true }
resvg = { version = "0.38", optional = true }
usvg = { version = "0.38", optional = true }
```

**Step 2: Define Features Section**

Add immediately after `[dependencies]`:

```toml
[features]
default = []
image = ["dep:image", "dep:imageproc"]
svg = ["dep:resvg", "dep:usvg"]
```

**Important:** Use `dep:` prefix for weak dependency features (Rust 2021 edition syntax).

**Step 3: Add Dev Dependencies**

Add `[dev-dependencies]` section:

```toml
[dev-dependencies]
criterion = { version = "0.7", features = ["html_reports"] }
tracing-subscriber = "0.3"
```

**Step 4: Verify Dependency Count**

Count dependencies manually:
- Core: ratatui, crossterm, thiserror, tracing (4 total) ✓
- Optional: image, imageproc, resvg, usvg (4 total, not counted toward core limit)
- Dev: criterion, tracing-subscriber (2 total, not counted toward core limit)

**Step 5: Build and Test**

```bash
# Clean builds
cargo clean

# Test core only
cargo build --release
# Should succeed, binary <2MB

# Test with image feature
cargo clean
cargo build --release --features image
# Should succeed, image/imageproc in Cargo.lock

# Test with svg feature
cargo clean
cargo build --release --features svg
# Should succeed, resvg/usvg in Cargo.lock

# Test with all features
cargo clean
cargo build --release --features image,svg
# Should succeed, all optional deps in Cargo.lock
```

**Step 6: Measure Binary Size**

```bash
cargo clean
cargo build --release
ls -lh target/release/libdotmax.rlib

# If over 2MB, investigate:
cargo tree  # Show dependency tree
cargo bloat --release  # Show what's taking space
```

**Step 7: Create Dependency Documentation**

Create `docs/dependencies.md`:

```markdown
# Dependency Justifications

## Core Dependencies

### ratatui (0.29)
- **Purpose**: Terminal UI framework
- **Why Required**: ...
- **Alternatives Considered**: ...

(continue for all deps)
```

**Step 8: Push and Validate CI**

```bash
git add Cargo.toml Cargo.lock docs/dependencies.md
git commit -m "Add core dependencies with feature flags (Story 1.3)"
git push

# Monitor GitHub Actions
# Watch for:
# - All platforms (Windows, Linux, macOS) pass
# - All feature combinations build successfully
# - cargo audit passes (no vulnerabilities)
# - Build completes in <5 min with warm cache
```

### Constraints and Gotcas

**1. MSRV Compatibility (Rust 1.70):**
- **Issue**: Some dependencies may require newer Rust versions
- **Check**: Review each dep's docs.rs page or Cargo.toml for rust-version field
- **Solution**: If dependency requires Rust >1.70, either find alternative or update MSRV (document in ADR)

**2. Transitive Dependencies Bloat:**
- **Issue**: Core deps may have large transitive dependency trees
- **Check**: Run `cargo tree` to inspect full dependency graph
- **Mitigation**: Use `cargo tree --duplicates` to find duplicate versions
- **If Binary >2MB**: Consider switching dependencies or feature-gating more aggressively

**3. Feature Flag Syntax (Rust 2021 Edition):**
- **Correct**: `image = ["dep:image", "dep:imageproc"]` (weak dependency features)
- **Incorrect**: `image = ["image", "imageproc"]` (old syntax, creates implicit features)
- **Why**: Weak dependency features prevent accidental feature activation

**4. Optional Dependency Not Compiling:**
- **Issue**: `optional = true` dependency fails to compile when feature enabled
- **Debug**: `cargo build --features image -vv` (verbose output)
- **Common Causes**: Version conflicts, platform-specific issues
- **Solution**: Check Cargo.lock for version resolution, test on all platforms via CI

**5. cargo-audit Failures:**
- **Issue**: New dependency has known vulnerability
- **Response**: Check RustSec advisory (CI output has link)
- **Options**:
  - Pin to older secure version
  - Wait for patch (acceptable if not critical)
  - Find alternative dependency
  - Accept risk (only if justified, document in docs/dependencies.md)

**6. CI Cache Miss:**
- **Issue**: First push after adding dependencies will be slow (10-15 min)
- **Expected**: Normal for dependency changes
- **Mitigation**: Warm cache reduces subsequent builds to <5 min
- **Not a Failure**: As long as CI passes, timing is acceptable

**7. Platform-Specific Failures:**
- **Issue**: Dependency builds on Linux but fails on Windows/macOS
- **Debug**: Check CI logs for platform-specific errors
- **Common**: Some deps use different code paths per platform (e.g., crossterm)
- **Solution**: Verify dependencies support all platforms via docs/CI badges

**8. Cargo.lock Conflicts:**
- **Issue**: Merge conflicts in Cargo.lock after adding dependencies
- **Solution**: Delete Cargo.lock, run `cargo build`, regenerate lock file
- **Never**: Manually edit Cargo.lock

### Dependencies

**Story Dependencies:**
- Story 1.1 (done): Cargo.toml exists with package metadata
- Story 1.2 (done): CI pipeline exists to validate dependencies across platforms

**Technical Dependencies:**
- Rust 1.70+ installed (MSRV requirement)
- Cargo for dependency management
- Internet access to download dependencies from crates.io

**Follow-on Stories:**
- Story 1.4: Quality tooling (clippy/rustfmt will lint dependency usage)
- Story 1.5: ADR system (may document dependency choices in ADRs)
- Story 1.6: Benchmarking (criterion from dev-dependencies will be used)
- Epic 2: Core rendering (will use ratatui, crossterm, thiserror, tracing)
- Epic 3: Image rendering (will use image, imageproc, resvg, usvg feature-gated)

### Security Considerations

**From Architecture Document (Section: Security Architecture, NFR-S3):**

**1. Dependency Security Scanning:**
- cargo-audit runs in CI (configured in Story 1.2)
- Checks RustSec Advisory Database for known vulnerabilities
- Fails CI if vulnerabilities detected
- This story adds 8 new dependencies → audit will scan all of them

**2. License Compliance (planned for Story 1.4):**
- cargo-deny will enforce MIT/Apache-2.0 compatible licenses
- No GPL/AGPL dependencies allowed (incompatible with dual licensing)
- This story should verify licenses manually: ratatui (MIT), crossterm (MIT), thiserror (MIT/Apache-2.0), tracing (MIT), image (MIT/Apache-2.0), etc.

**3. Supply Chain Security:**
- Pin major versions in Cargo.toml (e.g., `ratatui = "0.29"`)
- Allows patch updates (0.29.x) but blocks breaking changes (0.30.x)
- Dependabot PRs will propose major version updates for manual review

**4. Upstream Risk Mitigation (NFR-D2):**
- Abstract terminal backend via TerminalBackend trait (Epic 2)
- Reduces ratatui/crossterm lock-in risk
- If upstream breaks, can swap implementation without changing public API

**Validation Steps:**
- [ ] Run `cargo audit` locally before pushing
- [ ] Check licenses: `cargo license` (or manually via docs.rs)
- [ ] Verify no GPL/AGPL dependencies: `cargo tree | grep -i gpl` (should be empty)

[Source: docs/architecture.md#Security-Architecture, docs/PRD.md#NFR-S3]

### Performance Validation

**Binary Size Target (AC #8, NFR-P6):**

**Measurement Method:**
```bash
# Core only (no features)
cargo clean
cargo build --release
ls -lh target/release/libdotmax.rlib

# Or build minimal example (more realistic):
cargo run --release --example hello_braille
ls -lh target/release/examples/hello_braille
```

**Expected Results:**
- Core library (rlib): <500KB likely (minimal code, mostly deps)
- Example binary: <2MB (includes core deps + example code)
- If over 2MB: Investigate with `cargo bloat --release`

**Dependency Overhead Analysis:**
```bash
# Show what's using space
cargo bloat --release --crates

# Show dependency tree
cargo tree

# Find duplicate dependencies
cargo tree --duplicates
```

**CI Build Time (AC #10):**
- First build (cold cache): 10-15 minutes expected (new dependencies download)
- Subsequent builds (warm cache): <5 minutes target
- Monitor GitHub Actions timing after first push

**Build Time Breakdown Estimates:**
- Download dependencies: 1-2 min (cold cache)
- Compile dependencies: 5-10 min (cold cache, cached after)
- Compile dotmax: <1 min (minimal code in Epic 1)
- Run tests: <1 min (basic tests only)
- Total cold: ~10-15 min, Total warm: <5 min ✓

[Source: docs/architecture.md#Performance-Considerations, docs/PRD.md#NFR-P6]

### Definition of Done

Story is complete when:
- [ ] Cargo.toml has exactly 4 core dependencies (ratatui, crossterm, thiserror, tracing)
- [ ] Optional dependencies (image, imageproc, resvg, usvg) are feature-gated with `optional = true`
- [ ] Features section defines `default = []`, `image = ["dep:image", "dep:imageproc"]`, `svg = ["dep:resvg", "dep:usvg"]`
- [ ] `cargo build` (no features) compiles successfully
- [ ] `cargo build --features image,svg` compiles successfully
- [ ] Binary size for core-only build is <2MB (measured and documented)
- [ ] All dependency versions match Architecture Document specifications
- [ ] CI passes on all platforms (Windows, Linux, macOS) for all feature combinations
- [ ] cargo audit passes (no known vulnerabilities in dependencies)
- [ ] docs/dependencies.md exists and documents all dependency justifications
- [ ] Cargo.lock is committed and up-to-date
- [ ] Story status updated to "drafted" in sprint-status.yaml

## Dev Agent Record

### Context Reference

- docs/sprint-artifacts/1-3-define-core-dependencies-with-feature-flags.context.xml

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
