//! Measures how long each flourish takes to draw.
//!
//! The concern this answers: Flourish asks for the low-power adapter, which on
//! a laptop means the integrated GPU, and several effects are heavy per-pixel
//! shaders — Frosted Glass alone runs two nine-sample Voronoi passes and five
//! domain-warped blooms for every pixel. A presentation machine is also driving
//! a projector and running a deck. Missing frame budget mid-talk is exactly the
//! failure this tool cannot afford.
//!
//! Renders offscreen through [`Scene`], the same drawing path the on-screen
//! renderer uses, so the numbers describe the shipped code rather than a
//! reimplementation of it.

use std::time::Instant;

use flourish::{Flourish, MotionPreference};
use winit::dpi::PhysicalSize;

use crate::renderer::{Frame, POWER_PREFERENCE, RendererError, Scene};

/// Matches what a real surface picks: the first sRGB format offered.
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

/// Resolutions to sweep, smallest first.
const RESOLUTIONS: [(&str, u32, u32); 5] = [
    ("1080p", 1920, 1080),
    ("1440p", 2560, 1440),
    ("MBP 16\"", 3456, 2234),
    ("4K", 3840, 2160),
    ("5K", 5120, 2880),
];

/// Cheap size used to advance the simulations before measuring. Gravel keeps
/// its pile across a resize, so the pile can be built at a resolution that
/// costs nothing and then measured at every real one.
const WARM_UP_SIZE: PhysicalSize<u32> = PhysicalSize::new(640, 360);
/// Simulated seconds to advance before measuring, long enough for the gravel
/// pile to fill and the doom fire to reach the top of its field.
const SETTLE_FRAMES: u32 = 1_500;
const FRAME_STEP: f32 = 1.0 / 60.0;

/// Frames discarded at each resolution before timing, to let the driver settle
/// after a resize.
const DISCARDED_FRAMES: u32 = 8;
/// Frames timed per measurement. Submitted back to back and waited on once, so
/// the result is sustained throughput rather than per-frame latency.
const TIMED_FRAMES: u32 = 40;

const BUDGET_60HZ: f64 = 1000.0 / 60.0;
const BUDGET_120HZ: f64 = 1000.0 / 120.0;

/// An offscreen render target. The returned view keeps the texture alive, so
/// the texture itself does not need to be held.
fn make_target(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Benchmark target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

/// Runs the sweep and prints a table. Returns the worst frame time seen.
pub fn run() -> Result<f64, Box<dyn std::error::Error>> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: POWER_PREFERENCE,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .map_err(|_| RendererError::NoAdapter)?;

    let info = adapter.get_info();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("Flourish benchmark device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        trace: wgpu::Trace::Off,
    }))?;

    println!(
        "Adapter: {} ({:?}, {:?})",
        info.name, info.device_type, info.backend
    );
    println!(
        "Power preference: {POWER_PREFERENCE:?}   \
         budget: {BUDGET_60HZ:.2} ms at 60Hz, {BUDGET_120HZ:.2} ms at 120Hz"
    );
    println!("Sustained milliseconds per frame, {TIMED_FRAMES} frames per measurement.\n");

    let mut scene = Scene::new(device, queue, FORMAT, WARM_UP_SIZE);

    print!("{:<18}", "Flourish");
    for (label, _, _) in RESOLUTIONS {
        print!("{label:>10}");
    }
    println!();
    println!("{}", "-".repeat(18 + 10 * RESOLUTIONS.len()));

    let mut worst = 0.0_f64;
    let mut worst_case = String::new();

    for effect in Flourish::ALL.iter().copied() {
        // Full motion: the calm path holds a frozen clock, which would skip the
        // doom fire's compute step and understate it.
        scene.resize(WARM_UP_SIZE);
        scene.start_effect(effect, MotionPreference::Full);
        let warm_target = make_target(scene.device(), WARM_UP_SIZE.width, WARM_UP_SIZE.height);
        let mut clock = 0.0_f32;
        for _ in 0..SETTLE_FRAMES {
            clock += FRAME_STEP;
            scene.draw_into(&warm_target, effect, Frame::animated(clock, 0.0));
        }
        scene.device().poll(wgpu::PollType::wait_indefinitely())?;
        drop(warm_target);

        print!("{:<18}", effect.label());
        for (label, width, height) in RESOLUTIONS {
            let size = PhysicalSize::new(width, height);
            scene.resize(size);
            let target = make_target(scene.device(), width, height);

            for _ in 0..DISCARDED_FRAMES {
                clock += FRAME_STEP;
                scene.draw_into(&target, effect, Frame::animated(clock, 0.0));
            }
            scene.device().poll(wgpu::PollType::wait_indefinitely())?;

            let started = Instant::now();
            for _ in 0..TIMED_FRAMES {
                clock += FRAME_STEP;
                scene.draw_into(&target, effect, Frame::animated(clock, 0.0));
            }
            scene.device().poll(wgpu::PollType::wait_indefinitely())?;
            let elapsed = started.elapsed();

            let per_frame = elapsed.as_secs_f64() * 1000.0 / f64::from(TIMED_FRAMES);
            if per_frame > worst {
                worst = per_frame;
                worst_case = format!("{} at {label}", effect.label());
            }
            print!("{per_frame:>10.2}");
            drop(target);
        }
        println!();
    }

    println!();
    summarize(worst, &worst_case);
    Ok(worst)
}

