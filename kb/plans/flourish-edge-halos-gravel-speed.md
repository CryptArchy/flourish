---
id: flourish-edge-halos-gravel-speed
type: plan
project: flourish
tags: [wgpu, shaders, transparency, particles, visual-design, feedback]
status: final
outcome: partial
author: Christopher Andrews
created_date: 2026-07-20
upstream: [flourish-gravel-coverage-fire-edge, flourish-gravel-coverage-fire-edge-review]
---

# Flourish Edge Halos and Gravel Speed

## Overview

Remove the visible colored control bands at Fire and Frosted Glass leading
edges, and make Gravel Fall descend modestly faster without changing its
approved counts, layering, accumulation, palette, or dismissal.

## Current State Analysis

The supplied screenshot isolates Fire over both white and black backgrounds.
Its dark line is not a geometry seam: the outer signed-distance feather spans
1.8–5.2% of screen height and maps every low-heat boundary pixel to the same
dark ember color. Frost has the same class of artifact because its growth mask
crossfades over 16% of normalized screen space. Both create broad tinted bands
that are most obvious over light content.

Gravel's 1,656-rock pile is accepted, but hold gravity remains `0.82` and new
stones begin at only `0.035–0.145` normalized units per second.

## Desired End State

- Fire retains its broad tongues, edge breakup, palette, inner heat, sparks,
  and exit while its outer coverage feather is only a few physical pixels.
- Fire does not paint a uniform dark ember strip along the silhouette.
- Frost retains its opacity, crystal cells, blooms, and melt behavior while its
  growth frontier becomes a sharp, porous, noise-shaped crystalline boundary.
- Gravel rocks fall perceptibly but modestly faster; its final pile is
  unchanged.

## What We're NOT Doing

- Changing the window compositor or premultiplied-alpha contract; the screenshot
  shows valid transparency with effect-specific colored masks.
- Redesigning either accepted effect body or Frost's dismissal melt.
- Changing Gravel counts, spawn ordering, z planes, stacking, or release speed.

## Implementation Approach

Express Fire and Frost coverage in pixel-scaled feathers derived from render
height. Preserve their signed/noise fields, but sharpen low-alpha tails and
make the immediate Fire edge borrow irregular warm body heat so it cannot form
a continuous dark stroke. For Frost, perturb the growth distance with coarse
and fine fixed crystal noise, then make only the narrow frontier porous.

Raise Gravel hold gravity from `0.82` to approximately `1.08` and modestly lift
its initial velocity range. Do not change release gravity.

Risk gate: reversibility **low**, surface sensitivity **medium**, blast radius
**low**, ambiguity **low**, scale **low**. Overall **medium**: proceed under the
direct screenshot feedback, with real-display art direction still required.

## Phase 1: Pixel-scale Fire and Frost frontiers

### Changes Required

- Replace normalized broad edge feathers with resolution-aware antialiasing.
- Break Frost's leading front with multi-scale crystalline porosity.
- Prevent low-heat Fire boundary pixels from drawing a uniform ember outline.

### Success Criteria

#### Automated Verification

- [x] The complete shader catalog compiles and initializes on macOS/Metal.
- [x] Strict formatting, tests, and Clippy pass.

#### Manual Verification

- [ ] Fire has no obvious colored control line over white or black content.
- [ ] Frost grows without a broad tinted leading band.

## Phase 2: Faster Gravel descent

### Changes Required

- Increase hold-only gravity and initial fall velocity.
- Preserve settlement and release invariants.

### Success Criteria

#### Automated Verification

- [x] All 1,656 stones still settle onscreen and remain in the instance set.
- [x] Reset and floor-release tests remain green.

#### Manual Verification

- [ ] Gravel arrives a little faster without reading as rain or losing its
  sense of weight.

## Testing Strategy

Run the full deterministic Gravel suite, the complete Rust test and lint gates,
an optimized build, and direct release launches for Fire, Frosted Glass, and
Gravel Fall. Treat the screenshot as the edge-halo baseline and leave final
appearance checks to the actual presentation display.

## References

- User screenshot `Screenshot 2026-07-20 at 13.36.46.png`
- `src/shaders/flourishes.wgsl:187`
- `src/shaders/flourishes.wgsl:441`
- `src/gravel.rs:17`
- `src/gravel.rs:267`
- `kb/reviews/flourish-gravel-coverage-fire-edge-review.md`
