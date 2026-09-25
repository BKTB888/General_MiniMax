use std::fmt::{Display, Formatter};

use arrayvec::ArrayVec;
use colored::Colorize;
use general_minimax::{
    AS_USIZE,
    coordinate::Coordinate as C,
    mixers::Splitmix,
    result::{GameResult, get_player_color},
    state::GameState,
};

/// `(col, row)`, signed so it can step off the board.
type Pos = C<isize, isize>;

#[derive(Clone, Default)]
pub struct ConnectKState<const N: u8, const M: u8, const K: u8 = 4, const NUM_P: u8 = 2> {
    cells: [[Option<u8>; AS_USIZE::<N>]; AS_USIZE::<M>] = [[None; AS_USIZE::<N>]; AS_USIZE::<M>],
    player: u8,

    choices: [u8; AS_USIZE::<M>] = [0; AS_USIZE::<M>],
    result: Option<GameResult>,
    hash: u64,
    move_stack: Vec<u8>,
}

impl<const N: u8, const M: u8, const K: u8, const NUM_P: u8> Display
    for ConnectKState<N, M, K, NUM_P>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let border =
            |left, mid, right| format!("{left}{}{right}", vec!["───"; M as usize].join(mid));
        writeln!(f, "{}", border("┌", "┬", "┐"))?;

        for row in (0..N as usize).rev() {
            write!(f, "│")?;
            for col in 0..M as usize {
                match self.cells[col][row] {
                    Some(player) => {
                        let disc = " ● ".color(get_player_color(player));
                        write!(f, "{disc}│")?;
                    }
                    None => write!(f, " · │")?,
                }
            }
            writeln!(f)?;

            if row > 0 {
                writeln!(f, "{}", border("├", "┼", "┤"))?;
            }
        }

        writeln!(f, "{}", border("└", "┴", "┘"))?;

        // Column indices
        write!(f, " ")?;
        for i in 0..M {
            write!(f, " {} ", i.to_string().bright_white().bold())?;
            if i < M - 1 {
                write!(f, " ")?;
            }
        }
        writeln!(f)?;

        Ok(())
    }
}

impl<const N: u8, const M: u8, const K: u8, const NUM_P: u8> GameState
    for ConnectKState<N, M, K, NUM_P>
{
    type Choice = u8;
    type Moves = ArrayVec<u8, { AS_USIZE::<M> }>;
    const NUM_P: u8 = NUM_P;
    const TT_BITS: u8 = 16;

    fn make_move(&mut self, choice: Self::Choice) {
        if self.result.is_none() {
            let col = choice as usize;
            let row = self.choices[col] as usize;
            self.cells[col][row] = Some(self.player);
            self.choices[col] += 1;

            self.result = self.check_result(col);

            self.hash ^= zobrist_cell_key(col as u64, row as u64, self.player as u64);

            self.player = (self.player + 1) % NUM_P;
            self.move_stack.push(choice);
        } else {
            panic!(
                "Game is over, but player {} tried to make a move {choice}.",
                self.player
            );
        }
    }

    fn get_result(&self) -> Option<GameResult> {
        self.result
    }

    fn candidate_moves(&self) -> Self::Moves {
        (0..M)
            .filter(|&choice| self.choices[choice as usize] != N)
            .collect()
    }

    fn is_valid(&self, choice: Self::Choice) -> bool {
        self.candidate_moves().contains(&choice)
    }

    fn current_player(&self) -> u8 {
        self.player
    }

    fn hash(&self) -> u64 {
        self.hash
    }

    fn ply(&self) -> u32 {
        self.move_stack.len() as u32
    }

    fn undo(&mut self) {
        let col = self.move_stack.pop().unwrap() as usize;
        self.choices[col] -= 1;
        let row = self.choices[col] as usize;
        self.cells[col][row] = None;
        self.result = None;
        self.player = self.player.checked_sub(1).unwrap_or(NUM_P - 1);
        self.hash ^= zobrist_cell_key(col as u64, row as u64, self.player as u64);
    }
}

