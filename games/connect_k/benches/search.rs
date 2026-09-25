#![feature(
    generic_const_args,
    min_generic_const_args,
    macroless_generic_const_args
)]
#![allow(incomplete_features)]

use connect_k::state::ConnectKState;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use general_minimax::{
    player::{
        evals::stupid_eval,
        search::{ABSearch, Search, alphabeta, alphabeta_tt, minimax},
    },
    utils::position,
};

type Connect4 = ConnectKState<7, 6>;

/// Root searches to a fixed depth: alphabeta with and without the transposition table, the
/// table one also deepened iteratively from depth 0, and minimax, which runs on rayon so its
/// time depends on free cores.
fn search(c: &mut Criterion) {
    let mut state: Connect4 = position(0, 8);
    let plain = alphabeta(stupid_eval);

    let mut group = c.benchmark_group("connect4");
    for depth in [4, 6] {
        // `find_best` undoes its moves, so `state` is the same position every iteration.
        group.bench_function(BenchmarkId::new("alphabeta", depth), |b| {
            b.iter(|| plain.find_best(&mut state, depth))
        });
        group.bench_function(BenchmarkId::new("alphabeta_tt", depth), |b| {
            // A fresh table every iteration; a kept one would already hold this search.
            b.iter(|| alphabeta_tt(stupid_eval).find_best(&mut state, depth))
        });
        group.bench_function(BenchmarkId::new("alphabeta_tt_iterative", depth), |b| {
            // A fresh table every iteration, kept across the depths like `with_iterative` does.
            b.iter(|| {
                let search = alphabeta_tt(stupid_eval);
                for d in 0..=depth {
                    search.find_best(&mut state, d);
                }
            })
        });
        let mut mm = minimax(stupid_eval).to_player(depth);
        group.bench_function(BenchmarkId::new("minimax", depth), |b| {
            b.iter(|| mm(&state))
        });
    }
    group.finish();
}

criterion_group!(benches, search);
criterion_main!(benches);
