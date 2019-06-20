//! A module which provides some functionalities to work with the terminal screen.
//! Like allowing you to switch between the main and alternate screen or putting the terminal into raw mode.
#[macro_use]
extern crate crossterm_utils;

#[cfg(windows)]
extern crate winapi;

#[cfg(windows)]
extern crate crossterm_winapi;

mod screen;
mod sys;

pub use self::screen::{AlternateScreen, IntoRawMode, RawScreen};
