use general_minimax::{player::evals::Score, state::GameState};

use crate::state::FiveInRowState;

/// Beyond any sum of weights, but finite, since infinities are proven results.
const WIN: Score = 1e9;

/// What each live window holding 1, 2 and 3 stones is worth to the player to move.
const OWN: [Score; 3] = [35.0, 800.0, 15_000.0];
/// What each live window holding 1, 2, 3 and 4 stones is worth to the other player.
const OPP: [Score; 4] = [15.0, 400.0, 1_800.0, 100_000.0];

/// Scores `state` for the player to move by its live windows: five cells in a row holding
/// stones of one player only.
pub fn eval(state: &FiveInRowState) -> Score {
    let mover = state.current_player();
    let (own, opp) = (state.windows(mover), state.windows(mover ^ 1));

    // Its one empty cell makes five.
    if own[3] > 0 {
        return WIN;
    }
    // Two fours can only be blocked together if they share their empty cell.
    if opp[3] >= 2 {
        return -WIN / 2.0;
    }

    let own: Score = (0..3).map(|n| OWN[n] * own[n] as Score).sum();
    let opp: Score = (0..4).map(|n| OPP[n] * opp[n] as Score).sum();
    own - opp
}
