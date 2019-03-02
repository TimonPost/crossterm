//! This is a WINDOWS specific implementation for input related action.

use super::*;

use crossterm_utils::{TerminalOutput};
use std::{char, io};
use std::thread;
use winapi::um::winnt::INT;
use crossterm_winapi::{Handle, is_true};

use std::mem::zeroed;
use std::io::{Error, ErrorKind};
use winapi::um::{
    consoleapi::{ReadConsoleInputW, GetConsoleMode, SetConsoleMode},
    wincon::{
        INPUT_RECORD, KEY_EVENT, KEY_EVENT_RECORD,
        MOUSE_EVENT, MOUSE_EVENT_RECORD,
        WINDOW_BUFFER_SIZE_EVENT, FOCUS_EVENT, MENU_EVENT
    },
};
use winapi::shared::minwindef::DWORD;


pub struct WindowsInput;

impl WindowsInput {
    pub fn new() -> WindowsInput {
        WindowsInput
    }
}

const ENABLE_MOUSE_MODE: u32 = 0x0010 | 0x0080 | 0x0008;
// NOTE (@imdaveho): this global var is terrible -> move it elsewhere...
static mut orig_mode: u32 = 0;

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

    fn read_async(&self, _stdout: &Option<&Arc<TerminalOutput>>) -> AsyncReader {
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
        _stdout: &Option<&Arc<TerminalOutput>>,
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

    fn enable_mouse_mode(&self, __stdout: &Option<&Arc<TerminalOutput>>) -> io::Result<()> {
        let handle = Handle::current_in_handle()?;
        unsafe {
            if !is_true(GetConsoleMode(handle, &mut orig_mode)) {
                println!("Getting mode failed");
                return Err(Error::last_os_error());
            }
        }
        
        let new_mode = ENABLE_MOUSE_MODE;
        unsafe {
            if !is_true(SetConsoleMode(handle, new_mode)) {
                return Err(Error::last_os_error());
            }
        }
        Ok(())
    }

    fn disable_mouse_mode(&self, __stdout: &Option<&Arc<TerminalOutput>>) -> io::Result<()> {
        let handle = Handle::current_in_handle()?;
        let dw_mode: Result<u32> = {
            let mut console_mode = 0;
            unsafe {
                if !is_true(GetConsoleMode(handle, &mut console_mode)) {
                    println!("Getting mode failed");
                    return Err(Error::last_os_error());
                }
            }
            Ok(console_mode)
        };
        unsafe {
            if !is_true(SetConsoleMode(handle, orig_mode)) {
                return Err(Error::last_os_error());
            }
        }
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
                    if e.bKeyDown == 0 {
                        // NOTE (@imdaveho): only handle key down
                        // this is because unix limits key events to key press
                        continue
                    }
                    vts = handle_key_event(e);
                },
                MOUSE_EVENT => {
                    let e = input.Event.MouseEvent();
                    // TODO: handle mouse events
                    // println!("{:?}", e.dwButtonState);
                    vts = handle_mouse_event(e);
                },
                // NOTE (@imdaveho): ignore below
                WINDOW_BUFFER_SIZE_EVENT => (),
                FOCUS_EVENT => (),
                MENU_EVENT => (),
                e => unreachable!("invalid event type: {}", e),
            }
        }
    };
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
        0x08 => {
            // BACKSPACE 
            seq.push(b'\x7F'); 
        },
        0x1B => { 
            // ESC
            seq.push(b'\x1B');
        },
        0x0D => {
            // ENTER
            seq.push(b'\n');
        },
        0x70 | 0x71 | 0x72 | 0x73 => {
            // F1 - F4 are support by default VT100
            seq.push(b'\x1B');
            seq.push(b'O');
            seq.push([b'P', b'Q', b'R', b'S']
                     [(virtual_key - 0x70) as usize]);
        },
        0x74 | 0x75 | 0x76 | 0x77 => {
            // NOTE: F Key Escape Codes:
            // http://aperiodic.net/phil/archives/Geekery/term-function-keys.html
            // https://docs.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences
            // F5 - F8
            seq.push(b'\x1B');
            seq.push(b'[');
            seq.push(b'1');
            seq.push([b'5', b'7', b'8', b'9']
                     [(virtual_key - 0x74) as usize]);
            seq.push(b'~');
        },
        0x78 | 0x79 | 0x7A | 0x7B => {
            // F9 - F12
            seq.push(b'\x1B');
            seq.push(b'[');
            seq.push(b'2');
            seq.push([b'0', b'1', b'3', b'4']
                     [(virtual_key - 0x78) as usize]);
            seq.push(b'~');
        },
        0x25 | 0x26 | 0x27 | 0x28 => {
            // LEFT, UP, RIGHT, DOWN
            seq.push(b'\x1B');
            seq.push(b'[');
            seq.push([b'D', b'A', b'C', b'B']
                     [(virtual_key - 0x25) as usize]);
        },
        0x21 | 0x22 => {
            // PAGEUP, PAGEDOWN
            seq.push(b'\x1B');
            seq.push(b'[');
            seq.push([b'5', b'6']
                     [(virtual_key - 0x21) as usize]);
            seq.push(b'~');
        },
        0x23 | 0x24 => {
            // END, HOME
            seq.push(b'\x1B');
            seq.push(b'[');
            seq.push([b'F', b'H']
                     [(virtual_key - 0x23) as usize]);
        },
        0x2D | 0x2E => {
            // INSERT, DELETE
            seq.push(b'\x1B');
            seq.push(b'[');
            seq.push([b'2', b'3']
                     [(virtual_key - 0x2D) as usize]);
            seq.push(b'~');
        },
        _ => {
            // Modifier Keys (Ctrl, Alt, Shift) Support
            // NOTE (@imdaveho): test to check if characters outside of
            // alphabet or alphanumerics are supported
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
                    // NOTE (@imdaveho): Ctrl + Shift + key support has same output
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
                    // TODO: Alt + Ctrl + Key support
                    // mainly updating the Alt section of parse_event()
                    // and updating parse_utf8_char()
                    seq.push(b'\x00');
                },
                0x001A | 0x0115 | 0x0015 => {
                    // TODO: Alt + Ctrl + Shift Key support
                    // mainly updating the Alt section of parse_event()
                    // and updating parse_utf8_char()
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
                    seq.push(b'\x00');
                }
            }
        },
    };
    return seq;
}

