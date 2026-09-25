#![feature(
    generic_const_args,
    min_generic_const_args,
    macroless_generic_const_args
)]
#![allow(incomplete_features)]

use connect_k::state::ConnectKState;
use general_minimax::{
    player::search::{ABSearch, EvalResult::Score, alphabeta, alphabeta_tt},
    state::GameState,
    utils::position,
};

type Connect4 = ConnectKState<7, 6>;

/// In Connect 4 a position only comes back at the same remaining depth and the move order is
/// fixed, so a correct table can't change what the search finds.
#[test]
fn tt_finds_the_same_as_plain() {
    // Varied leaf values, so the table holds real bounds rather than all zeros.
    let eval = |s: &Connect4| Score((s.hash() % 1000) as f32);
    let (plain, tt) = (alphabeta(eval), alphabeta_tt(eval));
    for seed in 0..50 {
        let mut state: Connect4 = position(seed, seed as u32 % 20);
        let expected = plain.find_best(&mut state, 5);
        assert_eq!(tt.find_best(&mut state, 5), expected, "seed {seed}");
    }
}
