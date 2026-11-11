use criterion::{Criterion, criterion_group, criterion_main};
use general_minimax::{
    state::GameState,
    utils::{node_ops, position},
};
use hive::{eval, state::HiveState};

/// The per-node operations every search repeats, with and without the expansions.
fn ops(c: &mut Criterion) {
    set::<true, true, true>(c, "hive_node_ops");
    set::<false, false, false>(c, "hive_core_node_ops");
}

fn set<const MOSQUITO: bool, const LADYBUG: bool, const PILLBUG: bool>(
    c: &mut Criterion,
    name: &str,
) {
    // Both queens are down by then, and most moves are movements.
    let state: HiveState<MOSQUITO, LADYBUG, PILLBUG> = position(0, 20);

    let mut group = c.benchmark_group(name);
    node_ops(&mut group, state.clone(), eval);
    group.bench_function("hash", |b| b.iter(|| state.hash()));
    group.finish();
}

criterion_group!(benches, ops);
criterion_main!(benches);
