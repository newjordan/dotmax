# Story 1.1: Initialize Cargo Project with Optimal Structure

Status: ready-for-dev

## Story

As a **Rust library developer**,
I want a properly initialized Cargo project with clean structure and metadata,
so that the crate is ready for professional development and crates.io publication.

## Acceptance Criteria

1. Project initializes with `cargo new --lib dotmax` producing a compilable library crate
2. `Cargo.toml` contains complete metadata (name, version 0.1.0, authors, edition 2021, rust-version 1.70, description, license, repository, keywords, categories)
3. `src/lib.rs` exists as entry point with initial module structure documentation
4. `.gitignore` configured for Rust projects (target/, Cargo.lock exclusion for libraries)
5. Directory structure created: `src/`, `examples/`, `tests/`, `benches/`, `docs/`
6. `README.md` exists with placeholder sections (Installation, Quick Start, Features, License)
7. `LICENSE-MIT` and `LICENSE-APACHE` files present (dual licensing)
8. `CHANGELOG.md` initialized with v0.1.0 (unreleased) entry
9. `cargo build` succeeds without warnings on local platform
10. `cargo test` passes (empty test suite is acceptable)

## Tasks / Subtasks

- [ ] Task 1: Initialize Cargo library project (AC: #1, #3)
  - [ ] Run `cargo new --lib dotmax` or `cargo init --lib` in project root
  - [ ] Verify src/lib.rs is created and contains default library template
  - [ ] Verify Cargo.toml is created with basic package metadata

- [ ] Task 2: Configure Cargo.toml with complete metadata (AC: #2)
  - [ ] Set package.name = "dotmax"
  - [ ] Set package.version = "0.1.0"
  - [ ] Set package.authors = ["Frosty"]
  - [ ] Set package.edition = "2021"
  - [ ] Set package.rust-version = "1.70" (MSRV)
  - [ ] Set package.description = "High-performance terminal braille rendering for images, animations, and graphics"
  - [ ] Set package.license = "MIT OR Apache-2.0"
  - [ ] Set package.repository = "https://github.com/frosty40/dotmax"
  - [ ] Set package.keywords = ["terminal", "braille", "graphics", "cli", "visualization"]
  - [ ] Set package.categories = ["command-line-interface", "graphics", "rendering"]

- [ ] Task 3: Create project directory structure (AC: #5)
  - [ ] Create examples/ directory
  - [ ] Create tests/ directory
  - [ ] Create benches/ directory
  - [ ] Create docs/ directory
  - [ ] Create docs/adr/ directory for Architecture Decision Records

- [ ] Task 4: Configure .gitignore for Rust (AC: #4)
  - [ ] Add target/ to .gitignore (build artifacts)
  - [ ] Add Cargo.lock to .gitignore (libraries don't commit lock file)
  - [ ] Add **/*.rs.bk to .gitignore (rustfmt backups)
  - [ ] Add .idea/, .vscode/ to .gitignore (IDE files)
  - [ ] Add *.swp, *.swo to .gitignore (editor swap files)

- [ ] Task 5: Create README.md with placeholder content (AC: #6)
  - [ ] Add project title and one-line description
  - [ ] Add Installation section (placeholder: "Coming soon to crates.io")
  - [ ] Add Quick Start section (placeholder code example)
  - [ ] Add Features section (placeholder bullet list)
  - [ ] Add License section (MIT OR Apache-2.0)
  - [ ] Add repository link

- [ ] Task 6: Add dual license files (AC: #7)
  - [ ] Create LICENSE-MIT file with standard MIT license text
  - [ ] Create LICENSE-APACHE file with Apache 2.0 license text
  - [ ] Update license headers to reference dual licensing

- [ ] Task 7: Initialize CHANGELOG.md (AC: #8)
  - [ ] Create CHANGELOG.md with "Keep a Changelog" format
  - [ ] Add [Unreleased] section
  - [ ] Add v0.1.0 section with "Initial project setup" entry

- [ ] Task 8: Validate build and test (AC: #9, #10)
  - [ ] Run `cargo build` and verify success with zero warnings
  - [ ] Run `cargo test` and verify success (empty test suite passes)
  - [ ] Run `cargo check` to verify project metadata is valid

## Dev Notes

### Project Structure Alignment

This story creates the foundation directory structure as specified in the Architecture Document (docs/architecture.md):

**Directory Layout Created:**
- `src/` - Core library code (lib.rs entry point)
- `examples/` - Runnable examples (Story 1.7 will populate)
- `tests/` - Integration tests (Epic 7 will populate)
- `benches/` - Criterion.rs benchmarks (Story 1.6 will populate)
- `docs/` - Documentation and ADRs (Story 1.5 will populate)

**Cargo Metadata Standards:**
- Edition 2021: Latest Rust idioms and features
- MSRV 1.70: Balances modern features vs. broad compatibility
- Dual MIT/Apache-2.0: Matches Rust std library, maximizes adoption
- Repository URL: https://github.com/frosty40/dotmax (update if different)

### Architecture Patterns to Follow

**From Architecture Document (docs/architecture.md):**

1. **Naming Conventions** (Section: Implementation Patterns):
   - Files and modules: `snake_case`
   - Types (structs, enums, traits): `PascalCase`
   - Functions, variables, methods: `snake_case`
   - Constants: `SCREAMING_SNAKE_CASE`

2. **Module Organization** (Section: Project Structure):
   - One primary type per file, matching filename
   - Feature-based modules (grid, render, image, primitives, color, animation)
   - Tests in same file under `#[cfg(test)]`

3. **Error Handling** (Section: Error Handling):
   - All public functions return `Result<T, DotmaxError>`
   - Use thiserror crate for error derivation (Story 1.3 will add dependency)

4. **No Unsafe Code** (Section: Security Architecture):
   - Zero unsafe code in MVP
   - Rely on Rust's memory safety guarantees

### Testing Standards

**From Epic 1 Tech Spec (docs/sprint-artifacts/tech-spec-epic-1.md):**

This story requires minimal testing since it's infrastructure setup:
- Build test: `cargo build` succeeds
- Test suite: `cargo test` passes (empty suite is acceptable)
- Future stories will add unit tests, integration tests, and benchmarks

### References

- [Source: docs/architecture.md#Project-Structure] - Directory layout specification
- [Source: docs/architecture.md#Project-Initialization] - Cargo initialization commands
- [Source: docs/architecture.md#Decision-Summary] - Rust 2021 edition, MSRV 1.70 decisions
- [Source: docs/sprint-artifacts/tech-spec-epic-1.md#Data-Models-and-Contracts] - Cargo.toml structure example
- [Source: docs/epics.md#Story-1.1] - Original story acceptance criteria
- [Source: docs/PRD.md#Non-Functional-Requirements] - NFR-L1 (MIT/Apache-2.0 dual licensing)

### Implementation Guidance

**Cargo.toml Initial State:**
This story creates a minimal Cargo.toml with metadata only. Dependencies will be added in Story 1.3 (core deps) and later stories (optional feature-flagged deps).

**src/lib.rs Initial Content:**
Keep minimal for now - just a module documentation comment and empty exports. Structure:
```rust
//! dotmax - High-performance terminal braille rendering
//!
//! This library provides braille-based rendering capabilities for terminal applications,
//! enabling images, animations, and graphics in any terminal environment.

// Module structure will be populated in Epic 2+
// Core modules (Epic 2): grid, render, error
// Feature modules (Epic 3+): image, primitives, color, animation
```

**README.md Guidance:**
Keep minimal and accurate. Don't promise features that don't exist yet. Focus on:
- What dotmax will be (high-performance braille rendering)
- Installation method (cargo add dotmax - note "coming soon")
- License (MIT OR Apache-2.0)
- Basic placeholder example (will be replaced in Story 1.7)

**CHANGELOG.md Format:**
Follow [Keep a Changelog](https://keepachangelog.com/) format:
- Versions are `## [version] - YYYY-MM-DD`
- Unreleased section at top
- Categories: Added, Changed, Deprecated, Removed, Fixed, Security

### Constraints and Gotchas

1. **MSRV 1.70**: Ensure rust-version in Cargo.toml is set to "1.70" for CI validation in Story 1.2
2. **Cargo.lock for libraries**: Should NOT be committed to git (add to .gitignore)
3. **Repository URL**: Verify GitHub repository name - using "frosty40/dotmax" as assumed URL
4. **License files**: Use exact license text from https://opensource.org/licenses/MIT and https://www.apache.org/licenses/LICENSE-2.0.txt
5. **Edition 2021**: Required for latest Rust idioms - don't use older editions

### Dependencies

**Story Dependencies:**
- None - this is Story 1.1, the first story in Epic 1

**Technical Dependencies:**
- Rust 1.70+ toolchain installed
- Cargo package manager
- Git (for repository initialization)

**Follow-on Stories:**
- Story 1.2: CI/CD pipeline (depends on this structure existing)
- Story 1.3: Core dependencies (adds to Cargo.toml created here)
- Story 1.4: Quality tooling (depends on Cargo project existing)

## Dev Agent Record

### Context Reference

- `docs/sprint-artifacts/stories/1-1-initialize-cargo-project-with-optimal-structure.context.xml` - Generated 2025-11-16

### Agent Model Used

<!-- Agent will fill this in during execution -->

### Debug Log References

<!-- Agent will add debug log references during implementation -->

### Completion Notes List

<!-- Agent will document:
- Actual repository URL used (if different from frosty40/dotmax)
- Any deviations from standard Cargo.toml structure
- Initial src/lib.rs content decisions
- README.md example code chosen
- Build validation results
-->

### File List

<!-- Agent will list files created:
- Cargo.toml
- src/lib.rs
- .gitignore
- README.md
- LICENSE-MIT
- LICENSE-APACHE
- CHANGELOG.md
- Directory creation (examples/, tests/, benches/, docs/, docs/adr/)
-->
