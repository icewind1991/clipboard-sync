use clipboard::{ClipboardContext, ClipboardProvider};
#[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
use smithay_clipboard::WaylandClipboard;
use std::error::Error;

#[cfg(not(all(unix, not(any(target_os = "macos", target_os = "android")))))]
pub enum WaylandClipboard {}

pub enum ClipboardHandler {
    Wayland(WaylandClipboard),
    Rest(ClipboardContext),
}

impl ClipboardHandler {
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    fn new_wayland() -> Result<WaylandClipboard, ()> {
        use smithay_client_toolkit::reexports::client::Display;
        let (display, _) = Display::connect_to_env().map_err(|_| ())?;
        Ok(WaylandClipboard::new(&display))
    }

    #[cfg(not(all(unix, not(any(target_os = "macos", target_os = "android")))))]
    fn new_wayland() -> Result<WaylandClipboard, ()> {
        Err(())
    }
}

impl ClipboardProvider for ClipboardHandler {
    fn new() -> Result<Self, Box<dyn Error>> {
        Ok(match Self::new_wayland() {
            Ok(wayland_clipboard) => ClipboardHandler::Wayland(wayland_clipboard),
            Err(_) => ClipboardHandler::Rest(ClipboardContext::new()?),
        })
    }

    fn get_contents(&mut self) -> Result<String, Box<dyn Error>> {
        Ok(match self {
            ClipboardHandler::Wayland(clip) => clip.load(None)?,
            ClipboardHandler::Rest(clip) => clip.get_contents()?,
        })
    }

    fn set_contents(&mut self, content: String) -> Result<(), Box<dyn Error>> {
        Ok(match self {
            ClipboardHandler::Wayland(clip) => clip.store(None, content),
            ClipboardHandler::Rest(clip) => clip.set_contents(content)?,
        })
    }
}
