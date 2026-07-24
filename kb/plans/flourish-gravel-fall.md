---
id: flourish-gravel-fall
type: plan
project: flourish
tags: [wgpu, particles, physics, hello-gravel, visual-design]
status: final
outcome: pending
author: Christopher Andrews
created_date: 2026-07-18
upstream: [presentation-flourish-runner, flourish-visual-feedback-tuning-round-2-review]
---

# Flourish Gravel Fall

## Overview

Fix Pond Ripples' right-heavy composition and add Gravel Fall: varied faceted
stones fall from above, collide with a growing pile, and drop offscreen when
the shared Flourish exit signal removes their floor.

## Current State Analysis

Pond Ripples generates all seven impact centers through a hash. The algorithm
has no spatial balancing, so a deterministic run can cluster most visible
wavefronts on one side. Existing full-screen effects render one procedural
triangle, while Doom Fire demonstrates that an effect may own state and a
specialized GPU pass. Gravel needs per-rock state and geometry rather than a
fragment pattern.

## Desired End State

- Pond Ripples uses intentionally distributed impact centers spanning left,
  middle, and right while preserving staggered timing.
- Gravel Fall appears as independently sized, rotated, faceted stones in
  limestone, river gray, sand, slate, and iron-ochre shades.
- Rocks accelerate from above, settle against a growing uneven pile, and stop
  spawning at a bounded count.
- On first signal, collision disappears and every settled or falling stone
  accelerates below the screen; second signal still hides immediately.
- The effect resets cleanly on every launch and remains lightweight at idle.

## What We're NOT Doing

- Full rigid-body rock-to-rock collision, rolling, sound, dust, or photorealistic
  texture assets.
- Using a generic confetti emitter with brown sprites.
- Capturing the presentation or adding Hello Gravel logos/text to the effect.
- Adding a third-party particle authoring system.

## Implementation Approach

Replace hashed ripple locations with a curated full-screen arrangement. Add a
CPU-side gravel simulation with a fixed rock budget and a one-dimensional pile
heightfield. Each rock falls under normalized gravity; collision samples the
highest point under its footprint, then updates the heightfield with a rounded
stone profile. Exit disables collision and increases downward gravity.

Render rocks through a dedicated instanced wgpu pipeline. Each instance carries
center, physical-aspect-correct size, rotation, shade, and shape seed. The
vertex shader expands a rough nine-sided triangle fan; the fragment shader adds
subtle top-left facet lighting.

Palette: Limestone `#B8AA91`, Pea Gravel `#8A7A65`, River Gray `#625F58`,
Sand `#D4C3A4`, Slate `#41413F`, and Iron Ochre `#A77B4F`. Signature: the pile
obeys weight until its entire supporting floor vanishes.

Risk gate: reversibility **low**, surface sensitivity **medium**, blast radius
**medium** (specialized render path), ambiguity **medium** (pile fidelity),
scale **medium**. Overall **medium**: proceed with a fixed particle ceiling,
shared hard-kill behavior, and manual visual acceptance.

## Phase 1: Ripple composition correction

### Changes Required

- Replace random ripple centers with a balanced curated distribution.
- Keep independent ages and aspect-correct circular wavefronts.

### Success Criteria

#### Automated Verification

- [x] The complete WGSL catalog initializes on Metal.

#### Manual Verification

- [ ] Clearly visible ripples originate on both screen halves.

## Phase 2: Gravel simulation and rendering

### Changes Required

- Add resettable falling-rock state, deterministic random generation, pile
  heightfield collision, and floor-release behavior.
- Add an instanced rough-polygon render pipeline and gravel palette.
- Register Gravel Fall in the menu, CLI smoke-test selector, and documentation.

### Success Criteria

#### Automated Verification

- [x] Unit tests cover reset state, varied spawn metadata, pile collision, and
  floor release.
- [x] Format, tests, strict Clippy, release build, and KB lint pass.
- [x] The real Gravel Fall pipeline initializes and renders on macOS/Metal.

#### Manual Verification

- [ ] Rocks visibly vary in size, silhouette, rotation, and natural gravel shade.
- [ ] Falling stones form an uneven pile instead of a flat row.
- [ ] First signal drops the entire pile below screen; second signal immediately
  clears it.

## Testing Strategy

Unit-test pure simulation invariants without a GPU, then run the full static and
release suite. Launch Pond Ripples and Gravel Fall through effect-specific
autostart paths on Metal. Human review remains required for distribution,
weight, pile believability, and color.

## References

- `src/shaders/flourishes.wgsl`
- `src/renderer.rs`
- `src/doom_fire.rs`
- `kb/tickets/presentation-flourish-runner.md`
