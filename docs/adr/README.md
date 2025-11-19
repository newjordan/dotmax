# Architecture Decision Records (ADR)

## Purpose

This directory contains Architecture Decision Records (ADRs) documenting significant design decisions made in the dotmax project. ADRs help maintain project context across development gaps, supporting NFR-M2 (Resumable Development) by capturing WHY decisions were made, not just WHAT was decided.

## ADR Lifecycle

**Status States:**
- **Proposed**: Decision under consideration (draft)
- **Accepted**: Decision approved and being followed
- **Deprecated**: Decision outdated but not replaced
- **Superseded**: Decision replaced by newer ADR (see reference)

**Immutability Rule**: ADRs are immutable once accepted. To change a decision, create a new ADR that supersedes the old one.

## Guidelines

1. **Numbering**: Sequential format (0001, 0002, etc.) for sortable chronological order
2. **Conciseness**: Keep ADRs to 1-2 pages maximum, link to architecture/PRD for details
3. **Completeness**: All ADRs must include Status, Context, Decision, Consequences, Alternatives Considered
4. **Code Links**: Reference relevant ADRs in code comments (e.g., `// See ADR-0001`)
5. **Updates**: Never edit accepted ADRs - supersede with new ADR if decision changes

## Creating New ADRs

1. Copy `template.md` to `NNNN-decision-title.md` (next sequential number)
2. Fill in all sections completely
3. Set status to "Proposed"
4. Review and update status to "Accepted" when decision is final
5. Add entry to index table below

## ADR Index

| Number | Title | Status | Date |
|--------|-------|--------|------|
| [0001](0001-use-braille-unicode-for-rendering.md) | Use Unicode Braille (U+2800-U+28FF) for Terminal Rendering | Accepted | 2025-11-17 |

---

**Template**: See [template.md](template.md) for ADR structure.
