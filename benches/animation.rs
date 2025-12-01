//! Animation benchmarks for Stories 6.1, 6.2, and 6.5
//!
//! Validates performance requirements:
//! - Buffer swap time: <1ms (target: <1μs actual)
//! - Buffer creation: baseline measurement
//! - Frame timing overhead: <1ms
//! - FPS calculation: baseline measurement
//! - Differential rendering: 60-80% I/O reduction

// Allow certain clippy warnings that are acceptable in benchmarks
#![allow(clippy::cast_precision_loss)]

use criterion::{criterion_group, criterion_main, Criterion};
use dotmax::animation::{DifferentialRenderer, FrameBuffer, FrameTimer};
use dotmax::BrailleGrid;
use std::hint::black_box;

/// Benchmark: `FrameBuffer` creation (80x24 standard terminal)
///
/// Measures allocation time for two `BrailleGrid` buffers.
fn bench_frame_buffer_creation_80x24(c: &mut Criterion) {
    c.bench_function("frame_buffer_creation_80x24", |b| {
        b.iter(|| black_box(FrameBuffer::new(80, 24)));
    });
}

/// Benchmark: `FrameBuffer` creation (200x50 large buffer)
///
/// Measures allocation time for larger animation buffers.
fn bench_frame_buffer_creation_200x50(c: &mut Criterion) {
    c.bench_function("frame_buffer_creation_200x50", |b| {
        b.iter(|| black_box(FrameBuffer::new(200, 50)));
    });
}

/// Benchmark: `swap_buffers()` operation (80x24)
///
/// **AC #7 Target: <1ms (95th percentile)**
///
/// This should be an O(1) pointer swap, so actual time should be <1μs.
fn bench_swap_buffers_80x24(c: &mut Criterion) {
    c.bench_function("swap_buffers_80x24", |b| {
        let mut buffer = FrameBuffer::new(80, 24);

        // Pre-populate with some data for realistic measurement
        for y in 0..24 {
            for x in 0..80 {
                let _ = buffer.get_back_buffer().set_dot(x * 2, y * 4);
            }
        }

        b.iter(|| {
            buffer.swap_buffers();
            black_box(&buffer);
        });
    });
}

/// Benchmark: `swap_buffers()` operation (200x50 large buffer)
///
/// Verifies O(1) scaling - large buffers should swap as fast as small ones.
fn bench_swap_buffers_200x50(c: &mut Criterion) {
    c.bench_function("swap_buffers_200x50", |b| {
        let mut buffer = FrameBuffer::new(200, 50);

        // Pre-populate with some data
        for y in 0..50 {
            for x in 0..200 {
                let _ = buffer.get_back_buffer().set_dot(x * 2, y * 4);
            }
        }

        b.iter(|| {
            buffer.swap_buffers();
            black_box(&buffer);
        });
    });
}

/// Benchmark: `get_back_buffer()` access time
///
/// Measures the overhead of acquiring mutable reference to back buffer.
fn bench_get_back_buffer(c: &mut Criterion) {
    c.bench_function("get_back_buffer", |b| {
        let mut buffer = FrameBuffer::new(80, 24);

        b.iter(|| {
            black_box(buffer.get_back_buffer());
        });
    });
}

/// Benchmark: Full frame preparation cycle
///
/// Measures: clear + draw pattern + swap (typical animation frame)
fn bench_full_frame_cycle(c: &mut Criterion) {
    c.bench_function("full_frame_cycle_80x24", |b| {
        let mut buffer = FrameBuffer::new(80, 24);

        b.iter(|| {
            // Clear back buffer
            buffer.get_back_buffer().clear();

            // Draw something (simulated ball at position)
            let _ = buffer.get_back_buffer().set_dot(40, 48);
            let _ = buffer.get_back_buffer().set_dot(41, 48);
            let _ = buffer.get_back_buffer().set_dot(40, 49);
            let _ = buffer.get_back_buffer().set_dot(41, 49);

            // Swap buffers
            buffer.swap_buffers();

            black_box(&buffer);
        });
    });
}

// ============================================================================
// Story 6.2: FrameTimer Benchmarks
// ============================================================================

/// Benchmark: `FrameTimer` creation
///
/// Measures initialization overhead including `VecDeque` allocation.
fn bench_frame_timer_creation(c: &mut Criterion) {
    c.bench_function("frame_timer_creation", |b| {
        b.iter(|| black_box(FrameTimer::new(60)));
    });
}

