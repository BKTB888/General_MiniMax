use crate::players::Player;
use crate::search::EvalResult;
use crate::state::GameState;

pub trait Evaluation<S: GameState>: Fn(&S) -> EvalResult {
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
impl<S: GameState, F: Fn(&S) -> EvalResult> Evaluation<S> for F {}

pub fn stupid_eval<S: GameState>(_: &S) -> EvalResult {
    EvalResult::Eval(0.0)
}
