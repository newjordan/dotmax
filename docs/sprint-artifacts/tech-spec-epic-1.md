# Epic Technical Specification: Foundation & Project Setup

Date: 2025-11-15
Author: Frosty
Epic ID: 1
Status: Draft

---

## Overview

Epic 1 establishes the professional foundation for dotmax as a production-ready Rust library crate. This epic transforms dotmax from concept into a structured, testable, cross-platform project with automated quality controls, comprehensive documentation scaffolding, and performance tracking infrastructure.

The work focuses on project initialization, CI/CD pipeline configuration, dependency management with feature flags, code quality tooling (Clippy, Rustfmt, cargo-deny), Architecture Decision Records (ADR) system, benchmarking infrastructure (Criterion.rs), and example/documentation templates. This foundation enables all subsequent development while ensuring the project remains maintainable by a solo developer over years, with performance tracking built-in from day one.

This epic delivers no end-user functionality but establishes the critical infrastructure that ensures dotmax meets its aggressive performance targets (<25ms image rendering), maintains zero-panic reliability, and supports resumable development after long gaps.

## Objectives and Scope

### In Scope

**Infrastructure:**
- Cargo library project initialization with optimal metadata and structure
- GitHub Actions CI/CD pipeline (cross-platform: Windows, Linux, macOS)
- Code quality tooling: Clippy, Rustfmt, cargo-deny configurations
- Benchmark infrastructure with Criterion.rs and CI integration
- ADR system for documenting architectural decisions
- Example and documentation scaffolding

**Quality Gates:**
- Automated cross-platform testing on every commit
- Linting and formatting enforcement in CI
- Security vulnerability scanning (cargo-audit)
- Performance regression detection (benchmark tracking)
- License compliance verification (cargo-deny)

**Developer Experience:**
- Project compiles and tests successfully (`cargo build`, `cargo test`)
- Examples compile and run (`cargo run --example hello_braille`)
- Clear README with installation, quick start, features
- Dual licensing (MIT OR Apache-2.0) for maximum adoption

### Out of Scope

- Actual rendering code (Epic 2+)
- Image processing capabilities (Epic 3)
- Drawing primitives (Epic 4)
- Production release process (Epic 7)
- Community outreach or marketing
- crates.io publication (deferred to Epic 7)

### Success Criteria

1. **Clean Build**: `cargo build` succeeds on Windows, Linux, macOS
2. **Passing Tests**: `cargo test` passes (empty test suite acceptable)
3. **Quality Enforcement**: CI fails on clippy warnings, format violations, audit issues
4. **Documentation Foundation**: ADR system exists with first ADR documented
5. **Performance Tracking**: Benchmark infrastructure operational, ready for Epic 2 benchmarks
6. **Resumability**: Project can be understood and resumed after months by reading README + ADRs

## System Architecture Alignment

Epic 1 establishes the architectural scaffolding defined in the Architecture Document (docs/architecture.md):

**Project Structure**: Creates the directory layout with src/, benches/, examples/, tests/, docs/adr/ as specified in the architecture. This structure supports the modular, feature-based organization (grid, render, image, primitives, color, animation modules) that will be implemented in later epics.

**Dependency Management**: Implements the feature flag architecture critical to dotmax's minimal footprint requirement (<2MB core). Establishes the pattern where core dependencies (ratatui, crossterm, thiserror) are always included, while optional capabilities (image, svg, future video/raytrace) are feature-gated to prevent bloat.

**Quality Standards**: Enforces the zero-panic mandate through CI linting (Clippy pedantic mode, deny warnings). Establishes the performance-first culture through Criterion.rs benchmarking infrastructure, aligning with the aggressive performance targets (<25ms image rendering, 60-120fps animation).

**Solo Maintainability**: Implements the ADR system which is critical for resumable development (NFR-M2). ADRs document WHY decisions were made, enabling the solo developer to resume work after months/years without losing context.

**Cross-Platform Commitment**: CI matrix testing (Windows, Linux, macOS) validates the universal compatibility promise from day one, preventing platform-specific code from creeping in.

