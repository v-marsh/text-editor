use std::io::Write;

mod string_writer {

    use super::*;

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
                .map_err(|_| std::io::ErrorKind::InvalidData)?
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
    IOError(std::io::Error),
}

pub enum PieceID {
    ORIGINAL,
    ADDITION,
}


struct Piece {
    start: usize,
    stop: usize,
    content: PieceID
}


pub struct PieceTable {
    original: String,
    addition: String,
    pieces: Vec<Piece>,
}

impl PieceTable {
    /// Create a `PieceTable` from `s`.
    pub fn from_string(s:String) -> Self {
        let mut pieces = Vec::new();
        pieces.push(Piece { start: 0, stop: s.len(), content: PieceID::ORIGINAL });
        Self { original: s, addition: String::new(), pieces }
    }

    /// Create a `PieceTable` from `s`.
    pub fn from_str(s: &str) -> Self {
        let s = String::from(s);
        let mut pieces = Vec::new();
        pieces.push(Piece { start: 0, stop: s.len(), content: PieceID::ORIGINAL });
        Self { original: s, addition: String::new(), pieces }
    }


    /// Insert `content` at character number `loc`.
    ///
    /// # Examples
    /// ```
    /// use text_editor::piece_table::PieceTable;
    /// let piece_table = PieceTable::from_str("hello world");
    /// piece_table.write_to_loc(5, "123");
    /// let new_string = piece_table.write_contents_to_string();
    /// assert_eq!(&new_string, "hello123 world");
    /// ```
    pub fn write_to_loc(&mut self, loc: usize, content: &str) {
        // TODO: IMPLEMENT HERE
    }

    /// Write contents of `self` to `stream` in correct order 
    ///
    /// # Errors
    /// Each call to `write_contents_to_stream` may generate the following 
    /// errors:
    /// * `GotBadPieceID` if a piece trys to reference a non-existant 
    /// piece number.
    /// * `IOError` wrapping any errors from calling `write` on `stream`.
    pub fn write_contents_to_stream<T: Write>(&self, stream: &mut T) -> 
        Result<usize, PieceTableError> {
        let mut bytes = 0;

        for piece in &self.pieces {
            let buf = match &piece.content {
                PieceID::ORIGINAL => &self.original,
                PieceID::ADDITION => &self.addition,
            };
            let buf = buf
                .get(piece.start..piece.stop)
                .ok_or(PieceTableError::GotBadPieceRange)?;
            bytes += stream.write(buf.as_bytes())
                .or_else(|err| Err(PieceTableError::IOError(err)))?;
        }

        Ok(bytes)
    }

    /// Write contents of `self` to `String` in correct order.
    ///
    /// # Examples
    /// ```
    /// use text_editor::piece_table::PieceTable;
    /// let piece_table = PieceTable::from_str("hello world!");
    /// piece_table.write_to_loc(5, "123")
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
