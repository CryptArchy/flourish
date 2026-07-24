mod doom_fire;
mod gravel;
mod renderer;

use std::{sync::Arc, time::Duration, time::Instant};

use flourish::{Flourish, SignalResult, Timeline, TimelineUpdate};
use renderer::{FlourishRenderer, RenderOutcome};
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem},
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowId, WindowLevel},
};

#[derive(Debug)]
enum UserEvent {
    Menu(MenuId),
}

struct App {
    started_at: Instant,
    timeline: Timeline,
    active_effect: Option<Flourish>,
    window: Option<Arc<Window>>,
    renderer: Option<FlourishRenderer>,
    tray: Option<TrayIcon>,
    effect_menu_ids: Vec<(MenuId, Flourish)>,
    quit_menu_id: Option<MenuId>,
    proxy: EventLoopProxy<UserEvent>,
    autostart: Option<Flourish>,
}

impl App {
    fn new(proxy: EventLoopProxy<UserEvent>, autostart: Option<Flourish>) -> Self {
        Self {
            started_at: Instant::now(),
            timeline: Timeline::new(Duration::ZERO),
            active_effect: None,
            window: None,
            renderer: None,
            tray: None,
            effect_menu_ids: Vec::new(),
            quit_menu_id: None,
            proxy,
            autostart,
        }
    }

    fn now(&self) -> Duration {
        self.started_at.elapsed()
    }

    fn start_or_signal(&mut self, effect: Flourish) {
        let now = self.now();
        if self.timeline.is_active() {
            self.handle_signal(now);
            return;
        }

        self.timeline = Timeline::new(effect.exit_duration());
        self.timeline.start(now);
        self.active_effect = Some(effect);
        if let Some(renderer) = &mut self.renderer {
            renderer.start_effect(effect);
        }
        if let Some(window) = &self.window {
            window.set_cursor_visible(false);
            window.set_visible(true);
            window.focus_window();
            window.request_redraw();
        }
    }

    fn handle_signal(&mut self, now: Duration) {
        match self.timeline.signal(now) {
            SignalResult::Ignored => {}
            SignalResult::ExitStarted => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            SignalResult::HideImmediately => self.hide_overlay(),
        }
    }

    fn hide_overlay(&mut self) {
        if let Some(window) = &self.window {
            window.set_visible(false);
            window.set_cursor_visible(true);
        }
        self.active_effect = None;
    }

