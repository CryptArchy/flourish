---
id: flourish-active-monitor-targeting
type: ticket
project: flourish
tags: [multimonitor, tray, macos, winit, platform]
status: final
step: open
author: Christopher Andrews
created_date: 2026-07-20
source: file
---

# Flourish Active Monitor Targeting

## Description

Target a Flourish at the display where the presenter opened its status-bar or
tray menu, rather than always using the primary monitor.

## Context

The app creates one hidden fullscreen window at startup and does not select a
monitor. On macOS, simple fullscreen consequently targets the primary display.
`MenuEvent` identifies the chosen item but carries no screen position, so the
implementation must retain the tray click position (or equivalent platform
signal) and choose the matching `MonitorHandle` before showing an effect.

The user explicitly considers primary-monitor targeting acceptable for now;
this ticket preserves the preferred behavior without expanding the current
visual-effects pass.

## Requirements

### Functional

- Capture the screen-space location where the Flourish tray/status menu opens.
- Resolve that point against available monitor positions and physical sizes.
- Move/recreate and resize the transparent overlay and renderer for that monitor
  before starting the selected effect.
- Fall back predictably to the primary monitor when position data is absent.
- Preserve repeated launches across different monitors and all exit behavior.

### Non-functional

- Keep platform-specific tray-coordinate code isolated from the shared effect
  catalog and timeline.
- Handle negative monitor coordinates, mixed scale factors, and displays above
  or left of the primary display.
- Verify macOS first; do not claim Windows/Linux parity without real desktops.

## Research hints

`tray_icon::TrayIconEvent` click position and icon rectangle,
`winit::monitor::MonitorHandle`, borderless fullscreen monitor selection,
macOS simple fullscreen behavior, mixed-DPI coordinate conversion, menu event
ordering.

## Success Criteria

### Automated Verification

- [ ] Unit-test point-to-monitor resolution including negative coordinates and
  boundary fallback.
- [ ] Renderer resizes/reconfigures when the target display changes.

### Manual Verification

- [ ] Opening the Flourish menu on each macOS display targets that display.
- [ ] Repeatedly alternating monitors does not leave a stuck overlay or change
  presentation focus unexpectedly.
