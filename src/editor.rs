use termios::Termios;
use std::io::{ self, Read, Write };
use crate::terminal::terminal_utils;
use crate::piece_table::PieceTable;

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

pub struct Cursor {
    pub row: usize,
    pub column: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self { row: 0, column: 0 }
    }
    
    pub fn move_to_top_left(&mut self) {
        self.row = 0;
        self.column = 0;
        print!("\x1b[H")
    }

    pub fn move_left(&mut self, n: usize) {
        
    }
}

pub struct Editor {
    pub status: EditorStatus,
    pub screen_rows: usize,
    pub screen_colums: usize,
    pub text_buffer: PieceTable,
    pub cursor: Cursor,
}


impl Editor {
    pub fn build() -> Result<Self, EditorBuildError> {
        if let Some(size) = terminal_utils::get_terminal_size() {
            Ok(Self { 
                status: EditorStatus::RefershScreen,
                screen_rows: size.rows,
                screen_colums: size.cols,
                text_buffer: PieceTable::new(),
                cursor: Cursor::new(),
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
        // This needs to be updated such that if buffer contains a
        // alphanumeric character then push it to the editor text_buffer
        // otherwise do nothing currently.
        'q' => editor.status = EditorStatus::TerminalExitSuccess,
        _ => (),
    }

    Ok(())
}


fn editor_draw_empty_rows(n_rows: usize) {
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

    // Draw line 1 (empty)
    print!(" 1  \r\n");

    // Draw rows of tiles (like vim) minus first row
    editor_draw_empty_rows(editor.screen_rows - 2);

    // Move cursor to top
    print!("\x1b[{}A", editor.screen_rows);

    // Move cursor left 2
    print!("\x1b[4C");

    io::stdout().flush()
}

/// Returns the terminal to the state defined by `original_termios`,
/// clears the terminal and kills the program.
pub fn kill_editor(original_termios: Termios, status: EditorStatus) -> ! {
    crate::terminal::input_stream_editor::recover_original_stdin_mode(original_termios);

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
