//! Whether the presenter — or their audience — wants motion.
//!
//! Flourish is built out of movement, so "reduce motion" cannot mean "do
//! nothing". It means the same artwork, held still: a settled composition that
//! cross-fades in and out instead of sweeping, spiralling, or collapsing.
//!
//! This matters beyond personal preference. A flourish fills a projector screen
//! in front of a room, and several of them — the CRT's static, the fire's
//! flicker, the kaleidoscope's rotation — are exactly the kind of rapid,
//! high-contrast movement that provokes migraine and, in the worst case,
//! photosensitive seizures. The person affected is usually in the audience and
//! has no way to ask for it to stop.

use std::time::Duration;

/// How much movement a flourish may use.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MotionPreference {
    /// Flourishes animate as designed.
    #[default]
    Full,
    /// Flourishes hold a settled composition and cross-fade instead of moving.
    Reduced,
}

impl MotionPreference {
    #[must_use]
    pub const fn is_reduced(self) -> bool {
        matches!(self, Self::Reduced)
    }

    #[must_use]
    pub const fn from_reduced(reduced: bool) -> Self {
        if reduced { Self::Reduced } else { Self::Full }
    }

    /// The label for the menu item that toggles this.
    #[must_use]
    pub const fn menu_label(self) -> &'static str {
        match self {
            Self::Full => "Reduce Motion",
            Self::Reduced => "Reduce Motion ✓",
        }
    }

    #[must_use]
    pub const fn toggled(self) -> Self {
        match self {
            Self::Full => Self::Reduced,
            Self::Reduced => Self::Full,
        }
    }
}

/// How long a calm flourish takes to fade in.
///
/// Cutting instantly to a full-screen image is itself a jarring change, so even
/// the reduced path has an entrance — it is just an opacity ramp rather than a
/// movement.
pub const CALM_FADE_IN: Duration = Duration::from_millis(420);

/// The point on each effect's clock that the calm path holds.
///
/// Not zero: most flourishes are still assembling themselves at t=0 — the frost
/// has not crept in, the ripples have not spread, the fire has not caught — so a
/// frozen zero would show an unformed picture. Eight seconds is past the point
/// where every clock-driven effect has settled into its steady state.
pub const SETTLED_SECONDS: f32 = 8.0;

/// Reads the operating system's reduce-motion setting.
///
/// Returns [`MotionPreference::Full`] whenever the answer is unknown. Guessing
/// "reduced" on a machine that never asked for it would quietly replace every
/// flourish with a cross-fade and look like the animation was broken.
#[must_use]
pub fn detect() -> MotionPreference {
    MotionPreference::from_reduced(platform_prefers_reduced_motion().unwrap_or(false))
}

/// macOS exposes this through `NSWorkspace`. Both calls are safe bindings, so
/// reading it does not cost the crate its `forbid(unsafe_code)`.
///
/// Always answers, unlike the other platforms; the shared signature keeps the
/// "unknown" case available to them.
#[cfg(target_os = "macos")]
#[allow(clippy::unnecessary_wraps)]
fn platform_prefers_reduced_motion() -> Option<bool> {
    use objc2_app_kit::NSWorkspace;

    Some(NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion())
}

/// GNOME's `enable-animations` is the setting desktop toolkits map
/// `prefers-reduced-motion` onto. Absent `gsettings` — a non-GNOME desktop, or
/// none at all — the answer is unknown rather than false.
#[cfg(target_os = "linux")]
fn platform_prefers_reduced_motion() -> Option<bool> {
    let output = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "enable-animations"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    match String::from_utf8(output.stdout).ok()?.trim() {
        "false" => Some(true),
        "true" => Some(false),
        _ => None,
    }
}

/// Windows keeps this behind `SystemParametersInfo(SPI_GETCLIENTAREAANIMATION)`,
/// which has no safe binding, and the registry values that look equivalent
/// govern window animations rather than this setting. Rather than ship a guess,
/// Windows users get the menu toggle and `--reduce-motion`.
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn platform_prefers_reduced_motion() -> Option<bool> {
    None
}

#[cfg(test)]
mod tests {
    use super::{CALM_FADE_IN, MotionPreference, SETTLED_SECONDS, detect};

    #[test]
    fn detection_answers_without_panicking() {
        // Whatever this machine reports, asking must be safe and total.
        let _ = detect();
    }

    #[test]
    fn full_motion_is_the_default() {
        // Defaulting to reduced would silently flatten every flourish on a
        // machine that never asked for it.
        assert_eq!(MotionPreference::default(), MotionPreference::Full);
        assert!(!MotionPreference::default().is_reduced());
    }

    #[test]
    fn toggling_round_trips() {
        let full = MotionPreference::Full;
        assert_eq!(full.toggled(), MotionPreference::Reduced);
        assert_eq!(full.toggled().toggled(), full);
        assert!(full.toggled().is_reduced());
    }

    #[test]
    fn the_menu_label_shows_which_state_is_active() {
        assert_ne!(
            MotionPreference::Full.menu_label(),
            MotionPreference::Reduced.menu_label()
        );
    }

    #[test]
    fn the_settled_clock_is_past_every_effects_build_in() {
        // Frosted Glass has the slowest build-in. Its growth front advances at
        // 0.125 per second from a 0.035 head start, and the frost has covered
        // the screen once that front passes the largest distance-to-an-edge any
        // pixel has — 0.5, at the very centre. Holding the clock before that
        // point would freeze a half-frosted pane.
        const GROWTH_PER_SECOND: f32 = 0.125;
        const HEAD_START: f32 = 0.035;
        const FARTHEST_FROM_AN_EDGE: f32 = 0.5;

        assert!(
            SETTLED_SECONDS.mul_add(GROWTH_PER_SECOND, HEAD_START) > FARTHEST_FROM_AN_EDGE,
            "the frost has not reached the centre by {SETTLED_SECONDS}s"
        );
        assert!(CALM_FADE_IN.as_millis() > 0);
    }
}
