use std::fmt::{Display, Formatter};
use colored::Colorize;
use crate::result::{get_player_color, GameResult};
use crate::state::GameState;

#[derive(Clone)]
pub struct ConnectKState<const K: u8, const N: u8, const M: u8, const NUM_P: u8> where [(); M as usize]:, [(); N as usize]: {
    cells: [[Option<u8>; N as usize]; M as usize],
    player: u8,
    choices: [u8; M as usize]
}

impl<const K: u8, const N: u8, const M: u8, const NUM_P: u8> Default for ConnectKState<K, N, M, NUM_P> where [(); M as usize]:, [(); N as usize]: {
    fn default() -> Self {
        Self {
            cells: [[None; N as usize]; M as usize],
            player: 0,
            choices: [0; M as usize]
        }
    }
}

impl<const K: u8, const N: u8, const M: u8, const NUM_P: u8>  Display for ConnectKState<K, N, M, NUM_P> where [(); N as usize]:, [(); M as usize]: {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Top border
        writeln!(f, "┌{}┐", "───┬".repeat(M as usize - 1) + "───")?;

        for row in (0..N as usize).rev() {
            write!(f, "│")?;
            for col in 0..M as usize {
                match self.cells[col][row] {
                    Some(player) => {
                        let disc = " ● ".color(get_player_color(player));
                        write!(f, "{}│", disc)?;
                    }
                    None => write!(f, " · │")?,
                }
            }
            writeln!(f)?;

            if row > 0 {
                writeln!(f, "├{}┤", "───┼".repeat(M as usize - 1) + "───")?;
            }
        }

        // Bottom border
        writeln!(f, "└{}┘", "───┴".repeat(M as usize - 1) + "───")?;

        // Column indices
        write!(f, " ")?;
        for i in 0..M {
            write!(f, " {} ", i.to_string().bright_white().bold())?;
            if i < M - 1 { write!(f, " ")?; }
        }
        writeln!(f)?;

        Ok(())
    }
}

impl<const K: u8, const N: u8, const M: u8, const NUM_P: u8>  GameState for ConnectKState<K, N, M, NUM_P> where [(); M as usize]:, [(); N as usize]:  {
    type Choice = u8;
    const NUM_P: u8 = NUM_P;

    fn make_move(&mut self, choice: Self::Choice) {
        self.cells[choice as usize][self.choices[choice as usize] as usize] = Some(self.player);
        self.choices[choice as usize] += 1;
        self.player = (self.player + 1) % NUM_P;
    }

    fn get_result(&self) -> Option<GameResult> {
        //Todo
        None
    }

    fn candidate_moves(&self) -> Vec<Self::Choice> {
        (0..M)
            .filter(|&choice| self.choices[choice as usize] != N)
            .collect()
    }

    fn is_valid(&self, choice: Self::Choice) -> bool {
        self.candidate_moves().contains(&choice)
    }

    fn get_current_player(&self) -> u8 {
        self.player
    }
}