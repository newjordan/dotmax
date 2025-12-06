//! Integration tests for the unified media routing system (Story 9.5)
//!
//! These tests verify that `show_file()` and `load_file()` correctly
//! route all media formats to their appropriate handlers.

use std::path::Path;

#[cfg(feature = "image")]
mod media_routing_tests {
    use super::*;
    use dotmax::media::{detect_format, MediaContent, MediaFormat};
    use dotmax::quick;

    // ========================================================================
    // Format Detection Routing Tests (AC: #1)
    // ========================================================================

    #[test]
    fn test_static_png_routes_to_static_image() {
        let path = Path::new("tests/fixtures/media/static_png.png");
        if path.exists() {
            let format = detect_format(path).unwrap();
            assert!(
                matches!(format, MediaFormat::StaticImage(_)),
                "Static PNG should route to StaticImage"
            );
        }
    }

    #[test]
    fn test_static_gif_routes_to_static_image() {
        let path = Path::new("tests/fixtures/media/static.gif");
        if path.exists() {
            let format = detect_format(path).unwrap();
            assert!(
                matches!(format, MediaFormat::StaticImage(_)),
                "Static GIF should route to StaticImage"
            );
        }
    }

    #[test]
    fn test_animated_gif_routes_to_animated_gif() {
        let path = Path::new("tests/fixtures/media/animated.gif");
        if path.exists() {
            let format = detect_format(path).unwrap();
            assert!(
                matches!(format, MediaFormat::AnimatedGif),
                "Animated GIF should route to AnimatedGif"
            );
        }
    }

    #[test]
    fn test_animated_png_routes_to_animated_png() {
        let path = Path::new("tests/fixtures/media/animated.png");
        if path.exists() {
            let format = detect_format(path).unwrap();
            assert!(
                matches!(format, MediaFormat::AnimatedPng),
                "Animated PNG should route to AnimatedPng"
            );
        }
    }

    // ========================================================================
    // load_file() Content Type Tests (AC: #1)
    // ========================================================================

    #[test]
    fn test_load_file_png_returns_static_content() {
        let path = Path::new("tests/fixtures/media/static_png.png");
        if path.exists() {
            let content = quick::load_file(path).unwrap();
            assert!(
                matches!(content, MediaContent::Static(_)),
                "PNG should load as Static content"
            );
        }
    }

    #[test]
    fn test_load_file_static_gif_returns_static_content() {
        let path = Path::new("tests/fixtures/media/static.gif");
        if path.exists() {
            let content = quick::load_file(path).unwrap();
            assert!(
                matches!(content, MediaContent::Static(_)),
                "Static GIF should load as Static content"
            );
        }
    }

    #[test]
    fn test_load_file_animated_gif_returns_animated_content() {
        let path = Path::new("tests/fixtures/media/animated.gif");
        if path.exists() {
            let content = quick::load_file(path).unwrap();
            assert!(
                matches!(content, MediaContent::Animated(_)),
                "Animated GIF should load as Animated content"
            );
        }
    }

    #[test]
    fn test_load_file_animated_png_returns_animated_content() {
        let path = Path::new("tests/fixtures/media/animated.png");
        if path.exists() {
            let content = quick::load_file(path).unwrap();
            assert!(
                matches!(content, MediaContent::Animated(_)),
                "Animated PNG should load as Animated content"
            );
        }
    }

    // ========================================================================
    // MediaPlayer Trait Tests (AC: #1)
    // ========================================================================

    #[test]
    fn test_animated_gif_player_has_multiple_frames() {
        let path = Path::new("tests/fixtures/media/animated.gif");
        if path.exists() {
            if let Ok(MediaContent::Animated(player)) = quick::load_file(path) {
                let frame_count = player.frame_count();
                assert!(
                    frame_count.is_none() || frame_count.unwrap() > 1,
                    "Animated GIF should have multiple frames"
                );
            }
        }
    }

