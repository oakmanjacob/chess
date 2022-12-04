use eyre::{eyre, Result};
use crate::chess_game::piece::*;
use crate::chess_game::chess_move::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Board {
    grid: [[Option<Piece>;8];8],
    turn: PieceColor,
    castle_avail: [(bool, bool); 2],
    en_passant: Option<(usize, usize)>,
    half_moves: u8,
    full_moves: u16,
    previous_states: HashMap<([u64;4], u8, PieceColor), u8>
}

impl Board {
    pub fn new() -> Board {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Bad FEN string in Board::new()")
    }

    pub fn empty() -> Board {
        Board {
            grid: Default::default(),
            turn: PieceColor::White,
            castle_avail: [(true, true), (true, true)],
            en_passant: None,
            half_moves: 0,
            full_moves: 1,
            previous_states: HashMap::new(),
        }
    }

    // https://www.chess.com/terms/fen-chess
    // https://en.wikipedia.org/wiki/Algebraic_notation_(chess)
    pub fn from_fen(fen_str: &str) -> Result<Board> {
        let mut result = Board::empty();

        let sections: Vec<&str> = fen_str.split(" ").collect();

        if sections.len() != 6 {
            return Err(eyre!("Too few segments"));
        }

        let rows: Vec<&str> = sections[0].split("/").collect();
        if rows.len() != 8 {
            return Err(eyre!("Wrong number of rows in board"));
        }

        for (row, value) in rows.iter().enumerate() {
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
                            result.grid[row][col] = Some(p);
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
            if sections[2].len() <= 4 && sections[2].len() != 0 {
                for character in sections[2].chars() {
                    match character {
                        'K' => result.castle_avail[PieceColor::White as usize].0 = true,
                        'Q' => result.castle_avail[PieceColor::White as usize].1 = true,
                        'k' => result.castle_avail[PieceColor::Black as usize].0 = true,
                        'q' => result.castle_avail[PieceColor::Black as usize].1 = true,
                        _ => return Err(eyre!("Invalid Castling Indicator"))
                    }
                }
            }
            else {
                return Err(eyre!("Invalid Castling Indicator"));
            }
        }

        if sections[3] != "-" {
            result.en_passant = match Board::get_loc(sections[3]) {
                Ok(ep) => match result.turn {
                    PieceColor::Black => if ep.0 == 6 && result.grid[ep.0 - 1][ep.1].map_or(false, |p| p == Piece { piece_type: PieceType::Pawn, piece_color: PieceColor::White }) {
                        Some(ep)
                    }
                    else {
                        return Err(eyre!("Invalid En Passant Black"))
                    },
                    PieceColor::White => if ep.0 == 1 && result.grid[ep.0 - 1][ep.1].map_or(false, |p| p == Piece { piece_type: PieceType::Pawn, piece_color: PieceColor::Black }) {
                        Some(ep)
                    }
                    else {
                        return Err(eyre!("Invalid En Passant White"))
                    },
                },
                Err(_) => return Err(eyre!("Invalid En Passant"))
            };
        }

        if let Ok(half_moves) = sections[4].parse::<u8>() {
            result.half_moves = half_moves;
        }
        else {
            return Err(eyre!("Invalid halfmove clock"));
        }

        if let Ok(full_moves) = sections[5].parse::<u16>() {
            result.full_moves = full_moves;
        }
        else {
            return Err(eyre!("Invalid fullmove clock"));
        }

        return Ok(result);
    }

