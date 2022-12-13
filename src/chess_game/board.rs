use eyre::{eyre, Result};
use crate::chess_game::piece::*;
use crate::chess_game::chess_move::*;
use std::collections::HashMap;

use super::piece;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EncodedBoard
{
    grid: [u64; 4],
    flags: u8,
    turn: PieceColor
}

#[derive(Debug, Clone)]
pub struct Board {
    grid: [[Option<Piece>;8];8],
    turn: PieceColor,
    castle_avail: [(bool, bool); 2],
    en_passant: Option<(usize, usize)>,
    half_moves: u8,
    full_moves: u16,
    previous_states: HashMap<EncodedBoard, u8>
}

pub enum Line {
    Horizontal = 8,
    Vertical =  4,
    UpDiagonal =  2,
    DownDiagonal =  1,
}

pub enum Directions {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest
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
                        Some(Piece{piece_type: PieceType::Rook, piece_color: PieceColor::Black}) => {
                            if from == (0,0) {
                                self.castle_avail[self.turn as usize].1 = false;
                            }
                            else if from == (0,7) {
                                self.castle_avail[self.turn as usize].0 = false;
                            }
                        },
                        Some(Piece{piece_type: PieceType::Rook, piece_color: PieceColor::White}) => {
                            if from == (7,0) {
                                self.castle_avail[self.turn as usize].1 = false;
                            }
                            else if from == (7,7) {
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
            match self.previous_states.get_mut(&current_state) {
                Some(count) => {
                    *count += 1;
                },
                None => {
                    self.previous_states.insert(current_state, 1);
                },
            }
            if self.turn == PieceColor::Black {
                self.full_moves += 1;
            }

            if reset_hm {
                self.half_moves = 0;
            }
            else {
                self.half_moves += 1;
            }

            self.turn = !self.turn;
        }

        result
    }

