---
id: flourish-marquee-bulbs-and-constellation-review
type: review
project: flourish
tags: [wgpu, shaders, visual-design, catalog]
status: final
author: Christopher Andrews
created_date: 2026-07-26
upstream: [flourish-marquee-bulbs-and-constellation]
---

# Flourish Marquee Bulbs and Constellation Review

## Implementation Status

**Complete, pending on-screen confirmation.** Both effects are registered,
drawing, benchmarked, and documented. Every automated check passes. Visual
acceptance was done against offscreen renders rather than a real projector, so
the presentation-display pass is still owed.

## Automated Verification Results

- `cargo fmt --all -- --check`: pass
- `cargo test --locked --all-targets`: pass, 81 tests
- `cargo clippy --all-targets --locked -- -D warnings`: pass
- `cargo run --release -- --benchmark`: pass, worst case unchanged at Frosted
  Glass 4.29 ms at 5K
- `cargo run -- --list`: pass, fourteen slugs
- `kb check`: pass, 0 errors and 0 warnings

## Findings

### Matches plan

- Both effects are pure additions to `flourishes.wgsl` behind stable ids 12 and
  13, with no renderer, uniform, or pipeline changes. Gravel Fall remains the
  only dedicated pipeline.
- Marquee Bulbs models the incandescent asymmetry — fast warm-up, slow
  exponential cool — and runs two cycles of unrelated period per bulb, so the
  sign does not visibly loop.
- Constellation's exit is a scale about the radiant, which is invertible, so
  meteors survive their whole flight instead of dying near their origin.
- Both divide accumulated light back out before `composite()` premultiplies, so
  the backdrop can fade to nothing while the light keeps its brightness over the
  revealed screen.
- Alpha demonstrably reaches zero for both: the final frames of each exit sweep
  are the stand-in desktop, unmodified.
- Frame-time note re-measured in one run rather than having two effects appended
  to a table produced by a different run.

### Deviations

- **The plan was written after the work, not before it.** The session went
  straight from request to implementation. The plan document says so in its own
  overview rather than presenting a retroactive gate as a real one.
- Constellation's meteors have no per-star speed variation, because per-star
  speeds break the invertible map the whole exit depends on. Radial geometry
  supplies the variation instead: outer stars move faster than inner ones for
  free.
- The offscreen preview harness used to art-direct both effects was deleted
  rather than kept. It duplicated pipeline setup that `Scene` already owns and
  would have rotted against it; see Recommendations.

### Potential issues

- Marquee's burst reads milky between bulbs at roughly 60–70% of exit progress,
  where overlapping halos wash the gaps before the board has finished fading.
  One constant (`halo` overdrive amplitude) governs it.
- Constellation's sky wipe starts at 46% of exit progress and may be chasing the
  shower rather than following it; the shower is the better part of the effect
  and could be given longer.
- Both effects are in the expensive band — second and third worst at 5K. Still
  inside the 120Hz budget on Apple Silicon, but they inherit the frame-time
  note's Intel caveat more than the cheap effects do.
- Marquee's full-field flare is bright and brief. The reduced-motion path never
  reaches it, since that path pins `exit_progress` at zero, but it is the
  highest-contrast moment in the catalog for anyone in full motion.
- Constellation's star lattice becomes faintly perceptible in the early exit,
  when the whole field scales together before the streaks lengthen.

## Manual Testing Required

- Run each on a real presentation display and confirm the hold states read at
  projector contrast — particularly Constellation, whose dark sky has the least
  headroom on a washed-out projector.
- Dismiss Marquee once and confirm the bulbs pop individually rather than
  dissolving together, and that no square of halo is visible around any bulb.
- Dismiss Constellation once and watch a single meteor across its whole flight,
  confirming it does not blink out mid-trail.
- Double-signal during both exits and confirm immediate removal.
- Toggle Reduce Motion and confirm both hold a settled frame and cross-fade.

## Recommendations

Tune Marquee first through the halo's overdrive amplitude and Constellation
first through the wipe's start, preserving both signatures. Neither needs a
structural change.

If a third grid-neighbourhood effect is ever added, revisit whether the offscreen
preview harness should become a real `--frames` flag on the binary — reaching
`Scene` directly, rather than an example that rebuilds the pipeline beside it.
Two effects is not yet enough to justify the surface.
