use std::{
    collections::hash_map::Entry,
    fmt::{Display, Formatter, Result as FmtResult},
};

use colored::Colorize;
use general_minimax::{
    coordinate::Coordinate,
    mixers::Splitmix,
    result::{GameResult, get_player_color},
    state::GameState,
};

use crate::map::{FixedMap, Map, MapCoord, MapInt};

/// Empty cells drawn around `bounds` on every side.
pub const BORDER: MapInt = 2;

#[derive(Clone)]
pub struct KInARowState {
    cells: Map,
    player: u8,
    candidate_moves_with_counts: FixedMap<MapCoord, u16>,
    move_history_with_candidates: FixedMap<MapCoord, (u16, Vec<MapCoord>)>,
    result: Option<GameResult>,
    hash: u64,
    move_stack: Vec<MapCoord>,
}

impl Default for KInARowState {
    fn default() -> Self {
        Self {
            cells: Map::default(),
            player: 0,
            candidate_moves_with_counts: [((0, 0).into(), 1)].into_iter().collect(),
            move_history_with_candidates: FixedMap::default(),
            result: None,
            hash: 0,
            move_stack: Vec::new(),
        }
    }
}

impl From<Vec<MapCoord>> for KInARowState {
    fn from(coords: Vec<MapCoord>) -> Self {
        let mut result = Self::default();
        coords.into_iter().for_each(|coord| {
            result.make_move(coord);
        });

        result
    }
}

impl GameState for KInARowState {
    type Choice = MapCoord;
    // Unbounded on an infinite board.
    type Moves = Vec<Self::Choice>;
    const NUM_P: u8 = 2;

    fn make_move(&mut self, coord: Self::Choice) {
        if self.result.is_none() {
            let five = self.cells.place(coord, self.player);
            self.hash ^= zobrist_cell_key(coord, self.player);
            self.result = five.then_some(GameResult::Player(self.player));

            self.player = (self.player + 1) % Self::NUM_P;
            let prior_count = self.candidate_moves_with_counts.remove(&coord).unwrap_or(0);

            self.add_candidates(coord, prior_count);
            self.move_stack.push(coord);
        } else {
            panic!(
                "Game is over, but player {} tried to make a move {coord}.",
                self.player
            );
        }
    }

    //If none, there is no result
    fn get_result(&self) -> Option<GameResult> {
        self.result
    }

    fn candidate_moves(&self) -> Self::Moves {
        self.candidate_moves_with_counts.keys().copied().collect()
    }

    fn is_valid(&self, choice: Self::Choice) -> bool {
        self.cells.get(choice).is_none()
    }

    fn current_player(&self) -> u8 {
        self.player
    }

    fn hash(&self) -> u64 {
        self.hash
    }

    fn undo(&mut self) {
        let choice = self.move_stack.pop().unwrap();
        self.cells.remove(choice);
        self.player = self.player.checked_sub(1).unwrap_or(Self::NUM_P - 1);
        self.result = None;
        let (count, candidates) = self.move_history_with_candidates.remove(&choice).unwrap();
        for coord in candidates {
            if let Entry::Occupied(mut e) = self.candidate_moves_with_counts.entry(coord) {
                let count = e.get_mut();
                *count -= 1;
                if *count == 0 {
                    e.remove();
                }
            }
        }
        self.candidate_moves_with_counts.insert(choice, count);

        self.hash ^= zobrist_cell_key(choice, self.player);
    }
}

