# connect_k

Connect 4 uses a 16-bit table (1.5 MiB). Its games never fill it: between two table players
(`alphabeta-tt:11` vs `alphabeta-tt:12`) the HashMap stays at 10 MB, and the table trails it
by 5%.

## Measuring

Against a seeded random player:

```
cargo run --release -p app -- connect4 --p1 randy:42 --p2 iterative-tt:10 --games 30   # average the printed Depth
cargo run --release -p app -- connect4 --p1 randy:42 --p2 alphabeta-tt:12 --games 10
cargo run --release -p app -- connect4 --p1 randy:42 --p2 alphabeta-tt:6 --games 1000
```

| | iterative-tt:10 depth | alphabeta-tt:12 | alphabeta-tt:6 |
|---|---|---|---|
| HashMap (`bit-line-map`) | 10.74 | 26.6 ms | 193 µs |
| Fixed table, 16 bits | 10.61 | 25.6 ms | 172 µs |