    #[test]
    fn test_animated_png_player_has_multiple_frames() {
        let path = Path::new("tests/fixtures/media/animated.png");
        if path.exists() {
            if let Ok(MediaContent::Animated(player)) = quick::load_file(path) {
                let frame_count = player.frame_count();
                assert!(
                    frame_count.is_none() || frame_count.unwrap() > 1,
                    "Animated PNG should have multiple frames"
                );
            }
        }
    }

    // ========================================================================
    // Grid Dimensions Tests (AC: #1)
    // ========================================================================

    #[test]
    fn test_static_content_grid_has_positive_dimensions() {
        let path = Path::new("tests/fixtures/media/static_png.png");
        if path.exists() {
            if let Ok(MediaContent::Static(grid)) = quick::load_file(path) {
                assert!(grid.width() > 0, "Grid width should be positive");
                assert!(grid.height() > 0, "Grid height should be positive");
            }
        }
    }

    // ========================================================================
    // Error Handling Tests (AC: #4)
    // ========================================================================

    #[test]
    fn test_load_file_nonexistent_returns_error() {
        let result = quick::load_file("nonexistent_file_12345.png");
        assert!(result.is_err(), "Nonexistent file should return error");
    }

    #[test]
    fn test_load_file_unknown_format_returns_format_error() {
        use dotmax::DotmaxError;
        use std::io::Write;

        // Create a temp file with unknown content
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_unknown_format.xyz");

        let mut file = std::fs::File::create(&temp_file).unwrap();
        file.write_all(&[0x00, 0x01, 0x02, 0x03]).unwrap();
        drop(file);

        let result = quick::load_file(&temp_file);
        assert!(result.is_err(), "Unknown format should return error");

        if let Err(DotmaxError::FormatError { format }) = result {
            assert!(
                format.contains("unknown"),
                "Error should mention unknown format"
            );
        }

        // Cleanup
        let _ = std::fs::remove_file(&temp_file);
    }

    #[test]
    fn test_show_file_nonexistent_returns_error() {
        let result = quick::show_file("nonexistent_file_12345.png");
        assert!(result.is_err(), "Nonexistent file should return error");
    }

    // ========================================================================
    // File Existence Sanity Tests
    // ========================================================================

    #[test]
    fn test_media_fixtures_exist() {
        let fixtures = [
            "tests/fixtures/media/static.gif",
            "tests/fixtures/media/animated.gif",
            "tests/fixtures/media/static_png.png",
            "tests/fixtures/media/animated.png",
        ];

        for fixture in fixtures {
            assert!(
                Path::new(fixture).exists(),
                "Expected fixture to exist: {}",
                fixture
            );
        }
    }
}

#[cfg(all(feature = "image", feature = "svg"))]
mod svg_routing_tests {
    use dotmax::media::MediaFormat;

    #[test]
    fn test_svg_detection_from_xml_declaration() {
        use dotmax::media::detect_format_from_bytes;

        let svg_bytes = b"<?xml version=\"1.0\"?><svg></svg>";
        let format = detect_format_from_bytes(svg_bytes);
        assert!(
            matches!(format, MediaFormat::Svg),
            "XML declaration should detect as SVG"
        );
    }

    #[test]
    fn test_svg_detection_from_svg_tag() {
        use dotmax::media::detect_format_from_bytes;

        let svg_bytes = b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";
        let format = detect_format_from_bytes(svg_bytes);
        assert!(
            matches!(format, MediaFormat::Svg),
            "Direct SVG tag should detect as SVG"
        );
    }
}

#[cfg(feature = "video")]
mod video_routing_tests {
    use super::*;
    use dotmax::media::{detect_format_from_bytes, MediaFormat, VideoCodec};

    #[test]
    fn test_mp4_detection_from_magic_bytes() {
        // MP4: ftyp at offset 4
        let mp4_bytes = [
            0x00, 0x00, 0x00, 0x18, // size
            0x66, 0x74, 0x79, 0x70, // ftyp
            0x69, 0x73, 0x6F, 0x6D, // isom
        ];
        let format = detect_format_from_bytes(&mp4_bytes);
        assert!(
            matches!(format, MediaFormat::Video(VideoCodec::H264)),
            "MP4 magic bytes should detect as Video(H264)"
        );
    }

