use std::ops::Not;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: PieceColor,
}

impl Piece {
    pub fn get_piece(name: char) -> Option<Piece> {
        let color = if name.is_ascii_uppercase() { PieceColor::White } else { PieceColor::Black };
        PieceType::from_char(name).map(|pt| Piece{piece_type: pt, color})
    }

    pub fn to_char(&self) -> char {
        use PieceColor::*;

        let result = self.piece_type.to_char();

        match &self.color {
            Black => {
                result
            },
            White => {
                result.to_ascii_uppercase()
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {
    Pawn = 1,
    Knight = 2,
    Bishup = 3,
    Rook = 4,
    Queen = 5,
    King = 6,
}

impl PieceType {
    pub fn from_char(name: char) -> Option<PieceType> {
        match name.to_ascii_lowercase() {
            'k' => Some(Self::King),
            'q' => Some(Self::Queen),
            'b' => Some(Self::Bishup),
            'n' => Some(Self::Knight),
            'r' => Some(Self::Rook),
            'p' => Some(Self::Pawn),
            _ => None
        }
    }

    pub fn to_char(self) -> char {
        use PieceType::*;
        match self {
            King => 'k',
            Queen => 'q',
            Bishup => 'b',
            Knight => 'n',
            Rook => 'r',
            Pawn => 'p',
        }
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceColor {
    Black = 0,
    White = 1,
}

impl PieceColor {
    pub fn from_char(piece_char: char) -> Option<PieceColor> {
        match piece_char {
            'b' => Some(PieceColor::Black),
            'w' => Some(PieceColor::White),
            _ => None
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            PieceColor::Black => 'b',
            PieceColor::White => 'w',
        }
    } 
}

impl fmt::Display for PieceColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl Not for PieceColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black,
        }
    }
}