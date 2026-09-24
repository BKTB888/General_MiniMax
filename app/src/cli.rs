use std::{fmt::Display, str::FromStr, time::Duration};

use clap::{Parser, Subcommand, ValueEnum};
use connect_k::state::ConnectKState;
use general_minimax::{
    AS_USIZE,
    game::Game,
    player::{
        evals::{Evaluation, stupid_eval},
        players::{Player, human, randy},
        search::{ABSearch, alphabeta},
    },
    state::GameState,
};
use mancala::state::MancalaState;
use mega_tictactoe::{evaluation::eval_kinrow, player::human_kinrow, state::KInARowState};

#[derive(Parser)]
pub struct CLI {
    game: GameKind,
    /// human, randy, alphabeta:<depth> or iterative:<ms per move>
    #[arg(long, default_value = "human")]
    p1: PlayerKind,
    /// Same values as `--p1`.
    #[arg(long, default_value = "alphabeta:4")]
    p2: PlayerKind,
    #[command(subcommand)]
    mode: Mode,
}

impl CLI {
    /// Plays the chosen game with the chosen players and mode.
    pub fn run(&self) {
        // Const generics are fixed at compile time, so each game maps to one concrete type.
        match self.game {
            GameKind::Mancala => self.play::<MancalaState>(mancala::eval, mancala::human),
            GameKind::Connect4 => self.play::<ConnectKState<7, 6>>(stupid_eval, human),
            GameKind::FiveInRow => self.play::<KInARowState<5>>(eval_kinrow, human_kinrow),
        }
    }

    /// `eval` and `human` are the game's own.
    fn play<S: GameState + 'static>(
        &self,
        eval: impl Evaluation<S> + Copy + 'static,
        human: fn(&S) -> S::Choice,
    ) {
        let kinds = [self.p1, self.p2];
        let players: [Box<dyn Player<S>>; AS_USIZE::<{ S::NUM_P }>] =
            std::array::from_fn(|i| make_player(kinds[i], eval, human));
        let mut game = Game::<S>::new(players);

        match self.mode {
            Mode::Play => game.play().print_result(),
            Mode::Stats { games, parallel } => game.print_stats(games, parallel),
        }
    }
}

fn make_player<S: GameState + 'static>(
    kind: PlayerKind,
    eval: impl Evaluation<S> + 'static,
    human: fn(&S) -> S::Choice,
) -> Box<dyn Player<S>> {
    match kind {
        PlayerKind::Human => Box::new(human),
        PlayerKind::Randy => Box::new(randy),
        PlayerKind::Alphabeta(depth) => Box::new(alphabeta(eval).to_player(depth)),
        PlayerKind::Iterative(ms) => {
            Box::new(alphabeta(eval).with_iterative(Duration::from_millis(ms)))
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum GameKind {
    Mancala,
    Connect4,
    FiveInRow,
}

#[derive(Clone, Copy)]
pub enum PlayerKind {
    Human,
    Randy,
    Alphabeta(u8),
    /// Iterative deepening with a time budget in ms per move.
    Iterative(u64),
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
            "randy" => Ok(PlayerKind::Randy),
            "alphabeta" => Ok(PlayerKind::Alphabeta(number(name, arg)?)),
            "iterative" => Ok(PlayerKind::Iterative(number(name, arg)?)),
            _ => Err(format!(
                "unknown player `{name}`, expected human, randy, alphabeta:<depth> or iterative:<ms>"
            )),
        }
    }
}

/// Parses the number after `name:`, with errors naming the player.
fn number<T: FromStr<Err: Display>>(name: &str, arg: Option<&str>) -> Result<T, String> {
    arg.ok_or(format!("`{name}` needs a number, e.g. `{name}:4`"))?
        .parse()
        .map_err(|e| format!("`{name}`: {e}"))
}

#[derive(Subcommand)]
pub enum Mode {
    /// Play one game, printing every move.
    Play,
    /// Play many games and print the result percentages.
    Stats {
        #[arg(long, default_value_t = 1000)]
        games: u32,
        #[arg(long, default_value = "false")]
        parallel: bool,
    },
}