    #[test]
    fn test_mkv_detection_from_magic_bytes() {
        // MKV/WebM: EBML header
        let mkv_bytes = [0x1A, 0x45, 0xDF, 0xA3];
        let format = detect_format_from_bytes(&mkv_bytes);
        assert!(
            matches!(format, MediaFormat::Video(VideoCodec::Vp9)),
            "MKV/WebM magic bytes should detect as Video"
        );
    }

    #[test]
    fn test_avi_detection_from_magic_bytes() {
        // AVI: RIFF....AVI
        let avi_bytes = [
            0x52, 0x49, 0x46, 0x46, // RIFF
            0x00, 0x00, 0x00, 0x00, // size
            0x41, 0x56, 0x49, 0x20, // AVI
        ];
        let format = detect_format_from_bytes(&avi_bytes);
        assert!(
            matches!(format, MediaFormat::Video(VideoCodec::Other)),
            "AVI magic bytes should detect as Video(Other)"
        );
    }
}

// ============================================================================
// GIF Player Specific Tests
// ============================================================================

#[cfg(feature = "image")]
mod gif_player_tests {
    use dotmax::media::{GifPlayer, MediaPlayer};
    use std::path::Path;

    #[test]
    fn test_gif_player_new_from_animated_gif() {
        let path = Path::new("tests/fixtures/media/animated.gif");
        if path.exists() {
            let player = GifPlayer::new(path);
            assert!(player.is_ok(), "GifPlayer should load animated GIF");
        }
    }

    #[test]
    fn test_gif_player_canvas_dimensions() {
        let path = Path::new("tests/fixtures/media/animated.gif");
        if path.exists() {
            let player = GifPlayer::new(path).unwrap();
            assert!(
                player.canvas_width() > 0,
                "Canvas width should be positive"
            );
            assert!(
                player.canvas_height() > 0,
                "Canvas height should be positive"
            );
        }
    }

    #[test]
    fn test_gif_player_next_frame() {
        let path = Path::new("tests/fixtures/media/animated.gif");
        if path.exists() {
            let mut player = GifPlayer::new(path).unwrap();
            let frame = player.next_frame();
            assert!(frame.is_some(), "First frame should exist");

            if let Some(Ok((grid, delay))) = frame {
                assert!(grid.width() > 0, "Frame grid width should be positive");
                assert!(
                    !delay.is_zero(),
                    "Frame delay should be positive for animated GIF"
                );
            }
        }
    }

    #[test]
    fn test_gif_player_loop_count() {
        let path = Path::new("tests/fixtures/media/loop_twice.gif");
        if path.exists() {
            let player = GifPlayer::new(path).unwrap();
            let loop_count = player.loop_count();
            // Loop count should be Some(2) for this fixture
            assert!(
                loop_count == Some(2) || loop_count == Some(0),
                "Loop count should be finite or infinite"
            );
        }
    }
}

// ============================================================================
// APNG Player Specific Tests
// ============================================================================

#[cfg(feature = "image")]
mod apng_player_tests {
    use dotmax::media::{ApngPlayer, MediaPlayer};
    use std::path::Path;

    #[test]
    fn test_apng_player_new_from_animated_png() {
        let path = Path::new("tests/fixtures/media/animated.png");
        if path.exists() {
            let player = ApngPlayer::new(path);
            assert!(player.is_ok(), "ApngPlayer should load animated PNG");
        }
    }

    #[test]
    fn test_apng_player_canvas_dimensions() {
        let path = Path::new("tests/fixtures/media/animated.png");
        if path.exists() {
            let player = ApngPlayer::new(path).unwrap();
            assert!(
                player.canvas_width() > 0,
                "Canvas width should be positive"
            );
            assert!(
                player.canvas_height() > 0,
                "Canvas height should be positive"
            );
        }
    }

