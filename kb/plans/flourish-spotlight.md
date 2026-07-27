---
id: flourish-spotlight
type: plan
project: flourish
tags: [wgpu, shaders, visual-design, catalog, theatre]
status: final
outcome: shipped
author: Christopher Andrews
created_date: 2026-07-26
upstream: [flourish-future-catalog, flourish-marquee-bulbs-and-constellation]
---

# Flourish Spotlight

## Overview

Promote **Spotlight** out of the approved future catalog: a restrained searching
stage light that expands to give the screen back. Third of the four remaining
concepts to graduate, and the third theatre effect alongside Curtain and Marquee
Bulbs.

## Current State Analysis

Fourteen effects ship. The catalog already contains one iris-shaped reveal —
Projector Iris — so Spotlight has to earn its place against it rather than
repeat it. Iris is mechanical: hard-edged gunmetal blades, centred, rotating as
it spirals open. Everything about Spotlight should be the opposite: soft
penumbra, off-centre, moving, and warm.

The uniform set already carries everything needed. No renderer work.

## Desired End State

- Holds as a dark stage with one warm follow-spot pool and its visible beam,
  searching slowly. Restrained: a lazy drift, not a sweep.
- The exit expands the pool from **wherever the light happens to be standing**,
  not from the centre, and the screen returns behind the light as it goes, so
  what crosses the display is a bright ring running off the edges.
- Reduced motion holds a settled frame with the spot somewhere off-centre, beam
  visible, and cross-fades as every other effect does.

## What We're NOT Doing

- No second spot, no colour gels, no gobo patterns. "Restrained" is the
  requirement, and a second light would compete with the first.
- No centring the pool before the flood. Expanding from an off-centre position
  is the whole reason this reads differently from Projector Iris.
- No renderer state, new uniforms, or dedicated pipeline.

## Implementation Approach

`shader_id` 14, slug `spotlight`, 1,600 ms exit, placed third in the menu so the
theatre effects lead together.

**The stage.** Near-black with a faint cool cast and fine grain, so the warm
light has something to be warm against.

**The pool.** A slightly vertically squashed circle — a beam meeting a surface
at an angle — with a soft penumbra and a hotter core. Its centre drifts on a
Lissajous seeded per performance, with restrained amplitude.

*Revised during implementation:* the drift was to damp to zero as the exit
began, so the light settled while it flooded. Damping it toward its base
position would have pulled the pool to the screen centre, defeating the
off-centre reveal, and freezing it in place needs the exit's own duration inside
the shader — a second cross-language constant to keep in lockstep. Over a 1.6
second dismissal the drift moves the pool by a few hundredths of a screen, so
the machinery would have bought nothing observable. The drift simply continues.

**The beam.** The detail that separates a spotlight from a bright circle. The
lamp is a fixed apex above the top edge; the beam is the cone from that apex to
the moving pool, so the beam angle changes as the spot searches — which is what
a real follow-spot does. Width opens linearly from apex to pool, brightness
falls off along its length, and drifting value noise gives it haze and dust.

**The lamp.** A restrained arc flicker, a few percent, because a perfectly
steady stage light reads as a rendered circle.

**The exit, two overlapping stages.** The pool radius grows to cover the
farthest corner from wherever the spot stands, and a reveal radius chases it
from inside, turning the stage transparent behind the light.

*Revised during implementation:* the reveal was specified as concentric with the
pool, opening from its centre. Rendered, that is an eclipse — a crisp dark
ellipse punched out of a bright field, which reads as a defect rather than as
the screen returning. Three changes fixed it: the reveal now **trails the pool's
edge** by a gap that closes as the wave leaves the screen, so what crosses the
display is a bright ring; the gap closes early, so the frames where the opening
is still small pass quickly; and the boundary is perturbed by harmonics of the
bearing, so the dark is eaten away rather than cut. Sampled value noise was
tried for that rim first and left visible facets on a boundary this large.

No premultiplied divide is needed here, unlike Marquee Bulbs and Constellation.
The light lives *on* the stage, so it leaves with it: a pixel the reveal has
opened has nothing left to be lit, and alpha is simply the stage that remains.

Palette — Lamp Core `#FFF3DC`, Warm Throw `#FFD79A`, Haze `#FFCE94` at low
amplitude, Stage `#050507`.

Risk gate: reversibility **high** (one shader function plus a catalog row),
surface sensitivity **low**, blast radius **low**, ambiguity **low**, scale
**small**. Overall **low**: proceed, leaving visual acceptance manual.

## Phase 1: Spotlight

### Changes Required

- `flourish_catalog!` row: `Spotlight`, slug `spotlight`, id 14, 1,600 ms.
- `EFFECT_SPOTLIGHT` constant, the `spotlight` function, and a switch arm.
- README catalog row.

### Success Criteria

#### Automated Verification

- [x] `cargo test --locked --all-targets` passes, 81 tests, including catalog
  uniqueness and the shader-arm pairing.
- [x] `cargo clippy --all-targets --locked -- -D warnings` clean.
- [x] `cargo fmt --all -- --check` clean.
- [x] `--benchmark` keeps Spotlight out of the expensive band; it has no
  neighbourhood loop, so it should land near Curtain rather than near
  Constellation. Measured 0.43 ms at 5K against Curtain's 0.47 — the prediction
  held, and it is the cheapest full-screen effect in the catalog.

#### Manual Verification

- [ ] The beam is visible as a shaft, not merely a bright pool.
- [ ] The search reads as restrained rather than as a sweeping searchlight.
- [ ] The reveal is visibly off-centre and does not read as Projector Iris.
- [x] Alpha reaches zero: the final exit frames render as the stand-in desktop,
  unmodified.

## Phase 2: Record

### Changes Required

- Move Spotlight from the future catalog's remaining list into its promoted
  section, as CRT Shutdown and Frosted Glass were.
- Re-measure `kb/notes/flourish-frame-time-budget.md` in a single run.

### Success Criteria

#### Automated Verification

- [x] `kb check` clean.
- [x] Frame-time note re-measured; every pre-existing row reproduces to within a
  hundredth of a millisecond.

## Testing Strategy

Catalog and shader-validation tests cover registration generically. Visual
behaviour is checked by rendering the shared shader offscreen across the hold
and a sweep of exit progress, composited over a stand-in desktop so alpha is
observed rather than assumed. Frame time through the shipped `--benchmark`.

## References

- `src/lib.rs`
- `src/shaders/flourishes.wgsl`
- `kb/tickets/flourish-future-catalog.md`
- `kb/plans/flourish-marquee-bulbs-and-constellation.md`
