use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hasher},
    ops::{BitAnd, BitAndAssign, BitOrAssign, Not, Shl, Shr},
};

use general_minimax::{coordinate::Coordinate, mixers::Splitmix};

/// `HashMap` with a fixed seed, so iteration order is the same on every run.
pub(crate) type FixedMap<K, V> = HashMap<K, V, BuildHasherDefault<CoordHasher>>;

/// Hashes a `MapCoord` by packing its two `i16`s into one word and mixing it once.
#[derive(Default)]
pub(crate) struct CoordHasher(u64);

impl Hasher for CoordHasher {
    fn finish(&self) -> u64 {
        // The map picks buckets from the top bits, which a bare packed coordinate leaves empty.
        self.0.splitmix()
    }

    fn write(&mut self, _: &[u8]) {
        unreachable!("the map only hashes MapCoord keys");
    }

    fn write_i16(&mut self, i: i16) {
        self.0 = self.0 << 16 | i as u16 as u64;
    }
}

pub type MapInt = i16;
pub type MapCoord = Coordinate<MapInt, MapInt>;

/// One direction per line through a cell: vertical, horizontal and the two diagonals.
pub const DIRS: [MapCoord; 4] = [
    Coordinate(1, 0),
    Coordinate(0, 1),
    Coordinate(1, 1),
    Coordinate(1, -1),
];

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Map {
    cells: FixedMap<MapCoord, FourLines>,
}
impl Map {
    /// Puts `player`'s stone on `coord`. Returns whether it completes five in a row.
    pub fn place(&mut self, coord: MapCoord, player: u8) -> bool {
        let mut five = false;
        for (d, &dir) in DIRS.iter().enumerate() {
            // `coord` is cell `i` of the line centered `CENTER - i` steps along `dir`.
            for i in 0..WIDTH {
                let center = coord + dir * (CENTER as MapInt - i as MapInt);
                five |= self.cells.entry(center).or_default().place(d, i, player);
            }
        }
        five
    }
    /// The player whose stone is on `coord`, if any.
    pub fn get(&self, coord: MapCoord) -> Option<u8> {
        self.cells.get(&coord)?.owner()
    }
    /// Every stone, as (coordinate, owner), in no particular order.
    pub fn stones(&self) -> impl Iterator<Item = (MapCoord, u8)> {
        self.cells
            .iter()
            .filter_map(|(&coord, lines)| Some((coord, lines.owner()?)))
    }
    /// Takes the stone off `coord`, which must hold one.
    pub fn remove(&mut self, coord: MapCoord) {
        for (d, &dir) in DIRS.iter().enumerate() {
            for i in 0..WIDTH {
                let center = coord + dir * (CENTER as MapInt - i as MapInt);
                let lines = self.cells.get_mut(&center).unwrap();
                lines.clear(d, i);
                // Keeps the map sparse, and equal to how it was before the stone.
                if lines.is_empty() {
                    self.cells.remove(&center);
                }
            }
        }
    }
}

#[derive(Clone, Default, PartialEq, Debug)]
struct FourLines([Line; 4]);
impl FourLines {
    /// Sets cell `i` of the line in direction `d`. Returns whether that line is now five of `player`.
    fn place(&mut self, d: usize, i: u8, player: u8) -> bool {
        let line = &mut self.0[d];
        line.set(i, player);
        line.is_five(player)
    }
    /// The player on this entry's own coordinate, if any.
    fn owner(&self) -> Option<u8> {
        // Every line is centered on the entry's coordinate, so any of them will do.
        self.0[0].get(CENTER)
    }
    fn clear(&mut self, d: usize, i: u8) {
        self.0[d].clear(i);
    }
    fn is_empty(&self) -> bool {
        self.0.iter().all(Line::is_empty)
    }
}

/// Cells per line. Player `p`'s stones are bits `WIDTH * p ..WIDTH * (p + 1)`.
const WIDTH: u8 = 5;
/// The cell of a line that its own coordinate sits on.
const CENTER: u8 = WIDTH / 2;

#[derive(Clone, Default, PartialEq, Debug)]
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
    fn is_five(&self, player: u8) -> bool {
        const FULL: u16 = (1 << WIDTH) - 1;
        self.0.0 >> (WIDTH * player) & FULL == FULL
    }
    fn is_empty(&self) -> bool {
        self.0.0 == 0
    }
}

const BITS_OF<T>: u8 = (size_of::<T>() * 8) as u8;

#[derive(Clone, Default, PartialEq, Debug)]
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
