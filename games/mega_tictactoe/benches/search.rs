use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use general_minimax::{
    player::search::{ABSearch, Search, alphabeta, alphabeta_tt, minimax},
    utils::position,
};
use mega_tictactoe::{evaluation::eval_kinrow, state::KInARowState};

type FiveInRow = KInARowState<5>;

/// One root search at a fixed depth: alphabeta with and without the transposition table, and
/// minimax, which runs on rayon so its time depends on free cores.
fn search(c: &mut Criterion) {
    let mut state: FiveInRow = position(0, 8);
    let plain = alphabeta(eval_kinrow);
    let tt = alphabeta_tt(eval_kinrow);

    let mut group = c.benchmark_group("five_in_row");
    for depth in [1, 2] {
        // `find_best` undoes its moves, so `state` is the same position every iteration.
        group.bench_function(BenchmarkId::new("alphabeta", depth), |b| {
            b.iter(|| plain.find_best(&mut state, depth))
        });
        group.bench_function(BenchmarkId::new("alphabeta_tt", depth), |b| {
            b.iter(|| tt.find_best(&mut state, depth))
        });
        let mut mm = minimax(eval_kinrow).to_player(depth);
        group.bench_function(BenchmarkId::new("minimax", depth), |b| {
            b.iter(|| mm(&state))
        });
    }
    group.finish();
}

criterion_group!(benches, search);
criterion_main!(benches);
