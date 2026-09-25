use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use rayon::{iter::ParallelIterator, prelude::IntoParallelRefIterator};

use crate::{
    player::{
        evals::Evaluation,
        players::Player,
        search::EvalResult::{Loss, Win},
        transposition_table::{TTBound, TTable},
    },
    state::GameState,
};

pub trait Search<S: GameState>: Fn(&mut S, u8) -> EvalResult + Sync + Sized {
    fn to_eval(self, depth: u8) -> impl Evaluation<S>
    where
        Self: Send,
    {
        move |state| self(&mut state.clone(), depth)
    }
    fn to_player(self, depth: u8) -> impl Player<S> {
        move |state| {
            state
                .candidate_moves()
                .par_iter()
                .copied()
                .map_init(
                    || state.clone(), // one clone per worker
                    |state, game_move| {
                        state.make_move(game_move);
                        let result = self(state, depth);
                        state.undo();
                        (game_move, result)
                    },
                )
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap()
                .0
        }
    }
}
impl<S: GameState, F: Fn(&mut S, u8) -> EvalResult + Sync> Search<S> for F {}

pub trait ABSearch<S: GameState>: Fn(&mut S, u8, EvalResult, EvalResult) -> EvalResult {
    fn to_eval(self, depth: u8) -> impl Evaluation<S>
    where
        Self: Sized + Send + Sync,
    {
        move |state| self(&mut state.clone(), depth, Loss, Win)
    }

    fn to_player(self, depth: u8) -> impl Player<S>
    where
        Self: Sized,
    {
        move |state| self.find_best(&mut state.clone(), depth).0
    }

    fn find_best(&self, state: &mut S, depth: u8) -> (S::Choice, EvalResult) {
        let moves = state.candidate_moves();
        let mut alpha = Win;
        let mut alpha_move = moves[0];
        let beta = Loss;

        for game_move in moves {
            state.make_move(game_move);
            let score = self(state, depth, -beta, -alpha);
            state.undo();
            if score == beta {
                return (game_move, Win); // beta cutoff
            }
            if score < alpha {
                alpha = score;
                alpha_move = game_move;
            }
        }

        (alpha_move, -alpha)
    }

    fn with_iterative(self, duration: Duration) -> impl Player<S>
    where
        Self: Sized,
    {
        move |state| {
            let start = Instant::now();
            let mut depth = 0;
            let (mut game_move, mut result) = self.find_best(&mut state.clone(), depth);

            while start.elapsed() < duration && !result.is_terminal() {
                depth += 1;
                (game_move, result) = self.find_best(&mut state.clone(), depth);
            }

            println!("Depth: {depth}, Choice: {game_move}, Result: {result}");
            game_move
        }
    }
}
impl<S: GameState, F: Fn(&mut S, u8, EvalResult, EvalResult) -> EvalResult> ABSearch<S> for F {}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum EvalResult {
    Win,
    Loss,
    Draw,
    Score(f32),
}

pub fn alphabeta<S: GameState>(eval: impl Evaluation<S>) -> impl ABSearch<S> {
    fn recursive<S: GameState>(
        state: &mut S,
        depth: u8,
        mut alpha: EvalResult,
        beta: EvalResult,
        eval: &impl Evaluation<S>,
    ) -> EvalResult {
        if let Some(result) = EvalResult::terminal(state) {
            return result;
        }

        if depth == 0 {
            return eval(state);
        }

        for game_move in state.candidate_moves() {
            state.make_move(game_move);
            let score = recursive(state, depth - 1, -beta, -alpha, eval);
            state.undo();
            if score <= beta {
                return -beta; // beta cutoff
            }
            if score < alpha {
                alpha = score;
            }
        }

        -alpha
    }

    move |state, depth, alpha, beta| recursive(state, depth, alpha, beta, &eval)
}

pub fn alphabeta_tt<S: GameState>(eval: impl Evaluation<S>) -> impl ABSearch<S> {
    struct SearchState<E> {
        eval: E,
        table: TTable,
    }
    impl<E> SearchState<E> {
        fn search<S: GameState>(
            &mut self,
            state: &mut S,
            depth: u8,
            mut alpha: EvalResult,
            mut beta: EvalResult,
        ) -> EvalResult
        where
            E: Evaluation<S>,
        {
            if let Some(result) = EvalResult::terminal(state) {
                return result;
            }

            if depth == 0 {
                return (self.eval)(state);
            }

            let alpha_orig = alpha;
            let state_hash = state.hash();

            if let Some(entry) = self.table.get(state_hash, depth) {
                // Stored as this node's result; `alpha` and `beta` compare child scores, its negation.
                let value = -entry.value;
                match entry.bound {
                    TTBound::Exact => return entry.value,
                    TTBound::Lower => {
                        if value <= beta {
                            return -beta;
                        }
                        if value < alpha {
                            alpha = value;
                        }
                    }
                    TTBound::Upper => {
                        if value >= alpha {
                            return -alpha;
                        }
                        if value > beta {
                            beta = value;
                        }
                    }
                }
            }

            for game_move in state.candidate_moves() {
                state.make_move(game_move);
                let score = self.search(state, depth - 1, -beta, -alpha);
                state.undo();
                if score <= beta {
                    beta = -beta;
                    self.table.store(state_hash, depth, beta, TTBound::Lower);
                    return beta; // beta cutoff
                }
                if score < alpha {
                    alpha = score;
                }
            }

            let bound = if alpha == alpha_orig {
                TTBound::Upper
            } else {
                TTBound::Exact
            };

            alpha = -alpha;
            self.table.store(state_hash, depth, alpha, bound);
            alpha
        }
    }

    // A `Mutex` rather than a `RefCell`, since `to_eval` needs the search to be `Sync`.
    let search = Mutex::new(SearchState {
        eval,
        table: TTable::new(),
    });
    move |state, depth, alpha, beta| search.lock().unwrap().search(state, depth, alpha, beta)
}

pub fn minimax<S: GameState>(eval: impl Evaluation<S>) -> impl Search<S> {
    fn recursive<S: GameState>(state: &mut S, depth: u8, eval: &impl Evaluation<S>) -> EvalResult {
        if let Some(result) = EvalResult::terminal(state) {
            return result;
        }

        if depth == 0 {
            return eval(state);
        }

        -state
            .candidate_moves()
            .par_iter()
            .copied()
            .map_init(
                || state.clone(),
                |state, game_move| {
                    state.make_move(game_move);
                    let result = recursive(state, depth - 1, eval);
                    state.undo();
                    result
                },
            )
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    move |state: &mut S, depth| recursive(state, depth, &eval)
}
