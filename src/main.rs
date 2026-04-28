#![feature(generic_const_exprs)]
#![feature(decl_macro)]

use crate::evals::Evaluation;
use crate::evals::stupid_eval;
use crate::game::Game;
use crate::games::connect4::state::ConnectKState;
use crate::players::randys_from_seed;
use crate::search::ABSearch;
use crate::search::Search;
use crate::search::alphabeta;

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
    type Rules = ConnectKState<6, 7>;
    let p1 = randys_from_seed(42);
    let p2 = alphabeta(stupid_eval).to_player(5);

    let mut game = Game::<Rules>::new(boxed![p1, p2,]);
    game.print_stats(1000, false);

    /*
    play!(
        Rules,
        10_000,
        true,
        false,
        p1,
        p2,
    );

     */
}
