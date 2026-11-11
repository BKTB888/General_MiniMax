// A wrapping grid of hexes: each row runs on into the next, and the last into the first, like
// a spiral around a torus. A 5x5 example:
//
//         11  12  13  14  15
//           \ / \ / \ / \ /
//       15 - 0 - 1 - 2 - 3 - 4
//         \ / \ / \ / \ / \
//      3 - 4 - 5 - 6 - 7 - 8
//       \ / \ / \ / \ / \
//    7 - 8 - 9 -10 -11 -12
//     \ / \ / \ / \ / \
// 11 -12 -13 -14 -15 - 0
//     / \ / \ / \ / \
//    0   1   2   3   4

use general_minimax::coordinate::Coordinate;

/// The index of a hex on the grid.
pub type Hex = u16;

pub const ROW_SIZE: Hex = 32;
pub const GRID_SIZE: usize = ROW_SIZE as usize * ROW_SIZE as usize;
/// The middle of the grid, where the first piece goes.
pub const START_HEX: Hex = ROW_SIZE / 2 * (ROW_SIZE + 1);
pub const GRID_MASK: Hex = GRID_SIZE as Hex - 1;

/// How far each neighbor's index is, wrapped onto the grid.
#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    NW = GRID_MASK & (ROW_SIZE + 1).wrapping_neg(),
    NE = GRID_MASK & ROW_SIZE.wrapping_neg(),
    W = GRID_MASK,
    E = 1,
    SW = ROW_SIZE,
    SE = ROW_SIZE + 1,
}

impl Direction {
    /// Clockwise from NW.
    pub const ALL: [Direction; 6] = [
        Direction::NW,
        Direction::NE,
        Direction::E,
        Direction::SE,
        Direction::SW,
        Direction::W,
    ];

    pub fn apply(self, hex: Hex) -> Hex {
        GRID_MASK & hex.wrapping_add(self as Hex)
    }
}

/// The neighbors of `hex`, clockwise from NW.
pub fn adjacent(hex: Hex) -> [Hex; 6] {
    Direction::ALL.map(|dir| dir.apply(hex))
}

/// Columns east and rows south-west of the start hex.
pub type Loc = Coordinate<i8, i8>;

pub fn loc_to_hex(Coordinate(col, row): Loc) -> Hex {
    let offset = ROW_SIZE.wrapping_mul(row as Hex).wrapping_add(col as Hex);
    START_HEX.wrapping_add(offset) & GRID_MASK
}

/// The `Loc` of `hex`, with both parts within half a row of the start hex.
pub fn hex_to_loc(hex: Hex) -> Loc {
    let half = (ROW_SIZE / 2) as i8;
    // Shifted half a row, so a hex west of the start doesn't wrap into the row above.
    let shifted = hex.wrapping_sub(START_HEX).wrapping_add(ROW_SIZE / 2) & GRID_MASK;
    let row = (shifted / ROW_SIZE) as i8;
    let col = (shifted % ROW_SIZE) as i8 - half;
    Coordinate(col, if row >= half { row - 2 * half } else { row })
}

const HEXSET_NUM_WORDS: usize = GRID_SIZE / 32;
const HEXSET_SHIFT: u32 = GRID_SIZE.trailing_zeros() - 5;
const HEXSET_MASK: usize = HEXSET_NUM_WORDS - 1;

/// A set of hexes, a bit each.
pub struct HexSet {
    table: [u32; HEXSET_NUM_WORDS],
}

impl HexSet {
    pub fn new() -> Self {
        HexSet {
            table: [0; HEXSET_NUM_WORDS],
        }
    }

    pub fn set(&mut self, hex: Hex) {
        self.table[hex as usize & HEXSET_MASK] |= 1 << (hex as u32 >> HEXSET_SHIFT);
    }

    pub fn get(&self, hex: Hex) -> bool {
        self.table[hex as usize & HEXSET_MASK] >> (hex as u32 >> HEXSET_SHIFT) & 1 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction() {
        // Each direction undoes its opposite, from anywhere.
        for hex in 0..=GRID_MASK {
            assert_eq!(hex, Direction::NE.apply(Direction::SW.apply(hex)));
            assert_eq!(hex, Direction::E.apply(Direction::W.apply(hex)));
            assert_eq!(hex, Direction::SE.apply(Direction::NW.apply(hex)));
        }

        // E and NW cross the whole grid before coming back.
        for dir in [Direction::E, Direction::NW] {
            let mut hex = START_HEX;
            for _ in 1..GRID_SIZE {
                hex = dir.apply(hex);
                assert_ne!(hex, START_HEX);
            }
            assert_eq!(dir.apply(hex), START_HEX);
        }

        // NE comes back after one row.
        let mut hex = START_HEX;
        for _ in 1..ROW_SIZE {
            hex = Direction::NE.apply(hex);
            assert_ne!(hex, START_HEX);
        }
        assert_eq!(Direction::NE.apply(hex), START_HEX);
    }

    #[test]
    fn test_hex_loc() {
        let half = (ROW_SIZE / 2) as i8;
        for col in -half..half {
            for row in -half..half {
                let loc = Coordinate(col, row);
                assert_eq!(hex_to_loc(loc_to_hex(loc)), loc);
            }
        }
    }
}
