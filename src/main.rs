#![feature(generic_const_exprs)]
#![feature(trait_alias)]
#![feature(decl_macro)]

use crate::evals::Evaluation;
use crate::evals::stupid_eval;
use crate::game::Game;
use crate::games::connect4::state::ConnectKState;
use crate::players::randys_from_seed;
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
    let mut game = Game::<Rules>::new(boxed![
        alphabeta(stupid_eval).to_eval(4).to_player(),
        randys_from_seed(42)
    ]);
    game.print_stats(10_000);

    /*
    game = Game::<Rules>::new(boxed![
        randys_from_seed(42),
        ai_from_eval(
            minimax_creator(
                4,
                mancala::eval
            )
        )
    ]);

    game.print_stats(10_000);

     */
}
