use dotmax::image::{load_svg_from_path, to_grayscale, auto_threshold, pixels_to_braille};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg_path = Path::new("tests/fixtures/svg/svg_test.svg");
    
    println!("Loading SVG from: {:?}", svg_path);
    let img = load_svg_from_path(svg_path, 160, 96)?;
    println!("✓ SVG loaded: {}×{}", img.width(), img.height());
    
    let gray = to_grayscale(&img);
    println!("✓ Converted to grayscale");
    
    let binary = auto_threshold(&gray);
    println!("✓ Thresholded to binary");
    
    let grid = pixels_to_braille(&binary, 80, 24)?;
    println!("✓ Mapped to braille: {}×{}", grid.width(), grid.height());
    
    println!("\nFirst few lines of braille output:");
    for y in 0..5 {
        for x in 0..40 {
            print!("{}", grid.get(x, y).unwrap_or(' '));
        }
        println!();
    }
    
    Ok(())
}
