use clipboard::{ClipboardContext, ClipboardProvider};
use failure::Fail;
use std::error::Error;
#[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
use std::io::Read;
#[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
use wl_clipboard_rs::{
    copy::{self, copy, Options, Source},
    paste::{get_contents, ClipboardType, MimeType, Seat},
};

pub enum ClipboardHandler {
    Wayland,
    Rest(ClipboardContext),
}

impl ClipboardHandler {
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    fn is_wayland() -> bool {
        get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text).is_ok()
    }

    #[cfg(not(all(unix, not(any(target_os = "macos", target_os = "android")))))]
    fn is_wayland() -> bool {
        false
    }

    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    fn read_wayland() -> Result<String, Box<dyn Error>> {
        let (mut pipe, _) = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text)
            .map_err(|f| f.compat())?;
        let mut contents = vec![];
        pipe.read_to_end(&mut contents)?;
        Ok(String::from_utf8(contents)?)
    }

    #[cfg(not(all(unix, not(any(target_os = "macos", target_os = "android")))))]
    fn is_wayland() -> Result<String, Box<dyn Error>> {
        unreachable!()
    }

    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    fn write_wayland(content: String) -> Result<(), Box<dyn Error>> {
        Ok(copy(
            Options::default(),
            Source::Bytes(content.into_bytes().into()),
            copy::MimeType::Text,
        )
        .map_err(|f| f.compat())?)
    }

    #[cfg(not(all(unix, not(any(target_os = "macos", target_os = "android")))))]
    fn write_wayland(content: String) -> Result<String, Box<dyn Error>> {
        unreachable!()
    }
}

impl ClipboardProvider for ClipboardHandler {
    fn new() -> Result<Self, Box<dyn Error>> {
        Ok(match Self::is_wayland() {
            true => ClipboardHandler::Wayland,
            false => ClipboardHandler::Rest(ClipboardContext::new()?),
        })
    }

    fn get_contents(&mut self) -> Result<String, Box<dyn Error>> {
        Ok(match self {
            ClipboardHandler::Wayland => Self::read_wayland()?,
            ClipboardHandler::Rest(clip) => clip.get_contents()?,
        })
    }

    fn set_contents(&mut self, content: String) -> Result<(), Box<dyn Error>> {
        Ok(match self {
            ClipboardHandler::Wayland => Self::write_wayland(content)?,
            ClipboardHandler::Rest(clip) => clip.set_contents(content)?,
        })
    }
}
