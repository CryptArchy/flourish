//! Compile-time validation of the WGSL shader catalog.
//!
//! Shaders are otherwise only parsed when a pipeline is built, which happens
//! after a window is on screen. A typo therefore surfaced as a live failure
//! mid-presentation rather than as a failing build. These tests parse and
//! fully validate every shader the binary ships.

use naga::{
    front::wgsl,
    valid::{Capabilities, ValidationFlags, Validator},
};

const FLOURISHES: &str = include_str!("../src/shaders/flourishes.wgsl");
const DOOM_FIRE: &str = include_str!("../src/shaders/doom_fire.wgsl");
const GRAVEL: &str = include_str!("../src/shaders/gravel.wgsl");

fn validate(name: &str, source: &str) -> naga::valid::ModuleInfo {
    let module = wgsl::parse_str(source).unwrap_or_else(|error| {
        panic!("{name} failed to parse:\n{}", error.emit_to_string(source))
    });

    Validator::new(ValidationFlags::all(), Capabilities::default())
        .validate(&module)
        .unwrap_or_else(|error| panic!("{name} failed validation:\n{error:?}"))
}

#[test]
fn every_shader_parses_and_validates() {
    validate("flourishes.wgsl", FLOURISHES);
    validate("doom_fire.wgsl", DOOM_FIRE);
    validate("gravel.wgsl", GRAVEL);
}

#[test]
fn the_catalog_shader_declares_an_arm_for_every_effect_it_must_draw() {
    // Guards the pairing between Flourish::shader_id and the switch in
    // flourishes.wgsl. A missing arm falls through to the transparent default,
    // which is a flourish that silently does nothing.
    for effect in flourish::Flourish::ALL.iter().copied() {
        if effect.has_dedicated_pipeline() {
            continue;
        }
        let constant = format!("u32 = {}u", effect.shader_id());
        assert!(
            FLOURISHES.contains(&constant),
            "flourishes.wgsl declares no constant for {} (shader id {})",
            effect.label(),
            effect.shader_id()
        );
    }
}

#[test]
fn the_catalog_shader_does_not_claim_the_dedicated_pipeline_ids() {
    // Gravel Fall draws through its own pipeline. If the shared shader ever
    // grew an arm for its id, the two would silently disagree about who draws.
    for effect in flourish::Flourish::ALL.iter().copied() {
        if !effect.has_dedicated_pipeline() {
            continue;
        }
        let constant = format!("u32 = {}u", effect.shader_id());
        assert!(
            !FLOURISHES.contains(&constant),
            "flourishes.wgsl claims id {} which belongs to {}'s own pipeline",
            effect.shader_id(),
            effect.label()
        );
    }
}

#[test]
fn the_doom_heat_ceiling_agrees_across_the_language_boundary() {
    // doom_fire.rs seeds cells with MAX_HEAT and the fragment shader divides by
    // DOOM_MAX_HEAT to normalize. Drift between them rescales the whole palette.
    assert!(
        FLOURISHES.contains("const DOOM_MAX_HEAT: f32 = 36.0;"),
        "flourishes.wgsl no longer normalizes heat by 36.0"
    );
}