    fn create_tray(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let menu = Menu::new();
        for effect in Flourish::ALL {
            let item = MenuItem::new(effect.label(), true, None);
            menu.append(&item)?;
            self.effect_menu_ids.push((item.id().clone(), effect));
        }

        menu.append(&PredefinedMenuItem::separator())?;
        let quit = MenuItem::new("Quit Flourish", true, None);
        menu.append(&quit)?;

        let tray = TrayIconBuilder::new()
            .with_tooltip("Flourish")
            .with_icon(make_tray_icon()?)
            .with_icon_as_template(cfg!(target_os = "macos"))
            .with_menu(Box::new(menu))
            .build()?;

        self.quit_menu_id = Some(quit.id().clone());
        self.tray = Some(tray);

        let proxy = self.proxy.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let _ = proxy.send_event(UserEvent::Menu(event.id));
        }));
        Ok(())
    }

    fn create_window_and_renderer(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let attributes = Window::default_attributes()
            .with_title("Flourish")
            .with_visible(false)
            .with_transparent(true)
            .with_decorations(false)
            .with_resizable(false)
            .with_window_level(WindowLevel::AlwaysOnTop);

        #[cfg(target_os = "windows")]
        let attributes = {
            use winit::platform::windows::WindowAttributesExtWindows;
            attributes.with_skip_taskbar(true)
        };

        #[cfg(not(target_os = "macos"))]
        let attributes =
            attributes.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));

        let window = Arc::new(event_loop.create_window(attributes)?);

        #[cfg(target_os = "macos")]
        {
            use winit::platform::macos::WindowExtMacOS;
            if !window.set_simple_fullscreen(true) {
                return Err("macOS refused simple fullscreen mode".into());
            }
        }

        let renderer = pollster::block_on(FlourishRenderer::new(Arc::clone(&window)))?;
        self.window = Some(window);
        self.renderer = Some(renderer);
        Ok(())
    }

    fn draw(&mut self) {
        if !self.timeline.is_active() {
            return;
        }

        let now = self.now();
        if self.timeline.update(now) == TimelineUpdate::HideCompleted {
            self.hide_overlay();
            return;
        }

        let time = self.timeline.effect_time(now);
        let exit_progress = self.timeline.exit_progress(now).unwrap_or(0.0);
        let Some(effect) = self.active_effect else {
            self.timeline.complete();
            self.hide_overlay();
            return;
        };
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        match renderer.render(effect, time, exit_progress) {
            RenderOutcome::Presented | RenderOutcome::Skipped => {}
            RenderOutcome::Reconfigure => {
                if let Some(window) = &self.window {
                    renderer.resize(window.inner_size());
                }
            }
            RenderOutcome::SurfaceLost => {
                eprintln!("Flourish lost its window surface; hiding the overlay");
                self.timeline.complete();
                self.hide_overlay();
            }
            RenderOutcome::ValidationError => {
                eprintln!("Flourish encountered a GPU surface validation error");
            }
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        if let Err(error) = self.create_window_and_renderer(event_loop) {
            eprintln!("Could not initialize Flourish: {error}");
            event_loop.exit();
            return;
        }

        if let Err(error) = self.create_tray() {
            eprintln!("Could not create the Flourish menu: {error}");
            event_loop.exit();
            return;
        }

        if let Some(effect) = self.autostart {
            self.start_or_signal(effect);
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Menu(id) if self.quit_menu_id.as_ref() == Some(&id) => {
                event_loop.exit();
            }
            UserEvent::Menu(id) => {
                let selected = self
                    .effect_menu_ids
                    .iter()
                    .find_map(|(menu_id, effect)| (menu_id == &id).then_some(*effect));
                if let Some(effect) = selected {
                    self.start_or_signal(effect);
                }
            }
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window.as_ref().map(|window| window.id()) != Some(window_id) {
            return;
        }

        match event {
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left | MouseButton::Right | MouseButton::Middle,
                ..
            } => self.handle_signal(self.now()),
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                self.handle_signal(self.now());
            }
            WindowEvent::CloseRequested => {
                self.timeline.complete();
                self.hide_overlay();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.timeline.is_active() {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + Duration::from_millis(8),
            ));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}

fn make_tray_icon() -> Result<Icon, tray_icon::BadIcon> {
    const SIZE: u32 = 32;
    Icon::from_rgba(make_tray_rgba(), SIZE, SIZE)
}

#[allow(clippy::cast_precision_loss)]
fn make_tray_rgba() -> Vec<u8> {
    const SIZE: u32 = 32;
    const SAMPLES: u32 = 4;
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let mut accumulated = [0_u32; 3];
            let mut covered = 0_u32;
            for sample_y in 0..SAMPLES {
                for sample_x in 0..SAMPLES {
                    let point = [
                        (x as f32 + (sample_x as f32 + 0.5) / SAMPLES as f32) / SIZE as f32,
                        (y as f32 + (sample_y as f32 + 0.5) / SAMPLES as f32) / SIZE as f32,
                    ];
                    if let Some(color) = party_popper_color(point) {
                        accumulated[0] += u32::from(color[0]);
                        accumulated[1] += u32::from(color[1]);
                        accumulated[2] += u32::from(color[2]);
                        covered += 1;
                    }
                }
            }

            if covered == 0 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let alpha = u8::try_from(covered * 255 / (SAMPLES * SAMPLES))
                    .expect("supersample coverage always fits in one byte");
                rgba.extend_from_slice(&[
                    u8::try_from(accumulated[0] / covered)
                        .expect("averaged red channel always fits in one byte"),
                    u8::try_from(accumulated[1] / covered)
                        .expect("averaged green channel always fits in one byte"),
                    u8::try_from(accumulated[2] / covered)
                        .expect("averaged blue channel always fits in one byte"),
                    alpha,
                ]);
            }
        }
    }
    rgba
}

