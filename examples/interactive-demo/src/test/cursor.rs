#![allow(clippy::cognitive_complexity)]

use std::io::Write;

use crate::Result;
use crossterm::{cursor, execute, queue, style, style::Stylize, Command};
use std::thread;
use std::time::Duration;

fn test_move_cursor_up<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    draw_cursor_box(w, "Move Up (2)", |_, _| cursor::MoveUp(2))
}

fn test_move_cursor_down<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    draw_cursor_box(w, "Move Down (2)", |_, _| cursor::MoveDown(2))
}

fn test_move_cursor_left<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    draw_cursor_box(w, "Move Left (2)", |_, _| cursor::MoveLeft(2))
}

fn test_move_cursor_right<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    draw_cursor_box(w, "Move Right (2)", |_, _| cursor::MoveRight(2))
}

fn test_move_cursor_to_previous_line<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    draw_cursor_box(w, "MoveToPreviousLine (2)", |_, _| {
        cursor::MoveToPreviousLine(2)
    })
}

fn test_move_cursor_to_next_line<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    draw_cursor_box(w, "MoveToNextLine (2)", |_, _| cursor::MoveToNextLine(2))
}

fn test_move_cursor_to_column<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    draw_cursor_box(w, "MoveToColumn (2)", |center_x, _| {
        cursor::MoveToColumn(center_x + 2)
    })
}

fn test_hide_cursor<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    execute!(w, style::Print("HideCursor"), cursor::Hide)
}

fn test_show_cursor<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    execute!(w, style::Print("ShowCursor"), cursor::Show)
}

fn test_enable_cursor_blinking<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    execute!(
        w,
        style::Print("EnableCursorBlinking"),
        cursor::EnableBlinking
    )
}

fn test_disable_cursor_blinking<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    execute!(
        w,
        style::Print("DisableCursorBlinking"),
        cursor::DisableBlinking
    )
}

fn test_move_cursor_to<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    draw_cursor_box(
        w,
        "MoveTo (x: 1, y: 1) removed from center",
        |center_x, center_y| cursor::MoveTo(center_x + 1, center_y + 1),
    )
}

fn test_save_restore_cursor_position<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    execute!(w,
        cursor::MoveTo(CursorPosition { column: 0, row: 0 }),
        style::Print("Save position, print character else were, after three seconds restore to old position."),
        cursor::MoveToNextLine(2),
        style::Print("Save ->[ ]<- Position"),
        cursor::MoveTo(CursorPosition { column: 8, row: 2 }),
        cursor::SavePosition,
        cursor::MoveTo(CursorPosition { column: 10, row: 10 }),
        style::Print("Move To ->[√]<- Position")
    )?;

    thread::sleep(Duration::from_secs(3));

    execute!(w, cursor::RestorePosition, style::Print("√"))
}

/// Draws  a box with an colored center, this center can be taken as a reference point after running the given cursor command.
fn draw_cursor_box<W, F, T>(w: &mut W, description: &str, cursor_command: F) -> Result<()>
where
    W: Write,
    F: Fn(u16, u16) -> T,
    T: Command,
{
    execute!(
        w,
        cursor::Hide,
        cursor::MoveTo(CursorPosition { column: 0, row: 0 }),
        style::SetForegroundColor(style::Color::Red),
        style::Print(format!(
            "Red box is the center. After the action: '{}' another box is drawn.",
            description
        ))
    )?;

    let start_y = 2;
    let width = 21;
    let height = 11 + start_y;
    let center_x = width / 2;
    let center_y = (height + start_y) / 2;

    for row in start_y..=10 + start_y {
        for column in 0..=width {
            if (row == start_y || row == height - 1) || (column == 0 || column == width) {
                queue!(
                    w,
                    cursor::MoveTo(column, row),
                    style::PrintStyledContent("▓".red())
                )?;
            } else {
                queue!(
                    w,
                    cursor::MoveTo(column, row),
                    style::PrintStyledContent("_".red().on_white())
                )?;
            }
        }
    }

    queue!(
        w,
        cursor::MoveTo(center_x, center_y),
        style::PrintStyledContent("▀".red().on_white())
    )?;
    queue!(
        w,
        cursor_command(center_x, center_y),
        style::PrintStyledContent("√".magenta().on_white())
    )?;
    w.flush()?;
    Ok(())
}

pub fn run<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    run_tests!(
        w,
        test_hide_cursor,
        test_show_cursor,
        test_enable_cursor_blinking,
        test_disable_cursor_blinking,
        test_move_cursor_left,
        test_move_cursor_right,
        test_move_cursor_up,
        test_move_cursor_down,
        test_move_cursor_to,
        test_move_cursor_to_next_line,
        test_move_cursor_to_previous_line,
        test_move_cursor_to_column,
        test_save_restore_cursor_position
    );
    Ok(())
}
