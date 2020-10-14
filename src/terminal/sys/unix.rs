//! UNIX related logic for terminal manipulation.
use std::{io, mem, process, sync::Mutex};

use crate::event::sys::unix::file_descriptor::{tty_fd, FileDesc};
use lazy_static::lazy_static;
use libc::{
    cfmakeraw, ioctl, tcgetattr, tcsetattr, termios as Termios, winsize, STDOUT_FILENO, TCSANOW,
    TIOCGWINSZ,
};

use crate::error::{ErrorKind, Result};
use std::fs::File;
use std::os::unix::io::IntoRawFd;

lazy_static! {
    // Some(Termios) -> we're in the raw mode and this is the previous mode
    // None -> we're not in the raw mode
    static ref TERMINAL_MODE_PRIOR_RAW_MODE: Mutex<Option<Termios>> = Mutex::new(None);
}

pub(crate) fn is_raw_mode_enabled() -> bool {
    TERMINAL_MODE_PRIOR_RAW_MODE.lock().unwrap().is_some()
}

#[allow(clippy::useless_conversion)]
pub(crate) fn size() -> Result<(u16, u16)> {
    // http://rosettacode.org/wiki/Terminal_control/Dimensions#Library:_BSD_libc
    let mut size = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let file = File::open("/dev/tty").map(|file| (FileDesc::new(file.into_raw_fd(), true)));
    let fd = if let Ok(file) = &file {
        file.raw_fd()
    } else {
        // Fallback to libc::STDOUT_FILENO if /dev/tty is missing
        STDOUT_FILENO
    };

    if let Ok(true) = wrap_with_result(unsafe { ioctl(fd, TIOCGWINSZ.into(), &mut size) }) {
        Ok((size.ws_col, size.ws_row))
    } else {
        tput_size().ok_or_else(|| std::io::Error::last_os_error().into())
    }
}

pub(crate) fn enable_raw_mode() -> Result<()> {
    let mut original_mode = TERMINAL_MODE_PRIOR_RAW_MODE.lock().unwrap();

    if original_mode.is_some() {
        return Ok(());
    }

    let mut ios = get_terminal_attr()?;
    let original_mode_ios = ios;

    raw_terminal_attr(&mut ios);
    set_terminal_attr(&ios)?;

    // Keep it last - set the original mode only if we were able to switch to the raw mode
    *original_mode = Some(original_mode_ios);

    Ok(())
}

pub(crate) fn disable_raw_mode() -> Result<()> {
    let mut original_mode = TERMINAL_MODE_PRIOR_RAW_MODE.lock().unwrap();

    if let Some(original_mode_ios) = original_mode.as_ref() {
        set_terminal_attr(original_mode_ios)?;
        // Keep it last - remove the original mode only if we were able to switch back
        *original_mode = None;
    }

    Ok(())
}

/// execute tput with the given argument and parse
/// the output as a u16.
///
/// The arg should be "cols" or "lines"
fn tput_value(arg: &str) -> Option<u16> {
    match process::Command::new("tput").arg(arg).output() {
        Ok(process::Output { stdout, .. }) => {
            let value = stdout
                .iter()
                .map(|&b| b as u16)
                .take_while(|&b| b >= 48 && b <= 58)
                .fold(0, |v, b| v * 10 + (b - 48));
            if value > 0 {
                Some(value)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Returns the size of the screen as determined by tput.
///
/// This alternate way of computing the size is useful
/// when in a subshell.
fn tput_size() -> Option<(u16, u16)> {
    match (tput_value("cols"), tput_value("lines")) {
        (Some(w), Some(h)) => Some((w, h)),
        _ => None,
    }
}

// Transform the given mode into an raw mode (non-canonical) mode.
fn raw_terminal_attr(termios: &mut Termios) {
    unsafe { cfmakeraw(termios) }
}

fn get_terminal_attr() -> Result<Termios> {
    let fd = tty_fd(true)?;

    unsafe {
        let mut termios = mem::zeroed();
        wrap_with_result(tcgetattr(fd.raw_fd(), &mut termios))?;
        Ok(termios)
    }
}

fn set_terminal_attr(termios: &Termios) -> Result<bool> {
    let fd = tty_fd(true)?;
    wrap_with_result(unsafe { tcsetattr(fd.raw_fd(), TCSANOW, termios) })
}

pub fn wrap_with_result(result: i32) -> Result<bool> {
    if result == -1 {
        Err(ErrorKind::IoError(io::Error::last_os_error()))
    } else {
        Ok(true)
    }
}
