//! Shared, renderer-independent Flourish behavior.

mod timeline;

pub use timeline::{SignalResult, Timeline, TimelineUpdate};

use std::time::Duration;

/// The built-in presentation flourishes, in native-menu order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flourish {
    Curtain,
    ProjectorIris,
    GeologicalStrata,
    FrostedGlass,
    CrtShutdown,
    PondRipples,
    Fire,
    DoomFire,
    GravelFall,
    Blackout,
    Kaleidoscope,
    Mosaic,
}

impl Flourish {
    pub const ALL: [Self; 12] = [
        Self::Curtain,
        Self::ProjectorIris,
        Self::GeologicalStrata,
        Self::FrostedGlass,
        Self::CrtShutdown,
        Self::PondRipples,
        Self::Fire,
        Self::DoomFire,
        Self::GravelFall,
        Self::Blackout,
        Self::Kaleidoscope,
        Self::Mosaic,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Curtain => "Curtain",
            Self::ProjectorIris => "Projector Iris",
            Self::GeologicalStrata => "Geological Strata",
            Self::FrostedGlass => "Frosted Glass",
            Self::CrtShutdown => "CRT Shutdown",
            Self::PondRipples => "Pond Ripples",
            Self::Fire => "Fire",
            Self::DoomFire => "Doom Fire",
            Self::GravelFall => "Gravel Fall",
            Self::Blackout => "Blackout",
            Self::Kaleidoscope => "Kaleidoscope",
            Self::Mosaic => "Mosaic",
        }
    }

    /// Stable shader selector. Values are explicit to keep recorded visuals
    /// and future persisted preferences compatible as the catalog grows.
    #[must_use]
    pub const fn shader_id(self) -> f32 {
        match self {
            Self::Curtain => 0.0,
            Self::ProjectorIris => 8.0,
            Self::GeologicalStrata => 9.0,
            Self::FrostedGlass => 10.0,
            Self::CrtShutdown => 11.0,
            Self::PondRipples => 1.0,
            Self::Fire => 2.0,
            Self::DoomFire => 6.0,
            Self::GravelFall => 7.0,
            Self::Blackout => 3.0,
            Self::Kaleidoscope => 4.0,
            Self::Mosaic => 5.0,
        }
    }

    #[must_use]
    pub const fn exit_duration(self) -> Duration {
        match self {
            Self::Curtain | Self::DoomFire | Self::GravelFall | Self::ProjectorIris => {
                Duration::from_millis(1_800)
            }
            Self::GeologicalStrata => Duration::from_millis(1_900),
            Self::FrostedGlass => Duration::from_millis(1_700),
            Self::CrtShutdown => Duration::from_millis(1_300),
            Self::PondRipples => Duration::from_millis(1_400),
            Self::Fire => Duration::from_millis(1_600),
            Self::Blackout => Duration::from_millis(1_200),
            Self::Kaleidoscope | Self::Mosaic => Duration::from_millis(1_500),
        }
    }

    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "curtain" => Some(Self::Curtain),
            "projector-iris" => Some(Self::ProjectorIris),
            "geological-strata" => Some(Self::GeologicalStrata),
            "frosted-glass" => Some(Self::FrostedGlass),
            "crt-shutdown" => Some(Self::CrtShutdown),
            "pond-ripples" => Some(Self::PondRipples),
            "fire" => Some(Self::Fire),
            "doom-fire" => Some(Self::DoomFire),
            "gravel-fall" => Some(Self::GravelFall),
            "blackout" => Some(Self::Blackout),
            "kaleidoscope" => Some(Self::Kaleidoscope),
            "mosaic" => Some(Self::Mosaic),
            _ => None,
        }
    }
}

#[cfg(test)]
mod catalog_tests {
    use super::Flourish;
    use std::collections::HashSet;

    #[test]
    fn catalog_metadata_is_unique_and_presentation_safe() {
        let labels = Flourish::ALL
            .map(Flourish::label)
            .into_iter()
            .collect::<HashSet<_>>();
        let shader_ids = Flourish::ALL
            .map(Flourish::shader_id)
            .map(f32::to_bits)
            .into_iter()
            .collect::<HashSet<_>>();

        assert_eq!(labels.len(), Flourish::ALL.len());
        assert_eq!(shader_ids.len(), Flourish::ALL.len());
        assert!(
            Flourish::ALL
                .into_iter()
                .all(|effect| !effect.exit_duration().is_zero())
        );
    }

    #[test]
    fn catalog_order_starts_with_the_signature_effect() {
        assert_eq!(Flourish::ALL[0], Flourish::Curtain);
        assert_eq!(Flourish::Kaleidoscope.label(), "Kaleidoscope");
        assert_eq!(Flourish::ProjectorIris.label(), "Projector Iris");
        assert_eq!(Flourish::CrtShutdown.label(), "CRT Shutdown");
        assert_eq!(
            Flourish::from_slug("frosted-glass"),
            Some(Flourish::FrostedGlass)
        );
        assert_eq!(
            Flourish::from_slug("geological-strata"),
            Some(Flourish::GeologicalStrata)
        );
        assert_eq!(Flourish::from_slug("doom-fire"), Some(Flourish::DoomFire));
        assert_eq!(Flourish::from_slug("unknown"), None);
    }
}
