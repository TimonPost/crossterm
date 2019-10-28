use std::fmt::Display;

// TODO Should be removed? This adds just another way to achieve the same thing.
/// A crossterm functionality wrapper.
pub struct Crossterm;

impl Crossterm {
    /// Creates a new `Crossterm`.
    pub fn new() -> Crossterm {
        Crossterm
    }

    /// Creates a new `TerminalInput`.
    #[cfg(feature = "input")]
    pub fn input(&self) -> crate::input::TerminalInput {
        crate::input::TerminalInput::new()
    }

    /// Creates a new `TerminalColor`.
    #[cfg(feature = "style")]
    pub fn color(&self) -> crate::style::TerminalColor {
        crate::style::TerminalColor::new()
    }

    /// Creates a new `StyledContent`.
    #[cfg(feature = "style")]
    pub fn style<D>(&self, val: D) -> crate::style::StyledContent<D>
    where
        D: Display + Clone,
    {
        crate::style::ContentStyle::new().apply(val)
    }
}
