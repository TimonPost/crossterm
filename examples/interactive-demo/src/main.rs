#![allow(clippy::cognitive_complexity)]

use std::io::{self, Write};

pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Command, Result,
};

#[macro_use]
mod macros;
mod test;

const MENU: &str = r#"Crossterm interactive test

Controls:

 - 'q' - quit interactive test (or return to this menu)
 - any other key - continue with next step

Available tests:

1. cursor
2. color (foreground, background)
3. attributes (bold, italic, ...)
4. input

Select test to run ('1', '2', ...) or hit 'q' to quit.
"#;

fn run<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    loop {
        queue!(
            w,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(cursor::CursorPosition { column: 1, row: 1 })
        )?;

        for line in MENU.split('\n') {
            queue!(w, style::Print(line), cursor::MoveToNextLine(1))?;
        }

        w.flush()?;

        match read_char()? {
            '1' => test::cursor::run(w)?,
            '2' => test::color::run(w)?,
            '3' => test::attribute::run(w)?,
            '4' => test::event::run(w)?,
            'q' => break,
            _ => {}
        };
    }

    execute!(
        w,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;

    terminal::disable_raw_mode()
}

pub fn read_char() -> Result<char> {
    loop {
        if let Ok(Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            ..
        })) = event::read()
        {
            return Ok(c);
        }
    }
}

pub fn buffer_size() -> Result<(u16, u16)> {
    terminal::size()
}

fn main() -> Result<()> {
    let mut stdout = io::stdout();
    run(&mut stdout)
}
