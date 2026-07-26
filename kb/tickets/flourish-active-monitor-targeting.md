---
id: flourish-active-monitor-targeting
type: ticket
project: flourish
tags: [multimonitor, tray, macos, winit, platform]
status: final
step: done
author: Christopher Andrews
created_date: 2026-07-20
closed_date: 2026-07-26
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

- [x] Unit-test point-to-monitor resolution including negative coordinates and
  boundary fallback. — `src/display.rs`, 9 cases: negative origins, shared
  borders, gaps, non-finite input, single and zero monitors.
- [x] Renderer resizes/reconfigures when the target display changes. —
  `App::present_on` calls `renderer.resize` after the window is placed and
  full-screen has settled the final surface size.

### Manual Verification

- [x] Opening the Flourish menu on each macOS display targets that display.
- [x] Repeatedly alternating monitors does not leave a stuck overlay or change
  presentation focus unexpectedly.

## Implementation notes

**The signal is the pointer, not the tray click.** This ticket predates the
global shortcut, which carries no click position at all. The pointer covers
both entry points: clicking the menu-bar icon necessarily puts it on that
display, and pressing the shortcut puts it where the presenter is working.
`TrayIconEvent`'s position would only have covered the first.

**Everything happens in logical coordinates**, which turned out to be
load-bearing rather than stylistic. winit reports a monitor's `position()` as
its logical origin already multiplied by *that monitor's own* scale factor,
while `size()` is the true pixel count. On this machine that means:

| Display | Physical | Scale | Logical |
| --- | --- | --- | --- |
| Primary | 3456x2234 at (0, 0) | 2 | 1728x1117 at (0, 0) |
| External | 1920x1080 at (1728, 0) | 1 | 1920x1080 at (1728, 0) |

In physical space the primary spans x from 0 to 3456 and therefore contains the
external display's origin at 1728 — the two overlap, and a containment test
would put most of the external display "inside" the primary. Dividing both
position and size by each monitor's own scale factor makes them tile exactly,
with the primary ending at 1728 precisely where the external begins. That exact
abutment is also the check that the conversion is right: a wrong scale would
leave a gap or an overlap.

The same reasoning rules out `CGDisplayPixelsHigh` for flipping the pointer's
vertical axis, which is what `tray-icon` uses internally: it is a pixel height
subtracted from a point-space coordinate, so on any Retina primary it is off by
the scale factor. The flip here uses the primary's *logical* height, derived
the same way as the bounds it is compared against.

**Full-screen is now entered per flourish rather than once at startup**, since
the target display is only known when a flourish is asked for. On macOS simple
full-screen is a property of the screen the window currently occupies, so it is
released, the window moved and *shown*, and only then re-applied — engaging it
while the window is hidden or still on the old display resolves against the
wrong screen. Releasing it on hide also resolves a latent issue: simple
full-screen suppresses the menu bar and Dock for as long as it is engaged, and
the old code never turned it off.

The first implementation of this shipped with a regression that manual testing
caught; see the review for what broke and why. Placement lives in
`flourish::display::place_overlay` beside the coordinate reasoning it depends
on, and `examples/placement.rs` is a harness that aims the overlay at every
attached display and reports where it actually landed.
