//! Writes a filmstrip of every flourish without playing one on screen.
//!
//! Choosing a flourish for a talk otherwise means launching each candidate
//! full-screen, which is a poor way to compare seventeen of them and a worse
//! way to do it ten minutes before speaking.
//!
//! Renders offscreen through [`Scene`], the same drawing path the on-screen
//! renderer uses, so a strip shows the shipped effect rather than an
//! approximation of it.

use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use flourish::{Flourish, MotionPreference};
use winit::dpi::PhysicalSize;

use crate::renderer::{Frame, POWER_PREFERENCE, RendererError, Scene};

/// Matches what a real surface picks: the first sRGB format offered.
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

/// One tile of a strip. 512 pixels is 2048 bytes per row, a multiple of the
/// 256-byte alignment `copy_texture_to_buffer` requires — which is the only
/// reason this is not a rounder number.
const TILE_WIDTH: u32 = 512;
const TILE_HEIGHT: u32 = 288;
const BYTES_PER_PIXEL: u32 = 4;

/// Where each tile sits in the flourish's life: the hold, then four points
/// through the exit. The last is late enough to show the screen coming back
/// but early enough that something is still on its way out.
const STAGES: [f32; 5] = [0.0, 0.25, 0.5, 0.7, 0.9];

/// Simulated seconds of hold before capturing, so Gravel Fall has built a pile
/// and Doom Fire has filled its field rather than showing an empty floor.
const SETTLE_FRAMES: u32 = 900;
const FRAME_STEP: f32 = 1.0 / 60.0;

