//! The global shortcut.
//!
//! Reaching the menu bar means dropping out of a full-screen deck, which is
//! exactly the moment a presenter cannot afford it. A system-wide shortcut lets
//! a flourish be summoned and dismissed without ever leaving the slides.

use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use winit::event_loop::EventLoopProxy;

use crate::UserEvent;

/// Human-readable form of the shortcut, for the menu and for error messages.
#[cfg(target_os = "macos")]
pub const DESCRIPTION: &str = "⌃⌥⌘F";
#[cfg(not(target_os = "macos"))]
pub const DESCRIPTION: &str = "Ctrl+Alt+Shift+F";

/// Chosen to be conspicuously unlikely to collide with a presentation tool's
/// own bindings; Keynote, `PowerPoint`, and Google Slides all leave it alone.
#[cfg(target_os = "macos")]
fn binding() -> HotKey {
    HotKey::new(
        Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META),
        Code::KeyF,
    )
}

#[cfg(not(target_os = "macos"))]
fn binding() -> HotKey {
    HotKey::new(
        Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT),
        Code::KeyF,
    )
}

/// A live global shortcut registration.
///
/// The manager must outlive the registration, so this owns it; dropping the
/// binding releases the shortcut back to the system.
pub struct HotkeyBinding {
    _manager: GlobalHotKeyManager,
}

impl HotkeyBinding {
    /// Registers the shortcut and routes presses onto the winit event loop.
    ///
    /// Fails when another application already owns the combination, which is
    /// recoverable: the caller should carry on with the menu bar alone.
    pub fn register(proxy: EventLoopProxy<UserEvent>) -> Result<Self, Box<dyn std::error::Error>> {
        let manager = GlobalHotKeyManager::new()?;
        manager.register(binding())?;

        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            // Fire on release only; holding the key would otherwise repeat.
            if event.state == global_hotkey::HotKeyState::Released {
                let _ = proxy.send_event(UserEvent::Hotkey);
            }
        }));

        Ok(Self { _manager: manager })
    }
}

#[cfg(test)]
mod tests {
    use super::{DESCRIPTION, binding};

    #[test]
    fn the_shortcut_requires_multiple_modifiers() {
        // A single-modifier global shortcut would shadow a key combination
        // some other application legitimately wants.
        let hotkey = binding();
        let modifiers = hotkey.mods.bits().count_ones();
        assert!(
            modifiers >= 3,
            "expected a hard-to-collide shortcut, got {modifiers} modifier(s)"
        );
    }

    #[test]
    fn the_description_is_presentable() {
        assert!(!DESCRIPTION.is_empty());
        assert!(!DESCRIPTION.contains("Code::"));
    }
}
