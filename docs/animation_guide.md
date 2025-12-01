# Animation Guide

This guide covers the animation system in dotmax, from simple animations to advanced techniques like differential rendering and pre-rendered animation sequences.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [AnimationLoop (High-Level API)](#animationloop-high-level-api)
4. [FrameTimer (Frame Rate Control)](#frametimer-frame-rate-control)
5. [FrameBuffer (Double Buffering)](#framebuffer-double-buffering)
6. [PrerenderedAnimation (Cached Sequences)](#prerenderedanimation-cached-sequences)
7. [DifferentialRenderer (Optimized Updates)](#differentialrenderer-optimized-updates)
8. [Example Gallery](#example-gallery)
9. [Performance Tips](#performance-tips)

## Overview

Dotmax provides a comprehensive animation system built on these core components:

| Component | Purpose | When to Use |
|-----------|---------|-------------|
| `AnimationLoop` | High-level animation abstraction | Most applications |
| `FrameTimer` | Consistent frame rate control | Custom animation loops |
| `FrameBuffer` | Double-buffering for flicker-free updates | Direct buffer control |
| `PrerenderedAnimation` | Cached frame sequences | Repeating animations |
| `DifferentialRenderer` | Only render changed cells | Large static backgrounds |

## Quick Start

The fastest way to create an animation:

```rust
use dotmax::animation::AnimationLoop;

fn main() -> Result<(), dotmax::DotmaxError> {
    AnimationLoop::new(80, 24)
        .fps(60)
        .on_frame(|frame, buffer| {
            // Calculate position (bouncing effect)
            let x = (frame as usize * 2) % 160;
            let y = (frame as usize * 3) % 96;
            buffer.set_dot(x, y)?;
            Ok(true) // Continue animation
        })
        .run()
}
```

## AnimationLoop (High-Level API)

`AnimationLoop` is the recommended way to create animations. It handles:

- Terminal setup (raw mode, alternate screen, cursor hiding)
- Double-buffering automatically
- Frame timing for consistent FPS
- Graceful cleanup on exit (Ctrl+C, 'q', or callback returning `false`)

### Basic Usage

```rust
use dotmax::animation::AnimationLoop;

AnimationLoop::new(80, 24)  // Width × Height in terminal cells
    .fps(60)                 // Target 60 frames per second
    .on_frame(|frame, buffer| {
        // `frame` starts at 0, increments each frame
        // `buffer` is a mutable BrailleGrid (back buffer)

        // Draw your animation here
        buffer.set_dot(frame as usize % 160, 48)?;

        // Return true to continue, false to stop
        Ok(frame < 1000)
    })
    .run()?;
```

### Frame Callback

The callback receives:
- `frame: u64` - Frame number (starts at 0)
- `buffer: &mut BrailleGrid` - The back buffer to draw on

Return values:
- `Ok(true)` - Continue to next frame
- `Ok(false)` - Stop animation gracefully
- `Err(...)` - Stop with error

The back buffer is **automatically cleared** before each callback.

### Configuration Options

```rust
AnimationLoop::new(width, height)
    .fps(30)  // FPS range: 1-240 (clamped automatically)
```

## FrameTimer (Frame Rate Control)

For custom animation loops, `FrameTimer` provides frame rate control:

```rust
use dotmax::animation::FrameTimer;

let mut timer = FrameTimer::new(60); // Target 60 FPS

loop {
    // ... render frame ...

    timer.wait_for_next_frame();

    // Check actual performance
    println!("FPS: {:.1}", timer.actual_fps());
    println!("Last frame: {:?}", timer.frame_time());
}
```

### Key Methods

| Method | Description |
|--------|-------------|
| `new(fps)` | Create timer targeting given FPS (1-240) |
| `wait_for_next_frame()` | Sleep until next frame time |
| `actual_fps()` | Rolling average of achieved FPS |
| `frame_time()` | Duration of last frame |
| `target_fps()` | Configured target FPS |
| `target_frame_time()` | Target duration per frame |
| `reset()` | Clear frame history |

### Frame Drop Handling

When a frame takes longer than target duration:
- No sleep occurs (already late)
- Debug log emitted via `tracing`
- Actual FPS reflects the slowdown
- No "catch-up" frames attempted

## FrameBuffer (Double Buffering)

`FrameBuffer` provides explicit double-buffering control:

```rust
use dotmax::animation::FrameBuffer;
use dotmax::TerminalRenderer;

let mut frame_buffer = FrameBuffer::new(80, 24);
let mut renderer = TerminalRenderer::new()?;

loop {
    // Get back buffer and clear it
    let buffer = frame_buffer.get_back_buffer();
    buffer.clear();

    // Draw to back buffer
    buffer.set_dot(x, y)?;

    // Swap front/back (O(1) pointer swap)
    frame_buffer.swap_buffers();

    // Render front buffer to terminal
    frame_buffer.render(&mut renderer)?;
}
```

### Why Double Buffering?

Without double buffering, drawing directly causes **screen tearing** and **flicker** because the display shows partial frames. Double buffering ensures:

1. All drawing happens on invisible back buffer
2. Atomic swap makes complete frame visible
3. No tearing or partial updates

## PrerenderedAnimation (Cached Sequences)

For repeating animations, pre-render frames once and replay:

```rust
use dotmax::animation::PrerenderedAnimation;
use dotmax::BrailleGrid;

// Pre-render 60 frames
let mut frames = Vec::new();
for i in 0..60 {
    let mut grid = BrailleGrid::new(80, 24)?;
    // Draw frame i onto grid
    draw_frame(&mut grid, i)?;
    frames.push(grid);
}

// Create cached animation
let mut anim = PrerenderedAnimation::new(frames);

// Playback loop
loop {
    let frame = anim.get_current_frame();
    renderer.render(frame)?;
    anim.advance();
}
```

### Looping Modes

```rust
// Loop forever (default)
let anim = PrerenderedAnimation::new(frames);

// Play once, stop at last frame
anim.set_looping(false);

// Get frame by index
let frame = anim.get_frame(30)?;

// Reset to first frame
anim.reset();
```

### Memory Considerations

Each `BrailleGrid` uses ~2KB for 80×24. A 60-frame animation uses ~120KB. Consider:
- Smaller grid dimensions
- Fewer frames with interpolation
- Generating frames on-demand for very long sequences

## DifferentialRenderer (Optimized Updates)

When most of the screen is static, only render changed cells:

```rust
use dotmax::animation::DifferentialRenderer;
use dotmax::BrailleGrid;

let mut diff_renderer = DifferentialRenderer::new(80, 24);
let mut current = BrailleGrid::new(80, 24)?;

// Initial full render
diff_renderer.full_render(&current, &mut renderer)?;

loop {
    // Draw new frame
    let new_frame = generate_frame();

    // Only render changed cells
    let changes = diff_renderer.diff_render(&new_frame, &mut renderer)?;
    println!("Changed {} cells", changes);

    current = new_frame;
}
```

### Performance Benefits

- Full render: O(width × height) terminal writes
- Diff render: O(changed_cells) terminal writes
- Typical savings: 90%+ for animations with static backgrounds

### When to Use

Best for:
- UI with static elements and moving highlights
- Games with fixed backgrounds
- Visualizations with updating data regions

Not ideal for:
- Rapidly changing full-screen content
- Particle systems covering the whole screen

## Example Gallery

### 1. Bouncing Ball (Physics Simulation)

```bash
cargo run --example bouncing_ball
```

Demonstrates:
- Gravity and velocity physics
- Bounce damping (energy loss)
- Real-time FPS display
- `AnimationLoop` high-level API

### 2. Loading Spinner (UI Pattern)

```bash
cargo run --example loading_spinner
```

Demonstrates:
- Rotating indicators
- Multiple spinner styles
- Style cycling
- `FrameTimer` for consistent rotation

### 3. Waveform (Visualization)

```bash
cargo run --example waveform
```

Demonstrates:
- Sine wave animation
- Multiple overlapping waves
- Color schemes from Epic 5
- Line drawing primitives

### 4. Fireworks (Particle System)

```bash
cargo run --example fireworks
```

Demonstrates:
- Particle system (position, velocity, lifetime)
- Gravity and drag physics
- Color fading over lifetime
- Random burst patterns

### 5. Analog Clock (Real-Time)

```bash
cargo run --example clock
```

Demonstrates:
- Real-time system clock
- Circle and line drawing
- Colored clock hands
- Low FPS mode (1 FPS efficiency)

## Performance Tips

### 1. Choose Appropriate FPS

```rust
// UI elements: 10-15 FPS sufficient
AnimationLoop::new(80, 24).fps(15)

// Smooth motion: 30-60 FPS
AnimationLoop::new(80, 24).fps(60)

// Real-time clock: 1 FPS
AnimationLoop::new(80, 24).fps(1)
```

### 2. Minimize Drawing Operations

```rust
// Bad: Clear and redraw everything
fn on_frame(frame, buffer) {
    for y in 0..96 {
        for x in 0..160 {
            if should_draw(x, y, frame) {
                buffer.set_dot(x, y)?;
            }
        }
    }
}

// Good: Only draw what changed
fn on_frame(frame, buffer) {
    let (old_x, old_y) = previous_position(frame - 1);
    let (new_x, new_y) = current_position(frame);

    // AnimationLoop clears buffer automatically
    buffer.set_dot(new_x, new_y)?;
}
```

### 3. Use Differential Rendering for Static Content

```rust
// If 90% of screen is static background
let mut diff = DifferentialRenderer::new(80, 24);
diff.diff_render(&new_frame, &mut renderer)?;
```

### 4. Pre-render Repeating Sequences

```rust
// If animation loops every 60 frames
let frames: Vec<BrailleGrid> = (0..60)
    .map(|i| generate_frame(i))
    .collect();
let anim = PrerenderedAnimation::new(frames);
```

### 5. Profile with Tracing

```rust
// Enable tracing to see timing data
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

### Expected Performance

| Scenario | Target | Typical |
|----------|--------|---------|
| 60 FPS animation | <16.67ms/frame | ~2-5ms |
| Buffer swap | <1ms | ~2.4ns |
| Full render (80×24) | <10ms | ~3-5ms |
| Diff render (10% change) | <2ms | <1ms |

## Troubleshooting

### Animation Stuttering

1. Check actual FPS with `timer.actual_fps()`
2. Reduce target FPS
3. Simplify drawing operations
4. Use differential rendering

### Terminal Not Cleaning Up

The cleanup is automatic, but if interrupted:
```bash
reset  # Restore terminal
```

### Colors Not Showing

Verify terminal supports 24-bit color:
```rust
use dotmax::detect_color_capability;
let caps = detect_color_capability();
```

## Further Reading

- API Documentation: `cargo doc --open`
- Source Code: `src/animation/`
- More Examples: `examples/animations/`
