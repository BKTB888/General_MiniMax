#![feature(
    generic_const_args,
    min_generic_const_args,
    macroless_generic_const_args
)]
#![allow(incomplete_features)]

use std::fmt::{self, Display, Formatter};

use connect_k::state::ConnectKState;
use general_minimax::{
    player::search::{ABSearch, EvalResult::Score, alphabeta, alphabeta_tt},
    result::GameResult,
    state::GameState,
    utils::position,
};

type Connect4 = ConnectKState<7, 6>;

#[test]
fn tt_finds_the_same_as_plain() {
    assert_tt_finds_the_same_as_plain::<16>();
}

/// 16 entries, overwritten constantly: lost entries may slow the search but not change it.
#[test]
fn tiny_tt_finds_the_same_as_plain() {
    assert_tt_finds_the_same_as_plain::<4>();
}

/// Within one Connect 4 search a position only comes back at the same remaining depth, so a
/// correct table of `1 << BITS` entries can't change what the search finds.
fn assert_tt_finds_the_same_as_plain<const BITS: u8>() {
    // Varied leaf values, so the table holds real bounds rather than all zeros.
    let eval = |s: &WithTable<BITS>| Score((s.hash() % 1000) as f32);
    let plain = alphabeta(eval);
    for seed in 0..50 {
        let mut state = WithTable::<BITS>(position(seed, seed as u32 % 20));
        let expected = plain.find_best(&mut state, 5, None);
        // A fresh table per seed, since one kept from an earlier seed can hold deeper results.
        let tt = alphabeta_tt(eval);
        assert_eq!(tt.find_best(&mut state, 5, None), expected, "seed {seed}");
    }
}

/// Connect 4 with a transposition table of `1 << BITS` entries.
#[derive(Clone, Default)]
struct WithTable<const BITS: u8>(Connect4);

impl<const BITS: u8> GameState for WithTable<BITS> {
    type Choice = <Connect4 as GameState>::Choice;
    type Moves = <Connect4 as GameState>::Moves;
    const NUM_P: u8 = Connect4::NUM_P;
    const TT_BITS: u8 = BITS;

    fn make_move(&mut self, choice: Self::Choice) {
        self.0.make_move(choice)
    }
    fn get_result(&self) -> Option<GameResult> {
        self.0.get_result()
    }
    fn candidate_moves(&self) -> Self::Moves {
        self.0.candidate_moves()
    }
    fn is_valid(&self, choice: Self::Choice) -> bool {
        self.0.is_valid(choice)
    }
    fn current_player(&self) -> u8 {
        self.0.current_player()
    }
    fn hash(&self) -> u64 {
        self.0.hash()
    }
    fn ply(&self) -> u32 {
        self.0.ply()
    }
    fn undo(&mut self) {
        self.0.undo()
    }
}

impl<const BITS: u8> Display for WithTable<BITS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
