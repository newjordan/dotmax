# Story 7.6: Publish to crates.io and Create Release Process

Status: in-progress

## Story

As a **library maintainer**,
I want **an automated release process that publishes to crates.io**,
so that **users can easily install dotmax via `cargo add` and trust the release quality**.

## Acceptance Criteria

1. **AC1: Pre-publication checklist passes**
   - All tests pass (`cargo test --all-features`)
   - Zero clippy warnings (`cargo clippy --all-features -- -D warnings`)
   - Documentation builds (`cargo doc --no-deps --all-features`)
   - Benchmarks complete without errors (`cargo bench`)
   - Security audit passes (`cargo audit`)
   - License check passes (`cargo deny check`)

2. **AC2: Version set to 0.1.0**
   - `Cargo.toml` version = "0.1.0"
   - Version follows semantic versioning (0.x allows breaking changes)
   - MSRV (rust-version) properly set to 1.70

3. **AC3: CHANGELOG.md updated**
   - Contains 0.1.0 release notes
   - Lists all major features by epic
   - Documents breaking changes (if any)
   - Links to docs.rs documentation

4. **AC4: Dry-run succeeds**
   - `cargo publish --dry-run` exits 0
   - No missing files or metadata errors
   - Package size reasonable (<10MB)

5. **AC5: Release workflow exists**
   - `.github/workflows/release.yml` triggers on tag push (`v*`)
   - Runs full test suite before publishing
   - Publishes to crates.io via `cargo publish`
   - Creates GitHub release with notes

6. **AC6: Published to crates.io**
   - `cargo add dotmax` works in any project
   - Package visible on crates.io/crates/dotmax
   - Correct metadata (description, keywords, categories)

7. **AC7: GitHub release created**
   - Tag `v0.1.0` exists and is annotated
   - GitHub release page has release notes
   - Links to documentation and changelog

8. **AC8: docs.rs builds**
   - Documentation available at docs.rs/dotmax
   - All features documented
   - No build errors on docs.rs

9. **AC9: Post-release verification**
   - Installation tested in clean environment
   - Verify `cargo add dotmax` in new project
   - Verify basic functionality works
   - Verify feature flags work (`cargo add dotmax -F image,svg`)

## Tasks / Subtasks

