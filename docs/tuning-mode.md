# Render Tuning Mode

## Overview

The Render Tuner is an interactive tool for discovering optimal render settings for images and videos. It provides real-time visual feedback as you adjust parameters, then exports the exact API code to reproduce your settings.

## Usage

```bash
# For images
cargo run --example render_tuner --features image -- image.png

# For video (plays in loop while tuning)
cargo run --example render_tuner --features image,video -- video.mp4
```

## Controls

| Key       | Action                                      |
|-----------|---------------------------------------------|
| `D`       | Cycle dithering: None → Floyd → Bayer → Atkinson |
| `T`       | Toggle threshold mode: Auto ↔ Manual        |
| `↑`/`↓`   | Adjust threshold (±5, when manual)          |
| `B`/`b`   | Increase/decrease brightness (±0.1)         |
| `C`/`c`   | Increase/decrease contrast (±0.1)           |
| `G`/`g`   | Increase/decrease gamma (±0.1)              |
| `M`       | Cycle color mode: Mono → Gray → TrueColor   |
| `Space`   | Pause/Resume video playback                 |
| `R`       | Reset all settings to defaults              |
| `S`       | Show API snippet inline                     |
| `Q`/`Esc` | Quit and print API snippet                  |

## HUD Display

The tuner displays a status bar at the bottom showing:

```
Playing | Dither: FloydSteinberg | Thresh: Auto | FPS: 30.2 (45.1)
Bright: 1.00 | Contrast: 1.00 | Gamma: 1.00 | Color: Mono
[D]ither [T]hresh [B]right [C]ontrast [G]amma [M]ode [Space]Pause [R]eset [S]nippet [Q]uit
```

**FPS Display**: Shows two values:
- First number: Actual playback FPS (respects video frame timing)
- Parenthesized number: Raw render capability (how fast the system can render)

## Exported API Snippet

When you quit or press `S`, the tuner outputs code like:

```rust
// For ImageRenderer:
ImageRenderer::new()
    .load_from_path(Path::new("image.png"))?
    .dithering(DitheringMethod::FloydSteinberg)
    .brightness(1.2)?
    .contrast(1.1)?
    .color_mode(ColorMode::TrueColor)
    .render()?

// For VideoPlayer:
VideoPlayer::new("video.mp4")?
    .dithering(DitheringMethod::FloydSteinberg)
    .brightness(1.2)
    .contrast(1.1)
    .color_mode(ColorMode::TrueColor)
```

## Performance

The tuner uses optimized rendering techniques:

1. **Differential Rendering**: Only redraws lines that changed since the last frame
2. **Color Batching**: Groups consecutive same-color characters to minimize ANSI escape codes
3. **Pre-allocated Buffers**: Reuses memory buffers to avoid per-frame allocations
4. **Efficient Polling**: Uses 10ms input polling intervals to reduce CPU overhead

Typical performance: 25-40 FPS on standard terminals (up from ~8 FPS before optimization).

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   render_tuner                       │
├─────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐                 │
│  │ TunerSettings│    │ FrameBuffer │                │
│  │ - dithering │    │ - lines[]   │                 │
│  │ - threshold │    │ - width     │                 │
│  │ - brightness│    │ - height    │                 │
│  │ - contrast  │    └──────┬──────┘                 │
│  │ - gamma     │           │                        │
│  │ - color_mode│           │ differential           │
│  └──────┬──────┘           │ updates                │
│         │                  ▼                        │
│         │         ┌────────────────┐                │
│         │         │ render_grid_line│               │
│         │         │ (color batching)│               │
│         │         └────────┬───────┘                │
│         │                  │                        │
│         ▼                  ▼                        │
│  ┌─────────────┐   ┌─────────────┐                  │
│  │ VideoPlayer │   │   stdout    │                  │
│  │ or          │   │ (ANSI)      │                  │
│  │ ImageRenderer   └─────────────┘                  │
│  └─────────────┘                                    │
└─────────────────────────────────────────────────────┘
```

## Use Cases

1. **Finding optimal dithering**: Compare Floyd-Steinberg vs Bayer vs Atkinson for your content
2. **Adjusting for terminal**: Tune brightness/contrast for your specific terminal's colors
3. **Color mode selection**: Determine if TrueColor improves quality enough to justify performance cost
4. **Video preview**: Test video rendering settings before implementing in your application
5. **Documentation**: Generate exact API code to share with team members

## Limitations

- Video tuning requires the `video` feature (FFmpeg)
- Performance varies by terminal emulator (some handle ANSI codes faster than others)
- WSL terminals may have additional latency compared to native Linux/macOS terminals
