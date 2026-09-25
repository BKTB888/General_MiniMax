use clap::{CommandFactory, Parser, ValueEnum, error::ErrorKind};
use connect_k::state::ConnectKState;
use general_minimax::{
    game::play_multiple,
    player::{
        evals::{Evaluation, stupid_eval},
        player_kind::PlayerKind,
        players::human,
    },
    state::GameState,
};
use mancala::state::MancalaState;
use mega_tictactoe::{evaluation::eval_kinrow, player::human_kinrow, state::KInARowState};

#[derive(Parser)]
pub struct CLI {
    game: GameKind,
    /// human, randy[:<seed>], minimax:<depth>, alphabeta:<depth>, alphabeta-tt:<depth>, iterative:<ms per move> or iterative-tt:<ms per move>
    #[arg(long, default_value = "human")]
    p1: PlayerKind,
    /// Same values as `--p1`.
    #[arg(long, default_value = "alphabeta:4")]
    p2: PlayerKind,
    #[arg(long, default_value_t = 1)]
    games: u32,
    #[arg(long, conflicts_with_all = ["print_board", "print_result"])]
    parallel: bool,
    /// Print the board after every move. Defaults to false.
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true")]
    print_board: Option<bool>,
    /// Print the final board and result of every game. Defaults to true if a human plays.
    #[arg(long, num_args = 0..=1, require_equals = true, default_missing_value = "true")]
    print_result: Option<bool>,
}

impl CLI {
    /// Plays the chosen game with the chosen players.
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
        eval: impl Evaluation<S> + Sync + Copy + 'static,
        human: fn(&S) -> S::Choice,
    ) {
        let kinds = [self.p1, self.p2];
        let human_plays = kinds.iter().any(|kind| matches!(kind, PlayerKind::Human));
        if self.parallel && human_plays {
            CLI::command()
                .error(
                    ErrorKind::ArgumentConflict,
                    "--parallel can't be used with a human player",
                )
                .exit();
        }

        play_multiple(
            &S::default(),
            // `from_fn` takes its length from `NUM_P`, which generic code can't match to `kinds`'.
            std::array::from_fn(|i| kinds[i].to_player_creator(eval, human)),
            self.games,
            self.parallel,
            self.print_board.unwrap_or(false),
            self.print_result.unwrap_or(human_plays),
        )
        .print();
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum GameKind {
    Mancala,
    Connect4,
    FiveInRow,
}
