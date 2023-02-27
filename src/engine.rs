use crate::game::piece::{PieceType, Piece};

use super::game::{chess_move::ChessMove, piece::PieceColor, position::Position, Game};
use rand::Rng;
use std::cmp;
use lazy_static::lazy_static;

pub struct Engine {
    pub game: Game,
    search_depth: u16,
    player: PieceColor,
}

pub struct Node {
    children: Vec<(ChessMove, Node)>,
    value: i32,
    depth: u16,
}

impl Engine {
    pub fn new(game: Game, player: PieceColor) -> Engine {
        Engine {
            game,
            search_depth: 5,
            player,
        }
    }

    // TODO: Implement iterative deepening
    pub fn search_tree(&self, game: &Game, depth: u16, mut alpha: i32, mut beta: i32) -> i32 {
        if depth == 0 {
            return self.evaluate_state(game);
        }

        let moves = game.get_moves();

        if moves.is_empty() {
            if game
                .board
                .has_check(&game.board.get_king(&game.turn).unwrap(), &game.turn)
            {
                if self.player == game.turn {
                    return i32::MIN;
                } else {
                    return i32::MAX;
                }
            } else {
                return 0;
            }
        }

        let mut value;

        // Evaluate
        if game.turn == self.player {
            value = i32::MIN;
            for chess_move in moves.iter() {
                let mut next_game = game.clone();
                next_game.make_move(chess_move);
                value = cmp::max(value, self.search_tree(&next_game, depth - 1, alpha, beta));

                if value > beta {
                    break;
                }
                alpha = cmp::max(value, alpha);
            }
        } else {
            // min
            value = i32::MAX;
            for chess_move in moves.iter() {
                let mut next_game = game.clone();
                next_game.make_move(chess_move);

                if next_game.board.get_king(&PieceColor::White).is_none()
                    || next_game.board.get_king(&PieceColor::White).is_none()
                {
                    println!(
                        "For {} after move {} we have no king in board {}",
                        game.to_fen(),
                        chess_move,
                        next_game.to_fen()
                    );
                }

                value = cmp::min(value, self.search_tree(&next_game, depth - 1, alpha, beta));

                if value < alpha {
                    break;
                }

                beta = cmp::min(value, beta);
            }
        }

        value
    }

    pub fn get_best_move(&self) -> Option<ChessMove> {
        let moves = self.game.get_moves();

        let mut returned_move: Option<ChessMove> = None;
        let mut max_value = i32::MIN;

        for chess_move in moves.iter() {
            let mut next_game = self.game.clone();
            next_game.make_move(chess_move);

            let value = self.search_tree(&next_game, self.search_depth - 1, i32::MIN, i32::MAX);

            if value > max_value || (value == max_value && returned_move.is_none()) {
                max_value = value;
                returned_move = Some(*chess_move);
            }
        }

        returned_move
    }

    pub fn advance_move(&mut self, chess_move: ChessMove) {
        self.game.make_move(&chess_move);
    }

