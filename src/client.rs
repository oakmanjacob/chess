use thirtyfour::prelude::*;
use regex::*;
use lazy_static::lazy_static;
use crate::game::position::Position;

use super::game::board::Board;
use super::game::piece::{PieceType, PieceColor};
use super::game::chess_move::ChessMove;
use super::game::{Game, piece::*, tracker::*};

pub struct Client {
    board_pieces: Vec<(Piece, Position)>,
    driver: WebDriver,
}

impl Client {

    pub async fn new(client_url: &str) -> WebDriverResult<Client> {
        let caps = DesiredCapabilities::chrome();
        let driver = WebDriver::new("http://localhost:9515", caps).await?;
        driver.goto(client_url).await?;

        let game = Game::new();

        let mut client = Client{board_pieces: vec!(), driver};
        client.update_pieces_from_board(&game.board);
        Ok(client)
    }

    pub async fn get_player_color(&mut self) -> WebDriverResult<PieceColor> {
        // TODO: Handle unwrapping better
        let classes = self.driver.find(By::Css("chess-board.board")).await?.class_name().await?.expect("Could not locate board element!");

        if classes.contains("flipped") {
            println!("Playing as Black!");
            Ok(PieceColor::Black)
        }
        else {
            println!("Playing as White!");
            Ok(PieceColor::White)
        }
    }

    pub async fn update_pieces_from_board(&mut self, board: &Board) {
        self.board_pieces = vec!();
        for row in 0usize..=7usize {
            for column in 0usize..=7usize {
                if let Some(piece) = board.get(&Position::encode(row, column)) {
                    self.board_pieces.push((*piece, Position::encode(row, column)));
                }
            }
        }
    }

    pub async fn update_board(&mut self, player_color: &PieceColor) -> WebDriverResult<Option<ChessMove>> {
        // <chess-board class="board" id="board-single">
        // contains div with class piece
        // piece type defined by class (w|b)(p|n|b|r|q|k)
        // piece square defined by class square-(column)(row)
        lazy_static! {
            static ref SQUARE_REGEX: Regex = Regex::new(r"square-(?P<column>[1-8])(?P<row>[1-8])").unwrap();
            static ref PIECE_REGEX: Regex = Regex::new(r"(?P<color>[b|w])(?P<piece_type>[p|n|r|b|q|k])").unwrap();
        }

        //let elem_board = self.driver.find(By::ClassName("board")).await?;
        let pieces = self.driver.find_all(By::Css("chess-board.board div.piece")).await?;
        
        let mut handles = vec![];
        
        for piece in pieces.iter() {
            handles.push(piece.class_name());
        }

        let piece_class_names = futures::future::join_all(handles).await;

        let mut piece_positions: Vec<(Piece, Position)> = vec!();
        for class_names in piece_class_names.iter() {
            let mut piece: Option<Piece> = None;
            let mut position: Option<Position> = None;

            for name in class_names.as_ref().unwrap().as_ref().unwrap().split(' ') {
                if let Some(captures) = SQUARE_REGEX.captures(name) {
                    position = Some(Position::encode(captures["row"].parse::<usize>().unwrap() - 1, &captures["column"].parse::<usize>().unwrap() - 1));
                }
                else if let Some(captures) = PIECE_REGEX.captures(name) {
                    piece = Some(Piece{piece_type: PieceType::from_char(captures["piece_type"].chars().next().unwrap()).unwrap(), color: PieceColor::from_char(captures["color"].chars().next().unwrap()).unwrap()});
                }
            }

            if let Some(piece) = piece {
                if let Some(position) = position {
                    piece_positions.push((piece, position));
                }
                else {
                    println!("Could not find position for piece");
                }
            }
            else {
                println!("Could not find type for piece");
            }
        }

        let mut to_piece_positions: Vec<(Piece, Position)> = vec!();
        for (piece, position) in piece_positions.iter() {
            if !self.board_pieces.iter().any(|(old_piece, old_position)| old_piece == piece && old_position == position) {
                to_piece_positions.push((*piece, *position));
            }
        }

        let mut from_piece_positions: Vec<(Piece, Position)> = vec!();
        for (piece, position) in self.board_pieces.iter() {
            if !piece_positions.iter().any(|(new_piece, new_position)| new_piece == piece && new_position == position) {
                from_piece_positions.push((*piece, *position));
            }
        }

        self.board_pieces = piece_positions;

        if from_piece_positions.len() == 2 && to_piece_positions.len() == 2 {
            // Castling
            if let Some((_, from)) = from_piece_positions.iter().find(|(Piece{piece_type, color}, position)| piece_type == &PieceType::Rook) {
                let (from_row, from_column) = from.decode();
                
                if from_column == 7 {
                    return Ok(Some(ChessMove::CastleKingside));
                }
                else if from_column == 0 {
                    return Ok(Some(ChessMove::CastleQueenside));
                }
            }
        }
        else if let Some((Piece{piece_type: from_piece_type, color: _}, from)) = from_piece_positions.iter().find(|(Piece{piece_type, color}, position)| color == player_color) {
            if let Some((Piece{piece_type: to_piece_type, color: _}, to)) = to_piece_positions.iter().find(|(Piece{piece_type, color}, position)| color == player_color) {
                if from_piece_type != to_piece_type {
                    return Ok(Some(ChessMove::PawnPromote(*from, *to, *to_piece_type)))
                }
                else {
                    return Ok(Some(ChessMove::Move(*from, *to)))
                }
            }
        }

        Ok(None)
    }

