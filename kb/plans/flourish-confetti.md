---
id: flourish-confetti
type: plan
project: flourish
tags: [wgpu, shaders, visual-design, catalog, celebration]
status: final
outcome: shipped
author: Christopher Andrews
created_date: 2026-07-27
upstream: [flourish-elevator-doors, flourish-surprise-and-frames]
---

# Flourish Confetti

## Overview

The first flourish proposed from scratch rather than drawn from the closed
backlog, and the argument for it is the app's own icon: the menu-bar and app
icon is a **party popper**, and nothing in seventeen effects celebrates.

## Current State Analysis

The catalog covers theatre, materials, retro tech, nature, and abstraction. It
has no celebration, which is odd for a tool whose stated purpose is "a little
tada during a presentation" and whose icon has promised one since the beginning.

Gravel Fall is the nearest neighbour — falling particles that accumulate — and
it differs on every axis that matters: dull tumbling stone against bright
tumbling foil, a pile that builds against a shower that passes through, a heavy
CPU simulation with a dedicated pipeline against arithmetic in the shared
shader.

## Desired End State

- **It does not cover the screen.** Confetti falls *over* whatever is on the
  display. Every other flourish takes the screen and gives it back; this one is
  a celebration laid over the slide you just finished on.
- Foil pieces tumble as they fall, flashing as they turn edge-on, in two depth
  layers so the shower has thickness.
- The field arrives fast enough to read as a pop rather than as weather
  starting, then settles into a steady shower for as long as it holds.
- The exit stops the source: the shower runs out from the top down while the
  last pieces accelerate off the bottom.
- Reduced motion holds a settled frame of the shower and cross-fades.

## What We're NOT Doing

- No opaque backing, vignette, or wash. The slide underneath stays legible.
- **No full-screen flash at the start.** A bright pop would sell the burst, and
  it is exactly the kind of high-contrast event Flourish deliberately avoids
  putting in front of a room; the reduced-motion path would never even see it.
- No accumulation on the floor. That is Gravel Fall's signature and the reason
  it owns a CPU simulation.
- No renderer state, new uniforms, or dedicated pipeline.

## Implementation Approach

`shader_id` 17, slug `confetti`, 1,500 ms exit, placed second in the menu behind
Curtain: the signature effect stays first, and the celebration follows it.

**Falling is a translation, and a translation is invertible.** As with
Constellation, the motion is written so a pixel can undo it. The field lives in
a flow space that slides down the screen — `flow = uv.y * rows - fall(time)` —
so a cell keeps its identity while its screen position moves. A pixel back-maps
into flow space, reads the nine nearest cells, and asks which pieces cover it.

Each cell holds at most one piece, hashed into colour, size, phase, and a sway
bounded to under half a cell so the three-by-three neighbourhood always contains
it. Two layers at different scales and speeds give depth: the far layer smaller,
dimmer, and slower, drawn behind.

**Tumble is the whole read.** A rectangle of foil rotating about its long axis
presents a width of `|cos(spin·t + phase)|`, so it periodically vanishes to a
line and flashes back. That single term is what separates confetti from falling
squares, and it costs one cosine. The same term picks which face is showing:
the back of a piece is the duller side of its own colour, and the moment near
edge-on is where a foil specular flash belongs.

**The exit stops the source rather than fading the field.** A line descends the
screen; above it the shower has run out, below it pieces keep falling and
accelerate away. Expressed against `exit_progress` directly, so it needs no
knowledge of the exit's duration inside the shader — the cross-language constant
Spotlight deliberately avoided.

Palette — Gold `#F2C14E`, Magenta `#E5486F`, Cyan `#3FC1C9`, Lime `#8CC63F`,
Coral `#F26B5B`, Paper White `#F7F4EF`, each with a duller reverse.

Risk gate: reversibility **high**, surface sensitivity **low**, blast radius
**low**, ambiguity **medium** — a transparent flourish is a first for the
catalog and only a viewing settles whether it reads as celebration or as
clutter over a slide — scale **small**. Overall **low**: proceed.

## Phase 1: Confetti

### Changes Required

- `flourish_catalog!` row: `Confetti`, slug `confetti`, id 17, 1,500 ms.
- `EFFECT_CONFETTI` constant, the `confetti` function and helpers, and a switch
  arm.
- README catalog row and effect count.

### Success Criteria

#### Automated Verification

- [x] `cargo test --locked --all-targets` passes, 95 tests, including catalog
  uniqueness and the shader-arm pairing.
- [x] `cargo clippy --all-targets --locked -- -D warnings` clean.
- [x] `cargo fmt --all -- --check` clean.
- [x] `--benchmark` in a single clean run. Expected in Marquee Bulbs' band or
  below: two neighbourhood layers, but arithmetic rather than noise. **Landed
  slightly above** at 2.99 ms against Marquee's 2.73 — between Marquee and
  Constellation. Two arithmetic loops cost a little more than one loop that also
  samples noise, not a little less.
- [x] Alpha reaches zero by the end of the exit, without a final-clear term.

#### Manual Verification

Viewed on 2026-07-27. The report was an overall read — "it looks celebratory" —
rather than an item-by-item check, which settles the risk the whole plan hung on
and covers the first two by implication: a field of falling squares over an
illegible slide is not what celebratory looks like.

- [x] Pieces read as tumbling foil, not as falling squares.
- [x] The slide underneath stays legible throughout.
- [ ] The field arrives as a pop rather than as weather starting. Not separately
  reported.
- [ ] The exit empties from the top rather than fading everything at once. Not
  separately reported; visible in the contact sheet, unverified in motion.

## Phase 2: Record

### Changes Required

- README, and `kb/notes/flourish-frame-time-budget.md` re-measured in one run.

### Success Criteria

#### Automated Verification

- [x] `kb check` clean.
- [x] Frame-time note re-measured; the whole table is from one run.

## Testing Strategy

Catalog and shader-validation tests cover registration. Composition and alpha
are checked with `--frames`, which is now the shipped path for exactly this and
replaces the throwaway harness the last few effects used. **Motion is checked by
launching it**: Paper Tear cost three rounds because stills cannot show whether
something moves like the thing it imitates, and tumble is precisely that kind of
claim.

## References

- `src/lib.rs`
- `src/shaders/flourishes.wgsl`
- `src/icon.rs`
- `kb/reviews/flourish-elevator-doors-review.md`
