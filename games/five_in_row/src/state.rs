use std::fmt::{Display, Formatter, Result as FmtResult};

use colored::{ColoredString, Colorize};
use general_minimax::{
    coordinate::Coordinate,
    mixers::Splitmix,
    result::{GameResult, get_player_color},
    state::GameState,
};
use smallvec::SmallVec;

use crate::map::{Map, MapCoord, MapInt};

/// Empty cells drawn around `bounds` on every side.
pub const BORDER: MapInt = 2;

/// Terminal columns and rows one cell takes when displayed, the columns counting the grid line
/// to its right.
pub const CELL_WIDTH: MapInt = 2;
pub const CELL_HEIGHT: MapInt = 1;

/// Candidate moves held without a heap allocation.
const INLINE_MOVES: usize = 64;

#[derive(Clone, Default)]
pub struct FiveInRowState {
    cells: Map,
    player: u8,
    result: Option<GameResult>,
    hash: u64,
}

impl From<Vec<MapCoord>> for FiveInRowState {
    fn from(coords: Vec<MapCoord>) -> Self {
        let mut result = Self::default();
        coords.into_iter().for_each(|coord| {
            result.make_move(coord);
        });

        result
    }
}

impl GameState for FiveInRowState {
    type Choice = MapCoord;
    // Unbounded on an infinite board, but usually few enough to skip the heap.
    type Moves = SmallVec<[Self::Choice; INLINE_MOVES]>;
    const NUM_P: u8 = 2;
    // 20 evicts so much that a game of `alphabeta-tt:4` against `alphabeta-tt:6` takes twice as
    // long.
    const TT_BITS: u8 = 22;

