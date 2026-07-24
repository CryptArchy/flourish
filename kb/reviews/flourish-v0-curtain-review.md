---
id: flourish-v0-curtain-review
type: review
project: flourish
tags: [rust, shaders, macos, verification]
status: final
author: Christopher Andrews
created_date: 2026-07-18
upstream: [flourish-v0-curtain]
---

# Flourish V0 Curtain Review

## Implementation Status

**Partial pass.** The planned Rust vertical slice is implemented and the
automated suite passes. The real macOS tray process, Metal surface, fullscreen
window, graceful input path, natural completion, and hard-kill path ran. Visual
approval of the curtain and transparent reveal is still owed because macOS
denied automated screen capture. Windows/Linux CI is configured but has not run
in this unborn repository.

## Automated Verification Results

- `cargo fmt --all -- --check`: pass
- `cargo test`: pass, 6 tests
- `cargo clippy --all-targets -- -D warnings`: pass
- `cargo build --release`: pass
- Release binary: 4.5 MB on macOS arm64
- `kb check`: pass, 0 errors and 0 warnings

## Findings

### Matches plan

- Renderer-independent timeline implements idle, hold, graceful exit, natural
  completion, and second-signal immediate hiding.
- Hidden transparent winit window uses macOS simple fullscreen and Accessory
  activation policy, avoiding a Dock app and a new native fullscreen Space.
- wgpu 30 compiles one procedural WGSL curtain for Metal and negotiates a
  non-opaque compositor alpha mode.
- Native tray menu contains Curtain and Quit actions.
- Windows skip-taskbar behavior and macOS-specific behavior are `cfg`-isolated.
- README and CI distinguish implemented code from unverified platforms.

### Deviations

- Added a non-default `--autostart` development path for repeatable overlay
  launches during verification.
- The first effect is fragment-shader procedural cloth rather than the optional
  tessellated mesh fidelity pass described in research.

### Potential issues

- The procedural curtain may need art direction after human review; shader
  compilation and timing do not establish that it looks convincing.
- Wayland ignores winit's always-on-top window level. Focused borderless
  fullscreen behavior needs GNOME, KDE, and wlroots tests.
- Linux packaging needs GTK 3, xdo, and AppIndicator/Ayatana dependencies.
- Surface loss currently hides the effect with an error rather than rebuilding
  the GPU surface. That is safe for a presentation but can be improved.

## Manual Testing Required

- Confirm the closed curtain fully covers the display and looks like velvet.
- Confirm the opening reveals the live screen rather than black pixels.
- Judge rustle amplitude, lower-panel lag, edge motion, and 1.8-second timing.
- Exercise Quit through the native menu.
- Run the configured suite and the actual overlay on Windows 11 and Linux under
  GNOME Wayland, KDE Wayland, and one wlroots compositor.

## Recommendations

Keep the ticket active and the plan outcome pending until the macOS visual check
is accepted. If the shader reads as a flat wipe, move Curtain to a tessellated
two-panel mesh using the BSD-licensed Qt example's top/bottom-width spring model.
After visual approval, create the initial commit/remote so the platform matrix
can establish compile evidence before packaging work begins.
