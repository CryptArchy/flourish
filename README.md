# Flourish

Flourish is a lightweight desktop utility for adding a little theatrical
punctuation to presentations. Pick an effect from the menu bar — or press the
global shortcut without leaving your slides — let it own the screen for a
moment, then dismiss it with any click or key.

The signature effect is **Curtain**: dark oxblood velvet with antique-gold trim
and warm stage lights. It rustles while it waits, then draws open from the
center to reveal the live screen underneath. A second signal during any
flourish's exit removes it immediately.

Every flourish is seeded per performance, so showing the same one twice in a
talk does not replay the same picture.

## Flourishes

| Flourish | While it waits | Graceful exit |
| --- | --- | --- |
| Curtain | Lit velvet subtly rustles | Draws open from the center |
| Confetti | Foil tumbles down over whatever is on screen | The source stops and the last of it falls away |
| Marquee Bulbs | Edison bulbs warm and cool at their own pace | Every filament runs past full, then they pop out one by one |
| Spotlight | A warm follow-spot searches a dark stage through its own dusty beam | The pool floods outward and the screen returns behind the light |
| Projector Iris | Soot-black overlapping blades hold around a tungsten pinhole | The mechanical aperture spirals open |
| Elevator Doors | Brushed steel with reflections drifting across it | The doors part on a driven, mechanical slide |
| Geological Strata | Textured sediment bands form a dramatic road cut | A crooked fault opens and both land masses shear away |
| Paper Tear | A fibrous cream sheet covers the screen | A ragged tear runs down from the top and both halves curl away |
| Frosted Glass | Dendritic ice creeps over a translucent pane | Irregular warm fronts melt through the frost |
| CRT Shutdown | Phosphor scanlines and restrained analog noise fill a dark tube | The image collapses to a line, contracts to a dot, and blinks out |
| Pond Ripples | Independent concentric ripples cross a calm surface | Dissipates in place |
| Fire | Fluid procedural flames and embers lick upward | Gutters down to nothing |
| Doom Fire | Pixel heat propagates through a PSX-inspired automaton | Source cools and the field fades |
| Gravel Fall | Faceted stones tumble down and build a natural pile | The floor vanishes and the whole pile drops away |
| Constellation | A twinkling star field with quiet asterism lines | The lines retract, the sky leaves as a meteor shower, and the night is swept off behind it |
| Blackout | A clean, nearly pure black screen | Diagonal wipe reveal |
| Kaleidoscope | Jewel-toned mirrored facets turn | Radial aperture reveal |
| Mosaic | Colored beveled tiles drift by row | Tiles shrink away in sequence |

## Surprise Me

With eighteen effects, picking one from a menu mid-sentence is its own kind of
friction. **Surprise Me** is the first item in the menu, and it sticks: once
chosen, the global shortcut draws a different flourish every press, never
repeating the one just played. Choose a named effect instead and the shortcut
goes back to replaying that one.

```sh
cargo run -- --autostart=random
```

## Dismissing a flourish

A flourish is a full-screen, always-on-top window that hides the cursor, so it
is deliberately hard to get stuck behind one:

- **Click or press any key** to begin its graceful exit.
- **Signal again during the exit** to remove it immediately.
- **Press the global shortcut** (`⌃⌥⌘F` on macOS, `Ctrl+Alt+Shift+F`
  elsewhere) to summon or dismiss without leaving your deck.
- **Do nothing.** Every flourish gives the screen back on its own after
  15 seconds, and losing window focus also starts its exit — a presenter whose
  pointer is on another display is never stranded.

## Multiple displays

A flourish plays on the display the pointer is on, so it appears where you are
looking rather than wherever the window system calls primary. That covers both
ways of starting one: clicking the menu-bar icon puts the pointer on that
display, and the global shortcut puts it wherever you are working.

If a flourish lands on the wrong screen, this prints the layout as the window
system sees it, which is usually not how it looks on the desk:

```sh
cargo run -- --displays
```

Automatic targeting is macOS-only. Elsewhere the primary display is used.

Placement itself depends on window-system behaviour no unit test can reproduce,
so it has its own harness. It aims the overlay at every attached display twice
and reports where each one actually landed:

