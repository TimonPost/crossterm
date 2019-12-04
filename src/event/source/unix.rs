use std::collections::VecDeque;
use std::{io, time::Duration};

use mio::{unix::EventedFd, Events, Poll, PollOpt, Ready, Token};
use signal_hook::iterator::Signals;

use crate::Result;

use super::super::{
    source::EventSource,
    sys::unix::{parse_event, tty_fd, FileDesc},
    timeout::PollTimeout,
    Event, InternalEvent,
};

// Tokens to identify file descriptor
const TTY_TOKEN: Token = Token(0);
const SIGNAL_TOKEN: Token = Token(1);
const WAKE_TOKEN: Token = Token(2);

// I (@zrzka) wasn't able to read more than 1_022 bytes when testing
// reading on macOS/Linux -> we don't need bigger buffer and 1k of bytes
// is enough.
const TTY_BUFFER_SIZE: usize = 1_204;

/// Creates a new pipe and returns `(read, write)` file descriptors.
fn pipe() -> Result<(FileDesc, FileDesc)> {
    let (read_fd, write_fd) = unsafe {
        let mut pipe_fds: [libc::c_int; 2] = [0; 2];
        if libc::pipe(pipe_fds.as_mut_ptr()) == -1 {
            return Err(io::Error::last_os_error().into());
        }
        (pipe_fds[0], pipe_fds[1])
    };

    let read_fd = FileDesc::new(read_fd, true);
    let write_fd = FileDesc::new(write_fd, true);

    Ok((read_fd, write_fd))
}

pub(crate) struct UnixInternalEventSource {
    poll: Poll,
    events: Events,
    parser: Parser,
    tty_buffer: [u8; TTY_BUFFER_SIZE],
    tty_fd: FileDesc,
    signals: Signals,
    wake_read_fd: FileDesc,
    wake_write_fd: FileDesc,
}

impl UnixInternalEventSource {
    pub fn new() -> Result<Self> {
        Ok(UnixInternalEventSource::from_file_descriptor(tty_fd()?)?)
    }

    pub(crate) fn from_file_descriptor(input_fd: FileDesc) -> Result<Self> {
        let poll = Poll::new()?;

        // PollOpt::level vs PollOpt::edge mio documentation:
        //
        // > With edge-triggered events, operations must be performed on the Evented type until
        // > WouldBlock is returned.
        //
        // TL;DR - DO NOT use PollOpt::edge.
        //
        // Because of the `try_read` nature (loop with returns) we can't use `PollOpt::edge`. All
        // `Evented` handles MUST be registered with the `PollOpt::level`.
        //
        // If you have to use `PollOpt::edge` and there's no way how to do it with the `PollOpt::level`,
        // be aware that the whole `TtyInternalEventSource` have to be rewritten
        // (read everything from each `Evented`, process without returns, store all InternalEvent events
        // into a buffer and then return first InternalEvent, etc.). Even these changes wont be
        // enough, because `Poll::poll` wont fire again until additional `Evented` event happens and
        // we can still have a buffer filled with InternalEvent events.
        let tty_raw_fd = input_fd.raw_fd();
        let tty_ev = EventedFd(&tty_raw_fd);
        poll.register(&tty_ev, TTY_TOKEN, Ready::readable(), PollOpt::level())?;

        let signals = Signals::new(&[signal_hook::SIGWINCH])?;
        poll.register(&signals, SIGNAL_TOKEN, Ready::readable(), PollOpt::level())?;

        let (wake_read_fd, wake_write_fd) = pipe()?;
        let wake_read_raw_fd = wake_read_fd.raw_fd();
        let wake_read_ev = EventedFd(&wake_read_raw_fd);
        poll.register(
            &wake_read_ev,
            WAKE_TOKEN,
            Ready::readable(),
            PollOpt::level(),
        )?;

        Ok(UnixInternalEventSource {
            poll,
            events: Events::with_capacity(3),
            parser: Parser::default(),
            tty_buffer: [0u8; TTY_BUFFER_SIZE],
            tty_fd: input_fd,
            signals,
            wake_read_fd,
            wake_write_fd,
        })
    }
}

