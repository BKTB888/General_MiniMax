//! Plays two evaluations against each other from seeded random openings, each opening once from
//! either side, and prints the first one's score.
//!
//! `cargo run --release -p mega_tictactoe --example tournament -- <a> <b> <openings> <depth | ms>ms`

use std::{sync::Arc, time::Duration};

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
type Eval = Arc<dyn Fn(&S) -> Score + Send + Sync>;
type Player = Box<dyn FnMut(&S) -> <S as GameState>::Choice>;

/// Random stones before the contestants take over.
const OPENING_PLIES: u32 = 4;
/// Plies after which a game counts as a draw.
const MAX_PLIES: usize = 120;

/// `stupid`, `eval`, `sym`, or `w:<own>/<opp>`: the player to move's windows holding 1, 2, 3
/// and 4 stones weighed by `own`, less the other player's by `opp`, four weights each, e.g.
/// `w:1,10,100,1000/1,10,100,1000`.
fn contestant(name: &str) -> Eval {
    match name {
        "stupid" => Arc::new(stupid_eval),
        "eval" => Arc::new(mega_tictactoe::eval),
        "sym" => contestant("w:1,10,100,1000/1,10,100,1000"),
        _ => {
            let weights = |list: &str| -> [Score; 4] {
                let weights: Vec<Score> = list.split(',').map(|w| w.parse().unwrap()).collect();
                weights.try_into().expect("four weights")
            };
            let (own, opp) = name
                .strip_prefix("w:")
                .and_then(|w| w.split_once('/'))
                .unwrap_or_else(|| panic!("unknown evaluation `{name}`"));
            let (own, opp) = (weights(own), weights(opp));
            Arc::new(move |state| weigh(state, own, opp))
        }
    }
}

/// The player to move's windows weighed by `own`, less the other player's weighed by `opp`.
fn weigh(state: &S, own: [Score; 4], opp: [Score; 4]) -> Score {
    let mover = state.current_player();
    let (mine, theirs) = (state.windows(mover), state.windows(mover ^ 1));
    (0..4)
        .map(|n| own[n] * mine[n] as Score - opp[n] * theirs[n] as Score)
        .sum()
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
    fn player(self, eval: &Eval) -> Player {
        let eval = eval.clone();
        let eval = move |state: &S| eval(state);
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

    // Per opening, `a`'s points over its two games: 2 a win and 1 a draw.
    let pairs: Vec<u8> = (0..openings)
        .into_par_iter()
        .map(|seed| {
            let start: S = position(seed, OPENING_PLIES);
            let first = play(start.clone(), [budget.player(&a), budget.player(&b)]);
            let second = play(start, [budget.player(&b), budget.player(&a)]);
            let points = |result: Option<u8>, a_plays: u8| match result {
                Some(winner) => 2 * (winner == a_plays) as u8,
                None => 1,
            };
            points(first, 0) + points(second, 1)
        })
        .collect();

    let count = |points: fn(u8) -> bool| pairs.iter().filter(|&&p| points(p)).count();
    let (won, split, lost) = (count(|p| p > 2), count(|p| p == 2), count(|p| p < 2));
    // The two games of an opening depend on each other, so the error comes from the pairs.
    let n = pairs.len() as f64;
    let scores: Vec<f64> = pairs.iter().map(|&p| p as f64 / 4.0).collect();
    let score = scores.iter().sum::<f64>() / n;
    let variance = scores.iter().map(|s| (s - score).powi(2)).sum::<f64>() / (n - 1.0);
    let error = 2.0 * (variance / n).sqrt();
    eprintln!(
        "{} vs {}: pairs +{won} ={split} -{lost}, score {:.1}% ± {:.1}",
        args[0],
        args[1],
        100.0 * score,
        100.0 * error
    );
}
