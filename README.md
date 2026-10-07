# General MiniMax

Minimax and alpha-beta search in Rust, written once and generic over any game.
Each game only describes its rules; the search, the players and the play loop are shared.

## Games

| Game | Crate | CLI name |
|---|---|---|
| [Mancala](games/mancala/README.md) (my family's rules) | `mancala` | `mancala` |
| Connect K: any board size, any K, any number of players | `connect_k` | `connect4` (7×6, 2 players) |
| Five in a row, on bitboards | `five_in_row` | `five-in-row` |

## Playing

Needs nightly Rust; `rust-toolchain.toml` makes rustup pick it. From the workspace root:

```sh
cargo run --release -p app -- mancala                # you vs alphabeta:4
cargo run --release -p app -- connect4 --p1 alphabeta:8 --p2 randy --games 1000 --parallel
cargo run --release -p app -- five-in-row --p1 human --p2 iterative-tt:500 --print-board
```

`--games` plays several games and prints how often each result came up and the time per game, `--parallel` spreads them over all cores (not with a human player).
`--print-board` shows the board after every move, `--print-result` the final board of every game.

## Players

`--p1` and `--p2` (defaults: `human` and `alphabeta:4`) take:

| Player | Plays |
|---|---|
| `human` | moves typed in the terminal |
| `randy[:<seed>]` | random moves, reproducible when seeded |
| `minimax:<depth>` | full search without pruning, spread over all cores |
| `alphabeta:<depth>` | alpha-beta search |
| `alphabeta-tt:<depth>` | alpha-beta with a transposition table |
| `iterative:<ms>` | iterative deepening with a time budget per move |
| `iterative-tt:<ms>` | iterative deepening with a transposition table |

Mancala has no position hash yet, so it can't use the `-tt` players.

## Add your own game

Make a crate under `games/`, implement [`GameState`](general_minimax/src/state.rs) for its state
and add it to `GameKind` in [`app/src/cli.rs`](app/src/cli.rs). Give it an evaluation, any
`Fn(&State) -> Score` that scores the position for the player to move, or use `stupid_eval` to
search without one. [`games/connect_k`](games/connect_k) is a small example.

## Layout

- [`general_minimax/`](general_minimax): the library: searches, transposition table, players, play loop
- [`games/`](games): one crate per game
- [`app/`](app): the command line

## Benchmarks

Each game has [Criterion](https://github.com/bheisler/criterion.rs) benches for single moves,
searches and whole games:

```sh
cargo bench -p connect_k --bench node_ops    # or search, full_game
```

`games/profile-bench.sh <package> <bench>` profiles one with [samply](https://github.com/mstange/samply).
