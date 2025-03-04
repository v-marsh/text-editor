//#[derive(Debug)]
//pub enum PieceTableError {
//    GotBadPieceID,
//    GotBadPieceRange,
//    GotBadLoc,
//    IOError(std::io::Error),
//}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BufferType {
    READ,
    APPEND,
}

#[derive(Debug)]
struct Piece {
    pub start: usize,
    pub len: usize,
    pub buffer: BufferType,
}

struct WriteLocation {
    position: usize,
    piece_id: usize,
}
pub struct PieceTable {
    /// An read-only buffer containing the original contents.
    read: String,
    /// A append-only buffer containing updates to the contents of the
    /// original buffer.
    append: String,
    /// A ordered collection of pieces used to specify how the new
    /// buffer in constructed from the `read` and `append`
    /// buffers.
    pieces: Vec<Piece>,
    /// Used to denote the location of the previous `insert` command,
    /// for increased speed of successive `insert` commands. Is set to
    /// `None` if an operation has changed the buffer such that the
    /// location must be re-computed. Note it is assumed that this only
    /// refers to a valid piece which points to the `addition` buffer.
    previous_write: Option<WriteLocation>,
}

impl PieceTable {
    /// Create a `PieceTable` from `s`.
    pub fn new(buf: String) -> Self {
        let mut pieces = Vec::new();
        pieces.push(Piece { start: 0, len: buf.len(), buffer: BufferType::READ });
        Self { read: buf, append: String::new(), pieces, previous_write: None }
    }

    /// Insert `content` at `position`.
    ///
    /// This will insert content at the index given by position and
    /// return the number of characters written. If position is greater
    /// than the length of the piece table then the content will be
    /// appended to the end of the piece table.
    pub fn insert(&mut self, position: usize, content: &str) {

        // Determine the index of the piece to edit
        let idx: usize;
        let idx_opt: Option<usize> = None;
        if position == 0 {
            idx_opt = Some(0);
        } else {
            let mut counter = 0;
            for (i, piece) in self.pieces.iter().enumerate() {
                if position < counter + piece.len {
                    idx_opt = Some(i);
                    break;
                }
                counter += piece.len;
            }
            idx = idx_opt.or(Some(counter)).expect("");
        }

        // Determine if the edit occurs at the edge
        let is_edge: bool;
        let is_edge_opt: Option<bool> = None;
        if position == 0 {
            is_edge_opt = Some(true);
        } else {
            let mut counter = 0;
            for (i, piece) in self.pieces.iter().enumerate() {
                if position == counter + piece.len {
                    is_edge_opt = Some(true);
                    break;
                }
                counter += piece.len;
            }
            is_edge = is_edge_opt.or(Some(false)).expect("");
        }

        if idx != 0 && is_edge {

        }

        // Append to piece from last insert if a valid write_location
        // exists.
        if let Some(write_location) = &self.previous_write {
            if position == write_location.position {
                self.append.push_str(content);
                self.pieces.get_mut(write_location.piece_id).expect("").len += content.len();
            }
        }

        // General case: write to any position in the table
        let mut position = position;
        let mut piece: Option<&Piece> = None;
        let mut piece_id: Option<usize> = None;
        let mut piece_start_loc: Option<usize> = None;
        let mut current_loc = 0;
        let mut next_loc = 0;

        // Find the piece containing 'position' within its range
        for (id, _piece) in self.pieces.iter().enumerate() {
            next_loc = next_loc + _piece.len
            if next_loc >= position {
                // This can always be safely unwrapped since 'id' is
                // bounded by the length of self.pieces.
                piece = Some(&self.pieces.get(id).unwrap());
                piece_id = Some(id);
                piece_start_loc = Some(current_loc);
                break;
            }
            current_loc = next_loc;
        }

        let piece = piece.unwrap_or(&self.pieces.last().unwrap());
        let piece_id = piece_id.unwrap_or(self.pieces.len() - 1);
        let piece_start_loc = piece_start_loc.unwrap_or(
            current_loc+Position(piece.len())
        );
        // Insert content at the end of the piece table if the position
        // is larger than any valid value.
        if position > next_loc { position = next_loc };

        // Determine location to insert the new piece
        let new_piece_id: usize;
        if position == piece_start_loc {
            new_piece_id = piece_id;
        } else if position == piece_start_loc + piece.stop - piece.start {
            new_piece_id = piece_id + 1;
        } else {
            let piece_loc = position - piece_start_loc;
            // Can unwrap here since function only fails if piece_id is
            // out of range.
            self.split_piece(piece_id, piece_loc).unwrap();
            new_piece_id = piece_id + 1;
        }

        // Insert the new piece
        let start = Position(self.append.len());
        self.append.push_str(content);
        let stop = Position(self.append.len());
        let n_chars = stop.0 - start.0;
        let new_piece = Piece { start, stop, buffer: BufferType::APPEND };
        self.pieces.insert(new_piece_id, new_piece);

        // Update write location for faster insert
        self.previous_write = Some(WriteLocation{
            position,
            piece_id: new_piece_id
        });
    }

    /// Delete slices from `PieceTable`.
    ///
    /// This will delete the slice in the range [`start`,`end`). If
    /// `start` >= `end` then nothing is deleted. If `end` > buffer current
    /// buffer length then will delete in range [`start`, end of buffer]
    pub fn delete(&mut self, start: Position, end: Position) {
        // Determine the piece to split and location in the piece
        self.previous_write = None

    }

    /// Join pieces to from read and append buffers.
    pub fn display_result(&self) -> String {
        let mut result = String::new();
        for piece in self.pieces.iter() {
            match piece.buffer {
                BufferType::READ => result.push_str(&self.read.as_str()[piece.start.0..piece.stop.0]),
                BufferType::APPEND => result.push_str(&self.append.as_str()[piece.start.0..piece.stop.0]),
            }
        }
        return result
    }

    fn split_piece(&mut self, piece_id: usize, piece_position: Position) -> Result<(),()> {
        let mut left = self.pieces.get_mut(piece_id).ok_or(())?;
        let right = Piece{
            start: left.start + piece_position,
			stop: left.stop,
			buffer: BufferType::APPEND
        };
        left.stop = right.start;
        self.pieces.insert(piece_id + 1, right);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testing(){}

}
