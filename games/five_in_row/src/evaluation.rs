use general_minimax::{player::evals::Score, state::GameState};

use crate::state::FiveInRowState;

/// What each live window holding 1, 2, 3 and 4 stones is worth to the other player. Each stone
/// is worth five times the last: ×4 and ×6 both lost to it.
const OPP: [Score; 4] = [1.0, 5.0, 25.0, 125.0];
/// What the same windows are worth to the player to move, who can add to them first: 1.5 times
/// as much, and a four is five on this move. More than ×2 lost, and so did favouring the
/// other player.
const OWN: [Score; 4] = [1.5, 7.5, 37.5, 100_000.0];

/// Scores `state` for the player to move by its live windows: five cells in a row holding
/// stones of one player only.
pub fn eval(state: &FiveInRowState) -> Score {
    let mover = state.current_player();
    let (own, opp) = (state.windows(mover), state.windows(mover ^ 1));
    (0..4)
        .map(|n| OWN[n] * own[n] as Score - OPP[n] * opp[n] as Score)
        .sum()
}
