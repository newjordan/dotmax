# Sprint Change Proposal - Epic 3 Issue Resolution

**Date**: 2025-11-20
**Project**: dotmax
**Scope**: MINOR (Direct Implementation)
**Epic**: Epic 3 - 2D Image Rendering Pipeline
**Story**: Story 3.9 - Manual Testing, Validation, and Feedback Refinement
**Author**: Bob (Scrum Master)
**Approved By**: Frosty (Project Maintainer)

---

## Executive Summary

During manual human evaluation testing for Story 3.9, three critical functional issues were discovered that block Epic 3 completion and v0.1.0 release. These are implementation bugs, not scope issues. The recommended approach is **Option 1: Direct Adjustment** - fix all three issues within Story 3.9 without new stories or rollbacks.

**Timeline Impact**: v0.1.0 release delayed by 1-2 weeks (5-8 days for fixes + validation).

**Risk Level**: LOW - Isolated fixes, strong test coverage (234 tests), clear architectural patterns to follow.

**Approval Status**: ✅ APPROVED by Frosty on 2025-11-20

---

## Section 1: Issue Summary

### Context

**Discovery Phase**: Story 3.9 (Manual Testing, Validation, and Feedback Refinement)
**Discovery Method**: Human evaluation testing with real terminal interaction
**Current Status**: Epic 3 BLOCKED from completion, v0.1.0 release on hold

Manual testing revealed runtime behavior issues not caught by automated test suite (234 tests passing, but user-facing bugs exist). All three issues violate functional requirements and architectural patterns.

---

### Issue #1: Color Mode State Disconnection

**Severity**: HIGH
**Affected Story**: Story 3.7 (Implement Color Mode Image Rendering) - marked DONE

**Problem Statement**:
When user toggles color mode via 'C' key, the system creates an isolated/new render instead of modifying the current image's display state.

**Symptoms**:
- Image resizes to small corner element
- Ignores current dither settings
- Creates disconnected rendering operation with reset parameters
- Violates user expectation: color mode should be a "view mode switch" not "reload page"

**User Quote**:
> "when i swap color modes via C color, the image resizes to a small corner element, ignores dither, and is reverting to what looks like a very simple implementation. It is not applying color, or difference to the current image and dither type. its creating a new render, with its own, disconnected set of operations."

**Impact**:
- ❌ Violates FR17 (Render in color mode)
- ❌ Conflicts with Architecture Pattern 2 (Image-to-Braille Pipeline) - color should be final pipeline stage, not separate execution
- ⚠️ Blocks Epic 5 (Color System) - establishes wrong pattern for color state management

**Root Cause**: Story 3.7 implementation creates new render instead of modifying state.

---

### Issue #2: Incomplete Initial Renders

**Severity**: CRITICAL
**Affected Stories**: Root cause unknown - could be Story 2.3, 3.5, or 3.8

**Problem Statement**:
Images don't fully render on first display - some cell areas remain unfilled/blank with inconsistent behavior.

