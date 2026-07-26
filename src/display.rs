//! Choosing which display a flourish plays on.
//!
//! A presenter with two screens expects the flourish where they are looking,
//! not on whichever display the window system calls primary.
//!
//! # Why logical coordinates
//!
//! This module works entirely in the window system's **global logical** space —
//! points, not pixels — and that choice is load-bearing rather than incidental.
//!
//! winit reports a monitor's `position()` as its logical origin already
//! multiplied by *that monitor's own* scale factor, while `size()` is the true
//! pixel count. On a uniform-DPI desktop the two conventions coincide and any
//! arithmetic appears to work. Mix a 2× laptop panel with a 1× external display
//! and physical monitor origins stop forming a single coordinate system: the
//! same global point maps to different physical values depending on which
//! monitor did the scaling, and containment tests silently pick the wrong
//! screen. Dividing both position and size by that monitor's scale factor
//! recovers the one space every monitor genuinely shares.

/// A monitor's rectangle in the global logical coordinate space, with the
/// origin at the top-left of the primary display and `y` increasing downwards.
///
/// Coordinates may be negative: a display arranged above or to the left of the
/// primary one has a negative origin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonitorBounds {
    pub origin: [f64; 2],
    pub size: [f64; 2],
}

impl MonitorBounds {
    #[must_use]
    pub const fn new(origin: [f64; 2], size: [f64; 2]) -> Self {
        Self { origin, size }
    }

    /// Whether `point` falls inside this monitor.
    ///
    /// Half-open on the far edges, so two abutting displays never both claim a
    /// point on their shared border.
    #[must_use]
    pub fn contains(&self, point: [f64; 2]) -> bool {
        point[0] >= self.origin[0]
            && point[0] < self.origin[0] + self.size[0]
            && point[1] >= self.origin[1]
            && point[1] < self.origin[1] + self.size[1]
    }

    /// Squared distance from `point` to the nearest spot on this monitor; zero
    /// when the point is inside.
    ///
    /// Squared to keep the comparison exact and avoid a needless square root.
    #[must_use]
    pub fn distance_squared_to(&self, point: [f64; 2]) -> f64 {
        let clamped_x = point[0].clamp(self.origin[0], self.origin[0] + self.size[0]);
        let clamped_y = point[1].clamp(self.origin[1], self.origin[1] + self.size[1]);
        let dx = point[0] - clamped_x;
        let dy = point[1] - clamped_y;
        dx.mul_add(dx, dy * dy)
    }
}

/// Picks the monitor a point belongs to.
///
/// Returns an index into `monitors`, or `None` if there are none. A point that
/// lands in a gap between displays — desktops need not tile, and the cursor can
/// sit in dead space between mismatched screens — resolves to the nearest
/// monitor rather than giving up, because falling back to primary would send
/// the flourish to a screen the presenter is demonstrably not using.
#[must_use]
pub fn monitor_for_point(monitors: &[MonitorBounds], point: [f64; 2]) -> Option<usize> {
    if monitors.is_empty() {
        return None;
    }
    if !point[0].is_finite() || !point[1].is_finite() {
        return Some(0);
    }

    if let Some(index) = monitors.iter().position(|monitor| monitor.contains(point)) {
        return Some(index);
    }

    let mut best = 0;
    let mut best_distance = f64::INFINITY;
    for (index, monitor) in monitors.iter().enumerate() {
        let distance = monitor.distance_squared_to(point);
        if distance < best_distance {
            best_distance = distance;
            best = index;
        }
    }
    Some(best)
}

#[cfg(test)]
mod tests {
    use super::{MonitorBounds, monitor_for_point};

    /// A 2× laptop panel with a 1× external display to its right. In physical
    /// pixels these two overlap; in logical points they abut correctly.
    fn mixed_dpi_desktop() -> Vec<MonitorBounds> {
        vec![
            MonitorBounds::new([0.0, 0.0], [1512.0, 982.0]),
            MonitorBounds::new([1512.0, 0.0], [1920.0, 1080.0]),
        ]
    }

