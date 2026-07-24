---
id: flourish-gravel-coverage-fire-edge
type: plan
project: flourish
tags: [wgpu, shaders, particles, visual-design, feedback]
status: final
outcome: partial
author: Christopher Andrews
created_date: 2026-07-20
upstream: [flourish-gravel-frost-tuning, flourish-gravel-frost-tuning-review]
---

# Flourish Gravel Coverage and Fire Edge

## Overview

Increase Gravel Fall from roughly half-screen accumulation toward a dense
avalanche, merge its large and medium z planes, and remove the newly observed
procedural contour at Fire's flame edge. Preserve accepted Frosted Glass, CRT
Shutdown, and Geological Strata unchanged.

## Current State Analysis

The supplied screenshot shows the 951-rock Gravel pass forming convincing,
compact aggregate across the width, but its settled top remains around the
middle of the screen with black gaps above. Current planes are boulder, large,
medium, and small. Counts are 9, 36, 210, and 696.

Fire's outer silhouette is a single `smoothstep` around one height field. Even
with two noise octaves, that produces a coherent red contour—the equivalent of
the control-line artifact removed from Frost.

## Desired End State

- Gravel has 18 boulders, 36 large rocks, 210 medium rocks, and 1,392 small
  rocks: 1,656 total.
- Large and medium sizes share one physical pile surface; boulders remain the
  background plane and small rocks the foreground plane.
- Foreground emission is accelerated so the doubled budget does not double the
  wait.
- Every settled rock remains represented and intersects the viewport.
- Fire preserves its original preferred broad tongues, palette, inner heat, and
  sparks while replacing the coherent top contour with locally eroded,
  multi-scale opacity.
- Frosted Glass, CRT Shutdown, and Geological Strata remain byte-for-byte
  unchanged after direct acceptance.

## What We're NOT Doing

- Changing rock silhouettes, palette, minimum sizes, release behavior, or the
  compact average-surface settlement fix.
- Collapsing all Gravel sizes into one pile plane.
- Replacing the preferred Fire with Doom Fire or a new simulation.
- Adding an outline-softening blur that would leave the underlying contour
  mathematically intact.

## Implementation Approach

Double boulder and small counts, derive the 1,656-instance GPU buffer from the
new total, map `Large` and `Medium` to the same pile index, and reduce the
heightfield array to three planes. Remove the pause between large and medium
emission and raise small-rock spawn rate. Extend count, merged-plane,
settlement, viewport, reset, and release invariants.

For Fire, keep the existing coarse and fine noise that define the preferred
tongues. Add two faster-moving breakup fields only near the outer flame, vary
the local feather width, and erode partial edge opacity with those fields. The
result should dissolve into small flame fragments instead of tracing one
continuous mathematical boundary.

Risk gate: reversibility **low**, surface sensitivity **medium**, blast radius
**low**, ambiguity **medium**, scale **medium**. Overall **medium**: proceed
under direct user feedback and preserve manual visual acceptance.

## Phase 1: Three-plane 1,656-rock Gravel

### Changes Required

- Double boulder and small counts.
- Share the middle heightfield between large and medium rocks.
- Accelerate small emission and keep all settled rocks onscreen.

### Success Criteria

#### Automated Verification

- [x] Tests prove exact counts, a 1,656 total, and three populated pile planes.
- [x] Large and medium layers resolve to the same pile index.
- [x] All settled rocks remain onscreen and in the instance set.
- [x] Reset and floor-release behavior remain green.

#### Manual Verification

- [ ] Final Gravel accumulation covers substantially more than half the display
  without returning to runaway towers.

## Phase 2: Broken Fire silhouette

### Changes Required

- Add high-frequency animated edge breakup.
- Vary feather width and partial-alpha erosion near the flame front.
- Preserve the accepted underlying Fire palette and motion.

### Success Criteria

#### Automated Verification

- [x] Complete shader catalog initializes on Metal.
- [x] Format, tests, strict Clippy, optimized build, and KB lint pass.

#### Manual Verification

- [ ] No single continuous control line traces Fire's upper edge.
- [ ] Fire still reads as the preferred original effect rather than Doom Fire.

## Testing Strategy

Run the deterministic simulation through full settlement at the larger budget,
asserting counts, shared middle surface, viewport intersection, and instance
retention. Run all static and optimized checks, then launch Gravel Fall and Fire
on Metal. Use the supplied screenshot as the coverage baseline and retain final
art-direction checks for the presentation display.

## References

- User screenshot `Screenshot 2026-07-20 at 13.21.49.png`
- `src/gravel.rs`
- `src/shaders/flourishes.wgsl`
- `kb/reviews/flourish-gravel-frost-tuning-review.md`
