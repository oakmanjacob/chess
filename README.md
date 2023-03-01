# Chess Bot: Shallow Beige
Jacob Oakman

## Overview
This will contain the basic pieces of a chess engine

### Game

### Move Generation

### Client

### Engine

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
We can improve the efficiency of alpha beta pruning by 



