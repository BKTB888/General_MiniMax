#![feature(
    gca_const_items,
    gca_min_const_items,
    gca_macroless_args
)]
#![allow(incomplete_features)]

use std::time::Instant;

use connect_k::state::ConnectKState;
use general_minimax::{
    player::{
        evals::{Score, stupid_eval},
        search::{ABSearch, alphabeta, alphabeta_tt, alphabeta_tt_sized},
    },
    state::GameState,
    utils::position,
};

type Connect4 = ConnectKState<7, 6>;

#[test]
fn tt_finds_the_same_as_plain() {
    assert_tt_finds_the_same_as_plain(Connect4::TT_BITS);
}

/// 16 entries, overwritten constantly: lost entries may slow the search but not change it.
#[test]
fn tiny_tt_finds_the_same_as_plain() {
    assert_tt_finds_the_same_as_plain(4);
}

/// Within one Connect 4 search a position only comes back at the same remaining depth, so a
/// correct table of `1 << bits` entries can't change what the search finds.
fn assert_tt_finds_the_same_as_plain(bits: u8) {
    // Varied leaf values, so the table holds real bounds rather than all zeros.
    let eval = |s: &Connect4| (s.hash() % 1000) as Score;
    let plain = alphabeta(eval);
    for seed in 0..50 {
        let mut state: Connect4 = position(seed, seed as u32 % 20);
        let expected = plain.find_best(&mut state, 5, None);
        // A fresh table per seed, since one kept from an earlier seed can hold deeper results.
        let tt = alphabeta_tt_sized(eval, bits);
        assert_eq!(tt.find_best(&mut state, 5, None), expected, "seed {seed}");
    }
}

#[test]
fn passed_deadline_aborts_and_leaves_the_state() {
    let mut state: Connect4 = position(1, 8);
    let hash = state.hash();
    let tt = alphabeta_tt(stupid_eval);
    assert_eq!(tt.find_best_until(&mut state, 5, None, Some(Instant::now())), None);
    assert_eq!(state.hash(), hash);
}
