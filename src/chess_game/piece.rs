use std::fmt;
use std::ops::Not;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub piece_color: PieceColor,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceType {
    King,
    Queen,
    Bishup,
    Knight,
    Rook,
    Pawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceColor {
    Black,
    White
}

impl fmt::Display for PieceColor {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Black => write!(f, "Black"),
            Self::White => write!(f, "White"),
        }
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

impl PieceType {
    pub fn get(name: char) -> Option<PieceType> {
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
}

impl Piece {
    pub fn get_piece(name: char) -> Option<Piece> {
        let color = if name.is_ascii_uppercase() { PieceColor::White } else { PieceColor::Black };
        match PieceType::get(name) {
            Some(pt) => Some(Piece{piece_type: pt, piece_color: color}),
            None => None
        }
    }

    pub fn get_name(&self) -> char {
        use PieceType::*;
        use PieceColor::*;

        let result = match &self.piece_type {
            King => 'k',
            Queen => 'q',
            Bishup => 'b',
            Knight => 'n',
            Rook => 'r',
            Pawn => 'p',
        };

        match &self.piece_color {
            Black => {
                result
            },
            White => {
                result.to_ascii_uppercase()
            }
        }
    }
}