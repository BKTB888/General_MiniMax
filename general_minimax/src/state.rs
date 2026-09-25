use std::{fmt::Display, ops::DerefMut, str::FromStr};

use crate::result::GameResult;

pub trait GameState: Default + Display + Clone + Send + Sync {
    type Choice: FromStr + Display + Copy + PartialEq + Send + Sync;
    /// Owned, so the state can change while its moves are being iterated.
    type Moves: IntoIterator<Item = Self::Choice> + DerefMut<Target = [Self::Choice]>;
    const NUM_P: u8;
    /// The transposition table holds about `1 << TT_BITS` entries, with `TT_BITS` at least 2,
    /// and is built anew for every search player, so a bigger one costs time before the first
    /// move.
    const TT_BITS: u8;
    fn make_move(&mut self, choice: Self::Choice);
    fn get_result(&self) -> Option<GameResult>;
    fn candidate_moves(&self) -> Self::Moves;
    fn is_valid(&self, choice: Self::Choice) -> bool;
    fn current_player(&self) -> u8;
    /// A Zobrist hash with all 64 bits mixed, since the transposition table uses it unhashed.
    fn hash(&self) -> u64;
    /// How many moves have been made.
    fn ply(&self) -> u32;
    fn undo(&mut self);
}
