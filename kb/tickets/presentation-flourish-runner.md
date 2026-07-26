---
id: presentation-flourish-runner
type: ticket
project: flourish
tags: [desktop, presentations, shaders, cross-platform]
status: final
step: active
author: Christopher Andrews
created_date: 2026-07-18
source: file
---

# Presentation Flourish Runner

## Description

Build Flourish, a lightweight desktop utility that lets a presenter launch
short, full-screen visual effects from the macOS menu bar or the equivalent
Windows/Linux notification area.

## Context

The effects are presentation punctuation rather than screen filters or
wallpapers. A flourish owns the screen briefly, reacts to a dismissal signal,
plays a graceful exit when it has one, and then gets out of the way.

## Requirements

### Functional

- A persistent tray/menu-bar menu lists the installed flourishes.
- Selecting a flourish opens a full-screen effect over the current display.
- A click or key press asks the active flourish to exit gracefully.
- A second click or key press during graceful exit hides it immediately.
- Effects that naturally complete hide themselves.
- The first effect is a closed red theater curtain that subtly rustles while
  waiting, then draws from the center and reveals the real desktop underneath.
- The curtain disappears automatically once it is fully off-screen.
- The menu-bar icon reads as a small celebratory Flourish: monochrome in the
  macOS template style and colorful where tray platforms preserve color.
- The curtain uses dark oxblood velvet, antique-gold center and lower trim,
  and restrained stage-light texture.
- The initial catalog also includes Pond Ripples, Fire, Doom Fire, Blackout,
  Kaleidoscope, and Mosaic, each with an effect-specific graceful exit.
- Gravel Fall drops varied stones into a growing pile and releases the entire
  pile downward when dismissed.

### Non-functional

- Keep the idle process and packaged application small enough for a background
  utility.
- Share the renderer and effect lifecycle across macOS, Windows, and Linux.
- Do not require screen-recording permission just to reveal the underlying
  presentation.
- Keep effects local and usable without a network connection.
- Respect high-DPI displays and render at the display refresh cadence.

## Research hints

ShaderGlass, ShaderToy players, dynamic wallpaper tools, transparent overlay
windows, system tray frameworks, winit, wgpu, SDL3, Qt Quick, Tauri, Wayland,
macOS simple fullscreen, graceful effect lifecycle.

## Success Criteria

### Automated Verification

Ticked 2026-07-26 against commit `c6c4536`; 81 tests passing.

- [x] Effect lifecycle tests cover start, graceful exit, double-signal kill,
  and natural completion. — `src/timeline.rs`, one test per transition:
  `first_signal_starts_a_graceful_exit` (start and graceful exit),
  `second_signal_during_exit_hides_immediately` (double-signal kill),
  `graceful_exit_completes_at_its_deadline` and
  `natural_completion_returns_to_idle` (natural completion). Three more cover
  the hold ceiling added later: `an_unattended_hold_dismisses_itself`,
  `the_hold_ceiling_preserves_the_effect_clock`, and
  `a_manual_signal_still_wins_before_the_ceiling`.
- [x] Catalog tests cover stable menu labels, shader identifiers, and exit
  durations for every built-in flourish. — `src/lib.rs`,
  `catalog_metadata_is_unique_and_presentation_safe` asserts labels, slugs, and
  shader ids are each unique across `Flourish::ALL` and that no exit duration is
  zero; `every_slug_round_trips` and `slugs_are_command_line_safe` pin the
  command-line identifiers. Every one of these iterates `Flourish::ALL`, so
  adding a flourish extends the coverage rather than escaping it — and since the
  catalog is generated from a single macro table, a variant cannot be added
  without a label, slug, id, and duration.
- [x] The renderer and tray shell compile on the supported desktop targets. —
  CI builds and tests on macos-latest, windows-latest, and ubuntu-latest, plus a
  job pinned to the declared 1.88 MSRV and a release build that exercises the
  LTO profile the debug jobs never touch. All green:
  <https://github.com/CryptArchy/flourish/actions/runs/30224911169>
- [x] Shader validation and Rust static checks pass. — `tests/shaders.rs` parses
  and fully validates every WGSL file with naga at test time rather than
  discovering shader errors on stage, and cross-checks that the shared catalog
  declares an arm for each effect's id. Static checks are
  `cargo fmt --all -- --check` and `cargo clippy --all-targets --locked -D
  warnings` under `unsafe_code = "forbid"` with clippy pedantic enabled.

Two harnesses cover behaviour no unit test can reach, and both are run manually
rather than in CI because they need real hardware:
`cargo run --example placement` (the overlay lands on the display it was aimed
at) and `cargo run --release -- --benchmark` (frame-time against budget; results
in the `flourish-frame-time-budget` note).

### Manual Verification

- [ ] Selecting Curtain from the menu bar covers the active display.
- [ ] The closed curtain looks alive without becoming distracting.
- [ ] One click/key draws it open and reveals the actual screen.
- [ ] A second signal during opening removes it immediately.
- [ ] The app returns to an idle menu-bar-only state after completion.
- [ ] Every catalog item launches from the menu and exits without exposing a
  stuck overlay.
- [ ] Windows and Linux behavior is verified on real desktops before claiming
  those platforms shipped.
