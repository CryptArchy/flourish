---
id: flourish-gravel-frost-tuning-review
type: review
project: flourish
tags: [wgpu, shaders, particles, visual-design, feedback]
status: final
author: Christopher Andrews
created_date: 2026-07-20
upstream: [flourish-gravel-frost-tuning]
---

# Flourish Gravel and Frost Tuning Review

## Implementation Status

**Partial pass pending visual confirmation.** Gravel's true offscreen-settlement
bug is corrected, the requested 951-rock budget is implemented, and Frosted
Glass no longer uses periodic ridge fields. Automated checks, optimized build,
and live Metal initialization pass. The user directly accepted CRT Shutdown and
Geological Strata; they were intentionally unchanged.

## Automated Verification Results

- `cargo fmt --all -- --check`: pass
- `cargo test --locked`: pass, 17 tests
- `cargo clippy --locked --all-targets -- -D warnings`: pass
- `cargo check --locked`: pass
- `cargo build --release --locked`: pass
- `kb check`: pass, 0 errors and 0 warnings
- Real revised Frosted Glass launch on macOS/Metal: pass
- Real 951-rock Gravel Fall launch on macOS/Metal: pass

## Findings

### Matches plan

- Gravel's total budget is derived from 9 boulders, 36 large rocks, 210 medium
  rocks, and 696 small rocks: 951 instances.
- Diagnosis proved the apparent disappearance was not the viewport filter. The
  full-footprint collision used the highest heightfield sample, allowing a
  narrow peak to bridge later rocks upward until many settled entirely above
  the screen.
- Collision now uses the average local pile surface. This compacts each
  independent depth plane and avoids runaway towers while preserving its own
  rounded heightfield.
- A full-settlement regression asserts all 951 rocks are settled, remain in the
  instance set, and intersect the viewport before dismissal.
- The instance buffer derives from `MAX_ROCKS`, so the GPU allocation grows with
  the CPU budget automatically.
- Frost's three periodic sine ridge systems were removed completely.
- The replacement uses two jittered nearest/second-nearest cellular edge fields,
  low-frequency domain warping, and five locally bounded crystal blooms.
- Frost hold alpha now spans approximately 78–98%, up from 56–90%. Melt and
  final-clear alpha remain premultiplied and unchanged in lifecycle semantics.
- CRT Shutdown and Geological Strata were left untouched after explicit user
  acceptance.

### Deviations

- The retention test exposed an actual stacking defect beyond the initially
  suspected cross-plane occlusion. The implementation therefore changes the
  collision sample from peak to average surface, an in-scope correction that
  was not known when the plan was written.

### Potential issues

- Average-surface settlement deliberately allows modest silhouette overlap
  within a layer. This reads as packed aggregate but is not rigid-body contact.
- Doubling the small layer extends the full arrival sequence by roughly five
  seconds; the effect still responds to dismissal at any time.
- Cellular ice boundaries can still produce closed polygonal cells, but they no
  longer share a rectangular grid or global directional axis.
- Live launches validate shader compilation and hold stability, not subjective
  density, opacity, or the clicked melt sequence.

## Manual Testing Required

- Let all Gravel layers settle and confirm no batch appears to lose rocks above
  the screen, the larger budget feels appropriately dense, and the compacted
  overlap still looks like gravel.
- Dismiss Gravel after full settlement and confirm all layers fall away.
- Let Frost fully grow and inspect for visible control lines, square grids, or
  global repetition. Confirm the pane feels materially more opaque.
- Dismiss Frost and confirm the organic melt fronts reveal the screen without a
  residual colored layer; double-signal to confirm immediate kill.

## Recommendations

Keep the new Gravel retention invariant. Further density tuning should change
counts or spawn timing, not restore peak-based collision. For Frost, tune
cellular scale, bloom contrast, or opacity from screenshots; do not reintroduce
screen-wide periodic ridge fields.
