//#![deny(unused_imports, unused_must_use)]
//! # Input
//!
//! This documentation does not contain a lot of examples. The reason is that it's fairly
//! obvious how to use this crate. Although, we do provide
//! [examples](https://github.com/crossterm-rs/examples) repository
//! to demonstrate the capabilities.
//!
//! ## Synchronous vs Asynchronous
//!
//! ### Synchronous Reading
//!
//! Read the input synchronously from the user, the reads performed will be blocking calls.
//! Using synchronous over asynchronous reading has the benefit that it is using fewer resources than
//! the asynchronous because background thread and queues are left away.
//!
//! See the [`SyncReader`](struct.SyncReader.html) documentation for more details.
//!
//! ### Asynchronous Reading
//!
//! Read the input asynchronously, input events are gathered in the background and queued for you to read.
//! Using asynchronous reading has the benefit that input events are queued until you read them. You can poll
//! for occurred events, and the reads won't block your program.
//!
//! See the [`AsyncReader`](struct.AsyncReader.html) documentation for more details.
//!
//! ### Technical details
//!
//! On UNIX systems crossterm reads from the TTY, on Windows, it uses `ReadConsoleInputW`.
//! For asynchronous reading, a background thread will be fired up to read input events,
//! occurred events will be queued on an MPSC-channel, and the user can iterate over those events.
//!
//! The terminal has to be in the raw mode, raw mode prevents the input of the user to be displayed
//! on the terminal screen.

pub use self::{
    event_pool::{poll, read},
    event_source::{fake::FakeEventSource, EventSource},
    events::{Event, KeyEvent, MouseButton, MouseEvent},
};

mod ansi;
mod event_poll;
mod event_source;
mod sys;

pub(crate) mod event_pool;
pub(crate) mod events;
pub(crate) mod poll_timeout;
