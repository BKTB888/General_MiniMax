# General MiniMax

## In progress: transposition table (branch `fixed-size-tt`)

`TTable` (`general_minimax/src/player/transposition_table.rs`) is a flat array of about
`1 << GameState::TT_BITS` entries in buckets of 3 slots. Entries keep their full hash. A new
position evicts the slot worth least, where worth is depth minus age in plies
(`GameState::ply`). `alphabeta_tt` builds one table per player, kept for the whole game.

It replaced an unbounded `HashMap`. On the workloads measured so far it only ties it (see
Measuring). It is kept mostly for the ideas below that need a fixed-size table.

Open questions:
- Is aging still worth it with 3 slots? The weight of 1 per ply was tuned with 2 slots at
  16 bits, and 3 slots without aging was never measured.
- Table size per game. Connect-K uses 16 bits. 18 bits matched the HashMap on
  `iterative-tt:10`, but its cost on short `alphabeta-tt:6` games is unmeasured. 20 bits and
  up got slower (cache misses, per-game allocation).
- Mancala has no hash yet (`MancalaState::hash` is `todo!()`), so it can't use the table.

## Ideas

- Fail-soft `alphabeta_tt`: return and store the best score seen instead of the window edge,
  so stored bounds are tighter and cut more often.
- Lazy SMP: threads search the same root and share one table without a lock, catching torn
  writes by storing the key XORed with the entry.
- Keep one table across games, letting aging push out old entries, so no game pays for
  building a table.
- Prefetch the child's bucket right after `make_move`.
- Smaller entries (a partial key check) so more fit per cache line.
- A long-search benchmark (seconds per move, five-in-a-row), where the HashMap's growth
  should hurt and a fixed table should win.

## Measuring

Compare builds in scratch copies of the tree, one run at a time. Connect 4 vs a seeded random
player:

```
cargo run --release -p app -- connect4 --p1 randy:42 --p2 iterative-tt:10 --games 30   # average the printed Depth
cargo run --release -p app -- connect4 --p1 randy:42 --p2 alphabeta-tt:12 --games 10
cargo run --release -p app -- connect4 --p1 randy:42 --p2 alphabeta-tt:6 --games 1000
```

| | iterative-tt:10 depth | alphabeta-tt:12 | alphabeta-tt:6 |
|---|---|---|---|
| HashMap (`c2eb5ac`) | 12.60 | ~27.5 ms | 237 µs |
| Fixed table, 16 bits | 12.48 | 26-28 ms | 219 µs |

Criterion benches: `cargo bench -p connect_k --bench search`.
