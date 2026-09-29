# General MiniMax

Game-tree search in Rust. `general_minimax` is the library: minimax, alpha-beta with and
without a transposition table, players and the play loop. Each game is a crate under `games/`
(`connect_k`, `five_in_row`, `gomoku`, `infinite_connect4`, `mancala`), and `app` is the command
line that plays them.

Notes live next to what they describe:
- `general_minimax/CLAUDE.md`: the transposition table, its aging, and ideas for speeding up
  the search.
- `games/five_in_row/CLAUDE.md`, `games/connect_k/CLAUDE.md`, `games/mancala/CLAUDE.md`: each
  game's table size and measurements.

## Measuring

Compare builds in scratch copies of the tree, one run at a time. Games are deterministic, so
matching `--print-result=true` output shows two builds did the same work.

Each game has Criterion benches `node_ops`, `search` and `full_game`, run as
`cargo bench -p <game> --bench <bench>`. In `search`, `alphabeta_tt` includes building the
table and `alphabeta_tt_search` leaves it out.
