---
id: 2026-07-18_doom-fire-and-alpha
type: research
project: flourish
tags: [wgpu, alpha-compositing, cellular-automata, shaders]
status: final
author: Christopher Andrews
created_date: 2026-07-18
upstream: [presentation-flourish-runner, flourish-effect-catalog-review]
---

# Doom Fire and Overlay Alpha

## Question

Why do Kaleidoscope and Mosaic leave their primary colors visible while their
exit masks shrink, what motion model should replace Water Drops, and what is
required to implement the recognizable PlayStation Doom fire effect rather
than a similarly colored procedural flame?

## Summary

The color persistence is an alpha-convention mismatch made visible by the two
opaque effects: the surface selects the first reported pre- or post-multiplied
mode, while the shader only premultiplies RGB for the former. The observed
macOS output shows that fading alpha without simultaneously extinguishing RGB
is unsafe for this overlay. Prefer `PreMultiplied` explicitly and always make
exit geometry return RGB scaled by the same alpha.

Water Drops is the wrong model. Its entire sampling coordinate moves vertically
on exit, so independent droplets necessarily travel as one sheet. Replace it
with stationary, aspect-correct ripple origins whose concentric wavefronts
expand locally and dissipate in place.

Authentic Doom Fire needs state. A fragment-only function cannot propagate the
previous frame's temperatures upward. Use two small storage buffers, a compute
pass that reads one and writes the other, and swap them each frame; render the
result through a discrete heat palette.

## Detailed Findings

### Overlay alpha

- `src/renderer.rs:90-101` accepts whichever compatible alpha mode appears
  first instead of explicitly preferring `PreMultiplied`.
- `src/shaders/flourishes.wgsl:56-60` leaves RGB unscaled under
  `PostMultiplied`. Kaleidoscope and Mosaic animate alpha over a full-color
  field, matching the reported visible-color / disappearing-underlayer split.
- A safe macOS path is to prefer premultiplied composition and guarantee
  `rgb = color * alpha` for transparent output. Fully opaque pixels are
  unchanged.

### Pond ripples

- `src/shaders/flourishes.wgsl:134-162` translates one global sample coordinate
  by exit progress. That explains common upward motion and the easing reversal
  perceived as a bounce.
- Pond ripples should use fixed impact centers, expanding ring radii, decaying
  amplitude, and an exit that only damps amplitude.

### PlayStation Doom fire

Fabien Sanglard's reverse-engineering account describes a framebuffer of heat
indices, black at zero and white-hot at the maximum, initialized with a hot
bottom row. Each update propagates heat upward, randomly cools it, and randomly
spreads it left or right. The visual character comes from the combination of
that automaton and its black/red/orange/yellow/white palette, not a continuous
flame equation.

Source: <https://fabiensanglard.net/doom_fire_psx/> (core algorithm and cleaned
reference implementation, accessed 2026-07-18).

## Code / Document References

- `src/renderer.rs:90-124`
- `src/shaders/flourishes.wgsl:56-60`
- `src/shaders/flourishes.wgsl:134-192`
- `kb/reviews/flourish-effect-catalog-review.md`
- <https://fabiensanglard.net/doom_fire_psx/>

## Open Questions

- Human review must choose the most comfortable ripple count and Doom Fire
  height for screen sharing; shader validity cannot settle intensity.
- Post-multiplied-only platforms may require a separate fallback after
  Windows/Linux runtime tests. macOS should use the explicitly preferred
  premultiplied path.
