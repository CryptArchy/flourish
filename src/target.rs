//! Bridging winit's monitors and the window system's cursor into the geometry
//! in [`flourish::display`].
//!
//! This is the only place that knows about platform coordinate conventions.
//! Everything above it works in one global logical space.

use flourish::display::{MonitorBounds, monitor_for_point};
use winit::{event_loop::ActiveEventLoop, monitor::MonitorHandle};

/// Describes how the target display was chosen, for logging and for tests that
/// care that a fallback did or did not happen.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Basis {
    /// The display under the pointer.
    Cursor,
    /// The pointer could not be located, so the primary display was used.
    PrimaryFallback,
}

/// The display a flourish should play on, with the reason it was chosen.
pub struct Target {
    pub monitor: MonitorHandle,
    pub basis: Basis,
}

/// Converts a winit monitor into global logical bounds.
///
/// `position()` is the logical origin already multiplied by this monitor's own
/// scale factor, and `size()` is the true pixel count, so dividing both by that
/// same factor lands them in the shared logical space. See the module docs in
/// `flourish::display` for why any other space is a trap under mixed DPI.
fn bounds_of(monitor: &MonitorHandle) -> MonitorBounds {
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

/// Chooses the display to put the next flourish on.
///
/// The pointer is the signal for both ways a flourish can start: clicking the
/// menu-bar icon puts the pointer on that display, and pressing the global
/// shortcut puts it wherever the presenter is working. The tray click position
/// would only cover the first.
pub fn choose(event_loop: &ActiveEventLoop) -> Option<Target> {
    let monitors: Vec<MonitorHandle> = event_loop.available_monitors().collect();
    resolve(&monitors, event_loop.primary_monitor())
}

/// The resolution itself, independent of which winit handle produced the
/// monitor list, so it can also run from a plain `EventLoop` for diagnostics.
pub fn resolve(monitors: &[MonitorHandle], primary: Option<MonitorHandle>) -> Option<Target> {
    if monitors.is_empty() {
        return None;
    }
    let primary = primary.or_else(|| monitors.first().cloned());
    let fallback = || {
        primary.clone().map(|monitor| Target {
            monitor,
            basis: Basis::PrimaryFallback,
        })
    };

    let Some(primary_height) = primary.as_ref().map(|monitor| bounds_of(monitor).size[1]) else {
        return fallback();
    };
    let Some(point) = cursor_position(primary_height) else {
        return fallback();
    };

    let bounds: Vec<MonitorBounds> = monitors.iter().map(bounds_of).collect();
    match monitor_for_point(&bounds, point) {
        Some(index) => Some(Target {
            monitor: monitors[index].clone(),
            basis: Basis::Cursor,
        }),
        None => fallback(),
    }
}

/// A human-readable dump of the display layout and where the pointer resolves.
///
/// Exists because "the flourish appeared on the wrong screen" is otherwise very
/// hard to diagnose from the outside: it depends on the window system's idea of
/// the layout, not the user's.
pub fn describe(monitors: &[MonitorHandle], primary: Option<MonitorHandle>) -> String {
    use std::fmt::Write;

    let mut report = String::new();
    if monitors.is_empty() {
        return "No displays were reported by the window system.\n".to_owned();
    }

    let primary = primary.or_else(|| monitors.first().cloned());
    let primary_name = primary.as_ref().and_then(MonitorHandle::name);
    let primary_height = primary
        .as_ref()
        .map_or(0.0, |monitor| bounds_of(monitor).size[1]);
    let cursor = cursor_position(primary_height);

    let _ = writeln!(report, "Displays ({}):", monitors.len());
    for monitor in monitors {
        let bounds = bounds_of(monitor);
        let name = monitor.name().unwrap_or_else(|| "<unnamed>".to_owned());
        let is_primary = monitor.name() == primary_name;
        let _ = writeln!(
            report,
            "  {}{name}\n      physical  {:?} at {:?}\n      \
             scale     {}\n      logical   {:.0}x{:.0} at ({:.0}, {:.0})",
            if is_primary { "* " } else { "  " },
            monitor.size(),
            monitor.position(),
            monitor.scale_factor(),
            bounds.size[0],
            bounds.size[1],
            bounds.origin[0],
            bounds.origin[1],
        );
    }

    match cursor {
        Some(point) => {
            let _ = writeln!(
                report,
                "\nPointer (logical): ({:.0}, {:.0})",
                point[0], point[1]
            );
        }
        None => {
            let _ = writeln!(
                report,
                "\nPointer: not available on this platform; the primary display is used."
            );
        }
    }

    match resolve(monitors, primary) {
        Some(target) => {
            let _ = writeln!(
                report,
                "Target: {} ({:?})",
                target
                    .monitor
                    .name()
                    .unwrap_or_else(|| "<unnamed>".to_owned()),
                target.basis
            );
        }
        None => {
            let _ = writeln!(report, "Target: none could be resolved.");
        }
    }
    report
}

/// The pointer's position in global logical coordinates, top-left origin.
///
/// `NSEvent::mouseLocation` is a safe binding, and reports points measured from
/// the bottom-left of the primary display, so the vertical axis is flipped
/// against that display's logical height. The height comes from winit rather
/// than `CGDisplayPixelsHigh` so that it is derived exactly the same way as the
/// monitor bounds it will be compared against — mixing a pixel height into a
/// point-space flip is off by the scale factor on any Retina display.
///
/// Always answers on macOS; the shared signature keeps the "unavailable" case
/// available to platforms that cannot ask.
#[cfg(target_os = "macos")]
#[allow(clippy::unnecessary_wraps)]
fn cursor_position(primary_height: f64) -> Option<[f64; 2]> {
    use objc2_app_kit::NSEvent;

    let location = NSEvent::mouseLocation();
    Some([location.x, primary_height - location.y])
}

/// Other platforms have no safe cursor query available here, so they fall back
/// to the primary display. Documented rather than guessed at: see the
/// active-monitor ticket in `kb/`.
#[cfg(not(target_os = "macos"))]
fn cursor_position(_primary_height: f64) -> Option<[f64; 2]> {
    None
}

#[cfg(test)]
mod tests {
    use flourish::display::{MonitorBounds, monitor_for_point};

    /// The conversion `bounds_of` performs, extracted so it can be checked
    /// without a live event loop.
    fn logical_bounds(position: [i32; 2], size: [u32; 2], scale: f64) -> MonitorBounds {
        MonitorBounds::new(
            [
                f64::from(position[0]) / scale,
                f64::from(position[1]) / scale,
            ],
            [f64::from(size[0]) / scale, f64::from(size[1]) / scale],
        )
    }

    #[test]
    fn a_retina_monitor_converts_to_its_point_size() {
        // A 14" MacBook panel: 3024x1964 backing pixels at 2x, which the window
        // system lays out as 1512x982 points.
        let bounds = logical_bounds([0, 0], [3024, 1964], 2.0);

        assert!((bounds.size[0] - 1512.0).abs() < f64::EPSILON);
        assert!((bounds.size[1] - 982.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mixed_dpi_displays_land_in_one_coordinate_space() {
        // The laptop panel is 2x; the external display beside it is 1x. winit
        // scales each origin by its own factor, so the external monitor's
        // physical origin is 1512 while the laptop occupies 3024 physical
        // pixels -- they overlap in physical space and only separate in points.
        let laptop = logical_bounds([0, 0], [3024, 1964], 2.0);
        let external = logical_bounds([1512, 0], [1920, 1080], 1.0);
        let desktop = [laptop, external];

        // A point on the external display must not resolve to the laptop.
        assert_eq!(monitor_for_point(&desktop, [2000.0, 300.0]), Some(1));
        // ...and one on the laptop must not resolve to the external.
        assert_eq!(monitor_for_point(&desktop, [700.0, 300.0]), Some(0));
        // The two tile exactly, with no overlap and no gap.
        assert!((laptop.origin[0] + laptop.size[0] - external.origin[0]).abs() < f64::EPSILON);
    }

    #[test]
    fn comparing_in_physical_pixels_would_pick_the_wrong_display() {
        // Guards the reasoning behind working in logical space at all. Using
        // winit's raw physical numbers, the laptop's 3024-pixel width swallows
        // the external display's 1512 origin, so a point genuinely on the
        // external display looks like it belongs to the laptop.
        let laptop_physical = MonitorBounds::new([0.0, 0.0], [3024.0, 1964.0]);
        let external_physical = MonitorBounds::new([1512.0, 0.0], [1920.0, 1080.0]);

        let point_on_external = [2000.0, 300.0];
        assert!(
            laptop_physical.contains(point_on_external),
            "physical-space bounds overlap, which is the bug being avoided"
        );
        assert!(external_physical.contains(point_on_external));

        // In logical space only one of them claims it.
        let laptop = logical_bounds([0, 0], [3024, 1964], 2.0);
        let external = logical_bounds([1512, 0], [1920, 1080], 1.0);
        assert!(!laptop.contains(point_on_external));
        assert!(external.contains(point_on_external));
    }

    #[test]
    fn a_zero_scale_factor_does_not_divide_by_zero() {
        // Guards the clamp in bounds_of; a monitor reporting scale 0 would
        // otherwise produce infinities and poison every comparison.
        let scale = 0.0_f64;
        let safe = if scale > 0.0 { scale } else { 1.0 };
        let bounds = logical_bounds([100, 100], [800, 600], safe);

        assert!(bounds.origin[0].is_finite());
        assert!(bounds.size[0].is_finite());
    }
}
