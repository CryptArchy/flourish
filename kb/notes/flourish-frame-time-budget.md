---
id: flourish-frame-time-budget
type: note
project: flourish
tags: [performance, wgpu, shaders, gpu, measurement]
status: final
author: Christopher Andrews
created_date: 2026-07-26
---

# Flourish Frame-Time Budget

Closes the last open performance question from the code review: Flourish asks
for the **low-power** adapter, and several flourishes are heavy per-pixel
shaders. Could one miss frame budget on a presentation machine that is also
driving a projector and running a deck?

**Measured answer: no, with roughly 2x headroom at the worst point — on this
class of hardware.** The caveat matters; see below.

## Method

`flourish --benchmark` renders every flourish offscreen through `Scene`, the
same drawing path the on-screen renderer uses, so the numbers describe shipped
code rather than a reimplementation. Simulations are advanced 25 simulated
seconds at a cheap 640x360 before measuring, so Gravel Fall is timed with a
full 1,656-stone pile and Doom Fire with heat at the top of its field. Each
measurement submits 40 frames back to back and waits once, giving sustained
throughput rather than per-frame latency.

Run it in **release**; a debug build measures the wrong thing.

## Results, 2026-07-26

Apple M5 Max (integrated, Metal). Sustained milliseconds per frame.

| Flourish | 1080p | 1440p | MBP 16" | 4K | 5K |
| --- | ---: | ---: | ---: | ---: | ---: |
| Curtain | 0.08 | 0.13 | 0.25 | 0.27 | 0.47 |
| Confetti | 0.46 | 0.77 | 1.58 | 1.73 | 2.99 |
| Marquee Bulbs | 0.41 | 0.71 | 1.44 | 1.55 | 2.73 |
| Spotlight | 0.08 | 0.13 | 0.23 | 0.25 | 0.43 |
| Projector Iris | 0.06 | 0.09 | 0.18 | 0.19 | 0.33 |
| Elevator Doors | 0.08 | 0.12 | 0.24 | 0.25 | 0.43 |
| Geological Strata | 0.12 | 0.20 | 0.39 | 0.42 | 0.73 |
| Paper Tear | 0.15 | 0.25 | 0.51 | 0.55 | 0.96 |
| **Frosted Glass** | **0.63** | **1.09** | **2.25** | **2.42** | **4.36** |
| CRT Shutdown | 0.05 | 0.07 | 0.13 | 0.13 | 0.23 |
| Pond Ripples | 0.15 | 0.24 | 0.48 | 0.51 | 0.89 |
| Fire | 0.11 | 0.17 | 0.33 | 0.36 | 0.62 |
| Doom Fire | 0.06 | 0.05 | 0.10 | 0.10 | 0.16 |
| Gravel Fall | 0.06 | 0.07 | 0.07 | 0.08 | 0.10 |
| Constellation | 0.50 | 0.87 | 1.77 | 1.90 | 3.37 |
| Blackout | 0.05 | 0.06 | 0.07 | 0.08 | 0.13 |
| Kaleidoscope | 0.07 | 0.09 | 0.18 | 0.19 | 0.32 |
| Mosaic | 0.05 | 0.08 | 0.14 | 0.15 | 0.25 |

Re-measured in one run as Marquee Bulbs, Constellation, Spotlight, Paper Tear,
Elevator Doors, and Confetti landed.
Every row is from that one run; none are carried over. The pre-existing rows
reproduce their previous values to within a few hundredths of a millisecond —
Frosted Glass is the largest mover at 0.11 ms, about 2.5%, which is ordinary
run-to-run variation for the most expensive shader in the catalog. That
agreement is the check that the catalog's growth costs the effects already in
it nothing.

Budget is 16.67 ms at 60Hz and 8.33 ms at 120Hz. Worst case is Frosted Glass at
5K, 4.36 ms — about half the 120Hz budget.

## What the numbers say

