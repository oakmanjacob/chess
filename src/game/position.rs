use super::piece::PieceColor;
use std::fmt;
use eyre::{eyre, Result};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Position {
    row: usize,
    column: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (row, column) = self.decode();
        let column_char = match column {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            _ => '-',
        };

        write!(f, "{}{}", column_char, row + 1)
    }
}

impl Position {
    pub fn encode(row: usize, column: usize) -> Position {
        Position {row, column}
    }

    pub fn encode_checked(row: isize, column: isize) -> Option<Position> {
        if !(0..=7).contains(&row) || !(0..=7).contains(&column){
            return None
        }

        Some(Position::encode(row as usize, column as usize))
    }

    // Returns (row, column)
    pub fn decode(&self) -> (usize, usize) {
        (self.row, self.column())
    }

    pub fn decode_isize(&self) -> (isize, isize) {
        (self.row as isize, self.column as isize)
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn from_str(loc: &str) -> Result<Position> {
        let bytes: Vec<char> = loc.chars().collect();

        if bytes.len() != 2 {
            return Err(eyre!("Invalid position string length"));
        }

        if bytes[0] < 'a' || bytes[0] > 'h' {
            return Err(eyre!("Invalid column indicator"));
        }

        if bytes[1] < '1' || bytes[1] > '8' {
            return Err(eyre!("Invalid row indicator: {}", bytes[1]));
        }

        let col = bytes[0] as usize - 'a' as usize;
        let row = bytes[1] as usize - '1' as usize;

        Ok(Position::encode(row, col))
    }

    pub fn forward(&self, player_color: &PieceColor) -> Option<Position> {
        match player_color {
            PieceColor::Black => if self.row != 0 {
                Some(Position{row: self.row - 1, column: self.column})
            }
            else {
                None
            },
            PieceColor::White => if self.row != 7 {
                Some(Position{row: self.row + 1, column: self.column})
            }
            else {
                None
            }
        }
    }

    pub fn backward(&self, player_color: &PieceColor) -> Option<Position> {
        match player_color {
            PieceColor::Black => if self.row != 7 {
                Some(Position{row: self.row + 1, column: self.column})
            }
            else {
                None
            },
            PieceColor::White => if self.row != 0 {
                Some(Position{row: self.row - 1, column: self.column})
            }
            else {
                None
            }
        }
    }
}