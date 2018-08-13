extern crate crossterm;

use self::crossterm::input::input;
use self::crossterm::Screen;

pub fn read_char() {
    let input = input(&Screen::default());

    match input.read_char() {
        Ok(s) => println!("char typed: {}", s),
        Err(e) => println!("char error : {}", e),
    }
}

pub fn read_line() {
    let input = input(&Screen::default());

    match input.read_line() {
        Ok(s) => println!("string typed: {}", s),
        Err(e) => println!("error: {}", e),
    }
}
