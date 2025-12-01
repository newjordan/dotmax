# Story 7.7: Create Proof-of-Concept Integration (yazi or bat)

Status: drafted

## Story

As a **library maintainer validating real-world usability**,
I want **a proof-of-concept integration with an existing Rust TUI application**,
so that **I can validate the API design and collect external feedback before v1.0**.

## Acceptance Criteria

1. **AC1: Integration target selected**
   - Choose yazi (file manager) or bat (file viewer) with documented rationale
   - Consider: maintainer responsiveness, community size, integration complexity
   - Alternative targets if primary rejected: hx/helix, gitui

2. **AC2: Integration < 100 lines**
   - Core dotmax usage in < 100 LOC (excluding boilerplate)
   - Demonstrates library ease-of-use
   - Clean, idiomatic Rust code

3. **AC3: Feature works end-to-end**
   - Image preview renders correctly in target application
   - Braille output displays properly in terminal
   - No crashes or panics during normal use

4. **AC4: PR submitted (optional)**
   - Draft PR or fork demonstrating integration
   - Code follows target project's style guidelines
   - Include description explaining dotmax benefits

5. **AC5: Performance acceptable**
   - No visible lag in target application
   - Image renders within <100ms (user-perceivable threshold)
   - No UI blocking or freezing

6. **AC6: Feedback collected**
   - At least 1 external user/maintainer feedback
   - Document reactions to API, performance, output quality
   - Note any confusion or friction points

7. **AC7: Lessons documented**
   - `docs/integration-lessons.md` with findings
   - What worked well, what was difficult
   - Recommendations for API improvements

8. **AC8: API pain points identified**
   - List of usability issues discovered during integration
   - Categorize by severity (blocking, annoying, minor)
   - Propose solutions where applicable

9. **AC9: Improvements backlogged**
   - GitHub issues created for identified improvements
   - Link issues to specific pain points
   - Prioritize for post-v1.0 roadmap

## Tasks / Subtasks

