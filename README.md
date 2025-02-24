# Text Editor

# Requirements
A basic text editor that should render text in a linux terminal that meets the following requirements:
* contain a single cursor that can navigate the text using the arrow keys.
* contain functionality for a simple search of the text
    * this should allow the user to enter a phrase and find all maching phrases in the text
* contain basic functionality that allows the user to save or delete changes
* contain basic functionality that allows the user to undo and redo changes

# Current todo:
* [ ] Write testing to ensure that stdin is being correctly changed by 'activate_stdin_raw_mode' since the file descriptor is opened in the function and then dropped upon return. This would imply that stdin will NOT be altered after the function returns. See [here](src/terminal.rs).
* [ ] Editor currently fails to display the exit status when 'kill_editor' is called.
* [ ] Create cursor object to determine location of insertions/deletion
    * [  ] find solution for situation when cursor is a and edge and asked to move off the page
* allow appending to the end of a text buffer and have text load to
screen
    * [ ] this requires completing the comment on line 57 in src/editor.rs
    [here](src/editor.rs)
