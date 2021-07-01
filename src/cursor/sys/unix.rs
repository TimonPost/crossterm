use std::{
    io::{self, Error, ErrorKind, Write},
    time::Duration,
};

use crate::{
    cursor::CursorPosition,
    event::{filter::CursorPositionFilter, poll_internal, read_internal, InternalEvent},
    terminal::{disable_raw_mode, enable_raw_mode, sys::is_raw_mode_enabled},
    Result,
};

/// Returns the cursor position.
pub fn position() -> Result<CursorPosition> {
    if is_raw_mode_enabled() {
        read_position_raw()
    } else {
        read_position()
    }
}

fn read_position() -> Result<CursorPosition> {
    enable_raw_mode()?;
    let pos = read_position_raw();
    disable_raw_mode()?;
    pos
}

fn read_position_raw() -> Result<CursorPosition> {
    // Use `ESC [ 6 n` to and retrieve the cursor position.
    let mut stdout = io::stdout();
    stdout.write_all(b"\x1B[6n")?;
    stdout.flush()?;

    loop {
        match poll_internal(Some(Duration::from_millis(2000)), &CursorPositionFilter) {
            Ok(true) => {
                if let Ok(InternalEvent::CursorPosition(position)) =
                    read_internal(&CursorPositionFilter)
                {
                    return Ok(position);
                }
            }
            Ok(false) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "The cursor position could not be read within a normal duration",
                ));
            }
            Err(_) => {}
        }
    }
}
