//! Integration tests for temporal coherence algorithms.
//!
//! These tests verify that the temporal coherence module reduces flicker
//! when processing sequences of frames.

use dotmax::image::temporal::{
    average_flicker, measure_flicker, FrameBlender, HysteresisFilter, TemporalCoherence,
    TemporalConfig,
};
use image::GrayImage;

// ============================================================================
// Flicker Measurement Tests
// ============================================================================

#[test]
fn test_measure_flicker_identical_frames() {
    let frame = vec![true, false, true, false, true];
    let flicker = measure_flicker(&frame, &frame);
    assert!((flicker - 0.0).abs() < f64::EPSILON, "Identical frames should have 0 flicker");
}

#[test]
fn test_measure_flicker_completely_different() {
    let frame1 = vec![true, true, true, true];
    let frame2 = vec![false, false, false, false];
    let flicker = measure_flicker(&frame1, &frame2);
    assert!((flicker - 1.0).abs() < f64::EPSILON, "Opposite frames should have 100% flicker");
}

#[test]
fn test_measure_flicker_partial_change() {
    let frame1 = vec![true, false, true, false];
    let frame2 = vec![true, true, true, false]; // 1 change out of 4
    let flicker = measure_flicker(&frame1, &frame2);
    assert!((flicker - 0.25).abs() < 0.001, "25% of dots changed");
}

#[test]
fn test_average_flicker_calculation() {
    let frames = vec![
        vec![true, false, true, false], // frame 0
        vec![true, true, true, false],  // 1 change from frame 0 (25%)
        vec![false, true, false, false], // 2 changes from frame 1 (50%)
    ];
    let avg = average_flicker(&frames);
    // (0.25 + 0.50) / 2 = 0.375
    assert!((avg - 0.375).abs() < 0.001, "Average should be 37.5%");
}

#[test]
fn test_average_flicker_single_frame() {
    let frames = vec![vec![true, false]];
    let avg = average_flicker(&frames);
    assert!((avg - 0.0).abs() < f64::EPSILON, "Single frame should have 0 average flicker");
}

#[test]
fn test_average_flicker_empty() {
    let frames: Vec<Vec<bool>> = vec![];
    let avg = average_flicker(&frames);
    assert!((avg - 0.0).abs() < f64::EPSILON, "Empty frames should have 0 flicker");
}

// ============================================================================
// Hysteresis Filter Tests
// ============================================================================

#[test]
fn test_hysteresis_reduces_noise_flicker() {
    // Simulate a sequence of frames where a pixel oscillates near the threshold
    // Without hysteresis, it would flicker rapidly
    let threshold: u8 = 128;
    let margin: u8 = 15;

    let mut filter = HysteresisFilter::new(margin);

    // Frame 1: Pixel at 130 (above threshold) -> ON
    let frame1 = GrayImage::from_fn(1, 1, |_, _| image::Luma([130]));
    let result1 = filter.apply(&frame1, threshold);
    assert_eq!(result1.get_pixel(0, 0).0[0], 255, "Should be ON");

    // Frame 2: Pixel drops to 120 (below threshold but above low_thresh=113)
    // With hysteresis, should stay ON
    let frame2 = GrayImage::from_fn(1, 1, |_, _| image::Luma([120]));
    let result2 = filter.apply(&frame2, threshold);
    assert_eq!(result2.get_pixel(0, 0).0[0], 255, "Should stay ON (hysteresis)");

    // Frame 3: Pixel drops to 110 (below low_thresh=113)
    // Now it should turn OFF
    let frame3 = GrayImage::from_fn(1, 1, |_, _| image::Luma([110]));
    let result3 = filter.apply(&frame3, threshold);
    assert_eq!(result3.get_pixel(0, 0).0[0], 0, "Should turn OFF");

    // Frame 4: Pixel rises to 130 (below high_thresh=143 but above threshold)
    // With hysteresis, should stay OFF
    let frame4 = GrayImage::from_fn(1, 1, |_, _| image::Luma([130]));
    let result4 = filter.apply(&frame4, threshold);
    assert_eq!(result4.get_pixel(0, 0).0[0], 0, "Should stay OFF (hysteresis)");

    // Frame 5: Pixel rises to 150 (above high_thresh=143)
    // Now it should turn ON
    let frame5 = GrayImage::from_fn(1, 1, |_, _| image::Luma([150]));
    let result5 = filter.apply(&frame5, threshold);
    assert_eq!(result5.get_pixel(0, 0).0[0], 255, "Should turn ON");
}

#[test]
fn test_hysteresis_dimension_change_resets() {
    let mut filter = HysteresisFilter::new(10);

    // Apply to 2x2 frame
    let frame1 = GrayImage::from_fn(2, 2, |_, _| image::Luma([150]));
    let _ = filter.apply(&frame1, 128);

    // Apply to 3x3 frame (dimension change)
    let frame2 = GrayImage::from_fn(3, 3, |_, _| image::Luma([100]));
    let result = filter.apply(&frame2, 128);

    // First frame after resize should use standard threshold
    assert_eq!(result.get_pixel(0, 0).0[0], 0, "100 < 128, should be OFF");
}