fn summarize(worst: f64, worst_case: &str) {
    println!("Worst: {worst:.2} ms/frame — {worst_case}");
    if worst > BUDGET_60HZ {
        println!(
            "OVER BUDGET: that cannot hold 60Hz ({BUDGET_60HZ:.2} ms). \
             Consider rendering the heavy effects at half resolution."
        );
    } else if worst > BUDGET_120HZ {
        println!(
            "Holds 60Hz but not 120Hz ({BUDGET_120HZ:.2} ms). Fine on a \
             projector; may drop frames on a ProMotion panel."
        );
    } else {
        println!("Within budget at both 60Hz and 120Hz.");
    }
    println!(
        "\nMeasured offscreen with no compositor or vsync, so this is the \
         drawing cost alone.\nA real frame also pays presentation."
    );
}

#[cfg(test)]
mod tests {
    use super::{BUDGET_60HZ, BUDGET_120HZ, RESOLUTIONS, WARM_UP_SIZE};

    #[test]
    fn the_budgets_are_the_refresh_intervals_in_milliseconds() {
        // A wrong budget would turn the verdict line into confident nonsense.
        assert!((BUDGET_60HZ - 16.666_666).abs() < 0.001);
        assert!((BUDGET_120HZ - 8.333_333).abs() < 0.001);
    }

    #[test]
    fn the_sweep_runs_smallest_resolution_first() {
        // The table reads as a cost curve, which only works in order.
        let pixels: Vec<u64> = RESOLUTIONS
            .iter()
            .map(|(_, width, height)| u64::from(*width) * u64::from(*height))
            .collect();
        assert!(
            pixels.windows(2).all(|pair| pair[0] < pair[1]),
            "resolutions are not strictly increasing: {pixels:?}"
        );
    }

    #[test]
    fn the_warm_up_size_is_cheap_and_shares_the_sweep_aspect() {
        // Simulations are advanced here and measured elsewhere, so this must
        // cost almost nothing while keeping the gravel aspect close enough
        // that retargeting does not reshape the pile.
        let warm_pixels = u64::from(WARM_UP_SIZE.width) * u64::from(WARM_UP_SIZE.height);
        let smallest = RESOLUTIONS
            .iter()
            .map(|(_, width, height)| u64::from(*width) * u64::from(*height))
            .min()
            .expect("the sweep has resolutions");
        assert!(
            warm_pixels * 4 < smallest,
            "warm-up at {warm_pixels} px is not cheap next to {smallest} px"
        );

        let warm_aspect = f64::from(WARM_UP_SIZE.width) / f64::from(WARM_UP_SIZE.height);
        assert!((warm_aspect - 16.0 / 9.0).abs() < 0.01);
    }
}