impl KInARowState {
    pub fn cells(&self) -> &Map {
        &self.cells
    }
    /// The lowest and highest row and column holding a piece, as `(min, max)`. `(0, 0)` for
    /// both on an empty board.
    pub fn bounds(&self) -> (MapCoord, MapCoord) {
        let mut coords = self.cells.stones().map(|(coord, _)| coord);
        let Some(first) = coords.next() else {
            return Default::default();
        };
        coords.fold((first, first), |(min, max), Coordinate(r, c)| {
            (
                Coordinate(min.0.min(r), min.1.min(c)),
                Coordinate(max.0.max(r), max.1.max(c)),
            )
        })
    }
    fn add_candidates(&mut self, from: MapCoord, prior_count: u16) {
        const R: MapInt = 1;
        const NEIGHBOURS: [MapCoord; ((2 * R + 1).pow(2) - 1) as usize] = neighbour_offsets(R);

        let candidates = NEIGHBOURS
            .iter()
            .map(|&coord| coord + from)
            .filter(|&coord| self.is_valid(coord))
            .collect::<Vec<_>>();

        candidates.iter().for_each(|coord| {
            *self.candidate_moves_with_counts.entry(*coord).or_default() += 1;
        });

        let (count, vec) = self.move_history_with_candidates.entry(from).or_default();
        *count = prior_count;
        vec.extend(candidates);
    }
}
impl Display for KInARowState {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let (Coordinate(min_r, min_c), Coordinate(max_r, max_c)) = self.bounds();
        for row in (min_r - BORDER..=max_r + BORDER).rev() {
            for col in min_c - BORDER..=max_c + BORDER {
                let coord = Coordinate(row, col);
                if let Some(player) = self.cells.get(coord) {
                    let colored = ["O", "X"][player as usize].color(get_player_color(player));
                    write!(f, "{colored}")?;
                } else {
                    write!(f, "·")?;
                }
            }
            writeln!(f, "\r")?; // newline after each row
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use general_minimax::coordinate::Coordinate as C;

    use super::*;

    #[test]
    fn test_default_state() {
        let state = KInARowState::default();
        assert_eq!(state.player, 0);
        assert!(state.cells.stones().next().is_none());
    }

    #[test]
    fn test_make_move_changes_player_and_cells() {
        let mut state = KInARowState::default();
        state.make_move(C(0, 0));
        assert_eq!(state.cells.get(C(0, 0)), Some(0));
        assert_eq!(state.player, 1);

        state.make_move(C(1, 0));
        assert_eq!(state.cells.get(C(1, 0)), Some(1));
        assert_eq!(state.player, 0);
    }

    #[test]
    fn test_try_get_result_detects_win() {
        let mut state = KInARowState::default();
        #[rustfmt::skip]
        make_moves(&mut state, &[
            C(0, 0), C(1, 0),
            C(0, 1), C(1, 1),
            C(0, 2), C(1, 2),
            C(0, 3), C(1, 3),
            C(0, 4), // Player 0 wins
        ]);

        assert_eq!(state.get_result(), Some(GameResult::Player(0)));
    }

    #[test]
    fn test_try_get_result_no_win() {
        let mut state = KInARowState::default();
        #[rustfmt::skip]
        make_moves(&mut state, &[
            C(0, 0), C(1, 0),
            C(0, 1), C(1, 1),
            C(0, 2), C(1, 2),
            C(0, 3), // Player 0 has four
        ]);

        assert_eq!(state.get_result(), None);
    }

    #[test]
    fn test_diagonal_win_through_negatives() {
        #[rustfmt::skip]
        let state = KInARowState::from(vec![
            C(2, -2),  C(5, 5),
            C(-2, 2),  C(5, 6),
            C(0, 0),   C(5, 7),
            C(1, -1),  C(-5, 7),
            C(-1, 1), // Player 0 wins with a stone inside the line, not at an end
        ]);

        println!("{state}");

        assert_eq!(state.get_result(), Some(GameResult::Player(0)));
    }

    fn make_moves(state: &mut KInARowState, moves: &[MapCoord]) {
        for &m in moves {
            state.make_move(m);
        }
    }

    fn assert_states_eq(a: &KInARowState, b: &KInARowState) {
        assert_eq!(a.cells, b.cells, "cells differ");
        assert_eq!(a.player, b.player, "player differs");
        assert_eq!(
            a.candidate_moves_with_counts, b.candidate_moves_with_counts,
            "candidate_moves_with_counts differs"
        );
        assert_eq!(
            a.move_history_with_candidates, b.move_history_with_candidates,
            "move_history_with_candidates differs"
        );
        assert_eq!(a.result, b.result, "result differs");
        assert_eq!(a.hash, b.hash, "hash differs");
    }

    #[test]
    fn test_zobrist_changes_on_move() {
        let s0 = KInARowState::default();
        let mut s1 = s0.clone();
        s1.make_move(C(0, 0));
        assert_ne!(s0.hash, s1.hash);
    }

    #[test]
    fn test_zobrist_same_position_same_hash() {
        let mut a = KInARowState::default();
        make_moves(&mut a, &[C(0, 0), C(1, 0), C(0, 1), C(1, 1)]);
        let mut b = KInARowState::default();
        make_moves(&mut b, &[C(0, 0), C(1, 0), C(0, 1), C(1, 1)]);
        assert_eq!(a.hash, b.hash);

        // Different move order, same final ownership: p0 owns (0,0)+(0,1), p1 owns (1,0)+(1,1).
        let mut c = KInARowState::default();
        make_moves(&mut c, &[C(0, 0), C(1, 0), C(0, 1), C(1, 1)]);
        let mut d = KInARowState::default();
        make_moves(&mut d, &[C(0, 1), C(1, 1), C(0, 0), C(1, 0)]);
        assert_eq!(c.hash, d.hash);
    }

    #[test]
    fn test_zobrist_distinct_for_distinct_positions() {
        let mut a = KInARowState::default();
        a.make_move(C(0, 0));
        let mut b = KInARowState::default();
        b.make_move(C(1, 0));
        assert_ne!(a.hash, b.hash);
    }

    #[test]
    fn test_zobrist_no_xor_cancellation() {
        let mut a = KInARowState::default();
        make_moves(&mut a, &[C(0, 0), C(1, 0), C(1, 1), C(0, 1)]);
        let mut b = KInARowState::default();
        make_moves(&mut b, &[C(1, 0), C(0, 0), C(0, 1), C(1, 1)]); // owners swapped
        assert_ne!(a.hash, b.hash);
        assert_ne!(a.hash, KInARowState::default().hash);
        assert_ne!(b.hash, KInARowState::default().hash);
    }

    #[test]
    fn test_zobrist_distinct_with_negative_coords() {
        let mut a = KInARowState::default();
        a.make_move(C(3, -1));
        let mut b = KInARowState::default();
        b.make_move(C(5, -1));
        assert_ne!(a.hash, b.hash);
    }

    #[test]
    fn test_undo_single_move_restores_default() {
        let original = KInARowState::default();
        let mut s = KInARowState::default();
        s.make_move(C(0, 0));
        s.undo();
        assert_states_eq(&s, &original);
    }

    #[test]
    fn test_undo_zobrist_round_trip() {
        let mut s = KInARowState::default();
        let h0 = s.hash;
        s.make_move(C(0, 0));
        assert_ne!(s.hash, h0);
        s.undo();
        assert_eq!(s.hash, h0);
    }

    #[test]
    fn test_undo_zobrist_round_trip_long_game() {
        let mut s = KInARowState::default();
        let h0 = s.hash;
        let moves = [
            C(0, 0),
            C(1, 0),
            C(0, 1),
            C(1, 1),
            C(2, 0),
            C(2, 1),
            C(0, 2),
            C(1, 2),
            C(2, 2),
            C(3, 0),
        ];
        for &m in &moves {
            s.make_move(m);
        }
        for _ in &moves {
            s.undo();
        }
        assert_eq!(s.hash, h0);
    }

    #[test]
    fn test_undo_chain_returns_to_default() {
        let original = KInARowState::default();
        let mut s = KInARowState::default();
        let moves = [
            C(0, 0),
            C(1, 0),
            C(0, 1),
            C(1, 1),
            C(2, 0),
            C(2, 1),
            C(0, 2),
            C(1, 2),
            C(2, 2),
            C(3, 0),
        ];
        for &m in &moves {
            s.make_move(m);
        }
        for _ in &moves {
            s.undo();
        }
        assert_states_eq(&s, &original);
    }

    #[test]
    fn test_undo_clears_winning_result() {
        let mut s = KInARowState::default();
        #[rustfmt::skip]
        make_moves(&mut s, &[
            C(0, 0), C(1, 0),
            C(0, 1), C(1, 1),
            C(0, 2), C(1, 2),
            C(0, 3), C(1, 3),
        ]);
        let snapshot = s.clone();
        s.make_move(C(0, 4)); // p0 wins along row 0
        assert_eq!(s.get_result(), Some(GameResult::Player(0)));
        s.undo();
        assert_eq!(s.get_result(), None);
        assert_states_eq(&s, &snapshot);
    }

    #[test]
    fn test_undo_player_wrap_2p() {
        let mut s = KInARowState::default();
        assert_eq!(s.current_player(), 0);
        s.make_move(C(0, 0));
        assert_eq!(s.current_player(), 1);
        s.undo();
        assert_eq!(s.current_player(), 0);
    }

    #[test]
    fn test_undo_nested_make_undo_pattern() {
        let original = KInARowState::default();
        let mut s = KInARowState::default();
        s.make_move(C(0, 0));
        let after_outer = s.clone();
        s.make_move(C(1, 0));
        s.undo();
        assert_states_eq(&s, &after_outer);
        s.make_move(C(0, 1));
        s.undo();
        assert_states_eq(&s, &after_outer);
        s.undo();
        assert_states_eq(&s, &original);
    }
}

/// Every offset at most `r` away on both axes, except `(0, 0)`. `N` must be `(2r + 1)² - 1`.
const fn neighbour_offsets<const N: usize>(r: MapInt) -> [MapCoord; N] {
    let mut arr = [Coordinate(0, 0); N];
    let mut i = 0;
    for a in -r..r + 1 {
        for b in -r..r + 1 {
            if a != 0 || b != 0 {
                arr[i] = Coordinate(a, b);
                i += 1;
            }
        }
    }
    arr
}

const fn zobrist_cell_key(coord: MapCoord, player: u8) -> u64 {
    // `as u16` keeps negatives to their own 16 bits instead of sign-extending over the others.
    let idx = (coord.0 as u16 as u64) << 32 | (coord.1 as u16 as u64) << 16 | player as u64;
    // Splitmix, not xorshift: a linear mixer lets keys XOR-cancel across cells.
    idx.splitmix()
}
