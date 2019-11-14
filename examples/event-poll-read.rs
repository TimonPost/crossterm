//
// cargo run --example event-poll-read
//
use std::io::{stdout, Write};
use std::time::Duration;

use crossterm::{
    cursor::position,
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    screen::RawScreen,
    Result,
};

const HELP: &str = r#"Blocking poll() & non-blocking read()
 - Keyboard, mouse and terminal resize events enabled
 - Prints "." every second if there's no event 
 - Hit "c" to print current cursor position
 - Use Esc to quit
"#;

fn print_events() -> Result<()> {
    loop {
        // Wait up to 1s for another event
        if poll(Some(Duration::from_millis(1_000)))? {
            // It's guaranteed that read() wont block if `poll` returns `Ok(true)`
            let event = read()?;

            println!("Event::{:?}\r", event);

            if event == Event::Key(KeyCode::Char('c').into()) {
                println!("Cursor position: {:?}\r", position());
            }

            if event == Event::Key(KeyCode::Esc.into()) {
                break;
            }
        } else {
            // Timeout expired, no event for 1s
            println!(".\r");
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("{}", HELP);

    let _r = RawScreen::into_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture)?;

    if let Err(e) = print_events() {
        println!("Error: {:?}\r", e);
    }

    execute!(stdout, DisableMouseCapture)?;
    Ok(())
}
