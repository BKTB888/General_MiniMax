use std::{fmt::Display, hash::Hash, str::FromStr};

use crate::result::GameResult;

pub trait GameState: Default + Display + Clone + Send + Sync {
    type Choice: FromStr + Display + Copy + Send + Sync;
    type Hash: Hash + Copy + Eq;
    const NUM_P: u8;
    fn make_move(&mut self, choice: Self::Choice);
    fn get_result(&self) -> Option<GameResult>;
    fn candidate_moves(&self) -> Vec<Self::Choice>;
    fn is_valid(&self, choice: Self::Choice) -> bool;
    fn current_player(&self) -> u8;
    fn hash(&self) -> Self::Hash;
    fn undo_move(&mut self, choice: Self::Choice);
}
