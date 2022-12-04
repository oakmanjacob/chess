use std::fmt;
use eyre::{eyre, Result};
use crate::chess_game::piece::*;
use crate::chess_game::board::*;

#[derive(Debug, Clone, Copy)]
pub enum ChessMove {
    CastleKingside,
    CastleQueenside,
    Move((usize, usize), (usize, usize)),
    PawnPromote((usize, usize), (usize, usize), PieceType),
}

impl fmt::Display for ChessMove {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CastleKingside => write!(f, "Castle Kingside"),
            Self::CastleQueenside => write!(f, "Castle Queenside"),
            Self::Move(from, to) => write!(f, "Move from {:?} to {:?}", from, to),
            Self::PawnPromote(from, to, p) => write!(f, "Move pawn from {:?} to {:?} and promote to {:?}", from, to, p)
        }
    }
}

// Need to change this to be cognisant of the board state for more advanced ideas
// This will require similar analysis to the "count threats" function to translate the moves
impl ChessMove {
    pub fn new(chess_move: &str) -> Result<ChessMove> {
        match chess_move {
            "O-O" => Ok(Self::CastleKingside),
            "O-O-O" => Ok(Self::CastleQueenside),
            _ => {
                let segments: Vec<&str> = chess_move.split("=").collect();
                if (segments.len() == 1 || segments.len() == 2) && segments[0].len() == 4 {
                    let (start_str, end_str) = segments[0].split_at(2);

                    match (Board::get_loc(start_str), Board::get_loc(end_str)) {
                        (Ok(start), Ok(end)) => {
                            if segments.len() == 2 {
                                let promotion: Vec<char> = segments[1].chars().collect();
                                if promotion.len() == 1 {
                                    match PieceType::get(promotion[0]) {
                                        Some(PieceType::King) => Err(eyre!("Cannot Promote to King")),
                                        Some(pt) => Ok(Self::PawnPromote(start, end, pt)),
                                        None => Err(eyre!("Invalid piece identifier"))
                                    }
                                }
                                else {
                                    Err(eyre!("Invalid Promotion to invalid piece type"))
                                }
                            }
                            else {
                                Ok(Self::Move(start, end))
                            }
                        },
                        _ => Err(eyre!("Invalid From or To Location {} {}", start_str, end_str))
                    }
                }
                else {
                    Err(eyre!("Invalid Move Format"))
                }
            }
        }
    }
}