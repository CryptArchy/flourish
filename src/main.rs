mod cli;
mod doom_fire;
mod gravel;
mod hotkey;
mod renderer;

use std::{sync::Arc, time::Duration, time::Instant};

use flourish::{Flourish, SignalResult, Timeline, TimelineUpdate};
use hotkey::HotkeyBinding;
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

/// Focus loss within this window of a flourish starting is treated as the
/// window manager settling rather than as the presenter dismissing it.
const FOCUS_GRACE: Duration = Duration::from_millis(400);

#[derive(Debug)]
enum UserEvent {
    Menu(MenuId),
    Hotkey,
}

struct App {
    started_at: Instant,
    timeline: Timeline,
    active_effect: Option<Flourish>,
    last_effect: Flourish,
    window: Option<Arc<Window>>,
    renderer: Option<FlourishRenderer>,
    tray: Option<TrayIcon>,
    hotkey: Option<HotkeyBinding>,
    effect_menu_ids: Vec<(MenuId, Flourish)>,
    quit_menu_id: Option<MenuId>,
    proxy: EventLoopProxy<UserEvent>,
    autostart: Option<Flourish>,
    /// Set when startup fails; reported by `main` after the loop unwinds.
    fatal_error: Option<(String, String)>,
}

impl App {
    fn new(proxy: EventLoopProxy<UserEvent>, autostart: Option<Flourish>) -> Self {
        Self {
            started_at: Instant::now(),
            timeline: Timeline::idle(),
            active_effect: None,
            last_effect: autostart.unwrap_or(Flourish::Curtain),
            window: None,
            renderer: None,
            tray: None,
            hotkey: None,
            effect_menu_ids: Vec::new(),
            quit_menu_id: None,
            proxy,
            autostart,
            fatal_error: None,
        }
    }