**Constraints Satisfied**:
- Rust 2021 edition, MSRV 1.70 (stable Rust only)
- Standard cargo tooling (no special templates or custom build systems)
- Minimal core dependencies (<10 total, only 4 required)
- Performance measurement infrastructure ready before optimization begins

## Detailed Design

### Services and Modules

This epic creates infrastructure, not runtime services. Key organizational modules:

| Component | Responsibility | Files Created | Owner |
|-----------|---------------|---------------|-------|
| **Project Manifest** | Define crate metadata, dependencies, feature flags, build config | Cargo.toml, .gitignore | Story 1.1, 1.3 |
| **CI/CD Pipeline** | Automated testing, linting, security scanning across platforms | .github/workflows/ci.yml, .github/workflows/benchmark.yml | Story 1.2, 1.6 |
| **Quality Tools** | Code formatting, linting, license/security compliance | .rustfmt.toml, clippy.toml, .deny.toml | Story 1.4 |
| **ADR System** | Architecture decision documentation and knowledge preservation | docs/adr/README.md, template.md, 0001-use-braille-unicode.md | Story 1.5 |
| **Benchmark Framework** | Performance measurement and regression tracking | benches/rendering.rs, criterion config | Story 1.6 |
| **Example Templates** | API demonstration and DX validation | examples/hello_braille.rs, examples/README.md | Story 1.7 |
| **Documentation Scaffold** | User-facing docs, dependency justifications, performance tracking | README.md, docs/dependencies.md, docs/performance.md | Story 1.7 |

### Data Models and Contracts

**Cargo.toml Structure** (Story 1.1, 1.3):
```toml
[package]
name = "dotmax"
version = "0.1.0"
authors = ["Frosty"]
edition = "2021"
rust-version = "1.70"
description = "High-performance terminal braille rendering for images, animations, and graphics"
license = "MIT OR Apache-2.0"
repository = "https://github.com/frosty40/dotmax"
keywords = ["terminal", "braille", "graphics", "cli", "visualization"]
categories = ["command-line-interface", "graphics", "rendering"]

[dependencies]
ratatui = "0.29"
crossterm = "0.29"
thiserror = "2.0"
tracing = "0.1"

# Optional dependencies (feature-gated)
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

[[bench]]
name = "rendering"
harness = false
```

**ADR Template Structure** (Story 1.5):
```markdown
# ADR-XXXX: [Title]

## Status
[Proposed | Accepted | Deprecated | Superseded by ADR-YYYY]

## Context
[Problem, constraints, forces at play]

## Decision
[What was decided]

## Consequences
[Trade-offs, positive and negative outcomes]

## Alternatives Considered
[What was rejected and why]
```

**CI/CD Matrix Configuration** (Story 1.2):
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
    rust: [stable, 1.70]  # MSRV validation
```

### APIs and Interfaces

This epic creates build-time and development-time interfaces, not runtime APIs:

**Cargo CLI Interface** (Story 1.1):
- `cargo build` - Compile core library
- `cargo build --features image,svg` - Compile with optional features
- `cargo test` - Run test suite
- `cargo bench` - Run benchmark suite
- `cargo clippy` - Lint code
- `cargo fmt` - Format code
- `cargo deny check` - Validate licenses and security

**CI Workflow Interface** (Story 1.2):
- **Triggers**: Push to any branch, pull request creation
- **Outputs**: Build status (pass/fail), clippy warnings, format violations, security advisories
- **Artifacts**: Benchmark results, test coverage reports (future)

**Benchmark Interface** (Story 1.6):
- **Input**: Benchmark functions in `benches/` directory
- **Output**: Criterion.rs HTML reports in `target/criterion/`, comparison against baseline
- **CI Integration**: Performance regression detection (>10% slowdown triggers warning)

**Example Interface** (Story 1.7):
- `cargo run --example hello_braille` - Minimal braille demonstration
- Examples compile in CI to validate public API usability
- Each example <50 lines, demonstrates single concept

### Workflows and Sequencing

**Development Workflow** (enabled by this epic):
```
Developer writes code
    ↓
git commit
    ↓
GitHub Actions triggers
    ├── Build on [ubuntu, windows, macos] × [stable, MSRV 1.70]
    ├── cargo test (all platforms)
    ├── cargo clippy --deny warnings
    ├── cargo fmt --check
    ├── cargo audit (security vulnerabilities)
    └── cargo deny check (licenses, duplicates)
    ↓
