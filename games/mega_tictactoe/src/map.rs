use std::ops::{BitAnd, BitAndAssign, BitOrAssign, Not, Shl, Shr};

use general_minimax::coordinate::Coordinate;

pub type MapInt = i16;
pub type MapCoord = Coordinate<MapInt, MapInt>;

/// One direction per line through a cell: vertical, horizontal and the two diagonals.
pub const DIRS: [MapCoord; 4] = [
    Coordinate(1, 0),
    Coordinate(0, 1),
    Coordinate(1, 1),
    Coordinate(1, -1),
];

/// The stones on an unbounded board, with every cell's four lines and the candidate moves kept
/// up to date as stones are placed and undone.
#[derive(Clone, Debug)]
pub struct Map {
    /// A `width`×`width` square, row by row, that grows when a stone nears its edge.
    cells: Vec<Cell>,
    width: usize,
    /// The coordinate of `cells[0]`.
    origin: MapCoord,
    /// Empty cells next to a stone, and the origin while it is empty.
    candidates: Vec<MapCoord>,
    /// Every stone in the order placed.
    history: Vec<Placed>,
}

impl Default for Map {
    fn default() -> Self {
        const START: usize = 32;
        let half = (START / 2) as MapInt;
        let mut map = Self {
            cells: vec![Cell::EMPTY; START * START],
            width: START,
            origin: Coordinate(-half, -half),
            candidates: Vec::new(),
            history: Vec::new(),
        };
        let origin = map.index(Coordinate(0, 0));
        map.add_candidate(origin, Coordinate(0, 0));
        map
    }
}

impl Map {
    /// Puts `player`'s stone on the empty `coord`. Returns whether it completes five in a row.
    pub fn place(&mut self, coord: MapCoord, player: u8) -> bool {
        let at = self.fit(coord);
        let was_at = self.reached(at).pos;
        if was_at != NONE {
            self.remove_candidate(at, was_at);
        }
        let before = self.candidates.len();

        self.reached(at).lines.place_center(player);
        // Per direction, the stones from four cells behind to four ahead, a 16-bit lane each.
        let mut runs = 0u64;
        for (d, stride) in self.strides().into_iter().enumerate() {
            let (mut behind, mut ahead) = (0, 0);
            for (i, k) in ALONG {
                let target = at.wrapping_add_signed(k * stride);
                let line = self.reached(target).lines.place(d, i, player);
                match k {
                    // The lines centered two cells away reach four past the stone.
                    -2 => behind = line.stones_of(player),
                    2 => ahead = line.stones_of(player),
                    _ => self.near_placed(target, coord + DIRS[d] * k as MapInt),
                }
            }
            let nine = behind | ahead << (WIDTH - 1);
            runs |= (nine as u64) << (16 * d);
        }
        let five = five_in_a_lane(runs);

        let added = (self.candidates.len() - before) as u8;
        self.history.push(Placed {
            coord,
            player,
            was_at,
            added,
        });
        five
    }

    /// Takes the last stone placed back off. Returns where it was.
    pub fn undo(&mut self) -> MapCoord {
        let Placed {
            coord,
            was_at,
            added,
            ..
        } = self.history.pop().expect("no stone to undo");
        let at = self.index(coord);
        // The neighbours this stone made candidates went on the end, and everything added after
        // them has been undone already.
        for _ in 0..added {
            let neighbour = self.candidates.pop().unwrap();
            let n = self.index(neighbour);
            self.cells[n].pos = NONE;
        }

        self.cells[at].lines.clear_center();
        for (d, stride) in self.strides().into_iter().enumerate() {
            for (i, k) in ALONG {
                let target = at.wrapping_add_signed(k * stride);
                self.reached(target).lines.clear(d, i);
            }
        }

        if was_at != NONE {
            self.restore_candidate(at, coord, was_at);
        }
        coord
    }

    /// The player whose stone is on `coord`, if any.
    pub fn get(&self, coord: MapCoord) -> Option<u8> {
        self.cells[self.try_index(coord)?].lines.owner()
    }

    /// Empty cells next to a stone, and the origin while it is empty.
    pub fn candidates(&self) -> &[MapCoord] {
        &self.candidates
    }

    /// Every stone, as (coordinate, owner), in the order placed.
    pub fn stones(&self) -> impl Iterator<Item = (MapCoord, u8)> {
        self.history
            .iter()
            .map(|placed| (placed.coord, placed.player))
    }

    /// Makes the cell at `at`, next to a new stone, a candidate if it is empty and not one yet.
    fn near_placed(&mut self, at: usize, coord: MapCoord) {
        let cell = self.reached(at);
        if cell.pos == NONE && cell.lines.owner().is_none() {
            self.add_candidate(at, coord);
        }
    }

