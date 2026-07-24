---
id: flourish-iris-strata-and-gravel-layers-review
type: review
project: flourish
tags: [wgpu, shaders, particles, hello-gravel, visual-design]
status: final
author: Christopher Andrews
created_date: 2026-07-18
upstream: [flourish-iris-strata-and-gravel-layers]
---

# Flourish Iris, Strata, and Gravel Layers Review

## Implementation Status

**Partial pass pending visual confirmation.** The layered Gravel redesign,
Projector Iris, and Geological Strata are implemented, documented, registered,
and GPU-valid. Static and release verification pass. Screen capture was denied
by macOS, so final art-direction acceptance remains with the user on the actual
presentation display.

## Automated Verification Results

- `cargo fmt --all -- --check`: pass
- `cargo test --locked`: pass, 15 tests
- `cargo clippy --locked --all-targets -- -D warnings`: pass
- `cargo check --locked`: pass
- `cargo build --release --locked`: pass
- `kb check`: pass, 0 errors and 0 warnings
- Real Projector Iris launch on macOS/Metal: pass
- Real Geological Strata launch on macOS/Metal: pass
- Real layered Gravel Fall launch on macOS/Metal: pass

## Findings

### Matches plan

- Gravel now has four ordered bands: 6 boulders, 36 large, 210 medium, and 348
  small stones, exactly preserving the 600-instance ceiling.
- The smallest radius rises from `0.006` to `0.015`, while boulders span
  `0.120–0.220`; no budget is spent on visually insignificant flecks.
- Each band owns a distinct radius range, spawn rate, inter-layer pause,
  gravity response, and color dullness. Spawn order doubles as back-to-front
  draw order.
- The existing rounded heightfield, floor-release transition, palette, rough
  silhouettes, reset behavior, and hard-kill path remain intact.
- Projector Iris renders twelve curved overlapping blade sectors, per-blade
  gunmetal variation, brushed highlights, tungsten aperture light, and sparse
  dust. The aperture exceeds the aspect-correct corner radius at completion.
- Geological Strata renders nine warped material beds with grain, lamination,
  embedded pebble texture, and a crooked multi-frequency fault.
- Strata reconstructs left and right pieces in source space before shading, so
  color and alpha shear together and fully clear the screen.
- Projector Iris and Geological Strata have stable IDs, labels, slugs, exit
  durations, menu entries, documentation, and catalog test coverage.
- The six other approved concepts are preserved in the finalized
  `flourish-future-catalog` ticket.

### Deviations

- The requested 3–9 boulder range is represented by a fixed six-boulder pass.
  Determinism makes visual regression and pile tests repeatable while remaining
  squarely inside the requested count.
- Automated screenshot critique could not run because macOS denied
  `screencapture`; no screen-recording permission was added to Flourish.

### Potential issues

- Boulders participate in the same one-dimensional pile heightfield as every
  other layer. Their scale may expose overlaps that were invisible on small
  rocks.
- Iris blade seams are polar procedural approximations, not modeled rigid
  leaves; visual review should confirm the overlap cue is strong enough.
- Strata intentionally favors a graphic road-cut over geological accuracy.
- All visual tuning remains sensitive to projector black level and display
  aspect ratio.

## Manual Testing Required

- Let Gravel Fall run through all four arrivals. Confirm the boulders establish
  broad mass, each later layer visibly fills gaps, and the result no longer
  resembles the supplied gravel-shower screenshot.
- Dismiss Gravel once and confirm every layer drops; signal again and confirm
  immediate disappearance.
- Confirm Projector Iris reads as overlapping metal blades at hold and as a
  mechanical spiral—not merely a circle wipe—during exit.
- Confirm Geological Strata reads as sedimentary material, the fault edge is
  organic, and both halves fully reveal the presentation.
- Relaunch all three from the native menu and confirm clean resets.

## Recommendations

Tune only from direct screenshots or meeting-display feedback. If Gravel still
lacks mass, increase boulder width/aspect before adding more particles. If Iris
reads as a flat mask, deepen the curved overlap shadow rather than decorating
the lens. If Strata feels too orderly, perturb band boundaries before adding
more surface texture.
