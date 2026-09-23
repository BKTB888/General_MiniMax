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
    #[arg(long, default_value = "human")]
    p1: PlayerKind,
    #[arg(long, default_value = "alphabeta")]
    p2: PlayerKind,
    /// Search depth for alphabeta players.
    #[arg(long, default_value_t = 4)]
    depth: u8,
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
            std::array::from_fn(|i| make_player(kinds[i], self.depth, eval, human));
        let mut game = Game::<S>::new(players);

        match self.mode {
            Mode::Play => game.print_play(),
            Mode::Stats { games, parallel } => game.print_stats(games, parallel),
        }
    }
}

fn make_player<S: GameState + 'static>(
    kind: PlayerKind,
    depth: u8,
    eval: impl Evaluation<S> + 'static,
    human: fn(&S) -> S::Choice,
) -> Box<dyn Player<S>> {
    match kind {
        PlayerKind::Human => Box::new(human),
        PlayerKind::Randy => Box::new(randy),
        PlayerKind::Alphabeta => Box::new(alphabeta(eval).to_player(depth)),
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum GameKind {
    Mancala,
    Connect4,
    FiveInRow,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum PlayerKind {
    Human,
    Randy,
    Alphabeta,
}

#[derive(Subcommand)]
pub enum Mode {
    /// Play one game, printing every move.
    Play,
    /// Play many games and print the result percentages.
    Stats {
        #[arg(long, default_value_t = 1000)]
        games: u32,
        #[arg(long)]
        parallel: bool,
    },
}
