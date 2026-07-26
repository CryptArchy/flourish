---
id: flourish-visual-feedback-tuning
type: plan
project: flourish
tags: [wgpu, compute, shaders, visual-tuning]
status: final
outcome: partial
author: Christopher Andrews
created_date: 2026-07-18
upstream: [presentation-flourish-runner, 2026-07-18_doom-fire-and-alpha]
---

# Flourish Visual Feedback Tuning

## Overview

Apply direct visual feedback to the first catalog: fix opaque-effect exits,
replace bubble-like Water Drops with pond ripples, make procedural Fire less
sine-like, and add a separate stateful Doom Fire flourish.

Second visual review retained the original fluid Fire as the preferred version,
accepted Doom Fire while identifying stretched cells, and requested a darker
Curtain whose light pools deform with the cloth and whose bottom trim reaches
the physical screen edge.

## Current State Analysis

Kaleidoscope and Mosaic reduce alpha over unscaled full-color RGB, exposing an
alpha convention mismatch on macOS. Water Drops moves one shared sample field
vertically and therefore cannot read as independent pond impacts. Fire uses
smooth value noise in a continuous height field. The renderer has one fragment
pipeline and no effect-specific persistent GPU state.

## Desired End State

- Kaleidoscope and Mosaic reveal the desktop cleanly throughout their exits,
  with no bright color layer left behind.
- `Pond Ripples` shows several independent, concentric surface disturbances;
  exit damps them in place without shared translation or bounce.
- `Fire` retains its original soft, fluid shader character.
- `Doom Fire` is a separate menu item backed by persistent temperature cells,
  randomized upward propagation, discrete cooling, lateral spread, and the
  recognizable heat palette.
- Doom Fire cells are square-to-slightly-tall on any practical display aspect.
- Curtain uses soft radial pools in fabric space, much darker unlit velvet, and
  gold trim through the final pixel row.
- All four changes retain first-signal graceful exit and second-signal kill.

## What We're NOT Doing

- Sampling or refracting the actual presentation behind the pond ripples.
- Copying GPL source code or PlayStation assets; implement the published rule
  independently and supply an original palette.
- Replacing the existing Fire; Doom Fire is an intentional alternative.
- Generalizing persistent simulations into a third-party plugin API.

## Implementation Approach

Prefer `PreMultiplied` compositor mode and make transparent shader output
premultiplied by construction. Rename Water Drops to Pond Ripples while
preserving shader ID `1` so catalog identity remains stable. Sharpen procedural
Fire with fractured multi-scale noise.

Add a low-resolution two-buffer Doom simulation. A compute pass propagates
integer heat upward and sideways while cooling; a fragment branch samples the
current storage buffer and maps temperature through a discrete-inspired
palette. Reset both buffers whenever Doom Fire starts.

Risk gate: reversibility **low**, surface sensitivity **medium**, blast radius
**medium** (renderer bind-group layout), ambiguity **medium** (art direction),
scale **medium**. Overall **medium**: proceed with the existing hard-kill safety
path and require human visual validation.

## Phase 1: Alpha and ripple corrections

### Changes Required

- Prefer premultiplied surface composition and premultiply shader output.
- Replace global droplet motion with independent concentric pond ripples.
- Rename the menu item and documentation to Pond Ripples.

### Success Criteria

#### Automated Verification

- [x] Rust and WGSL validation pass with the corrected compositor contract.
- [x] Catalog tests preserve unique labels and shader IDs.

#### Manual Verification

- [ ] Kaleidoscope and Mosaic reveal continuously without residual color.
- [ ] Ripples originate independently and dissipate without translation or
  bounce.

## Phase 2: Fire character and Doom automaton

### Changes Required

- Keep the first-pass Fire experiment available for comparison, then restore
  the original broad value-noise silhouette after visual review.
- Add resettable ping-pong GPU heat buffers and a compute update pass.
- Add Doom Fire to the catalog and render its heat field with an original
  black-to-white-hot palette.

### Success Criteria

#### Automated Verification

- [x] Buffer sizing, bottom-source initialization, compositor selection, and
  catalog metadata are unit tested where renderer independence permits.
- [x] The compute and fragment pipelines initialize and dispatch on macOS/Metal.
- [x] Format, tests, strict Clippy, release build, and KB lint pass.

#### Manual Verification

- [x] Doom Fire provides the preferred harder, cellular fire alternative.
- [ ] Doom Fire visibly propagates upward from its bottom source and dies down
  on first signal.
- [ ] Second signal clears both fire variants immediately.

## Phase 3: Second visual-feedback pass

### Changes Required

- Restore Fire's original broad value-noise silhouette and edge softness.
- Widen Doom Fire's backing field and select an aspect-correct visible column
  count so cells are never wider than tall.
- Replace Curtain's screen-space cone beams and lamp dots with radial gradients
  evaluated in inverse cloth coordinates before fold modulation.
- Darken unlit velvet and extend the bottom gold band to the screen edge.

### Success Criteria

#### Automated Verification

- [x] Format, tests, strict Clippy, release build, real Metal initialization,
  and KB lint continue to pass.

#### Manual Verification

- [ ] Fire matches the preferred original fluid version.
- [ ] Doom Fire pixels read square or subtly portrait, never landscape.
- [ ] Curtain light pools bend with folds and leave most velvet theater-dark.
- [ ] No red curtain is visible below the bottom gold trim.

## Testing Strategy

Unit-test catalog metadata and pure Doom buffer initialization math. Run the
full Rust verification suite and initialize both render and compute pipelines
on the actual macOS overlay. Visually exercise all corrected exits and both
fire variants; do not infer visual correctness from compilation.

## References

- `kb/research/2026-07-18_doom-fire-and-alpha.md`
- `src/renderer.rs`
- `src/shaders/flourishes.wgsl`
- <https://fabiensanglard.net/doom_fire_psx/>

## Deviations from Plan

- The initial request to make Fire more jagged was implemented and then
  explicitly reversed after visual comparison; Doom Fire now owns the harder,
  cellular aesthetic while Fire returns to its softer original identity.
