use crate::games::mega_tictactoe::state::KInARowState;
use std::collections::BTreeMap;
use crate::games::mega_tictactoe::coordinate::{Coordinate, MapInt};
use crate::state::GameState;

const DIRS: [(MapInt, MapInt); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];

/// For a given player, sum score² over all maximal unblocked runs in every direction.
/// A run is "unblocked" on one end if the cell beyond it is empty (not occupied by the opponent).
fn player_score<const K: u8, const NUM_P: u8>(
    state: &KInARowState<K, NUM_P>,
    player: u8,
) -> f32 {
    let cells = state.cells();

    // Track which (coord, dir_index) pairs we've already counted
    let mut visited: BTreeMap<(Coordinate, usize), bool> = BTreeMap::new();
    let mut total = 0.0f32;

    for (&start, &owner) in cells.iter() {
        if owner != player {
            continue;
        }

        for (dir_idx, &dir) in DIRS.iter().enumerate() {
            let neg_dir = (-dir.0, -dir.1);

            // Walk back to the true start of this run
            let mut run_start = start;
            while cells.get(&(run_start + neg_dir)).copied() == Some(player) {
                run_start += neg_dir;
            }

            if visited.contains_key(&(run_start, dir_idx)) {
                continue;
            }
            visited.insert((run_start, dir_idx), true);

            // Count run length
            let mut len = 0u8;
            let mut coord = run_start;
            while cells.get(&coord).copied() == Some(player) {
                len += 1;
                coord += dir;
            }

            // Only reward if the run isn't fully blocked on both ends
            let blocked_back = cells.get(&(run_start + neg_dir)).is_some(); // opponent or edge
            let blocked_front = cells.get(&coord).is_some();                // opponent or edge
            if blocked_back && blocked_front {
                // fully sandwiched by opponent pieces — no future value
                continue;
            }

            total += (len as f32).powi(2);
        }
    }

    total
}

pub fn eval_kinrow<const K: u8, const NUM_P: u8>(state: &KInARowState<K, NUM_P>) -> f32 {
    let current = state.get_current_player();

    let my_score: f32 = player_score(state, current);
    let opp_score: f32 = (0..NUM_P)
        .filter(|&p| p != current)
        .map(|p| player_score(state, p))
        .sum();

    my_score - opp_score
}