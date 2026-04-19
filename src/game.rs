use crate::players::Player;
use crate::result::GameResult;
use crate::state::GameState;
use std::collections::BTreeMap;

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

    pub fn stats(&mut self, num_of_games: u32) -> BTreeMap<GameResult, u32> {
        (0..num_of_games)
            .map(|_| self.play())
            .fold(BTreeMap::new(), |mut acc, result| {
                *acc.entry(result).or_default() += 1;
                acc
            })
    }

    pub fn print_stats(&mut self, num_of_games: u32) {
        let now = std::time::Instant::now();

        let stats = self.stats(num_of_games);

        let elapsed = now.elapsed();

        stats.iter().for_each(|(result, &count)| {
            println!(
                "{result}: {:.2}%",
                (count as f64 / num_of_games as f64) * 100.0
            )
        });

        println!("Time / game: {:.2?}", elapsed / num_of_games);
        println!("Elapsed time: {:.2?}", elapsed);
    }
}

macro play {
    ($($key:ident : $value:expr),* $(,)?) => {{
        (
            $($key, $value),*
        )
    }}
}
