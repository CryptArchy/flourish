---
id: flourish-elevator-doors-review
type: review
project: flourish
tags: [wgpu, shaders, visual-design, catalog, metal]
status: final
author: Christopher Andrews
created_date: 2026-07-27
upstream: [flourish-elevator-doors]
---

# Flourish Elevator Doors Review

## Implementation Status

**Complete and accepted.** Elevator Doors is registered, drawing, benchmarked,
and documented, and promoting it empties the future catalog's remaining list and
closes that ticket.

The plan named a single risk — that this reads as a metal Curtain rather than as
its own effect — and said only a viewing settles it. Viewed on 2026-07-27: it
reads as its own effect. That was the criterion the whole plan hung on, and it
passed on the first attempt, unlike Paper Tear.

## Automated Verification Results

- `cargo fmt --all -- --check`: pass
- `cargo test --locked --all-targets`: pass, 81 tests
- `cargo clippy --all-targets --locked -- -D warnings`: pass
- `cargo run --release -- --benchmark`: pass, 0.43 ms at 5K in a clean run with
  every other row at baseline; catalog worst case unchanged at Frosted Glass
  4.26 ms
- `kb check`: pass, 34 files

## Findings

### Matches plan

- One shader function plus one helper, one constant, one switch arm, one catalog
  row. No renderer, uniform, or pipeline changes.
- The grain rides in panel coordinates and the reflections in screen
  coordinates, so a sliding door passes *under* its own highlights. Confirmed
  against the renders: a bright band sits at the same screen position with the
  doors shut and a third of the way open.
- Motion is a load pause and then `pow` acceleration with no easing at the end.
- Leading edges carry a chamfer and a dark machined lip; at rest the two lips
  together are the seam, so the seam needed no separate code.
- Cost landed where predicted, tied with Spotlight and just under Curtain.

### Deviations

- **The slack take-up became shading rather than motion.** The plan wanted the
  doors to press together before parting. As geometry that means negative
  travel, which overlaps the two halves and swallows the seam for those frames.
  It is now a brief deepening of the seam shadow instead — the mechanism loading
  rather than the doors moving.

### Potential issues

- ~~The Curtain overlap is unresolved.~~ Settled by viewing: the two read as
  different effects. Worth keeping in mind that they remain the closest pair in
  the catalog, so a future effect that parts from the centre has two incumbents
  to differ from, not one.
- Reflections are two sine bands and a Gaussian sweep. They read as a lit room
  in stills; whether they read as a *reflection* while the door slides under
  them is exactly the thing stills cannot show.
- The hold is bright and low-contrast, so on a washed-out projector the doors
  may lose their seam and read as one grey field. Curtain and Paper Tear are the
  comparison points here — one dark, one bright.
- Live launch on Metal has not been done.

## Manual Testing Required

- ~~Watch it against Curtain, back to back.~~ Done; it reads as its own effect.
- Confirm the highlights sweep across each door rather than travelling with it.
- Confirm the motion reads as driven machinery: loaded, then moving, no settle.
- Double-signal during the exit and confirm immediate removal.
- Toggle Reduce Motion and confirm a settled closed pair and a cross-fade.

## Recommendations

If the doors need more mechanical character, the cheapest addition is a shallow
recessed track at the top and bottom edges of the frame — a fixed detail the
doors slide *within*, which reads as machinery without adding motion. That was
deliberately left out under the plan's "no floor indicator, no call button" rule,
but a track is structure rather than ornament and would not break it.

This closes the future catalog. Any further flourish is a fresh idea rather than
a backlog item, and the catalog is large enough now that the next one should
have to argue for its place against the seventeen already there — as this one had
to argue against Curtain.