    pub fn is_valid_move(&self, chess_move: ChessMove) -> bool {
        let player_side = match self.turn {
            PieceColor::Black => 0,
            PieceColor::White => 7,
        };

        match chess_move {
            ChessMove::CastleKingside => { 
                self.castle_avail[self.turn as usize].0 &&
                self.grid[player_side][5].is_none() &&
                self.grid[player_side][6].is_none() &&
                self.get_threats((player_side, 4), self.turn).len() == 0 &&
                self.get_threats((player_side, 5), self.turn).len() == 0 &&
                self.get_threats((player_side, 6), self.turn).len() == 0
            },
            ChessMove::CastleQueenside => {
                self.castle_avail[self.turn as usize].1 &&
                self.grid[player_side][1].is_none() &&
                self.grid[player_side][2].is_none() &&
                self.grid[player_side][3].is_none() &&
                self.get_threats((player_side, 2), self.turn).len() == 0 &&
                self.get_threats((player_side, 3), self.turn).len() == 0 &&
                self.get_threats((player_side, 4), self.turn).len() == 0
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
                                        let move_dist = from.0.abs_diff(to.0) + from.1.abs_diff(to.1);
                                        self.cast_ray(from, dir).map_or(true, |(p, distance)| distance > move_dist || (distance == move_dist && p.piece_color == !self.turn))
                                    }
                                    else {
                                        let move_dist = from.0.abs_diff(to.0);
                                        from.0.abs_diff(to.0) == from.1.abs_diff(to.1) && self.cast_ray(from, dir).map_or(true, |(p, distance)| distance > move_dist || (distance == move_dist && p.piece_color == !self.turn))
                                    }
                                },
                                PieceType::Bishup => {
                                    let dir = ((to.0 as isize - from.0 as isize) / (from.0.abs_diff(to.0).max(1) as isize), (to.1 as isize - from.1 as isize) / (from.1.abs_diff(to.1).max(1) as isize));
                                    let move_dist = from.0.abs_diff(to.0);
                                    from.0.abs_diff(to.0) == from.1.abs_diff(to.1) && self.cast_ray(from, dir).map_or(true, |(p, distance)| distance > move_dist || (distance == move_dist && p.piece_color == !self.turn))
                                },
                                PieceType::Knight => {
                                    ((from.0.abs_diff(to.0) == 1 && from.1.abs_diff(to.1) == 2) || (from.0.abs_diff(to.0) == 2 && from.1.abs_diff(to.1) == 1)) && self.grid[to.0][to.1].map_or(true, |p| p.piece_color == !self.turn)
                                },
                                PieceType::Rook => {
                                    let dir = ((to.0 as isize - from.0 as isize) / (from.0.abs_diff(to.0).max(1) as isize), (to.1 as isize - from.1 as isize) / (from.1.abs_diff(to.1).max(1) as isize));
                                    let move_dist = from.0.abs_diff(to.0) + from.1.abs_diff(to.1);
                                    (dir.0 == 0 || dir.1 == 0) && self.cast_ray(from, dir).map_or(true, |(p, distance)| distance > move_dist || (distance == move_dist && p.piece_color == !self.turn))
                                },
                                PieceType::Pawn => {
                                    let pawn_slots = match self.turn {
                                        PieceColor::Black => (1,2,3,1,6),
                                        PieceColor::White => (6,5,4,-1,1),
                                    };

                                    if from.0 != pawn_slots.4 {
                                        if from.1 == to.1 {
                                            (from.0 == pawn_slots.0 && to.0 == pawn_slots.2 && self.grid[pawn_slots.1][from.1].is_none() && self.grid[pawn_slots.2][from.1].is_none()) ||
                                            (from.0 as isize + pawn_slots.3 == to.0 as isize && self.grid[to.0][to.1].is_none())
                                        }
                                        else {
                                            from.0 as isize + pawn_slots.3 == to.0 as isize && (self.en_passant.map_or(false, |sq| sq == to) || self.grid[to.0][to.1].map_or(true, |p| p.piece_color == !self.turn))
                                        }
                                    }
                                    else {
                                        false
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
            ChessMove::PawnPromote(from, to, promotion) => {
                // TODO find a better way to convert between directions for turns
                if from.0 < 8 || from.1 < 8 && to.0 < 8 || to.1 < 8 && // Pieces are within bounds of the checkerboard
                    from.0 == (7 - player_side) + 2 * (self.turn as usize) - 1 && // Must be moving from second to last row to last row
                    promotion != PieceType::King && promotion != PieceType::Pawn && // Can't promote to King or Pawn
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

    pub fn get_king_loc(&self, piece_color: PieceColor) -> Option<(usize, usize)> {
        for x in 0..8 {
            for y in 0..8 {
                if self.grid[y][x].map_or(false, |p| p.piece_color == piece_color && p.piece_type == PieceType::King) {
                    return Some((y, x));
                }
            }
        }

        None
    }

    // Determines whether the current player's king is in check
    pub fn has_check(&self) -> bool {
        if let Some(king_pos) = self.get_king_loc(self.turn) {
            self.get_threats(king_pos, self.turn).len() > 0
        }
        else {
            false
        }
    }

    pub fn get_threats(&self, loc: (usize, usize), cur_player: PieceColor) -> Vec<(usize, usize)> {
        let mut result = vec!();
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
                        result.push((knight_sq.0 as usize, knight_sq.1 as usize));
                    }
                }
            }
        }

        // Check Lines
        for dir in [(0,1),(0,-1),(1,0),(-1,0)] {
            match self.cast_ray(loc, dir) {
                Some((Piece{piece_color, piece_type}, distance)) => {
                    if piece_color != cur_player && ((distance == 1 && piece_type == PieceType::King) || piece_type == PieceType::Queen || piece_type == PieceType::Rook) {
                        result.push(((iloc.0 + (distance as isize) * dir.0) as usize, (iloc.1 + (distance as isize) * dir.1) as usize));
                    }
                },
                None => {},
            };
        }

        // Check Diagonals
        for dir in [(1,1),(1,-1),(-1,1),(-1,-1)] {
            let pawn_dir = match cur_player {
                PieceColor::Black => 1,
                PieceColor::White => -1,
            };

            match self.cast_ray(loc, dir) {
                Some((Piece{piece_color, piece_type}, distance)) => {
                    if piece_color != cur_player && ((distance == 1 && dir.0 == pawn_dir && piece_type == PieceType::Pawn) || [PieceType::Queen, PieceType::Bishup].contains(&piece_type)) {
                        result.push(((iloc.0 + (distance as isize) * dir.0) as usize, (iloc.1 + (distance as isize) * dir.1) as usize));
                    }
                },
                None => {},
            };
        }

        result
    }