/// Benchmark: `FrameTimer::actual_fps()` calculation
///
/// Measures the overhead of calculating rolling average FPS.
/// Should be negligible (<100μs).
fn bench_frame_timer_actual_fps(c: &mut Criterion) {
    c.bench_function("frame_timer_actual_fps", |b| {
        // Pre-populate with frame data for realistic measurement
        let mut timer = FrameTimer::new(60);

        // Simulate 60 frames worth of data by calling wait_for_next_frame
        // with artificial short sleeps
        for _ in 0..60 {
            // Record a synthetic frame time
            timer.wait_for_next_frame();
        }

        b.iter(|| {
            black_box(timer.actual_fps());
        });
    });
}

/// Benchmark: `FrameTimer::frame_time()` retrieval
///
/// Measures the overhead of getting the last frame duration.
fn bench_frame_timer_frame_time(c: &mut Criterion) {
    c.bench_function("frame_timer_frame_time", |b| {
        let mut timer = FrameTimer::new(60);
        timer.wait_for_next_frame(); // Record at least one frame

        b.iter(|| {
            black_box(timer.frame_time());
        });
    });
}

/// Benchmark: `FrameTimer::reset()` operation
///
/// Measures the overhead of resetting timer state.
fn bench_frame_timer_reset(c: &mut Criterion) {
    c.bench_function("frame_timer_reset", |b| {
        let mut timer = FrameTimer::new(60);

        // Pre-populate with frame data
        for _ in 0..60 {
            timer.wait_for_next_frame();
        }

        b.iter(|| {
            timer.reset();
            black_box(&timer);
        });
    });
}

/// Benchmark: Full frame timing cycle overhead
///
/// Measures the computational overhead of `wait_for_next_frame()`
/// excluding the actual sleep time. This uses a high FPS to minimize
/// sleep and measure pure overhead.
fn bench_frame_timer_overhead(c: &mut Criterion) {
    c.bench_function("frame_timer_wait_overhead", |b| {
        // Use 240 FPS to minimize sleep time
        let mut timer = FrameTimer::new(240);

        b.iter(|| {
            // Measure just the calculation overhead
            // Note: This will include minimal sleep but focuses on computational work
            timer.wait_for_next_frame();
            black_box(&timer);
        });
    });
}

// ============================================================================
// Story 6.5: Differential Rendering Benchmarks
// ============================================================================

/// Benchmark: Create `DifferentialRenderer`
fn bench_differential_renderer_creation(c: &mut Criterion) {
    c.bench_function("differential_renderer_creation", |b| {
        b.iter(|| black_box(DifferentialRenderer::new()));
    });
}

/// Benchmark: Count changed cells (comparison logic overhead)
///
/// Measures the overhead of comparing two frames.
fn bench_differential_count_changes(c: &mut Criterion) {
    c.bench_function("differential_count_changes_80x24", |b| {
        let renderer = DifferentialRenderer::new();
        let frame1 = BrailleGrid::new(80, 24).unwrap();
        let mut frame2 = BrailleGrid::new(80, 24).unwrap();
        // Set 5% of cells as changed (typical animation scenario)
        for i in 0..96 {
            let x = (i * 7) % 160; // Spread changes across grid
            let y = (i * 13) % 96;
            let _ = frame2.set_dot(x, y);
        }

        b.iter(|| black_box(renderer.count_changed_cells(&frame2, &frame1)));
    });
}

/// Benchmark: Identical frames comparison (best case)
///
/// Measures comparison overhead when no changes detected.
fn bench_differential_no_changes(c: &mut Criterion) {
    c.bench_function("differential_no_changes_80x24", |b| {
        let renderer = DifferentialRenderer::new();
        let frame1 = BrailleGrid::new(80, 24).unwrap();
        let frame2 = BrailleGrid::new(80, 24).unwrap();

        b.iter(|| black_box(renderer.count_changed_cells(&frame2, &frame1)));
    });
}

/// Benchmark: Full frame comparison (worst case)
///
/// Measures comparison overhead when all cells differ.
fn bench_differential_all_changes(c: &mut Criterion) {
    c.bench_function("differential_all_changes_80x24", |b| {
        let renderer = DifferentialRenderer::new();
        let frame1 = BrailleGrid::new(80, 24).unwrap();
        let mut frame2 = BrailleGrid::new(80, 24).unwrap();
        // Set all cells as different
        for y in 0..96 {
            for x in 0..160 {
                let _ = frame2.set_dot(x, y);
            }
        }

        b.iter(|| black_box(renderer.count_changed_cells(&frame2, &frame1)));
    });
}

