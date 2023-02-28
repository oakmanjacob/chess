pub mod board;
pub mod piece;
pub mod chess_move;
pub mod position;

use std::hash::Hash;

use board::*;
use piece::*;
use position::Position;
use chess_move::ChessMove;
use eyre::{eyre, Result};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastleRights {
    pub kingside: bool,
    pub queenside: bool,
}

impl CastleRights {
    pub fn default() -> CastleRights {
        CastleRights{kingside: false, queenside: false}
    }
}

// TODO: Implement 50 moves rule

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Game {
    pub board: Board,
    pub en_passant: Option<Position>,
    pub turn: PieceColor,
    pub castle_rights: [CastleRights; 2],
    pub half_moves: u16,
}

impl Game {
    pub fn default() -> Game {
        Game {
            board: Board::default(),
            en_passant: None,
            turn: PieceColor::White,
            castle_rights: [CastleRights::default(); 2],
            half_moves: 0,
        }
    }

    pub fn new() -> Game {
        Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Decode standard FEN failed")
    }

    pub fn from_fen(fen_str: &str) -> Result<Game> {
        let mut result = Game::default();

        let sections: Vec<&str> = fen_str.split(' ').collect();

        if sections.len() != 6 {
            return Err(eyre!("Too few segments"));
        }

        let rows: Vec<&str> = sections[0].split('/').collect();
        if rows.len() != 8 {
            return Err(eyre!("Wrong number of rows in board"));
        }

        for (row, value) in rows.iter().rev().enumerate() {
            let mut col: usize = 0;
            for character in value.chars() {
                if col >= 8 {
                    return Err(eyre!("Row contains too many columns in row {}", row));
                }

                match character.to_digit(10) {
                    Some(skip) => {
                        col += skip as usize;
                    },
                    None => {
                        if let Some(p) = Piece::get_piece(character) {
                            result.board.add_piece(p, &Position::encode(row, col));
                            col += 1;
                        }
                        else {
                            return Err(eyre!("Invalid board square value of {}", character));
                        }
                    }
                }
            }

            if col < 8 {
                return Err(eyre!("Row contains too few columns in row {}", row));
            }
        }

        result.turn = match sections[1] {
            "b" | "B" => PieceColor::Black,
            "w" | "W" => PieceColor::White,
            _ => return Err(eyre!("Invalid current turn indicator")),
        };

        if sections[2] != "-" {
            if sections[2].len() <= 4 && !sections[2].is_empty() {
                for character in sections[2].chars() {
                    match character {
                        'K' => result.castle_rights[PieceColor::White as usize].kingside = true,
                        'Q' => result.castle_rights[PieceColor::White as usize].queenside = true,
                        'k' => result.castle_rights[PieceColor::Black as usize].kingside = true,
                        'q' => result.castle_rights[PieceColor::Black as usize].queenside = true,
                        _ => return Err(eyre!("Invalid Castling Indicator"))
                    }
                }
            }
            else {
                return Err(eyre!("Invalid Castling Indicator"));
            }
        }

        if sections[3] != "-" {
            result.en_passant = match Position::from_str(sections[3]) {
                Ok(pos) => {
                    let pos_tuple = pos.decode();
                    match result.turn {
                        PieceColor::Black => if pos_tuple.0 == 2 && result.board.get(&pos.forward(&!result.turn)).map_or(false, |&p| p == Piece { piece_type: PieceType::Pawn, color: PieceColor::White }) {
                            Some(pos)
                        }
                        else {
                            return Err(eyre!("Invalid En Passant Black"))
                        },
                        PieceColor::White => if pos_tuple.0 == 5 && result.board.get(&pos.forward(&!result.turn)).map_or(false, |&p| p == Piece { piece_type: PieceType::Pawn, color: PieceColor::Black }) {
                            Some(pos)
                        }
                        else {
                            return Err(eyre!("Invalid En Passant White"))
                        },
                    }
                },
                Err(msg) => return Err(eyre!("Invalid En Passant {}, {}", sections[3], msg))
            };
        }

        Ok(result)
    }

