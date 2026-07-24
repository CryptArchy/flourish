---
id: flourish-gravel-fall-review
type: review
project: flourish
tags: [wgpu, particles, physics, hello-gravel, visual-design]
status: final
author: Christopher Andrews
created_date: 2026-07-18
upstream: [flourish-gravel-fall]
---

# Flourish Gravel Fall Review

## Implementation Status

**Partial pass pending visual confirmation.** Pond Ripples now uses a balanced
set of impact origins, and Gravel Fall is fully registered with a bounded,
resettable rock simulation and its own instanced render pipeline. Automated
verification and real Metal launches pass; composition, weight, and pile
believability remain manual art-direction checks.

## Automated Verification Results

- `cargo fmt --all -- --check`: pass
- `cargo test --locked`: pass, 14 tests
- `cargo clippy --locked --all-targets -- -D warnings`: pass
- `cargo build --release --locked`: pass
- `kb check`: pass, 0 errors and 0 warnings
- Real Gravel Fall render launch on macOS/Metal: pass
- Real Pond Ripples render launch on macOS/Metal: pass

## Findings

### Matches plan

- Pond Ripples replaces unbounded hashed positions with seven curated impact
  centers spanning the left, middle, and right portions of the screen.
- Gravel Fall emits up to 600 independently sized, rotated, and colored rocks
  through a deterministic CPU simulation.
- Settled rocks update a 320-bin rounded heightfield, building an uneven pile
  without a heavyweight rigid-body dependency.
- The dedicated wgpu pipeline expands every instance into a rough nine-sided
  silhouette and adds restrained facet lighting and grain.
- Starting graceful exit stops spawning, releases settled rocks, disables pile
  collision, and increases gravity so the entire mass falls away.
- Relaunch resets the simulation, pile, random stream, and instance count.
- Existing second-signal hard kill remains owned by the shared timeline.

### Deviations

- None material. The implementation uses procedural facets rather than texture
  assets, as planned.

### Potential issues

- The one-dimensional heightfield prioritizes a convincing aggregate silhouette
  over individual rock rolling, so a close inspection may reveal occasional
  overlap.
- Palette values are intentionally darkened in the renderer; a projector may
  need a small brightness lift compared with the built-in display.
- Curated ripple positions guarantee spatial coverage, but their staggered ages
  can still make one half momentarily more active than the other.

## Manual Testing Required

- Confirm visible Pond Ripple origins appear on both halves of the presentation
  display over several seconds.
- Confirm Gravel Fall reads as stone rather than confetti: varied scale,
  irregular facets, earthy shade, and weight.
- Let the pile grow and confirm its top edge remains uneven rather than forming
  a straight shelf.
- Dismiss once and confirm the entire pile drops below the screen; dismiss twice
  during that fall and confirm the overlay vanishes immediately.
- Relaunch Gravel Fall and confirm it starts from an empty floor.

## Recommendations

Keep the plan outcome pending until the two effects are seen on the actual
meeting display. If Gravel Fall needs more physical character, add a restrained
dust puff on impact before considering heavier rigid-body simulation.