    pub fn make_move(&mut self, move_str: &str) -> bool {
        let chess_move = match ChessMove::new(move_str) {
            Ok(cm) => cm,
            Err(_) => return false,
        };

        let player_side = match self.turn {
            PieceColor::Black => 0,
            PieceColor::White => 7,
        };

        let mut reset_hm = false;
        let current_state = self.encode_squares();

        let result = if self.is_valid_move(chess_move) {
            match chess_move {
                ChessMove::CastleKingside => {
                    self.grid[player_side][4] = None;
                    self.grid[player_side][5] = Some(Piece{piece_color: self.turn, piece_type: PieceType::Rook});
                    self.grid[player_side][6] = Some(Piece{piece_color: self.turn, piece_type: PieceType::King});
                    self.grid[player_side][7] = None;

                    self.en_passant = None;
                    true
                },
                ChessMove::CastleQueenside => {
                    self.grid[player_side][0] = None;
                    self.grid[player_side][2] = Some(Piece{piece_color: self.turn, piece_type: PieceType::King});
                    self.grid[player_side][3] = Some(Piece{piece_color: self.turn, piece_type: PieceType::Rook});
                    self.grid[player_side][4] = None;

                    self.en_passant = None;
                    true
                },
                ChessMove::Move(from, to) => {
                    if self.en_passant.map_or(false, |sq| sq == to) && self.grid[from.0][from.1].map_or(false, |p| p.piece_type == PieceType::Pawn) {
                        match self.turn {
                            PieceColor::Black => self.grid[to.0 - 1][to.1] = None,
                            PieceColor::White => self.grid[to.0 + 1][to.1] = None,
                        }
                    }
                    self.en_passant = None;

                    match self.grid[from.0][from.1] {
                        Some(Piece{piece_type: PieceType::King, piece_color: _}) => {
                            self.castle_avail[self.turn as usize] = (false, false);
                        },
                        Some(Piece{piece_type: PieceType::Rook, piece_color: _}) => {
                            if from == (7,0) || from == (0,0) {
                                self.castle_avail[self.turn as usize].1 = false;
                            }
                            else if from == (0,7) || from == (7,7) {
                                self.castle_avail[self.turn as usize].0 = false;
                            }
                        },
                        Some(Piece{piece_type: PieceType::Pawn, piece_color: _}) => {
                            if from.0.abs_diff(to.0) == 2 {
                                self.en_passant = match self.turn {
                                    PieceColor::Black => Some((2, from.1)),
                                    PieceColor::White => Some((5, from.1)),
                                }
                            }
                            reset_hm = true;
                        },
                        _ => {},
                    }

                    reset_hm |= self.grid[to.0][to.1].is_some();

                    self.grid[to.0][to.1] = self.grid[from.0][from.1];
                    self.grid[from.0][from.1] = None;

                    true
                },
                ChessMove::PawnPromote(from, to, piece_type) => {
                    self.grid[to.0][to.1] = Some(Piece{piece_color: self.turn, piece_type});
                    self.grid[from.0][from.1] = None;

                    reset_hm = true;
                    self.en_passant = None;
                    true
                },
            }
        }
        else {
            false
        };

        if result {
            if self.turn == PieceColor::Black {
                self.full_moves += 1;
            }

            if reset_hm {

            }
            else {
                self.half_moves += 1;
            }

            self.turn = !self.turn;
        }

        result
    }