impl<const N: u8, const M: u8, const K: u8, const NUM_P: u8> ConnectKState<N, M, K, NUM_P> {
    fn check_result(&self, col: usize) -> Option<GameResult> {
        let height = self.choices[col];
        let at = C(col as isize, height as isize - 1);

        // Nothing is above the new piece, so a vertical line is the top `K` cells of its column.
        let vertical = height >= K
            && self.cells[col][(height - K) as usize..height as usize]
                .iter()
                .all(|&cell| cell == Some(self.player));
        const AXES: [Pos; 3] = [C(1, 0), C(1, 1), C(1, -1)];
        let won = vertical
            || AXES.into_iter().any(|dir| {
                let ahead = self.run(at, dir);
                ahead == K - 1 || 1 + ahead + self.run(at, -dir) >= K
            });
        let full = self.choices.iter().all(|&h| h == N);

        won.then_some(GameResult::Player(self.player))
            .or(full.then_some(GameResult::Draw))
    }

    /// How many of `self.player`'s cells follow `at` in a row, stepping by `dir`.
    /// Stops at `K - 1`, the most a win needs.
    fn run(&self, mut at: Pos, dir: Pos) -> u8 {
        for n in 0..K - 1 {
            at += dir;
            if self.cell(at) != Some(self.player) {
                return n;
            }
        }
        K - 1
    }

    /// The owner of the cell at `(col, row)`, or `None` if it's empty or off the board.
    fn cell(&self, C(col, row): Pos) -> Option<u8> {
        *self
            .cells
            .get(usize::try_from(col).ok()?)?
            .get(usize::try_from(row).ok()?)?
    }
}

const fn zobrist_cell_key(col: u64, row: u64, player: u64) -> u64 {
    // Splitmix, not xorshift: a linear mixer lets keys XOR-cancel across cells.
    (col << 16 | row << 8 | player).splitmix()
}

#[cfg(test)]
mod tests {
    use general_minimax::{result::GameResult, state::GameState};

    use super::*;

    type C4 = ConnectKState<6, 7>; // standard Connect Four

    fn make_moves<const N: u8, const M: u8, const K: u8, const NUM_P: u8>(
        state: &mut ConnectKState<N, M, K, NUM_P>,
        moves: &[u8],
    ) {
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
        make_moves(
            &mut state,
            &[
                0, // p0 (0,0)
                1, 1, // p1 (1,0), p0 (1,1)
                2, 2, 2, // p1 (2,0), p0 (2,1) -- wait, interleave needed
            ],
        );
        // Reset and use a known working sequence
        let mut state = C4::default();
        // sequence that gives p0 an ascending diagonal at rows 0-3
        make_moves(
            &mut state,
            &[
                1, 2, 2, 3, 3, 3, // build up cols 1,2,3
                0, // p1's turn — col 0, row 0  (p1)
            ],
        );
        // Give p0 the diagonal: cols 0,1,2,3 at rows 0,1,2,3
        // Full correct sequence:
        let mut state = C4::default();
        make_moves(
            &mut state,
            &[
                0, // p0 → (0,0)
                1, // p1 → (1,0)
                1, // p0 → (1,1)
                2, // p1 → (2,0)
                2, // p0 → (2,1)
                3, // p1 → (3,0)
                2, // p0 → (2,2)
                3, // p1 → (3,1)
                3, // p0 → (3,2)
                6, // p1 → dummy
                3, // p0 → (3,3)  — ascending diagonal complete
            ],
        );
        assert_eq!(state.get_result(), Some(GameResult::Player(0)));
    }

    #[test]
    fn test_diagonal_win_descending_player0() {
        let mut state = C4::default();
        // p0 lands on (3,0),(2,1),(1,2),(0,3) — descending diagonal
        make_moves(
            &mut state,
            &[
                3, // p0 → (3,0)
                2, // p1 → (2,0)
                2, // p0 → (2,1)
                1, // p1 → (1,0)
                1, // p0 → (1,1)
                0, // p1 → (0,0)
                1, // p0 → (1,2)
                0, // p1 → (0,1)
                0, // p0 → (0,2)
                6, // p1 → dummy
                0, // p0 → (0,3) — descending diagonal complete
            ],
        );
        assert_eq!(state.get_result(), Some(GameResult::Player(0)));
    }

