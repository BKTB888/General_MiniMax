use crate::evals::Evaluation;
use crate::result::GameResult;
use crate::search::EvalResult::{Draw, Eval, Loss, Win};
use crate::state::GameState;
use std::fmt::Display;
use std::ops::Neg;

pub trait Search<S: GameState>: Fn(&S, u8) -> EvalResult {
    fn to_eval(self, depth: u8) -> impl Evaluation<S>
    where
        Self: Sized,
    {
        move |state| self(state, depth)
    }
}
impl<S: GameState, F: Fn(&S, u8) -> EvalResult> Search<S> for F {}

#[derive(PartialEq, Copy, Clone)]
pub enum EvalResult {
    Win,
    Loss,
    Draw,
    Eval(f32),
}

impl EvalResult {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Win | Loss | Draw)
    }

    fn score(&self) -> f32 {
        match self {
            Win => f32::INFINITY,
            Loss => f32::NEG_INFINITY,
            Draw => 0.0,
            Eval(score) => *score,
        }
    }
}
impl Neg for EvalResult {
    type Output = EvalResult;
    fn neg(self) -> Self::Output {
        match self {
            Win => Loss,
            Loss => Win,
            Draw => Draw,
            Eval(score) => Eval(-score),
        }
    }
}
impl PartialOrd for EvalResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score().partial_cmp(&other.score())
    }
}
impl Display for EvalResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Win => write!(f, "Win"),
            Loss => write!(f, "Loss"),
            Draw => write!(f, "Draw"),
            Eval(score) => write!(f, "{score}"),
        }
    }
}

pub fn alphabeta<S: GameState>(eval: impl Evaluation<S>) -> impl Search<S> {
    move |state, depth| {
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
        recursive(state, depth, Loss, Win, &eval)
    }
}
