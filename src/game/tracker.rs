use super::piece::*;
use super::position::Position;
use super::board::Board;

#[derive(Clone)]
pub struct VectorPieceTracker {
    piece_lists: [Vec<(Position, PieceType)>; 2]
}

impl VectorPieceTracker {
    pub fn default() -> VectorPieceTracker {
        VectorPieceTracker { piece_lists: [vec!(), vec!()] }
    }

    pub fn from_board(board: &Board) -> VectorPieceTracker {
        let mut tracker = VectorPieceTracker::default();

        for row in 0usize..=7usize {
            for column in 0usize..=7usize {
                if let Some(piece) = board.get(&Position::encode(row, column)) {
                    tracker.piece_lists[piece.color as usize].push((Position::encode(row, column), piece.piece_type));
                }
            }
        }

        tracker
    }

    pub fn replace_piece(&mut self, position: &Position, piece: &Piece) {
        self.piece_lists[0].retain(|(item_position, _)| item_position != position);
        self.piece_lists[1].retain(|(item_position, _)| item_position != position);
        self.piece_lists[piece.color as usize].push((*position, piece.piece_type));
    }

    pub fn remove_piece(&mut self, position: &Position, player_color: &PieceColor) {
        self.piece_lists[*player_color as usize].retain(|(item_position, _)| item_position != position);
    }

    pub fn get_pieces(&self, player_color: &PieceColor) -> Vec<(Position, PieceType)> {
        self.piece_lists[*player_color as usize].to_owned()
    }

    pub fn eq(&self, other: &VectorPieceTracker) -> bool {
        if self.piece_lists[0].len() != other.piece_lists[0].len() {
            println!("Black piece count not equal");
            return false;
        }

        for (position, piece) in self.piece_lists[0].iter() {
            if !other.piece_lists[0].iter().fold(false, |acc, (other_position, other_piece)| acc | (position == other_position && piece == other_piece)) {
                println!("Black {} {} not found in other", position, piece);
                return false;
            }
        }

        if self.piece_lists[1].len() != other.piece_lists[1].len() {
            println!("White piece count not equal");
            return false;
        }

        for (position, piece) in self.piece_lists[1].iter() {
            if !other.piece_lists[1].iter().fold(false, |acc, (other_position, other_piece)| acc | (position == other_position && piece == other_piece)) {
                println!("White {} {} not found in other", position, piece);
                return false;
            }
        }

        true
    }

    pub fn print(&self) {
        println!("Black");
        for (position, piece) in self.piece_lists[0].iter() {
            println!("{} {}", position, piece);
        }

        println!("White");
        for (position, piece) in self.piece_lists[1].iter() {
            println!("{} {}", position, piece);
        }
    }
}