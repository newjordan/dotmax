//! Generate braille art examples for README documentation
//!
//! Run with:
//! ```bash
//! cargo run --example generate_readme_examples --features image
//! ```

use dotmax::image::ImageRenderer;
use std::fs;
use std::path::Path;

fn grid_to_string(grid: &dotmax::BrailleGrid) -> String {
    let mut output = String::new();
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            output.push(grid.get_char(x, y));
        }
        output.push('\n');
    }
    output
}

fn render_image_to_file(
    image_path: &Path,
    output_path: &Path,
    width: usize,
    height: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Rendering {} -> {}", image_path.display(), output_path.display());

    let grid = ImageRenderer::new()
        .load_from_path(image_path)?
        .resize(width, height, true)?
        .render()?;

    let braille_text = grid_to_string(&grid);
    fs::write(output_path, &braille_text)?;

    println!("  Done! Grid size: {}x{}", grid.width(), grid.height());
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let assets_dir = Path::new("assets/examples");
    let output_dir = Path::new("assets/examples/output");

    // Create output directory
    fs::create_dir_all(output_dir)?;

    // Image dimensions for README examples (terminal-sized)
    let width = 80;
    let height = 30;

    // List of images to process
    let images = ["coastal.jpg", "ant.jpg", "portrait.jpg", "landscape.jpg", "abstract.jpg"];

    println!("Generating braille art examples for README...\n");

    for image_name in &images {
        let image_path = assets_dir.join(image_name);
        if image_path.exists() {
            let output_name = image_name.replace(".jpg", ".txt").replace(".png", ".txt");
            let output_path = output_dir.join(&output_name);

            if let Err(e) = render_image_to_file(&image_path, &output_path, width, height) {
                eprintln!("  Error processing {}: {}", image_name, e);
            }
        } else {
            println!("Skipping {} (not found)", image_name);
        }
    }

    println!("\nDone! Check {} for output files.", output_dir.display());
    Ok(())
}
