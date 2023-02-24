use thirtyfour::prelude::*;
use regex::*;
use lazy_static::lazy_static;
use super::game::piece::{PieceType, PieceColor};
use super::game::chess_move::ChessMove;
use super::game::Game;
use tokio::time::{sleep, Duration};

pub struct Client {
    client_game: u64,
    client_url: String,
    self_timer: usize,
    opponent_timer: usize,
    player_color: PieceColor,
    driver: WebDriver,
}

pub enum ClientState {
    Init,
    PlayerTurn,
    OpponentTurn,
}

impl Client {

    pub async fn new(client_url: &str) -> WebDriverResult<Client> {
        let caps = DesiredCapabilities::chrome();
        let driver = WebDriver::new("http://localhost:9515", caps).await?;
        driver.goto(client_url).await?;
        // Get player color using flipped class on board
        let classes = driver.find(By::Css("chess-board.board")).await?.class_name().await.unwrap().unwrap();

        let player_color = if classes.contains("flipped") {
            println!("Playing as Black!");
            PieceColor::Black
        }
        else {
            println!("Playing as White!");
            PieceColor::White
        };

        Ok(Client{client_game: 0, client_url: client_url.to_owned(), self_timer: 60 * 5, opponent_timer: 60 * 5, player_color, driver})
    }

    pub async fn get_player_color(&mut self) -> WebDriverResult<()> {
        let classes = self.driver.find(By::Css("chess-board.board")).await?.class_name().await.unwrap().unwrap();

        self.player_color = if classes.contains("flipped") {
            println!("Playing as Black!");
            PieceColor::Black
        }
        else {
            println!("Playing as White!");
            PieceColor::White
        };

        Ok(())
    }

    pub async fn update_board(&mut self) -> WebDriverResult<()> {
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

        for class_names in piece_class_names.iter() {
            for name in class_names.as_ref().unwrap().as_ref().unwrap().split(' ') {
                if let Some(captures) = SQUARE_REGEX.captures(name) {
                    print!("{}, {}", &captures["column"], &captures["row"]);
                }
                else if let Some(captures) = PIECE_REGEX.captures(name) {
                    print!("{}, {}", &captures["color"], &captures["piece_type"]);
                }
            }
            println!();
        }

        Ok(())
    }

    pub async fn make_move(&mut self, chess_move: &ChessMove) -> WebDriverResult<()> {
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
                match self.player_color {
                    PieceColor::Black => {
                        ((7, 4), (7, 6))
                    },
                    PieceColor::White => {
                        ((0, 4), (0, 6))
                    },
                }
            },
            ChessMove::CastleQueenside => {
                match self.player_color {
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
            self.driver.find(By::Css(format!(".promotion-window .promotion-piece.{}{}", self.player_color.to_char(), piece_type.to_char()).as_str())).await?.click().await?;
        }

        Ok(())
    }

    pub async fn get_current_turn(&self) -> WebDriverResult<PieceColor> {
        let current_player_timer = self.driver.find(By::Css(".clock-component.clock-player-turn")).await?.class_name().await?.expect("Got empty class object for current turn clock component");

        if current_player_timer.contains("clock-bottom") {
            Ok(self.player_color)
        }
        else {
            Ok(!self.player_color)
        }
    }

    // pub async fn update_timers(&mut self) -> WebDriverResult<()> {
    //     // timers have class clock-component
    //     // opponent_timer will have class clock-top
    //     // Self timer will have class clock-bottom
    //     // Black timer has class clock-black
    //     // White timer has class clock-white
    //     // Current turn clock has class clock-player-turn
    //     // timers contain span data-cy="clock-time"
    //     let timer_top_handle = self.driver.find(By::Css(".clock-component.clock-top"));
    //     let timer_bottom_handle = self.driver.find(By::Css(".clock-component.clock-bottom"));
    //     let timer_top_values_handle = self.driver.find(By::Css(".clock-component.clock-top span[data-cy=\"clock-time\"]"));
    //     let timer_bottom_values_handle = self.driver.find(By::Css(".clock-component.clock-bottom span[data-cy=\"clock-time\"]"));

    //     let timer_top_handle = timer_top_handle.await?;
    //     let timer_top_class_handle = timer_top_handle.class_name();
    //     let timer_bottom_handle = timer_bottom_handle.await?;
    //     let timer_bottom_class_handle = timer_bottom_handle.class_name();
    //     let timer_top_values_handle = timer_top_values_handle.await?;
    //     let timer_top_values_text_handle = timer_top_values_handle.text();
    //     let timer_bottom_values_handle = timer_bottom_values_handle.await?;
    //     let timer_bottom_values_text_handle = timer_bottom_values_handle.text();

    //     let timer_top_classes = timer_top_class_handle.await?;
    //     let timer_bottom_classes = timer_bottom_class_handle.await?;
    //     let timer_top_values_text = timer_top_values_text_handle.await?;
    //     let timer_bottom_values_text = timer_bottom_values_text_handle.await?;

    //     println!();


    //     Ok(())
    // }

    pub fn send_message(&self, message: &str) {
        // input data-cy="chat-input-field"
        // max length 100
    }

    pub async fn disconnect(self) -> WebDriverResult<()> {
        self.driver.quit().await
    }
}
