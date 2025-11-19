# Terminal Compatibility Matrix

## Overview

Different terminal emulators report their dimensions in different ways. Some terminals report the visible **viewport size** (what the user actually sees), while others report the **buffer size** (which may be larger to enable scrollback). This document describes how dotmax detects and handles these differences.

## Terminal Detection Methodology

Dotmax uses a multi-stage detection approach:

1. **Environment Variables** (highest priority):
   - `WT_SESSION`: Indicates Windows Terminal
   - `WSL_DISTRO_NAME`: Indicates Windows Subsystem for Linux (WSL)
   - `TERM_PROGRAM`: Can indicate macOS Terminal.app or others

2. **Platform Detection** (fallback):
   - Uses Rust's `cfg(target_os)` to detect Windows, macOS, or Linux

3. **Conservative Defaults**:
   - Unknown terminals are treated conservatively (no offset applied)

## Compatibility Table

| Terminal Type | Dimensions Reported | Viewport Offset | Detection Method | Visual Testing Status |
|---------------|---------------------|-----------------|------------------|----------------------|
| **Windows Terminal** | Viewport (correct) | 0 rows | `WT_SESSION` env var | ✅ Recommended |
| **WSL (Ubuntu/etc.)** | May report buffer | 2 rows (if height > 20) | `WSL_DISTRO_NAME` env var | ⚠️ Needs validation |
| **PowerShell / cmd** | Buffer size | 4 rows (if height > 30) | Windows platform + no WT_SESSION | ⚠️ Needs validation |
| **macOS Terminal.app** | Viewport (correct) | 0 rows | `TERM_PROGRAM=Apple_Terminal` | ⚠️ Needs validation |
| **Linux Native** | Viewport (correct) | 0 rows | Linux platform (non-WSL) | ⚠️ Needs validation |
| **Unknown** | Unknown | 0 rows (conservative) | None of the above | ⚠️ Unknown |

## Platform-Specific Notes

### Windows Terminal

- **Best support**: Reports viewport size accurately
- **Detection**: `WT_SESSION` environment variable
- **Offset**: None needed
- **Recommendation**: Preferred terminal for Windows users

### WSL (Windows Subsystem for Linux)

- **Behavior**: May report buffer size instead of viewport
- **Detection**: `WSL_DISTRO_NAME` environment variable
- **Offset**: 2 rows for terminals taller than 20 rows
- **Note**: This is a conservative offset based on typical WSL behavior
- **Needs**: User validation to confirm correctness

### PowerShell / cmd (Windows Console)

- **Behavior**: Reports buffer size, which can be significantly larger than viewport
- **Detection**: Windows platform without `WT_SESSION`
- **Offset**: 4 rows for terminals taller than 30 rows
- **Note**: Buffer is often much larger than viewport in PowerShell
- **Recommendation**: Use Windows Terminal instead for better experience

### macOS Terminal.app

- **Behavior**: Reports viewport size accurately
- **Detection**: `TERM_PROGRAM` environment variable set to `Apple_Terminal`
- **Offset**: None needed
- **Status**: Needs visual validation on actual macOS hardware

### Linux Native Terminals

- **Behavior**: Typically report viewport size accurately
- **Examples**: gnome-terminal, konsole, xterm, alacritty
- **Detection**: Linux platform (not WSL)
- **Offset**: None needed
- **Note**: Most modern Linux terminals handle this correctly

## Implementation Details

### Terminal Type Enum

```rust
pub enum TerminalType {
    WindowsTerminal,  // WT_SESSION present
    Wsl,              // WSL_DISTRO_NAME present
    WindowsConsole,   // Windows without WT_SESSION
    MacOsTerminal,    // TERM_PROGRAM=Apple_Terminal
    LinuxNative,      // Linux platform (non-WSL)
    Unknown,          // Fallback
}
```

### Offset Calculation

The viewport offset is calculated dynamically based on:
1. Detected terminal type
2. Reported terminal height

```rust
impl TerminalType {
    pub const fn viewport_height_offset(self, reported_height: u16) -> u16 {
        match self {
            Self::WindowsTerminal => 0,
            Self::MacOsTerminal => 0,
            Self::LinuxNative => 0,
            Self::Wsl => if reported_height > 20 { 2 } else { 0 },
            Self::WindowsConsole => if reported_height > 30 { 4 } else { 0 },
            Self::Unknown => 0,
        }
    }
}
```

## Validation Checklist

The following validation is needed from users:

- [ ] **Ubuntu native terminal**: Verify rendering appears correctly positioned
- [ ] **PowerShell**: Verify rendering appears correctly positioned
- [ ] **WSL in Windows Terminal**: Verify no artifacts or misalignment
- [ ] **macOS Terminal.app**: Verify rendering on actual macOS hardware
- [ ] **Alacritty**: Test on Linux, macOS, Windows
- [ ] **Other terminals**: Document behavior

## Known Issues

1. **WSL Offset Conservative**: The 2-row offset for WSL may be too aggressive or too lenient depending on terminal configuration. User feedback needed.

2. **PowerShell Buffer**: The 4-row offset assumes typical PowerShell buffer configuration. Users with custom buffer sizes may need adjustment.

3. **Environment Variable Detection**: Some terminals may not set expected environment variables, leading to fallback to platform detection.

## User Feedback

If you experience rendering issues (content appearing too high or too low in the terminal), please report:

1. Your terminal emulator and version
2. Your operating system
3. Output of `echo $WT_SESSION $WSL_DISTRO_NAME $TERM_PROGRAM` (Unix) or equivalent
4. Whether content appears too high, too low, or correctly positioned
5. Your terminal's reported size vs actual visible size

## Future Improvements

Potential enhancements for better terminal detection:

1. **Query ANSI codes**: Use terminal capability queries for more accurate detection
2. **User configuration**: Allow users to override detected terminal type
3. **Runtime calibration**: Auto-detect offset by rendering test patterns
4. **Terminal database**: Maintain database of known terminals and their behaviors

## References

- [Crossterm documentation](https://docs.rs/crossterm)
- [Ratatui terminal handling](https://docs.rs/ratatui)
- [Windows Console API](https://docs.microsoft.com/en-us/windows/console/)
- Story 2.8: Implement Proper Viewport Detection and Rendering
