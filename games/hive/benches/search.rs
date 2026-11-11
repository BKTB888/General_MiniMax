use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use general_minimax::{
    player::search::{ABSearch, alphabeta, alphabeta_tt},
    utils::position,
};
use hive::{eval, state::HiveState};

/// One root search at a fixed depth, with and without the transposition table.
fn search(c: &mut Criterion) {
    set::<true, true, true>(c, "hive", false);
    set::<false, false, false>(c, "hive_core", false);
}

/// Root searches deeper than `search` goes, with the table only, since plain alphabeta would
/// take far longer.
fn deep_search(c: &mut Criterion) {
    set::<true, true, true>(c, "hive", true);
    set::<false, false, false>(c, "hive_core", true);
}

/// `deep` runs depths 3 and 4 with the table, else 1 and 2 with every search.
fn set<const MOSQUITO: bool, const LADYBUG: bool, const PILLBUG: bool>(
    c: &mut Criterion,
    name: &str,
    deep: bool,
) {
    // Both queens are down by then, and most moves are movements.
    let mut state: HiveState<MOSQUITO, LADYBUG, PILLBUG> = position(0, 20);
    let plain = alphabeta(eval);

    let mut group = c.benchmark_group(name);
    let depths = if deep {
        // The deepest searches are slow, and 10 is Criterion's minimum.
        group.sample_size(10);
        [3, 4]
    } else {
        [1, 2]
    };
    for depth in depths {
        // `find_best` undoes its moves, so `state` is the same position every iteration.
        if !deep {
            group.bench_function(BenchmarkId::new("alphabeta", depth), |b| {
                b.iter(|| plain.find_best(&mut state, depth, None))
            });
            group.bench_function(BenchmarkId::new("alphabeta_tt", depth), |b| {
                // A fresh table every iteration; a kept one would already hold this search.
                b.iter(|| alphabeta_tt(eval).find_best(&mut state, depth, None))
            });
        }
        group.bench_function(BenchmarkId::new("alphabeta_tt_search", depth), |b| {
            // As above, with building and dropping the table left out of the time.
            b.iter_batched_ref(
                || alphabeta_tt(eval),
                |search| search.find_best(&mut state, depth, None),
                BatchSize::PerIteration,
            )
        });
    }
    group.finish();
}

criterion_group!(benches, search, deep_search);
criterion_main!(benches);
