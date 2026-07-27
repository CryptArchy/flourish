---
id: flourish-surprise-and-frames
type: plan
project: flourish
tags: [cli, menu, tooling, wgpu]
status: final
outcome: shipped
author: Christopher Andrews
created_date: 2026-07-27
upstream: [flourish-elevator-doors, presentation-flourish-runner]
---

# Flourish Surprise Me and Contact Sheets

## Overview

Two changes around the catalog rather than in it, now that the catalog itself is
closed: a **Surprise Me** choice that makes the global shortcut a real "tada"
button, and a **`--frames`** flag that renders contact sheets offscreen so an
effect can be chosen without one seizing the screen.

## Current State Analysis

Seventeen effects ship. Two frictions have grown with the catalog:

**The shortcut replays one effect.** `toggle_via_hotkey` starts
`self.last_effect`, which begins as Curtain and only changes when a specific
effect is chosen from the menu. Pressing the shortcut repeatedly gives the same
flourish every time, which is the opposite of what a surprise should be — and
choosing from a seventeen-item menu mid-sentence is exactly the friction the
shortcut exists to avoid.

**Nothing shows an effect without playing it.** Deciding which flourish suits a
talk means launching each one full-screen. The offscreen path to do this better
already exists: `--benchmark` drives every effect through the real `Scene` at
five resolutions. It only throws the pixels away.

There is a second, quieter reason for `--frames`. Every visual change in the
last two days was reviewed by rebuilding a throwaway `examples/_preview.rs` —
four times in one session — because `Scene` lives in the binary crate and no
example can reach it. The Paper Tear review already records this as a follow-up.

## Desired End State

- A **Surprise Me** menu item, and `--autostart=random`, that play a flourish
  chosen at random and never the one just played.
- Choosing Surprise Me is *sticky*: the shortcut keeps rolling a new effect each
  press, while choosing a specific effect makes the shortcut replay that one.
- `flourish --frames [DIR]` writes one filmstrip PNG per flourish — the hold
  state and four points through its exit — composited over a stand-in desktop
  so both the reveal and the alpha are visible, then prints where they went.
- One source of entropy, shared by the per-performance shader seed and the
  random pick, rather than two hand-rolled mixers.

## What We're NOT Doing

- No weighting, no history beyond "not the last one", no favourites. A uniform
  draw that never immediately repeats is the whole feature.
- No labels or captions rendered into the contact sheets. Text means fonts; the
  CLI prints the filenames instead.
- No change to what a bare `--autostart` does. It stays Curtain.

## Implementation Approach

**Entropy moves into the library.** `renderer::fresh_seed` is a clock-plus-
counter mixer that already solves exactly the problem the picker has, including
the trap it documents: the wall clock's granularity is coarser than the gap
between two quick calls, so a counter is needed or back-to-back draws repeat. It
becomes `flourish::entropy::fresh_u32`, used by both the shader seed and the
picker, and gets the test it never had.

**`Choice` sits next to `Flourish`.** `Choice::Effect(Flourish)` or
`Choice::Surprise`, with `resolve(avoid)` returning a concrete flourish.
Threading a `Choice` rather than a `Flourish` through the CLI and `App` is what
makes stickiness fall out for free: the shortcut replays the *choice*, so a
sticky surprise needs no extra state or flag.

`Flourish::surprise(avoid)` draws uniformly from `ALL`, re-rolling once if it
lands on `avoid`. With seventeen effects a single re-roll leaves the
distribution near enough uniform, and it cannot loop.

*Revised during implementation:* a single re-roll can land on `avoid` again, and
returning it would break the one promise the feature makes. After two
collisions it steps to the next effect in the catalog, which is cheap, always
different, and cannot fail.

**`--frames` mirrors `--benchmark`.** A new `frames` module builds the same
offscreen `Scene`, advances each effect's simulation so Gravel and Doom Fire are
representative rather than empty, then captures the hold and four exit stages.
Tiles are 512x288 — 512 pixels is 2048 bytes per row, a multiple of the 256-byte
copy alignment, which is why that width rather than a rounder one. Frames are
composited over a synthetic desktop gradient on the CPU, since a transparent PNG
of a mostly-transparent exit frame shows nothing.

This makes `png` a real dependency rather than a dev-dependency. It is pure safe
Rust and only runs behind the flag; the alternative is writing an image format
by hand, which is worse.

## Phase 1: Surprise Me

### Changes Required

- `src/entropy.rs` in the library, with `fresh_u32`; `renderer.rs` uses it.
- `Choice` and `Flourish::surprise` in `lib.rs`.
- `cli.rs`: accept `--autostart=random`, thread `Option<Choice>`, and name
  `random` in both the catalog listing and the unknown-slug error.
- `main.rs`: `last_choice: Choice`, resolved at play time; a "Surprise Me" menu
  item above the effect list.
- README.

### Success Criteria

#### Automated Verification

- [x] `surprise` never returns the effect it was told to avoid, over many draws:
  200 draws against each of the seventeen as the previous effect.
- [x] `surprise` reaches every effect in the catalog over many draws.
- [x] `fresh_u32` returns different values on consecutive calls: 1,000 calls,
  1,000 distinct values, plus a spread check so a badly avalanched mixer cannot
  pass by clustering.
- [x] `--autostart=random` parses to the surprise choice; every slug still
  round-trips; an unknown slug still fails loudly.
- [x] fmt, clippy, and the full test suite pass — 95 tests, up from 81.

#### Manual Verification

- [ ] Choosing Surprise Me, then pressing the shortcut repeatedly, gives
  different effects.
- [ ] Choosing a specific effect, then pressing the shortcut, replays that one.

## Phase 2: Contact sheets

### Changes Required

- `png` promoted to a dependency in `Cargo.toml`.
- `src/frames.rs` with the offscreen capture and PNG writing.
- `cli.rs`: `--frames` and `--frames=DIR`, defaulting to `flourish-frames`.
- `main.rs`: dispatch, mirroring `--benchmark`.
- README.

### Success Criteria

#### Automated Verification

- [x] `--frames` and `--frames=DIR` parse, with the documented default; an
  empty `--frames=` is rejected rather than writing somewhere surprising.
- [x] The tile width keeps the copy row-alignment requirement, asserted in a
  test rather than left as a comment.
- [x] fmt, clippy, and the full test suite pass.

#### Manual Verification

- [x] Every flourish produces a filmstrip that reads as that effect.
- [x] Gravel Fall shows a built pile and Doom Fire a full field, not empty ones.
- [x] The last tile of each strip shows the stand-in desktop, unobscured.

## Testing Strategy

The picker and the CLI are ordinary unit-testable code and get real tests. The
capture path needs a GPU and is verified the way `--benchmark` is: by running it
and looking at what it produced.

## References

- `src/benchmark.rs`
- `src/renderer.rs`
- `src/cli.rs`
- `src/main.rs`
- `kb/reviews/flourish-paper-tear-review.md`