    pub async fn make_move(&mut self, chess_move: &ChessMove, player_color: &PieceColor) -> WebDriverResult<()> {
        // <div class="promotion-window top" style="transform: translateX(700%);">
        // <i class="close-button icon-font-chess x"></i>
        // <div class="promotion-piece wb"></div>
        // <div class="promotion-piece wn"></div>
        // <div class="promotion-piece wq"></div>
        // <div class="promotion-piece wr"></div>
        // </div>

        let mut promotion: Option<PieceType> = None;
        let ((from_row, from_column), (to_row, to_column)) = match chess_move {
            ChessMove::CastleKingside => {
                match player_color {
                    PieceColor::Black => {
                        ((7, 4), (7, 6))
                    },
                    PieceColor::White => {
                        ((0, 4), (0, 6))
                    },
                }
            },
            ChessMove::CastleQueenside => {
                match player_color {
                    PieceColor::Black => {
                        ((7, 4), (7, 2))
                    },
                    PieceColor::White => {
                        ((0, 4), (0, 2))
                    },
                }
            },
            ChessMove::Move(from, to) => {
                (from.decode(), to.decode())
            },
            ChessMove::PawnPromote(from, to, piece_type) => {
                promotion = Some(*piece_type);
                (from.decode(), to.decode())
            },
        };

        let piece_square = self.driver.find(By::Css(format!("chess-board.board div.piece.square-{}{}", from_column + 1, from_row + 1).as_str())).await?;
        piece_square.click().await?;

        if let Ok(captured_piece) = self.driver.find(By::Css(format!("chess-board.board div.piece.square-{}{}", to_column + 1, to_row + 1).as_str())).await {
            captured_piece.click().await?;
        }
        else {
            self.driver.execute(format!("arguments[0].classList.remove(\"square-{}{}\");arguments[0].classList.add(\"square-{}{}\");", from_column + 1, from_row + 1, to_column + 1, to_row + 1).as_str(), vec![piece_square.to_json()?]).await?;
            piece_square.click().await?;
        }

        if let Some(piece_type) = promotion {
            self.driver.find(By::Css(format!(".promotion-window .promotion-piece.{}{}", player_color.to_char(), piece_type.to_char()).as_str())).await?.click().await?;
        }

        Ok(())
    }

    pub async fn get_current_turn(&self) -> WebDriverResult<PieceColor> {
        // timers have class clock-component
        // opponent_timer will have class clock-top
        // Self timer will have class clock-bottom
        // Black timer has class clock-black
        // White timer has class clock-white
        // Current turn clock has class clock-player-turn
        // timers contain span data-cy="clock-time"

        let current_player_timer = self.driver.find(By::Css(".clock-component.clock-player-turn")).await?.class_name().await?.expect("Got empty class object for current turn clock component");

        if current_player_timer.contains("clock-black")  {
            Ok(PieceColor::Black)
        }
        else {
            Ok(PieceColor::White)
        }
    }

    pub async fn is_game_over(&self) -> bool {
        self.driver.find(By::Css(".game-over-modal-content,.modal-game-over-component")).await.is_ok()
    }

    pub async fn disconnect(self) -> WebDriverResult<()> {
        self.driver.quit().await
    }
}