```sh
cargo run --example placement
```

## Performance

Flourish deliberately asks for the low-power adapter, and some effects are
heavy per-pixel shaders, so the cost is measurable rather than assumed:

```sh
cargo run --release -- --benchmark
```

That renders every flourish offscreen from 1080p to 5K through the same drawing
path the app uses, and reports sustained milliseconds per frame against the
60Hz and 120Hz budgets. On an Apple M5 Max the worst case is Frosted Glass at
5K, about half the 120Hz budget; results and the caveat about older Intel
machines are in
[`kb/notes/flourish-frame-time-budget.md`](kb/notes/flourish-frame-time-budget.md).

## Choosing one without playing it

Deciding which flourish suits a talk should not mean launching eighteen of them
full-screen:

```sh
cargo run --release -- --frames
```

That writes one filmstrip PNG per flourish into `./flourish-frames` — the hold
state and four points through the exit, composited over a stand-in desktop so
the reveal and the transparency are both visible. It renders through the same
`Scene` the app uses, offscreen, so nothing appears on your display. Pass a
directory to put them elsewhere: `--frames=/tmp/strips`.

## Reduced motion

Flourish follows the system's reduce-motion setting. When it is on, every
flourish still appears — same velvet, same frost, same strata — but holds a
settled composition and cross-fades in and out instead of sweeping, spiralling,
or collapsing.

This is not only a personal preference setting. A flourish fills a projector
screen in front of a room, and the CRT's static, the fire's flicker, and the
kaleidoscope's rotation are the kind of rapid, high-contrast movement that
provokes migraine and, at worst, photosensitive seizures. The person affected is
usually in the audience and has no way to ask you to stop.

Because you may only learn that once you are already on stage, **Reduce Motion
is a menu item** as well as a setting, and can be toggled mid-talk.

```sh
cargo run -- --reduce-motion
```

`--full-motion` forces animation on even where the system asks otherwise.
Detection is automatic on macOS and on GNOME; elsewhere use the flag or the
menu.

## Current status

This repository contains an expanded macOS vertical slice:

- Native Rust shell using `winit`, `wgpu`, and `tray-icon`
- Menu-bar-only idle state with eighteen Flourishes, Surprise Me, and Quit
- Global shortcut for summoning and dismissing from inside a full-screen deck
- Flourishes target the display the pointer is on, across mixed-DPI layouts
- Celebratory party-popper template icon on macOS, with color on other trays
- Transparent full-screen procedural shader catalog, seeded per performance
- Reduced-motion path that holds each flourish still and cross-fades, following
  the system setting and toggleable from the menu
- Graceful first-signal exit, second-signal hard kill, and a self-dismiss
  ceiling
- Unit-tested effect lifecycle, and WGSL validated at build time

The renderer and tray libraries are cross-platform, and Windows taskbar
suppression is already isolated behind a platform boundary. Windows and Linux
build in CI but have **not** been manually verified and are not claimed as
shipped. Wayland stacking behavior and Linux GTK/AppIndicator packaging are
explicit follow-up work.

## Install on macOS

Build a proper menu-bar app and drag it to `/Applications`:

```sh
scripts/bundle-macos.sh --universal
```

The bundle is an agent — no Dock tile, no app-switcher entry, and no stealing
focus from your deck when it launches. Its icon is drawn by the same code that
draws the menu-bar icon, so there are no image files in this repository.

It is ad-hoc signed, which is enough to run on the machine that built it.
Distributing it to anyone else needs a Developer ID and notarization; see
[`packaging/macos/README.md`](packaging/macos/README.md).

## Run locally

You need Rust 1.88 or newer.

```sh
cargo run
```

Choose any effect from the Flourish menu-bar icon, or start one straight away:

```sh
cargo run -- --autostart=gravel-fall
```

`--list` prints every flourish and its slug — including `random`, which asks to
be surprised; `--help` covers the rest.

## Verify

```sh
cargo fmt --all -- --check
```

```sh
cargo test --locked --all-targets
```

```sh
cargo clippy --all-targets --locked -- -D warnings
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
