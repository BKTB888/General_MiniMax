#![feature(generic_const_exprs)]
#![feature(decl_macro)]

use crate::evals::Evaluation;
use crate::evals::stupid_eval;
use crate::game::Game;
use crate::games::mancala::state::MancalaState;
use crate::players::human;
use crate::search::ABSearch;
use crate::search::Search;
use crate::search::alphabeta;
use std::time::Duration;

mod evals;
mod game;
mod games;
mod players;
mod result;
mod search;
mod state;

macro boxed {
    [$x:expr] => {
        std::array::from_fn(|_| Box::new($x) as Box<dyn Player<_>>)
    },
    [$($x:expr),* $(,)?] => {
        [$(Box::new($x)),*]
    },
}

fn main() {
    type Rules = MancalaState;
    let mut game = Game::<Rules>::new(boxed![
        human,
        alphabeta(stupid_eval).with_iterative(Duration::from_millis(5)),
    ]);
    game.print_stats(100);
}
