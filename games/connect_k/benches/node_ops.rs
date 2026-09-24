#![feature(
    generic_const_args,
    min_generic_const_args,
    macroless_generic_const_args
)]
#![allow(incomplete_features)]

use connect_k::state::ConnectKState;
use criterion::{Criterion, criterion_group, criterion_main};
use general_minimax::{
    player::evals::stupid_eval,
    state::GameState,
    utils::{node_ops, position},
};

type Connect4 = ConnectKState<7, 6>;

/// The per-node operations every search repeats.
fn ops(c: &mut Criterion) {
    let state: Connect4 = position(0, 8);

    let mut group = c.benchmark_group("connect4_node_ops");
    node_ops(&mut group, state.clone(), stupid_eval);
    group.bench_function("hash", |b| b.iter(|| state.hash()));
    group.finish();
}

criterion_group!(benches, ops);
criterion_main!(benches);