    pub fn to_fen(&self) -> String {
        let mut board = "".to_owned();

        for row in (0..8).rev() {
            let mut counter = 0;
            for col in 0..8 {
                let position = Position::encode(row, col);
                match self.board.get(&position) {
                    Some(piece) => {
                        if counter > 0 {
                            board = format!("{}{}", board, counter);
                            counter = 0;
                        }

                        board = format!("{}{}", board, piece.to_char());
                    },
                    None => {
                        counter += 1;
                    },
                }
            }
            if counter > 0 {
                board = format!("{}{}", board, counter);
            }

            if row > 0 {
                board = format!("{}/", board);
            }
        }

        let mut castle = "".to_owned();
        if self.castle_rights[1].kingside {
            castle = format!("{}K", castle);
        }

        if self.castle_rights[1].queenside {
            castle = format!("{}Q", castle);
        }

        if self.castle_rights[0].kingside {
            castle = format!("{}k", castle);
        }

        if self.castle_rights[0].queenside {
            castle = format!("{}q", castle);
        }

        if castle.is_empty() {
            castle = "-".to_owned();
        }

        format!("{} {} {} {}", board, self.turn, castle, self.en_passant.map_or("-".to_owned(), |position| position.to_string()))
    }

    /// Gets all valid moves from a specific chess position
    pub fn get_moves(&self) -> Vec<ChessMove> {
        let mut moves = vec!();

        // TODO: Optimize function so we don't have to look at every check

        // Go through all pieces and check for valid moves
        let piece_positions: Vec<(Position, PieceType)> = self.board.get_pieces(&self.turn);

        let (king_position, _) = match piece_positions.iter().find(|(_, piece_type)| *piece_type == PieceType::King) {
            Some(val) => val,
            None => {
                println!("Attempted to get moves but piece list has no king!");
                return moves;
            }
        };

        for (from, cur_piece_type) in piece_positions.iter() {
            match cur_piece_type {
                PieceType::King => {
                    let (king_row, king_column) = from.decode_isize();

                    for increments in [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)] {
                        if let Some(to) = Position::encode_checked(king_row + increments.0, king_column + increments.1) {
                            if self.board.get(&to).map_or(true, |&Piece{piece_type: _, color}| color != self.turn) && self.board.test_move(from, &to, &to, &self.turn) {
                                moves.push(ChessMove::Move(*from, to));
                            }
                        }
                    }
                },
                PieceType::Queen => {
                    for to in self.board.get_bishup_move_positions(from, &self.turn, false) {
                        if self.board.test_move(from, &to, king_position, &self.turn) {
                            moves.push(ChessMove::Move(*from, to));
                        }
                    }

                    for to in self.board.get_rook_move_positions(from, &self.turn, false) {
                        if self.board.test_move(from, &to, king_position, &self.turn) {
                            moves.push(ChessMove::Move(*from, to));
                        }
                    }
                },
                PieceType::Bishup => {
                    for to in self.board.get_bishup_move_positions(from, &self.turn, false) {
                        if self.board.test_move(from, &to, king_position, &self.turn) {
                            moves.push(ChessMove::Move(*from, to));
                        }
                    }
                },
                PieceType::Rook => {
                    for to in self.board.get_rook_move_positions(from, &self.turn, false) {
                        if self.board.test_move(from, &to, king_position, &self.turn) {
                            moves.push(ChessMove::Move(*from, to));
                        }
                    }
                },
                PieceType::Knight => {
                    for to in self.board.get_knight_move_positions(from, &self.turn, false) {
                        if self.board.test_move(from, &to, king_position, &self.turn) {
                            moves.push(ChessMove::Move(*from, to));
                        }
                    }
                },
                PieceType::Pawn => {
                    let must_promote = [(PieceColor::Black, 1usize), (PieceColor::White, 6usize)].contains(&(self.turn, from.row()));
                    let promotion_types = [PieceType::Queen, PieceType::Rook, PieceType::Bishup, PieceType::Knight];

                    let to = from.forward(&self.turn);
                    let (to_row, to_column) = to.decode_isize();
                    if self.board.get(&to).is_none() {
                        if self.board.test_move(from, &to, king_position, &self.turn) {
                            if must_promote {
                                for piece_type in promotion_types {
                                    moves.push(ChessMove::PawnPromote(*from, to, piece_type))
                                }
                            }
                            else {
                                moves.push(ChessMove::Move(*from, to));
                            }
                        }

                        if [(PieceColor::Black, 6usize), (PieceColor::White, 1usize)].contains(&(self.turn, from.row())) {
                            let to = to.forward(&self.turn);
                            if self.board.get(&to).is_none() && self.board.test_move(from, &to, king_position, &self.turn) {
                                if must_promote {
                                    for piece_type in promotion_types {
                                        moves.push(ChessMove::PawnPromote(*from, to, piece_type))
                                    }
                                }
                                else {
                                    moves.push(ChessMove::Move(*from, to));
                                }
                            }
                        }
                    }

                    // Check captures
                    for position_values in [(to_row, to_column + 1),(to_row, to_column - 1)] {
                        if let Some(to) = Position::encode_checked(position_values.0, position_values.1) {
                            if Some(to) == self.en_passant {
                                let mut next_board = self.board;
                                next_board.make_move(from, &to);
                                if !next_board.has_check(king_position, &self.turn) {
                                    moves.push(ChessMove::Move(*from, to));
                                }
                            }
                            else if self.board.get(&to).map_or(false, |&Piece{piece_type: _, color}| color != self.turn) && self.board.test_move(from, &to, king_position, &self.turn) {
                                if must_promote {
                                    for piece_type in promotion_types {
                                        moves.push(ChessMove::PawnPromote(*from, to, piece_type))
                                    }
                                }
                                else {
                                    moves.push(ChessMove::Move(*from, to));
                                }
                            }
                        }
                    }
                },
            }
        }

