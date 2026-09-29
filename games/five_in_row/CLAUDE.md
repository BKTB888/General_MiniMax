# five_in_row

## Transposition table size

`TT_BITS` is 22. 20 is 6% faster on `tt:3 vs tt:4` but 63% slower on `tt:4 vs tt:5` and twice
as slow on `tt:4 vs tt:6`, and 24 was no faster than 22 on `tt:4 vs tt:5`. A 16-bit table was
1-2% faster against `randy`, but iterative search reached less deep.

## Where the time goes

In games against `randy` at depth 3, `Map::place` and `Map::undo` take about 83% of the
samples, the evaluation 6%, and the search with its table about 9%. The board is where the
search can get faster. Forcing `place` and `undo` inline made games 7% slower.

## Measuring

Games against `randy` stay short and the HashMap small, so the long games are between two
table players. The HashMap passes 2.9 GB on `tt:4 vs tt:6`, so run that one only with memory
to spare:

```
cargo run --release -p app -- five-in-row --p1 randy:42 --p2 alphabeta-tt:4 --games 10
cargo run --release -p app -- five-in-row --p1 alphabeta-tt:3 --p2 alphabeta-tt:4 --games 1
cargo run --release -p app -- five-in-row --p1 alphabeta-tt:4 --p2 alphabeta-tt:5 --games 1
cargo run --release -p app -- five-in-row --p1 alphabeta-tt:4 --p2 alphabeta-tt:6 --games 1
```

Time per game and peak memory, on an 8 GB M3:

| | HashMap (`bit-line-map`) | 18 bits | 20 bits | 22 bits | 24 bits |
|---|---|---|---|---|---|
| randy vs tt:4 | 75-76 ms | 78-80 ms | 82-84 ms | 87-91 ms | |
| tt:3 vs tt:4 | 644 ms, 103 MB | 975 ms, 14 MB | 716 ms, 50 MB | 760 ms, 195 MB | |
| tt:4 vs tt:5 | 14.5 s, 656 MB | | 26.5 s, 51 MB | 16.3 s, 195 MB | 16.3 s, 772 MB |
| tt:4 vs tt:6 | stopped past 2.9 GB | | 83.8 s, 50 MB | 38.5 s, 194 MB | |

`cargo bench -p five_in_row --bench search` has `alphabeta_tt_search` down to depth 6, where
the HashMap and the table still tie: one search doesn't fill a table. `--bench full_game`
plays `tt:3 vs tt:4` and `tt:4 vs tt:5`, which takes about 3 minutes per build.
