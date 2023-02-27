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