    #[test]
    fn a_point_resolves_to_the_monitor_containing_it() {
        let desktop = mixed_dpi_desktop();

        assert_eq!(monitor_for_point(&desktop, [10.0, 10.0]), Some(0));
        assert_eq!(monitor_for_point(&desktop, [1600.0, 500.0]), Some(1));
    }

    #[test]
    fn shared_borders_belong_to_exactly_one_monitor() {
        let desktop = mixed_dpi_desktop();

        // The seam at x=1512 is the second monitor's first column, not the
        // first monitor's last. Both claiming it would make the choice depend
        // on iteration order.
        assert!(!desktop[0].contains([1512.0, 400.0]));
        assert!(desktop[1].contains([1512.0, 400.0]));
        assert_eq!(monitor_for_point(&desktop, [1512.0, 400.0]), Some(1));

        assert!(desktop[0].contains([1511.0, 400.0]));
        assert_eq!(monitor_for_point(&desktop, [1511.0, 400.0]), Some(0));
    }

    #[test]
    fn displays_above_and_left_of_primary_have_negative_origins() {
        // macOS and Windows both allow this, and it is the arrangement most
        // likely to break naive containment arithmetic.
        let desktop = vec![
            MonitorBounds::new([0.0, 0.0], [1512.0, 982.0]),
            MonitorBounds::new([-1920.0, -200.0], [1920.0, 1080.0]),
        ];

        assert_eq!(monitor_for_point(&desktop, [-100.0, -50.0]), Some(1));
        assert_eq!(monitor_for_point(&desktop, [-1920.0, -200.0]), Some(1));
        assert_eq!(monitor_for_point(&desktop, [5.0, 5.0]), Some(0));
    }

    #[test]
    fn a_point_in_the_gap_between_displays_takes_the_nearest() {
        // Vertically offset displays leave dead space the cursor can occupy.
        let desktop = vec![
            MonitorBounds::new([0.0, 0.0], [1000.0, 1000.0]),
            MonitorBounds::new([1200.0, 0.0], [1000.0, 1000.0]),
        ];

        assert_eq!(monitor_for_point(&desktop, [1050.0, 500.0]), Some(0));
        assert_eq!(monitor_for_point(&desktop, [1150.0, 500.0]), Some(1));
    }

    #[test]
    fn a_point_far_outside_every_display_still_resolves() {
        let desktop = mixed_dpi_desktop();

        // Never None: the caller has to put the flourish somewhere.
        assert!(monitor_for_point(&desktop, [99_999.0, 99_999.0]).is_some());
        assert!(monitor_for_point(&desktop, [-99_999.0, -99_999.0]).is_some());
    }

    #[test]
    fn a_desktop_with_one_monitor_always_resolves_to_it() {
        let desktop = vec![MonitorBounds::new([0.0, 0.0], [1512.0, 982.0])];

        for point in [[0.0, 0.0], [700.0, 400.0], [-5000.0, 12.0], [9e9, -9e9]] {
            assert_eq!(monitor_for_point(&desktop, point), Some(0));
        }
    }

    #[test]
    fn no_monitors_resolves_to_nothing() {
        assert_eq!(monitor_for_point(&[], [0.0, 0.0]), None);
    }

    #[test]
    fn a_non_finite_point_falls_back_rather_than_comparing() {
        // NaN comparisons are all false, so an unguarded search would return
        // whichever monitor happened to be first anyway -- but by accident.
        let desktop = mixed_dpi_desktop();

        assert_eq!(monitor_for_point(&desktop, [f64::NAN, 0.0]), Some(0));
        assert_eq!(monitor_for_point(&desktop, [0.0, f64::INFINITY]), Some(0));
    }

    #[test]
    fn stacked_displays_resolve_by_vertical_position() {
        let desktop = vec![
            MonitorBounds::new([0.0, 0.0], [1920.0, 1080.0]),
            MonitorBounds::new([0.0, -1080.0], [1920.0, 1080.0]),
        ];

        assert_eq!(monitor_for_point(&desktop, [960.0, 500.0]), Some(0));
        assert_eq!(monitor_for_point(&desktop, [960.0, -500.0]), Some(1));
        assert_eq!(monitor_for_point(&desktop, [960.0, -1080.0]), Some(1));
    }
}
