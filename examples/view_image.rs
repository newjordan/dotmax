//! Example: View image metadata from any file path
//!
//! Run with:
//! ```bash
//! cargo run --example view_image --features image -- path/to/image.png
//! ```

use dotmax::image::load_from_path;
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Get image path from command line args
    let args: Vec<String> = env::args().collect();
    let image_path = if args.len() > 1 {
        &args[1]
    } else {
        "examples/tiger_1.png" // Default to tiger image
    };

    println!("═══════════════════════════════════════════════");
    println!("  dotmax - Image Viewer");
    println!("═══════════════════════════════════════════════\n");
    println!("📂 Loading: {}\n", image_path);

    let path = Path::new(image_path);
    match load_from_path(path) {
        Ok(img) => {
            println!("✅ Successfully loaded image!\n");
            println!("📊 Image Details:");
            println!("   - Dimensions: {}×{} pixels", img.width(), img.height());
            println!("   - Color Type: {:?}", img.color());
            println!("   - Total Pixels: {}", img.width() * img.height());

            // Calculate braille cell dimensions (2×4 dots per cell)
            let braille_width = (img.width() + 1) / 2; // 2 dots wide per cell
            let braille_height = (img.height() + 3) / 4; // 4 dots tall per cell
            println!("\n🔲 Braille Rendering Info:");
            println!(
                "   - Would render as: {}×{} braille cells",
                braille_width, braille_height
            );
            println!(
                "   - Terminal size estimate: {} columns × {} rows",
                braille_width, braille_height
            );

            println!("\n✨ Image loaded successfully!");
        }
        Err(e) => {
            println!("❌ Failed to load image:");
            println!("   Error: {}", e);
            println!("\nTip: Make sure the file exists and is a supported format");
            println!("     Supported: PNG, JPG, GIF, BMP, WebP, TIFF");
        }
    }

    println!("\n═══════════════════════════════════════════════");
    Ok(())
}
