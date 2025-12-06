//! Webcam selector example with camera enumeration.
//!
//! Demonstrates how to list available cameras and select a specific one
//! for display.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example webcam_selector --features video
//! ```
//!
//! # Features Demonstrated
//!
//! - Listing available webcams with `list_webcams()`
//! - Opening a specific camera by index or path
//! - Using the `WebcamPlayerBuilder` for custom configuration
//! - Runtime render settings adjustment
//!
//! # Requirements
//!
//! - FFmpeg libraries installed
//! - One or more connected webcams

use dotmax::prelude::*;
use std::io::{self, Write};

fn main() -> dotmax::Result<()> {
    println!("=== Webcam Selector Demo ===\n");

    // List available webcams
    let cameras = list_webcams();

    if cameras.is_empty() {
        println!("No webcams detected on this system.");
        println!("\nTroubleshooting:");
        println!("  - Ensure a webcam is connected");
        println!("  - On Linux: check that /dev/video* devices exist");
        println!("  - On macOS: grant camera access in System Preferences");
        println!("  - On Windows: ensure camera drivers are installed");
        return Ok(());
    }

    // Display available cameras
    println!("Available webcams:\n");
    for (i, cam) in cameras.iter().enumerate() {
        println!("  [{i}] {}", cam.name);
        println!("      ID: {}", cam.id);
        println!("      Description: {}", cam.description);
        println!();
    }

    // Get user selection
    print!("Select camera (0-{}), or press Enter for default: ", cameras.len() - 1);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let selection: Option<usize> = if input.is_empty() {
        None
    } else {
        input.parse().ok()
    };

    println!();

    // Open the selected camera
    match selection {
        Some(idx) if idx < cameras.len() => {
            println!("Opening camera {idx}: {}", cameras[idx].name);
            println!("Press any key to exit.\n");

            // Use show_webcam_device with index
            show_webcam_device(idx)?;
        }
        Some(idx) => {
            println!("Invalid selection: {idx}");
            return Ok(());
        }
        None => {
            println!("Opening default camera...");
            println!("Press any key to exit.\n");

            // Use show_webcam for default camera
            show_webcam()?;
        }
    }

    println!("\nWebcam viewer closed.");
    Ok(())
}