    fn get_valid_moves(&self) -> Vec<ChessMove> {
        let mut result: Vec<ChessMove> = vec!();

        // Add castling options if possible
        if self.is_valid_move(ChessMove::CastleKingside) {
            result.push(ChessMove::CastleKingside);
        }

        if self.is_valid_move(ChessMove::CastleQueenside) {
            result.push(ChessMove::CastleQueenside);
        }

        // Get all pieces that have the king in check
        let king_threats = match self.get_king_loc(self.turn) {
            Some(king_position) => self.get_threats(king_position, self.turn),
            None => vec!(),
        };

        // Get the locations of all the current player's pieces
        let mut piece_locations = vec!();
        for y in 0 .. 8 {
            for x in 0 .. 8 {
                // Cannot move other player's pieces or from empty squares
                if self.grid[y][x].map_or(false, |Piece{piece_color, piece_type: _}| piece_color == self.turn) {
                    piece_locations.push((y, x));
                }
            }
        }

        // Get the moves for all individual pieces
        for from in piece_locations {
            // Impossible to escape double check without moving king
            if king_threats.len() > 1 && self.grid[from.0][from.1].map_or(false, |Piece{piece_color: _, piece_type}| piece_type != PieceType::King) {
                continue;
            }

            // Get moves for different pieces
            match self.grid[from.0][from.1] {
                Some(Piece{piece_color: _, piece_type: PieceType::King}) => {
                    for y in from.0.saturating_sub(1) ..= from.0 + 1 {
                        for x in from.1.saturating_sub(1) ..= from.1 + 1 {
                            if y < 8 && x < 8 && !(x == 0 && y == 0) {
                                if self.get_threats((y, x), self.turn).len() == 0 {
                                    result.push(ChessMove::Move(from, (y, x)));
                                }
                            }
                        }
                    }
                },
                Some(Piece{piece_color: _, piece_type: PieceType::Queen}) => {
                    let pins = self.get_pins(from, self.turn);
                    let mut to_options: Vec<(usize, usize)> = vec!();
                    
                    for y_direction in [-1, 0, 1] {
                        for x_direction in [-1, 0, 1] {
                            
                        }
                    }

                    for to in to_options {
                        if king_threats.len() == 0 || king_threats[0] == to || (self.grid[to.0][to.1].is_none() && self.get_pins(to, self.turn) > 0) {
                            result.push(ChessMove::Move(from, to));
                        }
                    }
                },
                Some(Piece{piece_color: _, piece_type: PieceType::Bishup}) => {

                },
                Some(Piece{piece_color: _, piece_type: PieceType::Knight}) => {

                },
                Some(Piece{piece_color: _, piece_type: PieceType::Rook}) => {

                },
                Some(Piece{piece_color: _, piece_type: PieceType::Pawn}) => {
                    let pins = self.get_pins(from, self.turn);
                    if pins & 0b1000 == 0 {
                        let (to_row, start_row, double_move_row, en_passant_capture_row) = match self.turn {
                            PieceColor::Black => (from.0 + 1, 6, 4, 3),
                            PieceColor::White => (from.0 - 1, 1, 3, 4),
                        };

                        let mut to_options: Vec<(usize, usize)> = vec!();
                        
                        // Push Pawn
                        if pins & (0x0011) == 0 && self.grid[to_row][from.1].is_none() {
                            to_options.push((to_row, from.1));

                            // Double Move
                            if from.0 == start_row && self.grid[double_move_row][from.1].is_none() {
                                to_options.push((double_move_row, from.1));
                            }
                        }
                        
                        // Capture Left
                        if from.1 != 0 && pins & 0b0110 == 0 {
                            let to = (to_row, from.1 - 1);

                            if self.en_passant.map_or(false, |en_passant_square| en_passant_square == to) && self.get_pins((en_passant_capture_row, to.1), self.turn) == 0 {
                                to_options.push(to);
                            } else if self.grid[to.0][to.1].map_or(false, |Piece{piece_type:_, piece_color: captured_piece_color}| captured_piece_color != self.turn) {
                                
                                if king_threats.get(0).map_or(true, |threat_position| threat_position == &to) {
                                    to_options.push(to);
                                }
                            }
                        }

                        // Capture Right
                        if from.1 != 7 && pins & 0b0101 == 0 {
                            
                        }

                        for to in to_options {
                            if from.0 == 1 {
                                for promotion_type in [PieceType::Queen, PieceType::Bishup, PieceType::Knight, PieceType::Rook] {
                                    result.push(ChessMove::PawnPromote(from, to, promotion_type));
                                }
                            }
                            else {
                                result.push(ChessMove::Move(from, to));
                            }
                        }
                    }
                },
                None => {},
            }
        }

        result
    }

