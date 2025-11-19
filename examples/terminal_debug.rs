//! Terminal Debug - Check what terminal actually reports
//!
//! Run with: cargo run --example `terminal_debug`

use crossterm::{
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Terminal Size Debug ===\n");

    // Check BEFORE entering alternate screen
    let (w1, h1) = size()?;
    println!("BEFORE alternate screen: {w1}×{h1}");

    // Check environment variables
    println!("\nEnvironment variables:");
    if let Ok(val) = std::env::var("WT_SESSION") {
        println!("  WT_SESSION = {val}");
    }
    if let Ok(val) = std::env::var("WSL_DISTRO_NAME") {
        println!("  WSL_DISTRO_NAME = {val}");
    }
    if let Ok(val) = std::env::var("TERM") {
        println!("  TERM = {val}");
    }

    println!("\nPress ENTER to enter alternate screen...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Enter raw mode and alternate screen
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // Check AFTER entering alternate screen
    let (w2, h2) = size()?;

    // Write to alternate screen
    write!(stdout, "\r\nIN ALTERNATE SCREEN:\r\n")?;
    write!(stdout, "Size reported: {w2}×{h2}\r\n")?;
    write!(stdout, "\r\n")?;
    write!(stdout, "This text is at the TOP of alternate screen\r\n")?;
    write!(stdout, "\r\n")?;
    write!(stdout, "Is this visible at the top of your terminal?\r\n")?;
    write!(stdout, "\r\n")?;
    write!(stdout, "Press Ctrl+C to exit\r\n")?;
    stdout.flush()?;

    // Wait
    thread::sleep(Duration::from_secs(30));

    // Cleanup
    execute!(stdout, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    println!("\nAFTER exiting alternate screen");
    println!("  Before: {w1}×{h1}");
    println!("  During: {w2}×{h2}");

    Ok(())
}