All checks pass?
    ├── YES → CI green, PR mergeable
    └── NO  → CI red, developer fixes issues
```

**Story Execution Sequence** (sequential dependencies):
1. **Story 1.1**: Initialize project → Creates Cargo.toml, src/lib.rs, directory structure
2. **Story 1.2**: Configure CI → Requires Story 1.1 (Cargo project must exist)
3. **Story 1.3**: Define dependencies → Requires Story 1.1 (Cargo.toml must exist)
4. **Story 1.4**: Set up quality tools → Requires Story 1.2 (CI must exist to enforce)
5. **Story 1.5**: Create ADR system → Requires Story 1.1 (docs/ directory must exist)
6. **Story 1.6**: Set up benchmarks → Requires Story 1.1, 1.2 (Cargo project + CI)
7. **Story 1.7**: Create examples → Requires Story 1.1 (project structure exists)

**Benchmark Workflow** (Story 1.6):
```
Developer runs: cargo bench
    ↓
Criterion executes benches/rendering.rs
    ├── bench_braille_grid_creation (placeholder)
    ├── bench_grid_clear (placeholder)
    └── bench_unicode_conversion (placeholder)
    ↓
Criterion generates:
    ├── Console output with timing statistics
    ├── HTML reports in target/criterion/ (graphs, statistical analysis)
    └── Baseline for future comparisons
    ↓
CI stores benchmark results as artifacts
    ↓
