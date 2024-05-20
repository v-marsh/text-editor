use text_editor::editor::*;
use text_editor::terminal::{self, input_stream_editor};

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
