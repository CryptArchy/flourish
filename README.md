# Flourish

Flourish is a lightweight desktop utility for adding a little theatrical
punctuation to presentations. Pick an effect from the menu bar, let it own the
screen for a moment, then dismiss it with any click or key.

The signature effect is **Curtain**: dark oxblood velvet with antique-gold trim
and warm stage lights. It rustles while it waits, then draws open from the
center to reveal the live screen underneath. A second signal during any
flourish's exit removes it immediately.

## Flourishes

| Flourish | While it waits | Graceful exit |
| --- | --- | --- |
| Curtain | Lit velvet subtly rustles | Draws open from the center |
| Projector Iris | Soot-black overlapping blades hold around a tungsten pinhole | The mechanical aperture spirals open |
| Geological Strata | Textured sediment bands form a dramatic road cut | A crooked fault opens and both land masses shear away |
| Frosted Glass | Dendritic ice creeps over a translucent pane | Irregular warm fronts melt through the frost |
| CRT Shutdown | Phosphor scanlines and restrained analog noise fill a dark tube | The image collapses to a line, contracts to a dot, and blinks out |
| Pond Ripples | Independent concentric ripples cross a calm surface | Dissipates in place |
| Fire | Fluid procedural flames and embers lick upward | Gutters down to nothing |
| Doom Fire | Pixel heat propagates through a PSX-inspired automaton | Source cools and the field fades |
| Gravel Fall | Faceted stones tumble down and build a natural pile | The floor vanishes and the whole pile drops away |
| Blackout | A clean, nearly pure black screen | Diagonal wipe reveal |
| Kaleidoscope | Jewel-toned mirrored facets turn | Radial aperture reveal |
| Mosaic | Colored beveled tiles drift by row | Tiles shrink away in sequence |

## Current status

This repository contains an expanded macOS vertical slice:

- Native Rust shell using `winit`, `wgpu`, and `tray-icon`
- Menu-bar-only idle state with twelve Flourishes and Quit actions
- Celebratory party-popper template icon on macOS, with color on other trays
- Transparent full-screen procedural shader catalog
- Graceful first-signal exit and second-signal hard kill
- Unit-tested effect lifecycle

The renderer and tray libraries are cross-platform, and Windows taskbar
suppression is already isolated behind a platform boundary. Windows and Linux
have **not** been manually verified yet and are not claimed as shipped. Wayland
stacking behavior and Linux GTK/AppIndicator packaging are explicit follow-up
work.

## Run locally

You need a current stable Rust toolchain.

```sh
cargo run
```

Choose any effect from the Flourish menu-bar icon. Click or press any key to
start its exit; signal again during the exit to remove it immediately.

## Verify

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

On Debian/Ubuntu, `tray-icon` additionally requires GTK 3, xdo, and an
AppIndicator implementation:

```sh
sudo apt install libgtk-3-dev libxdo-dev libayatana-appindicator3-dev
```

## Project record

Research, scope, implementation plans, and reviews live in [`kb/`](kb/). The
initial technology survey is in
[`kb/research/2026-07-18_existing-flourish-tools.md`](kb/research/2026-07-18_existing-flourish-tools.md).
