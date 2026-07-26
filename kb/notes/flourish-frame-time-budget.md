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
| Projector Iris | 0.06 | 0.09 | 0.18 | 0.19 | 0.33 |
| Geological Strata | 0.12 | 0.20 | 0.39 | 0.42 | 0.73 |
| **Frosted Glass** | **0.63** | **1.10** | **2.27** | **2.43** | **4.32** |
| CRT Shutdown | 0.05 | 0.06 | 0.12 | 0.13 | 0.22 |
| Pond Ripples | 0.14 | 0.23 | 0.46 | 0.49 | 0.87 |
| Fire | 0.10 | 0.17 | 0.33 | 0.36 | 0.62 |
| Doom Fire | 0.04 | 0.05 | 0.09 | 0.10 | 0.16 |
| Gravel Fall | 0.04 | 0.04 | 0.06 | 0.06 | 0.10 |
| Blackout | 0.03 | 0.04 | 0.07 | 0.07 | 0.12 |
| Kaleidoscope | 0.06 | 0.09 | 0.17 | 0.18 | 0.31 |
| Mosaic | 0.05 | 0.07 | 0.14 | 0.15 | 0.25 |

Budget is 16.67 ms at 60Hz and 8.33 ms at 120Hz. Worst case is Frosted Glass at
5K, 4.32 ms — about half the 120Hz budget.

## What the numbers say

**The review named the right effect.** Frosted Glass costs 7x to 50x more than
anything else, which is what two nine-sample Voronoi passes plus five
domain-warped blooms per pixel buys. It was simply nowhere near expensive
enough to matter on this GPU.

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

## Measurement boundary

Offscreen, with no compositor and no vsync, so this is drawing cost alone. A
real frame additionally pays presentation, which is bounded by the display's
refresh rather than by this work.
