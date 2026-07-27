---
id: flourish-spotlight-review
type: review
project: flourish
tags: [wgpu, shaders, visual-design, catalog, theatre]
status: final
author: Christopher Andrews
created_date: 2026-07-26
upstream: [flourish-spotlight]
---

# Flourish Spotlight Review

## Implementation Status

**Complete, pending on-screen confirmation.** Spotlight is registered, drawing,
benchmarked, documented, and promoted out of the future catalog. Every automated
check passes. Visual acceptance was done against offscreen renders across the
hold and a sweep of exit progress; the presentation-display pass is still owed.

## Automated Verification Results

- `cargo fmt --all -- --check`: pass
- `cargo test --locked --all-targets`: pass, 81 tests
- `cargo clippy --all-targets --locked -- -D warnings`: pass
- `cargo run --release -- --benchmark`: pass, 0.43 ms at 5K; worst case in the
  catalog unchanged at Frosted Glass 4.25 ms
- `kb check`: pass, 0 errors and 0 warnings

## Findings

### Matches plan

- One shader function, one constant, one switch arm, one catalog row. No
  renderer, uniform, or pipeline changes.
- The lamp is fixed above the proscenium and the pool moves, so the beam angle
  changes as the spot searches — the behaviour of the real fixture rather than a
  cone rigidly attached to a circle.
- The flood expands from wherever the light is standing, and the reveal is
  visibly off-centre. It does not read as Projector Iris.
- Cheapest full-screen effect in the catalog, as predicted: closed-form shading
  with no neighbourhood sampling.

### Deviations

Both are recorded in the plan itself rather than only here.

- **The reveal chases the pool's edge instead of opening concentrically.** The
  planned centre-out reveal renders as an eclipse. Fixed by trailing the light's
  edge, closing the gap early, and perturbing the boundary with harmonics of the
  bearing.
- **The drift no longer settles as the exit begins.** Damping toward the base
  position would pull the pool to screen centre and defeat the off-centre
  reveal; freezing it in place needs the exit duration as a second cross-language
  constant. The motion involved is a few hundredths of a screen over 1.6 s.

### Potential issues

- A centre-out reveal cannot avoid an interval where the opening is a shape
  surrounded by light. That interval is now short and its boundary is irregular,
  but it is inherent to the geometry, and it is the thing to watch for on a real
  display.
- The rim harmonics are fixed in the bearing, so the opening's silhouette is the
  same shape every performance, merely rotated by the seed.
- The hold state is dark outside the pool and may lose its stage entirely on a
  low-contrast projector in a lit room, leaving a bright ellipse on grey.
- Live launch on Metal has not been done; every visual judgement here comes from
  offscreen renders.

## Manual Testing Required

- Run `--autostart=spotlight` on a presentation display and confirm the beam
  reads as a shaft rather than a gradient, and that the search reads as
  restrained rather than as a searchlight sweep.
- Dismiss once and watch the mid-exit interval specifically: confirm the opening
  reads as the screen returning, not as a hole in the light.
- Double-signal during the exit and confirm immediate removal.
- Toggle Reduce Motion and confirm a settled frame and cross-fade.

## Recommendations

If the mid-exit interval still reads as a hole on a real display, the next lever
is choreography rather than geometry: hold the flood a beat longer before the
reveal starts, so the light has filled more of the screen before anything opens
inside it. Changing the reveal's shape further is unlikely to help.

The frame-time note gained a section on reading a contended benchmark run, which
came out of this session: two runs here appeared to show a large regression in
the expensive effects, and a stash-and-remeasure A/B proved the machine was
contended rather than the code slower. That check is now written down.
