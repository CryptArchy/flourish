//! Command-line parsing.
//!
//! Deliberately hand-rolled: the surface is three flags, and a dependency-free
//! parser keeps a presentation tool's startup path trivial to audit.

use flourish::{Flourish, MotionPreference};

/// What the command line asked the program to do.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Invocation {
    /// Run normally, optionally playing a flourish immediately.
    Run {
        autostart: Option<Flourish>,
        /// `None` means the command line did not say, so ask the system.
        motion: Option<MotionPreference>,
    },
    /// Print the display layout and pointer resolution, then exit. Needs an
    /// event loop to enumerate monitors, so it cannot be answered here.
    DescribeDisplays,
    /// Print text and exit successfully.
    PrintAndExit(String),
    /// Print an error and exit unsuccessfully.
    Fail(String),
}

const USAGE: &str = "\
Flourish — theatrical punctuation for presentations.

USAGE:
    flourish [OPTIONS]

OPTIONS:
    --autostart[=<FLOURISH>]  Play a flourish immediately at launch.
                              Defaults to the signature Curtain.
    --list                    List every flourish and its slug.
    --displays                Show the display layout, where the pointer is,
                              and which display a flourish would target.
    --reduce-motion           Hold each flourish still and cross-fade instead
                              of animating. Overrides the system setting.
    --full-motion             Animate even if the system asks for reduced
                              motion.
    -h, --help                Print this help.
    -V, --version             Print version information.

Without either motion flag, Flourish follows the system's reduce-motion
setting, and the menu can toggle it at any time.

Once running, Flourish lives in the menu bar. Choose an effect from its menu,
then click or press any key to begin its exit; signal again during the exit to
remove it at once. An unattended flourish dismisses itself.";

/// Parses arguments as they arrive from the OS, minus the program name.
pub fn parse<I, S>(arguments: I) -> Invocation
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut autostart = None;
    let mut motion = None;

    for argument in arguments {
        let argument = argument.as_ref();
        match argument {
            "--reduce-motion" => motion = Some(MotionPreference::Reduced),
            "--full-motion" => motion = Some(MotionPreference::Full),
            "-h" | "--help" => return Invocation::PrintAndExit(USAGE.to_owned()),
            "-V" | "--version" => {
                return Invocation::PrintAndExit(format!(
                    "{} {}",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION")
                ));
            }
            "--list" => return Invocation::PrintAndExit(catalog_listing()),
            "--displays" => return Invocation::DescribeDisplays,
            "--autostart" => autostart = Some(Flourish::Curtain),
            _ => {
                let Some(slug) = argument.strip_prefix("--autostart=") else {
                    return Invocation::Fail(format!(
                        "unrecognized argument {argument:?}\n\n\
                         Run `flourish --help` for usage."
                    ));
                };
                // An unknown slug used to be silently ignored, leaving the app
                // running with no flourish and no explanation.
                let Some(effect) = Flourish::from_slug(slug) else {
                    return Invocation::Fail(format!(
                        "unknown flourish {slug:?}\n\n{}",
                        catalog_listing()
                    ));
                };
                autostart = Some(effect);
            }
        }
    }

    Invocation::Run { autostart, motion }
}

fn catalog_listing() -> String {
    use std::fmt::Write;

    let width = Flourish::ALL
        .iter()
        .map(|effect| effect.slug().len())
        .max()
        .unwrap_or(0);
    let mut listing = String::from("Available flourishes:\n");
    for effect in Flourish::ALL.iter().copied() {
        // Writing into a String is infallible.
        let _ = writeln!(listing, "    {:width$}  {}", effect.slug(), effect.label());
    }
    listing
}

#[cfg(test)]
mod tests {
    use super::{Invocation, parse};
    use flourish::Flourish;

    #[test]
    fn no_arguments_runs_without_autostart() {
        assert_eq!(
            parse(Vec::<String>::new()),
            Invocation::Run {
                autostart: None,
                motion: None
            }
        );
    }

    #[test]
    fn bare_autostart_picks_the_signature_effect() {
        assert_eq!(
            parse(["--autostart"]),
            Invocation::Run {
                autostart: Some(Flourish::Curtain),
                motion: None
            }
        );
    }

    #[test]
    fn every_slug_is_accepted_as_an_autostart_target() {
        for effect in Flourish::ALL.iter().copied() {
            assert_eq!(
                parse([format!("--autostart={}", effect.slug())]),
                Invocation::Run {
                    autostart: Some(effect),
                    motion: None
                },
                "slug {} did not round-trip through the CLI",
                effect.slug()
            );
        }
    }

    #[test]
    fn an_unknown_slug_fails_loudly_instead_of_starting_nothing() {
        let Invocation::Fail(message) = parse(["--autostart=nope"]) else {
            panic!("an unknown flourish must not launch a silent, effectless app");
        };
        assert!(message.contains("nope"));
        // The error should teach the correct spelling, not just reject.
        assert!(message.contains("curtain"));
    }

    #[test]
    fn an_unknown_flag_fails() {
        let Invocation::Fail(message) = parse(["--wat"]) else {
            panic!("unknown flags must be rejected");
        };
        assert!(message.contains("--wat"));
    }

    #[test]
    fn help_and_version_short_circuit() {
        assert!(matches!(parse(["--help"]), Invocation::PrintAndExit(_)));
        assert!(matches!(parse(["-h"]), Invocation::PrintAndExit(_)));
        assert!(matches!(parse(["-V"]), Invocation::PrintAndExit(_)));

        let Invocation::PrintAndExit(version) = parse(["--version"]) else {
            panic!("--version must print and exit");
        };
        assert!(version.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn list_names_every_flourish() {
        let Invocation::PrintAndExit(listing) = parse(["--list"]) else {
            panic!("--list must print and exit");
        };
        for effect in Flourish::ALL.iter().copied() {
            assert!(listing.contains(effect.slug()), "{} missing", effect.slug());
            assert!(
                listing.contains(effect.label()),
                "{} missing",
                effect.label()
            );
        }
    }

    #[test]
    fn help_wins_over_a_later_bad_argument() {
        assert!(matches!(
            parse(["--help", "--autostart=nope"]),
            Invocation::PrintAndExit(_)
        ));
    }
}
