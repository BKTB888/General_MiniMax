use criterion::{Criterion, criterion_group, criterion_main};
use general_minimax::utils::{node_ops, position};
use mancala::{eval, state::MancalaState};

/// The per-node operations every search repeats. No `hash`: `MancalaState::hash` is unimplemented.
fn ops(c: &mut Criterion) {
    let state: MancalaState = position(0, 8);

    let mut group = c.benchmark_group("mancala_node_ops");
    node_ops(&mut group, state, eval);
    group.finish();
}

criterion_group!(benches, ops);
criterion_main!(benches);
