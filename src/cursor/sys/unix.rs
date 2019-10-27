//! UNIX related logic to cursor manipulation.

use std::io::{self, Write};

use crate::input::{InputEvent, TerminalInput};
use crate::utils::{
    sys::unix::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled},
    Result,
};

/// Returns the cursor position (column, row).
///
/// The top left cell is represented `0,0`.
pub fn position() -> Result<(u16, u16)> {
    if is_raw_mode_enabled() {
        read_position_raw()
    } else {
        read_position()
    }
}

fn read_position() -> Result<(u16, u16)> {
    enable_raw_mode()?;
    let pos = read_position_raw();
    disable_raw_mode()?;
    pos
}

fn read_position_raw() -> Result<(u16, u16)> {
    // Use `ESC [ 6 n` to and retrieve the cursor position.
    let mut stdout = io::stdout();

    stdout.write_all(b"\x1B[6n")?;
    stdout.flush()?;

    let mut reader = TerminalInput::new().read_sync();

    loop {
        if let Some(InputEvent::CursorPosition(x, y)) = reader.next() {
            return Ok((x, y));
        }
    }
}
