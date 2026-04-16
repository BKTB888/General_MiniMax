use std::fmt::{Display, Formatter};
use colored::Colorize;
use crate::result::{get_player_color, GameResult};
use crate::state::GameState;

#[derive(Clone)]
pub struct ConnectKState<const K: u8, const N: u8, const M: u8, const NUM_P: u8> where [(); M as usize]:, [(); N as usize]: {
    cells: [[Option<u8>; N as usize]; M as usize],
    player: u8,
    choices: [u8; M as usize],
    result: Option<GameResult>
}

impl<const K: u8, const N: u8, const M: u8, const NUM_P: u8> Default for ConnectKState<K, N, M, NUM_P> where [(); M as usize]:, [(); N as usize]: {
    fn default() -> Self {
        Self {
            cells: [[None; N as usize]; M as usize],
            player: 0,
            choices: [0; M as usize],
            result: None,
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
        if self.result.is_none() {
            self.cells[choice as usize][self.choices[choice as usize] as usize] = Some(self.player);
            self.choices[choice as usize] += 1;
            self.player = (self.player + 1) % NUM_P;
            self.result = self.check_win();
        } else {
            panic!("Game is over, but player {} tried to make a move {choice}.", self.player);
        }
    }

    fn get_result(&self) -> Option<GameResult> {
        self.result
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

impl<const K: u8, const N: u8, const M: u8, const NUM_P: u8> ConnectKState<K, N, M, NUM_P> where [(); M as usize]:, [(); N as usize]: {
    fn check_win(&self) -> Option<GameResult> {
        let last_player = (self.player + NUM_P - 1) % NUM_P;

        const DIRS: [(i16, i16); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for col in 0..M as usize {
            let row = self.choices[col] as usize;
            if row == 0 { continue; }
            let row = row - 1;

            if self.cells[col][row] != Some(last_player) { continue; }

            for (dc, dr) in DIRS {
                let mut count = 1u8;

                let (mut c, mut r) = (col as i16 + dc, row as i16 + dr);
                while c >= 0 && c < M as i16 && r >= 0 && r < N as i16
                    && self.cells[c as usize][r as usize] == Some(last_player)
                {
                    count += 1;
                    c += dc;
                    r += dr;
                }

                let (mut c, mut r) = (col as i16 - dc, row as i16 - dr);
                while c >= 0 && c < M as i16 && r >= 0 && r < N as i16
                    && self.cells[c as usize][r as usize] == Some(last_player)
                {
                    count += 1;
                    c -= dc;
                    r -= dr;
                }

                if count >= K {
                    return Some(GameResult::Player(last_player));
                }
            }
        }

        if self.choices.iter().all(|&h| h == N) {
            return Some(GameResult::Draw);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::GameResult;
    use crate::state::GameState;

    type C4 = ConnectKState<4, 6, 7, 2>; // standard Connect Four

    fn make_moves<const K: u8, const N: u8, const M: u8, const NUM_P: u8>(
        state: &mut ConnectKState<K, N, M, NUM_P>,
        moves: &[u8],
    ) where
        [(); M as usize]:,
        [(); N as usize]:,
    {
        for &m in moves {
            state.make_move(m);
        }
    }

    // --- None cases ---

    #[test]
    fn test_initial_state_no_result() {
        let state = C4::default();
        assert_eq!(state.get_result(), None);
    }

    #[test]
    fn test_mid_game_no_result() {
        let mut state = C4::default();
        make_moves(&mut state, &[0, 1, 2, 3]);
        assert_eq!(state.get_result(), None);
    }

    #[test]
    fn test_k_minus_one_in_a_row_no_result() {
        // Player 0 has 3 in a row but not 4
        let mut state = C4::default();
        // p0: cols 0,1,2   p1: col 6,6,6
        make_moves(&mut state, &[0, 6, 1, 6, 2, 6]);
        assert_eq!(state.get_result(), None);
    }

    // --- Horizontal wins ---

    #[test]
    fn test_horizontal_win_player0() {
        let mut state = C4::default();
        // p0 fills cols 0,1,2,3; p1 fills col 6 between each
        make_moves(&mut state, &[0, 6, 1, 6, 2, 6, 3]);
        assert_eq!(state.get_result(), Some(GameResult::Player(0)));
    }

    #[test]
    fn test_horizontal_win_player1() {
        let mut state = C4::default();
        // p1 fills cols 0,1,2,3; p0 fills col 6
        make_moves(&mut state, &[6, 0, 6, 1, 6, 2, 5, 3]);
        assert_eq!(state.get_result(), Some(GameResult::Player(1)));
    }

    // --- Vertical win ---

    #[test]
    fn test_vertical_win_player0() {
        let mut state = C4::default();
        // p0 stacks col 0 four times; p1 uses col 1
        make_moves(&mut state, &[0, 1, 0, 1, 0, 1, 0]);
        assert_eq!(state.get_result(), Some(GameResult::Player(0)));
    }

    #[test]
    fn test_vertical_win_player1() {
        let mut state = C4::default();
        // p1 stacks col 0 four times; p0 leads with col 1
        make_moves(&mut state, &[1, 0, 1, 0, 1, 0, 2, 0]);
        assert_eq!(state.get_result(), Some(GameResult::Player(1)));
    }

    // --- Diagonal wins ---

    #[test]
    fn test_diagonal_win_ascending_player0() {
        let mut state = C4::default();
        // Build staircase so p0 lands on (0,0),(1,1),(2,2),(3,3)
        // p0 plays cols 0,1,2,3 in order; p1 fills beneath
        make_moves(&mut state, &[
            0,          // p0 (0,0)
            1, 1,       // p1 (1,0), p0 (1,1)
            2, 2, 2,    // p1 (2,0), p0 (2,1) -- wait, interleave needed
        ]);
        // Reset and use a known working sequence
        let mut state = C4::default();
        // sequence that gives p0 an ascending diagonal at rows 0-3
        make_moves(&mut state, &[
            1, 2, 2, 3, 3, 3, // build up cols 1,2,3
            0,                 // p1's turn — col 0, row 0  (p1)
        ]);
        // Give p0 the diagonal: cols 0,1,2,3 at rows 0,1,2,3
        // Full correct sequence:
        let mut state = C4::default();
        make_moves(&mut state, &[
            0,       // p0 → (0,0)
            1,       // p1 → (1,0)
            1,       // p0 → (1,1)
            2,       // p1 → (2,0)
            2,       // p0 → (2,1)
            3,       // p1 → (3,0)
            2,       // p0 → (2,2)
            3,       // p1 → (3,1)
            3,       // p0 → (3,2)
            6,       // p1 → dummy
            3,       // p0 → (3,3)  — ascending diagonal complete
        ]);
        assert_eq!(state.get_result(), Some(GameResult::Player(0)));
    }

    #[test]
    fn test_diagonal_win_descending_player0() {
        let mut state = C4::default();
        // p0 lands on (3,0),(2,1),(1,2),(0,3) — descending diagonal
        make_moves(&mut state, &[
            3,       // p0 → (3,0)
            2,       // p1 → (2,0)
            2,       // p0 → (2,1)
            1,       // p1 → (1,0)
            1,       // p0 → (1,1)
            0,       // p1 → (0,0)
            1,       // p0 → (1,2)
            0,       // p1 → (0,1)
            0,       // p0 → (0,2)
            6,       // p1 → dummy
            0,       // p0 → (0,3) — descending diagonal complete
        ]);
        assert_eq!(state.get_result(), Some(GameResult::Player(0)));
    }

    // --- Draw ---

    #[test]
    fn test_draw() {
        // Use a tiny board: Connect-5 on 3×3 with 2 players (unreachable win → force full board)
        type TinyNoWin = ConnectKState<5, 3, 3, 2>;
        let mut state = TinyNoWin::default();
        // Fill all 9 cells in an order that never produces 5 in a row (impossible on 3×3)
        for col in [0u8, 1, 2, 0, 1, 2, 0, 1, 2] {
            state.make_move(col);
        }
        assert_eq!(state.get_result(), Some(GameResult::Draw));
    }

    // --- Result is sticky after game ends ---

    #[test]
    fn test_result_sticky_after_win() {
        let mut state = C4::default();
        make_moves(&mut state, &[0, 6, 1, 6, 2, 6, 3]); // p0 wins
        let result = state.get_result();
        assert_eq!(result, Some(GameResult::Player(0)));
        // calling get_result again returns the same value
        assert_eq!(state.get_result(), result);
    }

    // --- Multiplayer (3-player) ---

    #[test]
    fn test_three_player_win() {
        type C3P = ConnectKState<3, 6, 7, 3>;
        let mut state = C3P::default();
        // turn order: p0, p1, p2, p0, p1, p2, p0, p1, p2
        make_moves(&mut state, &[
            6, 5, 0,  // p0→6, p1→5, p2→(0,0)
            6, 5, 0,  // p0→6, p1→5, p2→(0,1)
            6         // p0 wins
        ]);
        assert_eq!(state.get_result(), Some(GameResult::Player(0)));
    }
}