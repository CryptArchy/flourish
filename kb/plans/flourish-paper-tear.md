---
id: flourish-paper-tear
type: plan
project: flourish
tags: [wgpu, shaders, visual-design, catalog, paper]
status: final
outcome: shipped
author: Christopher Andrews
created_date: 2026-07-26
upstream: [flourish-future-catalog, flourish-spotlight]
---

# Flourish Paper Tear

## Overview

Promote **Paper Tear** out of the approved future catalog: a textured sheet
covering the screen that tears down the middle and curls away. Fourth of the
original set to graduate, leaving Chalkboard and Elevator Doors.

## Current State Analysis

Fifteen effects ship. Frosted Glass is the only pale one; everything else holds
a dark field, which matters on a projector in a lit room where dark effects have
the least headroom. A cream sheet is the second bright hold state in the
catalog and the only one that is opaque and evenly lit.

Geological Strata already splits a surface and shears the halves apart, so Paper
Tear has to differ in the same way Spotlight had to differ from Projector Iris.
It does, on both counts: Strata's fault is a clean shear of a rigid material
that stays flat, while paper is thin, fibrous, and — the whole point of the
concept — *rolls*.

## Desired End State

- Holds as a sheet of paper filling the screen: warm off-white, fibrous, softly
  mottled, with a broad shading that drifts slowly enough to read as a lit
  surface rather than a still image.
- The exit tears the sheet down the middle. The crack **propagates from the top
  down** with a ragged, fibrous edge; rows the crack has not reached are still
  joined.
- Behind the crack, each half pulls outward and its torn edge **curls into a
  roll** that thickens as it goes, until both halves leave the screen.
- The paper casts a soft shadow onto whatever is revealed beneath it, so the
  sheet reads as lying above the screen rather than being the screen.
- Reduced motion holds the flat sheet and cross-fades.

## What We're NOT Doing

- No page-turn, no folding, no crumpling. One tear, two rolls.
- No rotation of the halves. The roll axis is vertical, and a tilt would break
  the mapping that makes the curl cheap; a slight downward drift supplies the
  same life.
- No renderer state, new uniforms, or dedicated pipeline.

## Implementation Approach

`shader_id` 15, slug `paper-tear`, 1,900 ms exit, placed after Geological
Strata so the two material effects sit together.

**The mapping is the whole effect.** A pixel has to answer "which piece of paper
is over me, and what part of that piece is it?" — so, as with Constellation, the
motion is written as something a pixel can invert rather than something the
material does forward.

Sheet coordinates are screen coordinates at rest. For each half:

- The tear is two scales. `tear_axis(y)` is the smooth large-scale path and
  carries the curl, because a cylinder needs a straight axis: driving it with
  the full ragged edge wobbles the roll sideways by as much as it is wide, and
  it reads as a lumpy ribbon. `tear_rag(y)` is the kinks and fibre roughness.
- The rag lives in different places depending on how curled the paper is. Flat,
  it is simply where the edge is. Curled, it becomes a difference in how far
  that row has *wrapped*, so the torn edge stays ragged on a straight roll.
  It has to blend between the two: forcing rag into the wrap while the sheet is
  still flat invents a curl on paper that has not curled, and draws a ragged
  ghost line down the held sheet.
- `row_open(y)` gates everything by whether the crack has reached that row:
  the crack front races down the screen over the first third of the exit.
- Beyond the crack, the half translates outward by `travel` and its torn edge
  becomes a roll of radius `R`, both scaled by `row_open`, so a row that is
  still joined is still flat.

*Added during implementation:* rows that have been open longer are pulled
further apart, so the tear opens as a V rather than as a parallel gap. Gating
only on `row_open` gives every opened row the same separation, which reads as
two panels sliding apart behind a ragged mask.

*Revised after the first on-screen viewing:* the wedge is constant in time and
sized so even the slowest row clears the screen. It originally equalized over
the second half of the exit, which made the bottom catch up as a distinct late
motion — read on screen as two sheets tearing at different times. The crack
front also has to overshoot the bottom of the screen by more than the row
smoothstep is wide, or the last rows never reach full separation and a joined
band survives along the bottom edge for the whole exit.

**The curl.** *Rebuilt after the second viewing; the original model is described
at the end of this section because its failure is the instructive part.*

The paper leaves the flat plane at a tangent point, wraps a cylinder of radius
`R` through `wrap` radians, and ends at the torn edge. Past a quarter turn the
wrapped paper comes back **over** the sheet, and what the viewer sees there is
its *back face* lying on top of the paper it came from. That flap is what reads
as a curl. Three surfaces can cover a pixel — the flap, the concave inside of
the curl, and the flat sheet — and the flap is nearest the viewer wherever it
reaches, so it is tested first.

