use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use general_minimax::{
    player::search::{ABSearch, Search, alphabeta, minimax},
    utils::position,
};
use mancala::{eval, state::MancalaState};

/// One root search at a fixed depth: alphabeta, and minimax, which runs on rayon so its time
/// depends on free cores. No `alphabeta_tt`: `MancalaState::hash` is unimplemented.
fn search(c: &mut Criterion) {
    let mut state: MancalaState = position(0, 8);
    let plain = alphabeta(eval);

    let mut group = c.benchmark_group("mancala");
    for depth in [6, 8] {
        // `find_best` undoes its moves, so `state` is the same position every iteration.
        group.bench_function(BenchmarkId::new("alphabeta", depth), |b| {
            b.iter(|| plain.find_best(&mut state, depth, None))
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
