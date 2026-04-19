use crate::result::GameResult;
use crate::state::GameState;
use rand::{
    SeedableRng,
    prelude::{IndexedRandom, StdRng},
    rng,
};
use std::io;
use std::time::Duration;

pub trait Evaluation<S: GameState>: Fn(&S) -> f32 {
    fn to_player(self) -> impl Player<S>
    where
        Self: Sized,
    {
        move |state: &S| {
            state
                .candidate_moves()
                .into_iter()
                .map(|game_move| {
                    let mut state = state.clone();
                    state.make_move(game_move);
                    (game_move, self(&state))
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap()
                .0
        }
    }
}

pub trait Search<S: GameState>: Fn(&S, u8) -> f32 {
    fn to_eval(self, depth: u8) -> impl Evaluation<S>
    where
        Self: Sized,
    {
        move |state| self(state, depth)
    }
}

pub trait Player<S: GameState>: FnMut(&S) -> <S as GameState>::Choice {}

impl<S: GameState, F: Fn(&S) -> f32> Evaluation<S> for F {}
impl<S: GameState, F: Fn(&S, u8) -> f32> Search<S> for F {}
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

pub fn alphabeta<S: GameState>(eval: impl Evaluation<S>) -> impl Search<S> {
    move |state, depth| {
        fn recursive<S: GameState>(
            state: &S,
            depth: u8,
            mut alpha: f32,
            beta: f32,
            eval: &impl Evaluation<S>,
        ) -> f32 {
            if let Some(result) = state.get_result() {
                return if let GameResult::Player(player) = result {
                    if player == state.get_current_player() {
                        f32::INFINITY
                    } else {
                        f32::NEG_INFINITY
                    }
                } else {
                    0.0
                };
            }

            if depth == 0 {
                return eval(state);
            }

            for game_move in state.candidate_moves() {
                let mut next = state.clone();
                next.make_move(game_move);
                let score = -recursive(&next, depth - 1, -beta, -alpha, eval);
                if score >= beta {
                    return beta; // beta cutoff
                }
                if score > alpha {
                    alpha = score;
                }
            }

            alpha
        }
        recursive(state, depth, f32::NEG_INFINITY, f32::INFINITY, &eval)
    }
}

pub fn stupid_eval<S: GameState>(_: &S) -> f32 {
    0.0
}

pub fn randys_from_seed<S: GameState>(seed: u64) -> impl Player<S> {
    let mut rng = StdRng::seed_from_u64(seed);
    move |state: &S| state.candidate_moves().choose(&mut rng).unwrap().clone()
}

pub fn iterative_deepening<S: GameState>(
    eval: impl Search<S>,
    think_duration: Duration,
) -> impl Evaluation<S> {
    todo!()
}
