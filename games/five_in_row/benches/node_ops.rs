use criterion::{Criterion, criterion_group, criterion_main};
use five_in_row::{eval, state::FiveInRowState};
use general_minimax::{
    state::GameState,
    utils::{node_ops, position},
};

/// The per-node operations every search repeats.
fn ops(c: &mut Criterion) {
    let state: FiveInRowState = position(0, 8);

    let mut group = c.benchmark_group("five_in_row_node_ops");
    node_ops(&mut group, state.clone(), eval);
    group.bench_function("hash", |b| b.iter(|| state.hash()));
    group.finish();
}

criterion_group!(benches, ops);
criterion_main!(benches);