    // --- Draw ---

    #[test]
    fn test_draw() {
        // Use a tiny board: Connect-5 on 3×3 with 2 players (unreachable win → force full board)
        type TinyNoWin = ConnectKState<3, 3, 5>;
        let mut state = TinyNoWin::default();
        // Fill all 9 cells in an order that never produces 5 in a row (impossible on 3×3)
        for col in [0u8, 1, 2, 0, 1, 2, 0, 1, 2] {
            state.make_move(col);
        }
        assert_eq!(state.get_result(), Some(GameResult::Draw));
    }

    #[test]
    fn test_win_on_last_cell_is_not_draw() {
        type OneRow = ConnectKState<1, 3, 2>;
        let mut state = OneRow::default();
        make_moves(&mut state, &[0, 2, 1]); // p0 fills the board with (0,0),(1,0)
        assert_eq!(state.get_result(), Some(GameResult::Player(0)));
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
        type C3P = ConnectKState<6, 7, 3, 3>;
        let mut state = C3P::default();
        // turn order: p0, p1, p2, p0, p1, p2, p0, p1, p2
        make_moves(
            &mut state,
            &[
                6, 5, 0, // p0→6, p1→5, p2→(0,0)
                6, 5, 0, // p0→6, p1→5, p2→(0,1)
                6, // p0 wins
            ],
        );
        assert_eq!(state.get_result(), Some(GameResult::Player(0)));
    }

    // --- Zobrist hashing ---

    #[test]
    fn test_zobrist_changes_on_move() {
        let s0 = C4::default();
        let mut s1 = s0.clone();
        s1.make_move(3);
        assert_ne!(s0.hash, s1.hash);
    }

    #[test]
    fn test_zobrist_same_position_same_hash() {
        // Two distinct move orders that produce the identical board AND same side-to-move
        // must hash equal.
        let mut a = C4::default();
        make_moves(&mut a, &[0, 1, 0, 1]); // p0→(0,0), p1→(1,0), p0→(0,1), p1→(1,1)

        let mut b = C4::default();
        make_moves(&mut b, &[0, 1, 0, 1]); // identical sequence — sanity baseline
        assert_eq!(a.hash, b.hash);

        // Order-independent reach: p0 plays cols (0,2), p1 plays cols (1,3).
        let mut c = C4::default();
        make_moves(&mut c, &[0, 1, 2, 3]); // p0→(0,0), p1→(1,0), p0→(2,0), p1→(3,0)
        let mut d = C4::default();
        make_moves(&mut d, &[2, 3, 0, 1]); // p0→(2,0), p1→(3,0), p0→(0,0), p1→(1,0)
        assert_eq!(c.hash, d.hash);
    }

    #[test]
    fn test_zobrist_distinct_for_distinct_positions() {
        let mut a = C4::default();
        a.make_move(0);
        let mut b = C4::default();
        b.make_move(1);
        assert_ne!(a.hash, b.hash);
    }

    #[test]
    fn test_zobrist_no_xor_cancellation() {
        let mut a = C4::default();
        make_moves(&mut a, &[0, 1, 1, 0]); // p0 (0,0),(1,1); p1 (1,0),(0,1)
        let mut b = C4::default();
        make_moves(&mut b, &[1, 0, 0, 1]); // p0 (1,0),(0,1); p1 (0,0),(1,1)
        assert_ne!(a.hash, b.hash);
        assert_ne!(a.hash, C4::default().hash);
        assert_ne!(b.hash, C4::default().hash);
    }

    // --- Undo ---

    fn assert_states_eq<const N: u8, const M: u8, const K: u8, const NUM_P: u8>(
        a: &ConnectKState<N, M, K, NUM_P>,
        b: &ConnectKState<N, M, K, NUM_P>,
    ) {
        assert_eq!(a.cells, b.cells, "cells differ");
        assert_eq!(a.player, b.player, "player differs");
        assert_eq!(a.choices, b.choices, "choices differ");
        assert_eq!(a.result, b.result, "result differs");
        assert_eq!(a.hash, b.hash, "hash differs");
    }

