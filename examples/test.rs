use mio::Interest;
use signal_hook::iterator::Signals;
use std::path::Path;
use std::io;

fn main () {
    let poll = Poll::new()?;
    let registry = poll.registry();

    let tty_raw_fd = input_fd.raw_fd();
    let mut tty_ev = SourceFd(&tty_raw_fd);
    registry.register(&mut tty_ev, TTY_TOKEN, Interest::READABLE)?;

    let mut signals = Signals::new(&[signal_hook::SIGWINCH])?;
    registry.register(&mut signals, SIGNAL_TOKEN, Interest::READABLE)?;

}


fn open_rw<P: AsRef<Path>>(path: P) -> io::Result<RawFd> {
    use std::fs::OpenOptions;

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?;

    Ok(file.into_raw_fd())
}