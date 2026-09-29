use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use general_minimax::{
    player::search::{ABSearch, Search, alphabeta, alphabeta_tt, minimax},
    utils::position,
};
use mega_tictactoe::{eval, state::FiveInRowState};

/// One root search at a fixed depth: alphabeta with and without the transposition table, and
/// minimax, which runs on rayon so its time depends on free cores.
fn search(c: &mut Criterion) {
    let mut state: FiveInRowState = position(0, 8);
    let plain = alphabeta(eval);

    let mut group = c.benchmark_group("five_in_row");
    for depth in [1, 2] {
        // `find_best` undoes its moves, so `state` is the same position every iteration.
        group.bench_function(BenchmarkId::new("alphabeta", depth), |b| {
            b.iter(|| plain.find_best(&mut state, depth, None))
        });
        group.bench_function(BenchmarkId::new("alphabeta_tt", depth), |b| {
            // A fresh table every iteration; a kept one would already hold this search.
            b.iter(|| alphabeta_tt(eval).find_best(&mut state, depth, None))
        });
        group.bench_function(BenchmarkId::new("alphabeta_tt_search", depth), |b| {
            // As above, with building and dropping the table left out of the time.
            b.iter_batched_ref(
                || alphabeta_tt(eval),
                |search| search.find_best(&mut state, depth, None),
                BatchSize::PerIteration,
            )
        });
        let mut mm = minimax(eval).to_player(depth);
        group.bench_function(BenchmarkId::new("minimax", depth), |b| {
            b.iter(|| mm(&state))
        });
    }
    group.finish();
}

criterion_group!(benches, search);
criterion_main!(benches);
