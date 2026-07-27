mod benchmark;
mod cli;
mod doom_fire;
mod frames;
mod gravel;
mod hotkey;
mod renderer;
mod target;

use std::{sync::Arc, time::Duration, time::Instant};

use flourish::{
    Choice, Flourish, MotionPreference, SignalResult, Timeline, TimelineUpdate, motion,
};
use hotkey::HotkeyBinding;
use renderer::{FlourishRenderer, Frame, RenderOutcome};
use target::Target;
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
    /// Replayed by the global shortcut. A choice rather than an effect, so
    /// that Surprise Me keeps surprising instead of pinning its first draw.
    last_choice: Choice,
    /// The last flourish actually played, which a surprise will not repeat.
    /// Outlives `active_effect`, which is cleared the moment one is dismissed.
    previous_effect: Option<Flourish>,
    window: Option<Arc<Window>>,
    renderer: Option<FlourishRenderer>,
    tray: Option<TrayIcon>,
    hotkey: Option<HotkeyBinding>,
    effect_menu_ids: Vec<(MenuId, Flourish)>,
    surprise_menu_id: Option<MenuId>,
    quit_menu_id: Option<MenuId>,
    motion: MotionPreference,
    motion_menu_id: Option<MenuId>,
    motion_menu_item: Option<MenuItem>,
    proxy: EventLoopProxy<UserEvent>,
    autostart: Option<Choice>,
    /// Report the display layout and exit instead of running.
    describe_displays: bool,
    /// Set when startup fails; reported by `main` after the loop unwinds.
    fatal_error: Option<(String, String)>,
}

impl App {
    fn new(
        proxy: EventLoopProxy<UserEvent>,
        autostart: Option<Choice>,
        motion: MotionPreference,
    ) -> Self {
        Self {
            started_at: Instant::now(),
            timeline: Timeline::idle(),
            active_effect: None,
            last_choice: autostart.unwrap_or(Choice::Effect(Flourish::Curtain)),
            previous_effect: None,
            window: None,
            renderer: None,
            tray: None,
            hotkey: None,
            effect_menu_ids: Vec::new(),
            surprise_menu_id: None,
            quit_menu_id: None,
            motion,
            motion_menu_id: None,
            motion_menu_item: None,
            proxy,
            autostart,
            describe_displays: false,
            fatal_error: None,
        }
    }

