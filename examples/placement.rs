//! Checks that the overlay actually lands on the display it was aimed at.
//!
//! Placement depends on winit's coordinate conversions and on macOS's rules for
//! which screen a window belongs to, neither of which can be reproduced in a
//! unit test — and both of which have already caused a flourish to appear on
//! the wrong screen. This drives the real `place_overlay` path against every
//! attached display and reports where the window ended up.
//!
//! ```sh
//! cargo run --example placement
//! ```
//!
//! Nothing is drawn, so the window stays transparent throughout; on macOS the
//! menu bar hides briefly while each display is covered.

use flourish::display::{MonitorBounds, monitor_for_point, place_overlay, release_overlay};
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId, WindowLevel},
};

#[derive(Default)]
struct Check {
    failures: usize,
}

fn bounds_of(monitor: &winit::monitor::MonitorHandle) -> MonitorBounds {
    let scale = monitor.scale_factor();
    let scale = if scale > 0.0 { scale } else { 1.0 };
    let position = monitor.position();
    let size = monitor.size();
    MonitorBounds::new(
        [f64::from(position.x) / scale, f64::from(position.y) / scale],
        [
            f64::from(size.width) / scale,
            f64::from(size.height) / scale,
        ],
    )
}

impl ApplicationHandler for Check {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let monitors: Vec<_> = event_loop.available_monitors().collect();
        let all_bounds: Vec<MonitorBounds> = monitors.iter().map(bounds_of).collect();

        let attributes = Window::default_attributes()
            .with_title("Flourish placement check")
            .with_visible(false)
            .with_transparent(true)
            .with_decorations(false)
            .with_resizable(true)
            .with_window_level(WindowLevel::AlwaysOnTop);
        let window = match event_loop.create_window(attributes) {
            Ok(window) => window,
            Err(error) => {
                println!("could not create a window: {error}");
                event_loop.exit();
                return;
            }
        };

        // Twice around, because the reported bug only appeared from the second
        // placement onwards -- the first one happened to work.
        for pass in 1..=2 {
            for (index, monitor) in monitors.iter().enumerate() {
                let expected = all_bounds[index];
                place_overlay(&window, expected, monitor);

                let landed = window
                    .outer_position()
                    .map_or([f64::NAN, f64::NAN], |position| {
                        let scale = window.scale_factor();
                        [f64::from(position.x) / scale, f64::from(position.y) / scale]
                    });
                let landed_on = monitor_for_point(&all_bounds, landed);
                let size = window.inner_size();

                let name = monitor.name().unwrap_or_else(|| "<unnamed>".to_owned());
                let ok = landed_on == Some(index);
                if !ok {
                    self.failures += 1;
                }
                println!(
                    "pass {pass}  aim {index} ({name})\n    \
                     expected logical origin ({:.0}, {:.0}) size {:.0}x{:.0}\n    \
                     landed   logical origin ({:.0}, {:.0}) size {}x{} -> display {:?}  {}",
                    expected.origin[0],
                    expected.origin[1],
                    expected.size[0],
                    expected.size[1],
                    landed[0],
                    landed[1],
                    size.width,
                    size.height,
                    landed_on,
                    if ok { "OK" } else { "WRONG DISPLAY" }
                );
            }
        }

        release_overlay(&window);
        window.set_visible(false);

        println!(
            "\n{} placement(s) landed on the wrong display.",
            self.failures
        );
        event_loop.exit();
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: winit::event::WindowEvent) {}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = EventLoop::builder();

    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        builder
            .with_activation_policy(ActivationPolicy::Accessory)
            .with_default_menu(false);
    }

    let event_loop = builder.build()?;
    let mut check = Check::default();
    event_loop.run_app(&mut check)?;

    if check.failures > 0 {
        std::process::exit(1);
    }
    Ok(())
}