    fn now(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Starts `effect`, replacing whatever is on screen.
    ///
    /// Choosing from the menu or pressing the hotkey is an explicit request for
    /// that flourish. Treating it as a dismissal instead — as this once did —
    /// silently discarded the presenter's actual choice.
    fn start_effect(&mut self, effect: Flourish) {
        let now = self.now();
        self.timeline = Timeline::new(effect.exit_duration(), effect.hold_limit());
        self.timeline.start(now);
        self.active_effect = Some(effect);
        self.last_effect = effect;
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

    /// The hotkey both summons and dismisses: pressing it during a flourish
    /// advances that flourish's exit rather than restarting it.
    fn toggle_via_hotkey(&mut self) {
        if self.timeline.is_active() {
            self.handle_signal(self.now());
        } else {
            self.start_effect(self.last_effect);
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

    /// Records a fatal startup problem for `main` to report once the event loop
    /// has unwound.
    ///
    /// The dialog is deliberately not shown from here: it is a blocking modal,
    /// and running one from inside an event-loop callback risks a hang, which
    /// would be a worse failure than the one being reported.
    fn fail(&mut self, headline: &str, error: &impl std::fmt::Display) {
        eprintln!("{headline}: {error}");
        self.fatal_error
            .get_or_insert_with(|| (headline.to_owned(), error.to_string()));
    }

    fn hide_overlay(&mut self) {
        // Owns the whole teardown, including the timeline, so no caller can
        // leave a hidden window paired with a still-running timeline.
        self.timeline.complete();
        if let Some(window) = &self.window {
            window.set_visible(false);
            window.set_cursor_visible(true);
        }
        self.active_effect = None;
    }

    fn create_tray(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let menu = Menu::new();
        for effect in Flourish::ALL.iter().copied() {
            let item = MenuItem::new(effect.label(), true, None);
            menu.append(&item)?;
            self.effect_menu_ids.push((item.id().clone(), effect));
        }

        menu.append(&PredefinedMenuItem::separator())?;
        // Advertise the global shortcut, which is the only way to reach a
        // flourish without leaving a full-screen deck.
        let hint = MenuItem::new(format!("Replay: {}", hotkey::DESCRIPTION), false, None);
        menu.append(&hint)?;
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
        match self.timeline.update(now) {
            TimelineUpdate::Active => {}
            TimelineUpdate::HoldExpired => {
                eprintln!(
                    "Flourish held the screen for its full {} seconds; \
                     dismissing it automatically",
                    self.active_effect
                        .map_or(0, |effect| effect.hold_limit().as_secs())
                );
            }
            TimelineUpdate::HideCompleted => {
                self.hide_overlay();
                return;
            }
        }

        let time = self.timeline.effect_time(now);
        let exit_progress = self.timeline.exit_progress(now).unwrap_or(0.0);
        let Some(effect) = self.active_effect else {
            self.hide_overlay();
            return;
        };
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        match renderer.render(effect, time, exit_progress) {
            RenderOutcome::Presented | RenderOutcome::Skipped | RenderOutcome::Recovered => {}
            RenderOutcome::Reconfigure => {
                if let Some(window) = &self.window {
                    renderer.resize(window.inner_size());
                }
            }
            RenderOutcome::SurfaceLost => {
                eprintln!("Flourish could not recover its window surface; hiding the overlay");
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
            self.fail("Flourish could not start", &error);
            event_loop.exit();
            return;
        }

        if let Err(error) = self.create_tray() {
            self.fail("Flourish could not create its menu", &error);
            event_loop.exit();
            return;
        }

        // A missing hotkey is a degraded experience, not a reason to refuse to
        // run: the tray menu still works.
        match HotkeyBinding::register(self.proxy.clone()) {
            Ok(binding) => self.hotkey = Some(binding),
            Err(error) => eprintln!(
                "Flourish could not register the {} shortcut ({error}); \
                 use the menu-bar icon instead",
                hotkey::DESCRIPTION
            ),
        }

        if let Some(effect) = self.autostart {
            self.start_effect(effect);
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Hotkey => self.toggle_via_hotkey(),
            UserEvent::Menu(id) if self.quit_menu_id.as_ref() == Some(&id) => {
                event_loop.exit();
            }
            UserEvent::Menu(id) => {
                let selected = self
                    .effect_menu_ids
                    .iter()
                    .find_map(|(menu_id, effect)| (menu_id == &id).then_some(*effect));
                if let Some(effect) = selected {
                    self.start_effect(effect);
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
            // Losing focus means input is going somewhere else, so the overlay
            // can no longer be dismissed by clicking or typing at it. Treat it
            // as a dismissal rather than stranding the presenter behind a
            // full-screen window they can no longer talk to.
            WindowEvent::Focused(false) => {
                let now = self.now();
                // Compared in seconds rather than by rebuilding a Duration,
                // because Duration::from_secs_f32 panics on inputs a float
                // clock could produce.
                if self.timeline.effect_time(now) >= FOCUS_GRACE.as_secs_f32() {
                    self.handle_signal(now);
                }
            }
            WindowEvent::CloseRequested => self.hide_overlay(),
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
    Icon::from_rgba(flourish::icon::tray_rgba(SIZE), SIZE, SIZE)
}

/// Shows a fatal startup problem somewhere the user will actually see it.
///
/// A menu-bar app launched from Finder has no terminal attached, so `eprintln!`
/// alone means the app simply never appears and never says why.
fn show_fatal_dialog(headline: &str, detail: &str) {
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(headline)
        .set_description(detail)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let autostart = match cli::parse(std::env::args().skip(1)) {
        cli::Invocation::Run { autostart } => autostart,
        cli::Invocation::PrintAndExit(message) => {
            println!("{message}");
            return Ok(());
        }
        cli::Invocation::Fail(message) => {
            eprintln!("flourish: {message}");
            std::process::exit(2);
        }
    };
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

    // Reported here rather than from inside the loop, where a blocking modal
    // could hang instead of explaining itself.
    if let Some((headline, detail)) = app.fatal_error {
        show_fatal_dialog(&headline, &detail);
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod icon_tests {
    use super::make_tray_icon;

    #[test]
    fn the_tray_accepts_the_generated_icon() {
        // The art itself is covered in flourish::icon; this checks only that
        // the dimensions still satisfy tray-icon's RGBA constructor.
        assert!(make_tray_icon().is_ok());
    }
}
