//! With this module you can perform actions that are input related.
//! Like reading a line, reading a character and reading asynchronously.

use std::io;

use super::*;

/// Struct that stores an specific platform implementation for input related actions.
///
/// Check `/examples/version/input` the examples folder on github for more info.
///
/// #Example
///
/// ```rust
///
/// extern crate crossterm;
/// use self::crossterm::Crossterm;
///
/// let crossterm = Crossterm::new();
/// let input = crossterm.input();
///
/// let result = input.read_line();
/// let pressed_char = input.read_char();
///
/// ```
pub struct TerminalInput<'terminal> {
    terminal_input: Box<ITerminalInput>,
    screen_manager: &'terminal ScreenManager,
}

impl<'terminal> TerminalInput<'terminal> {
    /// Create new instance of TerminalInput whereon input related actions could be preformed.
    pub fn new(screen_manager: &'terminal ScreenManager) -> TerminalInput<'terminal> {
        #[cfg(target_os = "windows")]
        let input = Box::from(WindowsInput::new());

        #[cfg(not(target_os = "windows"))]
        let input = Box::from(UnixInput::new());

        TerminalInput {
            terminal_input: input,
            screen_manager: screen_manager,
        }
    }

    /// Read one line from the user input.
    ///
    /// #Example
    ///
    /// ```rust
    ///  let crossterm = Crossterm::new();
    ///  match crossterm.input().read_line() {
    ///     Ok(s) => println!("string typed: {}", s),
    ///     Err(e) => println!("error: {}", e),
    ///  }
    ///
    /// ```
    pub fn read_line(&self) -> io::Result<String> {
        self.terminal_input.read_line(&self.screen_manager)
    }

    /// Read one character from the user input
    ///
    /// #Example
    ///
    /// ```rust
    ///  let crossterm = Crossterm::new();
    ///  match crossterm.input().read_char() {
    ///     Ok(c) => println!("character pressed: {}", c),
    ///     Err(e) => println!("error: {}", e),
    //   }
    ///
    /// ```
    pub fn read_char(&self) -> io::Result<char> {
        return self.terminal_input.read_char(&self.screen_manager);
    }

    /// Read the input asynchronously from the user.
    ///
    /// #Example
    ///
    /// ```rust
    /// let crossterm = Crossterm::new();
    ///
    /// // we need to enable raw mode otherwise the characters will be outputted by default before we are able to read them.
    /// crossterm.enable_raw_mode();
    ///
    /// let mut stdin = crossterm.input().read_async().bytes();
    ///
    /// for i in 0..100 {
    ///
    ///     // Get the next character typed. This is None if nothing is pressed. And Some(Ok(u8 value of character))
    ///     let a = stdin.next();
    ///
    ///     println!("pressed key: {:?}", a);
    ///
    ///     if let Some(Ok(b'x')) = a {
    ///         println!("The key: `x` was pressed and program is terminated.");
    ///         break;
    ///     }
    ///     // simulate some timeout so that we can see the character on the screen.
    ///     thread::sleep(time::Duration::from_millis(50));
    /// }
    ///
    /// ```
    pub fn read_async(&self) -> AsyncReader {
        self.terminal_input.read_async(&self.screen_manager)
    }

    ///  Read the input asynchronously until a certain character is hit.
    ///
    /// #Example
    ///
    /// ```rust
    /// let crossterm = Crossterm::new();
    /// let input = crossterm.input();
    /// let terminal = crossterm.terminal();
    /// let mut cursor = crossterm.cursor();
    ///
    /// // we need to enable raw mode otherwise the characters will be outputted by default before we are able to read them.
    /// crossterm.enable_raw_mode();
    ///
    /// let mut stdin = input.read_until_async(b'\r').bytes();
    ///
    /// for i in 0..100 {
    ///     terminal.clear(ClearType::All);
    ///     cursor.goto(1, 1);
    ///     let a = stdin.next();
    ///
    ///     println!("pressed key: {:?}", a);
    ///
    ///     if let Some(Ok(b'\r')) = a {
    ///         println!("The enter key is hit and program is not listening to input anymore.");
    ///         break;
    ///     }
    ///
    ///     if let Some(Ok(b'x')) = a {
    ///          println!("The key: x was pressed and program is terminated.");
    ///         break;
    ///     }
    ///
    ///     thread::sleep(time::Duration::from_millis(100));
    /// }
    /// ```
    pub fn read_until_async(&self, delimiter: u8) -> AsyncReader {
        self.terminal_input
            .read_until_async(delimiter, &self.screen_manager)
    }
}

/// Get an Terminal Input implementation whereon input related actions can be performed.
pub fn input(screen_manager: &ScreenManager) -> TerminalInput {
    return TerminalInput::new(screen_manager);
}
