use crate::evals::Evaluation;
use crate::search::Search;
use crate::state::GameState;
use rand::{
    SeedableRng,
    prelude::{IndexedRandom, StdRng},
    rng,
};
use std::io;
use std::time::Duration;

pub trait Player<S: GameState>: FnMut(&S) -> <S as GameState>::Choice {}
impl<S: GameState, F: FnMut(&S) -> <S as GameState>::Choice> Player<S> for F {}

pub fn human<S: GameState>(state: &S) -> S::Choice {
    loop {
        println!("{}", state);
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        if let Ok(choice) = input.trim().parse::<S::Choice>()
            && state.is_valid(choice.clone())
        {
            return choice;
        }
    }
}

pub fn randy<S: GameState>(state: &S) -> S::Choice {
    state.candidate_moves().choose(&mut rng()).unwrap().clone()
}

pub fn randys_from_seed<S: GameState>(seed: u64) -> impl Player<S> {
    let mut rng = StdRng::seed_from_u64(seed);
    move |state: &S| state.candidate_moves().choose(&mut rng).unwrap().clone()
}

pub fn iterative_deepening<S: GameState>(
    search: impl Search<S>,
    time_constraint: Duration,
) -> impl Evaluation<S> {
    move |board| {
        let start = std::time::Instant::now();
        let mut result = search(board, 0);
        let mut depth = 0;

        loop {
            if start.elapsed() >= time_constraint || result.is_terminal() {
                break;
            }

            depth += 1;
            result = search(board, depth);
        }

        println!("Depth: {depth}, Result: {result}");
        result
    }
}
