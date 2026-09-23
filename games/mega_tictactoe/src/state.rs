use std::{
    collections::{HashMap, hash_map::Entry},
    fmt::{Display, Formatter, Result as FmtResult},
};

use colored::Colorize;
use general_minimax::{
    coordinate::Coordinate,
    mixers::xorshift,
    result::{GameResult, get_player_color},
    state::GameState,
};

pub type MapInt = i16;
pub type MapCoord = Coordinate<MapInt, MapInt>;
type Map = HashMap<MapCoord, u8>;

macro hash_type {
    { $caller:tt } => {
        general_minimax::tt_call::tt_return! {
            $caller
            type = [{ u32 }]
        }
    },
    () => { u32 }
}
type HashType = hash_type!();

#[derive(Clone)]
pub struct KInARowState<const K: u8, const NUM_P: u8 = 2> {
    cells: Map,
    player: u8,
    candidate_moves_with_counts: HashMap<MapCoord, u16>,
    move_history_with_candidates: HashMap<MapCoord, (u16, Vec<MapCoord>)>,
    result: Option<GameResult>,
    hash: HashType,
    move_stack: Vec<MapCoord>,
}

impl<const K: u8, const NUM_P: u8> Default for KInARowState<K, NUM_P> {
    fn default() -> Self {
        Self {
            cells: Map::new(),
            player: 0,
            candidate_moves_with_counts: HashMap::from([((0, 0).into(), 1)]),
            move_history_with_candidates: HashMap::new(),
            result: None,
            hash: 0,
            move_stack: Vec::new(),
        }
    }
}

impl<const K: u8, const NUM_P: u8> From<Vec<MapCoord>> for KInARowState<K, NUM_P> {
    fn from(coords: Vec<MapCoord>) -> Self {
        let mut result = Self::default();
        coords.into_iter().for_each(|coord| {
            result.make_move(coord);
        });

        result
    }
}

impl<const K: u8, const NUM_P: u8> GameState for KInARowState<K, NUM_P> {
    type Choice = MapCoord;
    type Hash = HashType;
    const NUM_P: u8 = NUM_P;

