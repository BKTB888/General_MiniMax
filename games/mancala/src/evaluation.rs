use general_minimax::player::search::{EvalResult, EvalResult::Score};

use crate::state::MancalaState;

pub fn eval(state: &MancalaState) -> EvalResult {
    let balls_at_op: u16 = state.opponent_side().into_iter().map(u16::from).sum();

    Score(balls_at_op as f32 / state.balls_in_play() as f32)
}