For a pixel `q` outward from the tangent, the flap is at `φ = π - asin(q/R)` and
the inside at `φ = asin(q/R)`; paper exists on the arc up to `wrap`. Flat paper
is rigid and simply translated. Rolling consumes paper, so each tangent recedes
by its own arc length — which means the curl opens the tear on its own, and the
translation has to be sized smaller to compensate.

`R = 0` at rest, which is a division by zero; rows below the crack front take a
flat branch rather than relying on a guard value.

**The original model curled the wrong way** — away from the viewer, so only the
front of a receding cylinder was ever visible. That draws a flat bright band
beside a flat bright sheet with a seam between them, which is a picture of two
stacked sheets, and no amount of shading rescues it. The direction of the curl,
not its shading, is what decides whether a curl reads as one.

**One light, upper-left, in front of the sheet.** The two rolls are mirror
images, so shading both from the same curve of `θ` lights each half as though it
had its own lamp — the single largest reason the halves stopped reading as one
sheet. The roll's normal is `(-side·cos θ, sin θ)`, and every other shading term
obeys the same direction: only the half whose roll sits on its lit side shadows
its own flat paper, and the gap is shadowed from the lit edge, with a tight
contact darkening on both edges because occlusion has no direction.

**The texture rides in sheet coordinates**, not screen coordinates, so the grain
travels with the paper and compresses into the roll instead of sliding under it.
That is the detail that separates paper from a moving gradient.

**The shadow.** Where a gap has opened, alpha is not zero: a soft dark falloff
from the nearest torn edge sits over the revealed screen. It is the only cue
that the sheet has thickness and height above what is beneath it.

Palette — Paper `#F0EAD9`, Mottle `#E2D9C4`, Roll Shade `#C9BFA8`, Underside
`#A99C82`, Deep Crease `#7A7060`, Fibre Highlight `#FBF7EE`.

Risk gate: reversibility **high** (one shader function plus a catalog row),
surface sensitivity **low**, blast radius **low**, ambiguity **medium** — the
curl is the first mapping in the catalog with a non-linear inverse, and it may
need more than one pass to read as paper rather than as a shaded band — scale
**small**. Overall **low**: proceed, leaving visual acceptance manual.

## Phase 1: Paper Tear

### Changes Required

- `flourish_catalog!` row: `PaperTear`, slug `paper-tear`, id 15, 1,900 ms.
- `EFFECT_PAPER_TEAR` constant, the `paper_tear` function and its helpers, and
  a switch arm.
- README catalog row and effect count.

### Success Criteria

#### Automated Verification

- [x] `cargo test --locked --all-targets` passes, 81 tests, including catalog
  uniqueness and the shader-arm pairing.
- [x] `cargo clippy --all-targets --locked -- -D warnings` clean.
- [x] `cargo fmt --all -- --check` clean.
- [x] `--benchmark` in a single run; the expected band is between Spotlight and
  Marquee Bulbs, since the mapping is closed-form but the paper texture costs
  several noise samples per pixel. Measured 0.91 ms at 5K, between Spotlight's
  0.43 and Marquee Bulbs' 2.72 — the prediction held, and the cost is the noise
  samples rather than the mapping.
- [x] Alpha reaches zero by the end of the exit: the last frame of the sweep
  renders as the stand-in desktop, unmodified.

#### Manual Verification

- [ ] The hold state reads as paper — fibre and mottling, not a flat cream
  rectangle.
- [ ] The tear propagates downward and its edge looks torn rather than cut.
- [ ] The torn edges read as rolling, with the grain compressing into the roll.
- [ ] The shadow sells the sheet as sitting above the screen.

## Phase 2: Record

### Changes Required

- Move Paper Tear into the future catalog's promoted section, leaving
  Chalkboard and Elevator Doors in its remaining list.
- Re-measure `kb/notes/flourish-frame-time-budget.md` in a single run.

### Success Criteria

#### Automated Verification

- [x] `kb check` clean.
- [x] Frame-time note re-measured in one run; every pre-existing row at baseline.

## Testing Strategy

Catalog and shader-validation tests cover registration generically. Visual
behaviour is checked by rendering the shared shader offscreen across the hold
and a sweep of exit progress, composited over a stand-in desktop so both the
shadow and the final clearance are observed rather than assumed. Frame time
through the shipped `--benchmark`, read against the contended-run check now
recorded in the note.

## References

- `src/lib.rs`
- `src/shaders/flourishes.wgsl`
- `kb/tickets/flourish-future-catalog.md`
- `kb/plans/flourish-spotlight.md`
