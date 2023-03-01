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
    phpsessid: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("Connecting to Chess.com");
    let mut client = Client::new(args.phpsessid).await.unwrap();

    loop {
        println!("Connected to Browser, Press Enter to Continue");
        let _ = std::io::stdin().read_line(&mut String::new()).unwrap();
        println!("Playing");
        run_client(&mut client).await;
        println!("Game Over!");
    }
}

async fn run_client(client: &mut Client) {
    let player_color = client.get_player_color().await.expect("Error! Could not get player color");
    let mut engine = Engine::new(Game::new(), player_color);
    client.update_pieces_from_board(&engine.game.board);

    let mut is_my_turn = player_color == PieceColor::White;
    let mut keep_playing = true;

    while keep_playing {
        keep_playing = if is_my_turn {
            is_my_turn = !is_my_turn;
            pick_and_make_move(client, &mut engine).await
        }
        else {
            is_my_turn = !is_my_turn;
            // TODO: Fix issue with getting bad moves
            wait_for_opponent_move(client, &mut engine).await
        }
    }
}

async fn pick_and_make_move(client: &mut Client, engine: &mut Engine) -> bool {
    if let Some(chess_move) = engine.get_best_move_parallel() {
        println!("{}", chess_move);
        while client.make_move(&chess_move, &engine.player).await.is_err() {
            println!("Client failed to make move")
        }
        engine.advance_move(chess_move);
        client.update_pieces_from_board(&engine.game.board);
    }
    else
    {
        println!("Checkmate!");
        return false;
    }

    true
}

async fn wait_for_opponent_move(client: &mut Client, engine: &mut Engine) -> bool {
    let mut opponent_move: Option<ChessMove> = None;
    let valid_moves = engine.game.get_moves();

    while !client.board_has_changed().await {
        // Wait for board to change
    }

    sleep(Duration::from_millis(100)).await;

    while opponent_move.is_none() {
        opponent_move = client.get_opponent_move(&!engine.player).await.ok().flatten();
        if engine.game.get_moves().is_empty() {
            return false;
        }

        if let Some(o_move) = opponent_move {
            if valid_moves.iter().any(|valid_move| &o_move == valid_move) {
                println!("{}", o_move);
                engine.advance_move(o_move);
            }
            else {
                println!("Bad move recognized {}! Trying again.", o_move);
                opponent_move = None;
                client.update_pieces_from_board(&engine.game.board);
            }
        }
    }

    true
}