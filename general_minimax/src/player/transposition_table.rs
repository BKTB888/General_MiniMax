use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hasher},
};

use crate::player::search::EvalResult;

/// Search results keyed by position hash, with `C` the game's choice type.
pub struct TTable<C> {
    // Keys are already mixed Zobrist hashes, so the map uses them as they are.
    entries: HashMap<u64, TTEntry<C>, BuildHasherDefault<PassHasher>>,
}

impl<C: Copy> TTable<C> {
    pub fn new() -> Self {
        Self {
            entries: HashMap::default(),
        }
    }

    /// The entry stored for `hash`.
    pub fn get(&self, hash: u64) -> Option<TTEntry<C>> {
        self.entries.get(&hash).copied()
    }

    pub fn store(
        &mut self,
        hash: u64,
        depth: u8,
        value: EvalResult,
        bound: TTBound,
        best_move: Option<C>,
    ) {
        self.entries.insert(
            hash,
            TTEntry {
                depth,
                value,
                bound,
                best_move,
            },
        );
    }
}

/// Hands a `u64` key to the map unchanged.
#[derive(Default)]
struct PassHasher(u64);

impl Hasher for PassHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        unreachable!("the table only hashes u64 keys");
    }

    fn write_u64(&mut self, key: u64) {
        self.0 = key;
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TTBound {
    Exact,
    Lower,
    Upper,
}
#[derive(Copy, Clone, Debug)]
pub struct TTEntry<C> {
    pub(crate) depth: u8,
    pub(crate) value: EvalResult,
    pub(crate) bound: TTBound,
    /// The move to search first.
    pub(crate) best_move: Option<C>,
}