    fn add_candidate(&mut self, at: usize, coord: MapCoord) {
        debug_assert!(self.candidates.len() < NONE as usize);
        self.cells[at].pos = self.candidates.len() as u16;
        self.candidates.push(coord);
    }

    /// Takes the candidate at `pos` out of the list, moving the last one into its place.
    fn remove_candidate(&mut self, at: usize, pos: u16) {
        self.candidates.swap_remove(pos as usize);
        self.reached(at).pos = NONE;
        if let Some(&moved) = self.candidates.get(pos as usize) {
            let m = self.index(moved);
            self.cells[m].pos = pos;
        }
    }

    /// Puts `coord`, at `at`, back at `pos` in the list, undoing `remove_candidate(at, pos)`.
    fn restore_candidate(&mut self, at: usize, coord: MapCoord, pos: u16) {
        if let Some(&moved) = self.candidates.get(pos as usize) {
            let m = self.index(moved);
            self.cells[m].pos = self.candidates.len() as u16;
            self.candidates.push(moved);
            self.candidates[pos as usize] = coord;
        } else {
            self.candidates.push(coord);
        }
        self.cells[at].pos = pos;
    }

    /// The cell at `at`, which must be in the lines of a stone on the board.
    fn reached(&mut self, at: usize) -> &mut Cell {
        debug_assert!(at < self.cells.len());
        // SAFETY: `fit` grows the grid to hold every cell in a stone's lines before placing it,
        // and the grid never shrinks.
        unsafe { self.cells.get_unchecked_mut(at) }
    }

    /// How far apart neighbouring cells along each of `DIRS` are in `cells`.
    fn strides(&self) -> [isize; 4] {
        DIRS.map(|Coordinate(dr, dc)| dr as isize * self.width as isize + dc as isize)
    }

    /// Where `coord`, which must be inside the grid, is in `cells`.
    fn index(&self, coord: MapCoord) -> usize {
        let (r, c) = self.offset(coord);
        debug_assert!(r < self.width && c < self.width, "{coord} is off the grid");
        r * self.width + c
    }

    /// Where `coord` is in `cells`, if it is inside the grid.
    fn try_index(&self, coord: MapCoord) -> Option<usize> {
        let (r, c) = self.offset(coord);
        (r < self.width && c < self.width).then(|| r * self.width + c)
    }

    /// `coord`'s row and column in the grid. Off the grid, one of them is `width` or more.
    fn offset(&self, Coordinate(r, c): MapCoord) -> (usize, usize) {
        // Wraps a negative offset to a huge one, so a single `<` catches both sides.
        let along = |x: MapInt, origin: MapInt| (x as i32 - origin as i32) as usize;
        (along(r, self.origin.0), along(c, self.origin.1))
    }

    /// Where `coord` is in `cells`, growing the grid first if a stone there would have lines
    /// reaching past its edge.
    fn fit(&mut self, coord: MapCoord) -> usize {
        let (r, c) = self.offset(coord);
        let reach = CENTER as usize;
        let room = self.width - 2 * reach;
        // An offset under `reach` wraps around to a huge one, so one compare covers both edges.
        if r.wrapping_sub(reach) < room && c.wrapping_sub(reach) < room {
            r * self.width + c
        } else {
            self.grow(coord);
            self.index(coord)
        }
    }

    /// Replaces the grid with one twice as wide as the old grid and `coord`'s lines need
    /// together, with both in the middle.
    fn grow(&mut self, Coordinate(r, c): MapCoord) {
        let reach = CENTER as i32;
        let (top, left) = (self.origin.0 as i32, self.origin.1 as i32);
        let last = self.width as i32 - 1;
        let (min_r, max_r) = (
            top.min(r as i32 - reach),
            (top + last).max(r as i32 + reach),
        );
        let (min_c, max_c) = (
            left.min(c as i32 - reach),
            (left + last).max(c as i32 + reach),
        );
        let width = 2 * (max_r - min_r).max(max_c - min_c) + 2;
        let origin = Coordinate(
            ((min_r + max_r - width) / 2) as MapInt,
            ((min_c + max_c - width) / 2) as MapInt,
        );

        let width = width as usize;
        let mut cells = vec![Cell::EMPTY; width * width];
        let (down, right) = (
            (top - origin.0 as i32) as usize,
            (left - origin.1 as i32) as usize,
        );
        for (row, old) in self.cells.chunks_exact(self.width).enumerate() {
            let start = (down + row) * width + right;
            cells[start..start + self.width].copy_from_slice(old);
        }
        (self.cells, self.width, self.origin) = (cells, width, origin);
    }

