use clap::{CommandFactory, Parser, error::ErrorKind};
use connect_k::state::ConnectKState;
use five_in_row::{player::human_five_in_row, state::FiveInRowState};
use hive::state::HiveState;
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
use std::str::FromStr;

#[derive(Parser)]
pub struct CLI {
    /// mancala, connect4, five-in-row or hive. Add hive expansions as letters in any order:
    /// hive-m (mosquito), hive-lp (ladybug, pillbug), hive-mlp...
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

/// Plays the `HiveState` whose const generics are the given bools, in order:
/// `play_hive!(self, mosquito, ladybug, pillbug)`.
macro_rules! play_hive {
    ($self:ident, [$($c:literal),*], []) => {
        $self.play::<HiveState<$($c),*>>(hive::eval, hive::human)
    };
    // Const generics can't come from a runtime bool, so branch on it and move it to the
    // settled list as a literal.
    ($self:ident, [$($c:literal),*], [$b:expr $(, $rest:expr)*]) => {
        if $b {
            play_hive!($self, [$($c,)* true], [$($rest),*])
        } else {
            play_hive!($self, [$($c,)* false], [$($rest),*])
        }
    };
    // Last, because the recursive calls' lists are array expressions and would match it too.
    ($self:ident, $($b:expr),+) => {
        play_hive!($self, [], [$($b),+])
    };
}

impl CLI {
    /// Plays the chosen game with the chosen players.
    pub fn run(&self) {
        // Const generics are fixed at compile time, so each game maps to one concrete type.
        match self.game {
            GameKind::Mancala => self.play::<MancalaState>(mancala::eval, mancala::human),
            GameKind::Connect4 => self.play::<ConnectKState<7, 6>>(stupid_eval, human),
            GameKind::FiveInRow => {
                self.play::<FiveInRowState>(five_in_row::eval, human_five_in_row)
            }
            GameKind::Hive { mosquito, ladybug, pillbug } => {
                play_hive!(self, mosquito, ladybug, pillbug)
            }
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

#[derive(Clone, Copy)]
pub enum GameKind {
    Mancala,
    Connect4,
    FiveInRow,
    /// The base pieces, plus each expansion piece that is true.
    Hive {
        mosquito: bool,
        ladybug: bool,
        pillbug: bool,
    },
}

impl GameKind {
    /// Hive with the expansions named by `letters`: any of `m`, `l`, `p`, each at most once.
    fn hive(letters: &str) -> Result<Self, String> {
        let (mut mosquito, mut ladybug, mut pillbug) = (false, false, false);
        for letter in letters.chars() {
            let expansion = match letter {
                'm' => &mut mosquito,
                'l' => &mut ladybug,
                'p' => &mut pillbug,
                _ => return Err(format!("unknown hive expansion `{letter}`, expected m, l or p")),
            };
            if *expansion {
                return Err(format!("hive expansion `{letter}` given twice"));
            }
            *expansion = true;
        }
        Ok(GameKind::Hive { mosquito, ladybug, pillbug })
    }
}

impl FromStr for GameKind {
    type Err = String;

    /// `mancala`, `connect4`, `five-in-row`, or `hive` followed by `-` and any of `m`, `l`, `p`.
    fn from_str(name: &str) -> Result<Self, String> {
        Ok(match name {
            "mancala" => GameKind::Mancala,
            "connect4" => GameKind::Connect4,
            "five-in-row" => GameKind::FiveInRow,
            "hive" => GameKind::hive("")?,
            _ => match name.strip_prefix("hive-") {
                Some(letters) if !letters.is_empty() => GameKind::hive(letters)?,
                _ => return Err(format!("unknown game `{name}`")),
            },
        })
    }
}
