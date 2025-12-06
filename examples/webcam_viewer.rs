//! Basic webcam viewer example.
//!
//! Demonstrates the simplest way to display live webcam video in the terminal
//! using braille rendering.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example webcam_viewer --features video
//! ```
//!
//! Press any key to exit.
//!
//! # Requirements
//!
//! - FFmpeg libraries installed on the system
//! - A connected webcam
//!
//! # Platform Notes
//!
//! - **Linux**: Requires V4L2 device (usually `/dev/video0`)
//! - **macOS**: Requires AVFoundation camera access permission
//! - **Windows**: Requires DirectShow video input device

use dotmax::prelude::*;

fn main() -> dotmax::Result<()> {
    // The simplest possible webcam display - one line!
    println!("Starting webcam viewer...");
    println!("Press any key to exit.");
    println!();

    // This opens the default webcam and displays live feed
    // until any key is pressed
    show_webcam()?;

    println!("Webcam viewer closed.");
    Ok(())
}
