use crate::result::GameResult;
use crate::state::GameState;
use crossterm::style::Stylize;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct MancalaState {
    board: [u8; 14],
    player: bool,
}

impl MancalaState {
    pub fn starting(with: u8) -> Self {
        Self {
            board: [
                with, with, with, 0, with, with, with, with, with, with, 0, with, with, with,
            ],
            player: false,
        }
    }
    const fn get_side_idxs(player: bool) -> [u8; 6] {
        if player {
            [7, 8, 9, 11, 12, 13]
        } else {
            [0, 1, 2, 4, 5, 6]
        }
    }

    fn get_side(&self, player: bool) -> [u8; 6] {
        Self::get_side_idxs(player).map(|idx| self.board[idx as usize])
    }

    pub fn opponent_side(&self) -> [u8; 6] {
        self.get_side(!self.player)
    }

    pub fn current_side(&self) -> [u8; 6] {
        self.get_side(self.player)
    }

    pub fn balls_in_play(&self) -> u16 {
        self.get_side(self.player)
            .iter()
            .map(|&x| x as u16)
            .sum::<u16>()
            + self
                .get_side(!self.player)
                .iter()
                .map(|&x| x as u16)
                .sum::<u16>()
    }
}

impl Default for MancalaState {
    fn default() -> Self {
        Self::starting(6)
    }
}

impl GameState for MancalaState {
    type Choice = u8;
    const NUM_P: u8 = 2;

    fn make_move(&mut self, mut choice: Self::Choice) {
        loop {
            let mut hand = self.board[choice as usize];
            self.board[choice as usize] = 0;

            while hand > 0 {
                choice += 1;
                choice %= 14;
                self.board[choice as usize] += 1;

                hand -= 1;
            }

            //Is in an ending
            if choice == 3 || choice == 10 || self.board[choice as usize] == 1 {
                break;
            }
        }
        self.player = !self.player;
    }

    fn get_result(&self) -> Option<GameResult> {
        const ZERO_ARRAY: [u8; 6] = [0u8; 6];

        if self.get_side(true) == ZERO_ARRAY || self.get_side(false) == ZERO_ARRAY {
            Some(GameResult::Player(!self.player as u8))
        } else {
            None
        }
    }

    fn candidate_moves(&self) -> Vec<Self::Choice> {
        Self::get_side_idxs(self.player)
            .into_iter()
            .filter(|&idx| self.board[idx as usize] != 0)
            .collect()
    }

    fn is_valid(&self, choice: Self::Choice) -> bool {
        self.candidate_moves().contains(&choice)
    }

    fn get_current_player(&self) -> u8 {
        self.player as u8
    }
}

impl Display for MancalaState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.board
            .iter()
            .map(|&num| num.to_string() + " ")
            .enumerate()
            .map(|(index, value)| match index {
                0..3 | 4..7 => value.red(),
                7..10 | 11..14 => value.green(),
                3 | 10 => value.blue().bold(),
                _ => value.black(),
            })
            .map(|colored| write!(f, "{}", colored))
            .collect::<fmt::Result>()
    }
}
