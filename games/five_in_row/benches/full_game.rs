use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use five_in_row::{eval, player::human_five_in_row, state::FiveInRowState};
use general_minimax::{
    game::play_multiple,
    player::{
        player_kind::PlayerKind,
        players::{creator_from_seed, randy},
    },
};

const GAMES: u32 = 100;

/// Whole games between seeded random players, from the empty board to a result.
fn full_game(c: &mut Criterion) {
    let start = FiveInRowState::default();

    let mut group = c.benchmark_group("five_in_row_full_game");
    group.bench_function(BenchmarkId::new("random", GAMES), |b| {
        b.iter(|| {
            play_multiple(
                &start,
                [
                    Box::new(creator_from_seed(0, randy)),
                    Box::new(creator_from_seed(1, randy)),
                ],
                GAMES,
                false,
                false,
                false,
            )
        })
    });
    group.finish();
}

/// Whole games between two table players, the second searching a ply deeper. Both defend, so the
/// games run long enough to fill the tables, which games against a random player never do.
fn table_game(c: &mut Criterion) {
    let start = FiveInRowState::default();

    let mut group = c.benchmark_group("five_in_row_full_game");
    // A game takes from about a second to half a minute.
    group.sample_size(10);
    for depths @ [first, second] in [[3, 4], [4, 5]] {
        let id = BenchmarkId::new("alphabeta_tt", format!("{first}_vs_{second}"));
        group.bench_function(id, |b| {
            b.iter(|| {
                play_multiple(
                    &start,
                    depths.map(|depth| {
                        PlayerKind::AlphabetaTT(depth).to_player_creator(eval, human_five_in_row)
                    }),
                    1,
                    false,
                    false,
                    false,
                )
            })
        });
    }
    group.finish();
}

criterion_group!(benches, full_game, table_game);
criterion_main!(benches);