        // Check for Castle Kingside
        if self.castle_rights[self.turn as usize].kingside {
            let transit_positions = match self.turn {
                PieceColor::Black => [Position::encode(7, 5), Position::encode(7, 6)],
                PieceColor::White => [Position::encode(0, 5), Position::encode(0, 6)],
            };

            // Make sure middle values are empty and king can't pass through check
            let mut is_kingside_valid = true;
            if self.board.has_check(king_position, &self.turn) {
                is_kingside_valid = false;
            }

            for transit_position in transit_positions {
                if self.board.get(&transit_position).is_some() || self.board.has_check(&transit_position, &self.turn) {
                    is_kingside_valid = false;
                    break;
                }
            }

            if is_kingside_valid {
                moves.push(ChessMove::CastleKingside);
            }
        }

        // Check for Castle Queenside
        if self.castle_rights[self.turn as usize].queenside {
            let (rook_transit, transit_positions) = match self.turn {
                PieceColor::Black => (Position::encode(7, 1), [Position::encode(7, 2), Position::encode(7, 3)]),
                PieceColor::White => (Position::encode(0, 1), [Position::encode(0, 2), Position::encode(0, 3)]),
            };

            // Make sure middle values are empty and king can't pass through check
            let mut is_queenside_valid = true;
            if self.board.get(&rook_transit).is_some() {
                is_queenside_valid = false;
            }

            if self.board.has_check(king_position, &self.turn) {
                is_queenside_valid = false;
            }

            for transit_position in transit_positions {
                if self.board.get(&transit_position).is_some() || self.board.has_check(&transit_position, &self.turn) {
                    is_queenside_valid = false;
                    break;
                }
            }

            if is_queenside_valid {
                moves.push(ChessMove::CastleQueenside);
            }
        }

