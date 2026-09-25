use std::{
    fmt,
    fmt::{Display, Formatter},
};

use crossterm::style::Stylize;
use general_minimax::{result::GameResult, state::GameState};

#[derive(Clone)]
pub struct MancalaState {
    board: [u8; 14],
    player: bool,
    board_stack: Vec<[u8; 14]>,
    // Moves only shift balls around, so this never changes.
    total_balls: u16,
}

impl MancalaState {
    pub fn starting(with: u8) -> Self {
        let board = [
            with, with, with, 0, with, with, with, with, with, with, 0, with, with, with,
        ];
        Self {
            board,
            player: false,
            board_stack: Vec::new(),
            total_balls: board.iter().map(|&b| u16::from(b)).sum(),
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
        self.total_balls - u16::from(self.board[3]) - u16::from(self.board[10])
    }
}

impl Default for MancalaState {
    fn default() -> Self {
        Self::starting(6)
    }
}

impl GameState for MancalaState {
    type Choice = u8;
    //TODO:
    const NUM_P: u8 = 2;

    fn make_move(&mut self, mut choice: Self::Choice) {
        self.board_stack.push(self.board);
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

        (self.get_side(true) == ZERO_ARRAY || self.get_side(false) == ZERO_ARRAY)
            .then_some(GameResult::Player(!self.player as u8))
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

    fn current_player(&self) -> u8 {
        self.player as u8
    }

    fn hash(&self) -> u64 {
        todo!()
    }

    fn undo(&mut self) {
        self.board = self
            .board_stack
            .pop()
            .expect("undo called with no move to undo");
        self.player = !self.player;
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
            .map(|colored| write!(f, "{colored}"))
            .collect::<fmt::Result>()
    }
}
