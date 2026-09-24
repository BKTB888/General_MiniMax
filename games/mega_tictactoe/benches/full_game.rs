use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use general_minimax::{game::Game, player::players::randys_from_seed};
use mega_tictactoe::state::KInARowState;

type FiveInRow = KInARowState<5>;

const GAMES: u32 = 100;

/// Whole games between seeded random players, from the empty board to a result.
fn full_game(c: &mut Criterion) {
    let mut group = c.benchmark_group("five_in_row_full_game");
    group.bench_function(BenchmarkId::new("random", GAMES), |b| {
        // Fresh players each iteration, so every iteration plays the same games.
        b.iter(|| {
            Game::<FiveInRow>::new([Box::new(randys_from_seed(0)), Box::new(randys_from_seed(1))])
                .stats(GAMES, false)
        })
    });
    group.finish();
}

criterion_group!(benches, full_game);
criterion_main!(benches);