    /// Whether every cell of `self` matches the cell on the same coordinate in `other`,
    /// counting cells off `other`'s grid as empty.
    fn cells_match(&self, other: &Self) -> bool {
        self.cells.iter().enumerate().all(|(i, cell)| {
            let (row, col) = ((i / self.width) as MapInt, (i % self.width) as MapInt);
            let coord = self.origin + (row, col);
            *cell
                == other
                    .try_index(coord)
                    .map_or(Cell::EMPTY, |j| other.cells[j])
        })
    }
}

/// Equal when they hold the same stones placed in the same order, whatever size each grid has
/// grown to.
impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        self.history == other.history
            && self.candidates == other.candidates
            && self.cells_match(other)
            && other.cells_match(self)
    }
}

/// For each line through a stone but its own cell's: the stone is cell `i` of the line
/// centered `k = CENTER - i` steps along the line's direction.
const ALONG: [(u8, isize); 4] = [(0, 2), (1, 1), (3, -1), (4, -2)];

/// Whether any 16-bit lane of `runs` has five bits set in a row. Only the low nine bits of each
/// lane may be set.
fn five_in_a_lane(runs: u64) -> bool {
    // A lane's top bits are clear, so bits shifted in from the next lane die in the first `&`.
    runs & runs >> 1 & runs >> 2 & runs >> 3 & runs >> 4 != 0
}

/// `Cell::pos` and `Placed::was_at` for a cell that is not a candidate.
const NONE: u16 = u16::MAX;

