use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use general_minimax::{
    game::play_multiple,
    player::players::{creator_from_seed, randy},
};
use mancala::state::MancalaState;

const GAMES: u32 = 100;

/// Whole games between seeded random players, from the starting board to a result.
fn full_game(c: &mut Criterion) {
    let start = MancalaState::default();

    let mut group = c.benchmark_group("mancala_full_game");
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
