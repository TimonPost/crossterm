macro_rules! run_tests {
    (
        $dst:expr,
        $(
            $testfn:ident
        ),*
        $(,)?
    ) => {
        use crossterm::{queue, style, terminal, cursor};
        $(
            queue!(
                $dst,
                style::ResetColor,
                terminal::Clear(terminal::ClearType::All),
                cursor::MoveTo(cursor::CursorPosition { column: 1, row: 1 }),
                cursor::Show,
                cursor::EnableBlinking
            )?;

            $testfn($dst)?;

            match $crate::read_char() {
                Ok('q') => return Ok(()),
                Err(e) => return Err(e),
                _ => { },
            };
        )*
    }
}