#[derive(Clone, Copy, PartialEq, Debug)]
struct Cell {
    lines: FourLines,
    /// Where this cell is in `Map::candidates`, or `NONE`.
    pos: u16,
}
impl Cell {
    const EMPTY: Self = Self {
        lines: FourLines([Line(Bits(0)); 4]),
        pos: NONE,
    };
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Placed {
    coord: MapCoord,
    player: u8,
    /// Where the stone's cell was in `Map::candidates`, or `NONE`.
    was_at: u16,
    /// How many of its neighbours the stone made candidates.
    added: u8,
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
struct FourLines([Line; 4]);
impl FourLines {
    /// Sets cell `i` of the line in direction `d`. Returns the line as it is now.
    fn place(&mut self, d: usize, i: u8, player: u8) -> Line {
        let line = &mut self.0[d];
        line.set(i, player);
        *line
    }
    /// Puts `player` on this cell's own coordinate.
    fn place_center(&mut self, player: u8) {
        for line in &mut self.0 {
            line.set(CENTER, player);
        }
    }
    /// The player on this cell's own coordinate, if any.
    fn owner(&self) -> Option<u8> {
        // Every line is centered on the cell's coordinate, so any of them will do.
        self.0[0].get(CENTER)
    }
    fn clear_center(&mut self) {
        for line in &mut self.0 {
            line.clear(CENTER);
        }
    }
    fn clear(&mut self, d: usize, i: u8) {
        self.0[d].clear(i);
    }
}

/// Cells per line. Player `p`'s stones are bits `WIDTH * p ..WIDTH * (p + 1)`.
const WIDTH: u8 = 5;
/// The cell of a line that its own coordinate sits on.
const CENTER: u8 = WIDTH / 2;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
struct Line(Bits<u16, 10>);
impl Line {
    fn set(&mut self, i: u8, player: u8) {
        debug_assert!(i < WIDTH);
        self.0.set(i + WIDTH * player);
    }
    fn clear(&mut self, i: u8) {
        debug_assert!(i < WIDTH);
        self.0.clear(i);
        self.0.clear(i + WIDTH);
    }
    /// The player owning cell `i`, if any.
    fn get(&self, i: u8) -> Option<u8> {
        debug_assert!(i < WIDTH);
        (0..2).find(|&player| self.0.get(i + WIDTH * player))
    }
    /// The cells holding `player`'s stones, as the low `WIDTH` bits.
    fn stones_of(self, player: u8) -> u16 {
        const ALL: u16 = (1 << WIDTH) - 1;
        self.0.0 >> (WIDTH * player) & ALL
    }
}

const BITS_OF<T>: u8 = (size_of::<T>() * 8) as u8;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
struct Bits<T = u32, const N: u8 = { BITS_OF::<T> }>(T);
impl<T, const N: u8> Bits<T, N>
where
    T: Copy
        + PartialEq
        + From<u8>
        + Shl<u8, Output = T>
        + Shr<u8, Output = T>
        + BitAnd<Output = T>
        + Not<Output = T>
        + BitOrAssign
        + BitAndAssign,
{
    fn set(&mut self, i: u8) {
        debug_assert!(i < N);
        self.0 |= T::from(1) << i;
    }
    fn get(&self, i: u8) -> bool {
        debug_assert!(i < N);
        self.0 >> i & T::from(1) == T::from(1)
    }
    fn clear(&mut self, i: u8) {
        debug_assert!(i < N);
        self.0 &= !(T::from(1) << i);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashMap};

    use general_minimax::mixers::Splitmix;

    use super::*;

    #[test]
    fn five_across_where_the_grid_grows() {
        let mut map = Map::default();
        let width = map.width;
        for col in 12..16 {
            assert!(!map.place(Coordinate(0, col), 0));
        }
        assert!(map.width > width, "a stone near the edge grows the grid");
        assert!(map.place(Coordinate(0, 16), 0));
    }

    #[test]
    fn five_in_every_direction_whichever_stone_completes_it() {
        for dir in DIRS {
            for last in 0..5 {
                let mut map = Map::default();
                let start = Coordinate(3, -2);
                for k in (0..5).filter(|&k| k != last) {
                    assert!(!map.place(start + dir * k, 0));
                }
                // The other player's stones on both ends change nothing.
                map.place(start + -dir, 1);
                map.place(start + dir * 5, 1);
                assert!(
                    map.place(start + dir * last, 0),
                    "{dir} completed at {last}"
                );
            }
        }
    }

    #[test]
    fn a_gap_or_the_other_players_stone_breaks_a_five() {
        for dir in DIRS {
            let mut map = Map::default();
            for k in [0, 1, 2, 4, 5] {
                assert!(!map.place(dir * k, 0));
            }
            map.place(dir * 3, 1);
            assert!(!map.place(dir * 6, 0), "{dir}");
        }
    }

    #[test]
    fn undo_after_a_growth_restores_the_map() {
        let mut map = Map::default();
        map.place(Coordinate(0, 0), 0);
        map.place(Coordinate(1, 1), 1);
        let before = map.clone();
        map.place(Coordinate(-40, 25), 0);
        assert_eq!(map.undo(), Coordinate(-40, 25));
        assert_eq!(map, before);
    }

    /// The same board kept the slow, obvious way.
    #[derive(Default)]
    struct Naive(HashMap<MapCoord, u8>);
    impl Naive {
        fn five_through(&self, coord: MapCoord, player: u8) -> bool {
            DIRS.iter().any(|&dir| {
                let run = |step: MapCoord| {
                    (1..5)
                        .take_while(|&k| self.0.get(&(coord + step * k)) == Some(&player))
                        .count()
                };
                run(dir) + run(-dir) + 1 >= 5
            })
        }
        fn candidates(&self) -> BTreeSet<MapCoord> {
            let near = self.0.keys().flat_map(|&stone| {
                DIRS.iter()
                    .flat_map(move |&dir| [stone + dir, stone + -dir])
            });
            near.chain([Coordinate(0, 0)])
                .filter(|coord| !self.0.contains_key(coord))
                .collect()
        }
    }

    #[test]
    fn matches_a_naive_board_through_random_play() {
        let mut seed = 0u64;
        let mut random = move |below: usize| {
            seed += 1;
            (seed.splitmix() % below as u64) as usize
        };
        for _ in 0..20 {
            let (mut map, mut naive) = (Map::default(), Naive::default());
            // The candidates before each stone still on the board, to check undo's order.
            let mut before = Vec::new();
            for _ in 0..300 {
                if !map.history.is_empty() && random(3) == 0 {
                    naive.0.remove(&map.undo());
                    assert_eq!(map.candidates, before.pop().unwrap());
                } else {
                    // Mostly next to the stones, sometimes far off to make the grid grow.
                    let coord = if random(10) == 0 {
                        Coordinate(random(121) as MapInt - 60, random(121) as MapInt - 60)
                    } else {
                        map.candidates[random(map.candidates.len())]
                    };
                    if naive.0.contains_key(&coord) {
                        continue;
                    }
                    let player = (map.history.len() % 2) as u8;
                    if random(20) == 0 {
                        let unplaced = map.clone();
                        map.place(coord, player);
                        map.undo();
                        assert_eq!(map, unplaced);
                    }
                    before.push(map.candidates.clone());
                    let five = map.place(coord, player);
                    naive.0.insert(coord, player);
                    assert_eq!(
                        five,
                        naive.five_through(coord, player),
                        "five through {coord}"
                    );
                }

                let expected = naive.candidates();
                assert_eq!(
                    map.candidates.len(),
                    expected.len(),
                    "a candidate is listed twice"
                );
                assert_eq!(
                    map.candidates.iter().copied().collect::<BTreeSet<_>>(),
                    expected
                );
                for (pos, &coord) in map.candidates.iter().enumerate() {
                    assert_eq!(map.cells[map.index(coord)].pos as usize, pos);
                }
                for (&coord, &player) in &naive.0 {
                    assert_eq!(map.get(coord), Some(player));
                }
            }
        }
    }
}