fn handle_mouse_event(e: &MOUSE_EVENT_RECORD) -> Vec<u8> {
    let mut seq = Vec::new();
    let button = e.dwButtonState;
    let movemt = e.dwEventFlags;
    let coords = e.dwMousePosition; // NOTE (@imdaveho) coords can be larger than u8 (255)
    let cx = coords.X as u8;
    let cy = coords.Y as u8;
    match movemt {
        0x0 => {
            // Single click
            match button {
                0 => {
                    // release
                    // seq = vec![b'\x1B', b'[', b'<', b'\x03', b';', cx, b';', cy, b';', b'm'];
                    seq.push(b'\x1B');
                    seq.push(b'[');
                    seq.push(b'M');
                    seq.push(3 + 32);
                    seq.push(cx);
                    seq.push(cy);
                }
                1 => {
                    // left click
                    // seq = vec![b'\x1B', b'[', b'<', b'\x00', b';', cx, b';', cy, b';', b'M'];
                    seq.push(b'\x1B');
                    seq.push(b'[');
                    seq.push(b'M');
                    seq.push(0 + 32);
                    seq.push(cx);
                    seq.push(cy);
                },
                2 => {
                    // right click
                    // seq = vec![b'\x1B', b'[', b'<', b'\x02', b';', cx, b';', cy, b';', b'M'];
                    seq.push(b'\x1B');
                    seq.push(b'[');
                    seq.push(b'M');                    
                    seq.push(2 + 32);
                    seq.push(cx);
                    seq.push(cy);
                },
                4 => {
                    // middle click
                    // seq = vec![b'\x1B', b'[', b'<', b'\x01', b';', cx, b';', cy, b';', b'M'];
                    seq.push(b'\x1B');
                    seq.push(b'[');
                    seq.push(b'M');                    
                    seq.push(1 + 32);
                    seq.push(cx);
                    seq.push(cy);
                }
                _ => (),
            }
        },
        0x1 => {
            // Move
            // seq = vec![b'\x1B', b'[', b'<', 32, cx, cy, b'M'];
            ()
            // seq.push(b'\x1B');
            // seq.push(b'[');
            // seq.push(b'<');
            // seq.push(32);
            // seq.push(b';');
            // seq.push(cx);
            // seq.push(b';');
            // seq.push(cy);
            // seq.push(b';');
            // seq.push(b'M');
        },
        0x4 => {
            // Vertical scroll
            if button >= 0 {
                // WheelUp
                // seq = vec![b'\x1B', b'[', b'<', 64, b';', cx, b';', cy, b';', b'M'];
                seq.push(b'\x1B');
                seq.push(b'[');
                seq.push(b'M');
                seq.push(0);
                seq.push(cx);
                seq.push(cy);
            } else {
                // WheelDown
                // seq = vec![b'\x1B', b'[', b'<', 65, b';', cx, b';', cy, b';', b'M'];
                seq.push(b'\x1B');
                seq.push(b'[');
                seq.push(b'M');
                seq.push(1);
                seq.push(cx);
                seq.push(cy);
            }
        },
        0x2 => (), // NOTE (@imdaveho): double click not supported by unix terminals
        0x8 => (), // NOTE (@imdaveho): horizontal scroll not supported by unix terminals
        _ => (), // TODO: Handle Ctrl + Mouse, Alt + Mouse, etc.
    };
    return seq;
}