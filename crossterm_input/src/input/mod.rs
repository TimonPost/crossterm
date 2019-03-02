//! A module that contains all the actions related to reading input from the terminal.
//! Like reading a line, reading a character and reading asynchronously.

mod input;

#[cfg(not(target_os = "windows"))]
mod unix_input;
#[cfg(target_os = "windows")]
mod windows_input;

#[cfg(not(target_os = "windows"))]
use self::unix_input::UnixInput;
#[cfg(target_os = "windows")]
use self::windows_input::WindowsInput;

pub use self::input::{input, TerminalInput, parse_event};

use std::io::{self, Read};
use std::sync::{mpsc, Arc};

use crossterm_utils::{TerminalOutput};

/// This trait defines the actions that can be preformed with the terminal input.
/// This trait can be implemented so that a concrete implementation of the ITerminalInput can fulfill
/// the wishes to work on a specific platform.
///
/// ## For example:
///
/// This trait is implemented for Windows and UNIX systems.
/// Unix is using the 'TTY' and windows is using 'libc' C functions to read the input.
trait ITerminalInput {
    /// Read one character from the user input
    fn read_char(&self, stdout: &Option<&Arc<TerminalOutput>>) -> io::Result<char>;
    /// Read the input asynchronously from the user.
    fn read_async(&self, stdout: &Option<&Arc<TerminalOutput>>) -> AsyncReader;
    ///  Read the input asynchronously until a certain character is hit.
    fn read_until_async(&self, delimiter: u8, stdout: &Option<&Arc<TerminalOutput>>)
        -> AsyncReader;
    fn enable_mouse_mode(&self, stdout: &Option<&Arc<TerminalOutput>>) -> crossterm_utils::Result<()>;
    fn disable_mouse_mode(&self, stdout: &Option<&Arc<TerminalOutput>>) -> crossterm_utils::Result<()>;
}

/// This is a wrapper for reading from the input asynchronously.
/// This wrapper has a channel receiver that receives the input from the user whenever it typed something.
/// You only need to check whether there are new characters available.
pub struct AsyncReader {
    recv: mpsc::Receiver<io::Result<u8>>,
}

/// This enum represents key events which could be caused by the user.
// pub enum KeyEvent {
//     /// Represents a specific key press.
//     OnKeyPress(u8),
//     /// Represents a key press from any key.
//     OnAnyKeyPress,
//     /// Represents a key press from enter.
//     OnEnter,
// }

pub enum InputEvent {
    Keyboard(KeyEvent),
    Mouse(MouseEvent),
    Unsupported(Vec<u8>),
    Unknown,
}

pub enum MouseEvent {
    Press(MouseButton, u16, u16),
    Release(u16, u16),
    Hold(u16, u16),
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
    WheelUp,
    WheelDown,
}

pub enum KeyEvent {
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Alt(char),
    Ctrl(char),
    Null,
    Esc,
}

impl Read for AsyncReader {
    /// Read from the byte stream.
    ///
    /// This will never block, but try to drain the event queue until empty. If the total number of
    /// bytes written is lower than the buffer's length, the event queue is empty or that the event
    /// stream halted.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut total = 0;

        loop {
            if total >= buf.len() {
                break;
            }

            match self.recv.try_recv() {
                Ok(Ok(value)) => {
                    buf[total] = value;
                    total += 1;
                }
                Ok(Err(e)) => return Err(e),
                Err(_) => break,
            }
        }

        Ok(total)
    }
}
