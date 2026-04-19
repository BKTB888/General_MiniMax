use crate::games::coordinate::Coordinate;
use crate::result::{GameResult, get_player_color};
use crate::state::GameState;
use colored::Colorize;
use std::collections::{BTreeMap, HashSet};
use std::fmt::{Display, Formatter, Result as FmtResult};

pub type MapInt = i16;
pub type MapCoord = Coordinate<MapInt, MapInt>;
type Map = BTreeMap<MapCoord, u8>;

#[derive(Clone)]
pub struct KInARowState<const K: u8, const NUM_P: u8 = 2> {
    cells: Map,
    player: u8,
    candidate_moves: HashSet<MapCoord>,
    result: Option<GameResult>,
}

impl<const K: u8, const NUM_P: u8> Default for KInARowState<K, NUM_P> {
    fn default() -> Self {
        Self {
            cells: Map::new(),
            player: 0,
            candidate_moves: HashSet::from([(0, 0).into()]),
            result: None,
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
    const NUM_P: u8 = NUM_P;

    fn make_move(&mut self, coord: Self::Choice) {
        if self.result.is_none() {
            self.cells.insert(coord.into(), self.player);
            //self.result = Self::has_n_from((coord, self.player), K, &mut self.cells);

            self.player = (self.player + 1) % NUM_P;
            self.candidate_moves.remove(&coord);

            self.add_candidates(coord);
        } else {
            panic!(
                "Game is over, but player {} tried to make a move {coord}.",
                self.player
            );
        }
    }

    //If none, there is no result
    fn get_result(&self) -> Option<GameResult> {
        let mut not_visited = self.cells.clone();
        while let Some(first) = not_visited.pop_first() {
            if Self::has_n_from(first, K, &mut not_visited) {
                return Some(GameResult::Player(first.1));
            }
        }
        None
    }

    fn candidate_moves(&self) -> Vec<Self::Choice> {
        self.candidate_moves.iter().copied().collect()
    }

    fn is_valid(&self, choice: Self::Choice) -> bool {
        !self.cells.contains_key(&choice)
    }

    fn get_current_player(&self) -> u8 {
        self.player
    }
}

impl<const K: u8, const NUM_P: u8> KInARowState<K, NUM_P> {
    pub fn cells(&self) -> &Map {
        &self.cells
    }

    fn has_n_from((start, player): (MapCoord, u8), n: u8, cells: &mut Map) -> bool {
        const DIRS: [(MapInt, MapInt); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];

        //Check ↑ → ↗ ↖
        for dir in DIRS {
            let mut coord = start + dir;
            let mut k = 1;
            while let Some(&cell) = cells.get(&coord)
                && cell == player
            {
                k += 1;
                coord += dir;

                if k >= n {
                    return true;
                }
            }
        }
        false
    }

    //Todo: make this more efficient
    pub fn player_has_n(&self, player: u8, n: u8) -> bool {
        let mut not_visited = self
            .cells
            .iter()
            .filter_map(|(&coord, &p)| if p == player { Some((coord, p)) } else { None })
            .collect::<BTreeMap<_, _>>();

        while let Some(first) = not_visited.pop_first() {
            if Self::has_n_from(first, n, &mut not_visited) {
                return true;
            }
        }
        false
    }
    pub fn has_won_from(&self, from: MapCoord) -> bool {
        const DIRS: [(MapInt, MapInt); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];

        //Check ↑ → ↗ ↖
        for dir in DIRS {
            let mut coord = from + dir;
            let mut k = 0;
            while let Some(&cell) = self.cells.get(&coord)
                && cell == self.player
            {
                k += 1;
                coord += dir;

                if k >= K {
                    return true;
                }
            }
        }
        false
    }

    pub fn check_result(&self) -> Option<GameResult> {
        todo!()
    }

    fn add_candidates(&mut self, from: MapCoord) {
        const R: MapInt = 1;
        const NEIGHBOURS: [MapCoord; ((2 * R + 1).pow(2) - 1) as usize] = neighbour_offsets!(R);

        self.candidate_moves.extend(
            NEIGHBOURS
                .iter()
                .map(|&coord| coord + from)
                .filter(|coord| !self.cells.contains_key(coord)),
        );
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
    use super::*;
    use crate::games::coordinate::Coordinate as C;

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
    fn test_has_enough_from_horizontal_win() {
        let mut cells = BTreeMap::new();
        let first = (C(0, 0), 0);

        cells.insert(first.0, first.1);
        cells.insert(C(1, 0), 0);
        cells.insert(C(2, 0), 0);

        assert!(KIR3::has_n_from(first, 3, &mut cells));
    }

    #[test]
    fn test_has_enough_from_vertical_win() {
        let mut cells = BTreeMap::new();
        let first = (C(0, 0), 1);
        cells.insert(first.0, first.1);
        cells.insert(C(0, 1), 1);
        cells.insert(C(0, 2), 1);

        assert!(KIR3::has_n_from(first, 3, &mut cells));
    }

    #[test]
    fn test_has_enough_from_diagonal_win() {
        let mut cells = BTreeMap::new();
        let first = (C(0, 0), 0);

        cells.insert(first.0, first.1);
        cells.insert(C(1, 1), 0);
        cells.insert(C(2, 2), 0);

        assert!(KIR3::has_n_from(first, 3, &mut cells));
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
            C(4, 8),  // O
            C(8, 6),  // X
            C(2, 6),  // O
            C(6, 6),  // X
            C(3, 9),  // X  ← one extra X since X=7, O=5
        ]);

        println!("{state}");

        assert_eq!(state.get_result().unwrap(), GameResult::Player(0));
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
