use std::fmt::Display;
use std::str::FromStr;
use crate::result::GameResult;

pub trait GameState: Default + Display + Clone {
    type Choice: FromStr + Display + Clone + Copy;
    const NUM_P: u8;
    fn make_move(&mut self, choice: Self::Choice);
    fn get_result(&self) -> Option<GameResult>;
    fn candidate_moves(&self) -> Vec<Self::Choice>;
    fn is_valid(&self, choice: Self::Choice) -> bool;
    fn get_current_player(&self) -> u8;
}