//! This is a WINDOWS specific implementation for input related action.

use super::*;

use crossterm_utils::{TerminalOutput, Result};
use std::{char, io};
use std::thread;
use winapi::um::winnt::INT;
use crossterm_winapi::{ConsoleMode, Handle};

use std::mem::zeroed;
use std::io::{Error, ErrorKind, Result};
use crossterm_winapi::Handle;
use winapi::um::{
    consoleapi::ReadConsoleInputW,
    wincon::{
        INPUT_RECORD, KEY_EVENT, MOUSE_EVENT, KEY_EVENT_RECORD
        // , MOUSE_EVENT_RECORD
    },
};
use winapi::shared::minwindef::DWORD;


pub struct WindowsInput;

impl WindowsInput {
    pub fn new() -> WindowsInput {
        WindowsInput
    }
}

impl ITerminalInput for WindowsInput {
    fn read_char(&self, stdout: &Option<&Arc<TerminalOutput>>) -> io::Result<char> {
        let is_raw_screen = match stdout {
            Some(output) => output.is_in_raw_mode,
            None => false,
        };

        // _getwch is without echo and _getwche is with echo
        let pressed_char = unsafe {
            if is_raw_screen {
                _getwch()
            } else {
                _getwche()
            }
        };

        // we could return error but maybe option to keep listening until valid character is inputted.
        if pressed_char == 0 || pressed_char == 0xe0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Given input char is not a valid char, mostly occurs when pressing special keys",
            ));
        }

        match char::from_u32(pressed_char as u32) {
            Some(c) => {
                return Ok(c);
            }
            None => Err(io::Error::new(
                io::ErrorKind::Other,
                "Could not parse given input to char",
            )),
        }
    }

    fn read_async(&self, stdout: &Option<&Arc<TerminalOutput>>) -> AsyncReader {
        let (tx, rx) = mpsc::channel();

        // let is_raw_screen = match stdout {
        //     Some(output) => output.is_in_raw_mode,
        //     None => false,
        // };

        thread::spawn(move || {
            // TODO: drop this thread once finished
            loop {
                // // _getwch is without echo and _getwche is with echo
                // let pressed_char = unsafe {
                //     if is_raw_screen {
                //         _getwch()
                //     } else {
                //         _getwche()
                //     }
                // };

                // // we could return error but maybe option to keep listening until valid character is inputted.
                // if pressed_char == 0 || pressed_char == 0xe0 {
                //     return;
                // }

                // if let Err(_) = tx.send(Ok(pressed_char as u8)) {
                //     println!("Could not send pressed char to receiver.")
                // }

                for i in into_virtual_terminal_sequence().unwrap() {
                    if tx.send(Ok(i)).is_err() {
                        return;
                    }
                }

            }
        });

        AsyncReader { recv: rx }
    }

    fn read_until_async(
        &self,
        delimiter: u8,
        stdout: &Option<&Arc<TerminalOutput>>,
        ) -> AsyncReader {
        let (tx, rx) = mpsc::channel();

            // let is_raw_screen = match stdout {
            //     Some(output) => output.is_in_raw_mode,
            //     None => false,
            // };

            thread::spawn(move || {
                // TODO: drop this thread once finished
                loop {
                    // // _getwch is without echo and _getwche is with echo
                    // let pressed_char = unsafe {
                    //     if is_raw_screen {
                    //         _getwch()
                    //     } else {
                    //         _getwche()
                    //     }
                    // } as u8;

                    // let end_of_stream = pressed_char == delimiter;

                    // // we could return error but maybe option to keep listening until valid character is inputted.
                    // if pressed_char == 0 || pressed_char == 0xe0 || end_of_stream {
                    //     return;
                    // }

                    // if let Err(_) = tx.send(Ok(pressed_char as u8)) {
                    //     println!("Could not send pressed char to receiver.")
                    // }

                    for i in into_virtual_terminal_sequence().unwrap() {
                        if i == delimiter {
                            return;
                        } else {
                            if tx.send(Ok(i)).is_err() {
                                return;
                            }
                        }
                    }

                }
            });

            AsyncReader { recv: rx }
    }

    fn enable_mouse(&self, __stdout: &Option<&Arc<TerminalOutput>>) -> Result<()> {
        let console_mode = ConsoleMode::new()?;
        let dw_mode = console_mode.mode()?;
        let ENABLE_MOUSE_MODE = 0x0010 | 0x0080;
        let new_mode = dw_mode | ENABLE_MOUSE_MODE;
        console_mode.set_mode(new_mode)?;
        Ok(())
    }

    fn disable_mouse(&self, __stdout: &Option<&Arc<TerminalOutput>>) -> Result<()> {
        let console_mode = ConsoleMode::new()?;
        let dw_mode = console_mode.mode()?;
        let ENABLE_MOUSE_MODE = 0x0010 | 0x0080;
        let new_mode = dw_mode & !ENABLE_MOUSE_MODE;
        console_mode.set_mode(new_mode)?;
        Ok(())
    }
}

