---
id: flourish-future-catalog
type: ticket
project: flourish
tags: [catalog, visual-design, presentations, backlog]
status: final
step: done
author: Christopher Andrews
created_date: 2026-07-18
closed_date: 2026-07-27
source: file
closed_reason: All six concepts resolved: five promoted and shipped, Chalkboard declined
---

# Flourish Future Catalog

## Description

Preserve the remaining approved follow-up Flourish concepts so each can graduate
into a visual plan and implementation. **All of them have now been resolved:**
five promoted and shipped, one declined. See Promoted and Declined.

## Context

The user explicitly approved the full proposed set and asked not to lose the
remaining ideas while the first two are built.

## Requirements

### Functional

- Nothing remains. Every concept is either promoted or declined below.
- Every concept must define an intentional hold state, graceful exit, and
  second-signal immediate kill before implementation.

### Declined

- **Chalkboard** will not be built (2026-07-27). It does not fit the catalog.
  Every other flourish is texture and motion over an abstract field; Chalkboard
  is the only one that would have to draw *content* — legible diagrams — and a
  presentation overlay that draws its own diagrams competes with the deck it is
  punctuating rather than punctuating it. The concept is recorded here rather
  than deleted, so it does not get re-proposed as a fresh idea.

### Promoted

- **CRT Shutdown** and **Frosted Glass** were promoted into
  `flourish-frosted-crt-depth-layers` on 2026-07-20.
- **Spotlight** was promoted into `flourish-spotlight` on 2026-07-26. It expands
  from wherever the light is standing rather than from the centre, which is what
  keeps it distinct from Projector Iris.
- **Paper Tear** was promoted into `flourish-paper-tear` on 2026-07-26. Its
  crack propagates downward so the tear opens as a V, which is what separates it
  from Geological Strata's clean shear of a rigid material.
- **Elevator Doors** was promoted into `flourish-elevator-doors` on 2026-07-27.
  It parts from the centre as Curtain does, and earns the overlap by being its
  opposite in every respect that carries the movement: rigid against draped,
  one driven motion profile against fabric lag, specular against diffuse.

### Non-functional

- Preserve Flourish's small local/offline runtime and transparent reveal.
- Prefer procedural rendering over bundled video or texture assets.
- Give each effect one signature movement rather than generic ambient motion.

## Research hints

Paper fracture shaders, polar melt masks, CRT phosphor collapse, signed-distance
chalk strokes, Voronoi ice growth, brushed-metal anisotropy, elevator door
parallax.

## Success Criteria

### Automated Verification

- [x] Each promoted concept receives a finalized plan with catalog metadata,
  shader validation, and lifecycle coverage. Four plans cover the five:
  `flourish-frosted-crt-depth-layers` (CRT Shutdown, Frosted Glass),
  `flourish-spotlight`, `flourish-paper-tear`, `flourish-elevator-doors`.

### Manual Verification

- [x] Each effect remains distinct and useful as presentation punctuation. The
  one genuine question was Elevator Doors against Curtain, both parting from the
  centre; viewed on 2026-07-27 and they read as different effects.
