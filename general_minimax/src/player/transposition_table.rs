use std::mem::MaybeUninit;

use crate::player::evals::Score;

/// Slots a position can go in. More than 3 kept no more of what searches reuse.
const SLOTS: usize = 3;

type Bucket<C> = [Option<Slot<C>>; SLOTS];

/// Search results keyed by position hash, with `C` the game's choice type. Positions share
/// slots, so a store can push out another position's entry.
pub struct TTable<C> {
    // A new position pushes out the slot in its bucket worth least, by depth and age. Left
    // unwritten until the first store into it, since writing them all costs more than a short
    // search.
    buckets: Box<[MaybeUninit<Bucket<C>>]>,
    // One bit per bucket, set once it is written.
    written: Vec<u64>,
    generation: u32,
}

impl<C: Copy> TTable<C> {
    /// A table of `1 << bits` entries, rounded down to whole buckets, with `bits` at least 2.
    pub fn new(bits: u8) -> Self {
        let buckets = (1 << bits) / SLOTS;
        Self {
            buckets: Box::new_uninit_slice(buckets),
            written: vec![0; buckets.div_ceil(64)],
            generation: 0,
        }
    }

    /// Marks the entries stored from now on. Generations are expected to only grow, and an
    /// entry counts as one ply shallower for each generation it falls behind.
    pub fn set_generation(&mut self, generation: u32) {
        self.generation = generation;
    }

    /// The entry stored for `hash`, unless another position's store has replaced it since.
    pub fn get(&self, hash: u64) -> Option<TTEntry<C>> {
        let index = self.bucket(hash);
        if self.written[index / 64] & 1 << (index % 64) == 0 {
            return None;
        }
        // SAFETY: the bit is set only after the bucket is written.
        let bucket = unsafe { self.buckets[index].assume_init_ref() };
        bucket
            .iter()
            .flatten()
            .find(|slot| slot.key == hash)
            .map(|slot| slot.entry)
    }

    pub fn store(
        &mut self,
        hash: u64,
        depth: u8,
        value: Score,
        bound: TTBound,
        best_move: Option<C>,
    ) {
        let entry = TTEntry {
            depth,
            value,
            bound,
            best_move,
        };
        let index = self.bucket(hash);
        let bit = 1 << (index % 64);
        let bucket = if self.written[index / 64] & bit == 0 {
            self.written[index / 64] |= bit;
            self.buckets[index].write([None; SLOTS])
        } else {
            // SAFETY: the bit is set only after the bucket is written.
            unsafe { self.buckets[index].assume_init_mut() }
        };
        let generation = self.generation;
        let worth = |slot: Option<Slot<C>>| slot.map_or(i32::MIN, |slot| slot.worth(generation));
        let slot = match bucket
            .iter()
            .position(|slot| slot.is_some_and(|s| s.key == hash))
        {
            Some(own) => own,
            None => (0..SLOTS).min_by_key(|&i| worth(bucket[i])).unwrap(),
        };
        bucket[slot] = Some(Slot {
            key: hash,
            generation,
            entry,
        });
    }

    /// The index of `hash`'s bucket.
    fn bucket(&self, hash: u64) -> usize {
        // Scales the hash onto the bucket count, which isn't a power of two for a mask.
        ((hash as u128 * self.buckets.len() as u128) >> 64) as usize
    }
}

#[derive(Copy, Clone)]
struct Slot<C> {
    // The full hash, to tell the entry apart from other positions sharing the bucket.
    key: u64,
    generation: u32,
    entry: TTEntry<C>,
}

impl<C> Slot<C> {
    /// How much keeping the entry is worth while the table is at `generation`.
    fn worth(&self, generation: u32) -> i32 {
        // One ply of depth off per generation. Stockfish's 8 per search throws out deep entries
        // that later searches of the same game still reach.
        self.entry.depth as i32 - generation.saturating_sub(self.generation) as i32
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
    pub(crate) value: Score,
    pub(crate) bound: TTBound,
    /// The move to search first.
    pub(crate) best_move: Option<C>,
}
