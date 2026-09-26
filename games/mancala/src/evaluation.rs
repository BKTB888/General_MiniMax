use general_minimax::player::evals::Score;

use crate::state::MancalaState;

pub fn eval(state: &MancalaState) -> Score {
    let balls_at_op: u16 = state.opponent_side().into_iter().map(u16::from).sum();

    balls_at_op as Score / state.balls_in_play() as Score
}
