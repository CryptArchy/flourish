---
id: flourish-gravel-frost-tuning
type: plan
project: flourish
tags: [wgpu, shaders, particles, visual-design, feedback]
status: final
outcome: partial
author: Christopher Andrews
created_date: 2026-07-20
upstream: [flourish-frosted-crt-depth-layers, flourish-frosted-crt-depth-layers-review]
---

# Flourish Gravel and Frost Tuning

## Overview

Apply direct visual feedback to Gravel Fall and Frosted Glass while preserving
the now-accepted CRT Shutdown and Geological Strata effects unchanged.

## Current State Analysis

Gravel retains every settled rock before dismissal, but later opaque depth
planes naturally occlude portions of earlier planes. The current budget is 600:
6 boulders, 36 large, 210 medium, and 348 small. The user wants more visible
mass: about 50% more boulders and twice as many smallest rocks.

Frosted Glass builds crystal ridges from three periodic directional sine fields.
Their shared frequencies create visible control/grid lines. Hold opacity ranges
from roughly 56% to 90%, lower than the desired frosted-pane coverage.

## Desired End State

- Gravel has 9 boulders, 36 large, 210 medium, and 696 small rocks: 951 total.
- All pre-dismissal rocks remain in the instance set; apparent disappearance is
  limited to intentional foreground occlusion, not culling or offscreen piles.
- Frost uses irregular, domain-warped cellular boundaries and local crystal
  blooms with no global periodic lattice.
- Frost hold opacity rises to approximately 78–98% while melt fronts still
  clear color and alpha together.
- CRT Shutdown and Geological Strata are unchanged and recorded as accepted by
  direct user feedback.

## What We're NOT Doing

- Making cross-plane rocks collide; independent z planes remain intentional.
- Making foreground rocks translucent merely to expose every background stone.
- Reintroducing rocks smaller than the approved minimum.
- Blurring or layering noise over the periodic Frost pattern; the pattern
  generator itself is replaced.

## Implementation Approach

Raise fixed layer counts and derive `MAX_ROCKS` from their sum so CPU and GPU
budgets cannot drift. Extend simulation duration in tests, then assert all 951
rocks settle within the viewport and remain represented before release.

Replace `frost_ridge` with a jittered nearest/second-nearest cellular edge field
at two scales, domain-warped by low-frequency value noise. Add several curved
local blooms for dendritic character without a screen-wide axis. Raise the base
pane alpha while preserving premultiplied compositing and final clear.

Risk gate: reversibility **low**, surface sensitivity **medium**, blast radius
**low**, ambiguity **medium**, scale **low**. Overall **medium**: proceed under
direct visual feedback and retain manual art-direction acceptance.

## Phase 1: Gravel density and retention

### Changes Required

- Increase boulders from 6 to 9 and small rocks from 348 to 696.
- Derive the 951-instance ceiling from all four layer constants.
- Prove every rock settles onscreen and remains rendered before dismissal.

### Success Criteria

#### Automated Verification

- [x] Tests assert exact counts, 951 total rocks, four pile planes, and no
  pre-dismissal instance loss.
- [x] Existing reset and floor-release behavior remains green.

#### Manual Verification

- [ ] Gravel feels substantially denser without looking like a single shared
  stack.

## Phase 2: Organic opaque Frost

### Changes Required

- Remove periodic directional ridge fields.
- Add warped cellular ice boundaries and irregular local blooms.
- Raise hold opacity while retaining multi-origin melt and final transparency.

### Success Criteria

#### Automated Verification

- [x] Shader catalog initializes on Metal.
- [x] Format, tests, strict Clippy, optimized build, and KB lint pass.

#### Manual Verification

- [ ] No visible control lines, rectangular grid, or global repeating axis.
- [ ] Frost is more opaque but still reads as ice rather than white paint.
- [ ] Dismissal leaves no lingering colored underlayer.

## Testing Strategy

Run deterministic Gravel simulation through full settlement and assert instance
retention. Run all Rust checks and optimized compilation, then launch Gravel
Fall and Frosted Glass on Metal. Leave density, organic appearance, and opacity
as explicit manual checks on the presentation display.

## References

- `src/gravel.rs`
- `src/shaders/flourishes.wgsl`
- `kb/reviews/flourish-frosted-crt-depth-layers-review.md`
