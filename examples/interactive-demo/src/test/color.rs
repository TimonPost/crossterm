#![allow(clippy::cognitive_complexity)]

use crate::Result;
use crossterm::{cursor, queue, style, style::Color};
use std::io::Write;

const COLORS: [Color; 21] = [
    Color::Black,
    Color::DarkGrey,
    Color::Grey,
    Color::White,
    Color::DarkRed,
    Color::Red,
    Color::DarkGreen,
    Color::Green,
    Color::DarkYellow,
    Color::Yellow,
    Color::DarkBlue,
    Color::Blue,
    Color::DarkMagenta,
    Color::Magenta,
    Color::DarkCyan,
    Color::Cyan,
    Color::AnsiValue(0),
    Color::AnsiValue(15),
    Color::Rgb { r: 255, g: 0, b: 0 },
    Color::Rgb { r: 0, g: 255, b: 0 },
    Color::Rgb { r: 0, g: 0, b: 255 },
];

fn test_set_foreground_color<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    queue!(
        w,
        style::Print("Foreground colors on the black & white background"),
        cursor::MoveToNextLine(2)
    )?;

    for color in &COLORS {
        queue!(
            w,
            style::SetForegroundColor(*color),
            style::SetBackgroundColor(Color::Black),
            style::Print(format!(
                "{:>width$} ",
                format!("{:?} ████████████", color),
                width = 40
            )),
            style::SetBackgroundColor(Color::White),
            style::Print(format!(
                "{:>width$}",
                format!("{:?} ████████████", color),
                width = 40
            )),
            cursor::MoveToNextLine(1)
        )?;
    }

    w.flush()?;

    Ok(())
}

fn test_set_background_color<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    queue!(
        w,
        style::Print("Background colors with black & white foreground"),
        cursor::MoveToNextLine(2)
    )?;

    for color in &COLORS {
        queue!(
            w,
            style::SetBackgroundColor(*color),
            style::SetForegroundColor(Color::Black),
            style::Print(format!(
                "{:>width$} ",
                format!("{:?} ▒▒▒▒▒▒▒▒▒▒▒▒", color),
                width = 40
            )),
            style::SetForegroundColor(Color::White),
            style::Print(format!(
                "{:>width$}",
                format!("{:?} ▒▒▒▒▒▒▒▒▒▒▒▒", color),
                width = 40
            )),
            cursor::MoveToNextLine(1)
        )?;
    }

    w.flush()?;

    Ok(())
}

fn test_color_values_matrix_16x16<W, F>(w: &mut W, title: &str, color: F) -> Result<()>
where
    W: Write,
    F: Fn(u16, u16) -> Color,
{
    queue!(w, style::Print(title))?;

    for idx in 0..=15 {
        queue!(
            w,
            cursor::MoveTo(cursor::CursorPosition {
                column: 1,
                row: idx + 4
            }),
            style::Print(format!("{:>width$}", idx, width = 2))
        )?;
        queue!(
            w,
            cursor::MoveTo(idx * 3 + 3, 3),
            style::Print(format!("{:>width$}", idx, width = 3))
        )?;
    }

    for row in 0..=15u16 {
        queue!(
            w,
            cursor::MoveTo(cursor::CursorPosition {
                column: 4,
                row: row + 4
            })
        )?;
        for col in 0..=15u16 {
            queue!(
                w,
                style::SetForegroundColor(color(col, row)),
                style::Print("███")
            )?;
        }
        queue!(
            w,
            style::SetForegroundColor(Color::White),
            style::Print(format!("{:>width$} ..= ", row * 16, width = 3)),
            style::Print(format!("{:>width$}", row * 16 + 15, width = 3))
        )?;
    }

    w.flush()?;

    Ok(())
}

fn test_color_ansi_values<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    test_color_values_matrix_16x16(w, "Color::Ansi values", |col, row| {
        Color::AnsiValue((row * 16 + col) as u8)
    })
}

fn test_rgb_red_values<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    test_color_values_matrix_16x16(w, "Color::Rgb red values", |col, row| Color::Rgb {
        r: (row * 16 + col) as u8,
        g: 0 as u8,
        b: 0,
    })
}

fn test_rgb_green_values<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    test_color_values_matrix_16x16(w, "Color::Rgb green values", |col, row| Color::Rgb {
        r: 0,
        g: (row * 16 + col) as u8,
        b: 0,
    })
}

fn test_rgb_blue_values<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    test_color_values_matrix_16x16(w, "Color::Rgb blue values", |col, row| Color::Rgb {
        r: 0,
        g: 0,
        b: (row * 16 + col) as u8,
    })
}

pub fn run<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    run_tests!(
        w,
        test_set_foreground_color,
        test_set_background_color,
        test_color_ansi_values,
        test_rgb_red_values,
        test_rgb_green_values,
        test_rgb_blue_values,
    );
    Ok(())
}
