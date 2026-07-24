---
id: 2026-07-18_existing-flourish-tools
type: research
project: flourish
tags: [desktop, shaders, overlays, cross-platform]
status: final
author: Christopher Andrews
created_date: 2026-07-18
upstream: [presentation-flourish-runner]
repository: flourish
branch: main
git_commit: unborn
---

# Existing Flourish Tools and Runtime Options

## Question

Does an existing project already provide the proposed presentation-flourish
interaction, and if not, which runtime gives us the smallest credible path to
a transparent full-screen shader, tray menu, and graceful effect lifecycle on
macOS, Windows, and Linux?

## Executive Answer

No maintained project found combines all four defining behaviors: tray launch,
an opaque-to-transparent full-screen effect, input-driven graceful exit, and a
second-signal hard kill. Several projects prove individual pieces. Flourish is
small enough to justify building, but it should reuse mature window, GPU, and
tray libraries rather than invent a platform shell.

The recommended stack is Rust with `winit` + `wgpu` + `tray-icon`. It preserves
a genuinely native, lightweight runtime, uses WGSL as one shader source across
Metal/D3D12/Vulkan, and exposes the platform controls this effect needs. The
main uncertainty is Linux desktop behavior, especially Wayland stacking and
tray dependencies, so macOS is the first verified vertical slice and Linux is
not called shipped until tested on representative compositors.

## Existing Projects

### ShaderGlass: closest product concept, wrong platform and lifecycle