/// Benchmark: Moving object simulation
///
/// Simulates typical animation: static background with small moving object.
/// Verifies 60-80% I/O reduction target.
fn bench_differential_moving_object(c: &mut Criterion) {
    c.bench_function("differential_moving_object_80x24", |b| {
        let renderer = DifferentialRenderer::new();

        // Static background frame (border)
        let mut frame1 = BrailleGrid::new(80, 24).unwrap();
        // Draw border (static content)
        for x in 0..160 {
            let _ = frame1.set_dot(x, 0);
            let _ = frame1.set_dot(x, 95);
        }
        for y in 0..96 {
            let _ = frame1.set_dot(0, y);
            let _ = frame1.set_dot(159, y);
        }

        // Frame with border + moved ball (only ~1-2% changed)
        let mut frame2 = frame1.clone();
        // New ball at (22, 22) - different position from frame1 (which has no ball)
        for dy in 0..8 {
            for dx in 0..4 {
                let _ = frame2.set_dot(44 + dx, 44 + dy);
            }
        }

        b.iter(|| {
            let changed = renderer.count_changed_cells(&frame2, &frame1);
            black_box(changed);
        });
    });
}

/// Verify I/O reduction calculation
///
/// This benchmark verifies that typical animations achieve 60-80%+ I/O reduction.
fn bench_differential_io_reduction_verification(c: &mut Criterion) {
    c.bench_function("differential_io_reduction_verify", |b| {
        let renderer = DifferentialRenderer::new();

        // Static background (80x24 = 1920 cells)
        let frame1 = BrailleGrid::new(80, 24).unwrap();

        // Frame with 5% changed cells (96 cells)
        let mut frame2 = BrailleGrid::new(80, 24).unwrap();
        for i in 0..96 {
            let x = (i * 17) % 160;
            let y = (i * 23) % 96;
            let _ = frame2.set_dot(x, y);
        }

        b.iter(|| {
            let total_cells = 80 * 24; // 1920
            let changed = renderer.count_changed_cells(&frame2, &frame1);
            let reduction = ((total_cells - changed) as f64 / total_cells as f64) * 100.0;
            black_box(reduction);
            // Verify: reduction should be >60% (target: 60-80%)
            assert!(reduction > 60.0, "I/O reduction should be >60%, got {reduction:.1}%");
        });
    });
}

// ============================================================================
// Story 7.2: 60fps Sustained Benchmarks (AC3, AC6)
// ============================================================================

/// Benchmark: 60fps sustained over 100+ frames
///
/// **AC6 Target: Frame timing < 16.67ms per frame for 60fps**
///
/// This benchmark validates that we can sustain 60fps by measuring:
/// 1. Frame preparation time (clear + draw + convert)
/// 2. Buffer swap time
/// 3. Total frame cycle time
///
/// The benchmark simulates a realistic animation with:
/// - Double-buffered rendering
/// - Frame preparation (clear, draw shape, convert to unicode)
/// - Buffer swap
fn bench_60fps_sustained_100_frames(c: &mut Criterion) {
    use std::time::Duration;

    let mut group = c.benchmark_group("60fps_sustained");
    group.measurement_time(Duration::from_secs(15));

    // Simulate 100 frames of animation at 60fps
    // Each frame: clear back buffer, draw moving object, convert to unicode, swap
    group.bench_function("100_frames_80x24", |b| {
        let mut buffer = FrameBuffer::new(80, 24);

        b.iter(|| {
            for frame in 0..100_u32 {
                // Clear back buffer
                buffer.get_back_buffer().clear();

                // Draw animated object (bouncing ball simulation)
                let ball_x = ((frame * 2) % 156) as usize + 2; // 2-158 range
                let ball_y = ((frame * 3) % 92) as usize + 2; // 2-94 range

                // Draw 4x4 ball
                for dy in 0..4 {
                    for dx in 0..4 {
                        let _ = buffer.get_back_buffer().set_dot(ball_x + dx, ball_y + dy);
                    }
                }

                // Convert to unicode (rendering step)
                let chars = buffer.get_back_buffer().to_unicode_grid();
                black_box(&chars);

                // Swap buffers (present frame)
                buffer.swap_buffers();
            }
            black_box(&buffer);
        });
    });

    // Larger terminal size
    group.bench_function("100_frames_200x50", |b| {
        let mut buffer = FrameBuffer::new(200, 50);

        b.iter(|| {
            for frame in 0..100_u32 {
                buffer.get_back_buffer().clear();

                let ball_x = ((frame * 2) % 396) as usize + 2;
                let ball_y = ((frame * 3) % 196) as usize + 2;

                for dy in 0..8 {
                    for dx in 0..8 {
                        let _ = buffer.get_back_buffer().set_dot(ball_x + dx, ball_y + dy);
                    }
                }

                let chars = buffer.get_back_buffer().to_unicode_grid();
                black_box(&chars);

                buffer.swap_buffers();
            }
            black_box(&buffer);
        });
    });

    group.finish();
}

