use crate::games::mancala::state::MancalaState;
use crate::search::EvalResult;
use crate::search::EvalResult::Score;

pub fn eval(state: &MancalaState) -> EvalResult {
    let balls_at_op: u16 = state
        .opponent_side()
        .into_iter()
        .map(|x| x as u16)
        .sum::<u16>();
    let balls_at_me: u16 = state
        .current_side()
        .into_iter()
        .map(|x| x as u16)
        .sum::<u16>();
    let sum = (balls_at_op + balls_at_me) as f32;

    Score((balls_at_op as f32) / sum)
}
