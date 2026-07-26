//! Writes the macOS `.iconset` directory that `iconutil` turns into an `.icns`.
//!
//! An example rather than a binary so it never ships inside the installed
//! program, and a program rather than a checked-in image so the icon can never
//! drift from the palette the rest of the app draws with.
//!
//! ```sh
//! cargo run --example iconset -- target/Flourish.iconset
//! ```

use std::{fs, io, path::Path};

/// The ladder macOS expects. Every entry is `(file stem, pixel size)`; the
/// `@2x` variants are the same image at twice the nominal point size.
const ICONSET: [(&str, u32); 10] = [
    ("icon_16x16", 16),
    ("icon_16x16@2x", 32),
    ("icon_32x32", 32),
    ("icon_32x32@2x", 64),
    ("icon_128x128", 128),
    ("icon_128x128@2x", 256),
    ("icon_256x256", 256),
    ("icon_256x256@2x", 512),
    ("icon_512x512", 512),
    ("icon_512x512@2x", 1024),
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let destination = std::env::args()
        .nth(1)
        .ok_or("usage: iconset <output .iconset directory>")?;
    let destination = Path::new(&destination);
    fs::create_dir_all(destination)?;

    for (stem, size) in ICONSET {
        let path = destination.join(format!("{stem}.png"));
        write_png(&path, size, &flourish::icon::app_icon_rgba(size))?;
        println!("{} ({size}x{size})", path.display());
    }

    println!(
        "\nwrote {} images to {}",
        ICONSET.len(),
        destination.display()
    );
    Ok(())
}

fn write_png(path: &Path, size: u32, rgba: &[u8]) -> Result<(), png::EncodingError> {
    let file = fs::File::create(path)?;
    let mut encoder = png::Encoder::new(io::BufWriter::new(file), size, size);
    // The rasterizer emits straight (non-premultiplied) 8-bit RGBA.
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(rgba)?;
    writer.finish()
}