    pub fn is_valid_move(&self, chess_move: ChessMove) -> bool {
        println!("{}", chess_move);

        let player_side = match self.turn {
            PieceColor::Black => 0,
            PieceColor::White => 7,
        };

        match chess_move {
            ChessMove::CastleKingside => { 
                self.castle_avail[self.turn as usize].0 &&
                self.grid[player_side][5].is_none() &&
                self.grid[player_side][6].is_none() &&
                self.count_threats((player_side, 4), self.turn) == 0 &&
                self.count_threats((player_side, 5), self.turn) == 0 &&
                self.count_threats((player_side, 6), self.turn) == 0
            },
            ChessMove::CastleQueenside => {
                self.castle_avail[self.turn as usize].1 &&
                self.grid[player_side][1].is_none() &&
                self.grid[player_side][2].is_none() &&
                self.grid[player_side][3].is_none() &&
                self.count_threats((player_side, 2), self.turn) == 0 &&
                self.count_threats((player_side, 3), self.turn) == 0 &&
                self.count_threats((player_side, 4), self.turn) == 0
            },
            ChessMove::Move(from, to) => {
                if from != to && from.0 < 8 || from.1 < 8 && to.0 < 8 || to.1 < 8 {
                    match self.grid[from.0][from.1] {
                        Some(piece) => {
                            if piece.piece_color == self.turn && match piece.piece_type {
                                PieceType::King => {
                                    from.0.abs_diff(to.0) <= 1 && from.1.abs_diff(to.1) <= 1 && self.grid[to.0][to.1].map_or(true, |p| p.piece_color == !self.turn)
                                },
                                PieceType::Queen => {
                                    let dir = ((to.0 as isize - from.0 as isize) / (from.0.abs_diff(to.0).max(1) as isize), (to.1 as isize - from.1 as isize) / (from.1.abs_diff(to.1).max(1) as isize));
                                    if dir.0 == 0 || dir.1 == 0 {
                                        let move_dist = (from.0.abs_diff(to.0) + from.1.abs_diff(to.1)) as u8;
                                        self.cast_ray(from, dir).map_or(true, |(p, distance)| distance > move_dist || (distance == move_dist as u8 && p.piece_color == !self.turn))
                                    }
                                    else {
                                        let move_dist = from.0.abs_diff(to.0) as u8;
                                        from.0.abs_diff(to.0) == from.1.abs_diff(to.1) && self.cast_ray(from, dir).map_or(true, |(p, distance)| distance > move_dist || (distance == move_dist as u8 && p.piece_color == !self.turn))
                                    }
                                },
                                PieceType::Bishup => {
                                    let dir = ((to.0 as isize - from.0 as isize) / (from.0.abs_diff(to.0).max(1) as isize), (to.1 as isize - from.1 as isize) / (from.1.abs_diff(to.1).max(1) as isize));
                                    let move_dist = from.0.abs_diff(to.0) as u8;
                                    from.0.abs_diff(to.0) == from.1.abs_diff(to.1) && self.cast_ray(from, dir).map_or(true, |(p, distance)| distance > move_dist || (distance == move_dist as u8 && p.piece_color == !self.turn))
                                },
                                PieceType::Knight => {
                                    ((from.0.abs_diff(to.0) == 1 && from.1.abs_diff(to.1) == 2) || (from.0.abs_diff(to.0) == 2 && from.1.abs_diff(to.1) == 1)) && self.grid[to.0][to.1].map_or(true, |p| p.piece_color == !self.turn)
                                },
                                PieceType::Rook => {
                                    let dir = ((to.0 as isize - from.0 as isize) / (from.0.abs_diff(to.0).max(1) as isize), (to.1 as isize - from.1 as isize) / (from.1.abs_diff(to.1).max(1) as isize));
                                    let move_dist = (from.0.abs_diff(to.0) + from.1.abs_diff(to.1)) as u8;
                                    (dir.0 == 0 || dir.1 == 0) && self.cast_ray(from, dir).map_or(true, |(p, distance)| distance > move_dist || (distance == move_dist as u8 && p.piece_color == !self.turn))
                                },
                                PieceType::Pawn => {
                                    let pawn_slots: (usize, usize, usize, isize) = match self.turn {
                                        PieceColor::Black => (1,2,3,1),
                                        PieceColor::White => (6,5,4,-1),
                                    };

                                    if from.1 == to.1 {
                                        (from.0 == pawn_slots.0 && to.0 == pawn_slots.2 && self.grid[pawn_slots.1][from.1].is_none() && self.grid[pawn_slots.2][from.1].is_none()) ||
                                        (from.0 as isize + pawn_slots.3 == to.0 as isize && self.grid[to.0][to.1].is_none())
                                    }
                                    else {
                                        from.0 as isize + pawn_slots.3 == to.0 as isize && (self.en_passant.map_or(false, |sq| sq == to) || self.grid[to.0][to.1].map_or(true, |p| p.piece_color == !self.turn))
                                    }
                                },
                            } {
                                let mut board_copy = self.clone();
                                board_copy.grid[to.0][to.1] = board_copy.grid[from.0][from.1];
                                board_copy.grid[from.0][from.1] = None;

                                if board_copy.en_passant.map_or(false, |sq| sq == to) && board_copy.grid[to.0][to.1].map_or(false, |p| p.piece_type == PieceType::Pawn) {
                                    match board_copy.turn {
                                        PieceColor::Black => board_copy.grid[to.0 - 1][to.1] = None,
                                        PieceColor::White => board_copy.grid[to.0 + 1][to.1] = None,
                                    }
                                }
    
                                !board_copy.has_check()
                            }
                            else {
                                false
                            }
                        },
                        None => false,
                    }
                }
                else {
                    false
                }
            },
            ChessMove::PawnPromote(from, to, _) => {
                // TODO find a better way to convert between directions for turns
                if from.0 < 8 || from.1 < 8 && to.0 < 8 || to.1 < 8 && // Pieces are within bounds of the checkerboard
                    from.0 == (7 - player_side) + 2 * (self.turn as usize) - 1 && // Must be moving from second to last row to last row
                    to.0 == (7 - player_side) && to.1 >= from.1.saturating_sub(1) && to.1 <= from.1 + 1 {
                    match self.grid[from.0][from.1] {
                        Some(Piece{piece_type: PieceType::Pawn, piece_color: turn}) => {
                            if turn == self.turn && ((to.1 == from.1 && self.grid[to.0][to.1].is_none()) || (to.1 != from.1 && self.grid[to.0][to.1].map_or(false, |p| p.piece_color == !self.turn))) {
                                // Must Move to empty
                                let mut next_board = self.clone();
                                next_board.grid[from.0][from.1] = None;
                                next_board.grid[to.0][to.1] = Some(Piece { piece_type: PieceType::Pawn, piece_color: self.turn });
                                next_board.turn = !next_board.turn;

                                !next_board.has_check()
                            }
                            else {
                                false
                            }
                        },
                        _ => false,
                    }
                }
                else {
                    false
                }
            },
        }
    }

