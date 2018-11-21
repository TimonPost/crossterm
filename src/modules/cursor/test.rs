use modules::cursor::ansi_cursor::AnsiCursor;

use modules::cursor::ITerminalCursor;

use Screen;

/* ======================== WinApi =========================== */
#[cfg(windows)]
mod winapi_tests {
    use super::*;
    use modules::cursor::winapi_cursor::WinApiCursor;

    #[test]
    fn goto_winapi() {
        let screen = Screen::default();
        let stdout = Some(&screen.stdout);
        let cursor = WinApiCursor::new();

        cursor.goto(5, 5, &stdout);
        let (x, y) = cursor.pos();

        assert_eq!(x, 5);
        assert_eq!(y, 5);
    }

    #[test]
    fn reset_safe_winapi() {
        let screen = Screen::default();
        let stdout = Some(&screen.stdout);
        let cursor = WinApiCursor::new();
        let (x, y) = cursor.pos();

        cursor.save_position(&stdout);
        cursor.goto(5, 5, &stdout);
        cursor.reset_position(&stdout);

        let (x_saved, y_saved) = cursor.pos();

        assert_eq!(x, x_saved);
        assert_eq!(y, y_saved);
    }
}

/* ======================== ANSI =========================== */
#[test]
fn reset_safe_ansi() {
    if try_enable_ansi() {
        let screen = Screen::default();
        let stdout = Some(&screen.stdout);
        let cursor = AnsiCursor::new();
        let (x, y) = cursor.pos();

        cursor.save_position(&stdout);
        cursor.goto(5, 5, &stdout);
        cursor.reset_position(&stdout);

        let (x_saved, y_saved) = cursor.pos();

        assert_eq!(x, x_saved);
        assert_eq!(y, y_saved);
    }
}

#[test]
fn goto_ansi() {
    if try_enable_ansi() {
        let screen = Screen::default();
        let stdout = Some(&screen.stdout);
        let cursor = AnsiCursor::new();

        cursor.goto(5, 5, &stdout);
        let (x, y) = cursor.pos();

        assert_eq!(x, 5);
        assert_eq!(y, 5);
    }
}

fn try_enable_ansi() -> bool {
    #[cfg(windows)]
    {
        if cfg!(target_os = "windows") {
            use kernel::windows_kernel::ansi_support::try_enable_ansi_support;

            if !try_enable_ansi_support() {
                return false;
            }
        }
    }

    return true;
}
