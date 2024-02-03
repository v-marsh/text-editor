mod input_stream_editor {
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


mod terminal {
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


mod editor {
    use termios::Termios;
    use std::io::{ self, Read, Write };
    use crate::terminal;

    pub enum EditorStatus{
        RefershScreen,
        TerminalExitSuccess,
        FailedToBuild(EditorBuildError),
        FailedToRefresh(io::Error),
        FailedToProcessKeypress(io::Error),
    }
    

    pub enum EditorBuildError {
        UnableToGetWindowSize,
    }
    

    pub struct Editor {
        pub status: EditorStatus,
        pub screen_rows: usize,
        pub screen_colums: usize,
    }

    
    impl Editor {
        pub fn build() -> Result<Self, EditorBuildError> {
            if let Some(size) = terminal::get_terminal_size() {
                Ok(Self { 
                    status: EditorStatus::RefershScreen,
                    screen_rows: size.rows,
                    screen_colums: size.cols,
                })
            } else {
                Err(EditorBuildError::UnableToGetWindowSize)
            }
        }
    }

    /// Processes the next keypress to stdin and updates `editor` as 
    /// required.
    ///
    /// # Errors
    /// * Returns an error if unable to read one u8 character from stdin
    pub fn editor_process_keypress(editor: &mut Editor) -> io::Result<()> {
        let mut buffer = [0;1];

        io::stdin().read_exact(&mut buffer)?;

        match char::from(buffer[0]) {
            'q' => editor.status = EditorStatus::TerminalExitSuccess,
            _ => (),
        }

        Ok(())
    }


    fn editor_draw_rows(n_rows: usize) {
        for _ in 0..n_rows {
            print!("~\r\n");
        }
    }


    /// Draws the next frame by clearing the screen and redrawing the
    /// contents of `editor`.
    ///
    /// # Errors
    /// * Returns an error if the stdin.flush fails to write all bytes
    /// to screen.
    pub fn editor_refresh_screen(editor: & Editor) -> io::Result<()> {
        // Clear contents of terminal
        print!("\x1b[2J");

        // Move cursor to top 
        print!("\x1b[H");

        // Draw rows of tiles (like vim)
        editor_draw_rows(editor.screen_rows);

        // Move cursor to top
        print!("\x1b[H");

        io::stdout().flush()
    }

    /// Returns the terminal to the state defined by `original_termios`,
    /// clears the terminal and kills the program.
    pub fn kill_editor(original_termios: Termios, status: EditorStatus) -> ! {
        crate::input_stream_editor::recover_original_stdin_mode(original_termios);

        print!("\x1b[2J");
        print!("\x1b[H");

        match io::stdout().flush() {
            Ok(()) => {
                std::process::exit(0);
            },

            Err(err) => {
                panic!("{}", err);
            }
        }
    }
}

use editor::*;

fn main() {
    // Set up terminal and editor 
    let original_termios = input_stream_editor::activate_stdin_raw_mode();
    let mut editor = match Editor::build() {
        Ok(editor) => editor,
        Err(e) => kill_editor(original_termios, EditorStatus::FailedToBuild(e)),
    };

    loop {
        if let Err(e) = editor_refresh_screen(&editor) {
            editor.status = EditorStatus::FailedToRefresh(e);
        }

        if let Err(e) = editor_process_keypress(&mut editor) {
            editor.status = EditorStatus::FailedToProcessKeypress(e);
        }

        if let EditorStatus::RefershScreen = editor.status {
            continue;
        } else {
            kill_editor(original_termios, editor.status);
        };
    }
}
