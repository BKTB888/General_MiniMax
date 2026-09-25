use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use general_minimax::{
    game::play_multiple,
    player::players::{creator_from_seed, randy},
};
use mega_tictactoe::state::KInARowState;

type FiveInRow = KInARowState<5>;

const GAMES: u32 = 100;

/// Whole games between seeded random players, from the empty board to a result.
fn full_game(c: &mut Criterion) {
    let start = FiveInRow::default();

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

criterion_group!(benches, full_game);
criterion_main!(benches);