    fn make_move(&mut self, coord: Self::Choice) {
        if self.result.is_none() {
            self.cells.insert(coord.into(), self.player);
            self.hash ^= zobrist_cell_key(
                coord.0 as HashType,
                coord.1 as HashType,
                self.player as HashType,
            );
            self.result = if self.has_won_from(coord) {
                Some(GameResult::Player(self.player))
            } else {
                None
            };

            self.player = (self.player + 1) % NUM_P;
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

    fn candidate_moves(&self) -> Vec<Self::Choice> {
        self.candidate_moves_with_counts.keys().copied().collect()
    }

    fn is_valid(&self, choice: Self::Choice) -> bool {
        !self.cells.contains_key(&choice)
    }

    fn current_player(&self) -> u8 {
        self.player
    }

    fn hash(&self) -> Self::Hash {
        self.hash
    }

    fn undo(&mut self) {
        let choice = self.move_stack.pop().unwrap();
        self.cells.remove(&choice);
        self.player = self.player.checked_sub(1).unwrap_or(NUM_P - 1);
        self.result = None;
        {
            let (count, candidates) = &self.move_history_with_candidates[&choice];
            candidates.iter().for_each(|&coord| {
                if let Entry::Occupied(mut e) = self.candidate_moves_with_counts.entry(coord) {
                    let count = e.get_mut();
                    *count -= 1;
                    if *count == 0 {
                        e.remove();
                    }
                }
            });
            self.candidate_moves_with_counts.insert(choice, *count);
        }

        self.move_history_with_candidates.remove(&choice);
        self.hash ^= zobrist_cell_key(
            choice.0 as HashType,
            choice.1 as HashType,
            self.player as HashType,
        );
    }
}

impl<const K: u8, const NUM_P: u8> KInARowState<K, NUM_P> {
    pub fn cells(&self) -> &Map {
        &self.cells
    }
    fn has_won_from(&self, from: MapCoord) -> bool {
        use Coordinate as C;
        const DIRS: [(MapCoord, MapCoord); 4] = [
            (C(1, 0), C(-1, 0)),
            (C(0, 1), C(0, -1)),
            (C(1, 1), C(-1, -1)),
            (C(1, -1), C(-1, 1)),
        ];

        for (dir_pos, dir_neg) in DIRS {
            let mut count = 1u8;

            for dir in [dir_pos, dir_neg] {
                let mut coord = from + dir;
                while let Some(&cell) = self.cells.get(&coord) {
                    if cell != self.player {
                        break;
                    }
                    count += 1;
                    if count >= K {
                        return true;
                    }
                    coord = coord + dir;
                }
            }
        }

        false
    }
    fn add_candidates(&mut self, from: MapCoord, prior_count: u16) {
        const R: MapInt = 1;
        const NEIGHBOURS: [MapCoord; ((2 * R + 1).pow(2) - 1) as usize] = neighbour_offsets!(R);

        let candidates = NEIGHBOURS
            .iter()
            .map(|&coord| coord + from)
            .filter(|coord| !self.cells.contains_key(coord))
            .collect::<Vec<_>>();

        candidates.iter().for_each(|coord| {
            *self.candidate_moves_with_counts.entry(*coord).or_default() += 1;
        });

        let (count, vec) = self.move_history_with_candidates.entry(from).or_default();
        *count = prior_count;
        vec.extend(candidates);
    }
}
impl<const K: u8, const NUM_P: u8> Display for KInARowState<K, NUM_P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let max_r = self
            .cells
            .keys()
            .map(|Coordinate(r, _)| *r)
            .max()
            .unwrap_or_default();
        let max_c = self
            .cells
            .keys()
            .map(|Coordinate(_, c)| *c)
            .max()
            .unwrap_or_default();
        let min_r = self
            .cells
            .keys()
            .map(|Coordinate(r, _)| *r)
            .min()
            .unwrap_or_default();
        let min_c = self
            .cells
            .keys()
            .map(|Coordinate(_, c)| *c)
            .min()
            .unwrap_or_default();

        let bx = 2;
        // Build board rows
        for row in (min_r - bx..=max_r + bx).rev() {
            for col in min_c - bx..=max_c + bx {
                let coord = Coordinate(row, col);
                if let Some(&player) = self.cells.get(&coord) {
                    // Choose a color for the player
                    let colored = match player {
                        0 => "O", // Red X
                        1 => "X", // Blue O
                        2 => "△", // Green triangle for player 2
                        3 => "◇", // Magenta diamond for player 3
                        _ => "?", // White ? for any extra player
                    }
                    .color(get_player_color(player));
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

    type KIR3 = KInARowState<3>;

    #[test]
    fn test_default_state() {
        let state = KIR3::default();
        assert_eq!(state.player, 0);
        assert!(state.cells.is_empty());
    }

    #[test]
    fn test_make_move_changes_player_and_cells() {
        let mut state = KIR3::default();
        let coord = C(0, 0);
        state.make_move(C(0, 0));
        assert_eq!(state.cells[&coord], 0);
        assert_eq!(state.player, 1);

        state.make_move(C(1, 0));
        assert_eq!(state.cells[&C(1, 0)], 1);
        assert_eq!(state.player, 0);
    }

    #[test]
    fn test_try_get_result_detects_win() {
        let mut state = KIR3::default();
        state.make_move(C(0, 0)); // Player 0
        state.make_move(C(0, 1)); // Player 1
        state.make_move(C(1, 0)); // Player 0
        state.make_move(C(1, 1)); // Player 1
        state.make_move(C(2, 0)); // Player 0 wins

        let result = state.get_result();
        assert!(matches!(result, Some(GameResult::Player(0))));
    }

    #[test]
    fn test_try_get_result_no_win() {
        let mut state = KIR3::default();
        state.make_move(C(0, 0));
        state.make_move(C(0, 1));
        state.make_move(C(1, 0));

        assert_eq!(state.get_result(), None);
    }

    #[test]
    fn complex() {
        let state = KInARowState::<3>::from(vec![
            // Player 0 (X), Player 1 (O) alternating, starting with X
            C(9, 4),  // X
            C(10, 4), // O
            C(8, 4),  // X
            C(11, 5), // O
            C(7, 3),  // X
            C(6, 8),  // O
            C(6, 2),  // X
        ]);

        println!("{state}");

        assert_eq!(state.get_result().unwrap(), GameResult::Player(0));
    }

    type KIR5 = KInARowState<5>;

    fn make_moves<const K: u8, const NUM_P: u8>(
        state: &mut KInARowState<K, NUM_P>,
        moves: &[MapCoord],
    ) {
        for &m in moves {
            state.make_move(m);
        }
    }

    fn assert_states_eq<const K: u8, const NUM_P: u8>(
        a: &KInARowState<K, NUM_P>,
        b: &KInARowState<K, NUM_P>,
    ) {
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
        let s0 = KIR5::default();
        let mut s1 = s0.clone();
        s1.make_move(C(0, 0));
        assert_ne!(s0.hash, s1.hash);
    }

    #[test]
    fn test_zobrist_same_position_same_hash() {
        let mut a = KIR5::default();
        make_moves(&mut a, &[C(0, 0), C(1, 0), C(0, 1), C(1, 1)]);
        let mut b = KIR5::default();
        make_moves(&mut b, &[C(0, 0), C(1, 0), C(0, 1), C(1, 1)]);
        assert_eq!(a.hash, b.hash);

        // Different move order, same final ownership: p0 owns (0,0)+(0,1), p1 owns (1,0)+(1,1).
        let mut c = KIR5::default();
        make_moves(&mut c, &[C(0, 0), C(1, 0), C(0, 1), C(1, 1)]);
        let mut d = KIR5::default();
        make_moves(&mut d, &[C(0, 1), C(1, 1), C(0, 0), C(1, 0)]);
        assert_eq!(c.hash, d.hash);
    }

    #[test]
    fn test_zobrist_distinct_for_distinct_positions() {
        let mut a = KIR5::default();
        a.make_move(C(0, 0));
        let mut b = KIR5::default();
        b.make_move(C(1, 0));
        assert_ne!(a.hash, b.hash);
    }

    #[test]
    fn test_undo_single_move_restores_default() {
        let original = KIR5::default();
        let mut s = KIR5::default();
        s.make_move(C(0, 0));
        s.undo();
        assert_states_eq(&s, &original);
    }

    #[test]
    fn test_undo_zobrist_round_trip() {
        let mut s = KIR5::default();
        let h0 = s.hash;
        s.make_move(C(0, 0));
        assert_ne!(s.hash, h0);
        s.undo();
        assert_eq!(s.hash, h0);
    }

    #[test]
    fn test_undo_zobrist_round_trip_long_game() {
        let mut s = KIR5::default();
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
        let original = KIR5::default();
        let mut s = KIR5::default();
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
        let mut s = KIR3::default();
        make_moves(&mut s, &[C(0, 0), C(0, 1), C(1, 0), C(1, 1)]);
        let snapshot = s.clone();
        s.make_move(C(2, 0)); // p0 wins horizontally at y=0
        assert_eq!(s.get_result(), Some(GameResult::Player(0)));
        s.undo();
        assert_eq!(s.get_result(), None);
        assert_states_eq(&s, &snapshot);
    }

    #[test]
    fn test_undo_player_wrap_2p() {
        let mut s = KIR5::default();
        assert_eq!(s.current_player(), 0);
        s.make_move(C(0, 0));
        assert_eq!(s.current_player(), 1);
        s.undo();
        assert_eq!(s.current_player(), 0);
    }

    #[test]
    fn test_undo_nested_make_undo_pattern() {
        let original = KIR5::default();
        let mut s = KIR5::default();
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

macro neighbour_offsets($r:expr) {{
    const R: MapInt = $r;
    const SIZE: usize = ((2 * R + 1) * (2 * R + 1) - 1) as usize;
    let mut arr = [Coordinate(0, 0); SIZE];
    let mut i = 0;
    let mut a = -R;
    while a <= R {
        let mut b = -R;
        while b <= R {
            if a != 0 || b != 0 {
                arr[i] = Coordinate(a, b);
                i += 1;
            }
            b += 1;
        }
        a += 1;
    }
    arr
}}

const fn zobrist_cell_key(col: HashType, row: HashType, player: HashType) -> HashType {
    let idx = col << 16 | row << 8 | player;
    xorshift!(idx + 1, hash_type!())
}