fn party_popper_color(point: [f32; 2]) -> Option<[u8; 3]> {
    const GOLD: [u8; 3] = [230, 177, 62];
    const PALE_GOLD: [u8; 3] = [255, 221, 118];
    const OXBLOOD: [u8; 3] = [102, 19, 36];
    const HOUSE_BLACK: [u8; 3] = [24, 12, 20];
    const TEAL: [u8; 3] = [66, 177, 170];

    let [x, y] = point;
    let star_x = (x - 0.72).abs();
    let star_y = (y - 0.23).abs();
    let four_point_star =
        (star_x / 0.055 + star_y / 0.20 <= 1.0) || (star_x / 0.20 + star_y / 0.055 <= 1.0);
    if four_point_star {
        return Some(PALE_GOLD);
    }

    if distance_to_segment(point, [0.48, 0.25], [0.43, 0.13]) < 0.027
        || distance_to_segment(point, [0.76, 0.49], [0.87, 0.43]) < 0.025
    {
        return Some(TEAL);
    }
    if distance_to_segment(point, [0.56, 0.42], [0.63, 0.34]) < 0.024
        || distance_to_segment(point, [0.88, 0.67], [0.84, 0.55]) < 0.026
    {
        return Some(OXBLOOD);
    }
    if distance_to_segment(point, [0.34, 0.37], [0.31, 0.24]) < 0.024 {
        return Some(GOLD);
    }

    let outer = point_in_triangle(point, [0.12, 0.88], [0.38, 0.40], [0.67, 0.68]);
    if !outer {
        return None;
    }
    let inner = point_in_triangle(point, [0.17, 0.82], [0.40, 0.46], [0.59, 0.66]);
    if !inner {
        return Some(HOUSE_BLACK);
    }

    let stripe = ((x + y) * 12.0).floor();
    Some(if stripe.rem_euclid(3.0) < 1.0 {
        OXBLOOD
    } else {
        GOLD
    })
}

fn point_in_triangle(point: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
    let sign = |p: [f32; 2], q: [f32; 2], r: [f32; 2]| {
        (p[0] - r[0]) * (q[1] - r[1]) - (q[0] - r[0]) * (p[1] - r[1])
    };
    let d1 = sign(point, a, b);
    let d2 = sign(point, b, c);
    let d3 = sign(point, c, a);
    let has_negative = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_positive = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_negative && has_positive)
}

fn distance_to_segment(point: [f32; 2], start: [f32; 2], end: [f32; 2]) -> f32 {
    let segment = [end[0] - start[0], end[1] - start[1]];
    let to_point = [point[0] - start[0], point[1] - start[1]];
    let length_squared = segment[0] * segment[0] + segment[1] * segment[1];
    let projection =
        ((to_point[0] * segment[0] + to_point[1] * segment[1]) / length_squared).clamp(0.0, 1.0);
    let nearest = [
        start[0] + segment[0] * projection,
        start[1] + segment[1] * projection,
    ];
    ((point[0] - nearest[0]).powi(2) + (point[1] - nearest[1]).powi(2)).sqrt()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let autostart = std::env::args().find_map(|argument| {
        if argument == "--autostart" {
            Some(Flourish::Curtain)
        } else {
            argument
                .strip_prefix("--autostart=")
                .and_then(Flourish::from_slug)
        }
    });
    let mut builder = EventLoop::<UserEvent>::with_user_event();

    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        builder
            .with_activation_policy(ActivationPolicy::Accessory)
            .with_default_menu(false);
    }

    let event_loop = builder.build()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let proxy = event_loop.create_proxy();
    let mut app = App::new(proxy, autostart);
    event_loop.run_app(&mut app)?;
    Ok(())
}

#[cfg(test)]
mod icon_tests {
    use super::make_tray_rgba;
    use std::collections::HashSet;

    #[test]
    fn tray_icon_is_a_non_empty_multicolor_rgba_mask() {
        let rgba = make_tray_rgba();
        assert_eq!(rgba.len(), 32 * 32 * 4);

        let opaque_pixels = rgba.chunks_exact(4).filter(|pixel| pixel[3] > 200);
        let colors = opaque_pixels
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect::<HashSet<_>>();
        assert!(colors.len() >= 4);
        assert!(rgba.chunks_exact(4).any(|pixel| pixel[3] == 0));
        assert!(rgba.chunks_exact(4).any(|pixel| pixel[3] == 255));
    }
}
