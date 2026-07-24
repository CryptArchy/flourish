---
id: flourish-visual-feedback-tuning-review
type: review
project: flourish
tags: [wgpu, compute, shaders, macos, verification]
status: final
author: Christopher Andrews
created_date: 2026-07-18
upstream: [flourish-visual-feedback-tuning]
---

# Flourish Visual Feedback Tuning Review

## Implementation Status

**Partial pass pending visual confirmation.** The reported alpha-path defect
has a direct renderer fix, Water Drops has been replaced by stationary Pond
Ripples, procedural Fire has a more fractured edge, and Doom Fire is a distinct
stateful GPU automaton. All automated checks and a real Metal compute dispatch
pass. The user's eyes remain the acceptance test for motion and compositing.

## Automated Verification Results

- `cargo fmt --all -- --check`: pass
- `cargo test --locked`: pass, 11 tests
- `cargo clippy --all-targets --locked -- -D warnings`: pass
- `cargo build --release --locked`: pass
- `kb check`: pass, 0 errors and 0 warnings
- `git diff --check`: pass
- `cargo run --locked -- --autostart=doom-fire`: pass; native tray, fragment
  pipeline, compute pipeline, storage-buffer swap, and live dispatch ran on
  macOS/Metal
- Release binary: 4.5 MB on macOS arm64

## Findings

### Matches plan

- Surface negotiation now explicitly prefers `PreMultiplied` even if wgpu
  reports `PostMultiplied` first, and shader output always extinguishes RGB
  with alpha.
- Pond Ripples uses seven fixed impact points with independent expanding rings;
  exit reduces amplitude only, removing sheet translation and easing bounce.
- Fire combines coarse, fine, fractured, and column noise with a narrower edge
  transition for broken tongues.
- Doom Fire owns two resettable 256x128 integer heat buffers. A 30 Hz compute
  pass seeds the bottom row, propagates heat upward with randomized cooling and
  lateral sampling, and swaps source/destination buffers before rendering.
- Doom Fire uses a new black, blood-red, red, orange, yellow, and white-hot
  palette and remains separate from modern Fire in the native menu.
- `--autostart=<effect-slug>` was added as a non-default development path for
  real effect-specific smoke tests.

### Deviations

- The classic rule scatters each source cell to a randomized destination.
  Parallel writes would race on GPU, so this independent implementation gathers
  from one randomized lower neighbor per destination. It preserves bottom heat,
  upward propagation, random cooling, and lateral spread without nondeterministic
  storage writes.
- The compute field uses heat range 0-36 from the published description, but
  the palette is original and no PlayStation assets or source code are copied.

### Potential issues

- Premultiplied RGB under a post-multiplied-only compositor fallback may make
  translucent edges darken more quickly. That is preferable to residual color,
  but must be revisited during Windows/Linux runtime testing.
- Doom Fire reaches its mature height over several seconds by design; a very
  short flourish hold will show a smaller developing flame.
- Pond ripples overlay light and color but cannot physically refract the live
  presentation without screen-capture permission.

## Manual Testing Required

- Re-test Kaleidoscope and Mosaic exits and confirm the colored field vanishes
  with the shrinking geometry throughout the animation.
- Confirm Pond Ripples reads as independent impacts on a pond, with no shared
  vertical travel or end bounce.
- Compare Fire and Doom Fire: Fire should feel fluid but jagged; Doom Fire
  should read as a pixel heat automaton building upward from the screen bottom.
- Signal Doom Fire once and confirm its source cools while the overlay fades;
  signal twice and confirm immediate removal.

## Recommendations

Keep the tuning plan outcome pending until the user repeats the visual test.
If Doom Fire needs to establish faster, seed the bottom 2-3 rows or temporarily
advance multiple compute steps at start rather than changing the propagation
rule. If ripples are too busy, reduce impact count before reducing ring contrast.