extern "C" {
    fn _getwche() -> INT;
    fn _getwch() -> INT;
}


/// https://github.com/retep998/wio-rs/blob/master/src/console.rs#L130
fn into_virtual_terminal_sequence() -> Result<Vec<u8>> {
    let handle = Handle::current_in_handle()?;
    // TODO: abstract this into a mode setting function for both *nix and Windows
    // let mut restore_mode: DWORD = 0;
    // let restore_mode_ptr: LPDWORD = &mut restore_mode;
    // if unsafe { GetConsoleMode(handle, restore_mode_ptr) } == 0 {
    //     return Err(Error::new(
    //         ErrorKind::Other, "Problem occurred getting the Console mode")
    //     )
    // }
    // let console_mode = ENABLE_WINDOW_INPUT | ENABLE_MOUSE_INPUT | ENABLE_EXTENDED_FLAGS;
    // if unsafe { SetConsoleMode(handle, console_mode) } == 0 {
    //     return Err(Error::new(
    //         ErrorKind::Other, "Problem occurred setting the Console mode")
    //     )
    // }

    // NOTE: confirm size of 0x1000
    let mut buf: [INPUT_RECORD; 0x1000] = unsafe { zeroed() };
    let mut size = 0;
    let res = unsafe { ReadConsoleInputW(handle, buf.as_mut_ptr(), buf.len() as DWORD, &mut size) };
    if res == 0 { 
        return Err(Error::new(
            ErrorKind::Other, "Problem occurred reading the Console input")
        )
    }

    let mut vts: Vec<u8> = Vec::new();

    for input in buf[..(size as usize)].iter() {
        unsafe {
            match input.EventType {
                KEY_EVENT => {
                    let e = input.Event.KeyEvent();
                    vts = handle_key_event(e);
                },
                MOUSE_EVENT => {
                    let e = input.Event.MouseEvent();
                    // TODO: handle mouse events
                    vts = Vec::new();
                },
                e => unreachable!("invalid event type: {}", e),
                // TODO (maybe): 
                // WINDOW_BUFFER_SIZE_EVENT => {
                //     let s = input.Event.WindowBufferSizeEvent().dwSize;
                //     Input::WindowBufferSize(s.X, s.Y)
                // },
                // MENU_EVENT => Input::Menu(input.Event.MenuEvent().dwCommandId),
                // FOCUS_EVENT => Input::Focus(input.Event.FocusEvent().bSetFocus != 0),
            }
        }
    };
    // TODO: abstract this into a mode setting function for both *nix and Windows
    // if unsafe { SetConsoleMode(handle, console_mode) } == 0 {
    //     return Err(Error::new(
    //         ErrorKind::Other, "Problem occurred setting the Console mode")
    //     )
    // }
    return Ok(vts);
}

