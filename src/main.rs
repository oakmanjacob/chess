mod game;
mod client;
mod engine;

use clap::Parser;
use client::Client;
use tokio::time::{sleep, Duration};
use game::{Game, chess_move::ChessMove, piece::PieceColor};
use engine::Engine;

#[derive(Parser)]
struct Args {
    game_url: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    println!("Connecting to ");
    let mut client = Client::new(args.game_url.as_str()).await.unwrap();
    println!("Connected to Browser, Press Enter to Continue");
    let _ = std::io::stdin().read_line(&mut String::new()).unwrap();

    println!("Starting client");
    println!("{}", client.is_game_over().await);
    run_client(&mut client).await;
    
    println!("Finished, Press Enter to Exit");
    let _ = std::io::stdin().read_line(&mut String::new()).unwrap();
    println!("{}", client.is_game_over().await);
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
    //let mut client = Client::new("https://www.chess.com/play/computer").await.unwrap();
    //let mut client = Client::new("https://www.chess.com/analysis").await.unwrap();
    //let mut client = Client::new(game_url.as_str()).await.unwrap();
    let player_color = client.get_player_color().await.expect("Error! Could not get player color");
    let mut engine = Engine::new(Game::new(), player_color);

    if player_color == PieceColor::Black {
        let mut opponent_move: Option<ChessMove> = None;
        while opponent_move.is_none() {
            opponent_move = client.update_board(&!player_color).await.unwrap();
        }
        println!("Opponent Played {}", opponent_move.unwrap());
        engine.advance_move(opponent_move.unwrap());
    }

    loop {
        if let Some(chess_move) = engine.get_best_move() {
            println!("We Played {}", chess_move);
            client.make_move(&chess_move, &player_color).await;
            client.update_board(&player_color).await;
            engine.advance_move(chess_move);
        }
        else
        {
            println!("Checkmate!");
            break;
        }

        
        let mut opponent_move: Option<ChessMove> = None;
        while opponent_move.is_none() {
            opponent_move = client.update_board(&!player_color).await.unwrap();
            sleep(Duration::from_secs(5));
        }
        println!("Opponent Played {}", opponent_move.unwrap());
        engine.advance_move(opponent_move.unwrap());
    }
    // client.make_move(&ChessMove::PawnPromote(Position::encode(6, 6), Position::encode(7, 7), PieceType::Queen)).await;
    // client.make_move(&ChessMove::Move(Position::encode(1, 4), Position::encode(2, 5))).await;
    //client.make_move(&ChessMove::CastleQueenside, &player_color).await.expect("Error! Failed to castle queenside");


}