use super::piece::*;
use super::position::*;
use std::cmp::{PartialEq, Eq};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board {
    grid: [[Option<Piece>; 8]; 8]
}

impl Board {

    pub fn get(&self, position: &Position) -> Option<&Piece> {
        let (row, column) = position.decode();
        self.grid[row][column].as_ref()
    }

    pub fn add_piece(&mut self, piece: Piece, position: &Position) -> Option<Piece> {
        let (row, column) = position.decode();
        self.grid[row][column].replace(piece)
    }

    pub fn remove_piece(&mut self, position: &Position) -> Option<Piece> {
        let (row, column) = position.decode();
        self.grid[row][column].take()
    }

    pub fn make_move(&mut self, from: &Position, to: &Position) -> Option<Piece> {
        let (from_row, from_column) = from.decode();
        let (to_row, to_column) = to.decode();

        // println!("Make move ({}, {}) ({}, {})", from_row, from_column, to_row, to_column);
        // println!("Move made {} {}", self.grid[from_row][from_column].is_none(), self.grid[to_row][to_column].is_some());
        let res = self.grid[from_row][from_column].take().and_then(|piece| self.grid[to_row][to_column].replace(piece));
        // println!("Move made {} {} {}", res.map_or("None".to_owned(), |piece| format!("{}", piece.to_char())), self.grid[from_row][from_column].is_none(), self.grid[to_row][to_column].is_some());
        res
    }

    pub fn test_move(&self, from: &Position, to: &Position, king_position: &Position, player_color: &PieceColor) -> bool {
        let mut next_board = *self;
        next_board.make_move(from, to);
        !next_board.has_check(king_position, player_color)
    }

    // TODO: Edit to exit even faster
    pub fn has_check(&self, position: &Position, player_color: &PieceColor) -> bool {
        // Check Knight Moves
        for threat_position in self.get_knight_move_positions(position, player_color, true) {
            if self.get(&threat_position).map_or(false, |&Piece{piece_type, color}| color != *player_color && piece_type == PieceType::Knight) {
                return true;
            }
        }
        
        let pawn_row = position.forward(player_color).row();

        // Check Diagonals
        for threat_position in self.get_bishup_move_positions(position, player_color, true) {
            let threat_row = threat_position.row();

            if self.get(&threat_position).map_or(false, |&Piece{piece_type, color}| color != *player_color && (piece_type == PieceType::Queen || piece_type == PieceType::Bishup || (piece_type == PieceType::Pawn && threat_row == pawn_row))) {
                return true;
            }
        }

        // Check Columns and Rows
        for threat_position in self.get_rook_move_positions(position, player_color, true) {
            if self.get(&threat_position).map_or(false, |&Piece{piece_type, color}| color != *player_color && (piece_type == PieceType::Queen || piece_type == PieceType::Rook)) {
                return true;
            }
        }

        false
    }

    pub fn get_threats(&self, position: &Position, player_color: &PieceColor) -> Vec<Position> {
        let mut threats = vec!();

        // Check Knight Moves
        for threat_position in self.get_knight_move_positions(position, player_color, true) {
            if self.get(&threat_position).map_or(false, |&Piece{piece_type, color}| color != *player_color && piece_type == PieceType::Knight) {
                threats.push(threat_position)
            }
        }
        
        let pawn_row = position.forward(player_color).row();

        // Check Diagonals
        for threat_position in self.get_bishup_move_positions(position, player_color, true) {
            let threat_row = threat_position.row();

            if self.get(&threat_position).map_or(false, |&Piece{piece_type, color}| color != *player_color && (piece_type == PieceType::Queen || piece_type == PieceType::Bishup || (piece_type == PieceType::Pawn && threat_row == pawn_row))) {
                threats.push(threat_position)
            }
        }

        // Check Columns and Rows
        for threat_position in self.get_rook_move_positions(position, player_color, true) {
            if self.get(&threat_position).map_or(false, |&Piece{piece_type, color}| color != *player_color && (piece_type == PieceType::Queen || piece_type == PieceType::Rook)) {
                threats.push(threat_position)
            }
        }

        threats
    }

    pub fn get_knight_move_positions(&self, position: &Position, player_color: &PieceColor, get_captures_only: bool) -> Vec<Position> {
        let (row, column) = position.decode_isize();
        let mut knight_positions = vec!();

        for (row_increment, column_increment) in [(-1,-2),(-1,2),(1,-2),(1,2),(-2,-1),(-2,1),(2,-1),(2,1)] {
            if let Some(knight_pos) = Position::encode_checked(row + row_increment, column + column_increment) {
                if self.get(&knight_pos).map_or(!get_captures_only, |&Piece{piece_type:_, color}| color != *player_color){
                    knight_positions.push(knight_pos);
                }
            }
        }

        knight_positions
    }

    pub fn get_rook_move_positions(&self, position: &Position, player_color: &PieceColor, get_captures_only: bool) -> Vec<Position> {
        let mut rook_moves = vec!();

        for increments in [(-1,0),(1,0),(0,-1),(0,1)] {
            self.add_positions_in_direction(position, increments, player_color, get_captures_only, &mut rook_moves);
        }

        rook_moves
    }

    pub fn get_bishup_move_positions(&self, position: &Position, player_color: &PieceColor, get_captures_only: bool) -> Vec<Position> {
        let mut bishup_moves = vec!();

        for increments in [(-1,-1),(-1,1),(1,-1),(1,1)] {
            self.add_positions_in_direction(position, increments, player_color, get_captures_only, &mut bishup_moves);
        }

        bishup_moves
    }

    fn add_positions_in_direction(&self, position: &Position, increments: (isize, isize), player_color: &PieceColor, get_captures_only: bool, moves: &mut Vec<Position>) {
        let (row, column) = position.decode_isize();
        let (mut search_row, mut search_column) = (row + increments.0, column + increments.1);

        while let Some(search_position) = Position::encode_checked(search_row, search_column) {
            if let Some(piece) = self.get(&search_position) {
                if piece.color != *player_color {
                    moves.push(search_position);
                }
                break;
            }
            if !get_captures_only {
                moves.push(search_position);
            }
            search_row += increments.0;
            search_column += increments.1;
        }
    }

    pub fn default() -> Board {
        Board {
            grid: Default::default()
        }
    }

    pub fn print(&self) {
        use colored::*;
        let mut toggle = false;
        for (row, index) in self.grid.iter().rev().zip((1..=8).rev()) {
            print!("{} ", index);
            for square in row.iter() {
                let value = format!(" {} ", match square {
                    Some(p) => p.to_char(),
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