PR comments if performance regresses >10%
```

## Non-Functional Requirements

### Performance

**Targets for Epic 1**:
- **Build Time**: Clean build <2 minutes on CI (ubuntu-latest)
- **CI Execution**: Total CI pipeline <5 minutes for all checks
- **Benchmark Infrastructure**: Criterion overhead <10ms per benchmark function

**Validation**:
- CI timing logged and monitored
- Benchmark infrastructure tested with placeholder functions
- Future epics will add real performance benchmarks (image rendering <25ms, animation 60-120fps)

**Rationale**: Performance tracking infrastructure must be lightweight to avoid slowing development cycles. Actual performance optimization occurs in Epic 2-7.

### Security

**Requirements**:
1. **Dependency Security**: `cargo audit` runs on every CI build, fails on known vulnerabilities
2. **License Compliance**: `cargo-deny` blocks GPL/AGPL dependencies (incompatible with dual MIT/Apache-2.0)
3. **Supply Chain**: Pin dependency versions in Cargo.toml to prevent unexpected upstream changes
4. **Secret Management**: No secrets in repository (enforced via .gitignore, GitHub secret scanning)

**Threat Model** (for infrastructure):
- **Threat**: Malicious dependency introduced
  - **Mitigation**: cargo-audit detects known vulnerabilities, cargo-deny enforces license policy
- **Threat**: Compromised CI credentials
  - **Mitigation**: GitHub Actions uses OIDC tokens, no long-lived secrets stored
- **Threat**: Accidental commit of API keys
  - **Mitigation**: .gitignore excludes .env, credentials.json, etc.

**Validation**:
- `cargo deny check advisories` passes (no known vulnerabilities)
- `cargo deny check licenses` passes (no GPL/AGPL deps)
- CI fails if audit/deny checks fail

### Reliability/Availability

**Build Reliability**:
- **Target**: 99% CI success rate for clean code (no flaky tests)
- **Strategy**: Deterministic builds, no network dependencies in tests, cache dependencies
- **Monitoring**: Track CI failure rate, investigate flakes immediately

**Cross-Platform Reliability**:
- **Target**: 100% compatibility Windows/Linux/macOS for core library
- **Strategy**: CI matrix testing on all platforms, no platform-specific code without abstraction
- **Validation**: All CI jobs must pass (Windows, Linux, macOS)

**Resumability** (NFR-M2):
- **Target**: Developer can resume work after 6+ months gap with <1 hour ramp-up
- **Strategy**: ADR system documents WHY decisions made, README explains WHAT/HOW
- **Validation**: ADRs exist for major decisions, README has complete quick-start

### Observability

**Build Observability**:
- **CI Logs**: GitHub Actions provides detailed logs for each step (build, test, clippy, fmt, audit)
- **Benchmark Reports**: Criterion generates HTML reports with graphs, statistical analysis
- **Artifact Storage**: Benchmark results stored as CI artifacts for historical comparison

**Developer Feedback**:
- **Clippy**: Linting feedback during development (`cargo clippy`) and in CI
- **Rustfmt**: Format violations detected pre-commit and in CI
- **cargo-deny**: Immediate feedback on license/security issues

**Performance Tracking** (Story 1.6):
- Criterion stores baselines in `target/criterion/`
- CI compares against previous run, comments on PRs if regression >10%
- HTML reports show performance over time (graphs, percentiles, outliers)

## Dependencies and Integrations

### Core Dependencies

| Dependency | Version | Purpose | Justification | Optional? |
|------------|---------|---------|---------------|-----------|
| **ratatui** | 0.29 | Terminal UI framework | Industry standard for Rust TUIs. Provides cross-platform terminal abstraction. | No |
| **crossterm** | 0.29 | Terminal I/O | Cross-platform terminal control (cursor, colors, events). Works with ratatui. | No |
| **thiserror** | 2.0 | Error handling | Minimal boilerplate for error derivation. Library users need typed errors. | No |
| **tracing** | 0.1 | Structured logging | Standard for Rust instrumentation. Multiple log levels, structured context. | No |

**Total Core Dependencies**: 4 (meets <10 target)

### Optional Dependencies (Feature-Gated)

| Dependency | Version | Feature Flag | Purpose | Epic |
|------------|---------|--------------|---------|------|
| **image** | 0.25 | `image` | Load PNG/JPG/GIF/BMP/WebP/TIFF | Epic 3 |
| **imageproc** | 0.24 | `image` | Image processing (dithering, threshold) | Epic 3 |
| **resvg** | 0.38 | `svg` | SVG rasterization | Epic 3 |
| **usvg** | 0.38 | `svg` | SVG parsing | Epic 3 |

### Development Dependencies

| Dependency | Version | Purpose | Epic |
|------------|---------|---------|------|
| **criterion** | 0.7 | Benchmarking | Epic 1 (infrastructure), Epic 2-7 (actual benchmarks) |
| **tracing-subscriber** | 0.3 | Logging in tests | All epics |

### External System Integrations

**GitHub Actions**:
- **Integration Point**: `.github/workflows/` YAML files
- **Authentication**: GitHub OIDC tokens (automatic)
- **Artifacts**: Benchmark results, build logs
- **Version**: GitHub Actions runner (ubuntu-latest, windows-latest, macos-latest)

**crates.io** (future, Epic 7):
- **Integration Point**: `cargo publish`
- **Authentication**: API token (stored in GitHub Secrets)
- **Version**: Cargo 1.70+

**docs.rs** (future, Epic 7):
- **Integration Point**: Automatic from crates.io publication
- **Documentation**: Generated from rustdoc comments

### Version Pinning Strategy

**Cargo.toml Dependencies**:
- **Major Version Locked**: `ratatui = "0.29"` (allows 0.29.x patches, blocks 0.30.x)
- **Rationale**: Semantic versioning in Rust allows patch/minor updates within major version. Major version updates reviewed manually.

**CI Rust Toolchain**:
- **Stable**: Always latest stable Rust (auto-updated by GitHub Actions)
- **MSRV**: 1.70 explicitly tested to ensure minimum version compatibility

## Acceptance Criteria (Authoritative)

### Epic-Level Acceptance Criteria

**AC1**: Project initializes and builds successfully
- **Given** an empty directory
- **When** developer runs `cargo new --lib dotmax` and sets up structure per Story 1.1
- **Then** `cargo build` succeeds on Windows, Linux, macOS without warnings

**AC2**: CI pipeline operational and enforcing quality gates
- **Given** code pushed to GitHub
- **When** GitHub Actions CI runs (Story 1.2)
- **Then** all checks pass: build (3 platforms), test, clippy, fmt, audit

**AC3**: Feature flags prevent dependency bloat
- **Given** Cargo.toml configured per Story 1.3
- **When** developer runs `cargo build` (no features)
- **Then** only core deps (ratatui, crossterm, thiserror, tracing) are compiled
- **And** binary size <2MB (empty lib, just deps)

**AC4**: Code quality tools enforce standards
- **Given** clippy.toml, .rustfmt.toml, .deny.toml configured per Story 1.4
- **When** developer runs `cargo clippy`, `cargo fmt --check`, `cargo deny check`
- **Then** all pass without warnings for clean code
- **And** CI fails if any tool reports violations

**AC5**: ADR system documents decisions
- **Given** docs/adr/ structure per Story 1.5
- **When** developer reviews docs/adr/README.md
- **Then** ADR index exists with first ADR (0001-use-braille-unicode.md)
- **And** ADR template (template.md) is available for future decisions

**AC6**: Benchmark infrastructure operational
- **Given** benches/rendering.rs with placeholder benchmarks per Story 1.6
- **When** developer runs `cargo bench`
- **Then** Criterion executes successfully and generates HTML reports
- **And** CI stores benchmark results as artifacts

**AC7**: Examples and documentation scaffold exists
- **Given** examples/hello_braille.rs and README.md per Story 1.7
- **When** developer runs `cargo run --example hello_braille`
- **Then** example compiles and runs successfully
- **And** README.md includes Installation, Quick Start, Features, License sections

**AC8**: Project is resumable after gap
- **Given** complete Epic 1 deliverables
- **When** developer returns after months away
- **Then** README + ADRs provide enough context to understand project structure, decisions, and how to contribute
- **And** `cargo build && cargo test` succeeds immediately without manual setup

### Story-Level Acceptance Criteria

Detailed acceptance criteria exist in each story (Stories 1.1-1.7) within Epic 1 section of epics.md. Epic-level ACs above are authoritative roll-ups.

## Traceability Mapping

| Acceptance Criteria | Spec Section | Components | Test Approach |
|---------------------|--------------|------------|---------------|
| **AC1**: Build succeeds | Detailed Design → Project Manifest | Cargo.toml, src/lib.rs, directory structure | CI runs `cargo build` on 3 platforms |
| **AC2**: CI operational | Detailed Design → CI/CD Pipeline | .github/workflows/ci.yml | Manual verification: push code, observe CI pass/fail |
| **AC3**: Feature flags work | Detailed Design → Data Models (Cargo.toml) | Feature definitions, optional deps | CI: `cargo build` (no features) and `cargo build --features image,svg` |
| **AC4**: Quality tools enforce | Detailed Design → Quality Tools | clippy.toml, .rustfmt.toml, .deny.toml | CI fails on lint/format violations (negative test) |
| **AC5**: ADR system exists | Detailed Design → ADR System | docs/adr/ directory, template, first ADR | Manual review of docs/adr/ structure |
| **AC6**: Benchmarks run | Detailed Design → Benchmark Framework | benches/rendering.rs, Criterion config | CI runs `cargo bench`, stores artifacts |
| **AC7**: Examples work | Detailed Design → Example Templates | examples/hello_braille.rs, README.md | CI runs `cargo run --example hello_braille` |
| **AC8**: Resumability | Overview, ADR System | README.md, docs/adr/ | Manual test: new developer reviews docs and can start contributing within 1 hour |

### FR Traceability (Epic 1 covers FR61-67, FR75-80)

| FR | Title | Stories | Test Coverage |
|----|-------|---------|---------------|
| **FR61** | Install via `cargo add dotmax` | 1.1 (project setup) | Epic 7: crates.io publication |
| **FR62** | Binary size <2MB | 1.3 (feature flags) | `cargo build` size check (core only) |
| **FR63** | Minimal core dependencies | 1.3 (dependencies) | Verify Cargo.toml has ≤4 core deps |
| **FR64** | Feature flags for optional capabilities | 1.3 (feature flags) | CI builds with/without features |
| **FR65** | Compile on stable Rust | 1.1, 1.2 (MSRV check) | CI tests Rust 1.70 (MSRV) |
| **FR66** | API docs via rustdoc | 1.7 (doc scaffold) | Epic 7: full rustdoc generation |
| **FR67** | Examples directory | 1.7 (examples) | CI compiles examples |
| **FR75-80** | Cross-platform (Windows, Linux, macOS) | 1.2 (CI matrix) | CI builds on all 3 platforms |

## Risks, Assumptions, Open Questions

### Risks

**Risk 1**: CI build times exceed 5-minute target
- **Impact**: Slow developer feedback, frustration
- **Probability**: Low (simple lib, dependency caching)
- **Mitigation**: Use `Swatinem/rust-cache@v2` for dependency caching, monitor CI times
- **Contingency**: Optimize CI (parallelize jobs, reduce matrix if needed)

**Risk 2**: Dependency version conflicts (ratatui vs crossterm)
- **Impact**: Build failures, incompatible APIs
- **Probability**: Low (both maintained, stable)
- **Mitigation**: Pin compatible versions, test during Story 1.3
- **Contingency**: Downgrade/upgrade versions to find compatible pair

**Risk 3**: cargo-deny overly restrictive (blocks useful deps)
- **Impact**: Can't use needed dependencies
- **Probability**: Medium (some deps may have transitive GPL deps)
- **Mitigation**: Configure .deny.toml with exceptions if justified, document in ADR
- **Contingency**: Relax deny rules if blocker, but only after ADR review

**Risk 4**: Benchmark infrastructure overhead slows development
- **Impact**: Developers skip benchmarks to save time
- **Probability**: Low (Criterion is fast for small benchmarks)
- **Mitigation**: Keep benchmark suite small in Epic 1, optimize later
- **Contingency**: Make benchmarks optional in CI (manual trigger)

**Risk 5**: MSRV 1.70 too old for needed features
- **Impact**: Can't use modern Rust features
- **Probability**: Low (1.70 is fairly recent, most features available)
- **Mitigation**: Re-evaluate MSRV if blocker, document in ADR
- **Contingency**: Increase MSRV to 1.75 if absolutely required

### Assumptions

**Assumption 1**: GitHub Actions remains free for public repos
- **Validation**: True as of 2025, GitHub's stated policy
- **Impact if Wrong**: Must migrate to alternative CI (GitLab CI, Travis CI)

**Assumption 2**: Rust ecosystem remains stable (no Rust 2.0 breaking changes)
- **Validation**: Rust prioritizes stability, no breaking changes announced
- **Impact if Wrong**: Migration effort to new edition

**Assumption 3**: ratatui/crossterm remain actively maintained
- **Validation**: Both have active communities, recent releases
- **Impact if Wrong**: May need to fork or switch terminal libraries

**Assumption 4**: Solo developer has Windows/Linux/macOS access for testing
- **Validation**: CI provides cross-platform testing, local testing optional
- **Impact if Wrong**: None (CI catches platform issues)

**Assumption 5**: Dual MIT/Apache-2.0 license maximizes adoption
- **Validation**: Standard for Rust ecosystem (matches std library)
- **Impact if Wrong**: None (license choice is strategic, not technical)

### Open Questions

**Question 1**: Should benchmarks run on every PR or only on main branch?
- **Options**: (A) Every PR - catches regressions early but slows CI, (B) Main branch only - faster CI but regressions detected late
- **Recommendation**: Start with (B) main branch only, add PR benchmarks if regressions slip through
- **Decision Owner**: Developer (Story 1.6)

**Question 2**: Should examples include advanced features (color, animation) in Epic 1?
- **Options**: (A) Minimal only (hello_braille), (B) Placeholder advanced examples
- **Recommendation**: (A) Minimal only - advanced examples added in respective epics (Epic 5, 6)
- **Decision Owner**: Developer (Story 1.7)

**Question 3**: Should ADRs be numbered or dated?
- **Options**: (A) Numbered (0001, 0002), (B) Dated (2025-11-15-braille-unicode)
- **Recommendation**: (A) Numbered - sortable, less prone to conflicts
- **Decision Owner**: Developer (Story 1.5)

**Question 4**: Should cargo-deny fail on duplicate dependencies?
- **Options**: (A) Fail on any duplicates, (B) Warn only
- **Recommendation**: (B) Warn only initially - duplicates are common, block only if severe
- **Decision Owner**: Developer (Story 1.4)

## Test Strategy Summary

### Testing Levels

**Unit Tests** (Epic 1):
- **Scope**: None required (no functional code in Epic 1)
- **Future**: Epic 2+ will add unit tests for grid operations, rendering, etc.

**Integration Tests** (Epic 1):
- **Scope**: Examples compile and run (`cargo run --example hello_braille`)
- **Framework**: Cargo built-in test harness
- **CI**: `cargo build --examples` ensures examples always compile

**Build Tests** (Epic 1):
- **Scope**: Cross-platform compilation (Windows, Linux, macOS)
- **Framework**: GitHub Actions CI matrix
- **CI**: `cargo build` on all platforms, `cargo build --features image,svg`

**Linting Tests** (Epic 1):
- **Scope**: Code quality (clippy warnings, format violations)
- **Framework**: Clippy, Rustfmt
- **CI**: `cargo clippy --deny warnings`, `cargo fmt --check`

**Security Tests** (Epic 1):
- **Scope**: Dependency vulnerabilities, license compliance
- **Framework**: cargo-audit, cargo-deny
- **CI**: `cargo audit`, `cargo deny check advisories`, `cargo deny check licenses`

**Performance Tests** (Epic 1):
- **Scope**: Benchmark infrastructure operational (placeholder benchmarks)
- **Framework**: Criterion.rs
- **CI**: `cargo bench` on main branch pushes, store artifacts

### Test Coverage Goals

**Epic 1 Coverage**: N/A (infrastructure only, no functional code)

**Future Coverage** (Epic 2+):
- **Target**: 80%+ line coverage for core modules (grid, render, image)
- **Tool**: tarpaulin or cargo-llvm-cov
- **CI**: Coverage reports generated and tracked

### Test Data and Fixtures

**Epic 1**:
- **Fixtures**: None required (examples use minimal hardcoded data)

**Future** (Epic 3+):
- **Test Images**: tests/test_assets/ will contain sample.png, test_svg.svg, benchmark_image.jpg
- **Licensing**: Test assets must be CC0 or created by developer to avoid licensing issues

### CI Test Matrix

**Platforms**: ubuntu-latest, windows-latest, macos-latest
**Rust Versions**: stable (latest), 1.70 (MSRV)
**Feature Combinations**:
- No features (core only)
- `--features image`
- `--features svg`
- `--features image,svg`

**Total CI Jobs**: 3 platforms × 2 Rust versions = 6 jobs per push

### Regression Testing

**Performance Regression** (Story 1.6):
- Criterion stores baselines in `target/criterion/`
- CI compares against previous run
- PR comment if regression >10% detected
- Manual review required if performance degrades

**API Regression** (Epic 2+):
- Examples serve as API regression tests
- If public API changes break examples, CI fails
- Forces API stability (critical for post-1.0)

### Edge Cases and Negative Testing

**Epic 1 Edge Cases**:
- **Empty project**: `cargo build` on fresh clone (tests CI setup)
- **No features**: `cargo build` without feature flags (tests core-only compilation)
- **Malformed Cargo.toml**: Manual test (verify cargo errors are clear)
- **Missing dependencies**: Delete Cargo.lock, re-build (tests deterministic dep resolution)

**Negative Tests**:
- **Lint violations**: Intentionally introduce clippy warning → verify CI fails
- **Format violations**: Mis-format code → verify `cargo fmt --check` fails CI
- **Security issues**: Pin old vulnerable dep → verify `cargo audit` catches it

### Manual Testing Checklist

- [ ] Clone fresh repository
- [ ] Run `cargo build` → succeeds on local platform
- [ ] Run `cargo test` → passes (empty test suite OK)
- [ ] Run `cargo run --example hello_braille` → compiles and runs
- [ ] Push code → verify CI runs and passes all checks
- [ ] Review GitHub Actions logs → no unexpected warnings/errors
- [ ] Open `target/criterion/report/index.html` → benchmark reports visible
- [ ] Review docs/adr/ → ADR system complete and readable
- [ ] Read README.md → can understand project within 5 minutes

---

**End of Epic 1 Technical Specification**
