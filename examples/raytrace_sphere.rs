//! Minimal end-to-end raytracer demo.
//!
//! Renders a single lit sphere to green braille and prints it. The output
//! is high-resolution internally (2×4 pixels per braille cell) and then
//! packed into unicode braille characters for the terminal.
//!
//! Run with: `cargo run --example raytrace_sphere --features raytracer`

use dotmax::raytracer::{
    intensity_buffer_to_green_braille, render, Camera, RenderMode, Scene, Vector3,
};

fn main() {
    // Pick a rendering resolution. Braille packs 2×4 pixels into each terminal
    // cell, so a 160×96 buffer becomes an 80×24 terminal render.
    let width = 160;
    let height = 96;

    // Camera at the origin looking down -Z, 4:3 viewport.
    let camera = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);

    // Single sphere at (0, 0, -3) with a light up and to the left.
    let scene = Scene::new_with_sphere_and_light();

    let buffer = render(&scene, &camera, width, height, RenderMode::Solid);
    let text = intensity_buffer_to_green_braille(&buffer);
    print!("{}", text);
}