// ============================================================================
// Frame Blender Tests
// ============================================================================

#[test]
fn test_frame_blender_smooths_transitions() {
    let mut blender = FrameBlender::new(0.5);

    // Frame 1: All 100
    let frame1 = GrayImage::from_fn(2, 2, |_, _| image::Luma([100]));
    let result1 = blender.blend(&frame1);
    assert_eq!(result1.get_pixel(0, 0).0[0], 100, "First frame unchanged");

    // Frame 2: All 200
    // With alpha=0.5: output = 0.5*200 + 0.5*100 = 150
    let frame2 = GrayImage::from_fn(2, 2, |_, _| image::Luma([200]));
    let result2 = blender.blend(&frame2);
    assert_eq!(result2.get_pixel(0, 0).0[0], 150, "Blended to 150");

    // Frame 3: All 200 again
    // output = 0.5*200 + 0.5*150 = 175
    let frame3 = GrayImage::from_fn(2, 2, |_, _| image::Luma([200]));
    let result3 = blender.blend(&frame3);
    assert_eq!(result3.get_pixel(0, 0).0[0], 175, "Converging to 200");
}

#[test]
fn test_frame_blender_alpha_1_passes_through() {
    let mut blender = FrameBlender::new(1.0);

    let frame1 = GrayImage::from_fn(2, 2, |_, _| image::Luma([100]));
    let _ = blender.blend(&frame1);

    let frame2 = GrayImage::from_fn(2, 2, |_, _| image::Luma([200]));
    let result = blender.blend(&frame2);

    // With alpha=1.0, should be exactly frame2
    assert_eq!(result.get_pixel(0, 0).0[0], 200);
}

// ============================================================================
// Temporal Coherence Integration Tests
// ============================================================================

#[test]
fn test_temporal_config_presets() {
    let default = TemporalConfig::default();
    assert!(default.hysteresis_enabled);
    assert!(!default.frame_blend_enabled);
    assert!(!default.dot_filter_enabled);

    let video = TemporalConfig::video();
    assert!(video.hysteresis_enabled);
    assert!(video.frame_blend_enabled);

    let webcam = TemporalConfig::webcam();
    assert!(webcam.hysteresis_enabled);
    assert!(webcam.frame_blend_enabled);
    assert!(webcam.dot_filter_enabled);

    let disabled = TemporalConfig::disabled();
    assert!(!disabled.hysteresis_enabled);
    assert!(!disabled.frame_blend_enabled);
    assert!(!disabled.dot_filter_enabled);
}

#[test]
fn test_temporal_coherence_reduces_flicker() {
    // Create frames that would cause flicker without temporal coherence
    // Pixels alternate between 125 and 130 around threshold 128
    let threshold = 128u8;

    // Without temporal coherence: every frame would flip
    let frames_raw: Vec<GrayImage> = (0..10)
        .map(|i| {
            let value = if i % 2 == 0 { 125 } else { 130 };
            GrayImage::from_fn(10, 10, |_, _| image::Luma([value]))
        })
        .collect();

    // Process without temporal coherence (simple threshold)
    let results_without: Vec<Vec<bool>> = frames_raw
        .iter()
        .map(|frame| {
            frame
                .pixels()
                .map(|p| p.0[0] >= threshold)
                .collect()
        })
        .collect();

    let flicker_without = average_flicker(&results_without);

    // Process with temporal coherence
    let mut coherence = TemporalCoherence::new(TemporalConfig::default());
    let results_with: Vec<Vec<bool>> = frames_raw
        .iter()
        .map(|frame| {
            let binary = coherence.process_grayscale(frame, threshold);
            binary.pixels().map(|p| p.0[0] > 0).collect()
        })
        .collect();

    let flicker_with = average_flicker(&results_with);

    // Temporal coherence should reduce flicker
    assert!(
        flicker_with < flicker_without,
        "Temporal coherence should reduce flicker: without={:.2}%, with={:.2}%",
        flicker_without * 100.0,
        flicker_with * 100.0
    );
}

#[test]
fn test_temporal_coherence_reset() {
    let mut coherence = TemporalCoherence::new(TemporalConfig::video());

    // Process some frames
    let frame = GrayImage::from_fn(10, 10, |_, _| image::Luma([150]));
    let _ = coherence.process_grayscale(&frame, 128);
    let _ = coherence.process_grayscale(&frame, 128);

    // Reset
    coherence.reset();

    // Next frame should be treated as first frame
    let frame2 = GrayImage::from_fn(10, 10, |_, _| image::Luma([100]));
    let result = coherence.process_grayscale(&frame2, 128);

    // 100 < 128, so should be OFF
    assert_eq!(result.get_pixel(0, 0).0[0], 0, "After reset, should use standard threshold");
}

#[test]
fn test_temporal_coherence_config_update() {
    let mut coherence = TemporalCoherence::new(TemporalConfig::default());

    // Verify initial config
    assert!(coherence.config().hysteresis_enabled);
    assert!(!coherence.config().frame_blend_enabled);

    // Update config
    coherence.set_config(TemporalConfig::video());

    // Verify updated config
    assert!(coherence.config().hysteresis_enabled);
    assert!(coherence.config().frame_blend_enabled);
}
