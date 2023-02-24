mod game;
mod client;
//mod engine;

use clap::Parser;
use client::Client;
use tokio::time::{sleep, Duration};
use game::chess_move::ChessMove;
use game::position::Position;
use game::piece::PieceType;

#[derive(Parser)]
struct Args {
    game_url: String,
}

fn main() {
    let args = Args::parse();

    println!("Hello, world!");

    run_client(args.game_url);
}

#[tokio::main]
async fn run_client(game_url: String) {
    //let mut client = Client::new("https://www.chess.com/play/computer").await.unwrap();
    let mut client = Client::new(game_url.as_str()).await.unwrap();

    sleep(Duration::from_secs(30)).await;

    client.get_player_color().await.expect("Error! Could not get player color");
    client.update_board().await.unwrap();

    // e4 client.make_move(&ChessMove::Move(Position::encode(1, 4), Position::encode(3, 4))).await;
    // client.make_move(&ChessMove::PawnPromote(Position::encode(6, 6), Position::encode(7, 7), PieceType::Queen)).await;
    // client.make_move(&ChessMove::Move(Position::encode(1, 4), Position::encode(2, 5))).await;
    client.make_move(&ChessMove::CastleQueenside).await.expect("Error! Failed to castle queenside");


    sleep(Duration::from_secs(15)).await;

    client.disconnect().await.unwrap();
}