---
id: flourish-edge-halos-gravel-speed-review
type: review
project: flourish
tags: [wgpu, shaders, transparency, particles, visual-design, feedback]
status: final
author: Christopher Andrews
created_date: 2026-07-20
upstream: [flourish-edge-halos-gravel-speed]
---

# Flourish Edge Halos and Gravel Speed Review

## Implementation Status

**Automated pass; screenshot approval pending.** Fire and Frost no longer use
broad normalized semi-transparent leading bands. Gravel descends faster while
retaining its accepted 1,656-rock composition and dismissal behavior.

## Automated Verification Results

- `cargo fmt --check`: pass
- `cargo test`: pass, 17 tests
- `cargo clippy --all-targets -- -D warnings`: pass
- `cargo build --release`: pass
- `kb check`: pass, 0 errors and 0 warnings
- Revised Fire release launch on macOS/Metal: pass
- Revised Frosted Glass release launch on macOS/Metal: pass
- Revised Gravel Fall release launch on macOS/Metal: pass

## Findings

### Matches plan

- Fire's alpha transition changed from a `0.018–0.052` normalized feather to a
  resolution-aware 1.5–3.5 physical-pixel feather.
- Fire edge erosion is now confined to seven local feather widths rather than
  a `0.105`-deep normalized band. Low-alpha tails are sharpened.
- The immediate Fire silhouette borrows irregular warm body heat from its edge
  noise, preventing every boundary pixel from mapping to one dark ember color.
- Frost's former `0.160`-wide growth crossfade was replaced by a 1.5–3 pixel
  signed-distance frontier perturbed by coarse and fine fixed crystal noise.
- Frost's immediate frontier is porous and its low-alpha tail is sharpened;
  the accepted cellular ice, blooms, opacity, and melt implementation remain.
- Gravel hold gravity increased from `0.82` to `1.08`. Initial velocity changed
  from `0.035–0.145` to `0.050–0.190`; release gravity remains `2.35`.
- Full-settlement tests still retain all 1,656 stones onscreen and release every
  settled stone when the floor vanishes.

### Deviations

- Direct `screencapture` again failed with `could not create image from
  display`; visual confirmation cannot be automated under current macOS screen
  recording permissions.

### Potential issues

- Pixel-scale feathers are intentionally crisper. On a very low-DPI projector,
  the 1.5-pixel minimum may read slightly harder than on a Retina display.
- Fire's warmed boundary may need strength tuning if it reads as a bright rim;
  the two noise fields make it irregular rather than a uniform control line.
- Gravel's stronger hold gravity changes only descent feel, but final judgment
  of weight versus speed remains subjective.

## Manual Testing Required

- Run Fire over both white and black presentation content and confirm no dark
  stroke or translucent red control band sits outside the flames.
- Let Frost grow over mixed content and confirm the crystalline frontier does
  not carry a broad blue-gray leading line.
- Confirm both exits still clear gracefully and second-signal kills immediately.
- Confirm Gravel falls a little faster while retaining weight and the accepted
  final pile, then dismiss it after full settlement.

## Recommendations

Use the next real-display screenshot as the acceptance gate. If a residual Fire
rim remains, tune only the local `edge_heat`; if Frost still shows a band, tune
only frontier porosity and pixel feather. Do not restore normalized transition
widths or alter the accepted effect bodies.
