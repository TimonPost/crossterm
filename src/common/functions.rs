//! Some actions need to preformed platform independently since they can not be solved `ANSI escape codes`.

use super::ScreenManager;

#[cfg(windows)]
use kernel::windows_kernel::terminal::{buffer_size, exit, terminal_size};

#[cfg(windows)]
use kernel::windows_kernel::cursor::pos;

#[cfg(unix)]
use kernel::unix_kernel::terminal::{exit, pos, terminal_size};

/// Get the terminal size based on the current platform.
pub fn get_terminal_size() -> (u16, u16) {
    #[cfg(unix)]
    return terminal_size();

    #[cfg(windows)]
    return terminal_size();
}

/// Get the cursor position based on the current platform.
pub fn get_cursor_position(screen_manager: &ScreenManager) -> (u16, u16) {
    #[cfg(unix)]
    return pos();

    #[cfg(windows)]
    return pos(screen_manager);
}

/// exit the current terminal.
pub fn exit_terminal() {
    #[cfg(unix)]
    exit();

    #[cfg(windows)]
    exit();
}

#[cfg(windows)]
/// Get an module specific implementation of a the generic given type based on the current platform.
/// If the current platform is windows and it supports ansi escape codes it will return the ansi implementation and if not it will return the winapi implementation.
/// If the current platform is unix it will return the ansi implementation.
pub fn get_module<T>(winapi_impl: T, unix_impl: T) -> Option<T> {
    let mut term: Option<T> = None;
    let mut does_support = true;

    if cfg!(target_os = "windows") {
        #[cfg(windows)]
        use kernel::windows_kernel::ansi_support::try_enable_ansi_support;

        //   Try to enable ansi on windows if not than use WINAPI.
        does_support = try_enable_ansi_support();

        //        does_support = false;
        if !does_support {
            term = Some(winapi_impl);
        }
    }

    if does_support {
        term = Some(unix_impl);
    }

    term
}