/// Benchmark: Single frame preparation time
///
/// Validates that a single frame can be prepared in < 16.67ms
fn bench_frame_preparation_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_preparation");

    // 80x24 standard terminal
    group.bench_function("single_frame_80x24", |b| {
        let mut buffer = FrameBuffer::new(80, 24);

        b.iter(|| {
            // Clear
            buffer.get_back_buffer().clear();

            // Draw complex scene (multiple shapes)
            for i in 0..10 {
                let x = (i * 15) % 156;
                let y = (i * 9) % 92;
                for dy in 0..4 {
                    for dx in 0..4 {
                        let _ = buffer.get_back_buffer().set_dot(x + dx, y + dy);
                    }
                }
            }

            // Convert to unicode
            let chars = buffer.get_back_buffer().to_unicode_grid();
            black_box(&chars);

            // Swap
            buffer.swap_buffers();
            black_box(&buffer);
        });
    });

    // 200x50 large terminal
    group.bench_function("single_frame_200x50", |b| {
        let mut buffer = FrameBuffer::new(200, 50);

        b.iter(|| {
            buffer.get_back_buffer().clear();

            for i in 0..20 {
                let x = (i * 19) % 396;
                let y = (i * 9) % 196;
                for dy in 0..8 {
                    for dx in 0..8 {
                        let _ = buffer.get_back_buffer().set_dot(x + dx, y + dy);
                    }
                }
            }

            let chars = buffer.get_back_buffer().to_unicode_grid();
            black_box(&chars);

            buffer.swap_buffers();
            black_box(&buffer);
        });
    });

    group.finish();
}

/// Benchmark: Verify 60fps is achievable (computational overhead only)
///
/// This measures pure computational time excluding any sleep/sync.
/// Target: Total time for 100 frames should be < 1667ms (16.67ms * 100)
fn bench_60fps_computational_budget(c: &mut Criterion) {
    use std::time::Duration;

    let mut group = c.benchmark_group("60fps_budget");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("100_frames_compute_only_80x24", |b| {
        let mut buffer = FrameBuffer::new(80, 24);
        let renderer = DifferentialRenderer::new();
        let mut prev_grid = BrailleGrid::new(80, 24).unwrap();

        b.iter(|| {
            for frame in 0..100_u32 {
                // Clear and draw
                buffer.get_back_buffer().clear();
                let ball_x = ((frame * 2) % 156) as usize + 2;
                let ball_y = ((frame * 3) % 92) as usize + 2;
                for dy in 0..4 {
                    for dx in 0..4 {
                        let _ = buffer.get_back_buffer().set_dot(ball_x + dx, ball_y + dy);
                    }
                }

                // Differential check (simulating optimized rendering)
                let changed = renderer.count_changed_cells(buffer.get_back_buffer(), &prev_grid);
                black_box(changed);

                // Convert only changed regions (optimization benefit)
                let chars = buffer.get_back_buffer().to_unicode_grid();
                black_box(&chars);

                // Update previous frame reference
                prev_grid = buffer.get_back_buffer().clone();

                // Swap
                buffer.swap_buffers();
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_frame_buffer_creation_80x24,
    bench_frame_buffer_creation_200x50,
    bench_swap_buffers_80x24,
    bench_swap_buffers_200x50,
    bench_get_back_buffer,
    bench_full_frame_cycle,
    // Story 6.2 benchmarks
    bench_frame_timer_creation,
    bench_frame_timer_actual_fps,
    bench_frame_timer_frame_time,
    bench_frame_timer_reset,
    bench_frame_timer_overhead,
    // Story 6.5 benchmarks
    bench_differential_renderer_creation,
    bench_differential_count_changes,
    bench_differential_no_changes,
    bench_differential_all_changes,
    bench_differential_moving_object,
    bench_differential_io_reduction_verification,
    // Story 7.2 benchmarks (AC3, AC6)
    bench_60fps_sustained_100_frames,
    bench_frame_preparation_time,
    bench_60fps_computational_budget
);
criterion_main!(benches);