    #[test]
    fn test_undo_single_move_restores_default() {
        let original = C4::default();
        let mut s = C4::default();
        s.make_move(3);
        s.undo();
        assert_states_eq(&s, &original);
    }

    #[test]
    fn test_undo_each_column_restores_default() {
        let original = C4::default();
        for col in 0..7u8 {
            let mut s = C4::default();
            s.make_move(col);
            s.undo();
            assert_states_eq(&s, &original);
        }
    }

    #[test]
    fn test_undo_clears_winning_result() {
        let mut s = C4::default();
        make_moves(&mut s, &[0, 6, 1, 6, 2, 6]);
        let snapshot = s.clone();
        s.make_move(3); // p0 wins horizontally
        assert_eq!(s.get_result(), Some(GameResult::Player(0)));
        s.undo();
        assert_states_eq(&s, &snapshot);
        assert_eq!(s.get_result(), None);
    }

    #[test]
    fn test_undo_chain_returns_to_default() {
        let original = C4::default();
        let mut s = C4::default();
        let moves: [u8; 8] = [0, 1, 0, 1, 2, 3, 4, 5];
        for &m in &moves {
            s.make_move(m);
        }
        for _ in &moves {
            s.undo();
        }
        assert_states_eq(&s, &original);
    }

    #[test]
    fn test_undo_nested_make_undo_pattern() {
        // Simulates an alphabeta call site:
        //   make(A); { make(B); undo(B); make(C); undo(C); } undo(A)
        // After the full sequence, state must equal the original.
        let original = C4::default();
        let mut s = C4::default();

        s.make_move(2); // outer A
        let after_outer = s.clone();

        s.make_move(3);
        s.undo(); // inner B
        assert_states_eq(&s, &after_outer);

        s.make_move(4);
        s.undo(); // inner C
        assert_states_eq(&s, &after_outer);

        s.undo(); // undo outer A
        assert_states_eq(&s, &original);
    }

    #[test]
    fn test_undo_alternating_same_column() {
        let original = C4::default();
        let mut s = C4::default();
        for _ in 0..10 {
            s.make_move(0);
            s.undo();
            assert_states_eq(&s, &original);
        }
    }

    #[test]
    fn test_undo_full_column_then_unwind() {
        // Fill column 0 to capacity, then undo all 6 moves.
        let original = C4::default();
        let mut s = C4::default();
        for _ in 0..6 {
            s.make_move(0);
        }
        for _ in 0..6 {
            s.undo();
        }
        assert_states_eq(&s, &original);
    }

    #[test]
    fn test_undo_player_wrap_2p() {
        // After a single move, current player is p1; undo must restore to p0.
        let mut s = C4::default();
        assert_eq!(s.current_player(), 0);
        s.make_move(3);
        assert_eq!(s.current_player(), 1);
        s.undo();
        assert_eq!(s.current_player(), 0);
    }

    #[test]
    fn test_undo_player_wrap_3p() {
        // 3-player: 0 → 1 → 2 → 0. Undo from p0 must wrap back to p2.
        type C3P = ConnectKState<6, 7, 4, 3>;
        let mut s = C3P::default();
        s.make_move(0); // p0
        s.make_move(1); // p1
        s.make_move(2); // p2
        assert_eq!(s.current_player(), 0);
        s.undo();
        assert_eq!(s.current_player(), 2);
        s.undo();
        assert_eq!(s.current_player(), 1);
        s.undo();
        assert_eq!(s.current_player(), 0);
    }

    #[test]
    fn test_undo_zobrist_round_trip() {
        let mut s = C4::default();
        let h0 = s.hash;
        s.make_move(3);
        assert_ne!(s.hash, h0);
        s.undo();
        assert_eq!(s.hash, h0);
    }

    #[test]
    fn test_undo_zobrist_round_trip_long_game() {
        let mut s = C4::default();
        let h0 = s.hash;
        let moves: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 0, 1, 2];
        for &m in &moves {
            s.make_move(m);
        }
        for _ in &moves {
            s.undo();
        }
        assert_eq!(s.hash, h0);
    }
}