        moves
    }

    /// Performs a move on a board in place without validation
    /// 
    /// # Arguments
    /// 
    /// * `chess_move` - A ChessMove generated by the get_moves function 
    pub fn make_move(&mut self, chess_move: &ChessMove) {
        let mut remove_en_passant = true;

        self.half_moves += 1;

        match chess_move {
            ChessMove::CastleKingside => {
                self.castle_rights[self.turn as usize].kingside = false;
                self.castle_rights[self.turn as usize].queenside = false;

                let (king_from, king_to, rook_from, rook_to) = match self.turn {
                    PieceColor::White => (Position::encode(0, 4), Position::encode(0, 6), Position::encode(0, 7), Position::encode(0, 5)),
                    PieceColor::Black => (Position::encode(7, 4), Position::encode(7, 6), Position::encode(7, 7), Position::encode(7, 5)),
                };

                self.board.make_move(&king_from, &king_to);
                self.board.make_move(&rook_from, &rook_to);
            },
            ChessMove::CastleQueenside => {
                self.castle_rights[self.turn as usize].kingside = false;
                self.castle_rights[self.turn as usize].queenside = false;

                let (king_from, king_to, rook_from, rook_to) = match self.turn {
                    PieceColor::White => (Position::encode(0, 4), Position::encode(0, 2), Position::encode(0, 0), Position::encode(0, 3)),
                    PieceColor::Black => (Position::encode(7, 4), Position::encode(7, 2), Position::encode(7, 0), Position::encode(7, 3)),
                };

                self.board.make_move(&king_from, &king_to);
                self.board.make_move(&rook_from, &rook_to);
            },
            ChessMove::Move(from, to) => {
                // Handle moves which would break castling rights.
                if self.board.get(from).map_or(false, |&Piece{piece_type, color: _}| piece_type == PieceType::King) {
                    self.castle_rights[self.turn as usize].kingside = false;
                    self.castle_rights[self.turn as usize].queenside = false;
                }
                else if self.board.get(from).map_or(false, |&Piece{piece_type, color: _}| piece_type == PieceType::Rook) {
                    if from.column() == 7 {
                        self.castle_rights[self.turn as usize].kingside = false;
                    }
                    else {
                        self.castle_rights[self.turn as usize].queenside = false;
                    }
                }

                // Handle rook captures
                if self.board.get(to).map_or(false, |&Piece{piece_type, color: _}| piece_type == PieceType::Rook) {
                    if to.column() == 7 {
                        self.castle_rights[!self.turn as usize].kingside = false;
                    }
                    else {
                        self.castle_rights[!self.turn as usize].queenside = false;
                    }
                }

                // Handle capture by en passants
                if Some(to) == self.en_passant.as_ref() && self.board.get(from).map_or(false, |Piece{piece_type, color: _}| piece_type == &PieceType::Pawn) {
                    self.board.remove_piece(&to.backward(&self.turn)).take();
                }

                // Handle double move and marking en passant square
                let double_move_from_to = match self.turn {
                    PieceColor::Black => (6, 4),
                    PieceColor::White => (1, 3),
                };

                let from_row = from.row();
                let to_row = to.row();

                if (from_row, to_row) == double_move_from_to && self.board.get(from).map_or(false, |&Piece{piece_type, color: _}| piece_type == PieceType::Pawn) {
                    self.en_passant = Some(to.clone().backward(&self.turn));
                    remove_en_passant = false;
                }

                self.board.make_move(from, to);

            },
            ChessMove::PawnPromote(from, to, piece_type) => {
                // Handle rook captures
                if self.board.get(to).map_or(false, |&Piece{piece_type, color: _}| piece_type == PieceType::Rook) {
                    if to.column() == 7 {
                        self.castle_rights[!self.turn as usize].kingside = false;
                    }
                    else {
                        self.castle_rights[!self.turn as usize].queenside = false;
                    }
                }

                self.board.remove_piece(from);
                self.board.add_piece(Piece{piece_type: *piece_type, color: self.turn}, to);
            },
        }

        self.turn = !self.turn;
        if remove_en_passant {
            self.en_passant = None;
        }
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        println!("{}'s Turn", self.turn);
        println!("k:{}, q:{}, K:{}, Q:{}", self.castle_rights[0].kingside, self.castle_rights[0].queenside,self.castle_rights[1].kingside, self.castle_rights[1].queenside);

        self.board.print();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Game {
        pub fn perft(&mut self, depth: usize) -> Vec<(ChessMove, usize)> {
            let moves = self.get_moves();
        
            let mut result: Vec<(ChessMove, usize)> = vec!();
        
            for chess_move in moves.iter() {
                let mut next_game = self.clone();
                next_game.make_move(chess_move);
        
                result.push((*chess_move, next_game.perft_helper(depth - 1)));
            }
        
            result
        }
        
        fn perft_helper(&mut self, depth: usize) -> usize {
            if depth == 0 {
                return 1;
            }
        
            let moves = self.get_moves();
            let mut result = 0;
        
            for chess_move in moves.iter() {
                let mut next_game = self.clone();
                next_game.make_move(chess_move);
                result += next_game.perft_helper(depth - 1);
            }
        
            result
        }
    }

    // 333.39
    #[test]
    fn test_perft_start()
    {
        let mut curr_game = Game::new();

        let values = curr_game.perft(5);

        let expected_set: Vec<(&str, usize)> = vec!(
            ("a2a3", 181046),
            ("b2b3", 215255),
            ("c2c3", 222861),
            ("d2d3", 328511),
            ("e2e3", 402988),
            ("f2f3", 178889),
            ("g2g3", 217210),
            ("h2h3", 181044),
            ("a2a4", 217832),
            ("b2b4", 216145),
            ("c2c4", 240082),
            ("d2d4", 361790),
            ("e2e4", 405385),
            ("f2f4", 198473),
            ("g2g4", 214048),
            ("h2h4", 218829),
            ("b1a3", 198572),
            ("b1c3", 234656),
            ("g1f3", 233491),
            ("g1h3", 198502),
        );

        let expected_total = 4865609;

        let mut total = 0;
        for (chess_move, amount) in values {
            assert!(expected_set.iter().any(|pair| pair == &(chess_move.to_string().as_str(), amount)));
            total += amount;
        }

        assert!(total == expected_total);
    }

    #[test]
    fn test_perft_pos5()
    {
        let mut curr_game = Game::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").expect("");
        let values = curr_game.perft(4);

        let expected_set: Vec<(&str, usize)> = vec!(
            ("a2a3", 46833),
            ("b2b3", 46497),
            ("c2c3", 49406),
            ("g2g3", 44509),
            ("h2h3", 46762),
            ("a2a4", 48882),
            ("b2b4", 46696),
            ("g2g4", 45506),
            ("h2h4", 47811),
            ("d7c8q", 44226),
            ("d7c8r", 38077),
            ("d7c8b", 65053),
            ("d7c8n", 62009),
            ("b1d2", 40560),
            ("b1a3", 44378),
            ("b1c3", 50303),
            ("e2g1", 48844),
            ("e2c3", 54792),
            ("e2g3", 51892),
            ("e2d4", 52109),
            ("e2f4", 51127),
            ("c1d2", 46881),
            ("c1e3", 53637),
            ("c1f4", 52350),
            ("c1g5", 45601),
            ("c1h6", 40913),
            ("c4b3", 43453),
            ("c4d3", 43565),
            ("c4b5", 45559),
            ("c4d5", 48002),
            ("c4a6", 41884),
            ("c4e6", 49872),
            ("c4f7", 43289),
            ("h1f1", 46101),
            ("h1g1", 44668),
            ("d1d2", 48843),
            ("d1d3", 57153),
            ("d1d4", 57744),
            ("d1d5", 56899),
            ("d1d6", 43766),
            ("e1f1", 49775),
            ("e1d2", 33423),
            ("e1f2", 36783),
            ("O-O", 47054),
        );

        let expected_total = 2103487;

        let mut total = 0;
        for (chess_move, amount) in values {
            assert!(expected_set.iter().any(|pair| pair == &(chess_move.to_string().as_str(), amount)));
            total += amount;
        }

        assert!(total == expected_total);
    }
}