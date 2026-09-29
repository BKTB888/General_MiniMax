# General MiniMax

## In progress: transposition table (branch `bit-line-map-tt`)

`TTable` (`general_minimax/src/player/transposition_table.rs`) holds about
`1 << GameState::TT_BITS` entries in buckets of 3 slots. Entries keep their full hash. A new
position evicts the slot worth least, where worth is depth minus age in plies
(`GameState::ply`). A bucket is written on its first store, with one bit per bucket marking
the written ones, so a new table costs about 0.3 µs at 18 bits rather than the 69 µs of
writing every slot. `alphabeta_tt` builds one table per player, kept for the whole game.

It replaced an unbounded `HashMap`. Against `randy` it beats the HashMap in Connect 4 and
trails it by about 4% in five in a row. In a game between two table players it trails in
both: 5% in Connect 4, and 50% in five in a row, whose 18 bits evict too much (see
Measuring). It is kept mostly for the ideas below that need a fixed-size table.

Open questions:
- Is aging still worth it with 3 slots? The weight of 1 per ply was tuned with 2 slots at
  16 bits, and 3 slots without aging was never measured.
- Table size for five in a row. 20 bits brings the long game to within 5-9% of the HashMap
  but makes games against `randy` 5% slower; 22 bits is slower than 20 in both.
- Why the table is slower per node than the HashMap once it holds the search. A guess: a
  hashbrown miss reads only its compact control bytes, while every probe here reads a
  bucket's line somewhere in megabytes, pushing the board out of cache.
- Mancala has no hash yet (`MancalaState::hash` is `todo!()`), so it can't use the table.

## Aging, compared with Stockfish

Stockfish bumps a generation counter at the start of every search and stores it in each
entry (5 bits, wrapping after 32). Age is how many searches ago an entry was stored. A new
position evicts the entry in its 3-entry cluster with the lowest `depth - 8 * age`, so a
depth-12 entry from the last search (worth 4) outlasts a fresh depth-2 one. An entry for the
same position is only overwritten by an exact bound, a result at most about 3 plies
shallower, or when the old entry is from an earlier search.

Here the generation is `GameState::ply`, which grows every move rather than every search, and
worth is `depth - age` in plies. Stockfish's weight (about 4 per ply for us) made
`alphabeta-tt:12` take 47 ms instead of 25: in Connect 4 the last search's deep entries are
what the next search reuses most. Letting any newer entry beat any older one was worse still
(54 ms). Details come from reading Stockfish's `tt.cpp`; they change between versions, and
whether a probe hit refreshes an entry's generation was left unclear.

## Ideas

- Lazy SMP: threads search the same root and share one table without a lock, catching torn
  writes by storing the key XORed with the entry.
- Stockfish's same-position rule: keep a deeper old result instead of always overwriting the
  position's own slot.
- Prefetch the child's bucket right after `make_move`.
- A compact tag per slot in its own array, like hashbrown's control bytes, so a miss reads a
  byte instead of a bucket.

Tried without a clear gain: 64-byte buckets of 20-byte slots with 32-bit keys, and a 16-bit
table for five in a row (1-2% faster against `randy`, but iterative search reaches less deep).

## Measuring

Compare builds in scratch copies of the tree, one run at a time. Games are deterministic, so
matching `--print-result=true` output shows two builds did the same work. Connect 4 vs a
seeded random player:

```
cargo run --release -p app -- connect4 --p1 randy:42 --p2 iterative-tt:10 --games 30   # average the printed Depth
cargo run --release -p app -- connect4 --p1 randy:42 --p2 alphabeta-tt:12 --games 10
cargo run --release -p app -- connect4 --p1 randy:42 --p2 alphabeta-tt:6 --games 1000
```

| | iterative-tt:10 depth | alphabeta-tt:12 | alphabeta-tt:6 |
|---|---|---|---|
| HashMap (`bit-line-map`) | 10.74 | 26.6 ms | 193 µs |
| Fixed table, 16 bits | 10.61 | 25.6 ms | 172 µs |

Five in a row. Games against `randy` stay short and the HashMap small, so the long game is
one between two table players:

```
cargo run --release -p app -- five-in-row --p1 alphabeta-tt:3 --p2 alphabeta-tt:4 --games 1
cargo run --release -p app -- five-in-row --p1 randy:42 --p2 alphabeta-tt:4 --games 10
```

| | tt:3 vs tt:4, time and peak memory | randy vs tt:4, per game |
|---|---|---|
| HashMap (`bit-line-map`) | 0.70-0.77 s, 103 MB | 75-76 ms |
| Fixed table, 18 bits | 1.05-1.11 s, 14 MB | 78-80 ms |
| Fixed table, 20 bits | 0.76-0.81 s, 50 MB | 82-84 ms |
| Fixed table, 22 bits | 0.78-0.84 s, 195 MB | 87-91 ms |

Criterion benches: `cargo bench -p connect_k --bench search` and `-p mega_tictactoe`.
`alphabeta_tt` includes building the table, `alphabeta_tt_search` leaves it out.
