---
id: flourish-active-monitor-targeting-review
type: review
project: flourish
tags: [multimonitor, macos, winit, mixed-dpi, hello-gravel]
status: final
author: Christopher Andrews
created_date: 2026-07-26
upstream: [flourish-active-monitor-targeting]
---

# Flourish Active Monitor Targeting Review

## Implementation Status

**Complete and confirmed on hardware.** Flourishes target the display under the
pointer, resolved in global logical coordinates, and the overlay is placed and
made full-screen per flourish rather than once at startup. Manually verified on
a mixed-DPI two-display desk after one regression was found and fixed.

## Deviation From The Ticket

The ticket specified capturing the tray click position. The implementation uses
the **pointer** instead. The ticket predates the global shortcut, which carries
no click position at all; the pointer covers both entry points, since clicking
the menu-bar icon necessarily puts it on that display and the shortcut puts it
where the presenter is working. `TrayIconEvent`'s position would only have
covered the first, and would have needed a second mechanism for the shortcut.

## Regression Found In Manual Testing

First implementation played correctly on the external display once, then sent
every later flourish to the built-in panel. Two causes, both in the placement
step rather than the resolution:

1. The window was placed with the monitor's **physical** position. winit
   converts a physical position using the *window's* scale factor, while the
   monitor's position was scaled by the *monitor's* factor. Aiming at the
   external display's physical x=1728 while the window sat on the 2x built-in
   panel produced logical x=864 — still on the built-in. This is the same
   mixed-DPI trap the module documents, reintroduced by passing physical
   coordinates across the boundary two functions later.
2. Full-screen was engaged while the window was still hidden and still on the
   previous display. macOS simple full-screen resizes to whatever
   `NSWindow.screen` reports, resolved from the window's own frame, so it
   snapped back to the display it had never left.

The autostart appeared to work only because the window was newly created and
macOS happened to place it on the external display.

Fixed by placing in logical points, and by moving and showing the window before
engaging full-screen.

## Automated Verification Results

- `cargo fmt --all -- --check`: pass
- `cargo test --locked --all-targets`: pass, 78 tests
- `cargo clippy --all-targets --locked -- -D warnings`: pass
- `cargo build --locked --release`: pass
- `cargo +1.88 check --locked --all-targets`: pass (declared MSRV)
- `cargo run --example placement`: pass, 0 of 4 placements on the wrong display
- CI on macOS, Windows, Ubuntu, MSRV, audit, and app bundle: pass

The placement harness was confirmed to have teeth by reverting each half of the
fix: with physical placement restored it reports `WRONG DISPLAY` for the
external monitor on both passes, reproducing the reported symptom exactly.

## Manual Verification Results

- Opening a flourish on each display targets that display: **pass**
- Alternating displays repeatedly leaves no stuck overlay and does not disturb
  presentation focus: **pass**
- No `could not locate the pointer` fallback and no shortcut-registration
  warning across repeated runs, so the pointer resolved every time.

## Notes For Later

- Automatic targeting is macOS-only. Windows and Linux have no safe cursor
  query under `forbid(unsafe_code)` and fall back to the primary display; the
  ticket's non-functional requirement was explicit about not claiming parity
  without real desktops.
- `--displays` prints the layout as the window system sees it, which is the
  first thing to run if a flourish ever appears on the wrong screen.
- Releasing full-screen on hide also resolved a latent issue outside this
  ticket: macOS simple full-screen suppresses the menu bar and Dock for as long
  as it is engaged, and the previous code never turned it off.