    // Determines whether the current player's king is in check
    pub fn has_check(&self) -> bool {
        for x in 0..8 {
            for y in 0..8 {
                if self.grid[y][x].map_or(false, |p| p.piece_color == self.turn && p.piece_type == PieceType::King) {
                    if self.count_threats((y, x), self.turn) > 0 {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn count_threats(&self, loc: (usize, usize), cur_player: PieceColor) -> u8 {
        let mut result = 0;
        if loc.0 >= 8 || loc.1 >= 8 {
            return result;
        }

        // Check Knights
        let iloc = (loc.0 as isize, loc.1 as isize);
        for x in [-2, 2] {
            for y in [-1, 1] {
                for knight_sq in [(iloc.0 + x, iloc.1 + y), (iloc.0 + y, iloc.1 + x)]
                {
                    if knight_sq.0 >= 0 && knight_sq.0 < 8 && knight_sq.1 >= 0 && knight_sq.1 < 8 &&
                        self.grid[knight_sq.0 as usize][knight_sq.1 as usize].map_or(false, |p| p.piece_type == PieceType::Knight && p.piece_color == !cur_player) {
                        result += 1;
                    }
                }
            }
        }

        // Check Lines
        for dir in [(0,1),(0,-1),(1,0),(-1,0)] {
            result += match self.cast_ray(loc, dir) {
                Some((Piece{piece_color, piece_type}, distance)) => {
                    if piece_color != cur_player && ((distance == 1 && piece_type == PieceType::King) || piece_type == PieceType::Queen || piece_type == PieceType::Rook) {
                        println!("{:?} Threat", piece_type);
                        1
                    }
                    else {
                        0
                    }
                },
                None => 0,
            };
        }

        // Check Diagonals
        for dir in [(1,1),(1,-1),(-1,1),(-1,-1)] {
            let pawn_dir = match cur_player {
                PieceColor::Black => 1,
                PieceColor::White => -1,
            };

            result += match self.cast_ray(loc, dir) {
                Some((Piece{piece_color, piece_type}, distance)) => {
                    if piece_color != cur_player && ((distance == 1 && dir.0 == pawn_dir && piece_type == PieceType::Pawn) || piece_type == PieceType::Queen || piece_type == PieceType::Bishup) {
                        1
                    }
                    else {
                        0
                    }
                },
                None => 0,
            };
        }

        result
    }

    // Send a ray out in a particular direction return the first piece encountered and return the distance to it
    fn cast_ray(&self, loc: (usize, usize), dir: (isize, isize)) -> Option<(Piece, u8)> {
        if dir.0 == 0 && dir.1 == 0 {
            return None;
        }

        let mut iloc = (loc.0 as isize + dir.0, loc.1 as isize + dir.1);
        let mut count = 1;
        while iloc.0 >= 0 && iloc.0 < 8 && iloc.1 >= 0 && iloc.1 < 8 && self.grid[iloc.0 as usize][iloc.1 as usize].is_none() {
            iloc.0 += dir.0;
            iloc.1 += dir.1;
            count += 1;
        }

        if iloc.0 >= 0 && iloc.0 < 8 && iloc.1 >= 0 && iloc.1 < 8 {
            match self.grid[iloc.0 as usize][iloc.1 as usize] {
                Some(piece) => Some((piece.clone(), count)),
                None => None
            }
        }
        else {
            None
        }
    }

    pub fn get_loc(loc: &str) -> Result<(usize, usize), &str> {
        if !Board::is_valid_loc(loc) {
            return Err("Invalid square location");
        }

        let bytes = loc.as_bytes();

        Ok((7 - (bytes[1] - '1' as u8) as usize, (bytes[0] - 'a' as u8) as usize))
    }

    pub fn is_valid_loc(loc: &str) -> bool {
        let bytes: Vec<char> = loc.chars().collect();

        if bytes.len() != 2 {
            return false;
        }
        
        bytes[0] >= 'a' && bytes[0] <= 'h' && bytes[1] >= '1' && bytes[1] <= '8'
    }

    pub fn encode_squares(&self) -> ([u64; 4], u8, PieceColor) {
        let flags = (((self.castle_avail[0].0 as u8) | (self.castle_avail[0].1 as u8) | (self.castle_avail[1].0 as u8) |  (self.castle_avail[1].1 as u8)) << 4) |
                    self.en_passant.map_or(0, |(_, x)| (x as u8) | 8);

        let mut board: [u64; 4] = [0; 4];

        for y in 0..8 {
            for x in 0..8 {
                board[y / 2] <<= 4;
                board[y / 2] |= self.grid[y][x].map_or(0, |Piece{piece_color, piece_type}| (piece_color as u64) << 7 | (piece_type as u64) + 1 );
            }
        }

        (board, flags, self.turn)
    }

    pub fn encode_b13(&self) -> [u64; 4] {
        let mut result = [0u64; 5];

        for y in 0..8 {
            for x in 0..8 {
                for i in (0..4).rev() {
                    let value: u128 = (result[i] as u128) * 13u128;
                    result[i + 1] = (value >> 64) as u64;
                    result[i] = value as u64;
                }
                result[0] += self.grid[y][x].map_or(0, |Piece{piece_color, piece_type}| ((piece_color as u64) * 6) + (piece_type as u64) + 1 );
            }
        }

        let flags = ((self.turn as u64) << 8) | (((self.castle_avail[0].0 as u64) | (self.castle_avail[0].1 as u64) | (self.castle_avail[1].0 as u64) |  (self.castle_avail[1].1 as u64)) << 4) |
                    self.en_passant.map_or(0, |(_, x)| (x as u64) | 8);

        result[3] |= flags << 55;

        [result[0], result[1], result[2], result[3]]
    }

    pub fn print(&self) {
        use colored::*;

        println!("{}'s Turn", self.turn);
        println!("k:{}, q:{}, K:{}, Q:{}", self.castle_avail[0].0, self.castle_avail[0].1,self.castle_avail[1].0, self.castle_avail[1].1);
        //println!("   0  1  2  3  4  5  6  7 ");
        let mut toggle = false;
        for (row, index) in self.grid.iter().zip((1..9).rev()) {
            print!("{} ", index);
            for square in row.iter() {
                let value = format!(" {} ", match square {
                    Some(p) => p.get_name(),
                    None => ' ',
                }).normal();

                if toggle {
                    print!("{}", value.on_black());
                }
                else {
                    print!("{}", value.on_white());
                }
                toggle = !toggle;
            }
            toggle = !toggle;
            //println!(" {}", 8 - index);
            println!();
        }
        println!("   a  b  c  d  e  f  g  h ");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_loc()
    {
        for a in 'a'..='h' {
            for b in 1..=8 {
                let loc = format!("{}{}", a, b);
                assert!(Board::is_valid_loc(&loc));
            }
        }
    }

    #[test]
    fn test_bad_loc()
    {
        assert!(!Board::is_valid_loc("a123"));
        assert!(!Board::is_valid_loc(""));
        assert!(!Board::is_valid_loc("ab"));
        assert!(!Board::is_valid_loc("a9"));
        assert!(!Board::is_valid_loc("z1"));
        assert!(!Board::is_valid_loc("a"));
    }

    #[test]
    fn test_piece_equals()
    {
        let p1 = Piece::get_piece('P').unwrap();
        let p2 = Piece::get_piece('P').unwrap();
        let p3 = Piece::get_piece('p').unwrap();
        let p4 = Piece::get_piece('R').unwrap();

        assert_eq!(p1, p1);
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
        assert_ne!(p1, p4);
    }

    #[test]
    fn test_cast_ray()
    {
        let board = Board::new();

        for y in [-1, 0, 1] {
            for x in [-1, 0, 1] {
                let value = board.cast_ray((5, 3), (y, x));

                println!("{:?}, ({}, {})", value, y, x);
            }
        }

        println!("{}", board.count_threats((5,3), PieceColor::Black));
    }
}