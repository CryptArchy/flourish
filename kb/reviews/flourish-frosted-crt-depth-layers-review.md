---
id: flourish-frosted-crt-depth-layers-review
type: review
project: flourish
tags: [wgpu, shaders, particles, visual-design, macos]
status: final
author: Christopher Andrews
created_date: 2026-07-20
upstream: [flourish-frosted-crt-depth-layers]
---

# Flourish Frosted Glass, CRT, and Depth Layers Review

## Implementation Status

**Partial pass pending visual confirmation.** Gravel now uses independent pile
surfaces for all four depth planes, Geological Strata no longer telegraphs its
fault while holding, and Frosted Glass plus CRT Shutdown are implemented and
registered. Automated checks, optimized compilation, and live Metal
initialization pass. Graceful-exit art direction remains a manual check.

## Automated Verification Results

- `cargo fmt --all -- --check`: pass
- `cargo test --locked`: pass, 16 tests
- `cargo clippy --locked --all-targets -- -D warnings`: pass
- `cargo check --locked`: pass
- `cargo build --release --locked`: pass
- `kb check`: pass, 0 errors and 0 warnings
- Real Frosted Glass launch on macOS/Metal: pass
- Real CRT Shutdown launch on macOS/Metal: pass
- Real corrected Geological Strata launch on macOS/Metal: pass
- Real independent-plane Gravel Fall launch on macOS/Metal: pass

## Findings

### Matches plan

- `GravelSimulation` owns four 320-bin heightfields indexed by `GravelLayer`.
  Collision and settlement read and write only the matching surface.
- Spawn order still renders boulders first and progressively smaller layers
  afterward, providing explicit painter's-order depth.
- A regression test settles the boulder plane and proves the not-yet-spawned
  small plane remains at floor height. Existing count, size, reset, and release
  tests remain intact.
- Geological Strata multiplies both crack alpha and the broad fault shadow by a
  dismissal-only reveal. At zero exit progress, the material is continuous.
- Frosted Glass combines three directional branching ridge fields, fine ice,
  edge-driven creep, and four irregular aspect-correct melt fronts. Its final
  alpha is premultiplied and explicitly reaches zero.
- CRT Shutdown uses a full-screen dark tube hold, scanlines, analog snow, and a
  two-stage envelope: vertical collapse to a phosphor line, horizontal
  contraction to a dot, then final blink.
- Both new effects have stable enum variants, IDs, labels, slugs, durations,
  menu entries, CLI autostart support, README rows, and catalog tests.
- Frosted Glass and CRT Shutdown moved from the future catalog's remaining list
  into its promoted section.
- Active-monitor targeting is captured separately in the finalized
  `flourish-active-monitor-targeting` ticket.

### Deviations

- None material. Frost simulates a translucent pane rather than real optical
  refraction because Flourish intentionally does not capture the presentation.
- The project had moved from `/Users/candrews/Documents/flourish` to
  `/Users/candrews/Code/flourish`; implementation continued in the live project
  rather than creating a duplicate workspace.

### Potential issues

- Independent Gravel planes deliberately permit foreground rocks to visually
  overlap background boulders. This creates z-depth but is not rigid-body
  interpenetration.
- Frost's procedural branch density may read differently on low-contrast
  projectors, especially over bright slides.
- CRT's 1.3-second exit is intentionally brisk; visual review may prefer a
  slightly longer horizontal-dot stage.
- Live launches validate Metal pipelines and hold states, not the full
  click-driven exit choreography.

## Manual Testing Required

- Let Gravel complete all four spawn bands and confirm each size forms a pile
  on the floor in its own visual plane rather than stacking atop larger rocks.
- Inspect Strata before dismissal and confirm the future fault path is no longer
  apparent; click once and confirm the split emerges naturally.
- Let Frost grow, then dismiss once. Confirm multiple irregular melt holes open,
  crystalline veins remain recognizable, and no colored underlayer lingers.
- Dismiss CRT once and confirm the tube collapses to a bright horizontal line,
  contracts to a dot, and fully blinks out. Double-signal during either exit to
  confirm immediate disappearance.
- Relaunch all four effects from the native menu and confirm clean reset.

## Recommendations

Use direct presentation-display feedback for the next tuning pass. If Gravel
still feels too tall, lower only the per-plane rock counts or collision profile;
do not merge the heightfields again. Tune Frost first through alpha and ridge
contrast, and CRT first through phase timing, preserving their signatures.