    fn make_move(&mut self, coord: Self::Choice) {
        if self.result.is_none() {
            let five = self.cells.place(coord, self.player);
            self.hash ^= zobrist_cell_key(coord, self.player);
            self.result = five.then_some(GameResult::Player(self.player));

            self.player ^= 1;
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
        SmallVec::from_slice(self.cells.candidates())
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

    fn ply(&self) -> u32 {
        self.cells.stones().count() as u32
    }

    fn undo(&mut self) {
        let choice = self.cells.undo();
        self.player ^= 1;
        self.result = None;
        self.hash ^= zobrist_cell_key(choice, self.player);
    }
}

impl FiveInRowState {
    /// How many windows, five cells in a row holding stones of `player` only, hold 1, 2, 3 and
    /// 4 of them.
    pub fn windows(&self, player: u8) -> [u16; 4] {
        self.cells.windows(player)
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
}
impl Display for FiveInRowState {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let (Coordinate(min_r, min_c), Coordinate(max_r, max_c)) = self.bounds();
        let cols = min_c - BORDER..=max_c + BORDER;

        for row in (min_r - BORDER..=max_r + BORDER).rev() {
            // Underlining draws the line between rows without a terminal row of its own. Each
            // piece is underlined alone, since a stone's colour reset would also end the underline.
            let line = |s: ColoredString| {
                if row != min_r - BORDER {
                    s.underline()
                } else {
                    s
                }
                .to_string()
            };
            let cells: Vec<String> = cols
                .clone()
                .map(|col| match self.cells.get(Coordinate(row, col)) {
                    Some(player) => {
                        line(["O", "X"][player as usize].color(get_player_color(player)))
                    }
                    None => line(" ".normal()),
                })
                .collect();
            // `\r` because the human player draws in raw mode, where `\n` doesn't return the cursor.
            writeln!(f, "{}\r", cells.join(&line("│".normal())))?;
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
        let state = FiveInRowState::default();
        assert_eq!(state.player, 0);
        assert!(state.cells.stones().next().is_none());
    }

    #[test]
    fn test_make_move_changes_player_and_cells() {
        let mut state = FiveInRowState::default();
        state.make_move(C(0, 0));
        assert_eq!(state.cells.get(C(0, 0)), Some(0));
        assert_eq!(state.player, 1);

        state.make_move(C(1, 0));
        assert_eq!(state.cells.get(C(1, 0)), Some(1));
        assert_eq!(state.player, 0);
    }

    fn make_moves(state: &mut FiveInRowState, moves: &[MapCoord]) {
        for &m in moves {
            state.make_move(m);
        }
    }

    fn assert_states_eq(a: &FiveInRowState, b: &FiveInRowState) {
        assert_eq!(a.cells, b.cells, "cells differ");
        assert_eq!(a.player, b.player, "player differs");
        assert_eq!(a.result, b.result, "result differs");
        assert_eq!(a.hash, b.hash, "hash differs");
    }

    #[test]
    fn test_zobrist_changes_on_move() {
        let s0 = FiveInRowState::default();
        let mut s1 = s0.clone();
        s1.make_move(C(0, 0));
        assert_ne!(s0.hash, s1.hash);
    }

    #[test]
    fn test_zobrist_same_position_same_hash() {
        let mut a = FiveInRowState::default();
        make_moves(&mut a, &[C(0, 0), C(1, 0), C(0, 1), C(1, 1)]);
        let mut b = FiveInRowState::default();
        make_moves(&mut b, &[C(0, 0), C(1, 0), C(0, 1), C(1, 1)]);
        assert_eq!(a.hash, b.hash);

        // Different move order, same final ownership: p0 owns (0,0)+(0,1), p1 owns (1,0)+(1,1).
        let mut c = FiveInRowState::default();
        make_moves(&mut c, &[C(0, 0), C(1, 0), C(0, 1), C(1, 1)]);
        let mut d = FiveInRowState::default();
        make_moves(&mut d, &[C(0, 1), C(1, 1), C(0, 0), C(1, 0)]);
        assert_eq!(c.hash, d.hash);
    }

    #[test]
    fn test_zobrist_distinct_for_distinct_positions() {
        let mut a = FiveInRowState::default();
        a.make_move(C(0, 0));
        let mut b = FiveInRowState::default();
        b.make_move(C(1, 0));
        assert_ne!(a.hash, b.hash);
    }

    #[test]
    fn test_zobrist_no_xor_cancellation() {
        let mut a = FiveInRowState::default();
        make_moves(&mut a, &[C(0, 0), C(1, 0), C(1, 1), C(0, 1)]);
        let mut b = FiveInRowState::default();
        make_moves(&mut b, &[C(1, 0), C(0, 0), C(0, 1), C(1, 1)]); // owners swapped
        assert_ne!(a.hash, b.hash);
        assert_ne!(a.hash, FiveInRowState::default().hash);
        assert_ne!(b.hash, FiveInRowState::default().hash);
    }

    #[test]
    fn test_zobrist_distinct_with_negative_coords() {
        let mut a = FiveInRowState::default();
        a.make_move(C(3, -1));
        let mut b = FiveInRowState::default();
        b.make_move(C(5, -1));
        assert_ne!(a.hash, b.hash);
    }

    #[test]
    fn test_undo_single_move_restores_default() {
        let original = FiveInRowState::default();
        let mut s = FiveInRowState::default();
        s.make_move(C(0, 0));
        s.undo();
        assert_states_eq(&s, &original);
    }

    #[test]
    fn test_undo_chain_returns_to_default() {
        let original = FiveInRowState::default();
        let mut s = FiveInRowState::default();
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
        let mut s = FiveInRowState::default();
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
    fn test_undo_nested_make_undo_pattern() {
        let original = FiveInRowState::default();
        let mut s = FiveInRowState::default();
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

const fn zobrist_cell_key(coord: MapCoord, player: u8) -> u64 {
    // `as u16` keeps negatives to their own 16 bits instead of sign-extending over the others.
    let idx = (coord.0 as u16 as u64) << 32 | (coord.1 as u16 as u64) << 16 | player as u64;
    // Splitmix, not xorshift: a linear mixer lets keys XOR-cancel across cells.
    idx.splitmix()
}