    fn get_piece_moves(&self, directions: Vec<(isize, isize)>) {
        for dir in directions {

        }
        if y_direction == 0 && x_direction == 0 {
            continue;
        }

        let dir = (y_direction, x_direction);

        // Horizontal moves
        if dir.0 == 0 && pins & 0b0111 > 0 {
            continue;
        }

        // Vertical moves
        if dir.1 == 0 && pins & 0b1011 > 0 {
            continue;
        }

        // UpDiagonal moves
        if dir.0 == -dir.1 && pins & 0b1101 > 0 {
            continue;
        }

        // DownDiagonal moves
        if dir.0 == dir.1 && pins & 0b1110 > 0 {
            continue;
        }

        let mut ito = (from.0 as isize + dir.0, from.1 as isize + dir.1);
        while ito.0 >= 0 && ito.0 < 8 && ito.1 >= 0 && ito.1 < 8 {
            let to = (ito.0 as usize, ito.1 as usize);

            match self.grid[to.0][to.1] {
                Some(Piece{piece_type: _, piece_color}) => {
                    if piece_color != self.turn {
                        to_options.push(to);
                    }
                    break;
                },
                None => {
                    to_options.push(to);
                },
            }

            ito = (ito.0 + dir.0, ito.1 + dir.1);
        }
    }

    fn get_pins(&self, loc: (usize, usize), piece_color: PieceColor) -> u8 {
        let mut result = 0u8;
        let horizontal = (self.cast_ray(loc, (0, -1)), self.cast_ray(loc, (0, 1)));
        let vertical = (self.cast_ray(loc, (-1, 0)), self.cast_ray(loc, (1, 0)));
        let up_diagonal = (self.cast_ray(loc, (1, -1)), self.cast_ray(loc, (-1, 1)));
        let down_diagonal = (self.cast_ray(loc, (-1, -1)), self.cast_ray(loc, (1, 1)));

        for (index, line) in [horizontal, vertical, up_diagonal, down_diagonal].iter().enumerate() {
            let threat_piece_types = if index < 2 {
                [PieceType::Queen, PieceType::Rook]
            }
            else {
                [PieceType::Queen, PieceType::Bishup]
            };

            result <<= 1;
            result |= match line {
                (Some((p1, _)), Some((p2, _))) => {
                    if p1.piece_color != p2.piece_color && (
                        (p1 == &Piece{piece_type: PieceType::King, piece_color} && threat_piece_types.contains(&p2.piece_type)) ||
                        (p2 == &Piece{piece_type: PieceType::King, piece_color} && threat_piece_types.contains(&p1.piece_type))) {
                        1
                    }
                    else {
                        0
                    }
                },
                _ => 0,
            };
        }

        result
    }

    // Send a ray out in a particular direction return the first piece encountered and return the distance to it
    fn cast_ray(&self, loc: (usize, usize), dir: (isize, isize)) -> Option<(Piece, usize)> {
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

    pub fn encode_squares(&self) -> EncodedBoard {
        let flags = (((self.castle_avail[0].0 as u8) | (self.castle_avail[0].1 as u8) | (self.castle_avail[1].0 as u8) |  (self.castle_avail[1].1 as u8)) << 4) |
                    self.en_passant.map_or(0, |(_, x)| (x as u8) | 8);

        let mut board: [u64; 4] = [0; 4];

        for y in 0..8 {
            for x in 0..8 {
                board[y / 2] <<= 4;
                board[y / 2] |= self.grid[y][x].map_or(0, |Piece{piece_color, piece_type}| (piece_color as u64) << 7 | (piece_type as u64) + 1 );
            }
        }

        EncodedBoard{grid: board, flags, turn: self.turn}
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