**Symptoms**:
- Inconsistent cell filling on initial render (doesn't happen every time)
- User must cycle through color mode (C key) to force clean re-render
- Suggests race condition, buffer initialization issue, or incomplete cell filling in render path

**User Quote**:
> "images are not completely loading and I have to cycle through color mode C - to get a clean output, we are having some cell areas not fill in data sometimes, and its inconsistant."

**Impact**:
- ❌ Violates FR4 (Render grid to terminal via ratatui/crossterm)
- ❌ Conflicts with Architecture Pattern 1 (Braille Dot Matrix Mapping) - should always produce complete output
- 🚫 Blocks production readiness - unreliable rendering is unacceptable for v0.1.0

**Root Cause**: Unknown - requires investigation (likely buffer init, terminal API misuse, or race condition).

---

### Issue #3: Performance/Responsiveness

**Severity**: MEDIUM
**Affected Story**: Story 3.8 (Create High-Level Image Rendering API) - marked ready-for-review

**Problem Statement**:
Brightness/contrast/gamma adjustments trigger full-page refresh with slow visual feedback, preventing interactive parameter tweaking.

**Symptoms**:
- Full pipeline re-execution on every parameter change
- Very slow visual process (estimated 1-2 seconds per adjustment)
- User expectation: "like an image editor" with live parameter tweaking
- Current behavior: "slow/reload complete refresh"

**User Quote**:
> "when I chaneg the brightness or contrast etc, it refreshes the whole page, and is a very slow visual process - is there a way we can tighten this up and make it more responsive? like an image editor and not a slow/reload complete refresh?"

**Impact**:
- ⚠️ Partially violates FR19 (Adjust brightness/contrast/gamma before rendering) - adjustments work but not interactive
- ❌ Conflicts with Architecture Performance Targets (<50ms pipeline, buffer reuse pattern)
- 🎯 Degrades user experience - prevents rapid iteration on image settings
- ⚠️ Blocks Epic 6 (Animation) - animation requires efficient buffer reuse, this indicates buffer management problems

**Root Cause**: ImageRenderer re-executes full pipeline on every parameter change, no caching strategy.

---

## Section 2: Impact Analysis

### Epic Impact Assessment

**Epic 3 (Current Epic) - 2D Image Rendering Pipeline**:
- **Status**: ⚠️ BLOCKED from completion
- **Stories Affected**:
  - Story 3.7 (Color Mode) - marked DONE, needs bug fix
  - Story 3.8 (High-Level API) - marked ready-for-review, needs enhancement
  - Story 3.9 (Manual Testing) - drafted, must expand to include investigation + fixes
- **Required Changes**:
  - Expand Story 3.9 acceptance criteria to include "all 3 issues investigated and fixed"
  - Add investigation tasks to Story 3.9 (root cause analysis for Issue #2)
  - Add fix implementation tasks to Story 3.9 (Issues #1, #2, #3)
  - May need to reopen Stories 3.7 or 3.8 for targeted fixes

**Epic 4 (Drawing Primitives) - Indirect Impact**:
- Depends on BrailleGrid from Epic 2 (Issue #2 may affect)
- **Risk**: If Issue #2 is in Epic 2 terminal rendering, Epic 4 may inherit bug
- **Mitigation**: Fix Issue #2 before starting Epic 4

**Epic 5 (Color System) - Moderate Impact**:
- Builds on Story 3.7 color mode
- **Risk**: Color scheme application may have same "disconnected state" problem as Issue #1
- **Mitigation**: Issue #1 fix establishes correct pattern for color state management

**Epic 6 (Animation) - High Impact**:
- Performance critical (<60fps target)
- **Risk**: Issue #3 indicates buffer management problems; animation requires efficient buffer reuse
- **Mitigation**: Issue #3 fix must establish caching pattern for Epic 6 to build on

**Epic 7 (Production Readiness) - Blocks Start**:
- Cannot start polish/optimization until Epic 3 actually works
- **No changes needed** - just blocked waiting for Epic 3 completion

### Artifact Conflict Summary

**Functional Requirements (PRD)**:
| Requirement | Issue | Status |
|-------------|-------|--------|
| FR17 (Render in color mode) | Issue #1 | ❌ VIOLATED - color mode should modify existing render, not create new one |
| FR4 (Render grid to terminal) | Issue #2 | ❌ VIOLATED - renders must be complete and consistent |
| FR19 (Adjust brightness/contrast/gamma) | Issue #3 | ⚠️ PARTIAL - adjustments work but not interactive |

**Architecture Document (docs/architecture.md)**:
| Pattern/Target | Issue | Status |
|----------------|-------|--------|
| Pattern 2: Image-to-Braille Pipeline (lines 338-405) | Issue #1 | ❌ VIOLATED - color should be final stage of single pipeline |
| Pattern 1: Braille Dot Matrix Mapping (lines 262-335) | Issue #2 | ❌ VIOLATED - should always produce complete output |
| Buffer Reuse Pattern (lines 941-952) | Issue #3 | ❌ VIOLATED - should reuse allocations, not recreate |
| Performance Targets (lines 912-977) | Issue #3 | ❌ VIOLATED - <50ms pipeline target compromised by full re-execution |

**Testing Strategy**:
- **Gap Identified**: 234 automated tests passing but missed these user-facing bugs
- **Required Enhancement**: Add interactive/integration tests simulating user parameter adjustments
- **Required Enhancement**: Add visual regression tests for complete render verification

**Documentation** (potentially):
- Examples may need updating if Issue #3 fix changes ImageRenderer API
- Performance docs may need updating if caching improves speed

### MVP Impact

**Timeline Impact**:
- **Before fixes**: Epic 3 completion ETA was ~1 week (Story 3.9 testing)
- **After fixes**: Epic 3 completion ETA is now ~2-3 weeks (5-8 days fixes + 1 day re-testing)
- **v0.1.0 release**: Delayed by 1-2 weeks

**Scope Impact**:
- ❌ NO CHANGE to MVP scope - These are implementation bugs, not scope issues
- ✅ MVP remains: "Core Rendering + 2D Image (v0.1.0 → v1.0)"

**Quality Standard**:
- ✅ MAINTAINED - "Professional quality, production-ready" - we will NOT ship with known bugs

---

## Section 3: Path Forward Evaluation

### Option 1: Direct Adjustment (RECOMMENDED ✅)

**Approach**: Fix all three issues within Story 3.9 by adding investigation and implementation tasks. No new stories needed, no rollbacks required.

**Detailed Fix Plan**:

**Issue #1: Color Mode State Disconnection**
- **Fix Location**: `src/image/color_mode.rs` and `src/image/mod.rs` (ImageRenderer)
- **Fix Strategy**:
  - Refactor color mode to be a **display parameter** not a re-render trigger
  - ImageRenderer caches base grayscale/binary image
  - Color mode toggle applies color to cached data without re-executing pipeline
  - Preserves dither settings and other parameters
- **Estimated Effort**: 1-2 days (50-150 lines changed)
- **Risk**: LOW - isolated refactoring, well-defined scope

**Issue #2: Incomplete Initial Renders**
- **Fix Location**: TBD after investigation (likely `src/grid.rs`, `src/render.rs`, or `src/image/mapper.rs`)
- **Fix Strategy**:
  - **Phase 1**: Investigate root cause (buffer init? terminal API? race condition?)
  - **Phase 2**: Implement targeted fix based on findings
- **Estimated Effort**: 0.5-1 day investigation + 0.5-1 day fix (10-50 lines changed)
- **Risk**: LOW-MEDIUM - depends on root cause, but likely small bug fix

**Issue #3: Performance/Responsiveness**
- **Fix Location**: `src/image/mod.rs` (ImageRenderer)
- **Fix Strategy**: Implement caching for intermediate pipeline results
  - **Cache**: Load → Resize → Grayscale (slow stages, ~40ms)
  - **Re-run**: Adjustments → Dither → Map → Color (fast stages, ~10ms)
  - Parameter changes only re-run from cached grayscale forward
  - Establishes pattern for Epic 6 (animation buffer reuse)
- **Estimated Effort**: 2-3 days (100-200 lines added)
- **Risk**: LOW - architecture already mandates buffer reuse pattern

**Total Effort**: 4-7 days development + 1 day re-testing = **5-8 days total**

**Evaluation**:
- ✅ **Implementation Effort**: 4-7 days (fastest option)
- ✅ **Technical Risk**: LOW (isolated fixes, 234 tests catch regressions)
- ✅ **Team Morale**: Positive (fix bugs, learn, move forward)
- ✅ **Long-Term**: Best (establishes patterns for Epics 5, 6)
- ✅ **Stakeholder Value**: Delivers working Epic 3 fastest with quality

**Viability**: ✅ **VIABLE** - This is the recommended approach.

---

### Option 2: Potential Rollback (REJECTED ❌)

**Approach**: Revert recently completed stories (3.7, 3.8) and re-implement with fixes.

**Stories to Rollback**:
- Story 3.7 (Color Mode) - marked DONE
- Story 3.8 (High-Level API) - marked ready-for-review

**Analysis**:
- ❌ Rollback provides no benefit
- ❌ Loses 2 completed stories (1000+ lines of working, tested code)
- ❌ Re-implementation risk: might introduce new bugs
- ❌ Higher effort: 8-11 days (vs 4-7 days for direct fix)
- ❌ Higher risk: lose proven working code (234 tests passing)
- ❌ Demoralizing for team

**Viability**: ❌ **NOT VIABLE** - Rollback is wasteful and risky.

---

### Option 3: PRD MVP Review (REJECTED ❌)

**Approach**: Reduce Epic 3 scope, defer color mode or high-level API to post-MVP.

**Analysis**:
- ⚠️ MVP technically achievable with bugs, but compromised quality
- ❌ Doesn't solve the actual problems - just declares them "out of scope"
- ❌ Issue #1 is a bug, not a feature - can't "defer" a bug
- ❌ Issue #2 is critical - renders must work correctly for MVP
- ❌ Issue #3 could be deferred, but users will complain immediately
- ❌ Shipping known bugs damages reputation
- ❌ "Works but buggy" is worse than "delayed but quality"

**Viability**: ❌ **NOT VIABLE** - Scope reduction doesn't solve problems, compromises quality standards.

---

## Section 4: Recommended Approach

### Selected: Option 1 - Direct Adjustment

**Rationale**: Fix bugs in place within Story 3.9. This is fastest, lowest risk, maintains quality standards, and establishes correct patterns for future epics (5, 6).

**Trade-offs Acknowledged**:

**Pros**:
- ✅ Fastest path to working Epic 3 (5-8 days)
- ✅ Maintains all completed work (Stories 3.1-3.8)
- ✅ Establishes caching pattern needed for Epic 6
- ✅ Low technical risk with strong test coverage (234 tests)
- ✅ Maintains team momentum and quality standards

**Cons**:
- ⚠️ 5-8 day delay before Epic 3 completion
- ⚠️ Issue #2 root cause unknown (investigation required)
- ⚠️ May need minor API changes if caching affects ImageRenderer interface

---

## Section 5: Implementation Plan

### High-Level Action Plan

#### Phase 1: Investigation (Days 1-2)

**Task 1.1: Root Cause Analysis for Issue #2** (1 day)
- Add tracing/logging to terminal rendering pipeline
- Test with multiple images and terminal sizes
- Check BrailleGrid buffer initialization
- Check terminal API calls (crossterm/ratatui)
- Identify exact failure point

**Task 1.2: Design Caching Strategy for Issue #3** (1 day)
- Review ImageRenderer pipeline stages
- Identify cacheable intermediate results
- Design cache invalidation rules (when to clear cache)
- Document caching API (if user-facing changes needed)

**Checkpoint 1** (Day 2): Investigation complete
- Dev agent reports Issue #2 root cause
- Dev agent presents caching design for Issue #3
- **Go/No-Go**: Frosty approves fix approach

---

#### Phase 2: Implementation (Days 3-6)

**Task 2.1: Fix Issue #1 - Color Mode Refactor** (2 days)
- Refactor `src/image/color_mode.rs` to work with cached data
- Update ImageRenderer to cache base image
- Ensure color mode preserves all other parameters
- Update unit tests (color_mode.rs)
- Update integration tests

**Task 2.2: Fix Issue #2 - Incomplete Renders** (1 day)
- Implement fix based on root cause analysis from Task 1.1
- Add targeted tests to prevent regression
- Verify fix across multiple platforms (CI)

**Task 2.3: Fix Issue #3 - Implement Caching** (3 days)
- Add caching fields to ImageRenderer struct
- Implement cache invalidation logic
- Update `.render()` to use cached data when possible
- Update parameter adjustment methods (brightness/contrast/gamma)
- Add cache performance benchmarks
- Update examples if API changed

**Checkpoint 2** (Day 6): Implementation complete
- All code changes committed
- All tests updated and passing locally
- **Go/No-Go**: Dev agent signals ready for validation

---

#### Phase 3: Validation (Days 7-8)

**Task 3.1: Re-run Manual Testing** (1 day)
- Test all 3 issues are resolved
- Test with diverse images (formats, sizes, aspect ratios)
- Test all dithering methods with color modes
- Test parameter adjustments are responsive
- Verify no regressions in existing functionality

**Task 3.2: Automated Test Suite Validation** (0.5 day)
- Run full test suite: `cargo test --all-features`
- Verify all 234+ tests still passing
- Run clippy: `cargo clippy -- -D warnings`
- Run benchmarks: ensure performance targets met (<50ms)

**Task 3.3: Cross-Platform CI Verification** (0.5 day)
- Verify CI passes on Windows, Linux, macOS
- Check for platform-specific rendering differences
- Verify examples compile and run

**Checkpoint 3** (Day 8): Validation complete
- Manual testing passed
- CI green on all platforms
- **Go/No-Go**: Frosty signs off on Epic 3 completion

---

#### Phase 4: Completion (Day 8)

**Task 4.1: Update Documentation**
- Update Story 3.9 completion notes
- Update sprint-status.yaml (mark Epic 3 DONE)
- Update CHANGELOG.md with bug fixes
- Update examples/README.md if API changed

**Task 4.2: Epic 3 Sign-Off**
- Confirm all 9 Epic 3 stories complete
- Confirm all ACs met for Stories 3.1-3.9
- Document Epic 3 as production-ready
- Prepare for Epic 4 start

---

### Dependencies and Sequencing

**Critical Path**:
1. Issue #2 investigation MUST complete before Phase 2 (determines fix approach)
2. All fixes MUST complete before Phase 3 (can't validate partial fixes)
3. Manual re-testing MUST pass before Epic 3 sign-off

**Parallel Work Possible**:
- Issue #1 and #3 fixes can proceed in parallel (independent)
- Documentation updates can happen during Phase 3 validation

**Blocking Relationships**:
- Epic 4 BLOCKED until Epic 3 complete
- v0.1.0 release BLOCKED until Epic 3 complete
- Epic 6 caching pattern depends on Issue #3 fix

---

## Section 6: Handoff and Success Criteria

### Scope Classification: MINOR

**Rationale**: These are bug fixes and enhancements within a single epic (Epic 3). No PRD changes, no epic reordering, no major architectural shifts. Direct implementation by dev team.

### Handoff: Development Team (Dev Agent)

**Responsibility**: Implement all fixes within Story 3.9

**Deliverables**:
1. **Code Fixes**:
   - Issue #1 fix in `src/image/color_mode.rs` and `src/image/mod.rs`
   - Issue #2 fix (location TBD after investigation)
   - Issue #3 caching implementation in `src/image/mod.rs`

2. **Testing**:
   - Updated unit tests for all affected modules
   - Integration tests for end-to-end validation
   - Manual testing checklist completion for Story 3.9

3. **Documentation**:
   - Updated rustdoc if API changed
   - Updated examples if needed
   - Story 3.9 completion notes

### Success Criteria

**Definition of Done for Story 3.9**:
- ✅ All 3 issues investigated and root causes identified
- ✅ All 3 issues fixed and verified working
- ✅ All 234+ tests passing (no regressions)
- ✅ Zero clippy warnings
- ✅ Manual testing checklist complete (all scenarios pass)
- ✅ CI passing on Windows, Linux, macOS
- ✅ Performance targets met (<50ms pipeline)
- ✅ Examples compile and run successfully

**Definition of Done for Epic 3**:
- ✅ All stories 3.1-3.9 complete
- ✅ All Epic 3 functional requirements (FR9-FR20) met
- ✅ No blocking issues remain
- ✅ Production-ready for v0.1.0 release

### Timeline and Checkpoints

| Checkpoint | Day | Criteria | Approver |
|------------|-----|----------|----------|
| **Checkpoint 1** | Day 2 | Investigation complete, fix approach approved | Frosty |
| **Checkpoint 2** | Day 6 | Implementation complete, tests passing locally | Dev Agent |
| **Checkpoint 3** | Day 8 | Validation complete, CI green, manual testing passed | Frosty |

### Communication Plan

**Daily Updates**:
- Dev agent reports progress at end of each day
- Any blockers escalated immediately to Frosty

**Decision Points**:
- Issue #2 fix approach (Day 2)
- Caching API design if user-facing (Day 2)
- Any scope adjustments if issues bigger than expected

---

## Section 7: Approval Record

**Proposal Created**: 2025-11-20
**Created By**: Bob (Scrum Master)
**Reviewed By**: Frosty (Project Maintainer)
**Approval Status**: ✅ APPROVED
**Approval Date**: 2025-11-20
**Approval Method**: Interactive correct-course workflow

**Approver Comments**: "perfect, yes continue. we are on target" → "a" (approved)

---

## Section 8: References

**Documents**:
- [Source: docs/epics.md] - Epic 3 breakdown and story definitions
- [Source: docs/architecture.md] - Architectural patterns and performance targets
- [Source: docs/sprint-artifacts/3-7-implement-color-mode-image-rendering.md] - Story 3.7 specification
- [Source: docs/sprint-artifacts/3-8-create-high-level-image-rendering-api.md] - Story 3.8 specification
- [Source: docs/sprint-artifacts/3-9-manual-testing-validation-and-feedback-refinement.md] - Story 3.9 specification
- [Source: docs/sprint-artifacts/sprint-status.yaml] - Current sprint status

**Issues**:
- Issue #1: Color Mode State Disconnection (HIGH severity)
- Issue #2: Incomplete Initial Renders (CRITICAL severity)
- Issue #3: Performance/Responsiveness (MEDIUM severity)

**Related Workflows**:
- [Workflow: correct-course] - This proposal generated via correct-course workflow
- [Workflow: dev-story] - Will be used to execute Story 3.9 fixes

---

## Appendix: Checklist Completion Record

**Section 1: Understand Trigger and Context** ✅
- 1.1: Triggering story identified
- 1.2: Core problems precisely defined
- 1.3: Evidence gathered

**Section 2: Epic Impact Assessment** ✅
- 2.1: Current epic evaluated
- 2.2: Required epic changes determined
- 2.3: Future epics reviewed
- 2.4: No epics invalidated
- 2.5: Epic order unchanged

**Section 3: Artifact Conflict Analysis** ✅
- 3.1: PRD/FRs checked
- 3.2: Architecture reviewed
- 3.3: UI/UX specs examined
- 3.4: Other artifacts considered

**Section 4: Path Forward Evaluation** ✅
- 4.1: Option 1 (Direct Adjustment) - VIABLE
- 4.2: Option 2 (Rollback) - NOT VIABLE
- 4.3: Option 3 (Scope Reduction) - NOT VIABLE
- 4.4: Option 1 selected

**Section 5: Sprint Change Proposal Components** ✅
- 5.1: Issue summary created
- 5.2: Epic impact documented
- 5.3: Path forward presented
- 5.4: MVP impact defined
- 5.5: Handoff plan established

**Section 6: Final Review and Handoff** ✅
- 6.1: Checklist completion verified
- 6.2: Proposal accuracy verified
- 6.3: User approval obtained
- 6.4: Next steps confirmed

---

**End of Sprint Change Proposal**
