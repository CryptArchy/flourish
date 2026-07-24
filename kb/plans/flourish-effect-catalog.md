---
id: flourish-effect-catalog
type: plan
project: flourish
tags: [rust, shaders, menu-bar, visual-design]
status: final
outcome: pending
author: Christopher Andrews
created_date: 2026-07-18
upstream: [presentation-flourish-runner, flourish-v0-curtain-review]
---

# Flourish Effect Catalog

## Overview

Turn the accepted Curtain vertical slice into a small, coherent catalog. Give
the utility a recognizable celebratory icon, art-direct Curtain as a richer
theater scene, and add five new procedural GPU flourishes without weakening the
shared dismissal safety contract.

## Current State Analysis

The application has one hard-coded Curtain menu item and one renderer pipeline.
The renderer already supplies resolution, elapsed time, exit progress, and
compositor alpha convention to a full-screen WGSL shader. The renderer-neutral
timeline already provides exactly the hold / graceful exit / immediate second
signal behavior the catalog needs, but it assumes one global exit duration.

## Desired End State

- The status icon is a party-popper/spark mark: a macOS template mask and a
  restrained gold, oxblood, and teal mark on color-preserving trays.
- The menu lists Curtain, Water Drops, Fire, Blackout, Kaleidoscope, and Mosaic.
- Curtain is darker and more dimensional, with antique-gold braided trim and
  three warm pools of stage light.
- Each new flourish has a distinct hold animation and graceful reveal:
  droplets slide away, fire gutters out, black wipes diagonally, kaleidoscope
  opens radially, and mosaic tiles dissolve in a staggered field.
- All effects retain second-signal immediate dismissal and return to the same
  idle process.

## What We're NOT Doing

- Capturing or sampling presentation pixels; effects remain procedural and do
  not require screen-recording permission.
- Shipping third-party shader loading, effect installation, or a marketplace.
- Adding settings, thumbnails, global hotkeys, audio, or saved favorites.
- Claiming Windows/Linux runtime verification from a macOS build.
- Treating generative ambience as photorealistic fluid or fire simulation.

## Implementation Approach

Introduce a typed built-in flourish catalog with stable labels, shader IDs, and
effect-specific exit durations. Generalize the renderer from Curtain to an
effect-selected pipeline contract and keep all six visuals in one validated
WGSL module so they share compositor handling and full-screen geometry. Keep
the native menu flat and scannable; six choices do not yet justify nested UI.

Visual system: House Black `#090609`, Oxblood `#3A0714`, Velvet Red `#651324`,
Antique Gold `#D6A73C`, Footlight `#FFD978`, and Glass Blue `#8ED9E8`. The
signature is not a generic gradient: it is the tension between a tiny party
popper in the menu bar and full-screen effects that each reveal the live screen
with their own theatrical gesture.

Risk gate: reversibility **low**, surface sensitivity **medium** (presentation
overlay behavior), blast radius **low**, ambiguity **medium** (visual quality),
scale **medium**. Overall **medium**: proceed, preserve the hard-kill escape
hatch, and leave final visual approval to the user's meeting test.

## Phase 1: Catalog and menu identity

### Changes Required

- Add a built-in `Flourish` enum with labels, shader IDs, and exit durations.
- Replace the hard-coded Curtain menu ID with catalog-driven menu routing.
- Rasterize a party-popper/spark tray mark in process; use the alpha silhouette
  as a macOS template icon and preserve color elsewhere.

### Success Criteria

#### Automated Verification

- [x] Catalog identity, order, labels, IDs, and timing are unit tested.
- [x] Icon generation produces a correctly sized non-empty RGBA buffer.

#### Manual Verification

- [ ] The macOS icon is legible in both light and dark menu bars.
- [ ] The effect menu is quick to scan during a presentation.

## Phase 2: Curtain art direction

### Changes Required

- Darken the velvet palette and deepen fold contrast.
- Add animated antique-gold center piping and bottom braid.
- Add three soft, warm stage-light pools without flattening the folds.

### Success Criteria

#### Automated Verification

- [x] The enriched shader validates through wgpu initialization.

#### Manual Verification

- [ ] Curtain reads as dark velvet with visible gold trim.
- [ ] Stage lights add texture while the closed curtain remains calm.

## Phase 3: Five new procedural flourishes

### Changes Required

- Add transparent Water Drops and Fire overlays.
- Add opaque Blackout, Kaleidoscope, and Mosaic screens.
- Give each effect its own exit choreography while keeping timeline semantics
  shared.

### Success Criteria

#### Automated Verification

- [x] All built-in shader branches compile into one initialized pipeline.
- [x] Format, tests, Clippy, release build, and KB lint pass.

#### Manual Verification

- [ ] Every menu item launches the intended visual.
- [ ] First input exits gracefully; second input immediately clears every
  effect.
- [ ] Transparent effects reveal the live desktop rather than black.

## Testing Strategy

Unit-test stable catalog metadata and generated icon dimensions/content. Run
format, tests, strict Clippy, release build, and `kb check`. Launch each effect
on the actual macOS overlay for shader compilation and lifecycle smoke testing;
the user remains the authority on visual quality in a real meeting.

## References

- `src/main.rs`
- `src/renderer.rs`
- `src/shaders/flourishes.wgsl`
- `kb/reviews/flourish-v0-curtain-review.md`
