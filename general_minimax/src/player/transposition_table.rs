use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hasher},
};

use crate::player::search::EvalResult;

pub struct TTable {
    // Keys are already mixed Zobrist hashes, so the map uses them as they are.
    entries: HashMap<u64, TTEntry, BuildHasherDefault<PassHasher>>,
}

impl TTable {
    pub fn new() -> Self {
        Self {
            entries: HashMap::default(),
        }
    }

    /// The entry for `hash` if it was searched at least `depth` deep.
    pub fn get(&self, hash: u64, depth: u8) -> Option<TTEntry> {
        self.entries
            .get(&hash)
            .copied()
            .filter(|entry| entry.depth >= depth)
    }

    pub fn store(&mut self, hash: u64, depth: u8, value: EvalResult, bound: TTBound) {
        self.entries.insert(
            hash,
            TTEntry {
                depth,
                value,
                bound,
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
pub struct TTEntry {
    depth: u8,
    pub(crate) value: EvalResult,
    pub(crate) bound: TTBound,
}
