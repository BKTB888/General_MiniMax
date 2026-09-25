use std::io;

use rand::{
    Rng, SeedableRng,
    prelude::{IndexedRandom, StdRng},
};

use crate::state::GameState;

pub trait PlayerCreator<S: GameState> = Sync + Fn(u32) -> Box<dyn Player<S>>;

pub trait RngPlayer<S: GameState, P: Player<S>> = Sync + Fn(StdRng) -> P;

pub trait Player<S: GameState> = FnMut(&S) -> S::Choice;

pub fn human<S: GameState>(state: &S) -> S::Choice {
    loop {
        println!("{state}");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        if let Ok(choice) = input.trim().parse::<S::Choice>()
            && state.is_valid(choice)
        {
            return choice;
        }
    }
}

/// A player that plays uniformly random candidate moves drawn from `rng`.
pub fn randy<S: GameState>(mut rng: impl Rng) -> impl Player<S> {
    move |state: &S| *state.candidate_moves().choose(&mut rng).unwrap()
}

/// Makes the player for game `game` by passing `rng_player` an RNG seeded from `seed` and
/// `game`. Each game plays differently, and the same `seed` and `game` always play the same moves.
pub fn creator_from_seed<S: GameState, P: Player<S> + 'static>(
    seed: u64,
    rng_player: impl RngPlayer<S, P>,
) -> impl PlayerCreator<S> {
    move |game| Box::new(rng_player(StdRng::seed_from_u64(game_seed(seed, game))))
}

/// Distinct for each `(seed, game)` pair while `seed` stays below 2³², so no two games or
/// seeds share a random sequence.
fn game_seed(seed: u64, game: u32) -> u64 {
    seed.rotate_left(32) ^ game as u64
}
