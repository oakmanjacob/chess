mod game;
mod client;
mod engine;

use clap::Parser;
use client::Client;
// use tokio::time::{sleep, Duration};
use game::{Game, chess_move::ChessMove, piece::PieceColor};
use engine::Engine;

#[derive(Parser)]
struct Args {
    phpsessid: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("Connecting to Chess.com");
    let mut client = Client::new(args.phpsessid).await.unwrap();
    println!("Connected to Browser, Press Enter to Continue");
    let _ = std::io::stdin().read_line(&mut String::new()).unwrap();

    println!("Starting client");
    println!("{}", client.is_game_over().await);
    run_client(&mut client).await;
    
    println!("Finished, Press Enter to Exit");
    let _ = std::io::stdin().read_line(&mut String::new()).unwrap();
    client.disconnect().await.unwrap();
}

// Bot main client

// 0. Create driver and connect to chess.com
// 1. Wait for use input to start
// 2. Go into loop

// MyTurn
// Pick a move
// Make the move
// Verify move was successful
// OpponentTurn
// Wait for it to be my turn again



async fn run_client(client: &mut Client) {
    let player_color = client.get_player_color().await.expect("Error! Could not get player color");
    let mut engine = Engine::new(Game::new(), player_color);

    if player_color == PieceColor::Black {
        let mut opponent_move: Option<ChessMove> = None;
        while opponent_move.is_none() {
            opponent_move = client.update_board(&!player_color).await.unwrap();
        }
        println!("{}", opponent_move.unwrap());
        engine.advance_move(opponent_move.unwrap());
    }

    loop {
        if let Some(chess_move) = engine.get_best_move() {
            println!("{}", chess_move);
            client.make_move(&chess_move, &player_color).await;
            engine.advance_move(chess_move);
            client.update_pieces_from_board(&engine.game.board).await;
        }
        else
        {
            println!("Checkmate!");
            break;
        }

        
        let mut opponent_move: Option<ChessMove> = None;
        while opponent_move.is_none() {
            opponent_move = client.update_board(&!player_color).await.unwrap();
        }
        println!("{}", opponent_move.unwrap());
        engine.advance_move(opponent_move.unwrap());
    }
}