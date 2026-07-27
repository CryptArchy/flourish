---
id: flourish-paper-tear-review
type: review
project: flourish
tags: [wgpu, shaders, visual-design, catalog, paper]
status: final
author: Christopher Andrews
created_date: 2026-07-26
upstream: [flourish-paper-tear]
---

# Flourish Paper Tear Review

## Implementation Status

**Accepted after two rounds of on-screen feedback and a rebuild.**
Paper Tear is registered, drawing, documented, and promoted out of the future
catalog, leaving Chalkboard and Elevator Doors in its remaining list. Every
automated check passes.

The first viewing found a ghost seam and two halves that looked like separate
sheets tearing at different times. The second viewing found the ghost still
there, still two stacked sheets, and — the decisive report — **no curl at all**.
That last one was correct and structural: the curl model was wrong, not
mistuned, and has been replaced. See Deviations.

## Automated Verification Results

- `cargo fmt --all -- --check`: pass
- `cargo test --locked --all-targets`: pass, 81 tests
- `cargo clippy --all-targets --locked -- -D warnings`: pass
- `cargo run --release -- --benchmark`: pass, 0.96 ms at 5K on the rebuilt
  curl, in a clean run where every other row reproduces baseline. Catalog worst
  case unchanged at Frosted Glass 4.25 ms. Two earlier attempts were severely
  contended — every row inflated three to five times and non-monotonic across
  resolutions, with a local LLM runtime holding the GPU — and nothing from them
  was recorded. The three-surface curl costs 0.05 ms more than the one-branch
  roll it replaced.
- `kb check`: pass

## Findings

### Matches plan

- One shader function plus two helpers, one constant, one switch arm, one
  catalog row. No renderer, uniform, or pipeline changes.
- The crack propagates downward and rows it has not reached stay joined, which
  is the difference between a tear and two panels sliding apart.
- The curl is the planned half-cylinder inverse. Screen offset `R(1 - cos t)`
  gives `t = acos(1 - b/R)`, the wrapped arc length `R*t` supplies the texture
  compression, and `t` doubles as the surface normal for shading. The flat part
  carries the `R(pi - 2)` offset, so the roll meets the sheet without a seam.
- Texture is sampled in sheet coordinates and compresses into the roll with the
  paper, rather than sliding under it.
- The gap shadow reads: the sheet sits above what it covers.

### Deviations

Four, of which the last three came from watching it move rather than from
reading stills. All are recorded in the plan.

- **The tear opens as a V.** Added during implementation; the planned gating
  produced a parallel gap.
- **A ghost seam ran down the sheet.** `roll` began at a nonzero radius the
  moment the crack passed a row, while `pull` had a dead zone until 5% of the
  exit, so both halves drew a curled ridge while still touching — a rolled seam
  printed on paper that had not moved. The curl is now gated on separation.
- **The halves read as two separate sheets.** The two rolls are mirror images
  and both were shaded from the same curve of `θ`, which puts the highlight in
  the same relative position on each — what two lamps would do, one per half.
  Now there is one light, upper-left, and the roll's actual normal
  `(-side·cos θ, sin θ)` is used; the cast shadows follow the same direction,
  falling only where that light would throw them.
- **The halves also read as tearing at different times.** The wedge equalized
  over the second half of the exit, so the bottom caught up as a distinct late
  motion. It is now constant in time and sized so the slowest row still clears.
  A related bug: the crack front overshot the bottom edge by less than the row
  smoothstep was wide, so the last rows never fully separated and a joined band
  survived along the bottom for the whole exit.

After the second viewing — "still ghost-lining, worse on the left; still a
double stack of paper; there is no roll or curl":

- **The curl model was wrong.** It rolled the torn edge *away* from the viewer,
  so only the front of a receding cylinder was ever drawn: a flat bright band
  beside a flat bright sheet, divided by a seam. That is a picture of two
  stacked sheets — the exact words the feedback used — and it is why shading
  fixes kept failing. A curl reads because the paper comes back *over* the
  sheet and shows its back face. Rebuilt that way, with three surfaces and a
  visible-surface test.
