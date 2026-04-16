use std::io;
use dyn_clone::DynClone;
use rand::prelude::IndexedRandom;
use rand::rng;
use crate::result::GameResult;
use crate::state::GameState;

dyn_clone::clone_trait_object!(<S> Player<S> where S: GameState);

pub trait Player<S: GameState>: Fn(&S) -> <S as GameState>::Choice + DynClone {}
pub trait Evaluation<S: GameState> = Fn(&S) -> f32 + Clone + Copy;
impl<S: GameState, F: Fn(&S) -> <S as GameState>::Choice + DynClone> Player<S> for F {}

pub fn human<S: GameState>(state: &S) -> S::Choice {
    loop {
        println!("{}", state);
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        if let Ok(choice) = input.trim().parse::<S::Choice>() && state.is_valid(choice.clone()) {
            return choice
        }
    }
}

pub fn randy<S: GameState>(state: &S) -> S::Choice {
    state.candidate_moves()
        .choose(&mut rng())
        .unwrap()
        .clone()
}

pub fn minimax_creator<S: GameState>(depth: u8, eval: impl Evaluation<S>) -> impl Evaluation<S> {
    move |state: &S| {
        fn recursive<S: GameState>(state: &S, depth: u8, eval: impl Evaluation<S>) -> f32 {
            if let Some(result) = state.get_result() {
                return if let GameResult::Player(player) = result {
                    if player == state.get_current_player() { f32::INFINITY } else { f32::NEG_INFINITY }
                } else { 0.0 }
            }

            if depth == 0 {
                return eval(state);
            }

            -state.candidate_moves()
                .into_iter()
                .map(|game_move| {
                    let mut state = state.clone();
                    state.make_move(game_move);
                    state
                })
                .map(|state| recursive(&state, depth - 1, eval))
                .min_by(f32::total_cmp)
                .unwrap()
        }
        recursive(state, depth, eval)
    }
}

// ... existing code ...

pub fn alphabeta_creator<S: GameState>(depth: u8, eval: impl Evaluation<S>) -> impl Evaluation<S> {
    move |state: &S| {
        fn recursive<S: GameState>(state: &S, depth: u8, mut alpha: f32, beta: f32, eval: impl Evaluation<S>) -> f32 {
            if let Some(result) = state.get_result() {
                return if let GameResult::Player(player) = result {
                    if player == state.get_current_player() { f32::INFINITY } else { f32::NEG_INFINITY }
                } else { 0.0 }
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
        recursive(state, depth, f32::NEG_INFINITY, f32::INFINITY, eval)
    }
}

// ... existing code ...

pub fn stupid_eval<S: GameState>(_: &S) -> f32 { 0.0 }

pub fn ai_from_eval<S: GameState>(eval: impl Evaluation<S>) -> impl Player<S> {
    move |state: &S| {
        state.candidate_moves()
            .into_iter()
            .map(|game_move| {
                let mut state = state.clone();
                state.make_move(game_move);
                (game_move, eval(&state))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap().0
    }
}