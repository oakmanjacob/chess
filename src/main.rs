use crate::chess_game::board::Board;

pub mod chess_game;

fn main() {
    let mut board: Board = Board::new();
    board.print();

    loop {
        let mut line = String::new();
        println!("Enter a valid move :");
        let _ = std::io::stdin().read_line(&mut line).unwrap();
        
        match board.make_move(line.trim_end()) {
            true => println!("This is a valid move"),
            false => println!("Can't make this move"),
        }
        board.print();
    }
}
