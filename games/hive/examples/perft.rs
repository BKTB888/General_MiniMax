//! Counts the positions at each depth from the start and times each depth, to compare with
//! nokamute's `perft`:
//!
//! ```text
//! cargo run --release -p hive --example perft -- mlp 5
//! cargo run --release -p hive --example perft -- core 5
//! ```

use std::time::Instant;

use general_minimax::{state::GameState, utils::perft};
use hive::state::HiveState;

fn main() {
    let mut args = std::env::args().skip(1);
    let set = args.next().unwrap_or_else(|| "mlp".to_owned());
    let max_depth = args.next().map_or(5, |depth| depth.parse().expect("a depth"));
    match set.as_str() {
        "mlp" => run(HiveState::<true, true, true>::default(), max_depth),
        "core" => run(HiveState::<false, false, false>::default(), max_depth),
        _ => panic!("the set is `mlp` or `core`"),
    }
}

fn run<S: GameState>(mut state: S, max_depth: u8) {
    for depth in 0..=max_depth {
        let start = Instant::now();
        let count = perft(&mut state, depth);
        let elapsed = start.elapsed();
        let rate = count as f64 / elapsed.as_secs_f64() / 1e6;
        println!("depth {depth}: {count} positions in {elapsed:.2?}, {rate:.1} M/s");
    }
}
