---
id: flourish-visual-feedback-tuning-round-2-review
type: review
project: flourish
tags: [wgpu, shaders, macos, visual-tuning]
status: final
author: Christopher Andrews
created_date: 2026-07-18
upstream: [flourish-visual-feedback-tuning]
---

# Flourish Visual Feedback Tuning Round 2 Review

## Implementation Status

**Partial pass pending visual confirmation.** The preferred original Fire is
restored, Doom Fire now derives a square-to-portrait cell grid from display
aspect, and Curtain's lighting and lower trim have been rebuilt from the second
round of direct visual feedback. Automated verification and real Metal launches
pass; art-direction acceptance remains manual.

## Automated Verification Results

- `cargo fmt --all -- --check`: pass
- `cargo test --locked`: pass, 11 tests
- `cargo clippy --all-targets --locked -- -D warnings`: pass
- `cargo build --release --locked`: pass, 4.5 MB arm64 binary
- `kb check`: pass, 0 errors and 0 warnings
- Real Curtain launch on macOS/Metal: pass
- Real Doom Fire compute/render launch on macOS/Metal: pass
- `git diff --check`: pass

## Findings

### Matches plan

- Fire's original `7x` coarse and `19x` fine value-noise fields, height mix,
  and broad `0.035` edge transition are restored exactly.
- Doom Fire's backing field grows from 256x128 to 640x144. Rendering chooses a
  centered visible column count from screen aspect multiplied by `1.12`, making
  cells about 12% taller than wide without distorting the automaton.
- Curtain reconstructs a rest-space horizontal cloth coordinate from each
  compressed panel, adds fold displacement, and evaluates three soft radial
  pools in that fabric coordinate.
- Light is now part of the velvet base before fold and shadow modulation rather
  than three screen-space additive cone beams.
- Unlit Curtain colors move much closer to black; the former top lamp dots are
  removed.
- The lower gold band begins near the last 5.6% of screen height and remains
  solid through the bottom edge instead of fading back to red.

### Deviations

- The request allowed square or taller cells; this pass deliberately chooses a
  subtle 1.12 portrait factor because it preserves the vertical rise of the
  flame while avoiding conspicuously tall pixels.

### Potential issues

- Three overlapping radial pools may still feel too evenly spaced; visual
  review should decide whether a single dominant pool is more theatrical.
- The darker curtain may need a small brightness adjustment on dim projectors,
  but the gold trim remains a strong legibility anchor.
- Doom Fire's wider backing field increases its two-buffer allocation from
  roughly 256 KB to roughly 720 KB, still negligible for a GPU utility.

## Manual Testing Required

- Confirm Fire again matches the original preferred version.
- Confirm Doom Fire cells look square-to-tall on the actual presentation
  display, including an external display if one is used for meetings.
- Inspect Curtain at hold: most velvet should feel theater-dark and each soft
  light pool should visibly deform across folds rather than float above them.
- Inspect the final screen row and confirm gold reaches the physical bottom with
  no red strip.
- Open Curtain and ensure the fabric-space pools compress naturally with each
  gathered panel.

## Recommendations

Keep the plan pending until this visual pass is accepted. If the lights still
feel synthetic, reduce from three pools to one off-center key light before
adding more lighting structure.