    pub fn evaluate_state(&self, game: &Game) -> i32 {
        let mut rng = rand::thread_rng();
        let mut score = rng.gen_range(-10i32..=10);

        lazy_static!(
            static ref PAWN_BOARD: [[i32; 8]; 8] = [
                [100,100,100,100,100,100,100,100],
                [150,150,100,100,100,100,150,150],
                [130,150,125,125,125,125,150,130],
                [135,140,150,150,150,150,140,135],
                [135,140,150,150,150,150,140,135],
                [200,200,200,200,200,200,200,200],
                [300,300,300,300,300,300,300,300],
                [100,100,100,100,100,100,100,100]];

            static ref KNIGHT_BOARD: [[i32; 8]; 8] = [
                [300,300,300,300,300,300,300,300],
                [300,300,300,300,300,300,300,300],
                [300,300,350,350,350,350,300,300],
                [300,300,350,350,350,350,300,300],
                [300,300,350,350,350,350,300,300],
                [300,300,350,350,350,350,300,300],
                [300,300,300,300,300,300,300,300],
                [300,300,300,300,300,300,300,300],];
        );

        // TODO: Knights to center of board

        // TODO: Pawn positioning

        // TODO: Want to maximize threatened squares

        // TODO: Want to push king to corner in endgame

        if game.castle_rights[self.player as usize].kingside {
            score += 50;
        }

        if game.castle_rights[self.player as usize].queenside {
            score += 50;
        }

        if game.castle_rights[!self.player as usize].kingside {
            score -= 50;
        }

        if game.castle_rights[!self.player as usize].queenside {
            score -= 50;
        }

        for row in 0usize..=7usize {
            for column in 0usize..=7usize {
                if let Some(piece) = game.board.get(&Position::encode(row, column)) {
                    let piece_value = match piece.piece_type {
                        PieceType::King => 0,
                        PieceType::Queen => 900,
                        PieceType::Rook => 500,
                        PieceType::Bishup => 300,
                        PieceType::Knight => KNIGHT_BOARD[row][column],
                        PieceType::Pawn => match self.player {
                            PieceColor::Black => PAWN_BOARD[7 - row][column],
                            PieceColor::White => PAWN_BOARD[row][column]
                        },
                    };

                    if piece.color == self.player {
                        score += piece_value;
                    } else {
                        score -= piece_value;
                    }
                }
            }
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_engine_with_moves(moves_list: Vec<&str>) -> Engine {
        let mut engine = Engine::new(Game::new(), PieceColor::White);

        for move_text in moves_list.iter() {
            if let Some(chess_move) = ChessMove::from_str(move_text) {
                engine.advance_move(chess_move);
            } else {
                println!("Could not parse move from string {}", move_text);
            }
        }

        engine
    }

    #[test]
    fn test_could_not_find_king_0() {
        let moves_list = vec![
            "d2d4", "e7e6", "c2c4", "d8f6", "b1c3", "b8c6", "c3e4", "f8b4", "c1d2", "f6d4", "a1c1",
            "d4b2", "c1c3", "c6d4", "a2a4", "b4c3", "d2c3", "d4c2", "e1d2", "b2a2", "d1c2", "a2c4",
            "d2c1", "f7f5", "g1f3", "f5e4", "f3d2", "c4c5", "d2e4", "c5f5", "c3g7", "b7b5", "g7h8",
            "b5a4", "c2c7", "f5e4", "c1d2", "e4b4", "h8c3", "b4e4", "c3h8", "e4b4", "c7c3", "b4f4",
            "e2e3", "f4f2", "f1e2", "f2g2", "c3a5", "g2h1", "h8d4", "h1g2", "d2c2", "g2e2", "a5d2",
            "e2c4", "d2c3", "c4a2", "c2c1", "a8b8", "c3d3", "c8a6", "d3c2", "b8c8", "d4c5", "a2c2",
            "c1c2", "c8c5", "c2b2", "c5c4", "b2a3", "c4h4", "h2h3", "h4h3", "a3a4", "h3e3", "a4a5",
            "e3a3", "a5b4", "a3a2", "b4b3", "a2a1", "b3c3", "a1a2", "c3b4",
        ];

        let engine = get_engine_with_moves(moves_list);
        engine.get_best_move();
    }

    #[test]
    fn test_could_not_find_king_1() {
        let moves_list = vec![
            "e2e3", "e7e6", "d1g4", "g8f6", "g4h4", "b8c6", "f1b5", "a7a6", "b5c6", "b7c6", "b2b3",
            "h7h6", "g1f3", "d8e7", "c1a3", "c6c5", "f3g5", "h8g8", "g5e4", "d7d6", "e4g3", "c8d7",
            "h4f4", "e6e5", "f4f3", "a8b8", "d2d3", "e8d8", "g3f5", "e7e6", "g2g4", "d6d5", "h2h3",
            "b8b5", "b1c3", "b5a5", "a3c1", "d8c8", "c1d2", "e5e4", "d3e4", "f6e4", "c3e4", "d5e4",
            "f3d1", "a5b5", "c2c4", "b5b7", "d2a5", "c8b8", "h3h4", "b8a8", "f2f4", "e4f3", "d1f3",
            "d7c6", "e3e4", "c6e4", "f5d4", "c5d4", "f3e2", "e4h1", "e2e6", "f7e6", "g4g5", "h1f3",
            "g5h6", "g7h6", "e1f2", "f3e4", "a1e1", "g8g7", "e1e4", "g7g6", "e4f4", "f8e7", "f2f1",
            "a8a7", "f4d4", "c7c6", "d4e4", "e7c5", "f1e1", "c5b6", "a5b6", "a7b6", "h4h5", "g6f6",
            "b3b4", "b7g7", "e4e2", "f6f5", "e2e6", "g7g2", "e6h6", "b6c7", "h6h7", "c7d8", "h7h8",
            "d8c7", "h8h7", "c7b8", "h7h8", "b8a7", "h8h7", "a7b6", "c4c5", "b6b5", "h7b7", "b5a4",
            "b7b6", "f5f8", "b6c6", "f8h8", "e1f1", "a4b4", "f1g2", "h8h5", "c6a6", "b4c5",
        ];

        let engine = get_engine_with_moves(moves_list);
        engine.get_best_move();
    }

    #[test]
    fn test_could_not_find_king_2() {
        let moves_list = vec![
            "e2e3", "e7e6", "d1g4", "g8f6", "g4h4", "b8c6", "g1h3", "f8d6", "h4g5", "d6f8", "f1b5",
            "h7h6", "g5f4", "c6b4", "b5a4", "b4a6", "c2c3", "c7c6", "a4d1", "g7g5", "f4d4", "d7d6",
            "f2f4", "f8e7", "h3g5", "h6g5", "f4g5", "h8g8", "g5f6", "e7f6", "d4c4", "g8g2", "d1f3",
            "d6d5", "c4f1", "d8b6", "f3g2", "f6h4", "e1d1", "e6e5", "d1c2", "b6a5", "b2b4", "a5a4",
            "c2b2", "a6b4", "c3b4", "a4b4", "b2c2", "b4g4", "b1c3", "b7b5", "d2d3", "b5b4", "c3a4",
            "g4h5", "a1b1", "h5g4", "g2f3", "g4e6", "b1b4", "h4g5", "f3h5", "g5e7", "a4c5", "e7c5",
            "b4b5", "c6b5", "c1a3", "c5a3", "c2b1", "d5d4", "f1f3", "a8b8", "e3d4", "c8b7", "f3f2",
            "b7h1", "f2f1", "h1d5", "f1f2", "b8c8", "f2g1", "c8c1", "g1c1", "a3c1", "b1c1", "d5a2",
            "c1b2", "e8f8", "d4e5", "e6d5", "d3d4", "a7a5", "h2h3", "f7f5", "e5f6", "d5h5", "b2a2",
            "h5d1", "d4d5", "f8e8", "a2b2", "b5b4", "h3h4", "a5a4", "b2a2", "d1g4", "h4h5", "g4f5",
            "h5h6", "f5f6", "h6h7", "e8f7", "d5d6", "f6h4", "d6d7", "h4d4", "a2b1", "b4b3", "b1c1",
            "a4a3", "h7h8b", "d4h8", "c1d1", "f7e6", "d7d8r", "h8d8", "d1e2", "d8d7", "e2f1",
            "d7d6", "f1g1", "e6f5", "g1f1", "d6e6", "f1f2", "b3b2",
        ];

        let engine = get_engine_with_moves(moves_list);
        engine.get_best_move();
    }
    #[test]
    fn test_could_not_find_king_3() {
        let moves_list = vec![
            "e2e3", "c7c6", "d1h5", "d7d5", "f1e2", "g8f6", "h5e5", "h7h5", "h2h3", "h8g8", "e2h5",
            "b7b6", "h5f7", "e8f7", "g1f3", "a7a6", "b1a3", "b6b5", "e5f4", "g7g6", "c2c3", "e7e6",
            "f3g5", "f7e8", "d2d4", "g8g7", "a3b1", "d8e7", "b1d2", "a6a5", "g2g3", "c8d7", "f4c7",
            "f6h5", "c7b7", "g7g8", "b7a8", "e7g5", "a8b8", "e8f7", "d2f3", "g5e7", "f3e5", "f7f6",
            "b8a7", "e7d6", "a7d7", "d6d7", "e5d7", "f6g7", "d7b8", "c6c5", "b8d7", "g6g5", "d4c5",
            "g7h6", "h3h4", "g8g7", "d7f8", "g7a7", "h4g5", "h6g7", "f8e6", "g7g6", "c5c6", "g6f7",
            "c6c7", "f7e6", "c7c8q", "e6e5",
        ];

        let engine = get_engine_with_moves(moves_list);
        engine.get_best_move();
    }

    #[test]
    fn test_did_not_mate_0() {
        let moves_list = vec![
            "b1c3", "b8c6", "g1f3", "d7d5", "e2e3", "g8f6", "f1b5", "c8f5", "f3e5", "d8d6", "d2d4",
            "a8c8", "g2g4", "f5d7", "b5c6", "d7c6", "d1f3", "h7h6", "f3f5", "e7e6", "f5d3", "a7a5",
            "d3e2", "g7g5", "e2d1", "e8e7", "e5d3", "c8e8", "h2h3", "f8g7", "d1e2", "b7b5", "c3b1",
            "f6e4", "e2f3", "c6b7", "b1d2", "e8g8", "f3e2", "g8d8", "d2f1", "d8f8", "e2d1", "c7c6",
            "d1e2", "f8c8", "c2c3", "d6b8", "a2a3", "h8d8", "e2d1", "c8c7", "f2f3", "e4f6", "d1c2",
            "b8c8", "b2b4", "a5a4", "c2f2", "g7h8", "d3c5", "d8d6", "h3h4", "c8b8", "f2h2", "b8g8",
            "h4g5", "g8c8", "g5f6", "h8f6", "h2h6", "c8a8", "g4g5", "a8e8", "h6f6",
            "e7f8",
            // a1a2
            // Extra moves e8d8
            // Extra moves h1h8
        ];

        let engine = get_engine_with_moves(moves_list);
        println!("Got to FEN {}", engine.game.to_fen());
        let best_move = engine.get_best_move().expect("No move returned");
        assert_eq!(best_move.to_string(), "h1h8".to_string());
    }

    #[test]
    fn test_false_mate() {
        let moves_list = vec![
            "e2e3", "e7e5", "d1g4", "b8c6", "g4e4", "d7d5", "e4a4", "d8d7", "f1b5", "f8b4", "g1f3",
            "e5e4", "f3e5", "d7d6", "e5c6", "b4c5", "b2b4", "c5b6", "c6a7", "c7c6", "a7c8", "a8c8",
            "b5f1", "g8f6", "c1b2", "O-O", "d2d3", "c8e8", "b1a3", "f6g4", "d3e4", "e8e4", "c2c4",
            "d6f4", "a4c2", "g4f2", "c2f2", "e4e3", "f1e2", "f4d6", "c4c5", "b6c5", "b4c5", "d6c5",
            "e1f1", "f8e8", "e2h5", "e8e7", "a1d1", "c5a5", "a3c2", "e3e4", "b2a3", "e7d7", "d1d3",
            "a5a6", "f2f3", "a6a8", "f3f5", "a8d8", "a3c1", "g7g6", "h5g6", "f7g6", "d3g3", "c6c5",
            "c1h6", "d7f7", "g3g6", "h7g6", "f5f3", "f7f3", "g2f3", "d8f6", "f1f2", "f6h4", "f2f1",
            "e4e8", "h6e3", "d5d4", "e3c1", "g8g7", "c1a3", "d4d3", "a3b2", "g7f7", "c2a1", "e8e2",
        ];

        let engine = get_engine_with_moves(moves_list);
        println!("Got to FEN {}", engine.game.to_fen());
        let best_move = engine.get_best_move().expect("No move returned");
        assert_eq!(best_move.to_string(), "h1h8".to_string());
    }
}