    fn now(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Plays whatever `choice` settles on, and remembers the choice itself.
    ///
    /// Remembering the choice rather than its outcome is what makes Surprise Me
    /// sticky: the shortcut repeats the intent, so it draws a new flourish each
    /// press instead of pinning whichever one came up first.
    fn play_choice(&mut self, event_loop: &ActiveEventLoop, choice: Choice) {
        self.last_choice = choice;
        let effect = choice.resolve(self.previous_effect);
        self.start_effect(event_loop, effect);
    }

    /// Starts `effect`, replacing whatever is on screen.
    ///
    /// Choosing from the menu or pressing the hotkey is an explicit request for
    /// that flourish. Treating it as a dismissal instead — as this once did —
    /// silently discarded the presenter's actual choice.
    fn start_effect(&mut self, event_loop: &ActiveEventLoop, effect: Flourish) {
        // Chosen fresh each time: the presenter may have moved to another
        // screen since the last flourish.
        if let Some(target) = target::choose(event_loop) {
            if target.basis == target::Basis::PrimaryFallback {
                eprintln!("Flourish could not locate the pointer; using the primary display");
            }
            self.present_on(&target);
        }

        let now = self.now();
        self.timeline = Timeline::new(effect.exit_duration(), effect.hold_limit());
        self.timeline.start(now);
        self.active_effect = Some(effect);
        self.previous_effect = Some(effect);
        // After present_on, so a warm-up sizes itself to the display the
        // flourish will actually appear on.
        if let Some(renderer) = &mut self.renderer {
            renderer.start_effect(effect, self.motion);
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    /// The hotkey both summons and dismisses: pressing it during a flourish
    /// advances that flourish's exit rather than restarting it.
    fn toggle_via_hotkey(&mut self, event_loop: &ActiveEventLoop) {
        if self.timeline.is_active() {
            self.handle_signal(self.now());
        } else {
            // Resolved here rather than remembered, so a sticky surprise draws
            // a new flourish on every press.
            self.play_choice(event_loop, self.last_choice);
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
        // Ordered after hiding so nothing flashes as the display is released.
        self.release_display();
        self.active_effect = None;
    }

    fn create_tray(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let menu = Menu::new();
        // First, because with seventeen effects the point of the menu is
        // usually "give me one" rather than "give me that one".
        let surprise = MenuItem::new("Surprise Me", true, None);
        menu.append(&surprise)?;
        self.surprise_menu_id = Some(surprise.id().clone());
        menu.append(&PredefinedMenuItem::separator())?;

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

        // Reachable from the menu because the person who needs it is often not
        // the person who configured the machine: a presenter may only learn an
        // audience member needs stillness once they are already on stage.
        let motion_item = MenuItem::new(self.motion.menu_label(), true, None);
        menu.append(&motion_item)?;
        self.motion_menu_id = Some(motion_item.id().clone());
        self.motion_menu_item = Some(motion_item);

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
        // Deliberately not full-screen at construction. The display a flourish
        // belongs on is only known when one is asked for, so the window is
        // placed and made full-screen at that moment instead -- see
        // `move_to_target`.
        let attributes = Window::default_attributes()
            .with_title("Flourish")
            .with_visible(false)
            .with_transparent(true)
            .with_decorations(false)
            // Resizable so the window can be re-sized onto another display.
            // With no decorations there is nothing for a user to drag anyway.
            .with_resizable(true)
            .with_window_level(WindowLevel::AlwaysOnTop);

        #[cfg(target_os = "windows")]
        let attributes = {
            use winit::platform::windows::WindowAttributesExtWindows;
            attributes.with_skip_taskbar(true)
        };

        let window = Arc::new(event_loop.create_window(attributes)?);
        let renderer = pollster::block_on(FlourishRenderer::new(Arc::clone(&window)))?;
        self.window = Some(window);
        self.renderer = Some(renderer);
        Ok(())
    }

    /// Puts the overlay on `target`, shows it, and makes it cover that display.
    ///
    /// Called before every flourish, because the presenter may have moved to a
    /// different screen since the last one.
    ///
    /// The order here is load-bearing on macOS. Simple full-screen resizes the
    /// window to whatever `NSWindow.screen` reports, and that is resolved from
    /// the window's own frame — so the window has to be released from
    /// full-screen, moved onto the target, and made visible *before* it is
    /// engaged again. Doing it while the window is hidden or still sitting on
    /// the previous display sends the flourish back to the old screen.
    ///
    /// Native full-screen is deliberately not used: it animates into its own
    /// Space, which is the opposite of what an instant overlay wants.
    fn present_on(&mut self, target: &Target) {
        let Some(window) = self.window.clone() else {
            return;
        };

        window.set_cursor_visible(false);
        flourish::display::place_overlay(&window, target.bounds, &target.monitor);
        window.focus_window();

        // Only now is the surface size final: full-screen may have adjusted it,
        // and the new display may differ in scale from the last one.
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(window.inner_size());
        }
        window.request_redraw();
    }

    /// Releases full-screen so the overlay stops holding the display, restoring
    /// the menu bar and Dock that simple full-screen suppresses.
    fn release_display(&self) {
        if let Some(window) = &self.window {
            flourish::display::release_overlay(window);
        }
    }

    /// Turns the timeline's state into the three numbers the renderer needs.
    ///
    /// This is the whole difference between the two motion paths. Full motion
    /// animates the effect clock and drives each flourish's own geometric exit.
    /// Reduced motion holds a settled composition and moves nothing at all,
    /// carrying both the entrance and the exit in opacity.
    fn frame_for(&self, now: Duration) -> Frame {
        compose_frame(
            self.motion,
            self.timeline.effect_time(now),
            self.timeline.exit_progress(now).unwrap_or(0.0),
        )
    }

    fn set_motion(&mut self, motion: MotionPreference) {
        self.motion = motion;
        if let Some(item) = &self.motion_menu_item {
            item.set_text(motion.menu_label());
        }
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

        let frame = self.frame_for(now);
        let Some(effect) = self.active_effect else {
            self.hide_overlay();
            return;
        };
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        match renderer.render(effect, frame) {
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
        if self.describe_displays {
            let monitors: Vec<_> = event_loop.available_monitors().collect();
            print!(
                "{}",
                target::describe(&monitors, event_loop.primary_monitor())
            );
            event_loop.exit();
            return;
        }

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

        if let Some(choice) = self.autostart {
            self.play_choice(event_loop, choice);
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Hotkey => self.toggle_via_hotkey(event_loop),
            UserEvent::Menu(id) if self.quit_menu_id.as_ref() == Some(&id) => {
                event_loop.exit();
            }
            UserEvent::Menu(id) if self.motion_menu_id.as_ref() == Some(&id) => {
                self.set_motion(self.motion.toggled());
            }
            UserEvent::Menu(id) if self.surprise_menu_id.as_ref() == Some(&id) => {
                self.play_choice(event_loop, Choice::Surprise);
            }
            UserEvent::Menu(id) => {
                let selected = self
                    .effect_menu_ids
                    .iter()
                    .find_map(|(menu_id, effect)| (menu_id == &id).then_some(*effect));
                if let Some(effect) = selected {
                    self.play_choice(event_loop, Choice::Effect(effect));
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

/// Turns the timeline's state into the three numbers the renderer needs.
///
/// This is the entire difference between the two motion paths. Full motion
/// advances the effect clock and drives each flourish's own geometric exit.
/// Reduced motion holds a settled composition, moves nothing, and carries both
/// the entrance and the exit in opacity alone.
///
/// Free-standing so the fade curve can be tested without standing up a window.
fn compose_frame(motion: MotionPreference, effect_time: f32, exit_progress: f32) -> Frame {
    if !motion.is_reduced() {
        return Frame::animated(effect_time, exit_progress);
    }

    let fade_in = (effect_time / motion::CALM_FADE_IN.as_secs_f32()).clamp(0.0, 1.0);
    let fade_out = 1.0 - exit_progress.clamp(0.0, 1.0);
    Frame::calm(fade_in * fade_out)
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
    let invocation = cli::parse(std::env::args().skip(1));
    let (autostart, requested_motion) = match invocation {
        cli::Invocation::Run { autostart, motion } => (autostart, motion),
        // Deferred until after the event loop exists, since enumerating
        // monitors requires it.
        cli::Invocation::DescribeDisplays => (None, None),
        // Offscreen, so it needs no event loop, window, or display at all.
        cli::Invocation::Benchmark => {
            benchmark::run()?;
            return Ok(());
        }
        // Also offscreen: the whole point is to see the catalog without one
        // taking over the display.
        cli::Invocation::Frames(directory) => {
            frames::run(&directory)?;
            return Ok(());
        }
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

    // An explicit flag wins; otherwise ask the system.
    let motion = requested_motion.unwrap_or_else(motion::detect);
    let proxy = event_loop.create_proxy();
    let mut app = App::new(proxy, autostart, motion);
    // Monitors are only enumerable from a running loop, so the diagnostic is a
    // mode of the app: it reports and exits before opening any window or tray.
    app.describe_displays = matches!(invocation, cli::Invocation::DescribeDisplays);
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

#[cfg(test)]
mod motion_tests {
    use super::compose_frame;
    use flourish::{MotionPreference, motion};

    const FADE: f32 = 0.42;

    #[test]
    fn full_motion_passes_the_clock_and_the_exit_straight_through() {
        let frame = compose_frame(MotionPreference::Full, 3.0, 0.4);

        assert!((frame.time - 3.0).abs() < f32::EPSILON);
        assert!((frame.exit_progress - 0.4).abs() < f32::EPSILON);
        assert!((frame.presence - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reduced_motion_never_advances_the_clock_or_the_geometric_exit() {
        // Whatever the timeline says, the calm path must move nothing. These
        // two fields are the only ones that produce movement on screen.
        for effect_time in [0.0, 0.2, 1.0, 9.0, 900.0] {
            for exit_progress in [0.0, 0.25, 0.5, 1.0] {
                let frame = compose_frame(MotionPreference::Reduced, effect_time, exit_progress);
                assert!(
                    frame.exit_progress.abs() < f32::EPSILON,
                    "geometric exit leaked at t={effect_time} exit={exit_progress}"
                );
                assert!(
                    (frame.time - motion::SETTLED_SECONDS).abs() < f32::EPSILON,
                    "clock advanced at t={effect_time}"
                );
            }
        }
    }

    #[test]
    fn a_calm_flourish_fades_in_rather_than_cutting_in() {
        // Cutting straight to a full-screen image is itself an abrupt visual
        // change, so presence must start at zero and climb.
        let at = |t| compose_frame(MotionPreference::Reduced, t, 0.0).presence;

        assert!(at(0.0).abs() < f32::EPSILON);
        assert!(at(FADE / 2.0) > 0.4 && at(FADE / 2.0) < 0.6);
        assert!((at(FADE) - 1.0).abs() < f32::EPSILON);
        assert!(
            (at(FADE * 10.0) - 1.0).abs() < f32::EPSILON,
            "must not exceed one"
        );
    }

    #[test]
    fn a_calm_flourish_fades_out_to_nothing() {
        let held = |exit| compose_frame(MotionPreference::Reduced, 5.0, exit).presence;

        assert!((held(0.0) - 1.0).abs() < f32::EPSILON);
        assert!((held(0.5) - 0.5).abs() < 0.001);
        assert!(held(1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn calm_presence_is_monotonic_and_bounded() {
        // A non-monotonic opacity ramp would read as a flicker, which is
        // exactly what reduced motion exists to avoid.
        let mut previous = 0.0;
        for step in 0..=100_u16 {
            let presence =
                compose_frame(MotionPreference::Reduced, f32::from(step) * 0.01, 0.0).presence;
            assert!(
                (0.0..=1.0).contains(&presence),
                "presence {presence} left the unit range"
            );
            assert!(presence >= previous, "presence dipped at step {step}");
            previous = presence;
        }
    }
}
