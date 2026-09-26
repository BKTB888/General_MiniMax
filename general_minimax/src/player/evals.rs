use crate::{player::players::Player, state::GameState};

/// An alias, so f32 and f64 can be compared by changing one line.
pub type Score = f32;

/// Scores a position for the player to move. The score must be finite, since infinities mean a
/// proven win or loss.
pub trait Evaluation<S: GameState>: Fn(&S) -> Score + Send {
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
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .unwrap()
                .0
        }
    }
}
impl<S: GameState, F: Fn(&S) -> Score + Send> Evaluation<S> for F {}

pub fn stupid_eval<S: GameState>(_: &S) -> Score {
    0.1
}
