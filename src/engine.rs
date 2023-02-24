mod tree;

use crate::game::tracker::VectorPieceTracker;

use super::game::{Game, piece::PieceColor, chess_move::ChessMove};
use rand::seq::IteratorRandom;

pub struct Engine {
    search_depth: u16,
    player: PieceColor,
    pub game: Game,
}

impl Engine {
    pub fn new(game: Game, player: PieceColor) -> Engine {
        Engine{search_depth: 6, player, game}
    }

    pub fn search_tree(&mut self, depth: u16) -> ChessMove {
        todo!()
    }

    pub fn get_best_move(&self) -> Option<ChessMove> {
        let tracker = VectorPieceTracker::from_board(&self.game.board);
        let moves = self.game.get_moves(&tracker);
        let mut rng = rand::thread_rng();
        moves.iter().choose(&mut rng).and_then(|chess_move| Some(*chess_move))
    }

    pub fn advance_move(&mut self, chess_move: ChessMove) {
        let mut tracker = VectorPieceTracker::from_board(&self.game.board);
        self.game.make_move(&chess_move, &mut tracker);
    }

    pub fn evaluate_state(game: &Game) -> u16 {
        todo!()
    }
}