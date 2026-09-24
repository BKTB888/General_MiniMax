use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use crate::{AS_USIZE, player::players::Player, result::GameResult, state::GameState};

pub struct Game<S: GameState> {
    players: [Box<dyn Player<S>>; AS_USIZE::<{ S::NUM_P }>],
}

impl<S: GameState> Game<S> {
    pub fn new(players: [Box<dyn Player<S>>; AS_USIZE::<{ S::NUM_P }>]) -> Self {
        Game { players }
    }

    pub fn play(&mut self) -> GameResult {
        let mut state = S::default();
        loop {
            for player in self.players.iter_mut() {
                let choice = player(&state);
                state.make_move(choice);

                if let Some(result) = state.get_result() {
                    return result;
                }
            }
        }
    }

    pub fn print_play(&mut self) {
        let mut state = S::default();
        println!("Start:\n{state}");
        let mut i = 1;
        loop {
            for player in self.players.iter_mut() {
                let choice = player(&state);
                state.make_move(choice);
                println!("{i}:\n{state}");
                i += 1;

                if let Some(result) = state.get_result() {
                    result.print();
                    return;
                }
            }
        }
    }

    pub fn stats(&mut self, num_games: u32, parallel: bool) -> Stats {
        if parallel {
            todo!();
            /*
            (0..num_games)
                .into_par_iter()
                .map(|_| self.play())
                .fold(|| BTreeMap::new(), |mut acc, result| {
                    *acc.entry(result).or_default() += 1;
                    acc
                }).reduce(|| BTreeMap::new(), |mut a, b| {
                    for (result, v) in b {
                        *a.entry(result).or_default() += v;
                    }
                    a
                })

             */
        } else {
            let now = Instant::now();
            let results = (0..num_games)
                .map(|_| self.play())
                .fold(BTreeMap::new(), |mut acc, result| {
                    *acc.entry(result).or_default() += 1;
                    acc
                });
            let elapsed = now.elapsed();

            Stats::new(results, elapsed, num_games)
        }
    }
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
