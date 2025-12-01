# Dotmax Troubleshooting Guide

This guide covers common issues and their solutions when using dotmax.

## Display Issues

### Nothing Appears / Blank Screen

**Symptoms:** Running an example shows nothing or just a cursor.

**Solutions:**

1. **Check terminal Unicode support**
   ```bash
   # Test Unicode support
   echo "⡀⠁⠂⠃⠄⠅⠆⠇"
   ```
   If you see boxes or question marks, your terminal doesn't support Unicode.

2. **Use a modern terminal**
   - Windows: Use [Windows Terminal](https://aka.ms/terminal)
   - macOS: Terminal.app or iTerm2
   - Linux: gnome-terminal, konsole, alacritty

3. **Check locale settings**
   ```bash
   echo $LANG
   # Should show something like: en_US.UTF-8
   ```
   Set UTF-8 if needed:
   ```bash
   export LANG=en_US.UTF-8
   ```

### Garbled Characters

**Symptoms:** Output shows boxes, question marks, or wrong characters.

**Solutions:**

1. **Verify font supports braille**
   - Try: Fira Code, JetBrains Mono, or other programming fonts
   - Avoid: System default fonts may lack braille characters

2. **Check terminal encoding**
   ```bash
   locale charmap
   # Should show: UTF-8
   ```

3. **Reset terminal**
   ```bash
   reset
   ```

### Rendering Position Wrong

**Symptoms:** Content appears offset (too high or too low).

**Solutions:**

1. **Check terminal compatibility**
   See [Terminal Compatibility](terminal-compatibility.md) for known issues.

2. **Try different terminal**
   Some terminals report buffer size instead of viewport size.

3. **Enable debug logging**
   ```bash
   RUST_LOG=dotmax=debug cargo run --example hello_braille
   ```
   Look for "Rendering area" in the output.

## Color Issues

### Colors Not Displaying

**Symptoms:** Output is monochrome despite using color APIs.

**Solutions:**

1. **Check terminal color support**
   ```bash
   echo $COLORTERM
   # "truecolor" or "24bit" = best support
   # Empty or "256color" = limited support
   ```

2. **Test color capability**
   ```rust
   use dotmax::detect_color_capability;
   let caps = detect_color_capability();
   println!("Terminal supports: {:?}", caps);
   ```

3. **Force color mode** (if detection fails)
   ```bash
   export COLORTERM=truecolor
   ```

### Wrong Colors

**Symptoms:** Colors look different than expected.

**Causes:**
- Terminal color scheme (dark/light) affects appearance
- ANSI-256 fallback differs from true color
- Color scheme designed for different background

**Solutions:**
- Test with `color_detection` example
- Try different terminal color schemes
- Use predefined schemes designed for your background

## Animation Issues

### Animation Flickering

**Symptoms:** Screen flashes during animation.

**Solutions:**

1. **Use AnimationLoop** (handles double-buffering automatically)
   ```rust
   AnimationLoop::new(80, 24)
       .fps(60)
       .on_frame(|frame, buffer| {
           // Draw here
           Ok(true)
       })
       .run()
   ```

2. **For custom loops, use FrameBuffer**
   ```rust
   use dotmax::animation::FrameBuffer;
   let mut fb = FrameBuffer::new(80, 24);
   // Draw to back buffer, then swap
   fb.swap_buffers();
   ```

3. **Reduce render frequency**
   - Lower FPS if not needed
   - Use differential rendering for mostly-static content

### Animation Stuttering

**Symptoms:** Animation isn't smooth, drops frames.

**Solutions:**

1. **Check actual FPS**
   ```rust
   let timer = FrameTimer::new(60);
   // After rendering...
   println!("Actual FPS: {:.1}", timer.actual_fps());
   ```

2. **Reduce target FPS**
   ```rust
   AnimationLoop::new(80, 24).fps(30)  // Try 30 instead of 60
   ```

3. **Simplify drawing operations**
   - Pre-calculate positions
   - Use differential rendering
   - Cache static elements

4. **Use release mode**
   ```bash
   cargo run --release --example simple_animation
   ```

### Terminal Not Cleaning Up

**Symptoms:** Terminal left in raw mode after crash.

**Solutions:**

1. **Reset terminal**
   ```bash
   reset
   # or
   stty sane
   ```

2. **Ensure cleanup in code**
   ```rust
   let renderer = TerminalRenderer::new()?;
   // ... your code ...
   renderer.cleanup()?;  // Always call this
   ```

3. **Use `AnimationLoop`** - It handles cleanup automatically.

## Image Loading Issues

### Image Not Loading

**Symptoms:** `load_from_path` returns an error.

**Solutions:**

1. **Verify feature is enabled**
   ```toml
   [dependencies]
   dotmax = { version = "0.1", features = ["image"] }
   ```

2. **Check file exists and is readable**
   ```rust
   let path = Path::new("image.png");
   println!("Exists: {}", path.exists());
   ```

3. **Verify image format**
   Supported: PNG, JPEG, GIF, BMP, WEBP
   ```rust
   use dotmax::image::supported_formats;
   println!("{:?}", supported_formats());
   ```

### Image Appears Distorted

**Symptoms:** Image doesn't look right (stretched, cropped, etc.)

**Solutions:**

1. **Check aspect ratio preservation**
   ```rust
   use dotmax::image::resize_to_terminal;
   let resized = resize_to_terminal(&img, term_width, term_height)?;
   // Aspect ratio is preserved by default
   ```

2. **Adjust target dimensions**
   - Grid cells are ~2:1 aspect ratio (2 dots wide, 4 dots tall)
   - Square images may appear stretched vertically

3. **Try different dithering**
   ```rust
   use dotmax::image::{DitherAlgorithm, dither_image};
   let dithered = dither_image(&gray, DitherAlgorithm::Atkinson)?;
   ```

### SVG Not Rendering

**Symptoms:** SVG loads but appears blank or wrong.

**Solutions:**

1. **Enable SVG feature**
   ```toml
   dotmax = { version = "0.1", features = ["image", "svg"] }
   ```

2. **Check SVG compatibility**
   - Simple shapes work best
   - Text may require fonts installed
   - Gradients/filters may not render correctly

3. **Specify rasterization size**
   ```rust
   use dotmax::image::load_svg_from_path;
   let img = load_svg_from_path(path, 800, 600)?;  // Explicit size
   ```

## Build Issues

### Feature Not Found

**Symptoms:** `error: feature 'image' not found`

**Solutions:**

1. **Check Cargo.toml syntax**
   ```toml
   # Correct
   dotmax = { version = "0.1", features = ["image"] }

   # Wrong
   dotmax = { version = "0.1", feature = "image" }
   ```

2. **Run cargo update**
   ```bash
   cargo update
   ```

### Compilation Errors

**Symptoms:** Rust compilation failures.

**Solutions:**

1. **Check Rust version**
   ```bash
   rustc --version
   # Requires Rust 1.70+
   ```

2. **Update Rust**
   ```bash
   rustup update
   ```

3. **Check for conflicting dependencies**
   ```bash
   cargo tree | grep dotmax
   ```

## Debug Logging

Enable detailed logging to diagnose issues:

```bash
# All dotmax logs
RUST_LOG=dotmax=debug cargo run

# Specific modules
RUST_LOG=dotmax::animation=trace cargo run

# With timing
RUST_LOG=dotmax=debug,tokio=warn cargo run
```

In code:
```rust
fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Your code...
}
```

## Getting Help

If you're still stuck:

1. **Check examples** - [examples/README.md](../examples/README.md)
2. **Read API docs** - [docs.rs/dotmax](https://docs.rs/dotmax)
3. **Search issues** - [GitHub Issues](https://github.com/frosty40/dotmax/issues)
4. **File a bug report** - Include:
   - Rust version (`rustc --version`)
   - Terminal name and version
   - Operating system
   - Minimal reproduction code
   - Debug log output

## Quick Reference

| Symptom | Likely Cause | Solution |
|---------|--------------|----------|
| Blank screen | No Unicode | Use modern terminal |
| Boxes/??? | Missing font | Install programming font |
| No colors | Terminal caps | Check $COLORTERM |
| Flickering | No double-buffer | Use AnimationLoop |
| Stuttering | CPU overload | Lower FPS, simplify |
| Terminal broken | Crash in raw mode | Run `reset` |
| Image error | Missing feature | Add `features = ["image"]` |