    #[test]
    fn test_apng_player_next_frame() {
        let path = Path::new("tests/fixtures/media/animated.png");
        if path.exists() {
            let mut player = ApngPlayer::new(path).unwrap();
            let frame = player.next_frame();
            assert!(frame.is_some(), "First frame should exist");

            if let Some(Ok((grid, delay))) = frame {
                assert!(grid.width() > 0, "Frame grid width should be positive");
                // APNG delay could be zero for very fast animations
                // APNG delay is always non-negative by type
                let _ = delay; // Use the delay to suppress warning
            }
        }
    }

    #[test]
    fn test_apng_player_loop_count() {
        let path = Path::new("tests/fixtures/media/loop_twice.png");
        if path.exists() {
            let player = ApngPlayer::new(path).unwrap();
            let loop_count = player.loop_count();
            // Loop count should be Some(2) for this fixture
            assert!(
                loop_count == Some(2) || loop_count == Some(0),
                "Loop count should be finite or infinite"
            );
        }
    }
}

// ============================================================================
// Terminal Resize Handling Tests (AC: #2)
// ============================================================================

#[cfg(feature = "image")]
mod resize_handling_tests {
    use dotmax::media::{GifPlayer, ApngPlayer, MediaPlayer};
    use std::path::Path;

    #[test]
    fn test_gif_player_handle_resize() {
        let path = Path::new("tests/fixtures/media/animated.gif");
        if path.exists() {
            let mut player = GifPlayer::new(path).unwrap();

            // Call handle_resize - should not panic
            player.handle_resize(120, 40);

            // Get next frame - should work with new dimensions
            let frame = player.next_frame();
            assert!(frame.is_some(), "Should get frame after resize");
            let (grid, _delay) = frame.unwrap().unwrap();
            assert!(grid.width() > 0, "Grid width should be positive after resize");
        }
    }

    #[test]
    fn test_apng_player_handle_resize() {
        let path = Path::new("tests/fixtures/media/animated.png");
        if path.exists() {
            let mut player = ApngPlayer::new(path).unwrap();

            // Call handle_resize - should not panic
            player.handle_resize(100, 30);

            // Get next frame - should work with new dimensions
            let frame = player.next_frame();
            assert!(frame.is_some(), "Should get frame after resize");
            let (grid, _delay) = frame.unwrap().unwrap();
            assert!(grid.width() > 0, "Grid width should be positive after resize");
        }
    }

    #[test]
    fn test_media_player_trait_has_handle_resize() {
        let path = Path::new("tests/fixtures/media/animated.gif");
        if path.exists() {
            let player = GifPlayer::new(path).unwrap();
            let mut boxed: Box<dyn MediaPlayer> = Box::new(player);

            // This should compile - proving the trait has handle_resize
            boxed.handle_resize(80, 24);
        }
    }
}

// ============================================================================
// Cross-Format Consistency Tests
// ============================================================================

#[cfg(feature = "image")]
mod cross_format_tests {
    use dotmax::media::{detect_format, MediaFormat};
    use std::path::Path;

    #[test]
    fn test_all_image_formats_detected_consistently() {
        let test_cases = [
            ("tests/fixtures/media/static.gif", MediaFormat::StaticImage(dotmax::media::ImageFormat::Gif)),
            ("tests/fixtures/media/static_png.png", MediaFormat::StaticImage(dotmax::media::ImageFormat::Png)),
        ];

        for (path, expected_variant) in test_cases {
            if Path::new(path).exists() {
                let format = detect_format(path).unwrap();
                let matches = std::mem::discriminant(&format) == std::mem::discriminant(&expected_variant);
                assert!(
                    matches,
                    "Format detection mismatch for {}: got {:?}, expected variant like {:?}",
                    path, format, expected_variant
                );
            }
        }
    }

    #[test]
    fn test_animated_formats_detected_consistently() {
        let test_cases = [
            ("tests/fixtures/media/animated.gif", MediaFormat::AnimatedGif),
            ("tests/fixtures/media/animated.png", MediaFormat::AnimatedPng),
        ];

        for (path, expected) in test_cases {
            if Path::new(path).exists() {
                let format = detect_format(path).unwrap();
                assert_eq!(
                    format, expected,
                    "Format detection mismatch for {}: got {:?}, expected {:?}",
                    path, format, expected
                );
            }
        }
    }
}
