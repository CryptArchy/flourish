---
id: flourish-confetti-review
type: review
project: flourish
tags: [wgpu, shaders, visual-design, catalog, celebration]
status: final
author: Christopher Andrews
created_date: 2026-07-27
upstream: [flourish-confetti]
---

# Flourish Confetti Review

## Implementation Status

**Complete, pending the motion check.** Confetti is registered, drawing,
benchmarked, and documented — the eighteenth flourish and the first proposed
from scratch rather than drawn from the closed backlog.

Composition and alpha were reviewed from `--frames`, which is the shipped tool
for that now rather than a throwaway harness. Whether foil *tumbles* is a claim
about motion, and this review does not make it.

## Automated Verification Results

- `cargo fmt --all -- --check`: pass
- `cargo test --locked --all-targets`: pass, 95 tests
- `cargo clippy --all-targets --locked -- -D warnings`: pass
- `cargo run --release -- --benchmark`: pass, 2.99 ms at 5K in a clean run;
  catalog worst case unchanged at Frosted Glass, 4.36 ms
- `cargo run --release -- --frames`: strip inspected
- `kb check`: pass

## Findings

### Matches plan

- One shader function plus one helper, one constant, one switch arm, one catalog
  row. No renderer, uniform, or pipeline changes.
- The flow-space translation is invertible, so a pixel back-maps and reads nine
  cells, exactly as Constellation's radial map does. Sway is bounded under half
  a cell, which is what keeps that neighbourhood sufficient.
- It does not cover the screen. The stand-in desktop is visible through every
  tile of the strip, including the first.
- The exit stops the source: the strip empties from the top down and the last
  tile is the bare desktop, with no final-clear hack needed — the descending
  line passes the bottom edge on its own.
- No full-screen flash. The plan ruled one out and none was added.

### Deviations

- **The frame-time prediction was slightly wrong.** The plan expected "Marquee
  Bulbs' band or below"; it landed at 2.99 ms against Marquee's 2.73, between
  Marquee and Constellation. The reasoning was right — arithmetic is cheap — but
  two neighbourhood loops cost a little more than one that samples noise, not a
  little less.
- **The burst is the start rather than an event.** The plan wanted the field to
  "arrive as a pop". In flow space the field is infinite, so the screen is full
  of confetti on the first frame, and what was added was a launch term that
  decays: the pieces rush at first and settle to a drift. Whether that reads as
  a pop or as an abrupt start is a motion question.

### Potential issues

- **The transparency is the risk, and it cuts both ways.** Confetti over a live
  slide is the whole idea, but it is also the only flourish that never takes
  the screen, so it may read as clutter over content rather than as
  celebration. Pond Ripples is the precedent at 0.325 peak alpha; this one is
  opaque where a piece covers, transparent everywhere else.
- Pieces cast no shadow, so they sit *on* the slide rather than above it. A
  cheap offset shadow would fix it and would double the shape cost.
- The foil flash is a `pow(1 - |cos|, 6)` term near edge-on. It is plausible in
  a still; whether it reads as catching the light or as a flicker needs motion.
- Density is fixed. On a busy slide it may be too much, and there is no way to
  ask for less.
- Nothing accumulates at the bottom, deliberately — that is Gravel Fall's
  signature — but it does mean the shower has no destination.

## Manual Testing Required

- **Watch it over a real slide.** The question is whether it celebrates or
  clutters, and it is the only one that matters here.
- Confirm pieces read as tumbling foil rather than as falling squares.
- Confirm the opening reads as a pop rather than as an abrupt start.
- Double-signal during the exit and confirm immediate removal.
- Toggle Reduce Motion and confirm a settled frame and a cross-fade.

## Recommendations

If it reads as clutter, the first lever is density rather than opacity: fewer,
larger pieces keep the celebration and give the slide back. Thinning the near
layer alone would also buy most of the frame time back.

If it reads well but flat, the shadow is the next thing to add, and it is worth
the doubled shape cost — it is the only cue that would put the confetti in front
of the screen rather than on it.
