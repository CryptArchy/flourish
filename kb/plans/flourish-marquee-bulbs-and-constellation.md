---
id: flourish-marquee-bulbs-and-constellation
type: plan
project: flourish
tags: [wgpu, shaders, visual-design, catalog]
status: final
outcome: shipped
author: Christopher Andrews
created_date: 2026-07-26
executed_date: 2026-07-26
upstream: [flourish-effect-catalog, flourish-frosted-crt-depth-layers]
---

# Flourish Marquee Bulbs and Constellation

## Overview

Add two flourishes from outside the approved future catalog: **Marquee Bulbs**,
a theatre sign of Edison bulbs that burns itself out, and **Constellation**, a
star field that leaves as a meteor shower.

This document is written **after** implementation rather than before it. Both
effects were requested and built in one session, so it records what was decided
and why, in the same shape as the plans that preceded it, rather than pretending
to have gated the work.

## Current State Analysis

The catalog held twelve effects. Eleven draw through `flourishes.wgsl` on a
stable `shader_id`; Gravel Fall alone has a dedicated pipeline. Adding a shared
shader effect needs exactly three things: a `flourish_catalog!` row, a WGSL
constant plus function plus switch arm, and a README row. The catalog and
shader-arm tests are generic, so neither effect required a new test.

Nothing in the catalog was a field of discrete objects with independent lives —
every effect either holds a settled composition or moves as one continuous
material.

## Desired End State

- **Marquee Bulbs** holds as a dark sign board packed with Edison bulbs, each
  warming to full and cooling back to dark on its own schedule. Its exit drives
  every filament past full, fades the board out from under them, then pops the
  bulbs out one at a time.
- **Constellation** holds as a twinkling star field with faint asterism lines
  and a dusty band. Its exit retracts the lines, flings the whole sky away from
  a single radiant as a meteor shower, and sweeps the night off the screen
  behind it.
- Both obey the shared lifecycle: first signal starts the graceful exit, second
  signal kills immediately, and the reduced-motion path holds a settled frame at
  `SETTLED_SECONDS` with `exit_progress` pinned at zero.

## What We're NOT Doing

- No new renderer state, uniforms, pipelines, or CPU simulation. Both effects
  are pure fragment-shader work on the existing uniform set.
- No bundled textures. Bulb glass, filament, brass, stars, and trails are all
  procedural.
- No second dedicated pipeline. Gravel Fall remains the only effect that
  bypasses the shared catalog shader.

## Implementation Approach

Both effects read a 3x3 neighbourhood of a square-cell grid per pixel, which is
the structure that lets discrete objects light each other and overlap.

**Marquee Bulbs** (`shader_id` 12, 1,700 ms exit). Cells are square in a space
scaled by aspect, so bulbs stay round on any display. Each bulb runs two
`filament_pulse` cycles of unrelated period, because a single period is visibly
periodic within a few seconds of watching. The pulse is modelled rather than
eyeballed: a `smoothstep` warm-up over 0.16 s, then exponential cooling, since
that asymmetry is the whole difference between incandescent and LED. The halo
pass sums the nine nearest bulbs; its exponential falloff is multiplied by a
compact window that reaches zero inside the sampled neighbourhood, because an
exponential alone leaves a visible square of light around every bulb. The exit
gives each bulb a hashed stagger across surge, overdrive, and a deliberately
short life curve, so bulbs snap out rather than fade.

Palette — Cold Ember `#6B0E03`, Amber `#FF7517`, Warm White `#FFE5A8`, brass
base `#4B3820` under `#1B1409`, board near-black `#060504`.

**Constellation** (`shader_id` 13, 2,000 ms exit). Stars live on a hashed cell
grid with presence gating, a magnitude distribution skewed toward a few bright
ones, per-star twinkle and tint, and four-point diffraction on the brightest.
Asterism lines link a cell to one hashed neighbour where both ends exist.

The exit's key decision: the flight is a **scale about the radiant**, not a
per-star velocity. A grid-cell effect whose particles leave their home cell
stops being drawn, because the pixels they fly over never sample that cell — the
first attempt failed exactly this way and showed an empty sky. A uniform radial
scale is invertible, so a pixel can back-map into the rest frame and ask which
star is crossing it. Sampling is centred under the middle of the streak rather
than its head, so a whole trail stays inside the nine cells read. The geometry
also supplies the variation a per-star speed would otherwise fake: stars near
the radiant barely stir while the outer ones tear past.

Palette — night `#1C2152` to `#332B70`, star tints `#B8D1FF` to `#FFDBAD`,
asterism lines `#6B85C7`.

Both effects divide their accumulated light back out before `composite()`
premultiplies it. That is what lets the board — or the night — fade to nothing
underneath while the flare, or the last meteors, keep full brightness over the
screen already given back.

## Phase 1: Marquee Bulbs

### Changes Required

- `flourish_catalog!` row, placed second so the two theatre effects lead.
- `EFFECT_MARQUEE_BULBS`, `filament_pulse`, `bulb_offset`, `bulb_state`,
  `tungsten`, `marquee_bulbs`, and a switch arm.
- README catalog row.

### Success Criteria

#### Automated Verification

- [x] `cargo test --locked --all-targets`: 81 pass, including catalog metadata
  uniqueness and the shader-arm pairing.
- [x] `cargo clippy --all-targets --locked -- -D warnings`: clean.
- [x] `cargo fmt --all -- --check`: clean.
- [x] `--benchmark`: 2.72 ms at 5K, inside the 120Hz budget.

#### Manual Verification

- [ ] The sign reads as incandescent rather than as blinking LEDs.
- [ ] No square of light is visible around any bulb at any exit progress.
- [ ] Bulbs pop out individually and the screen comes back fully.

## Phase 2: Constellation

### Changes Required

- `flourish_catalog!` row after Gravel Fall.
- `EFFECT_CONSTELLATION`, `segment_probe`, `star_position`, `constellation`,
  and a switch arm.
- README catalog row.

### Success Criteria

#### Automated Verification

- [x] Catalog and shader-arm tests cover the new id.
- [x] `--benchmark`: 3.35 ms at 5K, inside the 120Hz budget.

#### Manual Verification

- [ ] Meteors persist for their whole flight rather than vanishing near their
  origin — the failure mode the radial map exists to prevent.
- [ ] Trails taper without a visible hard tail end.
- [ ] The last meteors read as burning over the already-revealed screen.

## Phase 3: Record

### Changes Required

- README catalog rows and the effect count.
- `kb/notes/flourish-frame-time-budget.md` re-measured in a single run.

### Success Criteria

#### Automated Verification

- [x] `kb check`: clean.

## Testing Strategy

Catalog and shader-validation tests are generic, so both effects were covered
the moment they were registered. Visual behaviour was checked by rendering the
shared shader offscreen to PNGs across the hold and a sweep of exit progress,
composited over a stand-in desktop so alpha behaviour was visible rather than
assumed; that harness was temporary and is deliberately not in the tree. Frame
time was measured through the shipped `--benchmark` path.

## References

- `src/lib.rs`
- `src/shaders/flourishes.wgsl`
- `kb/notes/flourish-frame-time-budget.md`
- `kb/tickets/flourish-future-catalog.md`