impl EventSource for UnixInternalEventSource {
    fn try_read(&mut self, timeout: Option<Duration>) -> Result<Option<InternalEvent>> {
        if let Some(event) = self.parser.next() {
            return Ok(Some(event));
        }

        let timeout = PollTimeout::new(timeout);
        let mut additional_input_events = Events::with_capacity(3);

        loop {
            self.poll.poll(&mut self.events, timeout.leftover())?;

            if self.events.is_empty() {
                // No readiness events = timeout
                return Ok(None);
            }

            for token in self.events.iter().map(|x| x.token()) {
                match token {
                    TTY_TOKEN => {
                        let read_count = self.tty_fd.read(&mut self.tty_buffer, TTY_BUFFER_SIZE)?;

                        if read_count > 0 {
                            self.poll
                                .poll(&mut additional_input_events, Some(Duration::from_secs(0)))?;

                            let additional_input_available = additional_input_events
                                .iter()
                                .any(|event| event.token() == TTY_TOKEN);

                            self.parser.advance(
                                &self.tty_buffer[..read_count],
                                additional_input_available,
                            );

                            if let Some(event) = self.parser.next() {
                                return Ok(Some(event));
                            }
                        }
                    }
                    SIGNAL_TOKEN => {
                        for signal in &self.signals {
                            match signal as libc::c_int {
                                signal_hook::SIGWINCH => {
                                    // TODO Should we remove tput?
                                    //
                                    // This can take a really long time, because terminal::size can
                                    // launch new process (tput) and then it parses its output. It's
                                    // not a really long time from the absolute time point of view, but
                                    // it's a really long time from the mio, async-std/tokio executor, ...
                                    // point of view.
                                    let new_size = crate::terminal::size()?;
                                    return Ok(Some(InternalEvent::Event(Event::Resize(
                                        new_size.0, new_size.1,
                                    ))));
                                }
                                _ => unreachable!("Synchronize signal registration & handling"),
                            };
                        }
                    }
                    WAKE_TOKEN => {
                        // Something happened on the self pipe. Try to read single byte
                        // (see wake() fn) and ignore result. If we can't read the byte,
                        // mio Poll::poll will fire another event with WAKE_TOKEN.
                        let mut buf = [0u8; 1];
                        let _ = self.wake_read_fd.read(&mut buf, 1);
                        return Ok(None);
                    }
                    _ => unreachable!("Synchronize Evented handle registration & token handling"),
                }
            }

            // Processing above can take some time, check if timeout expired
            if timeout.elapsed() {
                return Ok(None);
            }
        }
    }

    fn wake(&self) {
        // DO NOT write more than 1 byte. See try_read & WAKE_TOKEN
        // handling - it reads just 1 byte. If you write more than
        // 1 byte, lets say N, then the try_read will be woken up
        // N times.
        let _ = self.wake_write_fd.write(&[0x57]);
    }
}

//
// Following `Parser` structure exists for two reasons:
//
//  * mimick anes Parser interface
//  * move the advancing, parsing, ... stuff out of the `try_read` method
//
struct Parser {
    buffer: Vec<u8>,
    internal_events: VecDeque<InternalEvent>,
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            // This buffer is used for -> 1 <- ANSI escape sequence. Are we
            // aware of any ANSI escape sequence that is bigger? Can we make
            // it smaller?
            //
            // Probably not worth spending more time on this as "there's a plan"
            // to use the anes crate parser.
            buffer: Vec::with_capacity(256),
            // TTY_BUFFER_SIZE is 1_024 bytes. How many ANSI escape sequences can
            // fit? What is an average sequence length? Let's guess here
            // and say that the average ANSI escape sequence length is 8 bytes. Thus
            // the buffer size should be 1024/8=128 to avoid additional allocations
            // when processing large amounts of data.
            //
            // There's no need to make it bigger, because when you look at the `try_read`
            // method implementation, all events are consumed before the next TTY_BUFFER
            // is processed -> events pushed.
            internal_events: VecDeque::with_capacity(128),
        }
    }
}

impl Parser {
    fn advance(&mut self, buffer: &[u8], more: bool) {
        for (idx, byte) in buffer.iter().enumerate() {
            let more = idx + 1 < buffer.len() || more;

            self.buffer.push(*byte);

            match parse_event(&self.buffer, more) {
                Ok(Some(ie)) => {
                    self.internal_events.push_back(ie);
                    self.buffer.clear();
                }
                Ok(None) => {
                    // Event can't be parsed, because we don't have enough bytes for
                    // the current sequence. Keep the buffer and process next bytes.
                }
                Err(_) => {
                    // Event can't be parsed (not enough parameters, parameter is not a number, ...).
                    // Clear the buffer and continue with another sequence.
                    self.buffer.clear();
                }
            }
        }
    }
}

impl Iterator for Parser {
    type Item = InternalEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.internal_events.pop_front()
    }
}
