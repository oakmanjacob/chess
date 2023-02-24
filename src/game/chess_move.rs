use super::{piece::PieceType, position::Position};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChessMove {
    CastleKingside,
    CastleQueenside,
    Move(Position, Position),
    PawnPromote(Position, Position, PieceType),
}

impl ChessMove {
    pub fn from_str(from_str: &str, to_str: &str) -> ChessMove {
        let from = Position::from_str(from_str).expect("Invalid From string in chessmove generation");
        let to = Position::from_str(to_str).expect("Invalid to string in chessmove generation");

        ChessMove::Move(from, to)
    }
}

impl fmt::Display for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChessMove::CastleKingside => write!(f, "O-O"),
            ChessMove::CastleQueenside => write!(f, "O-O-O"),
            ChessMove::Move(from, to) => write!(f, "{}{}", from, to),
            ChessMove::PawnPromote(from, to, piece_type) => write!(f, "{}{}{}", from, to, piece_type),
        }
    }
}