- [x] **Task 1: Prepare Cargo.toml for Publication** (AC: #2, #4) ✅
  - [x] 1.1: Update version to "0.1.0" ✅ (already set)
  - [x] 1.2: Verify package metadata (name, description, license, repository) ✅
  - [x] 1.3: Verify keywords (5 max: terminal, braille, graphics, cli, visualization) ✅
  - [x] 1.4: Verify categories (command-line-interface, graphics, rendering) ✅
  - [x] 1.5: Add documentation URL (https://docs.rs/dotmax) ✅
  - [x] 1.6: Add homepage URL (https://github.com/frosty40/dotmax) ✅
  - [x] 1.7: Verify rust-version = "1.70" (MSRV) ✅
  - [x] 1.8: Add exclude patterns for dev files (.github, .bmad, .claude, .augment, .gemini, docs/sprint-artifacts, tests/fixtures, etc.) ✅

- [x] **Task 2: Create CHANGELOG.md** (AC: #3) ✅
  - [x] 2.1: Created CHANGELOG.md with Keep a Changelog format ✅
  - [x] 2.2: Added v0.1.0 release notes with date 2025-11-26 ✅
  - [x] 2.3: Documented all 7 epics with features ✅
  - [x] 2.4: Added performance metrics ✅
  - [x] 2.5: Added platform support section ✅
  - [x] 2.6: Added links to docs.rs, crates.io, GitHub ✅

- [x] **Task 3: Run Pre-Publication Checklist** (AC: #1) ✅
  - [x] 3.1: `cargo test --all-features` - 232 tests passed ✅
  - [x] 3.2: `cargo clippy --all-features -- -D warnings` - zero warnings ✅
  - [x] 3.3: `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` - docs build clean ✅
  - [x] 3.4: `cargo test --doc` - 163 doctests passed ✅
  - [x] 3.5: `cargo bench --all-features` - benchmarks compile and run ✅
  - [x] 3.6: `cargo audit` - 0 vulnerabilities (2 unmaintained warnings) ✅
  - [x] 3.7: `cargo deny check` - advisories ok, bans ok, licenses ok, sources ok ✅

- [x] **Task 4: Validate Dry-Run Publication** (AC: #4) ✅
  - [x] 4.1: `cargo package --allow-dirty --list` - 125 files ✅
  - [x] 4.2: Package created successfully (292KB) ✅
  - [x] 4.3: Package size excellent (<10MB target, actual 292KB) ✅
  - [x] 4.4: Excluded dev files (.bmad, .claude, .augment, .gemini, test fixtures) ✅

- [x] **Task 5: Create GitHub Actions Release Workflow** (AC: #5) ✅
  - [x] 5.1: Created `.github/workflows/release.yml` ✅
  - [x] 5.2: Triggers on tag push (v*) ✅
  - [x] 5.3: Pre-release validation job (tests, clippy, docs) ✅
  - [x] 5.4: Publish job with CARGO_REGISTRY_TOKEN ✅
  - [x] 5.5: GitHub release creation with changelog notes ✅

- [x] **Task 6: Prepare GitHub Repository** (AC: #7) ✅
  - [x] 6.1: Repository must be public (USER ACTION REQUIRED) ✅ documented
  - [x] 6.2: CARGO_REGISTRY_TOKEN secret (USER ACTION REQUIRED) ✅ documented
  - [x] 6.3: Repository description: "High-performance terminal braille rendering for images, animations, and graphics" ✅ documented
  - [x] 6.4: Repository topics: terminal, braille, graphics, cli, rust, tui, rendering ✅ documented

- [ ] **Task 7: Publish to crates.io** (AC: #6, #8) ⏳ REQUIRES USER ACTION
  - [ ] 7.1: Create and push annotated tag: `git tag -a v0.1.0 -m "Release v0.1.0"`
  - [ ] 7.2: Push tag: `git push origin v0.1.0`
  - [ ] 7.3: Monitor CI release workflow execution
  - [ ] 7.4: Verify crate appears on crates.io
  - [ ] 7.5: Verify docs.rs builds successfully
  - [ ] 7.6: Verify GitHub release is created with notes

- [ ] **Task 8: Post-Release Verification** (AC: #9)
  - [ ] 8.1: Create fresh test project: `cargo new test-dotmax && cd test-dotmax`
  - [ ] 8.2: Install dotmax: `cargo add dotmax`
  - [ ] 8.3: Verify basic hello_braille example works
  - [ ] 8.4: Install with features: `cargo add dotmax -F image`
  - [ ] 8.5: Verify image rendering example works
  - [ ] 8.6: Install with all features: `cargo add dotmax -F image,svg`
  - [ ] 8.7: Document verification results
  - [ ] 8.8: Clean up test project

- [ ] **Task 9: Final Documentation Updates** (AC: All)
  - [ ] 9.1: Update README.md with crates.io badge (after publication)
  - [ ] 9.2: Update README.md with docs.rs badge
  - [ ] 9.3: Update README.md with version badge
  - [ ] 9.4: Verify all links to crates.io work
  - [ ] 9.5: Document all 9 ACs with evidence

## Dev Notes

### Context and Purpose

**Epic 7 Goal:** Transform working code into a polished, professional library through API refinement, comprehensive benchmarking, performance optimization, enhanced testing, documentation excellence, and publication to crates.io.

**Story 7.6 Focus:** This is the culmination of all development work - publishing dotmax v0.1.0 to crates.io. The library is feature-complete (Epics 1-6), tested (557+ tests), documented (Story 7.5), and optimized (Story 7.3). This story handles:
1. Preparing metadata and version
2. Creating release automation
3. Publishing to crates.io
4. Verifying the release works

**Value Delivered:** Public availability of dotmax for the Rust ecosystem.

### Pre-publication Checklist Reference

From tech spec, the pre-publication validation should include:
```bash
# Security audit
cargo audit

# License compliance
cargo deny check licenses

# Advisory database check
cargo deny check advisories

# Banned crates check
cargo deny check bans

# Full validation
cargo test --all-features
cargo clippy --all-features -- -D warnings
cargo doc --no-deps --all-features
cargo bench
cargo publish --dry-run
```

### Release Workflow Reference

From tech spec (§Workflows and Sequencing):
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

### Cargo.toml Metadata Reference

From tech spec (§Data Models):
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

### Allowed Licenses (cargo-deny)

From tech spec (§Dependency Security Validation):
- MIT
- Apache-2.0
- BSD-2-Clause
- BSD-3-Clause
- ISC
- Zlib

### Project Structure Notes

**Files to create:**
- `CHANGELOG.md` - Version history (Keep a Changelog format)
- `.github/workflows/release.yml` - Automated release workflow

**Files to update:**
- `Cargo.toml` - Version bump to 0.1.0, metadata polish
- `README.md` - Add crates.io/docs.rs badges post-publication

**Files verified complete:**
- `src/lib.rs` - Public API surface (Story 7.1)
- `benches/*.rs` - Benchmarking suite (Story 7.2)
- `tests/*.rs` - Comprehensive tests (Story 7.4)
- `examples/*.rs` - Example suite (Story 7.5)
- `docs/*.md` - Documentation guides (Story 7.5)

### Learnings from Previous Story

**From Story 7.5 (Status: done)**

Story 7.5 focused on comprehensive documentation and examples. All documentation is now in place:

- **README.md:** Updated with visual demos, comparison to alternatives, complete feature list
- **examples/README.md:** Categorized example index
- **docs/getting_started.md:** User onboarding tutorial
- **docs/performance.md:** Performance optimization guide
- **docs/troubleshooting.md:** Common issues and solutions
- **All doctests passing:** `cargo test --doc` exits 0
- **Zero clippy warnings:** All examples compile cleanly

**Implications for Story 7.6:**
- Documentation is ready for docs.rs publication
- README.md already has structure for badges (just needs real URLs after publication)
- All examples work and can be referenced in CHANGELOG
- 232+ doc tests ensure rustdoc examples are correct

[Source: docs/sprint-artifacts/7-5-write-comprehensive-documentation-and-examples.md]

### Version Strategy

From architecture (§Versioning Strategy):
- `0.x.y` - Pre-1.0: Minor version may break compatibility
- Starting with 0.1.0 is recommended (allows breaking changes before stabilization)
- Post-publication, follow semver strictly

### CHANGELOG Format

Following Keep a Changelog (https://keepachangelog.com/):
```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - YYYY-MM-DD

### Added
- Feature list...

### Changed
- Changes list...
```

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-7.md#Story-7.6] - Authoritative acceptance criteria (AC7.6.1-7.6.9)
- [Source: docs/sprint-artifacts/tech-spec-epic-7.md#Workflows-and-Sequencing] - Release workflow reference
- [Source: docs/architecture.md#Deployment-Architecture] - Deployment/release strategy
- [Source: docs/architecture.md#Versioning-Strategy] - Semver and MSRV policy
- [Source: docs/sprint-artifacts/7-5-write-comprehensive-documentation-and-examples.md] - Previous story context

## Dev Agent Record

### Context Reference

- docs/sprint-artifacts/7-6-publish-to-cratesio-and-create-release-process.context.xml

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

## Change Log

**2025-11-26 - Story Drafted**
- Story created by SM agent (claude-opus-4-5-20251101)
- Status: drafted (from backlog)
- Epic 7: API Design, Performance & Production Readiness
- Story 7.6: Publish to crates.io and Create Release Process
- Prerequisites: Stories 7.1-7.5 complete (API designed, benchmarks established, tests enhanced, docs written)
- Automated workflow execution: /bmad:bmm:workflows:create-story
