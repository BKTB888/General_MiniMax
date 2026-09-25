use std::{fmt::Display, str::FromStr, time::Duration};

use rand::rng;

use crate::{
    player::{
        evals::Evaluation,
        players::{Player, PlayerCreator, creator_from_seed, randy},
        search::{ABSearch, Search, alphabeta, alphabeta_tt, minimax},
    },
    state::GameState,
};

#[derive(Clone, Copy)]
pub enum PlayerKind {
    Human,
    /// Random moves, reproducible when seeded.
    Randy(Option<u64>),
    /// Full search without pruning, spread over all cores.
    Minimax(u8),
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
            "minimax" => Ok(PlayerKind::Minimax(number(name, arg)?)),
            "alphabeta" => Ok(PlayerKind::Alphabeta(number(name, arg)?)),
            "alphabeta-tt" => Ok(PlayerKind::AlphabetaTT(number(name, arg)?)),
            "iterative" => Ok(PlayerKind::Iterative(number(name, arg)?)),
            "iterative-tt" => Ok(PlayerKind::IterativeTT(number(name, arg)?)),
            _ => Err(format!(
                "unknown player `{name}`, expected human, randy[:<seed>], minimax:<depth>, \
                 alphabeta:<depth>, alphabeta-tt:<depth>, iterative:<ms> or iterative-tt:<ms>"
            )),
        }
    }
}

impl PlayerKind {
    /// Returns a creator that makes this kind of player for a given game index. `eval` and `human`
    /// are the game's own.
    pub fn to_player_creator<S: GameState + 'static>(
        self,
        eval: impl Evaluation<S> + Sync + Copy + 'static,
        human: fn(&S) -> S::Choice,
    ) -> Box<dyn PlayerCreator<S>> {
        match self {
            PlayerKind::Human => every_game(move || human),
            PlayerKind::Randy(None) => every_game(|| randy(rng())),
            PlayerKind::Randy(Some(seed)) => Box::new(creator_from_seed(seed, randy)),
            PlayerKind::Minimax(depth) => every_game(move || minimax(eval).to_player(depth)),
            PlayerKind::Alphabeta(depth) => every_game(move || alphabeta(eval).to_player(depth)),
            PlayerKind::AlphabetaTT(depth) => {
                every_game(move || alphabeta_tt(eval).to_player(depth))
            }
            PlayerKind::Iterative(ms) => {
                every_game(move || alphabeta(eval).with_iterative(Duration::from_millis(ms)))
            }
            PlayerKind::IterativeTT(ms) => {
                every_game(move || alphabeta_tt(eval).with_iterative(Duration::from_millis(ms)))
            }
        }
    }
}

/// A creator that ignores the game index and makes each game's player with `make`.
fn every_game<S: GameState, P: Player<S> + 'static>(
    make: impl Fn() -> P + Sync + 'static,
) -> Box<dyn PlayerCreator<S>> {
    Box::new(move |_| Box::new(make()) as Box<dyn Player<S>>)
}

/// Parses the number after `name:`, with errors naming the player.
fn number<T: FromStr<Err: Display>>(name: &str, arg: Option<&str>) -> Result<T, String> {
    arg.ok_or(format!("`{name}` needs a number, e.g. `{name}:4`"))?
        .parse()
        .map_err(|e| format!("`{name}`: {e}"))
}
