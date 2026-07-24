---
id: flourish-frosted-crt-depth-layers
type: plan
project: flourish
tags: [wgpu, shaders, particles, visual-design, macos]
status: final
outcome: partial
author: Christopher Andrews
created_date: 2026-07-20
upstream: [presentation-flourish-runner, flourish-future-catalog, flourish-iris-strata-and-gravel-layers]
---

# Flourish Frosted Glass, CRT, and Depth Layers

## Overview

Correct Gravel Fall's interpretation of depth layers, conceal Geological
Strata's dormant fault, and promote Frosted Glass and CRT Shutdown from the
approved future catalog into the built-in Flourish menu.

## Current State Analysis

Gravel has explicit size bands but all rocks collide against one shared
heightfield, so smaller stones accumulate on top of boulders instead of forming
independent foreground piles. Strata applies a hold-state fault gap and a broad
dark shadow, telegraphing its exit. The shared shader catalog and lifecycle can
support both new effects without new renderer state.

## Desired End State

- Gravel retains its four counts, size ranges, palette, timing, and draw order,
  but each band settles against its own independent pile surface.
- Geological Strata is continuous at hold; its crooked fault becomes visible
  only after graceful exit begins.
- Frosted Glass grows pale dendritic ice over a translucent blue-white pane and
  melts through several irregular warm fronts on dismissal.
- CRT Shutdown holds as a nearly black phosphor-green tube with scanlines,
  bloom, and restrained analog noise, then collapses vertically to a bright
  horizontal line, contracts to a dot, and blinks out.
- Both effects retain the shared first-signal graceful exit and second-signal
  immediate kill behavior.

## What We're NOT Doing

- Capturing or refracting the real presentation beneath Frosted Glass.
- Adding audio, raster textures, video assets, or a persistent post-processing
  filter.
- Reworking Gravel counts or visual palette in this pass.
- Implementing active-monitor targeting in this shader-focused plan; that work
  is captured separately as `flourish-active-monitor-targeting`.

## Implementation Approach

Replace Gravel's single heightfield with one fixed heightfield per
`GravelLayer`, selecting only the matching plane for collision and settlement.
Keep spawn order as painter's order, so boulders remain behind large, medium,
and small foreground piles.

Gate Strata's fault alpha and shadow entirely by exit progress. Add Frosted
Glass and CRT Shutdown as catalog WGSL functions with stable IDs, labels,
slugs, and exit durations. Frost uses layered crystalline direction fields,
cellular grain, and four aspect-correct melt centers. CRT uses a two-stage
shape envelope so opaque color and alpha collapse together.

Palette — Frost: Ice White `#EAF5F7`, Rime `#C8E1E7`, Glacier `#8DB7C3`, Deep
Glass `#436F7C`, Melt Light `#FFF4D7`. CRT: Tube Black `#020504`, Phosphor
Shadow `#143321`, Phosphor Green `#8CFFAC`, Hot Core `#EEFFF1`.

Risk gate: reversibility **low**, surface sensitivity **medium**, blast radius
**medium**, ambiguity **medium**, scale **medium**. Overall **medium**: proceed
under direct user approval, retain bounded shader work and deterministic
simulation tests, and leave visual acceptance manual.

## Phase 1: Independent Gravel depth planes

### Changes Required

- Allocate and reset one pile heightfield for each Gravel layer.
- Route collision and settlement exclusively to the rock's own layer.
- Extend tests to prove layers cannot raise one another's pile surface.

### Success Criteria

#### Automated Verification

- [x] Gravel simulation tests cover four independent populated heightfields.
- [x] Existing counts, minimum size, reset, and release tests remain green.

#### Manual Verification

- [ ] Boulders remain visually behind; each smaller layer forms its own pile in
  front rather than stacking on the previous size.

## Phase 2: Concealed Strata fault

### Changes Required

- Remove hold-state crack alpha and fault shadow.
- Ease both cues in only after exit begins.

### Success Criteria

#### Automated Verification

- [x] Complete shader catalog initializes on Metal.

#### Manual Verification

- [ ] The fault path is not obvious before dismissal.
- [ ] The split still reads clearly once movement begins.

## Phase 3: Frosted Glass and CRT Shutdown

### Changes Required

- Register stable metadata for both effects.
- Implement crystalline frost growth and multi-origin melt reveal.
- Implement a scanlined CRT hold and two-stage phosphor collapse.
- Update catalog documentation and promoted backlog state.

### Success Criteria

#### Automated Verification

- [x] Catalog metadata remains unique and includes both new slugs.
- [x] Both effects initialize and render on macOS/Metal.
- [x] Format, tests, strict Clippy, optimized build, and KB lint pass.

#### Manual Verification

- [ ] Frost reads as branching ice rather than cloudy noise.
- [ ] Melt fronts reveal the screen without a lingering color underlayer.
- [ ] CRT visibly collapses to a line, then a dot, and fully disappears.

## Testing Strategy

Unit-test Gravel plane isolation and catalog metadata. Run format, all tests,
strict Clippy, debug check, optimized build, and KB lint. Launch Gravel Fall,
Geological Strata, Frosted Glass, and CRT Shutdown through their autostart slugs
on Metal; preserve visual acceptance as a manual presentation-display check.

## References

- `src/gravel.rs`
- `src/shaders/flourishes.wgsl`
- `src/lib.rs`
- `src/renderer.rs`
- `kb/tickets/flourish-future-catalog.md`
- `kb/plans/flourish-iris-strata-and-gravel-layers.md`