fn handle_key_event(e: &KEY_EVENT_RECORD) -> Vec<u8> {
    let mut seq = Vec::new();
    let virtual_key = e.wVirtualKeyCode;
    match virtual_key {
        0x10 | 0x11 | 0x12 => {
            // ignore SHIFT, CTRL, ALT standalone presses
            seq.push(b'\x00');
        },
        0x70 | 0x71 | 0x72 | 0x73 | 0x74 => {
            // F1 - F5
            seq.push(b'\x1B');
            seq.push(b'[');
            seq.push(b'[');
            let f_key_values_one_to_five = [b'A', b'B', b'C', b'D', b'E'];
            seq.push(f_key_values_one_to_five[(virtual_key - 0x70) as usize]);
        },
        0x75 | 0x76 | 0x77 | 0x78 | 0x79 | 0x7A | 0x7B => {
            // TODO: handle F6 to F12
            // seq.push(b'\x1B');
            // seq.push(b'[');
            // NOTE (imdaveho): not sure if this is how parse_event()
            // handles remaining F keys
            // seq.push(b'0');
            // seq.push(b'~');
            seq.push(b'\x00');
        },
        0x25 | 0x26 | 0x27 | 0x28 => {
            // LEFT, UP, RIGHT, DOWN
            seq.push(b'\x1B');
            seq.push(b'[');
            let arrow_key_values = [b'D', b'A', b'C', b'B'];
            seq.push(arrow_key_values[(virtual_key - 0x25) as usize]);
        },
        0x21 | 0x22 | 0x23 | 0x24 => {
            // PAGEUP, PAGEDOWN, END, HOME
            // seq.push(b'\x1B');
            // seq.push(b'[');
            // NOTE (imdaveho): not sure if this is how parse_event()
            // handles remaining keys
            // seq.push(b'0');
            // seq.push(b'~');
            // linenums(254 - 263)
            seq.push(b'\x00');
        },
        0x2D | 0x2E => {
            // INSERT, DELETE
            // seq.push(b'\x1B');
            // seq.push(b'[');
            // NOTE (imdaveho): not sure if this is how parse_event()
            // handles remaining keys
            // seq.push(b'0');
            // seq.push(b'~');
            // linenums(254 - 263)
            seq.push(b'\x00');
        },
        _ => {
            // modifier key support
            // TODO: add support for acceptable symbols (semicolon, comma, plus, minus)
            let chars: [u8; 2] = {
                (unsafe {
                    *e.uChar.UnicodeChar()
                } as u16).to_ne_bytes()
            };
            match e.dwControlKeyState {
                0x0002 | 0x0101 | 0x0001 => {
                    // Alt + chr support
                    seq.push(b'\x1B');
                    for ch in chars.iter() {
                        seq.push(*ch);
                    };
                },
                0x0008 | 0x0104 | 0x0004 => {
                    // Ctrl + key support (only Ctrl + {a-z})
                    // NOTE (imdaveho): Ctrl + Shift + key support has same output
                    let alphabet: Vec<u8> = (b'\x01'..b'\x1B').collect();
                    for ch in chars.iter() {
                        // Constrain to only Aa-Zz keys
                        if alphabet.contains(&ch) {
                            seq.push(*ch);
                        } else {
                            seq.push(b'\x00');
                        }
                    };
                },
                0x000A | 0x0105 | 0x0005 => {
                    // Alt + Ctrl + key only contains Virt Key signatures
                    // TODO: update parse_rxvt() in events.rs 
                    // (linenums: 247 - 251) to handle multiple modifiers
                    // seq.push(b'\x1B');
                    // seq.push(b'[');
                    // NOTE (imdaveho): not sure if this is how parse_event()
                    // handles multiple modifiers
                    // seq.push(b'0');
                    // seq.push(b'~');
                    seq.push(b'\x00');
                },
                0x001A | 0x0115 | 0x0015 => {
                    // Alt + Ctrl + + Shift key only contains Virt Key signatures
                    // TODO: update parse_rxvt() in events.rs 
                    // (linenums: 247 - 251) to handle multiple modifiers
                    // seq.push(b'\x1B');
                    // seq.push(b'[');
                    // NOTE (imdaveho): not sure if this is how parse_event()
                    // handles multiple modifiers
                    // seq.push(b'0');
                    // seq.push(b'~');
                    seq.push(b'\x00');
                },
                0x0000 => {
                    // Single key press 
                    for ch in chars.iter() {
                        seq.push(*ch);
                    };
                },
                0x0010 => {
                    // Shift + key press
                    // Essentially the same as single key press
                    // separating to be explicit about the Shift press
                    // for Event enum
                    for ch in chars.iter() {
                        seq.push(*ch);
                    };
                },
                _ => {
                    // TODO: additional support?
                    seq.push(b'\x00');
                }
            }
        },
    };
    return seq;
}