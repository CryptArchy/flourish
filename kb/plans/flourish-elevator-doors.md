---
id: flourish-elevator-doors
type: plan
project: flourish
tags: [wgpu, shaders, visual-design, catalog, metal]
status: final
outcome: shipped
author: Christopher Andrews
created_date: 2026-07-27
upstream: [flourish-future-catalog, flourish-paper-tear]
---

# Flourish Elevator Doors

## Overview

Promote **Elevator Doors** out of the approved future catalog: brushed-metal
doors that part with moving reflected highlights. The last of the original four,
now that Chalkboard is declined.

## Current State Analysis

Sixteen effects ship. The catalog has one specular surface — Projector Iris's
gunmetal blades — and no flat, mirror-like one.

**The real problem this plan has to solve is Curtain.** Curtain also parts from
the centre, and the ticket's own rule is one signature movement per effect. Two
centre-parting reveals is the closest overlap the catalog would contain, closer
than Spotlight to Projector Iris. It is worth doing anyway, because the two are
opposite in every respect that carries the movement:

| | Curtain | Elevator Doors |
| --- | --- | --- |
| Material | Draped velvet, soft edge | Rigid steel, hard machined edge |
| Motion | Fabric lag, settling wobble | One mechanical profile, no overshoot |
| Light | Diffuse pools on cloth | Specular reflections of a room |
| Edge | Gold braid trim | Chamfered door edge, shadowed opening |

If it still reads as a metal curtain when it runs, the answer is to lean harder
on the mechanics — the motion profile and the edge — not on the texture.

## Desired End State

- Holds as a closed pair of brushed-steel doors meeting at a centre seam, with
  reflected highlights drifting slowly across them.
- The exit parts them on a mechanical profile: a moment of load before anything
  moves, then a smooth accelerating slide, no bounce and no settle.
- Each door's leading edge shows a chamfer and casts a soft shadow into the
  opening, so the doors read as thick panels rather than as painted halves.
- Reduced motion holds the closed doors and cross-fades.

## What We're NOT Doing

- No floor indicator, no call button, no arrow lights, no "ding" flash. The
  concept is the doors.
- No lobby behind the doors: what they open onto is the presenter's screen.
- No renderer state, new uniforms, or dedicated pipeline.

## Implementation Approach

`shader_id` 16, slug `elevator-doors`, 1,600 ms exit, placed after Projector
Iris so the two mechanical effects sit together.

**The detail that sells it is that reflections do not move with the door.** A
flat mirror sliding within its own plane leaves its reflected image where it is
— the room does not move because the door did. So the brushed grain is sampled
in *panel* coordinates and travels with the door, while the reflected highlights
are sampled in *screen* coordinates and stay put. As a door slides, its
highlights appear to sweep across its face. That single split is most of the
difference between this and a pair of grey rectangles sliding apart, and it is
free: two coordinate systems, no extra samples.

**Brushed steel.** Fine vertical striations — noise stretched hard along y — at
two frequencies, over a broad vertical gradient standing in for a lit room. The
striations must ride in panel space or the metal shears as the door moves.

**Reflections.** Broad soft diagonal bands in screen space, drifting slowly, plus
one brighter specular sweep. Anisotropic: the brushing smears reflections along
the grain, so the bands are modulated by the striation field rather than laid
cleanly over it.

**The seam and the edges.** At rest, a dark hairline at the centre with a bright
chamfer either side. Once moving, each leading edge keeps that chamfer and adds
a soft shadow thrown into the opening, the same trick Paper Tear uses to say the
overlay has thickness.

**Motion.** A short hold under load, then `pow` acceleration with no easing at
the end, because a door that eases out looks like it is being placed rather than
driven. The doors must fully clear the screen before the exit ends.

*Revised during implementation:* the slack take-up is shading, not motion. As
geometry it means negative travel, which overlaps the two halves and swallows the
seam for those frames; it is now a brief deepening of the seam shadow, which
reads as the mechanism loading rather than as the doors moving.

Palette — Steel `#8A9099`, Bright Grain `#C6CCD3`, Shadowed Grain `#4E545C`,
Seam `#14171A`, Specular `#F2F6FA`.

Risk gate: reversibility **high** (one shader function plus a catalog row),
surface sensitivity **low**, blast radius **low**, ambiguity **medium** — the
overlap with Curtain is a judgement call that only a real viewing settles —
scale **small**. Overall **low**: proceed, leaving visual acceptance manual.

## Phase 1: Elevator Doors

### Changes Required

- `flourish_catalog!` row: `ElevatorDoors`, slug `elevator-doors`, id 16,
  1,600 ms.
- `EFFECT_ELEVATOR_DOORS` constant, the `elevator_doors` function and helpers,
  and a switch arm.
- README catalog row and effect count.

### Success Criteria

#### Automated Verification

- [x] `cargo test --locked --all-targets` passes, 81 tests, including catalog
  uniqueness and the shader-arm pairing.
- [x] `cargo clippy --all-targets --locked -- -D warnings` clean.
- [x] `cargo fmt --all -- --check` clean.
- [x] `--benchmark` in a single clean run, read against the contended-run check
  in the frame-time note. Expected near Curtain: the mapping is closed-form and
  only the striations cost samples. Measured 0.43 ms at 5K — tied with Spotlight,
  just under Curtain's 0.47, with every other row at baseline.
- [x] Alpha reaches zero by the end of the exit.

#### Manual Verification

- [ ] The doors read as brushed metal, not as grey panels.
- [ ] Highlights sweep across each door as it slides, rather than riding along
  with it.
- [ ] The motion reads as machinery: loaded, then driven, without a settle.
- [x] **It does not read as a metal Curtain.** This is the one that decides
  whether the effect earns its place. Confirmed by viewing, 2026-07-27.

## Phase 2: Record

### Changes Required

- Move Elevator Doors into the future catalog's promoted section, which empties
  its remaining list and closes the ticket.
- Re-measure `kb/notes/flourish-frame-time-budget.md` in a single run.

### Success Criteria

#### Automated Verification

- [x] `kb check` clean, 34 files.
- [x] The future-catalog ticket reaches `step: done`, with its remaining list
  empty for the first time since 2026-07-18.

## Testing Strategy

Catalog and shader-validation tests cover registration generically. Composition
and alpha are checked offscreen across the hold and a sweep of exit progress.
**Motion is checked by launching it**, not by reading frames: the last effect
cost three rounds of feedback because stills cannot show whether something moves
like the thing it is imitating, and this one's whole risk is exactly that.

## References

- `src/lib.rs`
- `src/shaders/flourishes.wgsl`
- `kb/tickets/flourish-future-catalog.md`
- `kb/reviews/flourish-paper-tear-review.md`
