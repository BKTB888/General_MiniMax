use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use rayon::{iter::ParallelIterator, prelude::IntoParallelIterator};

use crate::{
    AS_USIZE,
    player::players::{Player, PlayerCreator},
    result::GameResult,
    state::GameState,
};

/// One player per seat, in turn order.
pub type Players<S> = [Box<dyn Player<S>>; AS_USIZE::<{ <S as GameState>::NUM_P }>];

/// One player creator per seat, in turn order.
pub type PlayerCreators<S> = [Box<dyn PlayerCreator<S>>; AS_USIZE::<{ <S as GameState>::NUM_P }>];

/// Plays one game from `state`, printing every board if `PRINT_BOARD` and the result if
/// `PRINT_RESULT`.
pub fn play<S: GameState, const PRINT_BOARD: bool, const PRINT_RESULT: bool>(
    mut state: S,
    players: &mut Players<S>,
) -> GameResult {
    if PRINT_BOARD {
        println!("Start:\n{state}");
    }
    let mut i = 1;
    loop {
        for player in players.iter_mut() {
            let choice = player(&state);
            state.make_move(choice);
            if PRINT_BOARD {
                println!("{i}:\n{state}");
                i += 1;
            }

            if let Some(result) = state.get_result() {
                if PRINT_RESULT {
                    // With `PRINT_BOARD` the final board was just printed above.
                    if !PRINT_BOARD {
                        println!("{state}");
                    }
                    result.print();
                }
                return result;
            }
        }
    }
}

/// Plays `num_games` games from `start`, each with fresh players made by `creators` from the game
/// index, printing every board if `print_board` and each game's result if `print_result`.
pub fn play_multiple<S: GameState>(
    start: &S,
    creators: PlayerCreators<S>,
    num_games: u32,
    parallel: bool,
    print_board: bool,
    print_result: bool,
) -> Stats {
    let play: fn(S, &mut Players<S>) -> GameResult = match (print_board, print_result) {
        (false, false) => play::<S, false, false>,
        (false, true) => play::<S, false, true>,
        (true, false) => play::<S, true, false>,
        (true, true) => play::<S, true, true>,
    };
    // Players are made and used inside one thread, so they never need to be `Send`.
    let play_game = |game| {
        let mut players = creators.each_ref().map(|creator| creator(game));
        play(start.clone(), &mut players)
    };

    let now = Instant::now();
    let results = if parallel {
        (0..num_games)
            .into_par_iter()
            .map(play_game)
            .fold(BTreeMap::new, count)
            .reduce(BTreeMap::new, |mut counts, other| {
                for (result, n) in other {
                    *counts.entry(result).or_default() += n;
                }
                counts
            })
    } else {
        (0..num_games).map(play_game).fold(BTreeMap::new(), count)
    };

    Stats::new(results, now.elapsed(), num_games)
}

fn count(mut counts: BTreeMap<GameResult, u32>, result: GameResult) -> BTreeMap<GameResult, u32> {
    *counts.entry(result).or_default() += 1;
    counts
}

pub struct Stats {
    results: BTreeMap<GameResult, u32>,
    elapsed: Duration,
    num_games: u32,
}

impl Stats {
    fn new(results: BTreeMap<GameResult, u32>, elapsed: Duration, num_games: u32) -> Self {
        Self {
            results,
            elapsed,
            num_games,
        }
    }

    pub fn print(&self) {
        for (result, &count) in &self.results {
            println!(
                "{result}: {:.2}%",
                (count as f64 / self.num_games as f64) * 100.0
            )
        }

        println!("Time / game: {:.2?}", self.elapsed / self.num_games);
        println!("Elapsed time: {:.2?}", self.elapsed);
    }
}