/// Renders every flourish into `directory` and prints where each strip landed.
pub fn run(directory: &str) -> Result<(), Box<dyn std::error::Error>> {
    let directory = PathBuf::from(directory);
    fs::create_dir_all(&directory)?;

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: POWER_PREFERENCE,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .map_err(|_| RendererError::NoAdapter)?;

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("Flourish frames device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        trace: wgpu::Trace::Off,
    }))?;

    let size = PhysicalSize::new(TILE_WIDTH, TILE_HEIGHT);
    let mut scene = Scene::new(device, queue, FORMAT, size);

    let texture = scene.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("Frames target"),
        size: wgpu::Extent3d {
            width: TILE_WIDTH,
            height: TILE_HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let readback = scene.device().create_buffer(&wgpu::BufferDescriptor {
        label: Some("Frames readback"),
        size: u64::from(TILE_WIDTH * TILE_HEIGHT * BYTES_PER_PIXEL),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    println!(
        "Writing {} filmstrips to {}",
        Flourish::ALL.len(),
        directory.display()
    );

    for effect in Flourish::ALL.iter().copied() {
        let strip = capture(&mut scene, &texture, &view, &readback, effect)?;
        let path = directory.join(format!("{}.png", effect.slug()));
        write_png(&path, &strip)?;
        println!("  {}", path.display());
    }

    println!(
        "\nEach strip runs left to right: the hold state, then the exit at \
         {}.",
        STAGES[1..]
            .iter()
            .map(|stage| format!("{:.0}%", stage * 100.0))
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

/// One flourish's strip, as `TILE_WIDTH * STAGES.len()` wide RGBA rows.
fn capture(
    scene: &mut Scene,
    texture: &wgpu::Texture,
    view: &wgpu::TextureView,
    readback: &wgpu::Buffer,
    effect: Flourish,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let strip_width = TILE_WIDTH as usize * STAGES.len();
    let mut strip = vec![0_u8; strip_width * TILE_HEIGHT as usize * BYTES_PER_PIXEL as usize];

    // Full motion: the calm path holds a frozen clock, which would leave the
    // stateful simulations empty and every exit stage identical.
    scene.start_effect(effect, MotionPreference::Full);
    let mut clock = 0.0_f32;
    for _ in 0..SETTLE_FRAMES {
        clock += FRAME_STEP;
        scene.draw_into(view, effect, Frame::animated(clock, 0.0));
    }

    for (column, stage) in STAGES.iter().copied().enumerate() {
        // A few frames per stage so the simulations keep advancing rather than
        // freezing on the settled pile for the whole exit.
        for _ in 0..6 {
            clock += FRAME_STEP;
            scene.draw_into(view, effect, Frame::animated(clock, stage));
        }

        let mut encoder = scene
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Frames readback"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(TILE_WIDTH * BYTES_PER_PIXEL),
                    rows_per_image: Some(TILE_HEIGHT),
                },
            },
            wgpu::Extent3d {
                width: TILE_WIDTH,
                height: TILE_HEIGHT,
                depth_or_array_layers: 1,
            },
        );
        scene.queue().submit(Some(encoder.finish()));

        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        scene.device().poll(wgpu::PollType::wait_indefinitely())?;
        let tile = slice.get_mapped_range()?.to_vec();
        readback.unmap();

        place(&mut strip, strip_width, column, &tile);
    }

    Ok(strip)
}

/// Copies one captured tile into its column of the strip, converting BGRA to
/// RGBA and compositing over a stand-in desktop.
///
/// The compositing is the point: a flourish is a transparent overlay, so a
/// tile saved as-is would show a late exit stage as almost nothing at all, and
/// would hide the difference between "reveals the screen" and "fades to black".
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn place(strip: &mut [u8], strip_width: usize, column: usize, tile: &[u8]) {
    for y in 0..TILE_HEIGHT as usize {
        for x in 0..TILE_WIDTH as usize {
            let source = (y * TILE_WIDTH as usize + x) * BYTES_PER_PIXEL as usize;
            let target =
                (y * strip_width + column * TILE_WIDTH as usize + x) * BYTES_PER_PIXEL as usize;
            // The overlay is premultiplied, so the backdrop is simply added in
            // proportion to what the overlay left transparent.
            let alpha = f32::from(tile[source + 3]) / 255.0;
            let backdrop = desktop(x, y);
            for channel in 0..3 {
                // The target format is BGRA; PNG wants RGBA.
                let value = f32::from(tile[source + 2 - channel]);
                let composited = (value + backdrop[channel] * (1.0 - alpha)).clamp(0.0, 255.0);
                // Clamped above, so this cannot wrap or lose a sign.
                strip[target + channel] = composited.round() as u8;
            }
            strip[target + 3] = 255;
        }
    }
}

/// A stand-in for whatever is on the presenter's screen. Deliberately not flat:
/// a gradient shows at a glance which parts of a flourish are transparent.
#[allow(clippy::cast_precision_loss)]
fn desktop(x: usize, y: usize) -> [f32; 3] {
    // Both are bounded by the tile, which is a few hundred pixels.
    let across = x as f32 / TILE_WIDTH as f32;
    let down = y as f32 / TILE_HEIGHT as f32;
    [
        58.0 + 92.0 * across,
        104.0 + 26.0 * down,
        152.0 - 62.0 * down,
    ]
}

fn write_png(path: &Path, strip: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let width = TILE_WIDTH * u32::try_from(STAGES.len())?;
    let file = File::create(path)?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), width, TILE_HEIGHT);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(strip)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{BYTES_PER_PIXEL, STAGES, TILE_HEIGHT, TILE_WIDTH};

    /// What `copy_texture_to_buffer` demands of a row, in bytes.
    const COPY_ALIGNMENT: u32 = 256;

    #[test]
    fn a_tile_row_meets_the_copy_alignment() {
        // wgpu rejects a texture-to-buffer copy whose row is not a multiple of
        // 256 bytes. Picking a rounder tile width would fail at runtime, on a
        // path that only runs when someone asks for frames.
        assert_eq!((TILE_WIDTH * BYTES_PER_PIXEL) % COPY_ALIGNMENT, 0);
    }

    #[test]
    fn the_tile_is_a_widescreen_shape() {
        // Effects are composed for a presentation display; a square preview
        // would misrepresent every one of them.
        let aspect = f64::from(TILE_WIDTH) / f64::from(TILE_HEIGHT);
        assert!((aspect - 16.0 / 9.0).abs() < 0.01);
    }

    #[test]
    fn the_stages_start_at_the_hold_and_end_before_the_exit_does() {
        assert!(STAGES[0].abs() < f32::EPSILON, "the first tile is the hold");
        assert!(STAGES.windows(2).all(|pair| pair[0] < pair[1]));
        // A tile at 1.0 would be an empty frame for every flourish in the
        // catalog, which is a wasted fifth of every strip.
        assert!(STAGES[STAGES.len() - 1] < 1.0);
    }
}
