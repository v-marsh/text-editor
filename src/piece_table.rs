use std::io::Write;

mod string_writer {

    use std::io::{ Write, ErrorKind };

    pub struct StringWriter {
        pub contents: String,
    }

    impl StringWriter {
        pub fn new() -> Self {
            Self { contents: String::new() }
        }
    } 

    impl Write for StringWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let len = buf.len();
            self.contents.push_str(&String::from_utf8(buf.to_vec())
                .map_err(|_| ErrorKind::InvalidData)?
            );
            Ok(len)
        }
        
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum PieceTableError {
    GotBadPieceID,
    GotBadPieceRange,
    GotBadLoc,
    IOError(std::io::Error),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PieceBuf {
    ORIGINAL,
    ADDITION,
}

#[derive(Debug)]
pub struct Piece {
    pub start: usize,
    pub stop: usize,
    pub content: PieceBuf
}

impl Piece {
    pub fn len(&self) -> usize {
        self.stop - self.start
    }
}

impl PartialEq for Piece {
    fn eq(&self, rhs: &Self) -> bool {
        if self.start == rhs.start && self.stop == rhs.stop && self.content == rhs.content {
            true
        } else {
            false
        }
    }
}

impl Eq for Piece {}

pub struct PieceTable {
    original: String,
    addition: String,
    pieces: Vec<Piece>,
    current_piece_id: usize,
}

impl PieceTable {
    /// Create a `PieceTable` from `s`.
    pub fn from_string(s:String) -> Self {
        let mut pieces = Vec::new();
        pieces.push(Piece { start: 0, stop: s.len(), content: PieceBuf::ORIGINAL });
        Self { original: s, addition: String::new(), pieces, current_piece_id: 0 }
    }

    /// Create a `PieceTable` from `s`.
    pub fn from_str(s: &str) -> Self {
        let s = String::from(s);
        Self::from_string(s)
    }

    pub fn get_pieces(&self) -> &Vec<Piece> {
        &self.pieces
    }

    /// Insert `content` at `loc` in buffer.
    ///
    /// # Examples
    /// ```
    /// use text_editor::piece_table::PieceTable;
    /// let mut piece_table = PieceTable::from_str("hello world");
    /// piece_table.write_to_loc(5, "123").unwrap();
    /// let new_string = piece_table.write_contents_to_string();
    /// assert_eq!(&new_string, "hello123 world");
    /// ```
    pub fn write_to_loc(&mut self, loc: usize, content: &str) ->
        Result<(), PieceTableError> {
        // Need to find a neat solution for finding the starting point
        // for loc then the rest is simple
        // find respective piece, split (most likely), and insert
        // This is most easily solved by iterating through the pieces
        // in self.pieces and summing the lengths ()

        let mut piece: Option<&Piece> = None;
        let mut piece_id: Option<usize> = None;
        let mut piece_start_loc: Option<usize> = None;
        let mut current_loc = 0;
        let mut next_loc = 0;

        for (id, _piece) in self.pieces.iter().enumerate() {
            next_loc += _piece.len();
            if next_loc >= loc {
                // This can always be safely unwrapped since id is 
                // bounded by the lenth of self.pieces
                piece = Some(&self.pieces.get(id).unwrap());
                piece_id = Some(id);
                piece_start_loc = Some(current_loc);
                break;
            }
            current_loc = next_loc;
        }
        
        let piece = piece.ok_or(PieceTableError::GotBadLoc)?;
        let piece_id = piece_id.ok_or(PieceTableError::GotBadLoc)?;
        let piece_start_loc = piece_start_loc.ok_or(PieceTableError::GotBadLoc)?;

        // If loc is in the middle of a piece then split piece before
        // inputting.
        if loc != piece_start_loc + piece.len() {
            let piece_loc = loc - piece_start_loc;
            self.split_piece(piece_id, piece_loc).map_err(|_| PieceTableError::GotBadLoc)?;  
        }       
        
        let start = self.addition.len();
        self.addition.push_str(content);
        let stop = self.addition.len();
        let new_piece = Piece { start, stop, content: PieceBuf::ADDITION };
        self.pieces.insert(piece_id + 1, new_piece);

        Ok(())
    }

    /// Append `content` to the last piece that was written to.
    ///
    /// # Examples
    /// ```
    /// use text_editor::piece_table::PieceTable;
    /// let mut piece_table = PieceTable::from_str("hello world");
    /// piece_table.write_to_loc(5, "123");
    /// piece_table.write_to_current_piece("new");
    /// piece_table.write_to_loc(1, "22");
    /// piece_table.write_to_current_piece("test");
    /// let contents = piece_table.write_contents_to_string();
    /// assert_eq!(contents, "h22testello123new world");
    /// ```
    pub fn write_to_current_piece(&mut self, content: &str) {
        todo!();
    }

    /// Split a piece at `loc`, the distance from the start of the 
    /// piece.
    ///
    /// # Errors
    /// Each call to `split_piece` may generate the following errors:
    /// * `GotBadPieceID` if `piece_id` does not exists.
    /// * `GotBadPieceRange` if `loc` is outside of the range defined
    /// in the piece given by `piece_id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use text_editor::piece_table::*;
    /// let mut piece_table = PieceTable::from_str("hello world!");
    /// piece_table.split_piece(0, 5);
    /// piece_table.split_piece(1, 2);
    /// let pieces = piece_table.get_pieces();
    /// assert_eq!(
    ///     pieces.get(0).unwrap(),
    ///     &Piece { start: 0, stop: 5, content: PieceBuf::ORIGINAL }
    /// );
    /// assert_eq!(
    ///     pieces.get(1).unwrap(),
    ///     &Piece { start: 5, stop: 11, content: PieceBuf::ORIGINAL }
    /// );
    /// assert_eq!(
    ///     pieces.get(2).unwrap(), 
    ///     &Piece { start: 7, stop: 11, content: PieceBuf::ORIGINAL }
    /// )
    /// ```
    pub fn split_piece(&mut self, piece_id: usize, piece_loc: usize) -> 
        Result<(), PieceTableError> {
        let piece = self.pieces
            .get_mut(piece_id)
            .ok_or(PieceTableError::GotBadPieceID)?;

        let true_loc = piece_loc + piece.start;

        if true_loc <= piece.start || true_loc >= piece.stop {
            let new_piece_stop = piece.stop;
            piece.stop = true_loc;
            let new_piece = Piece { 
                start: true_loc, stop: new_piece_stop, content: piece.content.clone()
            };
            self.pieces.insert(piece_id + 1, new_piece);
        } else {
            return Err(PieceTableError::GotBadPieceRange);
        };

        Ok(())
    }

    /// Write contents of `self` to `stream` in correct order 
    ///
    /// # Errors
    /// Each call to `write_contents_to_stream` may generate the following 
    /// PieceTableError errors:
    /// * `GotBadPieceID` if a piece trys to reference a non-existant 
    /// piece number.
    /// * `IOError` wrapping any errors from calling `write` on `stream`.
    pub fn write_contents_to_stream<T: Write>(&self, stream: &mut T) -> 
        Result<usize, PieceTableError> {
        let mut n_bytes = 0;

        for piece in &self.pieces {
            let buf = match &piece.content {
                PieceBuf::ORIGINAL => &self.original,
                PieceBuf::ADDITION => &self.addition,
            };
            let contents = buf
                .get(piece.start..piece.stop)
                .ok_or(PieceTableError::GotBadPieceRange)?;
            n_bytes += stream.write(contents.as_bytes())
                .or_else(|err| Err(PieceTableError::IOError(err)))?;
        }

        Ok(n_bytes)
    }

    /// Write contents of `self` to `String` in correct order.
    ///
    /// # Examples
    ///
    /// ```
    /// use text_editor::piece_table::PieceTable;
    /// let mut piece_table = PieceTable::from_str("hello world!");
    /// piece_table.write_to_loc(5, "123");
    /// let contents = piece_table.write_contents_to_string();
    /// assert_eq!(contents, String::from("hello123 world"));
    /// ```
    pub fn write_contents_to_string(&self) -> String {
        let mut writer = string_writer::StringWriter::new();
        // NOTE: Need to hangle unwrap in a more suitable fasion
        self.write_contents_to_stream(&mut writer).unwrap();
        writer.contents
    }
}    

#[cfg(test)]
mod tests {
    use super::*;
    use string_writer::StringWriter;
    

    #[test]
    /// Check that `PieceTable::write_contents_to_stream` writes the 
    /// contents of `self.original` correctly to stream.
    fn write_piece_table_original_correctly() {
        let string = String::from("hello world!");
        let mut stream = StringWriter::new();
        
        let piece_table = PieceTable::from_string(string.clone());

        piece_table.write_contents_to_stream(&mut stream).unwrap();

        assert!(string == stream.contents);
    }

    #[test]
    /// Check that `PieceTable::write_contents_to_steam` writes the 
    /// contents of `self.original` and `self.add` correctly to steam.
    fn write_piece_table_additions_correctly() {
    }
}



