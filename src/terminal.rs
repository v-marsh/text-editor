pub mod input_stream_editor {
    use std::io;
    use std::os::fd::AsRawFd;
    use termios::*;


    /// Convert stdin from canonical to raw mode
    pub fn activate_stdin_raw_mode() -> Termios {
        let stdin_raw_fd = io::stdin().as_raw_fd();

        // Can safely unwrap here since `Termios::from_fd` will only 
        // fail if `raw_fd` is not an open file descriptor and stdin is
        // always open.
        let termios_original = Termios::from_fd(stdin_raw_fd).unwrap();
        let mut termios_new = termios_original.clone();

        termios_new.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        termios_new.c_oflag &= !OPOST;
        termios_new.c_cflag |= CS8;
        termios_new.c_lflag &= !(ECHO | ICANON | ISIG | IEXTEN);

        // Can safely unwrap here since `tscetattr` will only fail if 
        // `raw_fd` is not an open file descriptor and stdin is always 
        // open.
        tcsetattr(stdin_raw_fd, TCSANOW, &termios_new).unwrap();
        termios_original
    }


    /// Reset stdin to mode defined by `original_termios`.
    pub fn recover_original_stdin_mode(original_termios: Termios) {
        let raw_fd = io::stdin().as_raw_fd();
        
        // Should be able to safely unwrap here since tcsetattr should
        // only return an error if it is unable to execute the update,
        // but stdin is always open
        tcsetattr(raw_fd, TCSANOW, &original_termios).unwrap();
    }
}


pub mod terminal_utils {
    use rustix::{termios::{tcgetwinsize, isatty}, fd::{RawFd, BorrowedFd, AsRawFd}};
    use std::io;


    pub struct WindowSize {
        pub cols: usize,
        pub rows: usize, 
    }


    /// Attempt to get size of terminal from a raw file descriptor.
    ///
    /// # Parameters
    /// * `fd` should be a raw file descriptor associated with the
    /// terminal.
    ///
    /// # Errors
    /// *  Returns `None` if unable to determine terminal size from `fd`.
    fn get_terminal_size_from_fd(fd: RawFd) -> Option<WindowSize> {
        let fd = unsafe { BorrowedFd::borrow_raw(fd) };

        if !isatty(fd) {
            return None;
        }

        let winsize = tcgetwinsize(fd).ok()?;

        let rows = winsize.ws_row as usize;
        let cols = winsize.ws_col as usize;

        if rows > 0 && cols > 0 {
            Some(WindowSize { cols, rows })
        } else {
            None
        }
    }


    /// Attempt to get size of terminal from stdout, stderr, and stdin
    /// in that order. Returns upon first success.
    ///
    /// # Errors
    /// *  Returns `None` if unable to determine terminal size from 
    /// stdout, stderr, or stdin.
    pub fn get_terminal_size() -> Option<WindowSize> {
        if let Some(size) = get_terminal_size_from_fd(io::stdout().as_raw_fd()) {
            Some(size)
        } else if let Some(size) = get_terminal_size_from_fd(io::stderr().as_raw_fd()) {
            Some(size)
        } else if let Some(size) = get_terminal_size_from_fd(io::stdin().as_raw_fd()) {
            Some(size)
        } else {
            None
        }
    }
}
