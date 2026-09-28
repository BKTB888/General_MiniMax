//! Plays two evaluations against each other from seeded random openings, each opening once from
//! either side, and prints the first one's score.
//!
//! `cargo run --release -p mega_tictactoe --example tournament -- <a> <b> <openings> <depth | ms>ms`

use std::time::Duration;

use general_minimax::{
    player::{
        evals::{Score, stupid_eval},
        search::{ABSearch, alphabeta_tt},
    },
    result::GameResult,
    state::GameState,
    utils::position,
};
use mega_tictactoe::state::FiveInRowState;
use rayon::prelude::*;

type S = FiveInRowState;
type Eval = fn(&S) -> Score;
type Player = Box<dyn FnMut(&S) -> <S as GameState>::Choice>;

/// Random stones before the contestants take over.
const OPENING_PLIES: u32 = 4;
/// Plies after which a game counts as a draw.
const MAX_PLIES: usize = 200;

fn contestant(name: &str) -> Eval {
    match name {
        "stupid" => stupid_eval,
        "eval" => mega_tictactoe::eval,
        _ => panic!("unknown evaluation `{name}`"),
    }
}

/// How much each move may search.
#[derive(Clone, Copy)]
enum Budget {
    Depth(u8),
    Time(Duration),
}
impl Budget {
    /// `<depth>` or `<ms>ms`.
    fn parse(arg: &str) -> Self {
        match arg.strip_suffix("ms") {
            Some(ms) => Self::Time(Duration::from_millis(ms.parse().unwrap())),
            None => Self::Depth(arg.parse().unwrap()),
        }
    }
    fn player(self, eval: Eval) -> Player {
        match self {
            Self::Depth(depth) => Box::new(alphabeta_tt(eval).to_player(depth)),
            Self::Time(time) => Box::new(alphabeta_tt(eval).with_iterative(time)),
        }
    }
}

/// The winning player, or `None` for a draw after `MAX_PLIES`. `players[p]` plays player `p`.
fn play(mut state: S, mut players: [Player; 2]) -> Option<u8> {
    for _ in 0..MAX_PLIES {
        let choice = players[state.current_player() as usize](&state);
        state.make_move(choice);
        if let Some(GameResult::Player(winner)) = state.get_result() {
            return Some(winner);
        }
    }
    None
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [a, b, openings, budget] = args.as_slice() else {
        panic!("usage: tournament <a> <b> <openings> <depth | ms>ms");
    };
    let (a, b) = (contestant(a), contestant(b));
    let openings: u64 = openings.parse().unwrap();
    let budget = Budget::parse(budget);

    // Per game, from `a`'s side: 1 a win, 0 a loss, and `None` a draw.
    let results: Vec<Option<u8>> = (0..openings)
        .into_par_iter()
        .flat_map_iter(|seed| {
            let start: S = position(seed, OPENING_PLIES);
            let first = play(start.clone(), [budget.player(a), budget.player(b)]);
            let second = play(start, [budget.player(b), budget.player(a)]);
            [first.map(|winner| 1 - winner), second]
        })
        .collect();

    let games = results.len() as f64;
    let wins = results.iter().filter(|&&r| r == Some(1)).count();
    let losses = results.iter().filter(|&&r| r == Some(0)).count();
    let draws = results.len() - wins - losses;
    let score = (wins as f64 + draws as f64 / 2.0) / games;
    // Two standard errors, ignoring that draws narrow it.
    let error = 2.0 * (score * (1.0 - score) / games).sqrt();
    eprintln!(
        "{} vs {}: +{wins} -{losses} ={draws}, score {:.1}% ± {:.1}",
        args[0],
        args[1],
        100.0 * score,
        100.0 * error
    );
}
