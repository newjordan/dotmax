# ADR-0001: Use Unicode Braille (U+2800-U+28FF) for Terminal Rendering

**Status**: Accepted

**Date**: 2025-11-17

## Context

**Problem**: dotmax needs to render high-resolution graphics (images, animations, primitives) in terminal environments. The solution must work across all platforms (Windows, Linux, macOS), inside terminal multiplexers (tmux, screen), and over SSH connections without requiring specific terminal emulator support.

**Constraints**:
- Must work on all Unicode-capable terminals (universal compatibility)
- Must survive terminal multiplexers (tmux, screen) without breaking
- Must work over SSH and remote connections (no binary protocol negotiation)
- Must provide reasonable resolution for images and graphics
- Must be copy/paste friendly (text-based, not binary)

**Background**: Modern terminal emulators support various graphics protocols (Sixel, Kitty, iTerm2 inline images), but adoption is inconsistent and multiplexers break most protocols. The Rust crates.io ecosystem needs a rendering solution that works everywhere without terminal-specific feature detection.

## Decision

**What we decided**: Use Unicode braille characters (U+2800 through U+28FF, 256 characters total) for all terminal rendering in dotmax. Each terminal character cell represents a 2×4 grid of "pixels" (braille dots), providing 8 individually addressable dots per cell.

**Rationale**:
- **Universal Support**: Unicode braille is supported by all modern terminal emulators and fonts
- **Multiplexer Safe**: Text-based output survives tmux/screen without protocol awareness
- **Reasonable Resolution**: 2×4 dots per cell provides sufficient detail for recognizable images (e.g., 80×24 terminal = 160×96 effective pixels)
- **No Negotiation**: No terminal capability detection or protocol negotiation required
- **Proven Approach**: Successfully used by existing projects (e.g., crabmusic, from which dotmax evolved)

## Consequences

**Positive**:
- ✅ Works on all Unicode-capable terminals without feature detection
- ✅ Survives copy/paste operations (output is plain text)
- ✅ Works inside tmux, screen, and other multiplexers
- ✅ Works over SSH and remote connections
- ✅ No binary protocols or escape sequence negotiation
- ✅ Accessibility: Screen readers can announce braille output
- ✅ Simple implementation: Direct character mapping, no protocol state machine

**Negative**:
- ❌ Limited resolution: Fixed 2×4 grid per cell (lower than Sixel or Kitty graphics)
- ❌ Requires Unicode font support (not a problem for modern systems)
- ❌ Grayscale rendering requires dithering (no sub-pixel gray levels)
- ❌ Color requires ANSI color codes (foreground/background per cell, not per dot)
- ❌ Resolution tied to terminal size (80×24 terminal = only 160×96 pixels)

**Neutral**:
- Each cell is atomic (all dots same foreground color) - color applied per cell, not per dot
- Braille was designed for touch reading, so visual appearance may seem unusual to users unfamiliar with braille

## Alternatives Considered

### Alternative 1: Sixel Graphics Protocol
- **Description**: Sixel is a bitmap graphics protocol originally from DEC terminals, supported by xterm, mlterm, iTerm2, and a few modern emulators.
- **Pros**: True pixel-based rendering, high resolution, supports full RGB color
- **Cons**: Limited terminal support (not in most terminals), breaks in tmux/screen, requires protocol negotiation
- **Rejection Reason**: Incompatible with NFR-P1 (cross-platform compatibility). Most users don't have Sixel-capable terminals, and tmux/screen break it.

### Alternative 2: Kitty Graphics Protocol
- **Description**: Modern graphics protocol designed by the Kitty terminal emulator, supports images, animations, and transparency.
- **Pros**: High resolution, full RGB, modern design, good performance
- **Cons**: Only supported by Kitty and a handful of forks, completely broken by tmux/screen
- **Rejection Reason**: Fails NFR-P1 (cross-platform) and breaks in multiplexers (deal-breaker for many developers). Too niche for a general-purpose library.

### Alternative 3: ASCII Art (Box Drawing + Block Characters)
- **Description**: Use ASCII box-drawing characters (─│┌┐└┘) and block elements (█▀▄▌) for rendering.
- **Pros**: Universal support (ASCII is everywhere), simple implementation
- **Cons**: Very low resolution (1 or 2 effective pixels per cell), limited visual fidelity
- **Rejection Reason**: Insufficient resolution for recognizable images. Box drawing is 1 pixel per cell, block characters are ~2-4 pixels per cell vs braille's 8 dots per cell.

### Alternative 4: ANSI Block Characters for Density Rendering
- **Description**: Use ANSI block characters (█ ▓ ▒ ░) to represent different gray levels or densities.
- **Pros**: Simple, universally supported, good for density/heatmap visualizations
- **Cons**: Only 4-5 density levels, no fine detail, not suitable for images
- **Rejection Reason**: Not rejected - retained as complementary approach for Epic 4 (Density Rendering). Braille is primary for images/graphics, ANSI blocks are secondary for density visualization.

## References

- [PRD: NFR-P1 Cross-Platform Compatibility](../PRD.md#Non-Functional-Requirements)
- [PRD: Core Features - Braille Rendering](../PRD.md#Core-Features)
- [Architecture: System Architecture - Rendering Pipeline](../architecture.md#System-Architecture)
- [Unicode Braille Patterns Specification](https://unicode.org/charts/PDF/U2800.pdf)
- Inspiration: crabmusic (predecessor project using braille rendering)
