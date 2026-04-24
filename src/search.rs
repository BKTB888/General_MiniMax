use crate::evals::Evaluation;
use crate::players::Player;
use crate::result::GameResult;
use crate::search::EvalResult::{Draw, Loss, Win};
use crate::state::GameState;

pub trait Search<S: GameState>: Fn(&S, u8) -> EvalResult {
    fn to_eval(self, depth: u8) -> impl Evaluation<S>
    where
        Self: Sized,
    {
        move |state| self(state, depth)
    }
}
impl<S: GameState, F: Fn(&S, u8) -> EvalResult> Search<S> for F {}

pub trait ABSearch<S: GameState>: Fn(&S, u8, EvalResult, EvalResult) -> EvalResult {
    fn to_eval(self, depth: u8) -> impl Evaluation<S>
    where
        Self: Sized,
    {
        move |state| self(state, depth, Loss, Win)
    }

    fn to_player(self, depth: u8) -> impl Player<S>
    where
        Self: Sized,
    {
        assert!(depth > 0);
        move |state| {
            let moves = state.candidate_moves();
            let mut alpha = Loss;
            let mut alpha_move = moves[0];
            let beta = Win;

            for game_move in moves {
                let mut next = state.clone();
                next.make_move(game_move);
                let score = -self(&next, depth - 1, -beta, -alpha);
                if score >= beta {
                    return game_move; // beta cutoff
                }
                if score > alpha {
                    alpha = score;
                    alpha_move = game_move;
                }
            }

            alpha_move
        }
    }
}
impl<S: GameState, F: Fn(&S, u8, EvalResult, EvalResult) -> EvalResult> ABSearch<S> for F {}

#[derive(PartialEq, Copy, Clone)]
pub enum EvalResult {
    Win,
    Loss,
    Draw,
    Eval(f32),
}

pub fn alphabeta<S: GameState>(eval: impl Evaluation<S>) -> impl ABSearch<S> {
    fn recursive<S: GameState>(
        state: &S,
        depth: u8,
        mut alpha: EvalResult,
        beta: EvalResult,
        eval: &impl Evaluation<S>,
    ) -> EvalResult {
        if let Some(result) = state.get_result() {
            return if let GameResult::Player(player) = result {
                if player == state.get_current_player() {
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

    move |state, depth, alpha, beta| recursive(state, depth, alpha, beta, &eval)
}
