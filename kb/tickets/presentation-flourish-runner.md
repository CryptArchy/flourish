---
id: presentation-flourish-runner
type: ticket
project: flourish
tags: [desktop, presentations, shaders, cross-platform]
status: final
step: active
author: Christopher Andrews
created_date: 2026-07-18
source: file
---

# Presentation Flourish Runner

## Description

Build Flourish, a lightweight desktop utility that lets a presenter launch
short, full-screen visual effects from the macOS menu bar or the equivalent
Windows/Linux notification area.

## Context

The effects are presentation punctuation rather than screen filters or
wallpapers. A flourish owns the screen briefly, reacts to a dismissal signal,
plays a graceful exit when it has one, and then gets out of the way.

## Requirements

### Functional

- A persistent tray/menu-bar menu lists the installed flourishes.
- Selecting a flourish opens a full-screen effect over the current display.
- A click or key press asks the active flourish to exit gracefully.
- A second click or key press during graceful exit hides it immediately.
- Effects that naturally complete hide themselves.
- The first effect is a closed red theater curtain that subtly rustles while
  waiting, then draws from the center and reveals the real desktop underneath.
- The curtain disappears automatically once it is fully off-screen.
- The menu-bar icon reads as a small celebratory Flourish: monochrome in the
  macOS template style and colorful where tray platforms preserve color.
- The curtain uses dark oxblood velvet, antique-gold center and lower trim,
  and restrained stage-light texture.
- The initial catalog also includes Pond Ripples, Fire, Doom Fire, Blackout,
  Kaleidoscope, and Mosaic, each with an effect-specific graceful exit.
- Gravel Fall drops varied stones into a growing pile and releases the entire
  pile downward when dismissed.

### Non-functional

- Keep the idle process and packaged application small enough for a background
  utility.
- Share the renderer and effect lifecycle across macOS, Windows, and Linux.
- Do not require screen-recording permission just to reveal the underlying
  presentation.
- Keep effects local and usable without a network connection.
- Respect high-DPI displays and render at the display refresh cadence.

## Research hints

ShaderGlass, ShaderToy players, dynamic wallpaper tools, transparent overlay
windows, system tray frameworks, winit, wgpu, SDL3, Qt Quick, Tauri, Wayland,
macOS simple fullscreen, graceful effect lifecycle.

## Success Criteria

### Automated Verification

- [ ] Effect lifecycle tests cover start, graceful exit, double-signal kill,
  and natural completion.
- [ ] Catalog tests cover stable menu labels, shader identifiers, and exit
  durations for every built-in flourish.
- [ ] The renderer and tray shell compile on the supported desktop targets.
- [ ] Shader validation and Rust static checks pass.

### Manual Verification

- [ ] Selecting Curtain from the menu bar covers the active display.
- [ ] The closed curtain looks alive without becoming distracting.
- [ ] One click/key draws it open and reveals the actual screen.
- [ ] A second signal during opening removes it immediately.
- [ ] The app returns to an idle menu-bar-only state after completion.
- [ ] Every catalog item launches from the menu and exits without exposing a
  stuck overlay.
- [ ] Windows and Linux behavior is verified on real desktops before claiming
  those platforms shipped.
