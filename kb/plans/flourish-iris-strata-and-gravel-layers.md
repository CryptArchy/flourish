---
id: flourish-iris-strata-and-gravel-layers
type: plan
project: flourish
tags: [wgpu, shaders, particles, hello-gravel, visual-design]
status: final
outcome: partial
author: Christopher Andrews
created_date: 2026-07-18
upstream: [presentation-flourish-runner, flourish-gravel-fall]
---

# Flourish Iris, Strata, and Gravel Layers

## Overview

Recompose Gravel Fall as four deliberate depth and size layers, then add the
approved Projector Iris and Geological Strata effects to the built-in menu.
Preserve the other approved concepts as a separate catalog ticket.

## Current State Analysis

The first Gravel Fall pass uses a continuous radius range from `0.006` to
`0.023` screen height. The supplied screenshot shows hundreds of similarly
small, evenly salient stones with large gaps, reading as a gravel shower rather
than accumulated mass. The shared fullscreen shader already supports catalog
effects with aspect-correct coordinates and transparent graceful exits.

## Desired End State

- Gravel arrives back-to-front: 3–9 dull background boulders, a few dozen large
  rocks, about two hundred medium rocks, then the remaining 600-rock budget in
  small rocks no smaller than the old medium range.
- Earlier, larger layers establish broad coverage; later layers fill crevices
  without visually competing at the same scale.
- Projector Iris holds as nearly closed soot-black overlapping metal blades with
  tungsten edge leakage and lens dust, then mechanically spirals open.
- Geological Strata holds as a textured road-cut of limestone, ochre, clay,
  shale, and charcoal bands; dismissal opens a crooked fault and shears the two
  land masses away.
- Both new effects remain transparent overlays, respect shared graceful and
  immediate exit semantics, and appear in the menu and CLI selector.

## What We're NOT Doing

- Adding a general rigid-body engine, texture assets, sound, logos, or text.
- Making Gravel Fall photorealistic or shrinking stones below the prior medium
  size merely to consume the particle budget.
- Treating Projector Iris as only a plain circular wipe; blade structure must
  remain visible.
- Adding fossils, labels, or geological diagrams to Strata.

## Implementation Approach

Replace Gravel Fall's continuous spawn distribution with explicit layer specs
that own count, radius range, gravity response, color dullness, and timing.
Keep the proven heightfield collision and instanced polygon pipeline, drawing
instances in spawn order so later smaller stones sit visually above earlier
boulders.

Add both new effects to the shared WGSL catalog. Projector Iris uses polar blade
sectors, curved overlap seams, anisotropic metal shading, and an expanding
aspect-correct opening. Geological Strata reconstructs each displaced half in
source space around a noisy fault, ensuring the rendered color and alpha move
together during exit.

Palette — Iris: Carbon `#070808`, Gunmetal `#171A1B`, Worn Steel `#303333`,
Tungsten `#D59A45`. Strata: Limestone `#C0AF8E`, Iron Ochre `#A66D35`, Clay
`#7B4938`, Shale `#494844`, Charcoal `#262827`.

Risk gate: reversibility **low**, surface sensitivity **medium**, blast radius
**medium**, ambiguity **medium**, scale **medium**. Overall **medium**: proceed
under the user's approved art direction, preserve hard-kill behavior, and leave
visual acceptance manual.

## Phase 1: Layered gravel mass

### Changes Required

- Add explicit boulder, large, medium, and small spawn bands and minimum sizes.
- Preserve pile collision, floor release, palette variation, and reset.
- Add tests for layer counts, monotonic size bands, minimum size, and ordering.

### Success Criteria

#### Automated Verification

- [x] Simulation tests prove all four layers and the 600-rock budget.
- [x] No generated rock is smaller than the old medium-size floor.

#### Manual Verification

- [ ] Background boulders establish broad coverage before smaller layers arrive.
- [ ] The result reads as accumulated rock mass rather than confetti or rain.

## Phase 2: Projector Iris

### Changes Required

- Add catalog metadata and a polar overlapping-blade shader.
- Open the physical aperture on graceful exit and reveal the real screen.

### Success Criteria

#### Automated Verification

- [x] Catalog tests include stable Projector Iris metadata.
- [x] The shader initializes and renders on macOS/Metal.

#### Manual Verification

- [ ] The closed state visibly reads as overlapping mechanical blades.
- [ ] The opening feels like an iris mechanism, not a generic circle wipe.

## Phase 3: Geological Strata

### Changes Required

- Add catalog metadata and a warped, granular sediment-band shader.
- Split around a crooked fault and move both opaque color and alpha together.

### Success Criteria

#### Automated Verification

- [x] Catalog tests include stable Geological Strata metadata.
- [x] The shader initializes and renders on macOS/Metal.

#### Manual Verification

- [ ] Bands read as a geological road cut with distinct material character.
- [ ] The fault opens organically and both land masses clear the screen.

## Phase 4: Integration and verification

### Success Criteria

#### Automated Verification

- [x] Format, unit tests, strict Clippy, release build, and KB lint pass.

#### Manual Verification

- [ ] All three effects launch from the native menu and honor first/second
  signal behavior.

## Testing Strategy

Unit-test the deterministic Gravel simulation and catalog metadata. Run full
static and optimized builds, then launch each effect through its autostart slug
on Metal. Compare Gravel against the supplied screenshot, specifically checking
coverage, hierarchy, and minimum useful rock size.

## References

- User screenshot `Screenshot 2026-07-18 at 15.43.06.png`
- `src/gravel.rs`
- `src/shaders/gravel.wgsl`
- `src/shaders/flourishes.wgsl`
- `src/lib.rs`
- `kb/plans/flourish-gravel-fall.md`
