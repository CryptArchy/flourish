---
id: flourish-gravel-coverage-fire-edge-review
type: review
project: flourish
tags: [wgpu, shaders, particles, visual-design, feedback]
status: final
author: Christopher Andrews
created_date: 2026-07-20
upstream: [flourish-gravel-coverage-fire-edge]
---

# Flourish Gravel Coverage and Fire Edge Review

## Implementation Status

**Automated pass; manual art-direction approval pending.** Gravel now renders
1,656 retained stones across three independent z planes, with large and medium
rocks sharing the middle pile. Fire retains its accepted body and palette while
its previously coherent alpha contour is locally feathered and fragmented.
Accepted Frosted Glass, CRT Shutdown, and Geological Strata were not changed.

## Automated Verification Results

- `cargo fmt --check`: pass
- `cargo test`: pass, 17 tests
- `cargo clippy --all-targets -- -D warnings`: pass
- `cargo build --release`: pass
- `kb check`: pass, 0 errors and 0 warnings
- Revised Fire release launch on macOS/Metal: pass
- Revised 1,656-rock Gravel Fall release launch on macOS/Metal: pass

## Findings

### Matches plan

- Boulder count doubled from 9 to 18 and foreground small-rock count doubled
  from 696 to 1,392. The unchanged 36 large and 210 medium rocks produce 1,656
  total GPU instances.
- Large and medium rocks now share a single middle heightfield. Boulders retain
  their rear surface and small rocks retain their foreground surface.
- The pause between large and medium emission was removed and foreground spawn
  rate increased from 145 to 225 rocks per second.
- A 3.5%-from-top pile ceiling prevents a saturated plane from settling later
  rocks entirely outside the canvas. Full deterministic settlement retains all
  1,656 rocks in the instance buffer and every rock intersects the viewport.
- Fire's original coarse/fine tongue fields, ember-orange-yellow palette, heat
  core, sparks, and exit timing remain intact.
- Two animated high-frequency fields now perturb the signed flame edge, vary
  feather width, and break partial opacity into irregular fragments.
- Frosted Glass, CRT Shutdown, and Geological Strata remained unchanged after
  explicit user acceptance.

### Deviations

- The doubled budget reproduced offscreen settlement even with average-surface
  collision. The implementation adds a viewport-aware pile ceiling so excess
  stones pack along the upper edge instead of disappearing.
- Two edge-noise scales were used for Fire rather than the plan's initial single
  added scale. The second prevents the erosion mask itself from becoming a new
  coherent control contour.

### Potential issues

- Once a Gravel z plane reaches the upper 3.5% of the viewport, later rocks
  overlap along that boundary. This is intentional visual packing, not rigid
  body simulation.
- The 1,656-instance aggregate increases overdraw, especially when the pile is
  nearly opaque, though the optimized Metal launch remained stable.
- Release launches prove GPU pipeline creation and runtime stability, not the
  subjective amount of screen coverage or absence of a visible Fire contour.

## Manual Testing Required

- Let Gravel fully settle and confirm the new pile fills substantially more of
  the screen, the top packing does not read as a hard ceiling, and no arrival
  batch appears to vanish.
- Dismiss Gravel after full settlement and confirm all three z planes fall away;
  signal again during release to confirm immediate kill.
- Inspect Fire on the presentation display and confirm its top edge no longer
  forms a continuous control line while the body still reads as the preferred
  original Fire.
- Dismiss Fire normally and with a second signal to confirm lifecycle behavior.

## Recommendations

Treat these density and edge changes as the next visual-review candidate. If
Gravel still needs more cover, tune the per-plane ceiling/width distribution
before increasing the already dense foreground budget again. If Fire retains a
line, adjust edge fragmentation strength rather than its accepted base tongue
fields or color ramp.
