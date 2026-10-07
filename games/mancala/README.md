# Mancala
A Rust implementation of the mancala board game's very specific rule set. This rule set is only used in my family,
so it's not going to be much use to anyone but them. However, I will tell the rules here.

Mancala: https://en.wikipedia.org/wiki/Mancala

## Rules

### Board
![Board](https://verveculture.com/cdn/shop/files/board_games_web-18.jpg?v=1733425571)
The basic setup is like this. 6 'balls' in each hole except in the 2 big ones which I call trash, because they are not going to be used throughout the game.
The game is played by 2 players and each holds 1 half of the board, divided by the metal in the picture.

### Turns
The players decide who comes first and go one by one.
- The current player selects a hole (other than trash), and takes all the balls from it.
- The player places the balls counter-clockwise from their hand 1 by 1 into each hole (including trash) until their hand is empty.
- If the last ball was placed in a trash or in an empty hole, then the turn is finished.
- Otherwise the player takes every ball from the hole (including the one that should have been placed here) and starts the process again.

### Winning
The game ends when a player has no balls in their territory (other than trash). This player wins.
