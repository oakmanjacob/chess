# Chess Bot: Shallow Beige
Jacob Oakman

## Overview
This will contain the basic pieces of a chess engine

### Game
The game is represented as a series of structs with the board as a two dimensional array of optional pieces.

### Client
The client is built on top of the async thirtyfour library for selenium in rust with tokio as the runtime.

### Engine
The engine is a minimax algorithm with alpha beta pruning. This has been further optimized to run in parallel using the rayon library.
Parallelization is achieved by sequentially searching two moves deep and then creating a parallel iteration to activate the sequential minimax for each of those lines.
This allows us to search 2 levels deeper when on an Azure vm with 96 cores.

## Build Instructions
Download Chrome 
Download ChromeDriver https://chromedriver.chromium.org/downloads

```
cargo build --release
```

```
.\target\release\chessbot.exe <Chess Game Url>
```

## Testing
Testing is done using perft which counts the number of possible board states several levels deep for each of the possible moves from both the start move and a particularly weird position and compares with the correct values.

```
cargo test
```

## Future Optimizations
### Bitboards
Utilizing bitboards would be a more efficient way to store piece locations because we could use bitwise operations to find threatened squares and move locations.
This would also simplify how we look for checkmate which would hopefully make the move generation function faster

### Hashing board states
This is important both for implementing draw by repitition and also being able to prune branches where we already have searched

### Iterative deepening
We can improve the efficiency of alpha beta pruning by running the operation at deeper and deeper depths and using the previous searches to predict lucrative paths. This approach also works well with engine memory.

### Engine memory / Thinking on the opponent's turn
It would be cool if we could have move generation running as a seperate process that builds a deeper and deeper tree as the game is running. That way we can continue thinking while waiting on the opponent.



