use std::collections::BTreeMap;

use crate::{player::players::Player, result::GameResult, state::GameState};

pub struct Game<S: GameState>
where
    [(); S::NUM_P as usize]:,
{
    players: [Box<dyn Player<S>>; S::NUM_P as usize],
}

impl<S: GameState> Game<S>
where
    [(); S::NUM_P as usize]:,
{
    pub fn new(players: [Box<dyn Player<S>>; S::NUM_P as usize]) -> Self {
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
                    result.print_result();
                    return;
                }
            }
        }
    }

    pub fn stats(&mut self, num_games: u32, parallel: bool) -> BTreeMap<GameResult, u32> {
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
            (0..num_games)
                .map(|_| self.play())
                .fold(BTreeMap::new(), |mut acc, result| {
                    *acc.entry(result).or_default() += 1;
                    acc
                })
        }
    }

    pub fn print_stats(&mut self, num_games: u32, parallel: bool) {
        let now = std::time::Instant::now();

        let stats = self.stats(num_games, parallel);

        let elapsed = now.elapsed();

        stats.iter().for_each(|(result, &count)| {
            println!(
                "{result}: {:.2}%",
                (count as f64 / num_games as f64) * 100.0
            )
        });

        println!("Time / game: {:.2?}", elapsed / num_games);
        println!("Elapsed time: {:.2?}", elapsed);
    }
}
//TODO: Parallel does not work
pub macro play {
    ($rules:ty, $num_games:literal, true, $parallel:literal, $($player:expr,)* $(,)?) => {{
        let now = std::time::Instant::now();

        let stats = if $parallel {
            play!($rules, $num_games, true, $($player, )*)
        } else {
            play!($rules, $num_games, false, $($player, )*)
        };

        let elapsed = now.elapsed();

        stats.iter().for_each(|(result, &count)| {
            println!(
                "{result}: {:.2}%",
                (count as f64 / $num_games as f64) * 100.0
            )
        });

        println!("Time / game: {:.2?}", elapsed / $num_games);
        println!("Elapsed time: {:.2?}", elapsed);
    }},

    ($rules:ty, $num_games:literal, false, $parallel:literal, $($player:expr,)* $(,)?) => {{
        play!($rules, $num_games, $parallel, $($player, )*)
    }},


    ($rules:ty, $num_games:literal, true, $($player:expr,)* $(,)?) => {{
        (0..$num_games)
                .into_par_iter()
                .map(|_| play!($rules, $($player, )*))
                .fold(|| BTreeMap::new(), |mut acc, result| {
                    *acc.entry(result).or_default() += 1;
                    acc
                }).reduce(|| BTreeMap::new(), |mut a, b| {
                    for (result, v) in b {
                        *a.entry(result).or_default() += v;
                    }
                    a
                })
    }},

    ($rules:ty, $num_games:literal, false, $($player:expr,)* $(,)?) => {{
        (0..$num_games)
            .map(|_| play!($rules, $($player, )*))
            .fold(BTreeMap::<GameResult, u32>::new(), |mut acc, result| {
                *acc.entry(result).or_default() += 1;
                acc
            })
    }},

    ($rules:ty, $($player:expr,)* $(,)?) => {{
        //Todo: Should somehow count the players, so that there are no more then rules allow
        let mut state = <$rules>::default();

        loop {
            $(let choice = $player(&state);
            state.make_move(choice);

            if let Some(result) = state.get_result() {
                break result;
            })*
        }
    }},
}
