use crate::player::evals::Evaluation;
use crate::player::players::Player;
use crate::player::search::EvalResult::{Draw, Loss, Win};
use crate::result::GameResult;
use crate::state::GameState;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use std::time::Duration;

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
                .into_par_iter()
                .map_init(
                    || state.clone(), // one clone per worker
                    |state, game_move| {
                        state.make_move(game_move);
                        let result = self(state, depth);
                        state.undo_move(game_move);
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
            state.undo_move(game_move);
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
            let start = std::time::Instant::now();
            let mut depth = 0;
            let (mut game_move, mut result) = self.find_best(&mut state.clone(), depth);

            loop {
                if start.elapsed() >= duration || result.is_terminal() {
                    break;
                }

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
        if let Some(result) = state.get_result() {
            return if let GameResult::Player(player) = result {
                if player == state.current_player() {
                    Win
                } else {
                    Loss
                }
            } else {
                Draw
            };
        }

        if depth == 0 {
            return eval(state);
        }

        for game_move in state.candidate_moves() {
            state.make_move(game_move);
            let score = recursive(state, depth - 1, -beta, -alpha, eval);
            state.undo_move(game_move);
            if score <= beta {
                return -beta; // beta cutoff
            }
            if score < alpha {
                alpha = score;
            }
        }

        -alpha
    }

    move |state, depth, alpha, beta| recursive(&mut state.clone(), depth, alpha, beta, &eval)
}
/*
pub fn alphabeta_tt<S: GameState>(eval: impl Evaluation<S>) -> impl ABSearch<S> {
    struct Helper<S: GameState, E: Evaluation<S>> {
        eval: E,
        table: HashMap<<S as GameState>::Hash, TTEntry>,
    }
    impl<S: GameState, E: Evaluation<S>> Helper<S, E> {
        fn search(
            &mut self,
            state: &S,
            depth: u8,
            mut alpha: EvalResult,
            mut beta: EvalResult,
        ) -> EvalResult {
            if let Some(result) = state.get_result() {
                return if let GameResult::Player(player) = result {
                    if player == state.current_player() { Win } else { Loss }
                } else { Draw };
            }

            if depth == 0 {
                return (self.eval)(state);
            }

            let alpha_orig = alpha;
            let state_hash = state.hash();

            if let Some(entry) = self.table.get(&state_hash) {
                if entry.depth >= depth {
                    match entry.bound {
                        TTBound::Exact => return entry.value,
                        TTBound::Lower => {
                            if entry.value >= beta {
                                println!("Hit: {:#?}", entry);
                                return beta;
                            }
                            if entry.value > alpha {
                                alpha = entry.value;
                            }
                        }
                        TTBound::Upper => {
                            if entry.value <= alpha {
                                return alpha;
                            }
                            if entry.value < beta {
                                beta = entry.value;
                            }
                        }
                    }
                }
            }

            for game_move in state.candidate_moves() {
                let mut next = state.clone();
                next.make_move(game_move);
                let score = -self.search(&next, depth - 1, -beta, -alpha);
                if score >= beta {
                    self.table.insert(
                        state_hash,
                        TTEntry {
                            depth,
                            value: beta,
                            bound: TTBound::Lower,
                        },
                    );
                    return beta; // beta cutoff
                }
                if score > alpha {
                    alpha = score;
                }
            }

            let bound = if alpha == alpha_orig {
                TTBound::Upper
            } else {
                TTBound::Exact
            };
            self.table.insert(
                state_hash,
                TTEntry {
                    depth,
                    value: alpha,
                    bound,
                },
            );

            alpha
        }

        pub fn new(eval: E) -> Self {
            Helper { eval, table: HashMap::new() }
        }
    }


    move |state, depth, alpha, beta|
        Helper::new(eval.clone()).search(state, depth, alpha, beta)
}

struct AlphaBeta<S: GameState, E: Evaluation<S>> {
    eval: E,
    table: HashMap<<S as GameState>::Hash, TTEntry>,
}

impl<S: GameState, E: Evaluation<S>> AlphaBeta<S, E> {
    fn new(eval: E) -> Self {
        AlphaBeta { eval, table: HashMap::new() }
    }

    fn search(
        &mut self,
        state: &S,
        depth: u8,
        mut alpha: EvalResult,
        mut beta: EvalResult,
    ) -> EvalResult {
        if let Some(result) = state.get_result() {
            return if let GameResult::Player(player) = result {
                if player == state.current_player() {
                    Win
                } else {
                    Loss
                }
            } else {
                Draw
            };
        }

        if depth == 0 {
            return self.eval(state);
        }

        let alpha_orig = alpha;
        let key = state.hash();

        if let Some(entry) = self.table.get(&key) {
            if entry.depth >= depth {
                match entry.bound {
                    TTBound::Exact => return entry.value,
                    TTBound::Lower => {
                        if entry.value >= beta {
                            return beta;
                        }
                        if entry.value > alpha {
                            alpha = entry.value;
                        }
                    }
                    TTBound::Upper => {
                        if entry.value <= alpha {
                            return alpha;
                        }
                        if entry.value < beta {
                            beta = entry.value;
                        }
                    }
                }
            }
        }

        for game_move in state.candidate_moves() {
            let mut next = state.clone();
            next.make_move(game_move);
            let score = -self.search(&next, depth - 1, -beta, -alpha);
            if score >= beta {
                self.table.insert(
                    key,
                    TTEntry {
                        depth,
                        value: beta,
                        bound: TTBound::Lower,
                    },
                );
                return beta; // beta cutoff
            }
            if score > alpha {
                alpha = score;
            }
        }

        let bound = if alpha == alpha_orig {
            TTBound::Upper
        } else {
            TTBound::Exact
        };
        self.table.insert(
            key,
            TTEntry {
                depth,
                value: alpha,
                bound,
            },
        );

        alpha
    }


}

 */

pub fn minimax<S: GameState>(eval: impl Evaluation<S>) -> impl Search<S> {
    fn recursive<S: GameState>(state: &mut S, depth: u8, eval: &impl Evaluation<S>) -> EvalResult {
        if let Some(result) = state.get_result() {
            return if let GameResult::Player(player) = result {
                if player == state.current_player() {
                    Win
                } else {
                    Loss
                }
            } else {
                Draw
            };
        }

        if depth == 0 {
            return eval(state);
        }

        -state
            .candidate_moves()
            .into_par_iter()
            .map_init(
                || state.clone(),
                |state, game_move| {
                    state.make_move(game_move);
                    let result = recursive(state, depth - 1, eval);
                    state.undo_move(game_move);
                    result
                },
            )
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    move |state: &mut S, depth| recursive(state, depth, &eval)
}
