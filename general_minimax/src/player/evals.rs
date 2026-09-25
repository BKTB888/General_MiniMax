use std::{fmt::Display, ops::Neg};

use crate::{
    player::{
        players::Player,
        search::{
            EvalResult,
            EvalResult::{Draw, Loss, Score, Win},
        },
    },
    result::GameResult,
    state::GameState,
};

pub trait Evaluation<S: GameState>: Fn(&S) -> EvalResult + Send {
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
impl<S: GameState, F: Fn(&S) -> EvalResult + Send> Evaluation<S> for F {}

impl EvalResult {
    /// The finished game's result for the player to move, or `None` while it's still going.
    pub fn terminal<S: GameState>(state: &S) -> Option<Self> {
        Some(match state.get_result()? {
            GameResult::Player(player) if player == state.current_player() => Win,
            GameResult::Player(_) => Loss,
            GameResult::Draw => Draw,
        })
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Win | Loss | Draw)
    }

    fn score(&self) -> f32 {
        match self {
            Win => f32::INFINITY,
            Loss => f32::NEG_INFINITY,
            Draw => 0.0,
            Score(score) => *score,
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
            Score(score) => Score(-score),
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
            Score(score) => write!(f, "{score}"),
        }
    }
}

pub fn stupid_eval<S: GameState>(_: &S) -> EvalResult {
    Score(0.0)
}
