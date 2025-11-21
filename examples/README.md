# dotmax Examples

This directory contains runnable examples demonstrating how to use dotmax for terminal braille rendering.

## Available Examples

| Example | Description | Features Required | Difficulty |
|---------|-------------|-------------------|------------|
| `hello_braille.rs` | Minimal braille grid creation and rendering | `default` | Beginner |
| `simple_image.rs` | Load and render PNG image with automatic resize handling | `image` | Beginner |
| `image_browser.rs` | Interactive image viewer with settings controls and resize support | `image,svg` | Intermediate |

## Running Examples

Run any example using:

```bash
cargo run --example <example_name>
```

For example:

```bash
# Run the hello braille example
cargo run --example hello_braille
```

### Feature Flags

Some examples require optional features. Enable them with the `--features` flag:

```bash
# Examples with image rendering (available in Epic 3+)
cargo run --example render_image --features image

# Examples with SVG support (available in Epic 3+)
cargo run --example svg_graphics --features svg
```

## Future Examples

The following examples will be added in upcoming epics:

- **Epic 3: 2D Image Rendering**
  - `render_image.rs` - Load and render PNG/JPG images as braille
  - `dithering_comparison.rs` - Compare Floyd-Steinberg, Bayer, and Atkinson dithering
  - `svg_graphics.rs` - Render SVG files to terminal

- **Epic 4: Drawing Primitives**
  - `draw_shapes.rs` - Lines, circles, rectangles, polygons
  - `density_rendering.rs` - Character density-based shading

- **Epic 5: Color System**
  - `color_schemes.rs` - Demonstrate all 6+ color schemes
  - `custom_colors.rs` - Create custom color mappings

- **Epic 6: Animation**
  - `spinning_cube.rs` - Animated 3D wireframe cube
  - `bouncing_ball.rs` - Physics-based animation
  - `video_playback.rs` - Play video files in terminal

## Automatic Terminal Resize Handling

All interactive examples (those with event loops) support automatic re-rendering when the terminal window is resized. When you resize your terminal:

- The image automatically scales to fit the new dimensions
- Aspect ratio is preserved (no distortion)
- No manual refresh or restart required
- Performance is optimized (typically <50ms re-render)

**Implementation Pattern:**

```rust
use crossterm::event::{self, Event};
use std::time::Duration;

loop {
    if event::poll(Duration::from_millis(100))? {
        match event::read()? {
            Event::Resize(width, height) => {
                // Re-render with new terminal dimensions
                render_image(image_path)?;
            }
            Event::Key(key) => {
                // Handle keyboard input
            }
            _ => {}
        }
    }
}
```

This pattern ensures your terminal applications remain responsive and visually correct regardless of window size changes. See `simple_image.rs` and `image_browser.rs` for complete working examples.

## Example Guidelines

All examples in this repository follow these principles:

1. **Minimal** - Each example focuses on demonstrating one feature or use case
2. **Commented** - Inline comments explain key steps and API usage
3. **Runnable** - All examples compile and run successfully in CI
4. **Practical** - Examples show real-world usage patterns, not toy code
5. **Responsive** - Interactive examples handle terminal resize events automatically

## Getting Help

- Read the [main documentation](https://docs.rs/dotmax) (coming soon)
- See the [root README](../README.md) for installation and quick start
- Check the [architecture docs](../docs/architecture.md) for design decisions
