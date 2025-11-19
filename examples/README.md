# dotmax Examples

This directory contains runnable examples demonstrating how to use dotmax for terminal braille rendering.

## Available Examples

| Example | Description | Features Required | Difficulty |
|---------|-------------|-------------------|------------|
| `hello_braille.rs` | Minimal braille grid creation and rendering | `default` | Beginner |

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

## Example Guidelines

All examples in this repository follow these principles:

1. **Minimal** - Each example focuses on demonstrating one feature or use case
2. **Commented** - Inline comments explain key steps and API usage
3. **Runnable** - All examples compile and run successfully in CI
4. **Practical** - Examples show real-world usage patterns, not toy code

## Getting Help

- Read the [main documentation](https://docs.rs/dotmax) (coming soon)
- See the [root README](../README.md) for installation and quick start
- Check the [architecture docs](../docs/architecture.md) for design decisions
