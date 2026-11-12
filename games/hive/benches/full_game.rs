// For `play_multiple`'s player array, whose length comes from `NUM_P`.
#![feature(gca_const_items, gca_min_const_items, gca_macroless_args)]
#![expect(incomplete_features)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use general_minimax::{
    game::play_multiple,
    player::{
        evals::Evaluation,
        player_kind::PlayerKind,
        players::{PlayerCreator, creator_from_seed, human, randy},
    },
    state::GameState,
};
use hive::{eval, state::HiveState};

type Mlp = HiveState<true, true, true>;
type Core = HiveState<false, false, false>;

const GAMES: u32 = 100;

/// Whole games between seeded random players, from the empty board to a result.
fn full_game(c: &mut Criterion) {
    random::<Mlp>(c, "hive_full_game");
    random::<Core>(c, "hive_core_full_game");
}

fn random<S: GameState + 'static>(c: &mut Criterion, name: &str) {
    let start = S::default();

    let mut group = c.benchmark_group(name);
    group.bench_function(BenchmarkId::new("random", GAMES), |b| {
        b.iter(|| {
            play_multiple(
                &start,
                // `from_fn` takes its length from `NUM_P`, which a literal can't match here.
                std::array::from_fn(|seat| {
                    Box::new(creator_from_seed(seat as u64, randy)) as Box<dyn PlayerCreator<S>>
                }),
                GAMES,
                false,
                false,
                false,
            )
        })
    });
    group.finish();
}

/// One whole game between two table players, the second searching a ply deeper. 3 against 4
/// takes about a minute a game, too long to repeat.
fn table_game(c: &mut Criterion) {
    table::<Mlp>(c, "hive_full_game", eval);
    table::<Core>(c, "hive_core_full_game", eval);
}

fn table<S: GameState + 'static>(
    c: &mut Criterion,
    name: &str,
    eval: impl Evaluation<S> + Sync + Copy + 'static,
) {
    let start = S::default();

    let mut group = c.benchmark_group(name);
    // A game takes from a third of a second to over one.
    group.sample_size(10);
    let depths @ [first, second] = [2, 3];
    let id = BenchmarkId::new("alphabeta_tt", format!("{first}_vs_{second}"));
    group.bench_function(id, |b| {
        b.iter(|| {
            play_multiple(
                &start,
                std::array::from_fn(|seat| {
                    PlayerKind::AlphabetaTT(depths[seat]).to_player_creator(eval, human)
                }),
                1,
                false,
                false,
                false,
            )
        })
    });
    group.finish();
}

criterion_group!(benches, full_game, table_game);
criterion_main!(benches);