- **The ghost line was the tear's raggedness, not the roll.** Expressing rag as
  wrap angle keeps the roll's axis straight, but it invents a curl on rows with
  extra paper even when nothing has curled yet — a ragged line down a held
  sheet. The rag now sits in the tangent while flat and migrates into the wrap
  as the curl develops, so the hold is seamless.
- **The rates needed rebalancing.** A tangent recedes by its own arc length, so
  the curl opens the tear by itself; with the old translation the paper left
  the screen before the curl was ever visible. Translation is smaller now and
  the wrap is front-loaded, since below a half turn the flap is a crescent a
  few pixels wide and shows nothing.
- **The normals were in the wrong frame.** `q` runs opposite on the two halves,
  so a normal built from it needs the world-x sign or both rolls light
  identically — the same "two lamps" mistake as before, reintroduced in the new
  model. Caught by inspection rather than by viewing.
- **The sheet is larger than the screen.** The plan bounded it to the screen in
  both axes, which meant the sag exposed a strip along the top edge and needed
  its own shadow term to look deliberate. Treating the stock as oversized is
  what a cover sheet would be anyway, and it deleted a term rather than adding
  one.
- **Fibre needed two passes to stop reading as cloth.** Strongly stretched value
  noise lays down a lattice of rectangles; two of them crossing is visibly
  woven. Milder anisotropy and sparser thresholds fixed it. Worth remembering:
  the same trick that made a convincing wood-grain-like fibre in one direction
  becomes linen the moment it is crossed with itself.

### Potential issues

- The roll's silhouette ripples along its length, because the torn edge's
  fibre-scale noise varies per row and the roll follows it. It reads as
  hand-torn rather than as an artifact, but it is the first thing to look at if
  the curl seems wrong in motion.
- The roll is lit by a fixed analytic term with no relation to Spotlight's lamp
  or to any other effect's light direction. Nothing in the catalog shares a
  lighting convention; this only matters if one is ever introduced.
- The hold state is the brightest in the catalog. On a projector it will be the
  most visible effect in a lit room and the most aggressive against a dark
  slide deck.
- Live launch on Metal has not been done; every visual judgement here comes from
  offscreen renders.

## Manual Testing Required

- Run `--autostart=paper-tear` on a presentation display. Confirm the hold reads
  as paper rather than as a blank white screen — the failure mode the first
  implementation had exactly.
- Dismiss once and watch the crack: it should run from the top down with the
  gap widening behind it, not open along its whole length at once.
- Watch a torn edge specifically and confirm the grain compresses into the roll
  rather than sliding beneath it.
- Double-signal during the exit and confirm immediate removal.
- Toggle Reduce Motion and confirm a flat, settled sheet and a cross-fade.

## Recommendations

If the curl reads as too clean in motion, the lever is the roll radius growth
rather than the shading: real paper tightens its curl as it goes, and the radius
currently grows instead. That is a one-line change to the `roll` term and worth
trying before anything structural.

**The lesson worth carrying forward is that stills cannot review motion.** Every
symptom above survived a careful read of eight rendered frames and was obvious
within a second of watching the effect play. The offscreen harness is the right
tool for composition, colour, and proving alpha reaches zero; it cannot judge
whether two things look like one thing moving. Effects with more than one moving
part should get a real launch before they are called done.

**The sharper lesson from the second round is to check the model before tuning
it.** Two rounds went into shading and timing on a curl that could not have
looked like a curl at any setting, because it rolled the wrong way. "There is no
roll or curl" was a report about geometry; it was read as a report about
strength. When feedback says a thing does not read *as the thing*, suspect the
model, not the constants.

A second, narrower lesson: this catalog has no shared lighting convention, and
Paper Tear is the first effect where that cost something. Any future effect with
two symmetric halves inherits the same trap — mirror geometry shaded by a
mirror-symmetric function reads as two independently lit objects.

Chalkboard and Elevator Doors remain. Chalkboard is the one that needs a
decision before it can be planned: what the chalk diagrams actually are. Every
effect in the catalog is texture and motion; that one is the first that would
have to draw content.

*Resolved 2026-07-27: Chalkboard was declined for exactly that reason. See the
future catalog's Declined section.*
