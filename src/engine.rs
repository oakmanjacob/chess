mod tree;

use super::game::{Game, piece::PieceColor};

struct Engine {
    tree: Tree,
    search_depth: u16,
    player: PieceColor,
}

impl Engine {
    pub fn new(game: Game, player: PieceColor) -> Engine {
        Engine{tree: Tree::new(), search_depth: 6, player}
    }

    pub fn search_tree(&mut self, depth: u16) -> ChessMove {
        todo!()
    }

    pub fn get_best_move(&self, game: &Game) -> ChessMove {
        todo!()
    }

    pub fn advance_move(&mut self, chess_move: ChessMove) {
        todo!()
    }

    pub fn evaluate_state(game: &Game) -> u16 {
        todo!()
    }
}