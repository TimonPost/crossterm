//! This module contains the logic to style an object that contains some state witch can be styled.

use super::{Color, ObjectStyle, ScreenManager};

use std::fmt::{self, Display};
use std::io::Write;

#[cfg(unix)]
use super::Attribute;

#[cfg(windows)]
use super::super::super::manager::WinApiScreenManager;

/// Struct that contains both the style and the content wits can be styled.
pub struct StyledObject<'terminal, D: Display> {
    pub object_style: ObjectStyle,
    pub content: D,
    pub screen_manager: &'terminal ScreenManager,
}

impl<'terminal, D: Display> StyledObject<'terminal, D> {
    /// Set the foreground of the styled object to the passed `Color`
    ///
    /// #Example
    ///
    /// ```rust
    /// extern crate crossterm;
    /// use self::crossterm::style::{paint,Color};
    ///
    /// // create an styled object with the foreground color red.
    /// let styledobject = paint("I am colored red").with(Color::Red);
    /// // create an styled object with the foreground color blue.
    /// let styledobject1 = paint("I am colored blue").with(Color::Blue);
    ///
    /// // print the styledobject to see the result
    /// println!("{}", styledobject);
    /// println!("{}", styledobject1);
    /// // print an styled object directly.
    /// println!("{}", paint("I am colored green").with(Color::Green));
    ///
    /// ```
    pub fn with(mut self, foreground_color: Color) -> StyledObject<'terminal, D> {
        self.object_style = self.object_style.fg(foreground_color);
        self
    }

    /// Set the background of the styled object to the passed `Color`
    ///
    /// #Example
    ///
    /// ```rust
    /// extern crate crossterm;
    /// use self::crossterm::style::{paint,Color};
    ///
    /// // create an styled object with the background color red.
    /// let styledobject = paint("I am colored red").on(Color::Red);
    /// // create an styled object with the background color blue.
    /// let styledobject1 = paint("I am colored blue").on(Color::Blue);
    ///
    /// // print the styledobjects
    /// println!("{}", styledobject);
    /// println!("{}", styledobject1);
    /// // print an styled object directly.
    /// println!("{}", paint("I am colored green").on(Color::Green))
    ///
    /// ```
    pub fn on(mut self, background_color: Color) -> StyledObject<'terminal, D> {
        self.object_style = self.object_style.bg(background_color);
        self
    }

    /// Set the attribute of an styled object to the passed `Attribute`
    ///
    /// #Example
    ///
    /// ```rust
    ///
    /// extern crate crossterm;
    /// use self::crossterm::style::{paint,Attribute};
    ///
    /// println!("{}", paint("Bold").attr(Attribute::Bold));
    ///
    /// ```
    #[cfg(unix)]
    pub fn attr(mut self, attr: Attribute) -> StyledObject<'terminal, D> {
        &self.object_style.add_attr(attr);
        self
    }

    /// Increase the font intensity.
    #[cfg(unix)]
    #[inline(always)]
    pub fn bold(self) -> StyledObject<'terminal, D> {
        self.attr(Attribute::Bold)
    }
    /// Faint (decreased intensity) (Not widely supported).
    #[cfg(unix)]
    #[inline(always)]
    pub fn dim(self) -> StyledObject<'terminal, D> {
        self.attr(Attribute::Dim)
    }
    /// Make the font italic (Not widely supported; Sometimes treated as inverse).
    #[cfg(unix)]
    #[inline(always)]
    pub fn italic(self) -> StyledObject<'terminal, D> {
        self.attr(Attribute::Italic)
    }
    /// Underline font.
    #[cfg(unix)]
    #[inline(always)]
    pub fn underlined(self) -> StyledObject<'terminal, D> {
        self.attr(Attribute::Underlined)
    }
    /// Slow Blink (less than 150 per minute; not widely supported).
    #[cfg(unix)]
    #[inline(always)]
    pub fn slow_blink(self) -> StyledObject<'terminal, D> {
        self.attr(Attribute::SlowBlink)
    }
    /// Rapid Blink (MS-DOS ANSI.SYS; 150+ per minute; not widely supported).
    #[cfg(unix)]
    #[inline(always)]
    pub fn rapid_blink(self) -> StyledObject<'terminal, D> {
        self.attr(Attribute::RapidBlink)
    }
    /// Swap foreground and background colors.
    #[cfg(unix)]
    #[inline(always)]
    pub fn reverse(self) -> StyledObject<'terminal, D> {
        self.attr(Attribute::Reverse)
    }
    /// Hide text (Not widely supported).
    #[cfg(unix)]
    #[inline(always)]
    pub fn hidden(self) -> StyledObject<'terminal, D> {
        self.attr(Attribute::Hidden)
    }
    /// Characters legible, but marked for deletion. Not widely supported.
    #[cfg(unix)]
    #[inline(always)]
    pub fn crossed_out(self) -> StyledObject<'terminal, D> {
        self.attr(Attribute::CrossedOut)
    }
}

impl<'terminal, D: Display> Display for StyledObject<'terminal, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut colored_terminal = super::super::super::style::color::color(self.screen_manager);
        let mut reset = true;

        if let Some(bg) = self.object_style.bg_color {
            colored_terminal.set_bg(bg);
            reset = true;
        }

        if let Some(fg) = self.object_style.fg_color {
            colored_terminal.set_fg(fg);
            reset = true;
        }

        #[cfg(unix)]
        for attr in self.object_style.attrs.iter() {
            self.screen_manager
                .write_string(format!(csi!("{}m"), *attr as i16));
            reset = true;
        }

        use std::fmt::Write;
        let mut string = String::new();
        write!(string, "{}", self.content).unwrap();
        self.screen_manager.write_string(string);
        self.screen_manager.flush();

        if reset {
            colored_terminal.reset();
        }

        Ok(())
    }
}
