use dotmax::image::{load_svg_from_path, auto_threshold};
use std::path::Path;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let svg_path = Path::new("tests/fixtures/svg/dark_bg_light_content.svg");
    println!("Loading SVG from {:?}", svg_path);

    let img = load_svg_from_path(svg_path, 160, 96).expect("SVG should load");
    println!("Loaded image: {}x{}", img.width(), img.height());

    // Sample some pixels from the loaded image
    let rgba = img.to_rgba8();
    println!("\nSample pixels from rasterized image:");
    for y in [0, 48, 95].iter() {
        for x in [0, 80, 159].iter() {
            let pixel = rgba.get_pixel(*x, *y);
            println!("  Pixel ({}, {}): R={} G={} B={} A={}", x, y, pixel[0], pixel[1], pixel[2], pixel[3]);
        }
    }

    let binary = auto_threshold(&img);
    println!("\nBinary image: {}x{}", binary.width, binary.height);

    // Count black and white pixels
    let mut black_count = 0;
    let mut white_count = 0;
    for y in 0..binary.height as usize {
        for x in 0..binary.width as usize {
            if binary.pixels[y * binary.width as usize + x] {
                black_count += 1;
            } else {
                white_count += 1;
            }
        }
    }

    let total = black_count + white_count;
    let black_percentage = (black_count * 100) / total;

    println!("\nPixel counts:");
    println!("  Black: {} ({}%)", black_count, black_percentage);
    println!("  White: {} ({}%)", white_count, 100 - black_percentage);
    println!("  Total: {}", total);

    if black_count == total {
        println!("\n❌ PROBLEM: All pixels are black!");
    } else if white_count == total {
        println!("\n❌ PROBLEM: All pixels are white!");
    } else {
        println!("\n✅ SUCCESS: Mix of black and white pixels");
    }
}
