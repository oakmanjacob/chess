use super::{piece::PieceType, position::Position};
use std::fmt;
use regex::*;
use lazy_static::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChessMove {
    CastleKingside,
    CastleQueenside,
    Move(Position, Position),
    PawnPromote(Position, Position, PieceType),
}

impl ChessMove {
    #[allow(dead_code)]
    pub fn from_str(move_str: &str) -> Option<ChessMove> {
        lazy_static! {
            static ref MOVE_REGEX: Regex = Regex::new(r"(?P<from>[a-h][1-8])(?P<to>[a-h][1-8])").unwrap();
            static ref PROMOTE_REGEX: Regex = Regex::new(r"(?P<from>[a-h][1-8])(?P<to>[a-h][1-8])(?P<piece_type>[qrbn])").unwrap();
        }

        match move_str {
            "O-O" => Some(ChessMove::CastleKingside),
            "O-O-O" => Some(ChessMove::CastleQueenside),
            _ => {
                if let Some(captures) = PROMOTE_REGEX.captures(move_str) {
                    if let (Ok(from), Ok(to), Some(piece_type)) = (Position::from_str(&captures["from"]), Position::from_str(&captures["to"]), PieceType::from_str(&captures["piece_type"])) {
                        Some(ChessMove::PawnPromote(from, to, piece_type))
                    }
                    else {
                        None
                    }
                }
                else if let Some(captures) = MOVE_REGEX.captures(move_str) {
                    if let (Ok(from), Ok(to)) = (Position::from_str(&captures["from"]), Position::from_str(&captures["to"])) {
                        Some(ChessMove::Move(from, to))
                    }
                    else {
                        None
                    }
                }
                else {
                    None
                }
            }
        }
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