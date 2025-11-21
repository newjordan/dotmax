//! Save SVG Rasterization - Debug tool to see rasterized output

use dotmax::image::load_svg_from_path;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg_path = Path::new("tests/fixtures/svg/svg_test.svg");

    println!("Loading SVG: {}", svg_path.display());
    let img = load_svg_from_path(svg_path, 160, 96)?;

    // Save rasterized image to see what it looks like
    img.save("svg_raster_output.png")?;
    println!("Saved rasterized SVG to: svg_raster_output.png");
    println!("Check this image to see how resvg rasterized the SVG");

    Ok(())
}
