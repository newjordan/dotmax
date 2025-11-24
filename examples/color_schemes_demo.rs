//! Visual demonstration of all predefined color schemes.
//!
//! Run with:
//!   cargo run --example color_schemes_demo
//!
//! This example displays horizontal gradient bars for each of the 7 predefined
//! color schemes (rainbow, heat_map, blue_purple, green_yellow, cyan_magenta,
//! grayscale, monochrome), showing how intensity values map to colors.

use dotmax::{get_scheme, list_schemes};

fn main() {
    println!(
        "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    );
    println!("                           DOTMAX COLOR SCHEMES DEMO");
    println!(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"
    );
    println!(
        "Displaying all {} predefined color schemes.",
        list_schemes().len()
    );
    println!("Each bar shows the gradient from intensity 0.0 (left) to 1.0 (right).\n");

    // Display gradient bar for each scheme
    for scheme_name in list_schemes() {
        let scheme = get_scheme(&scheme_name).expect("All listed schemes should exist");

        // Print scheme name
        print!("{:>14}  ", scheme_name);

        // Render a gradient bar with 60 color blocks
        let bar_width = 60;
        for i in 0..bar_width {
            let intensity = i as f32 / (bar_width - 1) as f32;
            let color = scheme.sample(intensity);

            // Output a colored block using truecolor ANSI escape
            print!("\x1b[48;2;{};{};{}m \x1b[0m", color.r, color.g, color.b);
        }

        // Print intensity range labels
        println!("  (0.0 → 1.0)");
    }

    // Footer with usage hints
    println!(
        "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    );
    println!("                                  USAGE EXAMPLE");
    println!(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"
    );
    println!("use dotmax::{{rainbow, get_scheme, ColorScheme}};");
    println!();
    println!("// Method 1: Use free function");
    println!("let scheme = rainbow();");
    println!("let color = scheme.sample(0.5);");
    println!();
    println!("// Method 2: Use associated function");
    println!("let scheme = ColorScheme::rainbow();");
    println!();
    println!("// Method 3: Dynamic lookup by name");
    println!("let scheme = get_scheme(\"rainbow\").unwrap();");
    println!();

    // Show sample colors from rainbow scheme
    println!(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    );
    println!("                              RAINBOW SAMPLES");
    println!(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"
    );

    let rainbow = get_scheme("rainbow").unwrap();
    let samples = [0.0, 0.17, 0.33, 0.50, 0.67, 0.83, 1.0];
    let labels = ["Red", "Orange", "Yellow", "Green", "Cyan", "Blue", "Purple"];

    print!("  ");
    for (i, &intensity) in samples.iter().enumerate() {
        let color = rainbow.sample(intensity);
        print!("\x1b[48;2;{};{};{}m   \x1b[0m ", color.r, color.g, color.b);
        print!("{:<8}", labels[i]);
    }
    println!("\n");

    println!(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"
    );
}
