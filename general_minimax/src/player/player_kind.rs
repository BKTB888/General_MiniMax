use std::{fmt::Display, str::FromStr, time::Duration};

use rand::rng;

use crate::{
    player::{
        evals::Evaluation,
        players::{Player, PlayerCreator, creator_from_seed, randy},
        search::{ABSearch, alphabeta, alphabeta_tt},
    },
    state::GameState,
};

#[derive(Clone, Copy)]
pub enum PlayerKind {
    Human,
    /// Random moves, reproducible when seeded.
    Randy(Option<u64>),
    Alphabeta(u8),
    /// Alphabeta with a transposition table.
    AlphabetaTT(u8),
    /// Iterative deepening with a time budget in ms per move.
    Iterative(u64),
    /// Iterative deepening with a transposition table.
    IterativeTT(u64),
}

impl FromStr for PlayerKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, arg) = match s.split_once(':') {
            Some((name, arg)) => (name, Some(arg)),
            None => (s, None),
        };
        match name {
            "human" => Ok(PlayerKind::Human),
            "randy" => Ok(PlayerKind::Randy(
                arg.map(|_| number(name, arg)).transpose()?,
            )),
            "alphabeta" => Ok(PlayerKind::Alphabeta(number(name, arg)?)),
            "alphabeta-tt" => Ok(PlayerKind::AlphabetaTT(number(name, arg)?)),
            "iterative" => Ok(PlayerKind::Iterative(number(name, arg)?)),
            "iterative-tt" => Ok(PlayerKind::IterativeTT(number(name, arg)?)),
            _ => Err(format!(
                "unknown player `{name}`, expected human, randy[:<seed>], alphabeta:<depth>, \
                 alphabeta-tt:<depth>, iterative:<ms> or iterative-tt:<ms>"
            )),
        }
    }
}

impl PlayerKind {
    /// Returns a creator that makes this kind of player for a given game index. `eval` and `human`
    /// are the game's own.
    pub fn to_player_creator<S: GameState + 'static>(
        self,
        eval: impl Evaluation<S> + Copy + 'static,
        human: fn(&S) -> S::Choice,
    ) -> Box<dyn PlayerCreator<S>> {
        // The casts make every closure return exactly `Box<dyn Player<S>>`, as `PlayerCreator` needs.
        match self {
            PlayerKind::Human => Box::new(move |_: u32| Box::new(human) as Box<dyn Player<S>>),
            PlayerKind::Randy(None) => {
                Box::new(|_: u32| Box::new(randy(rng())) as Box<dyn Player<S>>)
            }
            PlayerKind::Randy(Some(seed)) => Box::new(creator_from_seed(seed, randy)),
            PlayerKind::Alphabeta(depth) => Box::new(move |_: u32| {
                Box::new(alphabeta(eval).to_player(depth)) as Box<dyn Player<S>>
            }),
            PlayerKind::AlphabetaTT(depth) => Box::new(move |_: u32| {
                Box::new(alphabeta_tt(eval).to_player(depth)) as Box<dyn Player<S>>
            }),
            PlayerKind::Iterative(ms) => Box::new(move |_: u32| {
                Box::new(alphabeta(eval).with_iterative(Duration::from_millis(ms)))
                    as Box<dyn Player<S>>
            }),
            PlayerKind::IterativeTT(ms) => Box::new(move |_: u32| {
                Box::new(alphabeta_tt(eval).with_iterative(Duration::from_millis(ms)))
                    as Box<dyn Player<S>>
            }),
        }
    }
}

/// Parses the number after `name:`, with errors naming the player.
fn number<T: FromStr<Err: Display>>(name: &str, arg: Option<&str>) -> Result<T, String> {
    arg.ok_or(format!("`{name}` needs a number, e.g. `{name}:4`"))?
        .parse()
        .map_err(|e| format!("`{name}`: {e}"))
}
