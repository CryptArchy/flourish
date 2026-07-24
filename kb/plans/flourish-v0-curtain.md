---
id: flourish-v0-curtain
type: plan
project: flourish
tags: [rust, shaders, desktop, macos]
status: final
outcome: pending
author: Christopher Andrews
created_date: 2026-07-18
upstream: [presentation-flourish-runner, 2026-07-18_existing-flourish-tools]
---

# Flourish V0 Curtain

## Overview

Deliver the first runnable vertical slice of Flourish: a menu-bar utility with
a Curtain menu item, a transparent full-screen procedural curtain, graceful
open-on-input behavior, double-signal immediate dismissal, and automatic return
to an idle background state.

## Current State Analysis

The repository is empty except for its Commonplace KB and has no commits or
remote. Research found adjacent tools but no existing project with the required
product lifecycle. The local machine has Rust 1.96 and Cargo 1.96; CMake is not
installed, which further favors a Cargo-native stack.

## Desired End State

- `cargo run` starts a menu-bar-only utility on macOS.
- The tray menu contains `Curtain` and `Quit`.
- `Curtain` shows a closed, subtly animated red curtain over the current screen.
- One mouse/key signal starts a center-opening exit; a second hides immediately.
- Natural completion hides the overlay and leaves the utility running.
- Core lifecycle behavior is unit tested.
- The code uses cross-platform crates and platform conditionals deliberately,
  but only macOS is claimed manually verified in this phase.

## What We're NOT Doing

- Signed/notarized installers, autostart, auto-update, or App Store packaging.
- A downloadable shader marketplace or arbitrary untrusted shader loading.
- Multi-monitor selection UI.
- Global hotkeys or presentation-software integrations.
- Claiming Windows/Linux shipped without real-machine verification.
- Photorealistic cloth simulation; the first curtain is a procedural visual
  proof with a clear path to a tessellated mesh fidelity pass.

## Implementation Approach

Use one Rust binary. `winit` owns the application and overlay event loop;
`tray-icon` owns the native menu; `wgpu` renders a full-screen WGSL shader to a
transparent surface. Keep lifecycle decisions in a renderer-independent state
machine so hard-kill semantics can be tested without a GUI.

Risk gate: reversibility **low**, surface sensitivity **medium** (full-screen
presentation behavior), blast radius **low**, ambiguity **medium** (visual
quality), scale **low**. Overall **medium**: proceed with a macOS-only verified
vertical slice and explicitly flag cross-platform/manual gaps.

## Phase 1: Project and lifecycle foundation

### Changes Required

- Add Cargo metadata, formatting/lint configuration, and a short project README.
- Implement `FlourishState` and signal handling independently of window code.
- Define hold, exit, hard-kill, and natural-completion behavior.

### Success Criteria

#### Automated Verification

- [x] `cargo test` covers first signal, repeated first-state signals, second
  signal, and natural completion.
- [x] `cargo fmt --check` passes.

#### Manual Verification

- [x] State names and timing match the pitch.

## Phase 2: Transparent curtain renderer

### Changes Required

- Create one hidden, transparent, undecorated overlay window.
- Configure a non-opaque wgpu surface and render a procedural WGSL curtain.
- Animate a subtle hold-state rustle and a center-opening exit.
- Hide the window when the lifecycle completes.

### Success Criteria

#### Automated Verification

- [x] `cargo check` and `cargo clippy --all-targets -- -D warnings` pass.
- [x] Shader compilation succeeds during renderer initialization.

#### Manual Verification

- [ ] Transparent pixels reveal the live screen instead of black.
- [x] The curtain covers the display without a native Space transition.
- [ ] Rendering is smooth and visually legible on a Retina display.

## Phase 3: Tray and input integration

### Changes Required

- Add a template-style tray icon and menu with Curtain and Quit items.
- Route menu selection, key presses, and clicks through the lifecycle.
- Keep the utility alive after the overlay naturally completes.

### Success Criteria

#### Automated Verification

- [x] The full crate test, lint, and format suite passes.

#### Manual Verification

- [x] Curtain launches from the macOS menu bar with no Dock presence.
- [x] One signal opens; a second signal during opening kills immediately.
- [ ] Quit exits cleanly from idle and active states.

## Phase 4: Cross-platform compile boundary and documentation

### Changes Required

- Keep Windows taskbar suppression and macOS activation/fullscreen behavior in
  small `cfg`-gated helpers.
- Document Linux tray packages and the unverified Wayland stacking risk.
- Add CI compile/check jobs where hosted runners can exercise them later.

### Success Criteria

#### Automated Verification

- [x] Host-native checks pass and platform-specific code is isolated.

#### Manual Verification

- [x] The README distinguishes implemented foundation, macOS verification, and
  remaining Windows/Linux validation.

## Testing Strategy

Run unit tests for state transitions; format, check, and lint the full crate;
launch the actual utility on macOS and visually test the shader, transparency,
focus, natural completion, double-signal kill, relaunch, and quit. Do not infer
Windows or Linux runtime success from a macOS build.

## References

- `kb/research/2026-07-18_existing-flourish-tools.md`
- `kb/tickets/presentation-flourish-runner.md`

## Deviations from Plan

- macOS denied automated screen capture, so the live shader's visual quality
  and the appearance of transparent pixels remain a human-visible check even
  though Metal accepted a non-opaque surface and the real window ran.
- A `--autostart` development flag was added to exercise the actual overlay
  without automating the menu bar. It does not change normal startup behavior.
- Cross-platform CI configuration is present, but it has not run because this
  repository has no remote or initial commit yet.
