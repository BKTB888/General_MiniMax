#[cfg(feature = "bench")]
use criterion::{BenchmarkGroup, measurement::WallTime};

#[cfg(feature = "bench")]
use crate::player::evals::Evaluation;
use crate::{
    player::players::{creator_from_seed, randy},
    state::GameState,
};

/// The position after `plies` seeded random moves from the start, stopping short of any move
/// that would end the game. The same `seed` and `plies` always give the same position.
pub fn position<S: GameState + 'static>(seed: u64, plies: u32) -> S {
    let mut player = creator_from_seed(seed, randy)(0);
    let mut state = S::default();
    for _ in 0..plies {
        let choice = player(&state);
        state.make_move(choice);
        if state.get_result().is_some() {
            state.undo();
            break;
        }
    }
    state
}

/// The positions `depth` moves from `state`, counted as perft usually is: a finished game
/// counts only at `depth` 0, and the last moves are counted rather than made.
pub fn perft<S: GameState>(state: &mut S, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    if state.get_result().is_some() {
        return 0;
    }
    let moves = state.candidate_moves();
    if depth == 1 {
        return moves.len() as u64;
    }
    let mut count = 0;
    for choice in moves {
        state.make_move(choice);
        count += perft(state, depth - 1);
        state.undo();
    }
    count
}

/// Adds one bench to `group` per operation a search repeats at every node, all on `state`.
/// `state` must not be finished, since `make_move_undo` plays its first candidate move.
#[cfg(feature = "bench")]
pub fn node_ops<S: GameState>(
    group: &mut BenchmarkGroup<WallTime>,
    mut state: S,
    eval: impl Evaluation<S>,
) {
    let choice = state.candidate_moves()[0];

    group.bench_function("candidate_moves", |b| b.iter(|| state.candidate_moves()));
    group.bench_function("get_result", |b| b.iter(|| state.get_result()));
    // Includes dropping the clone.
    group.bench_function("clone", |b| b.iter(|| state.clone()));
    group.bench_function("eval", |b| b.iter(|| eval(&state)));
    // Undoing restores `state`, so every iteration plays the same move.
    group.bench_function("make_move_undo", |b| {
        b.iter(|| {
            state.make_move(choice);
            state.undo();
        })
    });
}