**The review named the right effect.** Frosted Glass costs 7x to 50x more than
anything else, which is what two nine-sample Voronoi passes plus five
domain-warped blooms per pixel buys. It was simply nowhere near expensive
enough to matter on this GPU.

**The neighbourhood effects are next, and not close.** Constellation, Confetti,
and Marquee Bulbs each read a 3x3 cell neighbourhood per pixel — Confetti reads
two, one per depth layer — which puts them second, third, and fourth at 3.37,
2.99, and 2.73 ms at 5K. All still under Frosted Glass and inside the 120Hz
budget. A per-pixel neighbourhood loop is the shape of shader that lands in this
band; anything wider should be measured before it is added.

Confetti is the useful data point on how much a second layer costs: two 3x3
loops of pure arithmetic land near one 3x3 loop that also samples noise. The
loops are not what is expensive.

**What an effect looks like predicts nothing; what it samples predicts
everything.** Spotlight fills the screen with light, haze, and a moving beam and
costs 0.43 ms at 5K — a tenth of Frosted Glass — because every one of those is a
closed-form function of the pixel's position. The expensive band is populated
entirely by effects that sample a neighbourhood per pixel.

**Paper Tear is the useful counter-check at 0.96 ms.** It is the most elaborate
mapping in the catalog — a page curl, resolved per pixel against three possible
surfaces — and it costs a fifth of Frosted Glass, because arithmetic per pixel
is nearly free next to sampling. Its cost is the six noise lookups its paper
stock needs, not its geometry: rebuilding that geometry from a one-branch roll
into a three-surface curl moved it by 0.05 ms. Predicting a new effect's band
means counting its samples, not admiring its motion.

**Cost is almost perfectly pixel-bound.** 5K is 7.1x the pixels of 1080p, and
Frosted Glass costs 6.9x more there. That near-exact linearity is also the
check that the harness is measuring real work rather than an empty pass.

**Gravel Fall is nearly free** despite 1,656 instances and a CPU simulation per
frame — around 45,000 triangles is nothing. The sub-0.1 ms entries are probably
at the floor of command-submission overhead rather than measuring GPU work.

**No half-resolution path is warranted.** The review floated rendering the
expensive effects at half resolution if things were tight. They are not, and
adding that machinery now would be optimising against a measurement that says
not to.

## The caveat

`IntegratedGpu` here means an Apple M5 Max, which is a fast integrated GPU. An
older Intel Mac on Iris or UHD graphics can be an order of magnitude slower,
which would put Frosted Glass at 4K/5K at or over the 60Hz budget. The
conclusion is "fine on Apple Silicon", not "fine everywhere".

Re-run `--benchmark` on any machine that will actually present, especially
anything Intel. The verdict line reports which budget was missed if one is.

## Follow-ups if a slow machine ever shows up

- Frosted Glass is the only realistic candidate for a half-resolution pass; the
  rest have too much headroom to be worth complicating.
- The cheapest win would be reducing the frost's bloom count or Voronoi sample
  count rather than changing render resolution, since the cost is per-pixel
  shader work rather than bandwidth.

## Reading a contended run

Two runs during the Spotlight session reported Frosted Glass at 5.44 and 6.46 ms
while Curtain, Projector Iris, and Blackout reproduced their usual numbers
exactly. That pattern looks like a regression in the expensive shaders and is
not one. Stashing the session's changes and re-running gave Frosted Glass 4.25
on the same machine minutes later; restoring them gave 4.26.

**Interference is cost-proportional, so it hides in the cheap rows.** Anything
else competing for the GPU steals a share of real work, and the sub-0.3 ms
effects are dominated by command-submission overhead rather than by that work,
so they barely move while the heavy ones inflate 50%. "The cheap effects are
unchanged" is therefore *not* evidence that a run is clean.

The check that settles it costs two runs: stash, measure, restore, measure. Do
that before believing any regression this table appears to show.

## Measurement boundary

Offscreen, with no compositor and no vsync, so this is drawing cost alone. A
real frame additionally pays presentation, which is bounded by the display's
refresh rather than by this work.