- [ ] **Task 1: Research and Select Integration Target** (AC: #1)
  - [ ] 1.1: Research yazi - GitHub activity, maintainer responsiveness, Discord/community
  - [ ] 1.2: Research bat - GitHub activity, maintainer responsiveness, integration points
  - [ ] 1.3: Evaluate integration complexity for each (image preview hooks, plugin system)
  - [ ] 1.4: Check existing image preview implementations in each project
  - [ ] 1.5: Document selection rationale in story notes
  - [ ] 1.6: Reach out to maintainers (optional) to gauge interest

- [ ] **Task 2: Set Up Integration Development Environment** (AC: #1)
  - [ ] 2.1: Fork/clone selected target project
  - [ ] 2.2: Build and run target project successfully
  - [ ] 2.3: Understand project structure and contribution guidelines
  - [ ] 2.4: Identify integration points (preview hooks, renderer interface)
  - [ ] 2.5: Set up local dotmax dependency (path or git)

- [ ] **Task 3: Implement POC Integration** (AC: #2, #3)
  - [ ] 3.1: Create integration module/file in target project
  - [ ] 3.2: Import dotmax with required features (image, svg if needed)
  - [ ] 3.3: Implement image-to-braille rendering function
  - [ ] 3.4: Hook into target project's preview system
  - [ ] 3.5: Handle terminal size detection and grid sizing
  - [ ] 3.6: Add error handling for unsupported formats
  - [ ] 3.7: Verify LOC count < 100 for core integration
  - [ ] 3.8: Test with various image formats (PNG, JPG, GIF, SVG)

- [ ] **Task 4: Validate Performance** (AC: #5)
  - [ ] 4.1: Measure render time for typical images
  - [ ] 4.2: Test with large images (verify no UI blocking)
  - [ ] 4.3: Test rapid navigation (no lag when switching files)
  - [ ] 4.4: Profile if performance issues found
  - [ ] 4.5: Document performance characteristics

- [ ] **Task 5: Prepare PR or Fork** (AC: #4)
  - [ ] 5.1: Clean up code to match target project style
  - [ ] 5.2: Add inline documentation/comments
  - [ ] 5.3: Write PR description explaining dotmax benefits
  - [ ] 5.4: Include before/after screenshots or GIFs
  - [ ] 5.5: Submit draft PR or publish fork

- [ ] **Task 6: Collect External Feedback** (AC: #6)
  - [ ] 6.1: Share POC with target project maintainers
  - [ ] 6.2: Request feedback on API usability
  - [ ] 6.3: Ask about output quality satisfaction
  - [ ] 6.4: Document all feedback received
  - [ ] 6.5: Follow up on any questions or concerns

- [ ] **Task 7: Document Lessons Learned** (AC: #7, #8)
  - [ ] 7.1: Create `docs/integration-lessons.md`
  - [ ] 7.2: Document what worked well (easy parts of integration)
  - [ ] 7.3: Document friction points (difficult or confusing)
  - [ ] 7.4: List API pain points with severity ratings
  - [ ] 7.5: Propose solutions for each pain point
  - [ ] 7.6: Add recommendations for future integrations

- [ ] **Task 8: Create Backlog Issues** (AC: #9)
  - [ ] 8.1: Create GitHub issues for each identified improvement
  - [ ] 8.2: Link issues to specific pain points in lessons doc
  - [ ] 8.3: Add labels (enhancement, api, documentation, etc.)
  - [ ] 8.4: Prioritize issues for post-v1.0 roadmap
  - [ ] 8.5: Update docs/integration-lessons.md with issue links

- [ ] **Task 9: Final Validation** (AC: All)
  - [ ] 9.1: Verify integration target selection documented (AC1)
  - [ ] 9.2: Count LOC and verify < 100 (AC2)
  - [ ] 9.3: Test end-to-end functionality (AC3)
  - [ ] 9.4: Verify PR/fork exists if applicable (AC4)
  - [ ] 9.5: Confirm no performance issues (AC5)
  - [ ] 9.6: Verify feedback documented (AC6)
  - [ ] 9.7: Verify lessons document exists (AC7)
  - [ ] 9.8: Verify pain points listed (AC8)
  - [ ] 9.9: Verify GitHub issues created (AC9)

## Dev Notes

### Context and Purpose

**Epic 7 Goal:** Transform working code into a polished, professional library through API refinement, comprehensive benchmarking, performance optimization, enhanced testing, documentation excellence, and publication to crates.io.

**Story 7.7 Focus:** This is the final validation story before v1.0. By integrating dotmax into a real-world Rust TUI application, we validate that:
1. The API is intuitive and easy to use
2. Performance meets real-world needs
3. Documentation is sufficient for external developers
4. Output quality satisfies users

**Value Delivered:** External validation and feedback that ensures dotmax is production-ready.

### Integration Target Analysis

**Primary Targets:**

| Target | Type | Stars | Integration Points | Complexity |
|--------|------|-------|-------------------|------------|
| **yazi** | File manager | 18k+ | Preview plugins, image preview | Medium |
| **bat** | File viewer | 50k+ | Syntax themes, custom printing | Low |

**Secondary Targets (if primary rejected):**

| Target | Type | Stars | Notes |
|--------|------|-------|-------|
| **helix/hx** | Text editor | 35k+ | Image preview in editor splits |
| **gitui** | Git TUI | 18k+ | Diff visualization |

**Recommendation:** Start with **yazi** due to:
1. Active image preview ecosystem (sixel, kitty protocol already supported)
2. Braille would add universal terminal support
3. Maintainer is responsive (check GitHub issues)
4. Plugin architecture makes integration cleaner

### Integration Code Template

```rust
// Expected ~50-80 lines for core integration
use dotmax::{BrailleGrid, TerminalRenderer, ImageRenderer, DitherMethod};

pub fn render_image_preview(
    path: &Path,
    width: u16,
    height: u16,
) -> Result<String, dotmax::DotmaxError> {
    // Load and render image to braille
    let renderer = ImageRenderer::builder()
        .dither_method(DitherMethod::FloydSteinberg)
        .build();

    let grid = renderer.render_file(path, width as usize, height as usize)?;

    // Convert to string for output
    Ok(grid.to_string())
}
```

### Project Structure Notes

**Files to create:**
- `docs/integration-lessons.md` - Lessons learned document

**External artifacts:**
- Fork or branch of target project
- Draft PR (if submitted)
- GitHub issues for improvements

### Learnings from Previous Story

**From Story 7.6 (Status: backlog)**

Story 7.6 (crates.io publication) is a prerequisite for full external validation since `cargo add dotmax` won't work until published. Options:
1. Use git dependency in POC: `dotmax = { git = "https://github.com/newjordan/dotmax" }`
2. Use path dependency for local testing
3. Complete Story 7.6 first, then do POC with published crate

**From Story 7.5 (Status: done)**

Documentation is complete:
- `docs/getting_started.md` - Tutorial for new users
- `docs/performance.md` - Optimization guidance
- `docs/troubleshooting.md` - Common issues
- `examples/README.md` - Example index

These resources can be shared with POC feedback reviewers.

### Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Integration LOC | < 100 | Code line count |
| Render latency | < 100ms | Timer measurement |
| Feedback responses | ≥ 1 | Documentation |
| Pain points identified | ≥ 3 | Lessons doc |
| Issues created | = pain points | GitHub |

### References

- [Source: docs/sprint-artifacts/tech-spec-epic-7.md#Story-7.7] - Authoritative acceptance criteria (AC7.7.1-7.7.9)
- [Source: docs/sprint-artifacts/tech-spec-epic-7.md#Integration-Points] - Integration target analysis
- [Source: docs/architecture.md] - API design patterns
- [Source: docs/getting_started.md] - User onboarding documentation
- [Source: docs/performance.md] - Performance optimization guidance

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

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
- Story 7.7: Create Proof-of-Concept Integration (yazi or bat)
- Prerequisites: Story 7.6 (publication) recommended but not required (can use git dependency)
- Automated workflow execution: /bmad:bmm:workflows:create-story
