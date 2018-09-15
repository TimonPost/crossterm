//! This is an ANSI specific implementation for the screen write
//! This module is used for windows 10 terminals and unix terminals by default.
//! This module uses the stdout to write to the console.

use super::IStdout;

use std::any::Any;
use std::io::{self, Write,Stdout, stdout};

/// This struct is a wrapper for `Stdout`
pub struct AnsiOutput {
    pub handle: Stdout,
}

impl IStdout for AnsiOutput {
   fn write_str(&self, string: &str) -> io::Result<usize> {
        let out = &self.handle;
        let mut handle = out.lock();
        let amt = handle.write(string.as_bytes())?;
        handle.flush()?;
        Ok(amt)
    }

    fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let out = &self.handle;
        let mut handle = out.lock();
        handle.write(buf)
    }

    fn flush(&self) -> io::Result<()> {
        let out = &self.handle;
        let mut handle = out.lock();
        handle.flush()
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}

impl AnsiOutput {
    pub fn new() -> Self {
        AnsiOutput { handle: stdout() }
    }
}
