//! Shared, renderer-independent Flourish behavior.

pub mod display;
pub mod icon;
pub mod motion;
mod timeline;

pub use display::MonitorBounds;
pub use motion::MotionPreference;
pub use timeline::{SignalResult, Timeline, TimelineUpdate};

use std::time::Duration;

/// How long a flourish may hold the screen before it dismisses itself.
///
/// A flourish is a full-screen, always-on-top window that hides the cursor, so
/// a presenter whose pointer is on another display can be left with no obvious
/// way out. Every flourish therefore carries its own deadline; see
/// [`Timeline`] for the enforcement.
pub const DEFAULT_HOLD_LIMIT: Duration = Duration::from_secs(15);

/// Declares the flourish catalog exactly once.
///
/// Every per-effect fact lives in the table below, so adding a variant without
/// also giving it a slug, a label, a shader id, and an exit duration is a
/// compile error rather than an effect that silently never appears in the menu.
macro_rules! flourish_catalog {
    ($(
        $variant:ident {
            slug: $slug:literal,
            label: $label:literal,
            shader_id: $shader_id:literal,
            exit_ms: $exit_ms:literal,
        }
    ),+ $(,)?) => {
        /// The built-in presentation flourishes, in native-menu order.
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
        pub enum Flourish {
            $($variant),+
        }

        impl Flourish {
            /// Every flourish, in native-menu order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// Stable command-line and preference identifier.
            #[must_use]
            pub const fn slug(self) -> &'static str {
                match self {
                    $(Self::$variant => $slug),+
                }
            }

            /// Human-readable menu text.
            #[must_use]
            pub const fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label),+
                }
            }

            /// Stable shader selector. Values are explicit and never reused, so
            /// recorded visuals and persisted preferences survive catalog
            /// growth. The renderer dispatches on these exactly.
            #[must_use]
            pub const fn shader_id(self) -> u32 {
                match self {
                    $(Self::$variant => $shader_id),+
                }
            }

            /// How long the flourish's graceful exit animation runs.
            #[must_use]
            pub const fn exit_duration(self) -> Duration {
                match self {
                    $(Self::$variant => Duration::from_millis($exit_ms)),+
                }
            }
        }
    };
}

flourish_catalog! {
    Curtain          { slug: "curtain",           label: "Curtain",           shader_id: 0,  exit_ms: 1_800, },
    MarqueeBulbs     { slug: "marquee-bulbs",     label: "Marquee Bulbs",     shader_id: 12, exit_ms: 1_700, },
    Spotlight        { slug: "spotlight",         label: "Spotlight",         shader_id: 14, exit_ms: 1_600, },
    ProjectorIris    { slug: "projector-iris",    label: "Projector Iris",    shader_id: 8,  exit_ms: 1_800, },
    ElevatorDoors    { slug: "elevator-doors",    label: "Elevator Doors",    shader_id: 16, exit_ms: 1_600, },
    GeologicalStrata { slug: "geological-strata", label: "Geological Strata", shader_id: 9,  exit_ms: 1_900, },
    PaperTear        { slug: "paper-tear",        label: "Paper Tear",        shader_id: 15, exit_ms: 1_900, },
    FrostedGlass     { slug: "frosted-glass",     label: "Frosted Glass",     shader_id: 10, exit_ms: 1_700, },
    CrtShutdown      { slug: "crt-shutdown",      label: "CRT Shutdown",      shader_id: 11, exit_ms: 1_300, },
    PondRipples      { slug: "pond-ripples",      label: "Pond Ripples",      shader_id: 1,  exit_ms: 1_400, },
    Fire             { slug: "fire",              label: "Fire",              shader_id: 2,  exit_ms: 1_600, },
    DoomFire         { slug: "doom-fire",         label: "Doom Fire",         shader_id: 6,  exit_ms: 1_800, },
    GravelFall       { slug: "gravel-fall",       label: "Gravel Fall",       shader_id: 7,  exit_ms: 1_800, },
    Constellation    { slug: "constellation",     label: "Constellation",     shader_id: 13, exit_ms: 2_000, },
    Blackout         { slug: "blackout",          label: "Blackout",          shader_id: 3,  exit_ms: 1_200, },
    Kaleidoscope     { slug: "kaleidoscope",      label: "Kaleidoscope",      shader_id: 4,  exit_ms: 1_500, },
    Mosaic           { slug: "mosaic",            label: "Mosaic",            shader_id: 5,  exit_ms: 1_500, },
}

impl Flourish {
    /// Resolves a command-line slug against the catalog.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|flourish| flourish.slug() == slug)
    }

    /// How long this flourish may hold the screen before dismissing itself.
    #[must_use]
    pub const fn hold_limit(self) -> Duration {
        DEFAULT_HOLD_LIMIT
    }

    /// Whether the flourish draws through its own dedicated pipeline instead of
    /// the shared shader catalog.
    #[must_use]
    pub const fn has_dedicated_pipeline(self) -> bool {
        matches!(self, Self::GravelFall)
    }
}

#[cfg(test)]
mod catalog_tests {
    use super::Flourish;
    use std::collections::HashSet;

    #[test]
    fn catalog_metadata_is_unique_and_presentation_safe() {
        let labels = Flourish::ALL
            .iter()
            .map(|effect| effect.label())
            .collect::<HashSet<_>>();
        let slugs = Flourish::ALL
            .iter()
            .map(|effect| effect.slug())
            .collect::<HashSet<_>>();
        let shader_ids = Flourish::ALL
            .iter()
            .map(|effect| effect.shader_id())
            .collect::<HashSet<_>>();

        assert_eq!(labels.len(), Flourish::ALL.len());
        assert_eq!(slugs.len(), Flourish::ALL.len());
        assert_eq!(shader_ids.len(), Flourish::ALL.len());
        assert!(
            Flourish::ALL
                .iter()
                .all(|effect| !effect.exit_duration().is_zero())
        );
    }

    #[test]
    fn every_flourish_holds_for_a_bounded_time() {
        // A flourish that never yields the screen back can strand a presenter
        // whose pointer is on another display.
        assert!(
            Flourish::ALL
                .iter()
                .all(|effect| !effect.hold_limit().is_zero())
        );
        assert!(
            Flourish::ALL
                .iter()
                .all(|effect| effect.hold_limit() > effect.exit_duration())
        );
    }

    #[test]
    fn every_slug_round_trips() {
        for effect in Flourish::ALL.iter().copied() {
            assert_eq!(Flourish::from_slug(effect.slug()), Some(effect));
        }
        assert_eq!(Flourish::from_slug("unknown"), None);
        assert_eq!(Flourish::from_slug(""), None);
    }

    #[test]
    fn slugs_are_command_line_safe() {
        for effect in Flourish::ALL.iter().copied() {
            let slug = effect.slug();
            assert!(!slug.is_empty());
            assert!(
                slug.bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte == b'-'),
                "slug {slug:?} is not lowercase-and-dashes"
            );
        }
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
        assert_eq!(Flourish::from_slug("doom-fire"), Some(Flourish::DoomFire));
    }

    #[test]
    fn only_gravel_bypasses_the_shared_shader_catalog() {
        // The shared fragment shader dispatches on shader_id; an effect that
        // claims an id but renders elsewhere must be deliberate, not accidental.
        let dedicated = Flourish::ALL
            .iter()
            .copied()
            .filter(|effect| effect.has_dedicated_pipeline())
            .collect::<Vec<_>>();

        assert_eq!(dedicated, vec![Flourish::GravelFall]);
    }
}
