//! A module that contains all the actions related to cursor movement in the terminal.
//! Like: moving the cursor position; saving and resetting the cursor position; hiding showing and control the blinking of the cursor.

use super::*;

use crossterm_utils::{Command, Result};

#[cfg(windows)]
use crossterm_utils::supports_ansi;

/// Allows you to preform actions with the terminal cursor.
///
/// # Features:
///
/// - Moving n times Up, Down, Left, Right
/// - Goto a certain position
/// - Get cursor position
/// - Storing the current cursor position and resetting to that stored cursor position later
/// - Hiding and showing the cursor
/// - Control over blinking of the terminal cursor (only some terminals are supporting this)
///
/// Note that positions of the cursor are 0 -based witch means that the coordinates (cells) starts counting from 0
///
/// Check `/examples/cursor` in the library for more specific examples.
pub struct TerminalCursor {
    #[cfg(windows)]
    cursor: Box<(dyn ITerminalCursor + Sync + Send)>,
    #[cfg(unix)]
    cursor: AnsiCursor,
}

impl TerminalCursor {
    /// Create new `TerminalCursor` instance whereon cursor related actions can be performed.
    pub fn new() -> TerminalCursor {
        #[cfg(windows)]
        let cursor = if supports_ansi() {
            Box::from(AnsiCursor::new()) as Box<(dyn ITerminalCursor + Sync + Send)>
        } else {
            WinApiCursor::new() as Box<(dyn ITerminalCursor + Sync + Send)>
        };

        #[cfg(unix)]
        let cursor = AnsiCursor::new();

        TerminalCursor { cursor }
    }

    /// Goto some position (x,y) in the terminal.
    ///
    /// # Remarks
    /// position is 0-based, which means we start counting at 0.
    pub fn goto(&self, x: u16, y: u16) -> Result<()> {
        self.cursor.goto(x, y)
    }

    /// Get current cursor position (x,y) in the terminal.
    ///
    /// # Remarks
    /// position is 0-based, which means we start counting at 0.
    pub fn pos(&self) -> (u16, u16) {
        self.cursor.pos()
    }

    /// Move the current cursor position `n` times up.
    pub fn move_up(&mut self, count: u16) -> &mut TerminalCursor {
        self.cursor.move_up(count).unwrap();
        self
    }

    /// Move the current cursor position `n` times right.
    pub fn move_right(&mut self, count: u16) -> &mut TerminalCursor {
        self.cursor.move_right(count).unwrap();
        self
    }

    /// Move the current cursor position `n` times down.
    pub fn move_down(&mut self, count: u16) -> &mut TerminalCursor {
        self.cursor.move_down(count).unwrap();
        self
    }

    /// Move the current cursor position `n` times left.
    pub fn move_left(&mut self, count: u16) -> &mut TerminalCursor {
        self.cursor.move_left(count).unwrap();
        self
    }

    /// Save cursor position for recall later.
    ///
    /// Note that this position is stored program based not per instance of the `Cursor` struct.
    pub fn save_position(&self) -> Result<()> {
        self.cursor.save_position()
    }

    /// Return to saved cursor position
    pub fn reset_position(&self) -> Result<()> {
        self.cursor.reset_position()
    }

    /// Hide de cursor in the console.
    pub fn hide(&self) -> Result<()> {
        self.cursor.hide()
    }

    /// Show the cursor in the console.
    pub fn show(&self) -> Result<()> {
        self.cursor.show()
    }

    /// Enable or disable blinking of the terminal.
    ///
    /// # Remarks
    /// Not all terminals are supporting this functionality. Windows versions lower than windows 10 also are not supporting this version.
    pub fn blink(&self, blink: bool) -> Result<()> {
        self.cursor.blink(blink)
    }
}

/// Get a `TerminalCursor` instance whereon cursor related actions can be performed.
pub fn cursor() -> TerminalCursor {
    TerminalCursor::new()
}

pub struct Goto(pub u16, pub u16);

impl Command for Goto {
    type AnsiType = String;

    fn get_ansi_code(&self) -> Self::AnsiType {
        ansi_cursor::get_goto_ansi(self.0, self.1)
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> Result<()> {
        WinApiCursor::new().goto(self.0, self.1)
    }
}

pub struct UP(pub u16);

impl Command for UP {
    type AnsiType = String;

    fn get_ansi_code(&self) -> Self::AnsiType {
        ansi_cursor::get_move_up_ansi(self.0)
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> Result<()> {
        WinApiCursor::new().move_up(self.0)
    }
}

pub struct Down(pub u16);

impl Command for Down {
    type AnsiType = String;

    fn get_ansi_code(&self) -> Self::AnsiType {
        ansi_cursor::get_move_down_ansi(self.0)
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> Result<()> {
        WinApiCursor::new().move_down(self.0)
    }
}

pub struct Left(pub u16);

impl Command for Left {
    type AnsiType = String;

    fn get_ansi_code(&self) -> Self::AnsiType {
        ansi_cursor::get_move_left_ansi(self.0)
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> Result<()> {
        WinApiCursor::new().move_left(self.0)
    }
}

pub struct Right(pub u16);

impl Command for Right {
    type AnsiType = String;

    fn get_ansi_code(&self) -> Self::AnsiType {
        ansi_cursor::get_move_right_ansi(self.0)
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> Result<()> {
        WinApiCursor::new().move_right(self.0)
    }
}

pub struct SavePos;

impl Command for SavePos {
    type AnsiType = &'static str;

    fn get_ansi_code(&self) -> Self::AnsiType {
        ansi_cursor::SAFE_POS_ANSI
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> Result<()> {
        WinApiCursor::new().save_position()
    }
}

pub struct ResetPos;

impl Command for ResetPos {
    type AnsiType = &'static str;

    fn get_ansi_code(&self) -> Self::AnsiType {
        ansi_cursor::RESET_POS_ANSI
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> Result<()> {
        WinApiCursor::new().reset_position()
    }
}

pub struct Hide;

impl Command for Hide {
    type AnsiType = &'static str;

    fn get_ansi_code(&self) -> Self::AnsiType {
        ansi_cursor::HIDE_ANSI
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> Result<()> {
        WinApiCursor::new().hide()
    }
}

pub struct Show;

impl Command for Show {
    type AnsiType = &'static str;

    fn get_ansi_code(&self) -> Self::AnsiType {
        ansi_cursor::SHOW_ANSI
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> Result<()> {
        WinApiCursor::new().show()
    }
}

pub struct BlinkOn;

impl Command for BlinkOn {
    type AnsiType = &'static str;

    fn get_ansi_code(&self) -> Self::AnsiType {
        ansi_cursor::BLINK_ON_ANSI
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> Result<()> {
        Ok(())
    }
}

pub struct BlinkOf;

impl Command for BlinkOf {
    type AnsiType = &'static str;

    fn get_ansi_code(&self) -> Self::AnsiType {
        ansi_cursor::BLINK_OFF_ANSI
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> Result<()> {
        Ok(())
    }
}
