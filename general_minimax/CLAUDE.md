# general_minimax

## Transposition table

`TTable` (`src/player/transposition_table.rs`) holds about `1 << GameState::TT_BITS` entries
in buckets of 3 slots, or `1 << bits` through `alphabeta_tt_sized`. Entries keep their full
hash. A new position evicts the slot worth least, where worth is depth minus age in plies
(`GameState::ply`). A bucket is written on its first store, with one bit per bucket marking the
written ones, so a new table costs a few µs even at 22 bits; writing every slot took 69 µs at
18 bits. `alphabeta_tt` builds one table per player, kept for the whole game.

It replaced an unbounded `HashMap`. Against `randy` it beats the HashMap in Connect 4 and
trails it by 15-20% in five in a row. Between two table players it trails by 5% in Connect 4
and 12-18% in five in a row, until a game is long enough for the HashMap to outgrow memory
(see `games/five_in_row/CLAUDE.md`). It is kept for that bound and for the ideas below that
need a fixed size.

Open questions:
- Why the table is slower per node than the HashMap once it holds the search. A guess: a
  hashbrown miss reads only its compact control bytes, while every probe here reads a
  bucket's line somewhere in megabytes, pushing the board out of cache.
- Is aging still worth it with 3 slots? The weight of 1 per ply was tuned with 2 slots at
  16 bits, and 3 slots without aging was never measured.
- Table size. The best size depends on search depth and the machine, which is why engines take
  a memory budget (Stockfish's `Hash`, Gomocup's `max_memory`) and watch how full the table
  gets. Here each game picks its `TT_BITS`.

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
(54 ms). Details come from reading Stockfish's `tt.cpp` and change between versions. Both
Stockfish and Rapfi (a gomoku engine, `dhbloo/rapfi`) refresh an entry's generation when a
probe finds it; this table doesn't.

## Ideas for speeding up the search

- Stockfish's same-position rule: keep a deeper old result instead of always overwriting the
  position's own slot. Rapfi overwrites only with an exact bound or a depth at most 2 plies
  shallower.
- Refresh an entry's generation when a probe finds it, as Stockfish and Rapfi do.
- Rapfi's layout: 12-byte entries with a 32-bit key, 5 to a 64-byte bucket. That holds about
  twice as many positions per MB as 24-byte slots, and a probe reads one cache line.
- A compact tag per slot in its own array, like hashbrown's control bytes, so a miss reads a
  byte instead of a bucket.
- Count the `written` bits after a game, a `hashfull`-like number to size the table against.
- Size the table from a memory budget instead of `TT_BITS`, as engines do, so it can follow
  the machine and the time per move.
- Lazy SMP: threads search the same root and share one table without a lock, catching torn
  writes by storing the key XORed with the entry.
- Prefetch the child's bucket right after `make_move`. Little to gain while the search probes
  right after the move, with no work in between to hide the wait.

Tried without a clear gain: 64-byte buckets of 20-byte slots with 32-bit keys.
