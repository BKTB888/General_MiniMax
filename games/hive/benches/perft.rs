use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use general_minimax::{
    state::GameState,
    utils::{perft, position},
};
use hive::state::HiveState;

/// Move generation with `make_move` and `undo`, and no evaluation. From the start it counts
/// what nokamute's `perft` does, so the times compare directly; the openings are mostly
/// placements, so the middle game covers the movements.
fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("hive_perft");
    count(&mut group, "start", HiveState::<true, true, true>::default(), 5);
    count(&mut group, "start_core", HiveState::<false, false, false>::default(), 6);
    count(&mut group, "middle", position::<HiveState<true, true, true>>(0, 20), 3);
    count(&mut group, "middle_core", position::<HiveState<false, false, false>>(0, 20), 3);
    group.finish();
}

fn count<S: GameState>(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
    name: &str,
    mut state: S,
    depth: u8,
) {
    // `perft` undoes its moves, so `state` is the same position every iteration.
    group.bench_function(BenchmarkId::new(name, depth), |b| {
        b.iter(|| perft(&mut state, depth))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