[ShaderGlass](https://github.com/mausimus/ShaderGlass) is the closest mature
neighbor. It can apply GPU shaders over the Windows desktop in a transparent or
full-screen window and imports RetroArch shaders. As of v1.3.0 (2026-03-19), it
is a Windows 10/11 DirectX 11 application; Linux support is through Wine/Proton.
It is a persistent screen-processing tool, not a launcher for self-terminating
presentation transitions, and its GPL-3.0 codebase would materially determine
Flourish's license if reused.

Useful lesson: desktop overlays and shader libraries are viable. Reuse choice:
study behavior, do not fork it for this product.

### wewa: useful WebView shader runner, but a wallpaper tool

[wewa](https://github.com/ownself/wewa) is a small Rust CLI that renders local
ShaderToy-style GLSL or web content as a multi-monitor wallpaper using WebView2,
WKWebView, and WebKitGTK. It already has a local WebGL shader wrapper, display
selection, and IPC shutdown. It deliberately places content behind desktop
windows and has no tray, foreground overlay, input lifecycle, or transparent
reveal.

Its `Cargo.toml` declares MIT, but the repository has no license file and GitHub
reports no detected license as of 2026-07-18. Treat the code as reference-only
unless that ambiguity is resolved. Its platform modules are valuable evidence
that Linux display behavior deserves explicit compositor testing.

### SHADERed and shaderbang: authoring/CLI tools, not the shell

[SHADERed](https://github.com/dfranx/SHADERed) is a capable cross-platform
shader IDE. [shaderbang](https://github.com/astefanutti/shaderbang) runs
ShaderToy-style shaders directly through DRM/KMS on Linux. Both can help shader
development, but neither supplies the tray-launched desktop overlay lifecycle.

### Qt's curtain example: a good visual reference

The [Qt 6 Book curtain effect](https://www.qt.io/product/qt6/qml-book/ch10-effects-curtain-effect)
uses a tessellated mesh, a sine-displaced vertex shader, fragment shading, and
a spring lag between the top and bottom curtain widths. It opens from one side,
so Flourish still needs two mirrored panels and a different state model. The
example code is BSD-3-Clause, making it a safe conceptual reference with
attribution if code is adapted.

## Runtime Comparison

| Option | Strong points | Blocking or material costs | Decision |
|---|---|---|---|
| ShaderGlass fork | Proven Windows overlay; many shaders | Windows/DX11, GPL-3.0, wrong lifecycle | Reject |
| Tauri 2 + WebGL | Tray API, quick shader iteration, system WebViews | Transparent windows require Tauri's private-API feature on macOS, preventing App Store acceptance; tray click events are unsupported on Linux | Keep as fallback |
| Qt 6 Quick | Excellent transparent-window, tray, shader, and animation APIs; official curtain example | Larger deployment, C++/QML stack, LGPL/commercial packaging obligations | Technically sound, heavier than desired |
| SDL3 GPU | Native tray, transparent window flags, GPU abstraction | SDL maintainers explicitly state its GPU API cannot render to transparent windows; it now fails that combination | Reject |
| `winit` + `wgpu` + `tray-icon` | Native Rust, one WGSL path, transparent/full-screen window controls, portable tray menu | Linux GTK/AppIndicator packages; Wayland ignores always-on-top; transparency remains platform-sensitive | Recommend |

## Why the Recommended Stack Fits

- [`winit` 0.30](https://docs.rs/winit/latest/winit/window/struct.WindowAttributes.html)
  exposes transparent, undecorated, fullscreen, focus, and window-level
  attributes. On macOS, `set_simple_fullscreen` covers the current display
  without moving the presenter into a new Space. Windows can explicitly skip
  the taskbar entry.
- [`wgpu`](https://github.com/gfx-rs/wgpu) maps WGSL to Metal, D3D12, and Vulkan
  and exposes swapchain alpha compositing. That avoids maintaining separate
  Metal, HLSL, and Vulkan shader sources.
- [`tray-icon`](https://docs.rs/tray-icon/latest/tray_icon/) supports macOS,
  Windows, and GTK Linux and integrates with a winit event loop. Linux requires
  GTK plus AppIndicator/Ayatana packages, which packaging must declare.
- A transparent overlay reveals the real presentation directly. Unlike a
  screen-capture-and-clone design, it should not require macOS Screen Recording
  permission and cannot accidentally reveal a stale captured frame.

## Proposed Effect Contract

Every flourish should share a small deterministic lifecycle:

1. `idle`: no overlay window is visible.
2. `holding`: the flourish is visible and may loop indefinitely.
3. `exiting`: the first click/key asks it to play its exit sequence.
4. `idle`: natural completion hides the overlay.

Any signal received during `exiting` bypasses animation and returns immediately
to `idle`. Effects that do not loop may transition from `holding` to `idle` on
their own. The renderer gets elapsed hold time plus normalized exit progress;
the shell owns input, window visibility, and hard-kill semantics.

## Curtain Rendering Direction

Use one full-screen procedural WGSL fragment shader for the first vertical
slice. Two mirrored red panels cover the screen. Low-amplitude phase changes in
the fold normals create the waiting rustle. During exit, each panel's inner
edge moves toward its outer edge with eased progress; a small vertical lag and
damped oscillation make the lower fabric trail. Pixels between the panels emit
zero alpha, revealing the live screen below. This avoids texture licensing and
keeps the first effect to one shader asset.

A later fidelity pass can move to a tessellated curtain mesh, borrowing the
BSD-licensed Qt example's top/bottom width and spring concepts if the procedural
version looks too flat.

## Risks and Validation Needs

- **Wayland stacking:** winit documents `AlwaysOnTop` as unsupported on Wayland.
  A focused borderless fullscreen window should be sufficient, but GNOME, KDE,
  and one wlroots compositor must be tested.
- **Transparency:** the swapchain must support a non-opaque composite alpha
  mode. Startup should fail clearly rather than show a black reveal.
- **macOS spaces:** use simple fullscreen, not native borderless fullscreen on a
  separate Space.
- **Presentation focus:** launch must focus the overlay so ordinary key presses
  dismiss it; manual testing with Keynote, PowerPoint, and a browser is needed.
- **Multiple monitors:** start with the display containing the tray/menu action
  or primary display. Add an explicit display submenu after the single-display
  lifecycle is solid.

## Sources

- [ShaderGlass repository and v1.3.0 requirements](https://github.com/mausimus/ShaderGlass)
- [wewa repository and platform design](https://github.com/ownself/wewa)
- [Qt 6 Book curtain effect](https://www.qt.io/product/qt6/qml-book/ch10-effects-curtain-effect)
- [Qt QSystemTrayIcon platform support](https://doc.qt.io/qt-6/qsystemtrayicon.html)
- [Tauri 2 system tray](https://v2.tauri.app/learn/system-tray/)
- [Tauri transparent-window configuration](https://v2.tauri.app/reference/config/)
- [SDL3 transparent window creation](https://wiki.libsdl.org/SDL3/SDL_CreateWindowWithProperties)
- [SDL3 GPU transparent-window limitation](https://github.com/libsdl-org/SDL/issues/12410)
- [winit window attributes](https://docs.rs/winit/latest/winit/window/struct.WindowAttributes.html)
- [winit macOS simple fullscreen](https://docs.rs/winit/latest/winit/platform/macos/trait.WindowExtMacOS.html)
- [wgpu platform backends](https://github.com/gfx-rs/wgpu)
- [wgpu swapchain alpha configuration](https://docs.rs/wgpu/latest/wgpu/type.SurfaceConfiguration.html)
- [tray-icon platform notes](https://docs.rs/tray-icon/latest/tray_icon/)
