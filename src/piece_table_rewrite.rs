use std::ops::{self, Add, Sub}

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
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Position(usize);

impl Position {
    pub fn change_position(&mut self, distance: usize) {
        self.0 += distance;
        if self.0 < 0 {
            self.0 = 0;
        }
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Position(self.0 + rhs.0)
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Position(self.0 - rhs.0)
    }
}

#[derive(Debug)]
struct Piece {
    pub start: Position,
    pub stop: Position,
    pub content: PieceBuf,
}

struct WriteLocation {
    position: Position,
    piece_id: usize,
}
pub struct PieceTable {
    /// An immutable buffer containing the contents of the original
    /// string.
    original: String,
    /// A write-only buffer containing updates to the contents of the
    /// original buffer.
    addition: String,
    /// A ordered collection of pieces used to specify how the new
    /// buffer in constructed from the `original` and `addition`
    /// buffers.
    pieces: Vec<Piece>,
    /// Used to denote the location of the previous `insert` command,
    /// for increased speed of successive `insert` commands. Is set to
    /// `None` if an operation has changed the buffer such that the
    /// location must be re-computed. Note it is assumed that this only
    /// refers to a piece which points to the `addition` buffer.
    previous_write: Option<WriteLocation>,
}

impl PieceTable {
    /// Create a `PieceTable` from `s`.
    pub fn new(s:String) -> Self {
        let mut pieces = Vec::new();
        pieces.push(Piece { start: Position(0), stop: Position(s.len()), content: PieceBuf::ORIGINAL });
        Self { original: s, addition: String::new(), pieces, previous_write: None }
    }
    /// Insert `content` at `position`.
    ///
    /// This will insert content at the index given by position and
    /// return the number of characters written. If position < 0 then
    /// the content will be inserted at index 0 and if position is
    /// greater than the length of the piece table then the content
    /// will be appended to the end of the piece table.
    pub fn insert(&mut self, position: Position, content: &str) ->
        Result<usize, PieceTableError> {

        // Continue writing from previous position
        if let Some(write_location) = &self.previous_write {
            if position == write_location.position {
                self.addition.push_str(content);
                self.pieces
                    .get_mut(write_location.piece_id)
                    // unwrap here since a valid write_location is
                    // assumed for speed.
                    .unwrap()
                    .stop.change_position(content.len());
            }
        }

        // STILL NEED TO SOLVE FOR GENERAL CASE CURRENT CODE IS OUT OF DATE
        let mut piece: Option<&Piece> = None;
        let mut piece_id: Option<usize> = None;
        let mut piece_start_loc: Option<usize> = None;
        let mut current_loc = 0;
        let mut next_loc = 0;

        for (id, _piece) in self.pieces.iter().enumerate() {
            next_loc += _piece.stop - _piece.start;
            if next_loc >= position {
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
        if position != piece_start_loc + piece.len() {
            let piece_loc = position - piece_start_loc;
            self.split_piece(piece_id, piece_loc)
                .map_err(|_| PieceTableError::GotBadLoc)?;
        }

        let start = self.addition.len();
        self.addition.push_str(content);
        let stop = self.addition.len();
        let n_chars = stop - start;

        let new_piece = Piece { start, stop, content: PieceBuf::ADDITION };
        let new_piece_id = piece_id + 1;
        self.pieces.insert(new_piece_id, new_piece);
        self.current_piece_id = new_piece_id;


        Ok(n_chars)
    }

    /// Delete slices from `PieceTable`.
    ///
    /// This will delete the slice in the range [`start`,`end`). If
    /// `start` >= `end` then nothing is deleted. If `start` < 0 then
    /// will delete in the range [0, `end`). If `end` > buffer current
    /// buffer length then will delete in range [`start`, end of buffer]
    pub fn delete(&mut self, start: Position, end: Position) {
        // Determine the piece to split and location in the piece
        self.previous_write = None

    }

    fn get_piece_from_position(&self, position: Position) -> (Piece, usize)
}
