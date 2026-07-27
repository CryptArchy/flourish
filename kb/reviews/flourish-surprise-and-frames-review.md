---
id: flourish-surprise-and-frames-review
type: review
project: flourish
tags: [cli, menu, tooling, wgpu]
status: final
author: Christopher Andrews
created_date: 2026-07-27
upstream: [flourish-surprise-and-frames]
---

# Flourish Surprise Me and Contact Sheets Review

## Implementation Status

**Complete and accepted.** Both changes are in, the test count went from 81 to
95, `--frames` has been run against the whole catalog with its output inspected,
and the two app-level checks passed on 2026-07-27.

Stickiness behaved as designed in use: the shortcut is a repeat button or a
surprise button depending on the last thing chosen, and switching between the
two needs nothing but choosing differently.

## Automated Verification Results

- `cargo fmt --all -- --check`: pass
- `cargo test --locked --all-targets`: pass, 95 tests, up from 81
- `cargo clippy --all-targets --locked -- -D warnings`: pass
- `cargo run --release -- --frames`: 17 filmstrips written and inspected
- `kb check`: pass

## Findings

### Matches plan

- Entropy is one library module, used by both the shader seed and the picker,
  and it now has the test the renderer's private version never had — a thousand
  consecutive calls, all distinct.
- `Choice` threads from the CLI through `App`, and stickiness fell out of it as
  predicted: `play_choice` stores the choice, `start_effect` stores only what it
  played, and the shortcut resolves the choice afresh on each press. No flag, no
  mode, no extra state.
- The picker never repeats the previous effect and reaches all seventeen, both
  asserted over thousands of draws.
- `--frames` renders through the real `Scene`, and the settle pass does its job:
  Gravel Fall's strip shows a built pile and Doom Fire's a full field, which
  were the two effects that would otherwise have previewed as empty floors.
- The tile width's copy-alignment constraint is a test rather than a comment.

### Deviations

- **`Scene` gained a `queue()` accessor.** Reading a frame back means submitting
  a copy alongside the draw, and the queue was private. It mirrors the existing
  `device()` and is the smallest opening that works.
- **The picker steps to the next effect after two collisions.** The plan said
  "re-roll once"; a single re-roll can land on the avoided effect again, and
  returning it would break the one promise the feature makes. Stepping is cheap,
  always different, and cannot fail.
- **`png` moved from dev-dependencies to dependencies**, as the plan expected.
  It is the only new dependency the binary takes on in this work.

### Potential issues

- Gravel Fall's preview pile is built for a 512-pixel-wide frame, so it reads
  denser in the strip than it does full-screen. The strip is honest about the
  effect but not about its scale.
- The strips are 2560x288 PNGs, roughly 100–400 KB each. Seventeen of them is a
  few megabytes per run, written to the working directory by default, and
  nothing cleans them up.
- `--frames` takes about a minute for the catalog, most of it the 900-frame
  settle per effect. There is no progress output beyond one line per finished
  flourish.
- The draw is uniform over the catalog, so a presenter who wants "any of the
  theatrical ones" still cannot say that. Deliberate: the plan ruled out
  favourites and weighting.

## Manual Testing Required

- ~~Choose Surprise Me, then press the shortcut several times.~~ Done: a
  different flourish each press.
- ~~Choose a named effect, then press the shortcut.~~ Done: that effect replays.
- Confirm the menu reads well with Surprise Me above the separator. Not
  separately checked; the item was used from the menu, so it is at least
  findable.

## Recommendations

The throwaway `examples/_preview.rs` this work was partly meant to retire is now
redundant for whole-effect review — `--frames` covers it. What it does *not*
cover is authoring a single effect at specific exit progresses, which is what
the harness was rebuilt four times to do. If a future effect needs that again,
the cheap addition is `--only=<slug>` alongside a configurable stage list,
rather than another throwaway example.

The frame-time note is deliberately untouched: nothing in the drawing path
changed, and re-measuring would only add a run's worth of noise to a table that
is currently clean.
