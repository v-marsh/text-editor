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
    pub fn new(buf: &str) -> Self {
        let mut pieces = Vec::new();
        pieces.push(Piece { start: 0, len: buf.len(), buffer: BufferType::READ });
        let read_str = String::from(buf);
        Self { read: read_str, append: String::new(), pieces, previous_write: None }
    }

    /// Insert `content` at `position`.
    ///
    /// This will insert content at the index given by position and
    /// return the number of characters written. If position is greater
    /// than the length of the piece table then the content will be
    /// appended to the end of the piece table.
    pub fn insert(&mut self, pos: usize, content: &str) {

        // Determine the index of the piece to edit and the location in
        // in the piece.
        let idx: usize;
        let piece_pos: usize;
        let mut idx_opt: Option<usize> = None;
        let mut piece_pos_opt: Option<usize> = None;
        if pos == 0 {
            idx_opt = Some(0);
            piece_pos_opt = Some(0);
        }
        let mut counter: usize = 0;
        for (i, piece) in self.pieces.iter().enumerate() {
            if pos < counter + piece.len {
                idx_opt = Some(i);
                piece_pos_opt = Some(pos - counter);
                break;
            }
            counter += piece.len;
        }
            // Will never panic since 'idx_opt'' always contains 'Some'
            // by the time expect is called.
            idx = idx_opt.or(Some(counter)).expect("");
            piece_pos = piece_pos_opt.or(Some(0)).expect("");

        // Determine if the edit occurs at the edge of a piece.
        let is_edge: bool;
        if piece_pos == 0 && idx != 0 {
            is_edge = true;
        } else {
            is_edge = false;
        }

        // Extend piece if insert location points to the edge of a
        // piece which points to the end of the append buffer.
        if idx != 0 && is_edge {
            let prev_piece = &mut self.pieces[idx - 1];
            if prev_piece.start + prev_piece.len == self.append.len() {
                prev_piece.len += content.len();
                self.append.push_str(content);
                return;
            }
        }

        // Create new piece with 'content' if 'position' is at an edge.
        if is_edge {
            let piece = Piece {
                start: self.append.len(),
                len: content.len(),
                buffer: BufferType::APPEND
            };
            self.pieces.insert(idx, piece);
            self.append.push_str(content);
            return;
        }

        // Split piece and insert new piece if 'position' in not at the
        // edge of a piece.
        // Only fails if idx is out of range, this never occurs.
        let _ = self.split_piece(idx, piece_pos);
        let piece = Piece {
            start: self.append.len(),
			len: content.len(),
			buffer: BufferType::APPEND
        };
        self.pieces.insert(idx, piece);
        self.append.push_str(content);
    }

    /// Delete slices from `PieceTable`.
    ///
    /// This will delete the slice in the range [`start`,`end`). If
    /// `start` >= `end` then nothing is deleted. If `end` > buffer current
    /// buffer length then will delete in range [`start`, end of buffer]
    pub fn delete(&mut self, start: usize, end: usize) {
        // Determine the piece to split and location in the piece
        todo!();

    }

    /// Join pieces to from read and append buffers.
    pub fn display_result(&self) -> String {
        let mut result = String::new();
        for piece in self.pieces.iter() {
            match piece.buffer {
                BufferType::READ => result.push_str(&self.read.as_str()[piece.start..piece.start + piece.len]),
                BufferType::APPEND => result.push_str(&self.append.as_str()[piece.start..piece.start + piece.len]),
            }
        }
        return result
    }

    fn split_piece(&mut self, piece_id: usize, piece_pos: usize) -> Result<(),()> {
        let mut left = self.pieces.get_mut(piece_id).ok_or(())?;
        let right = Piece{
            start: left.start + piece_pos,
			len: left.len - piece_pos,
			buffer: left.buffer.clone()
        };
        left.len = piece_pos;
        self.pieces.insert(piece_id + 1, right);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut piecetable = PieceTable::new("hello world");
        piecetable.insert(6, "funny ");
        let output = piecetable.display_result();
        let expected = String::from("hello funny world");
        assert_eq!(expected, output);
    }

}
